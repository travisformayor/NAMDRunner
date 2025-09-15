import type { Result } from '../types/connection';
import type { ConnectionError } from '../types/errors';
import { ErrorBuilder, CONNECTION_ERRORS } from '../types/errors';
import { PathResolver, createPathResolver, type JobPaths as PathResolverJobPaths } from './pathResolver';
import { sshService, sftpService } from './index';

/**
 * Remote directory structure patterns and management
 * Handles consistent directory organization on SLURM clusters
 */

// Directory setup result
export interface DirectorySetupResult {
  basePath: string;
  namdrunnerPath: string;
  created: string[];
  existed: string[];
  errors: string[];
}

// Cleanup result
export interface CleanupResult {
  scanned: number;
  cleaned: number;
  failed: number;
  freedSpaceBytes?: number;
  errors: string[];
}

// Validation result
export interface ValidationResult {
  valid: boolean;
  basePath: string;
  namdrunnerPath: string;
  permissions: {
    readable: boolean;
    writable: boolean;
    executable: boolean;
  };
  diskSpace?: {
    total: number;
    used: number;
    available: number;
  };
  issues: string[];
}

/**
 * Remote directory manager interface
 */
export interface RemoteDirectoryManager {
  setupUserWorkspace(username: string): Promise<Result<DirectorySetupResult>>;
  getJobDirectory(username: string, jobId: string): string;
  ensureDirectoryExists(path: string): Promise<Result<boolean>>;
  cleanupOldJobs(username: string, daysOld: number): Promise<Result<CleanupResult>>;
  validateDirectoryStructure(username: string): Promise<Result<ValidationResult>>;
  getJobPaths(username: string, jobId: string): JobPaths;
  listJobs(username: string): Promise<Result<string[]>>;
  deleteJobDirectory(username: string, jobId: string): Promise<Result<void>>;
}

// Job directory paths (re-export from PathResolver for compatibility)
export type JobPaths = PathResolverJobPaths;

/**
 * Implementation of remote directory manager
 */
export class SLURMDirectoryManager implements RemoteDirectoryManager {
  private pathResolver: PathResolver;

  constructor(pathResolver?: PathResolver) {
    this.pathResolver = pathResolver || createPathResolver();
  }

  async setupUserWorkspace(username: string): Promise<Result<DirectorySetupResult>> {
    try {
      const basePathResult = this.pathResolver.getUserBasePath(username);
      const namdrunnerPathResult = this.pathResolver.getNAMDRunnerRoot(username);
      
      if (!basePathResult.success) {
        return basePathResult;
      }
      if (!namdrunnerPathResult.success) {
        return namdrunnerPathResult;
      }
      
      const basePath = basePathResult.data;
      const namdrunnerPath = namdrunnerPathResult.data;
      
      const result: DirectorySetupResult = {
        basePath,
        namdrunnerPath,
        created: [],
        existed: [],
        errors: []
      };

      // Check if base directory exists (should exist on SLURM clusters)
      const baseExists = await sftpService.exists(basePath);
      if (!baseExists.success || !baseExists.data) {
        return {
          success: false,
          error: ErrorBuilder.create(
            CONNECTION_ERRORS.DIRECTORY_ACCESS_DENIED,
            `Base directory ${basePath} does not exist or is not accessible`
          )
        };
      }

      // Ensure namdrunner root directory exists
      const namdrunnerExists = await sftpService.exists(namdrunnerPath);
      if (!namdrunnerExists.success) {
        result.errors.push(`Failed to check namdrunner directory: ${namdrunnerExists.error}`);
        return { success: false, error: namdrunnerExists.error };
      }

      if (!namdrunnerExists.data) {
        const createResult = await sftpService.createDirectory(namdrunnerPath);
        if (createResult.success) {
          result.created.push(namdrunnerPath);
        } else {
          result.errors.push(`Failed to create ${namdrunnerPath}: ${createResult.error}`);
          return { success: false, error: createResult.error };
        }
      } else {
        result.existed.push(namdrunnerPath);
      }

      return { success: true, data: result };
    } catch (error) {
      return {
        success: false,
        error: ErrorBuilder.fromError(error as Error, 'FileOperation')
      };
    }
  }

