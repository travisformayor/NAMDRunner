// Simple, clean service exports
// No complex dependency injection - just direct imports

export { sshService } from './ssh';
export { sftpService } from './sftp';

// Re-export types for convenience
export type {
  SSHService,
  SFTPService,
  ConnectionConfig,
  Result,
  CommandResult,
  FileInfo
} from '../types/connection';