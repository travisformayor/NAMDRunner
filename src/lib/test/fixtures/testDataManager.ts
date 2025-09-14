import { jobFixtures, jobProgressionScenarios, generateTestJobBatch } from './jobFixtures';
import { connectionFixtures, sessionStateFixtures, disconnectFixtures } from './sessionFixtures';
import { uploadFixtures, downloadFixtures, listFilesFixtures } from './fileFixtures';
import { 
  sbatchFixtures, 
  squeueFixtures, 
  sacctFixtures, 
  slurmCommandScenarios 
} from './slurmFixtures';

import type { JobInfo, ConnectionState, SessionInfo } from '../../types/api';

/**
 * Centralized test data manager for coordinating fixtures across the application
 * Provides scenario-based testing with consistent mock behavior
 */

export type TestScenario = 
  | 'clean_slate'        // Reset to clean state
  | 'basic_workflow'     // User with jobs in basic states
  | 'connection_issues'  // Basic connection problems
  | 'state_transitions'  // Connection state transition testing
  | 'error_recovery'     // Error handling and recovery scenarios
  | 'session_expiry'     // Session expiration testing
  | 'network_instability'; // Intermittent connection issues

export interface TestScenarioConfig {
  name: string;
  description: string;
  connectionState: ConnectionState;
  sessionInfo?: SessionInfo;
  jobs: JobInfo[];
  mockBehavior: {
    connectionLatency: number;
    fileOperationLatency: number;
    slurmLatency: number;
    errorRate: number; // 0-1, percentage of operations that should fail
  };
}

// Predefined test scenarios - simplified for practical development
export const testScenarios: Record<TestScenario, TestScenarioConfig> = {
  clean_slate: {
    name: 'Clean Slate',
    description: 'Reset to clean state for fresh testing',
    connectionState: 'Disconnected',
    jobs: [],
    mockBehavior: {
      connectionLatency: 200,
      fileOperationLatency: 150,
      slurmLatency: 100,
      errorRate: 0,
    },
  },

  basic_workflow: {
    name: 'Basic Workflow',
    description: 'User with jobs in basic states for testing core functionality',
    connectionState: 'Connected',
    sessionInfo: {
      host: 'mock.cluster.test',
      username: 'mockuser',
      connectedAt: new Date(Date.now() - 30 * 60 * 1000).toISOString(),
    },
    jobs: [
      jobFixtures.freshJob.jobInfo,
      jobFixtures.runningJob.jobInfo,
      jobFixtures.completedJob.jobInfo,
    ],
    mockBehavior: {
      connectionLatency: 300,
      fileOperationLatency: 200,
      slurmLatency: 150,
      errorRate: 0.05, // Minimal error rate for basic testing
    },
  },

  connection_issues: {
    name: 'Connection Issues',
    description: 'Basic connection problems for error handling testing',
    connectionState: 'Expired',
    sessionInfo: {
      host: 'mock.cluster.test',
      username: 'mockuser',
      connectedAt: new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString(),
    },
    jobs: [
      jobFixtures.pendingJob.jobInfo,
    ],
    mockBehavior: {
      connectionLatency: 1000,
      fileOperationLatency: 800,
      slurmLatency: 600,
      errorRate: 0.2, // 20% error rate for connection testing
    },
  },

  state_transitions: {
    name: 'State Transitions',
    description: 'Test connection state machine transitions and validation',
    connectionState: 'Disconnected',
    jobs: [],
    mockBehavior: {
      connectionLatency: 500,
      fileOperationLatency: 300,
      slurmLatency: 200,
      errorRate: 0.1, // Some errors to test recovery
    },
  },

  error_recovery: {
    name: 'Error Recovery',
    description: 'Test error handling and automatic recovery mechanisms',
    connectionState: 'Disconnected',
    jobs: [
      jobFixtures.freshJob.jobInfo,
    ],
    mockBehavior: {
      connectionLatency: 800,
      fileOperationLatency: 600,
      slurmLatency: 400,
      errorRate: 0.3, // High error rate for testing recovery
    },
  },

  session_expiry: {
    name: 'Session Expiry',
    description: 'Test session expiration and refresh scenarios',
    connectionState: 'Connected',
    sessionInfo: {
      host: 'mock.cluster.test',
      username: 'mockuser',
      connectedAt: new Date(Date.now() - 4 * 60 * 60 * 1000).toISOString(), // 4 hours ago (expired)
    },
    jobs: [
      jobFixtures.runningJob.jobInfo,
    ],
    mockBehavior: {
      connectionLatency: 600,
      fileOperationLatency: 400,
      slurmLatency: 300,
      errorRate: 0.15,
    },
  },

  network_instability: {
    name: 'Network Instability',
    description: 'Test intermittent connection issues and resilience',
    connectionState: 'Connected',
    sessionInfo: {
      host: 'unstable.cluster.test',
      username: 'mockuser',
      connectedAt: new Date(Date.now() - 45 * 60 * 1000).toISOString(), // 45 minutes ago
    },
    jobs: [
      jobFixtures.runningJob.jobInfo,
      jobFixtures.pendingJob.jobInfo,
    ],
    mockBehavior: {
      connectionLatency: 1500, // High latency
      fileOperationLatency: 1200,
      slurmLatency: 800,
      errorRate: 0.4, // Very high error rate for instability testing
    },
  },
};