  getJobDirectory(username: string, jobId: string): string {
    const result = this.pathResolver.getJobDirectory(username, jobId);
    if (!result.success) {
      throw new Error(`Failed to resolve job directory: ${result.error.message}`);
    }
    return result.data;
  }

  async ensureDirectoryExists(path: string): Promise<Result<boolean>> {
    try {
      const exists = await sftpService.exists(path);
      if (!exists.success) {
        return { success: false, error: exists.error };
      }

      if (!exists.data) {
        const createResult = await sftpService.createDirectory(path);
        if (!createResult.success) {
          return { success: false, error: createResult.error };
        }
        return { success: true, data: true }; // Created
      }

      return { success: true, data: false }; // Already existed
    } catch (error) {
      return {
        success: false,
        error: ErrorBuilder.fromError(error as Error, 'FileOperation')
      };
    }
  }

  async cleanupOldJobs(username: string, daysOld: number): Promise<Result<CleanupResult>> {
    try {
      const namdrunnerPathResult = this.pathResolver.getNAMDRunnerRoot(username);
      if (!namdrunnerPathResult.success) {
        return namdrunnerPathResult;
      }
      const namdrunnerPath = namdrunnerPathResult.data;
      const result: CleanupResult = {
        scanned: 0,
        cleaned: 0,
        failed: 0,
        errors: []
      };

      // List job directories
      const listResult = await sftpService.listFiles(namdrunnerPath);
      if (!listResult.success) {
        return { success: false, error: listResult.error };
      }

      const cutoffTime = Date.now() - (daysOld * 24 * 60 * 60 * 1000);

      for (const file of listResult.data.files) {
        if (!file.isDirectory) continue;
        
        result.scanned++;
        
        // Check if directory is old enough
        const modifiedTime = new Date(file.modifiedAt).getTime();
        if (modifiedTime > cutoffTime) continue;

        try {
          const jobPath = `${namdrunnerPath}/${file.name}`;
          const deleteResult = await this.deleteJobDirectory(username, file.name);
          
          if (deleteResult.success) {
            result.cleaned++;
          } else {
            result.failed++;
            result.errors.push(`Failed to delete ${jobPath}: ${deleteResult.error}`);
          }
        } catch (error) {
          result.failed++;
          result.errors.push(`Error processing ${file.name}: ${error}`);
        }
      }

      return { success: true, data: result };
    } catch (error) {
      return {
        success: false,
        error: ErrorBuilder.fromError(error as Error, 'FileOperation')
      };
    }
  }

  async validateDirectoryStructure(username: string): Promise<Result<ValidationResult>> {
    try {
      const basePathResult = this.pathResolver.getUserBasePath(username);
      const namdrunnerPathResult = this.pathResolver.getNAMDRunnerRoot(username);
      
      if (!basePathResult.success) {
        return basePathResult;
      }
      if (!namdrunnerPathResult.success) {
        return namdrunnerPathResult;
      }
      
      const basePath = basePathResult.data;
      const namdrunnerPath = namdrunnerPathResult.data;
      
      const result: ValidationResult = {
        valid: true,
        basePath,
        namdrunnerPath,
        permissions: {
          readable: false,
          writable: false,
          executable: false
        },
        issues: []
      };

      // Check base directory
      const baseInfo = await sftpService.getFileInfo(basePath);
      if (!baseInfo.success) {
        result.valid = false;
        result.issues.push(`Cannot access base directory: ${basePath}`);
      } else {
        result.permissions.readable = baseInfo.data.permissions.includes('r');
        result.permissions.writable = baseInfo.data.permissions.includes('w');
        result.permissions.executable = baseInfo.data.permissions.includes('x');
      }

      // Check namdrunner directory
      const namdrunnerExists = await sftpService.exists(namdrunnerPath);
      if (!namdrunnerExists.success || !namdrunnerExists.data) {
        result.valid = false;
        result.issues.push(`NAMDRunner directory does not exist: ${namdrunnerPath}`);
      }

      // Check disk space using SSH command
      try {
        const dfCommand = `df -h "${basePath}" | tail -1`;
        const dfResult = await sshService.executeCommand(dfCommand);
        
        if (dfResult.success) {
          const diskInfo = this.parseDiskUsage(dfResult.data.stdout);
          if (diskInfo) {
            result.diskSpace = diskInfo;
            
            // Warn if disk space is low
            const usagePercent = (diskInfo.used / diskInfo.total) * 100;
            if (usagePercent > 90) {
              result.issues.push(`Disk usage is high: ${usagePercent.toFixed(1)}%`);
            }
          }
        }
      } catch (error) {
        result.issues.push(`Could not check disk space: ${error}`);
      }

      return { success: true, data: result };
    } catch (error) {
      return {
        success: false,
        error: ErrorBuilder.fromError(error as Error, 'FileOperation')
      };
    }
  }

