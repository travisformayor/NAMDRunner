import { describe, it, expect, beforeEach, vi } from 'vitest';
import { SLURMDirectoryManager } from '../../services/directoryManager';
import { PathResolver, REMOTE_PATHS } from '../../services/pathResolver';
import type { 
  SSHConnection, 
  SFTPConnection,
  CommandResult,
  FileListResult,
  FileInfo,
  DirectoryResult,
  Result
} from '../../types/connection';

// Mock implementations
class MockSSHConnection implements SSHConnection {
  async connect(): Promise<Result<any>> {
    return { success: true, data: {} };
  }

  async disconnect(): Promise<Result<void>> {
    return { success: true, data: undefined };
  }

  async executeCommand(command: string): Promise<Result<CommandResult>> {
    // Mock df command for disk space
    if (command.includes('df -h')) {
      return {
        success: true,
        data: {
          stdout: '/dev/sda1    100G   80G   20G  80% /projects',
          stderr: '',
          exitCode: 0,
          duration: 100,
          timedOut: false
        }
      };
    }

    // Mock rm command for deletion
    if (command.includes('rm -rf')) {
      return {
        success: true,
        data: {
          stdout: '',
          stderr: '',
          exitCode: 0,
          duration: 200,
          timedOut: false
        }
      };
    }

    return {
      success: true,
      data: {
        stdout: 'mock command output',
        stderr: '',
        exitCode: 0,
        duration: 100,
        timedOut: false
      }
    };
  }

  async validateConnection(): Promise<Result<boolean>> {
    return { success: true, data: true };
  }

  getStatus(): any {
    return 'Connected';
  }

  isConnected(): boolean {
    return true;
  }
}

class MockSFTPConnection implements SFTPConnection {
  private shouldFailExists = false;
  private shouldFailCreate = false;

  setShouldFailExists(fail: boolean) {
    this.shouldFailExists = fail;
  }

  setShouldFailCreate(fail: boolean) {
    this.shouldFailCreate = fail;
  }

  async uploadFile(): Promise<Result<any>> {
    return { success: true, data: {} };
  }

  async downloadFile(): Promise<Result<any>> {
    return { success: true, data: {} };
  }

  async listFiles(remotePath: string): Promise<Result<FileListResult>> {
    const mockFiles: FileInfo[] = [
      {
        name: 'job_001',
        path: `${remotePath}/job_001`,
        size: 0,
        modifiedAt: new Date(Date.now() - 2 * 24 * 60 * 60 * 1000).toISOString(), // 2 days old
        permissions: 'drwxr-xr-x',
        isDirectory: true,
        owner: 'testuser',
        group: 'testgroup'
      },
      {
        name: 'job_002',
        path: `${remotePath}/job_002`,
        size: 0,
        modifiedAt: new Date().toISOString(), // Recent
        permissions: 'drwxr-xr-x',
        isDirectory: true,
        owner: 'testuser',
        group: 'testgroup'
      }
    ];

    return {
      success: true,
      data: {
        files: mockFiles,
        totalCount: mockFiles.length,
        path: remotePath
      }
    };
  }

  async createDirectory(remotePath: string): Promise<Result<DirectoryResult>> {
    if (this.shouldFailCreate) {
      return {
        success: false,
        error: new Error('Permission denied')
      };
    }

    return {
      success: true,
      data: {
        path: remotePath,
        created: true,
        existed: false
      }
    };
  }

  async deleteFile(): Promise<Result<any>> {
    return { success: true, data: {} };
  }

  async exists(remotePath: string): Promise<Result<boolean>> {
    if (this.shouldFailExists) {
      return {
        success: false,
        error: new Error('Access denied')
      };
    }

    // Mock specific directories that exist vs don't exist
    const existingDirs = [
      '/projects/testuser',
      '/projects/testuser/namdrunner_jobs',
      '/projects/testuser/existing_dir'
    ];
    
    const exists = existingDirs.includes(remotePath) || remotePath.includes('nonexistent') === false && remotePath === '/projects/testuser/namdrunner_jobs';
    return { success: true, data: existingDirs.includes(remotePath) };
  }

  async getFileInfo(remotePath: string): Promise<Result<FileInfo>> {
    return {
      success: true,
      data: {
        name: 'testdir',
        path: remotePath,
        size: 0,
        modifiedAt: new Date().toISOString(),
        permissions: 'drwxr-xr-x',
        isDirectory: true,
        owner: 'testuser',
        group: 'testgroup'
      }
    };
  }
}