/**
 * Test data manager class for coordinating mock behavior
 */
export class TestDataManager {
  private currentScenario: TestScenario = 'clean_slate';
  private scenarioData: TestScenarioConfig;
  private jobProgressionIndex: number = 0;

  constructor() {
    this.scenarioData = testScenarios[this.currentScenario];
  }

  // Scenario management
  setScenario(scenario: TestScenario): void {
    this.currentScenario = scenario;
    this.scenarioData = testScenarios[scenario];
    this.jobProgressionIndex = 0;
    console.log(`Test scenario changed to: ${scenario}`);
  }

  getCurrentScenario(): TestScenario {
    return this.currentScenario;
  }

  getScenarioConfig(): TestScenarioConfig {
    return { ...this.scenarioData };
  }

  // Enhanced connection fixtures
  getConnectionFixture(type: 'success' | 'failure' | 'timeout' | 'network_error' | 'auth_error' = 'success') {
    switch (type) {
      case 'success':
        return connectionFixtures.successfulConnection;
      case 'failure':
      case 'auth_error':
        return connectionFixtures.wrongPassword;
      case 'timeout':
        return connectionFixtures.connectionTimeout;
      case 'network_error':
        return connectionFixtures.networkUnreachable || connectionFixtures.connectionTimeout;
      default:
        return connectionFixtures.successfulConnection;
    }
  }

  // Get connection parameters that will trigger specific behaviors
  getConnectionParams(behavior: 'success' | 'invalid_host' | 'timeout' | 'auth_fail'): {
    host: string;
    username: string;
    password: string;
  } {
    const baseParams = {
      username: 'testuser',
      password: 'testpass',
    };

    switch (behavior) {
      case 'invalid_host':
        return { ...baseParams, host: 'invalid.cluster.test' };
      case 'timeout':
        return { ...baseParams, host: 'timeout.cluster.test' };
      case 'auth_fail':
        return { ...baseParams, host: 'login.cluster.test', password: 'wrongpassword' };
      case 'success':
      default:
        return { ...baseParams, host: 'login.cluster.test' };
    }
  }

  getDisconnectFixture() {
    if (this.shouldSimulateError()) {
      return disconnectFixtures.networkErrorDuringDisconnect;
    }
    return disconnectFixtures.gracefulDisconnect;
  }

  // Job fixtures
  getJobFixture(jobId: string) {
    const job = this.scenarioData.jobs.find(j => j.jobId === jobId);
    if (job) return job;

    // Return a default job if not found
    return jobFixtures.freshJob.jobInfo;
  }

  getAllJobs(): JobInfo[] {
    return [...this.scenarioData.jobs];
  }

  // File operation fixtures
  getUploadFixture(successRate: number = 0.9) {
    if (Math.random() > successRate || this.shouldSimulateError()) {
      return uploadFixtures.uploadFailure;
    }
    return uploadFixtures.successfulUpload;
  }

  getDownloadFixture() {
    if (this.shouldSimulateError()) {
      return downloadFixtures.downloadFailure;
    }
    return downloadFixtures.successfulDownload;
  }

  getListFilesFixture() {
    if (this.shouldSimulateError()) {
      return listFilesFixtures.listFilesFailure;
    }
    return listFilesFixtures.standardJobFiles;
  }

