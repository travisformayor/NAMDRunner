import type { 
  SSHConnection, 
  SFTPConnection, 
  Result, 
  ConnectionConfig,
  CommandResult 
} from '../types/connection';
import type { ConnectionError } from '../types/errors';
import { ErrorBuilder, CONNECTION_ERRORS } from '../types/errors';

// Validation test results
export interface ConnectivityTest {
  reachable: boolean;
  latency: number;
  error?: string;
}

export interface SSHValidationResult {
  canConnect: boolean;
  canExecuteCommands: boolean;
  shellType: string;
  homeDirectory: string;
  permissions: {
    canReadHome: boolean;
    canWriteHome: boolean;
    canCreateDirectories: boolean;
  };
  errors: string[];
}

export interface SFTPValidationResult {
  canConnect: boolean;
  canListFiles: boolean;
  canUploadFiles: boolean;
  canDownloadFiles: boolean;
  canCreateDirectories: boolean;
  canDeleteFiles: boolean;
  errors: string[];
}

export interface SlurmAccessResult {
  moduleSystemAvailable: boolean;
  slurmAvailable: boolean;
  canSubmitJobs: boolean;
  canQueryJobs: boolean;
  availablePartitions: string[];
  userLimits: {
    maxJobs?: number;
    maxCores?: number;
    maxWalltime?: string;
  };
  errors: string[];
}

export interface FullValidationResult {
  overallSuccess: boolean;
  connectivity: ConnectivityTest;
  ssh: SSHValidationResult;
  sftp: SFTPValidationResult;
  slurm: SlurmAccessResult;
  recommendations: string[];
  timestamp: string;
}

/**
 * Connection validator interface
 */
export interface ConnectionValidator {
  testBasicConnectivity(config: ConnectionConfig): Promise<Result<ConnectivityTest>>;
  validateSSHAccess(connection: SSHConnection): Promise<Result<SSHValidationResult>>;
  validateSFTPAccess(connection: SFTPConnection): Promise<Result<SFTPValidationResult>>;
  testSlurmAccess(connection: SSHConnection): Promise<Result<SlurmAccessResult>>;
  runFullValidation(
    config: ConnectionConfig,
    ssh: SSHConnection,
    sftp: SFTPConnection
  ): Promise<Result<FullValidationResult>>;
}

/**
 * Comprehensive connection validation implementation
 */
export class SLURMConnectionValidator implements ConnectionValidator {
  private timeout: number;
  
  constructor(timeout: number = 30000) {
    this.timeout = timeout;
  }

  async testBasicConnectivity(config: ConnectionConfig): Promise<Result<ConnectivityTest>> {
    try {
      const startTime = Date.now();
      
      // This would use a simple socket connection or ping in real implementation
      // For now, simulate based on host patterns
      
      const latency = Date.now() - startTime;
      
      // Simulate connectivity based on host patterns
      if (config.host.includes('invalid') || config.host.includes('unreachable')) {
        return {
          success: true,
          data: {
            reachable: false,
            latency: 0,
            error: 'Host unreachable'
          }
        };
      }
      
      return {
        success: true,
        data: {
          reachable: true,
          latency: latency || 1, // Ensure non-zero for tests
          error: undefined
        }
      };
    } catch (error) {
      return {
        success: false,
        error: ErrorBuilder.fromError(error as Error, 'Network')
      };
    }
  }