describe('SLURMDirectoryManager', () => {
  let directoryManager: SLURMDirectoryManager;
  let mockSSH: MockSSHConnection;
  let mockSFTP: MockSFTPConnection;
  let pathResolver: PathResolver;

  beforeEach(() => {
    mockSSH = new MockSSHConnection();
    mockSFTP = new MockSFTPConnection();
    pathResolver = new PathResolver();
    directoryManager = new SLURMDirectoryManager(mockSSH, mockSFTP, pathResolver);
  });

  describe('User Workspace Setup', () => {
    it('should setup workspace successfully when base exists', async () => {
      const result = await directoryManager.setupUserWorkspace('testuser');
      
      expect(result.success).toBe(true);
      expect(result.data.basePath).toBe('/projects/testuser');
      expect(result.data.namdrunnerPath).toBe('/projects/testuser/namdrunner_jobs');
      expect(result.data.created.length + result.data.existed.length).toBeGreaterThan(0);
    });

    it('should fail when base directory does not exist', async () => {
      mockSFTP.setShouldFailExists(true);
      
      const result = await directoryManager.setupUserWorkspace('nonexistentuser');
      
      expect(result.success).toBe(false);
      expect(result.error?.message).toContain('Cannot access');
    });

    it('should handle namdrunner directory creation failure', async () => {
      // Mock scenario where namdrunner directory doesn't exist and creation fails
      const customMockSFTP = new MockSFTPConnection();
      customMockSFTP.exists = vi.fn().mockImplementation((remotePath: string) => {
        if (remotePath.includes('/projects/testuser/namdrunner_jobs')) {
          return { success: true, data: false }; // Directory doesn't exist
        }
        return { success: true, data: true }; // Base directory exists
      });
      customMockSFTP.setShouldFailCreate(true);
      
      const customDirectoryManager = new SLURMDirectoryManager(mockSSH, customMockSFTP, pathResolver);
      const result = await customDirectoryManager.setupUserWorkspace('testuser');
      
      expect(result.success).toBe(false);
      expect(result.error).toBeDefined();
    });
  });

  describe('Directory Path Generation', () => {
    it('should generate correct job directory paths', () => {
      const jobDir = directoryManager.getJobDirectory('testuser', 'job_001');
      expect(jobDir).toBe('/projects/testuser/namdrunner_jobs/job_001');
    });

    it('should get complete job paths structure', () => {
      const paths = directoryManager.getJobPaths('testuser', 'job_001');
      
      expect(paths.jobDir).toBe('/projects/testuser/namdrunner_jobs/job_001');
      expect(paths.logsDir).toBe('/projects/testuser/namdrunner_jobs/job_001/logs');
      expect(paths.inputsDir).toBe('/projects/testuser/namdrunner_jobs/job_001/inputs');
      expect(paths.outputsDir).toBe('/projects/testuser/namdrunner_jobs/job_001/outputs');
      expect(paths.configFile).toBe('/projects/testuser/namdrunner_jobs/job_001/job.json');
      expect(paths.slurmScript).toBe('/projects/testuser/namdrunner_jobs/job_001/job.slurm');
    });
  });

  describe('Directory Operations', () => {
    it('should ensure directory exists (create new)', async () => {
      const result = await directoryManager.ensureDirectoryExists('/projects/testuser/new_dir');
      
      expect(result.success).toBe(true);
      expect(result.data).toBe(true); // Was created
    });

    it('should ensure directory exists (already exists)', async () => {
      const result = await directoryManager.ensureDirectoryExists('/projects/testuser/existing_dir');
      
      expect(result.success).toBe(true);
      expect(result.data).toBe(false); // Already existed
    });

    it('should handle directory creation failures', async () => {
      mockSFTP.setShouldFailCreate(true);
      
      const result = await directoryManager.ensureDirectoryExists('/projects/testuser/fail_dir');
      
      expect(result.success).toBe(false);
    });
  });

  describe('Job Listing', () => {
    it('should list existing jobs', async () => {
      const result = await directoryManager.listJobs('testuser');
      
      expect(result.success).toBe(true);
      expect(result.data).toEqual(['job_001', 'job_002']);
    });

    it('should handle listing failures', async () => {
      // Create custom mock that makes listFiles fail
      const failingMockSFTP = {
        ...mockSFTP,
        listFiles: vi.fn().mockResolvedValue({
          success: false,
          error: new Error('Permission denied')
        })
      } as any;
      
      const failingManager = new SLURMDirectoryManager(mockSSH, failingMockSFTP, pathResolver);
      const result = await failingManager.listJobs('testuser');
      
      expect(result.success).toBe(false);
    });
  });

  describe('Job Directory Deletion', () => {
    it('should delete job directory successfully', async () => {
      const result = await directoryManager.deleteJobDirectory('testuser', 'job_001');
      
      expect(result.success).toBe(true);
    });

    it('should handle deletion command failures', async () => {
      // Mock SSH command failure
      const failingSSH = {
        ...mockSSH,
        executeCommand: vi.fn().mockResolvedValue({
          success: true,
          data: {
            stdout: '',
            stderr: 'Permission denied',
            exitCode: 1,
            duration: 100,
            timedOut: false
          }
        })
      } as any;

      const failingManager = new SLURMDirectoryManager(failingSSH, mockSFTP, pathResolver);
      const result = await failingManager.deleteJobDirectory('testuser', 'job_001');
      
      expect(result.success).toBe(false);
      expect(result.error?.details).toContain('Failed to delete job directory');
    });
  });

  describe('Directory Cleanup', () => {
    it('should cleanup old job directories', async () => {
      const result = await directoryManager.cleanupOldJobs('testuser', 1); // 1 day old
      
      expect(result.success).toBe(true);
      expect(result.data.scanned).toBe(2);
      expect(result.data.cleaned).toBeGreaterThanOrEqual(0);
    });

    it('should handle cleanup failures gracefully', async () => {
      // Create custom mock that makes listFiles fail
      const failingMockSFTP = {
        ...mockSFTP,
        listFiles: vi.fn().mockResolvedValue({
          success: false,
          error: new Error('Access denied')
        })
      } as any;
      
      const failingManager = new SLURMDirectoryManager(mockSSH, failingMockSFTP, pathResolver);
      const result = await failingManager.cleanupOldJobs('testuser', 1);
      
      expect(result.success).toBe(false);
    });

    it('should not cleanup recent directories', async () => {
      const result = await directoryManager.cleanupOldJobs('testuser', 5); // 5 days old
      
      expect(result.success).toBe(true);
      expect(result.data.cleaned).toBe(0); // Nothing should be old enough
    });
  });

  describe('Directory Validation', () => {
    it('should validate directory structure successfully', async () => {
      const result = await directoryManager.validateDirectoryStructure('testuser');
      
      expect(result.success).toBe(true);
      expect(result.data.valid).toBe(true);
      expect(result.data.basePath).toBe('/projects/testuser');
      expect(result.data.namdrunnerPath).toBe('/projects/testuser/namdrunner_jobs');
      expect(result.data.permissions.readable).toBe(true);
      expect(result.data.diskSpace).toBeDefined();
      expect(result.data.diskSpace?.total).toBeGreaterThan(0);
    });

    it('should detect validation issues', async () => {
      mockSFTP.setShouldFailExists(true);
      
      const result = await directoryManager.validateDirectoryStructure('testuser');
      
      expect(result.success).toBe(true);
      expect(result.data.valid).toBe(false);
      expect(result.data.issues.length).toBeGreaterThan(0);
    });

    it('should handle disk space parsing errors gracefully', async () => {
      const badSSH = {
        ...mockSSH,
        executeCommand: vi.fn().mockResolvedValue({
          success: true,
          data: {
            stdout: 'invalid df output',
            stderr: '',
            exitCode: 0,
            duration: 100,
            timedOut: false
          }
        })
      } as any;

      const badManager = new SLURMDirectoryManager(badSSH, mockSFTP, pathResolver);
      const result = await badManager.validateDirectoryStructure('testuser');
      
      expect(result.success).toBe(true);
      // Should not crash, but disk space may be undefined
    });
  });
});

