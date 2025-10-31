
import type { SessionInfo, ConnectionState } from './api';

// Re-export ConnectionState for services
export type { ConnectionState } from './api';

// State transition record for history tracking
export interface StateTransition {
  from: ConnectionState;
  to: ConnectionState;
  timestamp: string;
  reason: string | undefined;
  success: boolean;
}

// Result pattern for error handling (keeping this - it adds real value)
export type Result<T, E = Error> =
  | { success: true; data: T }
  | { success: false; error: E };

// Connection configuration (simplified to what we actually use)
export interface ConnectionConfig {
  host: string;
  username: string;
  password: string;
  port?: number;
  timeout?: number;
}

// Command execution result
export interface CommandResult {
  stdout: string;
  stderr: string;
  exitCode: number;
}

// File information (simplified)
export interface FileInfo {
  name: string;
  path: string;
  size: number;
  modified_at: string;
  isDirectory: boolean;
}

// Simple, focused SSH service interface
export interface SSHService {
  connect(config: ConnectionConfig): Promise<Result<SessionInfo>>;
  disconnect(): Promise<Result<void>>;
  executeCommand(command: string, timeout?: number): Promise<Result<CommandResult>>;
  getConnectionStatus(): Promise<Result<{ state: ConnectionState; session_info?: SessionInfo }>>;

  /** @deprecated Use getConnectionStatus() instead for reliable state */
  isConnected(): boolean;
}

// Simple, focused SFTP service interface
export interface SFTPService {
  uploadFile(localPath: string, remotePath: string): Promise<Result<void>>;
  downloadFile(remotePath: string, localPath: string): Promise<Result<void>>;
  listFiles(remotePath: string): Promise<Result<FileInfo[]>>;
  exists(remotePath: string): Promise<Result<boolean>>;
  createDirectory(remotePath: string): Promise<Result<void>>;
  getFileInfo(remotePath: string): Promise<Result<FileInfo>>;
}

// Simple state manager interface for mock client
export interface ConnectionStateManager {
  getCurrentState(): ConnectionState;
  transitionTo(state: ConnectionState, reason?: string): Result<void>;
  isConnected(): boolean;
  getStateHistory?(): StateTransition[];
  getLastError?(): import('./errors').ConnectionError | null;
  canRetry?(): boolean;
}

// Simple session manager interface for mock client
export interface SessionManager {
  saveSession(session_info: SessionInfo): Promise<Result<void>>;
  loadSession(): Promise<Result<SessionInfo | null>>;
  clearSession(): Promise<Result<void>>;
  getSessionInfo?(): SessionInfo | null;
  getSessionDiagnostics?(): any;
}