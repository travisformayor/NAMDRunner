import type { RemoteDirectoryManager } from './directoryManager';
import type { ConnectionValidator } from './connectionValidator';
import { SLURMDirectoryManager } from './directoryManager';
import { SLURMConnectionValidator } from './connectionValidator';
import { PathResolver, createPathResolver } from './pathResolver';

/**
 * Simplified service container for remaining services
 * SSH/SFTP now use direct service imports instead of dependency injection
 */

export interface ServiceDependencies {
  directoryManager: RemoteDirectoryManager;
  connectionValidator: ConnectionValidator;
  pathResolver: PathResolver;
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
   * Create dependencies bundle for simplified services
   */
  createDependencies(): ServiceDependencies {
    return {
      directoryManager: this.get<RemoteDirectoryManager>('directoryManager'),
      connectionValidator: this.get<ConnectionValidator>('connectionValidator'),
      pathResolver: this.get<PathResolver>('pathResolver'),
    };
  }
}

/**
 * Simplified service container
 * SSH/SFTP now use direct service imports
 */
export function createServiceContainer(): ServiceContainer {
  const container = new ServiceContainer();

  // Register remaining services (SSH/SFTP removed)
  container.register('pathResolver', () => createPathResolver());

  // Directory manager now uses direct SSH/SFTP service imports
  container.register('directoryManager', () => {
    const pathResolver = container.get<PathResolver>('pathResolver');
    return new SLURMDirectoryManager(pathResolver);
  });

  // Connection validator
  container.register('connectionValidator', () => new SLURMConnectionValidator());

  return container;
}

/**
 * Simplified mock service container
 * SSH/SFTP now use direct service imports with mock mode
 */
export function createMockServiceContainer(): ServiceContainer {
  const container = new ServiceContainer();

  // Register simplified mock implementations
  container.register('pathResolver', () => createPathResolver());

  // Directory manager with simplified dependencies
  container.register('directoryManager', () => {
    const pathResolver = container.get<PathResolver>('pathResolver');
    return new SLURMDirectoryManager(pathResolver);
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


// registerConnections function removed - SSH/SFTP now use direct service imports