describe('PathResolver', () => {
  let pathResolver: PathResolver;

  beforeEach(() => {
    pathResolver = new PathResolver();
  });

  describe('Path Generation', () => {
    it('should generate correct job directory paths', () => {
      const result = pathResolver.getJobDirectory('testuser', 'job_001');
      expect(result.success).toBe(true);
      expect(result.data).toBe('/projects/testuser/namdrunner_jobs/job_001');
    });

    it('should generate complete job paths structure', () => {
      const result = pathResolver.getJobPaths('testuser', 'job_001');
      expect(result.success).toBe(true);
      expect(result.data.jobDir).toBe('/projects/testuser/namdrunner_jobs/job_001');
      expect(result.data.logsDir).toBe('/projects/testuser/namdrunner_jobs/job_001/logs');
      expect(result.data.inputsDir).toBe('/projects/testuser/namdrunner_jobs/job_001/inputs');
      expect(result.data.outputsDir).toBe('/projects/testuser/namdrunner_jobs/job_001/outputs');
      expect(result.data.scratchDir).toBe('/projects/testuser/namdrunner_jobs/job_001/scratch');
      expect(result.data.configFile).toBe('/projects/testuser/namdrunner_jobs/job_001/job.json');
      expect(result.data.slurmScript).toBe('/projects/testuser/namdrunner_jobs/job_001/job.slurm');
    });

    it('should generate extended job paths with log files', () => {
      const result = pathResolver.getExtendedJobPaths('testuser', 'job_001');
      expect(result.success).toBe(true);
      expect(result.data.logFiles.stdout).toBe('/projects/testuser/namdrunner_jobs/job_001/logs/job.out');
      expect(result.data.logFiles.stderr).toBe('/projects/testuser/namdrunner_jobs/job_001/logs/job.err');
      expect(result.data.logFiles.slurm).toBe('/projects/testuser/namdrunner_jobs/job_001/logs/slurm.log');
    });
  });

  describe('Path Validation', () => {
    it('should validate correct paths', () => {
      expect(pathResolver.validatePath('/projects/testuser/namdrunner_jobs')).toBe(true);
      expect(pathResolver.validatePath('/valid/absolute/path')).toBe(true);
    });

    it('should reject invalid paths', () => {
      expect(pathResolver.validatePath('relative/path')).toBe(false);
      expect(pathResolver.validatePath('/path/../with/traversal')).toBe(false);
      expect(pathResolver.validatePath('/path//with//double//slashes')).toBe(false);
    });
  });

  describe('Job ID Operations', () => {
    it('should sanitize job IDs', () => {
      expect(pathResolver.sanitizeJobId('job-001')).toBe('job-001');
      expect(pathResolver.sanitizeJobId('valid_job_123')).toBe('valid_job_123');
      expect(pathResolver.sanitizeJobId('')).toBe('unnamed'); // Empty string gets sanitized to 'unnamed'
      
      // Test that problematic chars get sanitized - this should produce 'job_001'
      const problematicResult = pathResolver.sanitizeJobId('job@#$%001');
      expect(problematicResult).toBe('job_001'); // Sanitized correctly
      
      // Test something that would trigger 'unnamed' fallback (empty after sanitization)
      const invalidResult = pathResolver.sanitizeJobId('');
      expect(invalidResult).toBe('unnamed'); // Empty gets 'unnamed' which passes validation
      
      // Test unicode chars get same treatment
      const unicodeResult = pathResolver.sanitizeJobId('€€€');
      expect(unicodeResult).toBe('unnamed'); // Unicode chars also become 'unnamed'
    });

    it('should extract job ID from paths', () => {
      const validPath = '/projects/testuser/namdrunner_jobs/job_001/logs/output.log';
      const invalidPath = '/projects/testuser/other_dir/job_001';
      
      expect(pathResolver.extractJobIdFromPath(validPath)).toBe('job_001');
      expect(pathResolver.extractJobIdFromPath(invalidPath)).toBeNull();
    });
  });

  describe('Path Security', () => {
    it('should check if path is allowed for user', () => {
      expect(pathResolver.isPathAllowed('/projects/testuser/namdrunner_jobs/job_001', 'testuser')).toBe(true);
      expect(pathResolver.isPathAllowed('/projects/otheruser/namdrunner_jobs/job_001', 'testuser')).toBe(false);
    });
  });
});

describe('REMOTE_PATHS Constants', () => {
  it('should have correct path templates', () => {
    expect(REMOTE_PATHS.USER_BASE).toBe('/projects/$USER');
    expect(REMOTE_PATHS.NAMDRUNNER_ROOT).toBe('/projects/$USER/namdrunner_jobs');
    expect(REMOTE_PATHS.JOB_TEMPLATE).toBe('/projects/$USER/namdrunner_jobs/{jobId}');
    expect(REMOTE_PATHS.LOGS_DIR).toBe('logs');
    expect(REMOTE_PATHS.INPUTS_DIR).toBe('inputs');
    expect(REMOTE_PATHS.OUTPUTS_DIR).toBe('outputs');
    expect(REMOTE_PATHS.CONFIG_FILE).toBe('job.json');
    expect(REMOTE_PATHS.SLURM_SCRIPT).toBe('job.slurm');
  });
});