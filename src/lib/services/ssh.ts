import { invoke } from '@tauri-apps/api/core';
import type {
  SSHService,
  ConnectionConfig,
  Result,
  SessionInfo,
  CommandResult,
  ConnectionState
} from '../types/connection';

/**
 * Clean SSH service implementation
 * Uses Tauri commands directly without unnecessary abstractions
 */
class SSHServiceImpl implements SSHService {

  async connect(config: ConnectionConfig): Promise<Result<SessionInfo>> {
    try {
      const result = await invoke('connect_to_cluster', { params: config });

      if (result.success) {
        return { success: true, data: result.sessionInfo };
      } else {
        return { success: false, error: new Error(result.error || 'Connection failed') };
      }
    } catch (error) {
      return { success: false, error: error as Error };
    }
  }

  async disconnect(): Promise<Result<void>> {
    try {
      const result = await invoke('disconnect');

      if (result.success) {
        return { success: true, data: undefined };
      } else {
        return { success: false, error: new Error(result.error || 'Disconnect failed') };
      }
    } catch (error) {
      return { success: false, error: error as Error };
    }
  }

  async executeCommand(command: string, timeout?: number): Promise<Result<CommandResult>> {
    try {
      const result = await invoke('ssh_execute_command', { command, timeout });

      if (result.success) {
        return { success: true, data: result.data };
      } else {
        return { success: false, error: new Error(result.error || 'Command execution failed') };
      }
    } catch (error) {
      return { success: false, error: error as Error };
    }
  }

  async getConnectionStatus(): Promise<Result<{ state: ConnectionState; sessionInfo?: SessionInfo }>> {
    try {
      const result = await invoke('get_connection_status');

      return {
        success: true,
        data: {
          state: result.state,
          sessionInfo: result.sessionInfo
        }
      };
    } catch (error) {
      return { success: false, error: error as Error };
    }
  }

  isConnected(): boolean {
    // Always query Rust for current state instead of maintaining local state
    // Note: This is now async by nature, but we keep the sync interface for compatibility
    // In real usage, callers should use getConnectionStatus() for reliable state
    throw new Error('isConnected() is deprecated - use getConnectionStatus() instead for reliable state');
  }
}

// Export singleton instance
export const sshService = new SSHServiceImpl();