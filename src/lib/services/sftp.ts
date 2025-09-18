import { invoke } from '@tauri-apps/api/core';
import type {
  SFTPService,
  Result,
  FileInfo
} from '../types/connection';

/**
 * Clean SFTP service implementation
 * Uses Tauri commands directly without unnecessary abstractions
 */
class SFTPServiceImpl implements SFTPService {
  async uploadFile(localPath: string, remotePath: string): Promise<Result<void>> {
    try {
      const result = await invoke('sftp_upload_file', { localPath, remotePath });

      if (result.success) {
        return { success: true, data: undefined };
      } else {
        return { success: false, error: new Error(result.error || 'Upload failed') };
      }
    } catch (error) {
      return { success: false, error: error as Error };
    }
  }

  async downloadFile(remotePath: string, localPath: string): Promise<Result<void>> {
    try {
      const result = await invoke('sftp_download_file', { remotePath, localPath });

      if (result.success) {
        return { success: true, data: undefined };
      } else {
        return { success: false, error: new Error(result.error || 'Download failed') };
      }
    } catch (error) {
      return { success: false, error: error as Error };
    }
  }

  async listFiles(remotePath: string): Promise<Result<FileInfo[]>> {
    try {
      const result = await invoke('sftp_list_files', { remotePath });

      if (result.success) {
        // The data is already properly typed FileInfo[] from Rust
        return { success: true, data: result.data };
      } else {
        return { success: false, error: new Error(result.error || 'List files failed') };
      }
    } catch (error) {
      return { success: false, error: error as Error };
    }
  }

  async exists(remotePath: string): Promise<Result<boolean>> {
    try {
      const result = await invoke('sftp_exists', { remotePath });

      if (result.success) {
        return { success: true, data: result.data };
      } else {
        return { success: false, error: new Error(result.error || 'Exists check failed') };
      }
    } catch (error) {
      return { success: false, error: error as Error };
    }
  }

  async createDirectory(remotePath: string): Promise<Result<void>> {
    try {
      const result = await invoke('sftp_create_directory', { remotePath });

      if (result.success) {
        return { success: true, data: undefined };
      } else {
        return { success: false, error: new Error(result.error || 'Create directory failed') };
      }
    } catch (error) {
      return { success: false, error: error as Error };
    }
  }

  async getFileInfo(remotePath: string): Promise<Result<FileInfo>> {
    try {
      const result = await invoke('sftp_get_file_info', { remotePath });

      if (result.success) {
        // The data is already properly typed FileInfo from Rust
        return { success: true, data: result.data };
      } else {
        return { success: false, error: new Error(result.error || 'Get file info failed') };
      }
    } catch (error) {
      return { success: false, error: error as Error };
    }
  }
}

// Export singleton instance
export const sftpService = new SFTPServiceImpl();