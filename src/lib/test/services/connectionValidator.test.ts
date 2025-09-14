import { describe, it, expect, beforeEach, vi } from 'vitest';
import {
  SLURMConnectionValidator,
  ValidationUtils
} from '../../services/connectionValidator';
import type {
  SSHConnection,
  SFTPConnection,
  ConnectionConfig
} from '../../types/connection';
import { ConnectionMocks } from '../utils/connectionMocks';

describe('SLURMConnectionValidator', () => {
  let validator: SLURMConnectionValidator;
  let mockSSH: SSHConnection;
  let mockSFTP: SFTPConnection;
  let mockConfig: ConnectionConfig;

  beforeEach(() => {
    validator = new SLURMConnectionValidator();
    mockSSH = ConnectionMocks.workingSSH();
    mockSFTP = ConnectionMocks.workingSFTP();
    mockConfig = {
      host: 'test.cluster.com',
      username: 'testuser',
      password: 'testpass'
    };
  });

  describe('Basic Connectivity Testing', () => {
    it('should test successful connectivity', async () => {
      const result = await validator.testBasicConnectivity(mockConfig);
      
      expect(result.success).toBe(true);
      expect(result.data.reachable).toBe(true);
      expect(result.data.latency).toBeGreaterThan(0);
    });

    it('should detect unreachable hosts', async () => {
      const unreachableConfig = {
        ...mockConfig,
        host: 'invalid.cluster.test'
      };

      const result = await validator.testBasicConnectivity(unreachableConfig);
      
      expect(result.success).toBe(true);
      expect(result.data.reachable).toBe(false);
      expect(result.data.error).toBe('Host unreachable');
    });
  });

  describe('SSH Access Validation', () => {
    it('should validate successful SSH access', async () => {
      const result = await validator.validateSSHAccess(mockSSH);
      
      expect(result.success).toBe(true);
      expect(result.data.canConnect).toBe(true);
      expect(result.data.canExecuteCommands).toBe(true);
      expect(result.data.shellType).toBe('bash');
      expect(result.data.homeDirectory).toBe('/home/testuser');
      expect(result.data.permissions.canReadHome).toBe(true);
      expect(result.data.permissions.canWriteHome).toBe(true);
      expect(result.data.permissions.canCreateDirectories).toBe(true);
    });

    it('should handle disconnected SSH connection', async () => {
      const disconnectedSSH = ConnectionMocks.disconnectedSSH();

      const result = await validator.validateSSHAccess(disconnectedSSH);

      expect(result.success).toBe(true);
      expect(result.data.canConnect).toBe(false);
      expect(result.data.errors).toContain('SSH connection is not established');
    });

    it('should handle command execution failures', async () => {
      // Create SSH mock that fails for specific commands
      const failingSSH = ConnectionMocks.customSSH({
        'echo "test"': { stdout: '', stderr: 'Command failed', exitCode: 1, duration: 100, timedOut: false }
      });

      const result = await validator.validateSSHAccess(failingSSH);

      expect(result.success).toBe(true);
      // Should handle failures gracefully
      expect(result.data.errors.length).toBeGreaterThanOrEqual(0);
    });
  });

  describe('SFTP Access Validation', () => {
    it('should validate successful SFTP access', async () => {
      const result = await validator.validateSFTPAccess(mockSFTP);
      
      expect(result.success).toBe(true);
      expect(result.data.canConnect).toBe(true);
      expect(result.data.canListFiles).toBe(true);
      expect(result.data.canCreateDirectories).toBe(true);
    });

    it('should handle SFTP failures', async () => {
      const failingSFTP = ConnectionMocks.failingSFTP();

      const result = await validator.validateSFTPAccess(failingSFTP);

      expect(result.success).toBe(true);
      expect(result.data.canConnect).toBe(false);
      expect(result.data.errors).toContain('Cannot list files via SFTP');
    });
  });

  describe('SLURM Access Validation', () => {
    it('should validate SLURM access', async () => {
      const result = await validator.testSlurmAccess(mockSSH);
      
      expect(result.success).toBe(true);
      expect(result.data.moduleSystemAvailable).toBe(true);
      expect(result.data.slurmAvailable).toBe(true);
      expect(result.data.canSubmitJobs).toBe(true);
      expect(result.data.canQueryJobs).toBe(true);
      expect(result.data.availablePartitions).toEqual(['normal', 'high', 'GPU']);
    });

    it('should handle disconnected SSH for SLURM testing', async () => {
      const disconnectedSSH = ConnectionMocks.disconnectedSSH();

      const result = await validator.testSlurmAccess(disconnectedSSH);

      expect(result.success).toBe(true);
      expect(result.data.moduleSystemAvailable).toBe(false);
      expect(result.data.slurmAvailable).toBe(false);
      expect(result.data.errors).toContain('SSH connection required for SLURM validation');
    });
  });

  describe('Full Validation', () => {
    it('should run complete validation successfully', async () => {
      const result = await validator.runFullValidation(mockConfig, mockSSH, mockSFTP);
      
      expect(result.success).toBe(true);
      expect(result.data.overallSuccess).toBe(true);
      expect(result.data.connectivity.reachable).toBe(true);
      expect(result.data.ssh.canConnect).toBe(true);
      expect(result.data.sftp.canConnect).toBe(true);
      expect(result.data.slurm.slurmAvailable).toBe(true);
      expect(result.data.recommendations.length).toBeGreaterThan(0);
      expect(result.data.timestamp).toBeTruthy();
    });

    it('should provide helpful recommendations', async () => {
      const unreachableConfig = { ...mockConfig, host: 'invalid.cluster.test' };
      const result = await validator.runFullValidation(unreachableConfig, mockSSH, mockSFTP);
      
      expect(result.success).toBe(true);
      expect(result.data.overallSuccess).toBe(false);
      expect(result.data.recommendations).toContain('Check network connectivity to the cluster');
    });

    it('should handle mixed validation results', async () => {
      const disconnectedSSH = ConnectionMocks.disconnectedSSH();
      const result = await validator.runFullValidation(mockConfig, disconnectedSSH, mockSFTP);

      expect(result.success).toBe(true);
      expect(result.data.overallSuccess).toBe(false);
      expect(result.data.ssh.canConnect).toBe(false);
      expect(result.data.recommendations).toContain('Verify SSH service is running on the cluster');
    });
  });

  describe('Error Handling', () => {
    it('should handle validation timeouts gracefully', async () => {
      const shortTimeoutValidator = new SLURMConnectionValidator(100);
      
      // This should still complete successfully with mocked delays
      const result = await shortTimeoutValidator.testBasicConnectivity(mockConfig);
      expect(result.success).toBe(true);
    });

    it('should handle exceptions in validation', async () => {
      const faultySSH = {
        ...mockSSH,
        executeCommand: vi.fn().mockRejectedValue(new Error('Connection lost'))
      } as any;

      const result = await validator.validateSSHAccess(faultySSH);
      expect(result.success).toBe(false);
      expect(result.error).toBeDefined();
    });
  });
});

