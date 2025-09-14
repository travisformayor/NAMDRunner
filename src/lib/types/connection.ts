
import type { SessionInfo } from './api';

// Re-export ConnectionState for services
export type { ConnectionState } from './api';

// Result pattern for error handling
export type Result<T, E = Error> = 
  | { success: true; data: T }
  | { success: false; error: E };

// Observable pattern for state management
export interface Observable<T> {
  subscribe(observer: (value: T) => void): () => void;
  getValue(): T;
}

// Connection configuration
export interface ConnectionConfig {
  host: string;
  username: string;
  password: string;
  port?: number;
  timeout?: number;
  retryAttempts?: number;
  retryDelay?: number;
  keepAliveInterval?: number;
  commandTimeout?: number;
}

// SSH Connection interface
export interface SSHConnection {
  connect(config: ConnectionConfig): Promise<Result<SessionInfo>>;
  disconnect(): Promise<Result<void>>;
  executeCommand(command: string, timeout?: number): Promise<Result<CommandResult>>;
  validateConnection(): Promise<Result<boolean>>;
  getStatus(): ConnectionState;
  isConnected(): boolean;
}

// SFTP Connection interface
export interface SFTPConnection {
  uploadFile(localPath: string, remotePath: string): Promise<Result<FileTransferResult>>;
  downloadFile(remotePath: string, localPath: string): Promise<Result<FileTransferResult>>;
  listFiles(remotePath: string): Promise<Result<FileListResult>>;
  createDirectory(remotePath: string): Promise<Result<DirectoryResult>>;
  deleteFile(remotePath: string): Promise<Result<FileOperationResult>>;
  exists(remotePath: string): Promise<Result<boolean>>;
  getFileInfo(remotePath: string): Promise<Result<FileInfo>>;
}

// Command execution result
export interface CommandResult {
  stdout: string;
  stderr: string;
  exitCode: number;
  duration: number;
  timedOut: boolean;
}

// File transfer result
export interface FileTransferResult {
  bytesTransferred: number;
  transferTime: number;
  checksum?: string;
  remotePath: string;
  localPath: string;
}

// File listing result
export interface FileListResult {
  files: FileInfo[];
  totalCount: number;
  path: string;
}

// File information
export interface FileInfo {
  name: string;
  path: string;
  size: number;
  modifiedAt: string;
  permissions: string;
  isDirectory: boolean;
  owner?: string;
  group?: string;
}

// Directory operation result
export interface DirectoryResult {
  path: string;
  created: boolean;
  existed: boolean;
}

// File operation result
export interface FileOperationResult {
  path: string;
  operation: 'delete' | 'move' | 'copy' | 'chmod';
  success: boolean;
  details?: string;
}

// Connection state manager interface
export interface ConnectionStateManager {
  getCurrentState(): ConnectionState;
  transitionTo(newState: ConnectionState, reason?: string): Result<void>;
  canTransition(from: ConnectionState, to: ConnectionState): boolean;
  isConnected(): boolean;
  canRetry(): boolean;
  getStateHistory(): StateTransition[];
  onStateChange(callback: (state: ConnectionState) => void): () => void;
}

// State transition record
export interface StateTransition {
  from: ConnectionState;
  to: ConnectionState;
  timestamp: string;
  reason?: string;
  success: boolean;
}

// Session manager interface
export interface SessionManager {
  saveSession(sessionInfo: SessionInfo): Promise<Result<void>>;
  loadSession(): Promise<Result<SessionInfo | null>>;
  isSessionValid(sessionInfo: SessionInfo): boolean;
  clearSession(): Promise<Result<void>>;
  refreshSession(): Promise<Result<SessionInfo>>;
  scheduleSessionRefresh(intervalMs: number): void;
  cancelSessionRefresh(): void;
  getSessionAge(): number | null;
}

// Connection factory for dependency injection
export interface ConnectionFactory {
  createSSHConnection(config?: Partial<ConnectionConfig>): SSHConnection;
  createSFTPConnection(config?: Partial<ConnectionConfig>): SFTPConnection;
  createStateManager(): ConnectionStateManager;
  createSessionManager(): SessionManager;
}

// Connection pool for managing multiple connections
export interface ConnectionPool {
  getConnection(id: string): SSHConnection | undefined;
  addConnection(id: string, connection: SSHConnection): void;
  removeConnection(id: string): void;
  hasConnection(id: string): boolean;
  getAllConnections(): Map<string, SSHConnection>;
  closeAll(): Promise<void>;
}

// Health check interface
export interface ConnectionHealthCheck {
  checkSSH(connection: SSHConnection): Promise<HealthCheckResult>;
  checkSFTP(connection: SFTPConnection): Promise<HealthCheckResult>;
  checkSLURM(connection: SSHConnection): Promise<HealthCheckResult>;
  runFullHealthCheck(connection: SSHConnection): Promise<FullHealthCheckResult>;
}

// Health check results
export interface HealthCheckResult {
  service: 'ssh' | 'sftp' | 'slurm';
  healthy: boolean;
  latency?: number;
  error?: string;
  details?: Record<string, any>;
}

export interface FullHealthCheckResult {
  overallHealth: boolean;
  checks: HealthCheckResult[];
  timestamp: string;
  recommendations?: string[];
}