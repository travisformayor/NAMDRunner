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
  | 'connection_issues'; // Basic connection problems

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

  // Connection fixtures
  getConnectionFixture(type: 'success' | 'failure' | 'timeout' = 'success') {
    switch (type) {
      case 'success':
        return connectionFixtures.successfulConnection;
      case 'failure':
        return connectionFixtures.wrongPassword;
      case 'timeout':
        return connectionFixtures.connectionTimeout;
      default:
        return connectionFixtures.successfulConnection;
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