  // SLURM command fixtures
  getSbatchFixture() {
    if (this.shouldSimulateError()) {
      return sbatchFixtures.submissionFailure;
    }
    return sbatchFixtures.successfulSubmission;
  }

  getSqueueFixture(jobId: string) {
    if (this.shouldSimulateError()) {
      return squeueFixtures.jobNotFound;
    }

    // Return appropriate fixture based on job state
    const job = this.getJobFixture(jobId);
    switch (job.status) {
      case 'PENDING':
        return squeueFixtures.pendingJob;
      case 'RUNNING':
        return squeueFixtures.runningJob;
      default:
        return squeueFixtures.jobNotFound;
    }
  }

  getSacctFixture(jobId: string) {
    if (this.shouldSimulateError()) {
      return sacctFixtures.jobNotFoundHistory;
    }

    const job = this.getJobFixture(jobId);
    switch (job.status) {
      case 'COMPLETED':
        return sacctFixtures.completedJob;
      case 'FAILED':
        return sacctFixtures.failedJob;
      case 'CANCELLED':
        return sacctFixtures.cancelledJob;
      default:
        return sacctFixtures.jobNotFoundHistory;
    }
  }

  // Job progression for testing state transitions
  advanceJobProgression(jobId: string): JobInfo | null {
    const job = this.getJobFixture(jobId);
    if (!job) return null;

    // Use successful progression scenario by default
    const progression = jobProgressionScenarios.successfulProgression;
    
    if (this.jobProgressionIndex < progression.length - 1) {
      this.jobProgressionIndex++;
      const nextState = progression[this.jobProgressionIndex];
      
      // Update the job in our scenario data
      const jobIndex = this.scenarioData.jobs.findIndex(j => j.jobId === jobId);
      if (jobIndex !== -1) {
        this.scenarioData.jobs[jobIndex] = { ...nextState, jobId };
        return this.scenarioData.jobs[jobIndex];
      }
    }

    return job;
  }

  // State transition testing
  getStateTransitionScenario(): Array<{
    from: ConnectionState;
    to: ConnectionState;
    shouldSucceed: boolean;
    reason: string;
  }> {
    return [
      { from: 'Disconnected', to: 'Connecting', shouldSucceed: true, reason: 'Valid initial connection' },
      { from: 'Connecting', to: 'Connected', shouldSucceed: true, reason: 'Successful authentication' },
      { from: 'Connecting', to: 'Disconnected', shouldSucceed: true, reason: 'Connection failed' },
      { from: 'Connected', to: 'Disconnected', shouldSucceed: true, reason: 'User disconnect' },
      { from: 'Connected', to: 'Expired', shouldSucceed: true, reason: 'Session timeout' },
      { from: 'Expired', to: 'Connecting', shouldSucceed: true, reason: 'Reconnection attempt' },
      { from: 'Expired', to: 'Disconnected', shouldSucceed: true, reason: 'Reset connection' },
      
      // Invalid transitions that should fail
      { from: 'Disconnected', to: 'Connected', shouldSucceed: false, reason: 'Cannot skip connecting state' },
      { from: 'Disconnected', to: 'Expired', shouldSucceed: false, reason: 'Cannot expire without connecting' },
      { from: 'Connecting', to: 'Expired', shouldSucceed: false, reason: 'Cannot expire during connection' },
      { from: 'Connected', to: 'Connecting', shouldSucceed: false, reason: 'Already connected' },
    ];
  }