  getJobPaths(username: string, jobId: string): JobPaths {
    const result = this.pathResolver.getJobPaths(username, jobId);
    if (!result.success) {
      throw new Error(`Failed to resolve job paths: ${result.error.message}`);
    }
    return result.data;
  }

  async listJobs(username: string): Promise<Result<string[]>> {
    try {
      const namdrunnerPathResult = this.pathResolver.getNAMDRunnerRoot(username);
      if (!namdrunnerPathResult.success) {
        return namdrunnerPathResult;
      }
      const namdrunnerPath = namdrunnerPathResult.data;
      const listResult = await sftpService.listFiles(namdrunnerPath);
      
      if (!listResult.success) {
        return { success: false, error: listResult.error };
      }

      const jobIds = listResult.data.files
        .filter(file => file.isDirectory)
        .map(file => file.name);

      return { success: true, data: jobIds };
    } catch (error) {
      return {
        success: false,
        error: ErrorBuilder.fromError(error as Error, 'FileOperation')
      };
    }
  }

  async deleteJobDirectory(username: string, jobId: string): Promise<Result<void>> {
    try {
      const jobDir = this.getJobDirectory(username, jobId);
      
      // Use SSH command for recursive delete (more reliable than SFTP)
      const rmCommand = `rm -rf "${jobDir}"`;
      const result = await sshService.executeCommand(rmCommand);
      
      if (!result.success || result.data.exitCode !== 0) {
        return {
          success: false,
          error: ErrorBuilder.create(
            CONNECTION_ERRORS.FILE_NOT_FOUND,
            `Failed to delete job directory: ${result.data?.stderr || 'Unknown error'}`
          )
        };
      }

      return { success: true, data: undefined };
    } catch (error) {
      return {
        success: false,
        error: ErrorBuilder.fromError(error as Error, 'FileOperation')
      };
    }
  }

  // Helper methods

  private parseDiskUsage(dfOutput: string): { total: number; used: number; available: number } | null {
    try {
      // Parse output like: "/dev/sda1    100G   80G   20G  80% /projects"
      const parts = dfOutput.trim().split(/\s+/);
      if (parts.length < 6) return null;
      
      const total = this.parseSize(parts[1]);
      const used = this.parseSize(parts[2]);
      const available = this.parseSize(parts[3]);
      
      if (total && used && available) {
        return { total, used, available };
      }
    } catch (error) {
      console.warn('Failed to parse disk usage:', error);
    }
    
    return null;
  }

  private parseSize(sizeStr: string): number | null {
    try {
      const match = sizeStr.match(/^(\d+(?:\.\d+)?)([KMGT]?)$/);
      if (!match) return null;
      
      const value = parseFloat(match[1]);
      const unit = match[2];
      
      const multipliers: Record<string, number> = {
        '': 1024, // Default is KB
        'K': 1024,
        'M': 1024 * 1024,
        'G': 1024 * 1024 * 1024,
        'T': 1024 * 1024 * 1024 * 1024
      };
      
      return value * (multipliers[unit] || 1);
    } catch (error) {
      return null;
    }
  }
}

// Simplified factory function
export function createDirectoryManager(pathResolver?: PathResolver): RemoteDirectoryManager {
  return new SLURMDirectoryManager(pathResolver);
}