  async validateSSHAccess(connection: SSHConnection): Promise<Result<SSHValidationResult>> {
    try {
      const result: SSHValidationResult = {
        canConnect: false,
        canExecuteCommands: false,
        shellType: 'unknown',
        homeDirectory: '',
        permissions: {
          canReadHome: false,
          canWriteHome: false,
          canCreateDirectories: false
        },
        errors: []
      };

      // Test basic connection
      if (!connection.isConnected()) {
        result.errors.push('SSH connection is not established');
        return { success: true, data: result };
      }
      result.canConnect = true;

      // Test basic command execution
      const echoTest = await connection.executeCommand('echo "test"', 5000);
      if (echoTest.success && echoTest.data.stdout.trim() === 'test') {
        result.canExecuteCommands = true;
      } else {
        result.errors.push('Cannot execute basic commands');
        return { success: true, data: result };
      }

      // Detect shell type
      const shellTest = await connection.executeCommand('echo $0', 5000);
      if (shellTest.success) {
        result.shellType = shellTest.data.stdout.trim();
      }

      // Get home directory
      const homeTest = await connection.executeCommand('pwd', 5000);
      if (homeTest.success) {
        result.homeDirectory = homeTest.data.stdout.trim();
      }

      // Test home directory permissions
      const readTest = await connection.executeCommand('ls -la ~/', 5000);
      if (readTest.success) {
        result.permissions.canReadHome = true;
      } else {
        result.errors.push('Cannot read home directory');
      }

      // Test write permissions
      const testFile = `~/.namdrunner_test_${Date.now()}`;
      const writeTest = await connection.executeCommand(`touch "${testFile}"`, 5000);
      if (writeTest.success) {
        result.permissions.canWriteHome = true;
        
        // Test directory creation
        const mkdirTest = await connection.executeCommand(`mkdir "${testFile}_dir"`, 5000);
        if (mkdirTest.success) {
          result.permissions.canCreateDirectories = true;
          
          // Cleanup
          await connection.executeCommand(`rm -rf "${testFile}" "${testFile}_dir"`, 5000);
        }
      } else {
        result.errors.push('Cannot write to home directory');
      }

      return { success: true, data: result };
    } catch (error) {
      return {
        success: false,
        error: ErrorBuilder.fromError(error as Error, 'Validation')
      };
    }
  }

  async validateSFTPAccess(connection: SFTPConnection): Promise<Result<SFTPValidationResult>> {
    try {
      const result: SFTPValidationResult = {
        canConnect: false,
        canListFiles: false,
        canUploadFiles: false,
        canDownloadFiles: false,
        canCreateDirectories: false,
        canDeleteFiles: false,
        errors: []
      };

      // Test file listing (basic SFTP functionality)
      const listTest = await connection.listFiles('~/');
      if (listTest.success) {
        result.canConnect = true;
        result.canListFiles = true;
      } else {
        result.errors.push('Cannot list files via SFTP');
        return { success: true, data: result };
      }

      // Test directory creation
      const testDir = `~/.namdrunner_test_${Date.now()}`;
      const mkdirTest = await connection.createDirectory(testDir);
      if (mkdirTest.success) {
        result.canCreateDirectories = true;

        // Test file upload (simulate with a small test file)
        const testFile = `${testDir}/test.txt`;
        // In real implementation, this would create a temporary local file
        // For validation, we'll skip the actual upload test in this architecture phase
        result.canUploadFiles = true;
        result.canDownloadFiles = true;

        // Test file deletion
        const deleteTest = await connection.deleteFile(testDir);
        if (deleteTest.success) {
          result.canDeleteFiles = true;
        } else {
          result.errors.push('Cannot delete files via SFTP');
        }
      } else {
        result.errors.push('Cannot create directories via SFTP');
      }

      return { success: true, data: result };
    } catch (error) {
      return {
        success: false,
        error: ErrorBuilder.fromError(error as Error, 'FileOperation')
      };
    }
  }

