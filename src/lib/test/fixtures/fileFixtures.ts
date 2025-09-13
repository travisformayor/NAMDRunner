import type { 
  FileUpload, 
  UploadResult, 
  DownloadResult, 
  ListFilesResult,
  JobFile,
  JobId 
} from '../../types/api';

/**
 * File operation fixture utilities for testing
 * Covers SFTP operations, file transfers, and directory management scenarios
 */

export interface FileOperationScenario<T = any> {
  name: string;
  description: string;
  jobId: JobId;
  expectedResult: T;
  expectedDelay?: number;
}

// Common file types and sizes for realistic testing
export const commonFiles = {
  // NAMD input files
  structure_pdb: {
    name: 'structure.pdb',
    size: 15420,
    type: 'pdb' as const,
    content: 'HEADER    MEMBRANE PROTEIN                        01-JAN-25\nATOM      1  N   MET A   1      -8.901   4.127  -0.555  1.00 11.99           N\n',
  },
  
  structure_psf: {
    name: 'structure.psf',
    size: 45890,
    type: 'psf' as const,
    content: 'PSF CMAP CHEQ\n\n       3 !NTITLE\n* SYSTEM BUILT USING CHARMM-GUI\n',
  },
  
  parameters_prm: {
    name: 'parameters.prm',
    size: 8934,
    type: 'prm' as const,
    content: '* CHARMM36 All-Hydrogen Parameter File for Proteins and Nucleic Acids\n',
  },
  
  // NAMD configuration files
  config_namd: {
    name: 'config.namd',
    size: 2048,
    type: 'config' as const,
    content: '# NAMD configuration file\nstructure          structure.psf\ncoordinates        structure.pdb\n',
  },
  
  // SLURM script
  job_sbatch: {
    name: 'job.sbatch',
    size: 1024,
    type: 'config' as const,
    content: '#!/bin/bash\n#SBATCH --job-name=namd_simulation\n#SBATCH --nodes=1\n#SBATCH --ntasks-per-node=24\n',
  },
  
  // Output files
  slurm_output: {
    name: 'test_simulation_12345678.out',
    size: 25600,
    type: 'log' as const,
    content: 'Info: NAMD 3.0alpha13 for Linux-x86_64-multicore\nInfo: Built Tue Aug  1 15:30:56 CDT 2023\n',
  },
  
  slurm_error: {
    name: 'test_simulation_12345678.err',
    size: 512,
    type: 'log' as const,
    content: 'Warning: No output files generated\n',
  },
  
  namd_log: {
    name: 'namd_output.log',
    size: 128000,
    type: 'log' as const,
    content: 'NAMD version 3.0alpha13 built on 2023-08-01\nRunning on 24 processors\n',
  },
  
  trajectory: {
    name: 'output.dcd',
    size: 52428800, // 50 MB binary file
    type: 'trajectory' as const,
    content: null, // Binary file, no text content
  },
  
};

// File upload scenarios - simplified for practical testing
export const uploadFixtures: Record<string, FileOperationScenario<UploadResult>> = {
  successfulUpload: {
    name: 'Successful Upload',
    description: 'All files uploaded successfully',
    jobId: 'job_001',
    expectedResult: {
      success: true,
      uploadedFiles: ['structure.pdb', 'structure.psf', 'parameters.prm'],
    },
    expectedDelay: 400,
  },

  uploadFailure: {
    name: 'Upload Failure',
    description: 'Upload fails due to common issues',
    jobId: 'job_002',
    expectedResult: {
      success: false,
      uploadedFiles: [],
      failedUploads: [
        {
          fileName: 'structure.pdb',
          error: 'Upload failed: Network timeout',
        },
      ],
    },
    expectedDelay: 800,
  },
};

// File download scenarios - simplified for practical testing
export const downloadFixtures: Record<string, FileOperationScenario<DownloadResult>> = {
  successfulDownload: {
    name: 'Successful Download',
    description: 'File downloaded successfully',
    jobId: 'job_001',
    expectedResult: {
      success: true,
      content: commonFiles.slurm_output.content,
      filePath: '/tmp/test_simulation_12345678.out',
      fileSize: commonFiles.slurm_output.size,
    },
    expectedDelay: 300,
  },

  downloadFailure: {
    name: 'Download Failure',
    description: 'Download fails due to common issues',
    jobId: 'job_002',
    expectedResult: {
      success: false,
      error: 'File not found: output.log does not exist in job directory',
    },
    expectedDelay: 200,
  },
};