  // Validation testing scenarios
  getValidationTestCases(): Array<{
    name: string;
    description: string;
    config: {
      host: string;
      username: string;
      password: string;
    };
    expectedResults: {
      connectivity: boolean;
      sshAccess: boolean;
      sftpAccess: boolean;
      slurmAccess: boolean;
    };
  }> {
    return [
      {
        name: 'Valid Cluster',
        description: 'All systems accessible and functional',
        config: {
          host: 'login.cluster.test',
          username: 'testuser',
          password: 'testpass'
        },
        expectedResults: {
          connectivity: true,
          sshAccess: true,
          sftpAccess: true,
          slurmAccess: true
        }
      },
      {
        name: 'Network Issue',
        description: 'Host unreachable due to network problems',
        config: {
          host: 'unreachable.cluster.test',
          username: 'testuser',
          password: 'testpass'
        },
        expectedResults: {
          connectivity: false,
          sshAccess: false,
          sftpAccess: false,
          slurmAccess: false
        }
      },
      {
        name: 'Authentication Failure',
        description: 'Invalid credentials provided',
        config: {
          host: 'login.cluster.test',
          username: 'testuser',
          password: 'wrongpass'
        },
        expectedResults: {
          connectivity: true,
          sshAccess: false,
          sftpAccess: false,
          slurmAccess: false
        }
      },
      {
        name: 'SLURM Unavailable',
        description: 'SSH/SFTP work but SLURM not available',
        config: {
          host: 'no-slurm.cluster.test',
          username: 'testuser',
          password: 'testpass'
        },
        expectedResults: {
          connectivity: true,
          sshAccess: true,
          sftpAccess: true,
          slurmAccess: false
        }
      }
    ];
  }

  // Error recovery test scenarios
  getErrorRecoveryScenarios(): Array<{
    errorType: string;
    shouldRetry: boolean;
    maxRetries: number;
    recoveryAction: string;
  }> {
    return [
      {
        errorType: 'network_timeout',
        shouldRetry: true,
        maxRetries: 3,
        recoveryAction: 'retry_with_backoff'
      },
      {
        errorType: 'authentication_failed',
        shouldRetry: false,
        maxRetries: 0,
        recoveryAction: 'request_new_credentials'
      },
      {
        errorType: 'session_expired',
        shouldRetry: true,
        maxRetries: 1,
        recoveryAction: 'refresh_session'
      },
      {
        errorType: 'permission_denied',
        shouldRetry: false,
        maxRetries: 0,
        recoveryAction: 'contact_administrator'
      },
      {
        errorType: 'connection_dropped',
        shouldRetry: true,
        maxRetries: 5,
        recoveryAction: 'reconnect'
      }
    ];
  }

  // Utility methods
  private shouldSimulateError(): boolean {
    return Math.random() < this.scenarioData.mockBehavior.errorRate;
  }

  getLatency(operation: 'connection' | 'file' | 'slurm'): number {
    const base = this.scenarioData.mockBehavior[`${operation}Latency`];
    // Add some random variation (±20%)
    const variation = base * 0.2 * (Math.random() - 0.5);
    return Math.max(100, base + variation);
  }

  // Reset methods for clean test runs
  reset(): void {
    this.setScenario('clean_slate');
  }

  resetToScenario(scenario: TestScenario): void {
    this.setScenario(scenario);
  }

  // Export current state for debugging
  exportState(): {
    scenario: TestScenario;
    config: TestScenarioConfig;
    progressionIndex: number;
  } {
    return {
      scenario: this.currentScenario,
      config: this.getScenarioConfig(),
      progressionIndex: this.jobProgressionIndex,
    };
  }

  // Import state for testing
  importState(state: {
    scenario: TestScenario;
    config: TestScenarioConfig;
    progressionIndex?: number;
  }): void {
    this.currentScenario = state.scenario;
    this.scenarioData = state.config;
    this.jobProgressionIndex = state.progressionIndex || 0;
  }

  // Get all available scenarios for UI selection
  static getAvailableScenarios(): Array<{ key: TestScenario; name: string; description: string }> {
    return Object.entries(testScenarios).map(([key, config]) => ({
      key: key as TestScenario,
      name: config.name,
      description: config.description,
    }));
  }
}

// Singleton instance for use across the application
export const testDataManager = new TestDataManager();

// Development-only helper for scenario switching
export function switchTestScenario(scenario: TestScenario): void {
  if (process.env.NODE_ENV === 'development') {
    testDataManager.setScenario(scenario);
  }
}

// Helper for creating custom test scenarios
export function createCustomScenario(
  name: string,
  config: Partial<TestScenarioConfig>
): TestScenarioConfig {
  return {
    name,
    description: config.description || 'Custom test scenario',
    connectionState: config.connectionState || 'Disconnected',
    sessionInfo: config.sessionInfo,
    jobs: config.jobs || [],
    mockBehavior: {
      connectionLatency: 500,
      fileOperationLatency: 300,
      slurmLatency: 200,
      errorRate: 0,
      ...config.mockBehavior,
    },
  };
}