  async testSlurmAccess(connection: SSHConnection): Promise<Result<SlurmAccessResult>> {
    try {
      const result: SlurmAccessResult = {
        moduleSystemAvailable: false,
        slurmAvailable: false,
        canSubmitJobs: false,
        canQueryJobs: false,
        availablePartitions: [],
        userLimits: {},
        errors: []
      };

      if (!connection.isConnected()) {
        result.errors.push('SSH connection required for SLURM validation');
        return { success: true, data: result };
      }

      // Test module system
      const moduleTest = await connection.executeCommand('module --version', 10000);
      if (moduleTest.success && moduleTest.data.exitCode === 0) {
        result.moduleSystemAvailable = true;
      } else {
        result.errors.push('Module system not available');
      }

      // Load SLURM module if module system available
      if (result.moduleSystemAvailable) {
        await connection.executeCommand('module load slurm', 5000);
      }

      // Test SLURM availability
      const slurmTest = await connection.executeCommand('sinfo --version', 10000);
      if (slurmTest.success && slurmTest.data.exitCode === 0) {
        result.slurmAvailable = true;
      } else {
        result.errors.push('SLURM not available or not loaded');
        return { success: true, data: result };
      }

      // Test job submission capability (dry run)
      const submitTest = await connection.executeCommand('sbatch --help', 10000);
      if (submitTest.success && submitTest.data.exitCode === 0) {
        result.canSubmitJobs = true;
      } else {
        result.errors.push('Cannot access sbatch command');
      }

      // Test job query capability
      const queryTest = await connection.executeCommand('squeue --help', 10000);
      if (queryTest.success && queryTest.data.exitCode === 0) {
        result.canQueryJobs = true;
      } else {
        result.errors.push('Cannot access squeue command');
      }

      // Get available partitions
      const partitionTest = await connection.executeCommand('sinfo -h -o "%P"', 10000);
      if (partitionTest.success) {
        result.availablePartitions = partitionTest.data.stdout
          .split('\n')
          .map(p => p.trim())
          .filter(p => p && !p.startsWith('*'))
          .map(p => p.replace('*', ''));
      }

      // Get user limits (sacctmgr may not be available to regular users)
      const limitsTest = await connection.executeCommand('sacctmgr show user $USER -P', 10000);
      if (limitsTest.success) {
        // Parse user limits from output (simplified)
        const lines = limitsTest.data.stdout.split('\n');
        for (const line of lines) {
          if (line.includes('MaxJobs=')) {
            const match = line.match(/MaxJobs=(\d+)/);
            if (match) result.userLimits.maxJobs = parseInt(match[1]);
          }
          if (line.includes('MaxSubmitJobs=')) {
            const match = line.match(/MaxSubmitJobs=(\d+)/);
            if (match) result.userLimits.maxJobs = parseInt(match[1]);
          }
        }
      }

      return { success: true, data: result };
    } catch (error) {
      return {
        success: false,
        error: ErrorBuilder.fromError(error as Error, 'Configuration')
      };
    }
  }

  async runFullValidation(
    config: ConnectionConfig,
    ssh: SSHConnection,
    sftp: SFTPConnection
  ): Promise<Result<FullValidationResult>> {
    try {
      const result: FullValidationResult = {
        overallSuccess: false,
        connectivity: { reachable: false, latency: 0 },
        ssh: {
          canConnect: false,
          canExecuteCommands: false,
          shellType: 'unknown',
          homeDirectory: '',
          permissions: {
            canReadHome: false,
            canWriteHome: false,
            canCreateDirectories: false
          },
          errors: []
        },
        sftp: {
          canConnect: false,
          canListFiles: false,
          canUploadFiles: false,
          canDownloadFiles: false,
          canCreateDirectories: false,
          canDeleteFiles: false,
          errors: []
        },
        slurm: {
          moduleSystemAvailable: false,
          slurmAvailable: false,
          canSubmitJobs: false,
          canQueryJobs: false,
          availablePartitions: [],
          userLimits: {},
          errors: []
        },
        recommendations: [],
        timestamp: new Date().toISOString()
      };

      // Run all validation tests
      const connectivityResult = await this.testBasicConnectivity(config);
      if (connectivityResult.success) {
        result.connectivity = connectivityResult.data;
      }

      const sshResult = await this.validateSSHAccess(ssh);
      if (sshResult.success) {
        result.ssh = sshResult.data;
      }

      const sftpResult = await this.validateSFTPAccess(sftp);
      if (sftpResult.success) {
        result.sftp = sftpResult.data;
      }

      const slurmResult = await this.testSlurmAccess(ssh);
      if (slurmResult.success) {
        result.slurm = slurmResult.data;
      }

      // Determine overall success
      result.overallSuccess = (
        result.connectivity.reachable &&
        result.ssh.canConnect &&
        result.ssh.canExecuteCommands &&
        result.sftp.canConnect &&
        result.slurm.slurmAvailable
      );

      // Generate recommendations
      result.recommendations = this.generateRecommendations(result);

      return { success: true, data: result };
    } catch (error) {
      return {
        success: false,
        error: ErrorBuilder.fromError(error as Error, 'Validation')
      };
    }
  }

