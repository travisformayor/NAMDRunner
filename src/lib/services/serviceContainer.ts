import type { 
  ConnectionStateManager,
  SessionManager,
  SSHConnection,
  SFTPConnection 
} from '../types/connection';
import type { RemoteDirectoryManager } from './directoryManager';
import type { ConnectionValidator } from './connectionValidator';
import { ConnectionStateMachine } from './connectionState';
import { SSHSessionManager } from './sessionManager';
import { SLURMDirectoryManager } from './directoryManager';
import { SLURMConnectionValidator } from './connectionValidator';
import { PathResolver, createPathResolver } from './pathResolver';

/**
 * Service container for dependency injection
 * Provides a clean way to manage service dependencies and testing
 */

export interface ServiceDependencies {
  stateManager: ConnectionStateManager;
  sessionManager: SessionManager;
  directoryManager: RemoteDirectoryManager;
  connectionValidator: ConnectionValidator;
  pathResolver: PathResolver;
  sshConnection?: SSHConnection;
  sftpConnection?: SFTPConnection;
}

export class ServiceContainer {
  private services: Map<string, any> = new Map();
  private singletons: Map<string, any> = new Map();

  /**
   * Register a service factory
   */
  register<T>(name: string, factory: () => T, singleton: boolean = true): void {
    if (singleton) {
      this.services.set(name, () => {
        if (!this.singletons.has(name)) {
          this.singletons.set(name, factory());
        }
        return this.singletons.get(name);
      });
    } else {
      this.services.set(name, factory);
    }
  }

  /**
   * Get a service by name
   */
  get<T>(name: string): T {
    const factory = this.services.get(name);
    if (!factory) {
      throw new Error(`Service '${name}' not registered`);
    }
    return factory();
  }

  /**
   * Check if a service is registered
   */
  has(name: string): boolean {
    return this.services.has(name);
  }

  /**
   * Clear all services (useful for testing)
   */
  clear(): void {
    this.services.clear();
    this.singletons.clear();
  }

  /**
   * Create dependencies bundle for easy injection
   */
  createDependencies(): ServiceDependencies {
    return {
      stateManager: this.get<ConnectionStateManager>('stateManager'),
      sessionManager: this.get<SessionManager>('sessionManager'),
      directoryManager: this.get<RemoteDirectoryManager>('directoryManager'),
      connectionValidator: this.get<ConnectionValidator>('connectionValidator'),
      pathResolver: this.get<PathResolver>('pathResolver'),
      sshConnection: this.has('sshConnection') ? this.get<SSHConnection>('sshConnection') : undefined,
      sftpConnection: this.has('sftpConnection') ? this.get<SFTPConnection>('sftpConnection') : undefined,
    };
  }
}

/**
 * Default service container with standard implementations
 */
export function createServiceContainer(): ServiceContainer {
  const container = new ServiceContainer();

  // Register core services
  container.register('stateManager', () => new ConnectionStateMachine());
  container.register('sessionManager', () => new SSHSessionManager());
  container.register('pathResolver', () => createPathResolver());
  
  // Directory manager depends on SSH/SFTP connections
  container.register('directoryManager', () => {
    const ssh = container.has('sshConnection') ? container.get<SSHConnection>('sshConnection') : null;
    const sftp = container.has('sftpConnection') ? container.get<SFTPConnection>('sftpConnection') : null;
    const pathResolver = container.get<PathResolver>('pathResolver');
    
    if (!ssh || !sftp) {
      throw new Error('SSH and SFTP connections required for directory manager');
    }
    
    return new SLURMDirectoryManager(ssh, sftp, pathResolver);
  });
  
  // Connection validator
  container.register('connectionValidator', () => new SLURMConnectionValidator());

  return container;
}

/**
 * Mock service container for testing
 */
export function createMockServiceContainer(): ServiceContainer {
  const container = new ServiceContainer();

  // Register mock implementations
  container.register('stateManager', () => new ConnectionStateMachine());
  container.register('sessionManager', () => new SSHSessionManager());
  container.register('pathResolver', () => createPathResolver());
  
  // Mock connections
  container.register('sshConnection', () => ({
    connect: async () => ({ success: true, data: {} }),
    disconnect: async () => ({ success: true, data: undefined }),
    executeCommand: async () => ({ success: true, data: { stdout: 'mock', stderr: '', exitCode: 0, duration: 100, timedOut: false } }),
    validateConnection: async () => ({ success: true, data: true }),
    getStatus: () => 'Connected',
    isConnected: () => true,
  }));

  container.register('sftpConnection', () => ({
    uploadFile: async () => ({ success: true, data: {} }),
    downloadFile: async () => ({ success: true, data: {} }),
    listFiles: async () => ({ success: true, data: { files: [], totalCount: 0, path: '/' } }),
    createDirectory: async () => ({ success: true, data: { path: '/', created: true, existed: false } }),
    deleteFile: async () => ({ success: true, data: {} }),
    exists: async () => ({ success: true, data: true }),
    getFileInfo: async () => ({ success: true, data: { name: 'mock', path: '/', size: 0, modifiedAt: '', permissions: '', isDirectory: false } }),
  }));

  // Directory manager with mock connections
  container.register('directoryManager', () => {
    const ssh = container.get<SSHConnection>('sshConnection');
    const sftp = container.get<SFTPConnection>('sftpConnection');
    const pathResolver = container.get<PathResolver>('pathResolver');
    return new SLURMDirectoryManager(ssh, sftp, pathResolver);
  });
  
  // Connection validator
  container.register('connectionValidator', () => new SLURMConnectionValidator());

  return container;
}

/**
 * Global service container instance
 * Can be replaced for testing
 */
let globalContainer: ServiceContainer | null = null;

/**
 * Get the global service container
 */
export function getServiceContainer(): ServiceContainer {
  if (!globalContainer) {
    globalContainer = createServiceContainer();
  }
  return globalContainer;
}

/**
 * Set the global service container (useful for testing)
 */
export function setServiceContainer(container: ServiceContainer): void {
  globalContainer = container;
}

/**
 * Reset to default service container
 */
export function resetServiceContainer(): void {
  globalContainer = null;
}

/**
 * Service decorators for easy dependency injection
 */
export function withDependencies<T extends any[], R>(
  fn: (deps: ServiceDependencies, ...args: T) => R
) {
  return (...args: T): R => {
    const container = getServiceContainer();
    const deps = container.createDependencies();
    return fn(deps, ...args);
  };
}

/**
 * Async version of withDependencies
 */
export function withDependenciesAsync<T extends any[], R>(
  fn: (deps: ServiceDependencies, ...args: T) => Promise<R>
) {
  return async (...args: T): Promise<R> => {
    const container = getServiceContainer();
    const deps = container.createDependencies();
    return fn(deps, ...args);
  };
}


/**
 * Register connections with the global service container
 */
export function registerConnections(ssh: SSHConnection, sftp: SFTPConnection): void {
  const container = getServiceContainer();
  container.register('sshConnection', () => ssh, true);
  container.register('sftpConnection', () => sftp, true);
}