import type { Result } from '../types/connection';
import type { ConnectionError } from '../types/errors';
import { ErrorBuilder } from '../types/errors';

/**
 * Remote path configuration for SLURM clusters
 * Centralized path templates and directory structure
 */
export const REMOTE_PATH_CONFIG = {
  // Base paths
  USER_BASE: '/projects/$USER',
  NAMDRUNNER_ROOT: '/projects/$USER/namdrunner_jobs',
  
  // Job directory structure
  JOB_TEMPLATE: '/projects/$USER/namdrunner_jobs/{jobId}',
  
  // Sub-directories within jobs
  LOGS_DIR: 'logs',
  INPUTS_DIR: 'inputs', 
  OUTPUTS_DIR: 'outputs',
  SCRATCH_DIR: 'scratch',
  
  // Standard files
  CONFIG_FILE: 'job.json',
  SLURM_SCRIPT: 'job.slurm',
  
  // Log files
  LOG_FILES: {
    stdout: 'job.out',
    stderr: 'job.err',
    slurm: 'slurm.log'
  }
} as const;

/**
 * Path variable substitution pattern
 */
interface PathVariables {
  [key: string]: string;
}

/**
 * Complete path information for a job
 */
export interface JobPaths {
  jobDir: string;
  logsDir: string;
  inputsDir: string;
  outputsDir: string;
  scratchDir: string;
  configFile: string;
  slurmScript: string;
}

/**
 * Extended job paths with log files
 */
export interface ExtendedJobPaths extends JobPaths {
  logFiles: {
    stdout: string;
    stderr: string;
    slurm: string;
  };
}

/**
 * Path resolution result with validation
 */
export interface PathResolutionResult {
  path: string;
  isValid: boolean;
  warnings: string[];
  sanitized: boolean;
}

/**
 * Centralized path resolver for all remote path operations
 * Handles path template expansion, validation, and sanitization
 */
export class PathResolver {
  private readonly config = REMOTE_PATH_CONFIG;

  /**
   * Expand a path template with variables
   */
  expandPath(template: string, variables: PathVariables): Result<PathResolutionResult> {
    try {
      let path = template;
      const warnings: string[] = [];
      let sanitized = false;

      // Substitute variables
      for (const [key, value] of Object.entries(variables)) {
        const sanitizedValue = this.sanitizePathComponent(value);
        if (sanitizedValue !== value) {
          sanitized = true;
          warnings.push(`Sanitized variable '${key}': '${value}' → '${sanitizedValue}'`);
        }
        
        // Support both $VAR and {VAR} patterns
        const patterns = [
          new RegExp(`\\$${key}\\b`, 'g'),
          new RegExp(`\\{${key}\\}`, 'g')
        ];
        
        for (const pattern of patterns) {
          path = path.replace(pattern, sanitizedValue);
        }
      }

      // Check for unresolved variables - warn but don't fail for basic templates
      const unresolvedVars = path.match(/(\$\w+|\{\w+\})/g);
      if (unresolvedVars) {
        warnings.push(`Unresolved variables: ${unresolvedVars.join(', ')}`);
        // For basic path operations, we should not fail - just note the issue
      }

      // Validate resulting path
      const isValid = this.validatePath(path);
      if (!isValid) {
        warnings.push('Generated path may not be valid on target system');
      }

      return {
        success: true,
        data: {
          path,
          isValid,
          warnings,
          sanitized
        }
      };
    } catch (error) {
      return {
        success: false,
        error: ErrorBuilder.fromError(error as Error, 'Validation')
      };
    }
  }

  /**
   * Get user base directory path
   */
  getUserBasePath(username: string): Result<string> {
    const result = this.expandPath(this.config.USER_BASE, { USER: username });
    return result.success 
      ? { success: true, data: result.data.path }
      : result;
  }

  /**
   * Get NAMDRunner root directory path
   */
  getNAMDRunnerRoot(username: string): Result<string> {
    const result = this.expandPath(this.config.NAMDRUNNER_ROOT, { USER: username });
    return result.success 
      ? { success: true, data: result.data.path }
      : result;
  }

  /**
   * Get job directory path
   */
  getJobDirectory(username: string, jobId: string): Result<string> {
    const result = this.expandPath(this.config.JOB_TEMPLATE, { 
      USER: username, 
      jobId: this.sanitizeJobId(jobId) 
    });
    return result.success 
      ? { success: true, data: result.data.path }
      : result;
  }

