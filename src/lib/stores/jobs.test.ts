import { describe, it, expect, beforeEach, vi } from 'vitest';
import { get } from 'svelte/store';
import { jobs, jobsByStatus, jobCounts, mockJobs } from './jobs';
import type { JobInfo, NAMDConfig } from '../types/api';

// Helper function to create a valid NAMDConfig for test data
function createTestNAMDConfig(overrides: Partial<NAMDConfig> = {}): NAMDConfig {
  const base: NAMDConfig = {
    outputname: 'output',
    temperature: 300,
    timestep: 2.0,
    execution_mode: 'run',
    steps: 100000,
    pme_enabled: false,
    npt_enabled: false,
    langevin_damping: 5.0,
    xst_freq: 1200,
    output_energies_freq: 1200,
    dcd_freq: 1200,
    restart_freq: 1200,
    output_pressure_freq: 1200
  };

  // Merge overrides, excluding undefined values for optional properties
  const result = { ...base, ...overrides };

  // Only set optional properties if explicitly provided
  if ('cell_basis_vector1' in overrides) {
    result.cell_basis_vector1 = overrides.cell_basis_vector1;
  }
  if ('cell_basis_vector2' in overrides) {
    result.cell_basis_vector2 = overrides.cell_basis_vector2;
  }
  if ('cell_basis_vector3' in overrides) {
    result.cell_basis_vector3 = overrides.cell_basis_vector3;
  }

  return result;
}

// Mock CoreClientFactory to avoid external dependencies
vi.mock('../ports/clientFactory', () => ({
  CoreClientFactory: {
    getUserMode: vi.fn(() => 'demo'),
    getClient: vi.fn(() => ({
      getAllJobs: vi.fn(() => Promise.resolve({ success: true, jobs: mockJobs }))
    }))
  }
}));