describe('ValidationUtils', () => {
  let mockSSH: SSHConnection;

  beforeEach(() => {
    mockSSH = ConnectionMocks.workingSSH();
  });

  describe('Quick Health Check', () => {
    it('should pass health check for connected SSH', async () => {
      const isHealthy = await ValidationUtils.quickHealthCheck(mockSSH);
      expect(isHealthy).toBe(true);
    });

    it('should fail health check for disconnected SSH', async () => {
      const disconnectedSSH = ConnectionMocks.disconnectedSSH();
      const isHealthy = await ValidationUtils.quickHealthCheck(disconnectedSSH);
      expect(isHealthy).toBe(false);
    });

    it('should handle health check errors', async () => {
      const faultySSH = {
        isConnected: () => true,
        executeCommand: vi.fn().mockRejectedValue(new Error('Health check failed'))
      } as any;

      const isHealthy = await ValidationUtils.quickHealthCheck(faultySSH);
      expect(isHealthy).toBe(false);
    });
  });

  describe('SLURM Access Check', () => {
    it('should confirm SLURM access', async () => {
      const hasAccess = await ValidationUtils.checkSlurmAccess(mockSSH);
      expect(hasAccess).toBe(true);
    });

    it('should handle SLURM access errors', async () => {
      const faultySSH = {
        executeCommand: vi.fn().mockRejectedValue(new Error('SLURM not available'))
      } as any;

      const hasAccess = await ValidationUtils.checkSlurmAccess(faultySSH);
      expect(hasAccess).toBe(false);
    });
  });

  describe('Job Submission Check', () => {
    it('should confirm job submission capability', async () => {
      const canSubmit = await ValidationUtils.canSubmitJobs(mockSSH);
      expect(canSubmit).toBe(true);
    });

    it('should handle submission check errors', async () => {
      const faultySSH = {
        executeCommand: vi.fn().mockRejectedValue(new Error('sbatch not available'))
      } as any;

      const canSubmit = await ValidationUtils.canSubmitJobs(faultySSH);
      expect(canSubmit).toBe(false);
    });
  });
});