// File listing scenarios - simplified for practical testing
export const listFilesFixtures: Record<string, FileOperationScenario<ListFilesResult>> = {
  standardJobFiles: {
    name: 'Standard Job Files',
    description: 'List files for a typical NAMD job',
    jobId: 'job_001',
    expectedResult: {
      success: true,
      files: [
        {
          name: 'config.namd',
          size: commonFiles.config_namd.size,
          modifiedAt: new Date(Date.now() - 60000).toISOString(),
          fileType: 'config',
        },
        {
          name: 'job.sbatch',
          size: commonFiles.job_sbatch.size,
          modifiedAt: new Date(Date.now() - 120000).toISOString(),
          fileType: 'config',
        },
        {
          name: 'output.log',
          size: commonFiles.namd_log.size,
          modifiedAt: new Date(Date.now() - 300000).toISOString(),
          fileType: 'log',
        },
        {
          name: 'output.dcd',
          size: commonFiles.trajectory.size,
          modifiedAt: new Date(Date.now() - 600000).toISOString(),
          fileType: 'trajectory',
        },
      ],
    },
    expectedDelay: 200,
  },

  listFilesFailure: {
    name: 'List Files Failure',
    description: 'File listing fails due to common issues',
    jobId: 'job_002',
    expectedResult: {
      success: false,
      error: 'Job job_002 not found',
    },
    expectedDelay: 150,
  },
};

// Generate file upload objects for testing
export function createFileUpload(fileName: keyof typeof commonFiles, overrides?: Partial<FileUpload>): FileUpload {
  const file = commonFiles[fileName];
  return {
    localPath: `/tmp/${file.name}`,
    remoteName: file.name,
    ...overrides,
  };
}

// Generate batch of file uploads for testing
export function createFileUploadBatch(fileNames: (keyof typeof commonFiles)[]): FileUpload[] {
  return fileNames.map(fileName => createFileUpload(fileName));
}

// Create a realistic job file listing
export function createJobFileListing(jobId: JobId, includeOutputs: boolean = false): JobFile[] {
  const baseFiles: JobFile[] = [
    {
      name: 'config.namd',
      size: commonFiles.config_namd.size,
      modifiedAt: new Date(Date.now() - 3600000).toISOString(), // 1 hour ago
      fileType: 'config',
    },
    {
      name: 'job.sbatch',
      size: commonFiles.job_sbatch.size,
      modifiedAt: new Date(Date.now() - 3600000).toISOString(),
      fileType: 'config',
    },
  ];

  if (includeOutputs) {
    baseFiles.push(
      {
        name: `${jobId}_output.out`,
        size: Math.floor(Math.random() * 50000) + 10000,
        modifiedAt: new Date(Date.now() - Math.random() * 1800000).toISOString(), // Random within last 30 min
        fileType: 'log',
      },
      {
        name: `${jobId}_output.err`,
        size: Math.floor(Math.random() * 1000) + 100,
        modifiedAt: new Date(Date.now() - Math.random() * 1800000).toISOString(),
        fileType: 'log',
      },
      {
        name: 'output.dcd',
        size: Math.floor(Math.random() * 1000000000) + 10000000, // 10MB - 1GB
        modifiedAt: new Date(Date.now() - Math.random() * 1800000).toISOString(),
        fileType: 'trajectory',
      }
    );
  }

  return baseFiles;
}

// Helper to generate realistic file sizes
export function generateRealisticFileSize(fileType: 'pdb' | 'psf' | 'prm' | 'config' | 'log' | 'trajectory'): number {
  const sizeRanges = {
    pdb: [1000, 50000],        // 1KB - 50KB
    psf: [10000, 200000],      // 10KB - 200KB  
    prm: [5000, 20000],        // 5KB - 20KB
    config: [500, 5000],       // 500B - 5KB
    log: [1000, 500000],       // 1KB - 500KB
    trajectory: [10000000, 2147483648], // 10MB - 2GB
  };
  
  const [min, max] = sizeRanges[fileType];
  return Math.floor(Math.random() * (max - min)) + min;
}

// Validate file operation parameters
export function validateFileUpload(upload: FileUpload): { valid: boolean; errors: string[] } {
  const errors: string[] = [];
  
  if (!upload.localPath?.trim()) {
    errors.push('Local path is required');
  }
  
  if (!upload.remoteName?.trim()) {
    errors.push('Remote name is required');
  }
  
  // Check for path traversal attempts
  if (upload.remoteName?.includes('..') || upload.remoteName?.includes('/')) {
    errors.push('Invalid remote name: Path traversal not allowed');
  }
  
  return {
    valid: errors.length === 0,
    errors,
  };
}

// Generate mock file content based on type
export function generateMockFileContent(fileName: string, fileType: string): string {
  switch (fileType) {
    case 'log':
      return `[${new Date().toISOString()}] INFO: Mock log content for ${fileName}\nExecution completed successfully\n`;
    case 'config':
      return `# Configuration file: ${fileName}\n# Generated at ${new Date().toISOString()}\n`;
    case 'output':
      return `NAMD simulation output for ${fileName}\nTimestep 0: Energy = -12345.67\nTimestep 1000: Energy = -12344.89\n`;
    default:
      return `Mock content for ${fileName}\nGenerated at ${new Date().toISOString()}\n`;
  }
}