  /**
   * Get complete job paths structure
   */
  getJobPaths(username: string, jobId: string): Result<JobPaths> {
    const jobDirResult = this.getJobDirectory(username, jobId);
    if (!jobDirResult.success) {
      return jobDirResult;
    }

    const jobDir = jobDirResult.data;
    
    return {
      success: true,
      data: {
        jobDir,
        logsDir: `${jobDir}/${this.config.LOGS_DIR}`,
        inputsDir: `${jobDir}/${this.config.INPUTS_DIR}`,
        outputsDir: `${jobDir}/${this.config.OUTPUTS_DIR}`,
        scratchDir: `${jobDir}/${this.config.SCRATCH_DIR}`,
        configFile: `${jobDir}/${this.config.CONFIG_FILE}`,
        slurmScript: `${jobDir}/${this.config.SLURM_SCRIPT}`
      }
    };
  }

  /**
   * Get extended job paths with log file paths
   */
  getExtendedJobPaths(username: string, jobId: string): Result<ExtendedJobPaths> {
    const pathsResult = this.getJobPaths(username, jobId);
    if (!pathsResult.success) {
      return pathsResult;
    }

    const paths = pathsResult.data;
    
    return {
      success: true,
      data: {
        ...paths,
        logFiles: {
          stdout: `${paths.logsDir}/${this.config.LOG_FILES.stdout}`,
          stderr: `${paths.logsDir}/${this.config.LOG_FILES.stderr}`,
          slurm: `${paths.logsDir}/${this.config.LOG_FILES.slurm}`
        }
      }
    };
  }

  /**
   * Get specific log file path
   */
  getLogFilePath(username: string, jobId: string, logType: keyof typeof REMOTE_PATH_CONFIG.LOG_FILES): Result<string> {
    const pathsResult = this.getExtendedJobPaths(username, jobId);
    if (!pathsResult.success) {
      return pathsResult;
    }

    return {
      success: true,
      data: pathsResult.data.logFiles[logType]
    };
  }

  /**
   * Extract job ID from a path
   */
  extractJobIdFromPath(path: string): string | null {
    const match = path.match(/namdrunner_jobs\/([^\/]+)/);
    return match ? match[1] : null;
  }

  /**
   * Extract username from a path
   */
  extractUsernameFromPath(path: string): string | null {
    const match = path.match(/\/projects\/([^\/]+)/);
    return match ? match[1] : null;
  }

  /**
   * Validate if a path is safe and well-formed
   */
  validatePath(path: string): boolean {
    // Basic path validation rules
    const rules = [
      // Must be absolute path
      () => path.startsWith('/'),
      
      // No double slashes
      () => !path.includes('//'),
      
      // No dangerous patterns
      () => !path.includes('../'),
      () => !path.includes('./'),
      
      // No control characters
      () => !/[\x00-\x1f\x7f]/.test(path),
      
      // Reasonable length
      () => path.length > 0 && path.length < 1000,
      
      // No trailing slash (except root)
      () => path === '/' || !path.endsWith('/')
    ];

    return rules.every(rule => rule());
  }

  /**
   * Sanitize a path component for safe filesystem use
   */
  sanitizePathComponent(component: string): string {
    return component
      // Remove/replace dangerous characters and non-alphanumeric chars except underscore and hyphen
      .replace(/[^a-zA-Z0-9_-]/g, '_')
      // Remove multiple consecutive underscores
      .replace(/_+/g, '_')
      // Remove leading/trailing underscores
      .replace(/^_+|_+$/g, '')
      // Ensure not empty
      || 'unnamed';
  }

  /**
   * Sanitize job ID for filesystem use
   */
  sanitizeJobId(jobId: string): string {
    const sanitized = this.sanitizePathComponent(jobId);
    
    // Ensure job ID has reasonable format
    if (!/^[a-zA-Z0-9_-]+$/.test(sanitized) || sanitized.length === 0) {
      return `job_${Date.now()}`;
    }
    
    return sanitized;
  }

  /**
   * Get relative path from base to target
   */
  getRelativePath(basePath: string, targetPath: string): string | null {
    if (!targetPath.startsWith(basePath)) {
      return null;
    }
    
    const relativePath = targetPath.substring(basePath.length);
    return relativePath.startsWith('/') ? relativePath.substring(1) : relativePath;
  }

  /**
   * Join path components safely
   */
  joinPaths(...components: string[]): string {
    return components
      .filter(component => component && component.length > 0)
      .map(component => component.replace(/^\/+|\/+$/g, ''))
      .join('/')
      .replace(/\/+/g, '/');
  }

  /**
   * Check if path is within allowed directory structure
   */
  isPathAllowed(path: string, username: string): boolean {
    const userBaseResult = this.getUserBasePath(username);
    if (!userBaseResult.success) {
      return false;
    }
    
    const userBase = userBaseResult.data;
    return path.startsWith(userBase) && this.validatePath(path);
  }
}

/**
 * Factory function for creating path resolver
 */

/**
 * Factory function for creating path resolver
 */
export function createPathResolver(): PathResolver {
  return new PathResolver();
}

// Export constants for direct use
export const REMOTE_PATHS = REMOTE_PATH_CONFIG;