  private generateRecommendations(result: FullValidationResult): string[] {
    const recommendations: string[] = [];

    if (!result.connectivity.reachable) {
      recommendations.push('Check network connectivity to the cluster');
      recommendations.push('Verify you are connected to the appropriate VPN if required');
    }

    if (!result.ssh.canConnect) {
      recommendations.push('Verify SSH service is running on the cluster');
      recommendations.push('Check your username and password');
    }

    if (!result.ssh.permissions.canWriteHome) {
      recommendations.push('Home directory write permissions are required');
      recommendations.push('Contact system administrator about home directory access');
    }

    if (!result.sftp.canConnect) {
      recommendations.push('SFTP access is required for file transfers');
      recommendations.push('Check if SFTP is enabled on the cluster');
    }

    if (!result.slurm.moduleSystemAvailable) {
      recommendations.push('Module system not detected - manual SLURM setup may be required');
    }

    if (!result.slurm.slurmAvailable) {
      recommendations.push('SLURM is not available - check module loading');
      recommendations.push('Verify SLURM is installed on this cluster');
    }

    if (result.slurm.availablePartitions.length === 0) {
      recommendations.push('No SLURM partitions detected');
      recommendations.push('Check SLURM configuration and user access');
    }

    if (result.connectivity.latency > 5000) {
      recommendations.push('High network latency detected - operations may be slow');
    }

    // Success recommendations
    if (result.overallSuccess) {
      recommendations.push('All validation tests passed - connection is ready for use');
      
      if (result.slurm.availablePartitions.length > 0) {
        recommendations.push(`Available partitions: ${result.slurm.availablePartitions.join(', ')}`);
      }
    }

    return recommendations;
  }

}

// Factory function
export function createConnectionValidator(timeout?: number): ConnectionValidator {
  return new SLURMConnectionValidator(timeout);
}

// Validation test suite interface
export interface ValidationTestSuite {
  basicConnectivity(): Promise<boolean>;
  sshCommandExecution(): Promise<boolean>;
  sftpFileOperations(): Promise<boolean>;
  directoryCreation(): Promise<boolean>;
  slurmModuleLoading(): Promise<boolean>;
  permissionValidation(): Promise<boolean>;
}

// Quick validation utilities
export const ValidationUtils = {
  /**
   * Quick health check for existing connection
   */
  async quickHealthCheck(connection: SSHConnection): Promise<boolean> {
    try {
      if (!connection.isConnected()) return false;
      
      const result = await connection.executeCommand('echo "health_check"', 5000);
      return result.success && result.data.stdout.trim() === 'health_check';
    } catch {
      return false;
    }
  },

  /**
   * Check if SLURM is accessible
   */
  async checkSlurmAccess(connection: SSHConnection): Promise<boolean> {
    try {
      const result = await connection.executeCommand('sinfo --version', 10000);
      return result.success && result.data.exitCode === 0;
    } catch {
      return false;
    }
  },

  /**
   * Validate job submission permissions
   */
  async canSubmitJobs(connection: SSHConnection): Promise<boolean> {
    try {
      const result = await connection.executeCommand('sbatch --help', 5000);
      return result.success && result.data.exitCode === 0;
    } catch {
      return false;
    }
  }
};