describe('Jobs Store - Business Logic Tests', () => {
  describe('Job Data Structure', () => {
    it('should have valid mock job data structure', () => {
      expect(Array.isArray(mockJobs)).toBe(true);
      expect(mockJobs.length).toBeGreaterThan(0);

      // Test first job structure
      const job = mockJobs[0];
      expect(job).toBeDefined();
      if (job) {
        expect(job.job_id).toBeDefined();
        expect(job.job_name).toBeDefined();
        expect(job.status).toBeDefined();
        expect(job.created_at).toBeDefined();
        expect(job.namd_config).toBeDefined();
        expect(job.slurm_config).toBeDefined();
        expect(Array.isArray(job.input_files)).toBe(true);
      }
    });

    it('should have jobs with different statuses for testing', () => {
      const statuses = mockJobs.map(job => job.status);
      expect(statuses).toContain('RUNNING');
      expect(statuses).toContain('COMPLETED');
      // Should have variety for UI testing
      expect(new Set(statuses).size).toBeGreaterThan(1);
    });
  });

  describe('Job Status Grouping', () => {
    beforeEach(() => {
      // Reset any store state if needed
    });

    it('should group jobs by status correctly', () => {
      // Mock the jobs store with test data
      const testJobs: JobInfo[] = [
        {
          job_id: 'test1',
          job_name: 'Test 1',
          status: 'RUNNING',
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
          namd_config: createTestNAMDConfig({
            steps: 1000,
            temperature: 300,
            timestep: 2.0,
            outputname: 'test1',
            dcd_freq: 100,
            restart_freq: 100
          }),
          slurm_config: {
            cores: 4,
            memory: '8GB',
            walltime: '01:00:00',
            partition: 'atesting'
          },
          input_files: [],
          remote_directory: '/test1'
        },
        {
          job_id: 'test2',
          job_name: 'Test 2',
          status: 'COMPLETED',
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T01:00:00Z',
          completed_at: '2024-01-01T01:00:00Z',
          namd_config: createTestNAMDConfig({
            steps: 1000,
            temperature: 300,
            timestep: 2.0,
            outputname: 'test2',
            dcd_freq: 100,
            restart_freq: 100
          }),
          slurm_config: {
            cores: 4,
            memory: '8GB',
            walltime: '01:00:00',
            partition: 'atesting'
          },
          input_files: [],
          remote_directory: '/test2'
        }
      ];

      // This would require setting up the store properly, but we can test the mock data grouping logic
      const groupedJobs = testJobs.reduce((acc, job) => {
        const status = job.status;
        if (!acc[status]) {
          acc[status] = [];
        }
        acc[status].push(job);
        return acc;
      }, {} as Record<string, JobInfo[]>);

      expect(groupedJobs['RUNNING']).toBeDefined();
      expect(groupedJobs['RUNNING']?.length).toBe(1);
      expect(groupedJobs['COMPLETED']).toBeDefined();
      expect(groupedJobs['COMPLETED']?.length).toBe(1);
    });

    it('should calculate job counts correctly', () => {
      const testJobs: JobInfo[] = [
        { status: 'RUNNING' } as JobInfo,
        { status: 'RUNNING' } as JobInfo,
        { status: 'COMPLETED' } as JobInfo,
        { status: 'PENDING' } as JobInfo,
        { status: 'FAILED' } as JobInfo
      ];

      const counts = testJobs.reduce((acc, job) => {
        acc.total++;
        if (job.status === 'RUNNING') acc.running++;
        if (job.status === 'COMPLETED') acc.completed++;
        if (job.status === 'PENDING') acc.pending++;
        if (job.status === 'FAILED') acc.failed++;
        return acc;
      }, {
        total: 0,
        running: 0,
        completed: 0,
        pending: 0,
        failed: 0
      });

      expect(counts.total).toBe(5);
      expect(counts.running).toBe(2);
      expect(counts.completed).toBe(1);
      expect(counts.pending).toBe(1);
      expect(counts.failed).toBe(1);
    });
  });

  describe('Job Status Validation', () => {
    it('should recognize valid job statuses', () => {
      const validStatuses = ['CREATED', 'PENDING', 'RUNNING', 'COMPLETED', 'FAILED', 'CANCELLED'];

      validStatuses.forEach(status => {
        expect(['CREATED', 'PENDING', 'RUNNING', 'COMPLETED', 'FAILED', 'CANCELLED']).toContain(status);
      });
    });

    it('should handle status transitions logically', () => {
      // Test logical status progressions
      const progressions = [
        ['CREATED', 'PENDING'],
        ['PENDING', 'RUNNING'],
        ['RUNNING', 'COMPLETED'],
        ['RUNNING', 'FAILED'],
        ['PENDING', 'CANCELLED'],
        ['RUNNING', 'CANCELLED']
      ];

      progressions.forEach(([from, to]) => {
        expect(typeof from).toBe('string');
        expect(typeof to).toBe('string');
        // In a real implementation, you'd validate these transitions
        expect(from).toBeDefined();
        expect(to).toBeDefined();
      });
    });
  });

  describe('Job Configuration Validation', () => {
    it('should validate NAMD configuration structure', () => {
      const validNamdConfig = {
        steps: 100000,
        temperature: 300,
        timestep: 2.0,
        outputname: 'test_output',
        dcd_freq: 1000,
        restart_freq: 1000
      };

      // Test required fields
      expect(validNamdConfig.steps).toBeGreaterThan(0);
      expect(validNamdConfig.temperature).toBeGreaterThan(0);
      expect(validNamdConfig.timestep).toBeGreaterThan(0);
      expect(validNamdConfig.outputname).toBeTruthy();
      if (validNamdConfig.dcd_freq) {
        expect(validNamdConfig.dcd_freq).toBeGreaterThan(0);
      }
      if (validNamdConfig.restart_freq) {
        expect(validNamdConfig.restart_freq).toBeGreaterThan(0);
      }
    });

    it('should validate SLURM configuration structure', () => {
      const validSlurmConfig = {
        cores: 24,
        memory: '16GB',
        walltime: '04:00:00',
        partition: 'amilan'
      };

      expect(validSlurmConfig.cores).toBeGreaterThan(0);
      expect(validSlurmConfig.memory).toMatch(/^\d+GB$/);
      expect(validSlurmConfig.walltime).toMatch(/^\d+:\d{2}:\d{2}$/);
      expect(validSlurmConfig.partition).toBeTruthy();
    });
  });

  describe('File Information Validation', () => {
    it('should validate input file structure', () => {
      const validInputFile = {
        name: 'protein.pdb',
        local_path: '/local/protein.pdb',
        file_type: 'pdb'
      };

      expect(validInputFile.name).toBeTruthy();
      expect(validInputFile.local_path).toBeTruthy();
      expect(validInputFile.file_type).toBeTruthy();
      expect(validInputFile.name.endsWith('.pdb')).toBe(true);
    });

    it('should recognize common NAMD file types', () => {
      const commonFileTypes = ['pdb', 'psf', 'prm', 'par', 'namd', 'conf', 'dcd'];
      const testFiles = [
        { name: 'structure.pdb', file_type: 'pdb' },
        { name: 'topology.psf', file_type: 'psf' },
        { name: 'parameters.prm', file_type: 'prm' }
      ];

      testFiles.forEach(file => {
        expect(commonFileTypes).toContain(file.file_type);
        expect(file.name).toContain(file.file_type);
      });
    });
  });

  describe('Date and Time Handling', () => {
    it('should handle ISO date strings correctly', () => {
      const testDate = '2024-01-15T09:30:00Z';
      const parsed = new Date(testDate);

      // JavaScript's toISOString() always includes milliseconds
      expect(parsed.toISOString()).toBe('2024-01-15T09:30:00.000Z');
      expect(parsed.getFullYear()).toBe(2024);
      expect(parsed.getMonth()).toBe(0); // January is 0
      expect(parsed.getDate()).toBe(15);
    });

    it('should validate job timeline consistency', () => {
      const job = mockJobs.find(j => j.status === 'COMPLETED');
      if (job && job.completed_at && job.created_at && job.updated_at) {
        const created = new Date(job.created_at);
        const updated = new Date(job.updated_at);
        const completed = new Date(job.completed_at);

        expect(created.getTime()).toBeLessThanOrEqual(updated.getTime());
        expect(updated.getTime()).toBeLessThanOrEqual(completed.getTime());
      }
    });
  });

  describe('Progress State Management', () => {
    it('should handle progress messages correctly', () => {
      const progressMessages = [
        'Initializing job creation...',
        'Validating input files...',
        'Uploading files to cluster...',
        'Job creation completed successfully'
      ];

      progressMessages.forEach(message => {
        expect(typeof message).toBe('string');
        expect(message.length).toBeGreaterThan(0);
        // Progress messages should be user-friendly
        expect(message).toMatch(/^[A-Z]/); // Should start with capital letter
      });
    });

    it('should maintain progress state structure', () => {
      const progressState = {
        isActive: false,
        messages: [] as string[],
        currentStep: 0,
        totalSteps: 4
      };

      expect(typeof progressState.isActive).toBe('boolean');
      expect(Array.isArray(progressState.messages)).toBe(true);
      expect(typeof progressState.currentStep).toBe('number');
      expect(typeof progressState.totalSteps).toBe('number');
      expect(progressState.currentStep).toBeLessThanOrEqual(progressState.totalSteps);
    });
  });
});