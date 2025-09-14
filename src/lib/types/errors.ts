// Error categories for connection operations
export type ErrorCategory = 
  | 'Network'
  | 'Authentication' 
  | 'Timeout'
  | 'Permission'
  | 'Configuration'
  | 'Validation'
  | 'FileOperation'
  | 'Unknown';

// Base connection error interface
export interface ConnectionError {
  category: ErrorCategory;
  code: string;
  message: string;
  details?: string;
  retryable: boolean;
  suggestions?: string[];
  timestamp: string;
  context?: ErrorContext;
}

// Error context for debugging
export interface ErrorContext {
  operation?: string;
  host?: string;
  username?: string;
  path?: string;
  command?: string;
  attemptNumber?: number;
  duration?: number;
}

// User-friendly error message
export interface UserError {
  title: string;
  message: string;
  suggestions: string[];
  retryable: boolean;
  actionLabel?: string;
  actionCallback?: () => void;
}

// Error recovery strategy
export interface ErrorRecoveryStrategy {
  canRecover(error: ConnectionError): boolean;
  getRecoveryActions(error: ConnectionError): RecoveryAction[];
  shouldRetry(error: ConnectionError, attemptCount: number): boolean;
  getRetryDelay(attemptCount: number): number;
  getMaxRetries(): number;
}

// Recovery action
export interface RecoveryAction {
  type: 'retry' | 'reconnect' | 'refresh' | 'manual' | 'ignore';
  label: string;
  description?: string;
  automated: boolean;
  execute: () => Promise<void>;
}

// Predefined error codes and messages
export const CONNECTION_ERRORS = {
  // Network errors
  NETWORK_UNREACHABLE: {
    code: 'NET_001',
    category: 'Network' as ErrorCategory,
    message: 'Cannot reach cluster host',
    suggestions: [
      'Check your network connection',
      'Verify the cluster hostname is correct',
      'Check if you need to be on VPN',
      'Try again in a moment'
    ],
    retryable: true
  },
  
  CONNECTION_TIMEOUT: {
    code: 'NET_002',
    category: 'Timeout' as ErrorCategory,
    message: 'Connection timed out',
    suggestions: [
      'Check your network speed',
      'The cluster may be under heavy load',
      'Try increasing the timeout setting',
      'Try again in a few moments'
    ],
    retryable: true
  },
  
  CONNECTION_REFUSED: {
    code: 'NET_003',
    category: 'Network' as ErrorCategory,
    message: 'Connection refused by server',
    suggestions: [
      'Verify the cluster is accepting SSH connections',
      'Check if SSH service is running on the cluster',
      'Verify the port number (default is 22)',
      'Contact system administrator if issue persists'
    ],
    retryable: true
  },
  
  // Authentication errors
  AUTH_FAILED: {
    code: 'AUTH_001',
    category: 'Authentication' as ErrorCategory,
    message: 'Authentication failed',
    suggestions: [
      'Check your username and password',
      'Verify your account is active',
      'Check if your password has expired',
      'Contact system administrator if you cannot log in'
    ],
    retryable: false
  },
  
  SESSION_EXPIRED: {
    code: 'AUTH_002',
    category: 'Authentication' as ErrorCategory,
    message: 'Session has expired',
    suggestions: [
      'Your session has timed out',
      'Please reconnect to continue',
      'This is normal after extended inactivity'
    ],
    retryable: true
  },
  
  // Permission errors
  PERMISSION_DENIED: {
    code: 'PERM_001',
    category: 'Permission' as ErrorCategory,
    message: 'Permission denied',
    suggestions: [
      'Check you have access to this resource',
      'Verify your user permissions',
      'Contact system administrator for access'
    ],
    retryable: false
  },
  
  DIRECTORY_ACCESS_DENIED: {
    code: 'PERM_002',
    category: 'Permission' as ErrorCategory,
    message: 'Cannot access directory',
    suggestions: [
      'Check directory permissions',
      'Verify the path exists',
      'Ensure you have read/write access'
    ],
    retryable: false
  },
  
  // File operation errors
  FILE_NOT_FOUND: {
    code: 'FILE_001',
    category: 'FileOperation' as ErrorCategory,
    message: 'File not found',
    suggestions: [
      'Verify the file path is correct',
      'Check if the file has been moved or deleted',
      'Ensure you have permission to access the file'
    ],
    retryable: false
  },
  
  UPLOAD_FAILED: {
    code: 'FILE_002',
    category: 'FileOperation' as ErrorCategory,
    message: 'File upload failed',
    suggestions: [
      'Check available disk space on cluster',
      'Verify network connection stability',
      'Try uploading smaller files',
      'Check file permissions'
    ],
    retryable: true
  },
  
  DOWNLOAD_FAILED: {
    code: 'FILE_003',
    category: 'FileOperation' as ErrorCategory,
    message: 'File download failed',
    suggestions: [
      'Check available disk space locally',
      'Verify the file exists on cluster',
      'Check network connection stability',
      'Try downloading again'
    ],
    retryable: true
  },
  
  // Configuration errors
  INVALID_CONFIG: {
    code: 'CFG_001',
    category: 'Configuration' as ErrorCategory,
    message: 'Invalid configuration',
    suggestions: [
      'Check your connection settings',
      'Verify all required fields are filled',
      'Review the configuration for errors'
    ],
    retryable: false
  },
  
  MODULE_NOT_FOUND: {
    code: 'CFG_002',
    category: 'Configuration' as ErrorCategory,
    message: 'Required module not found',
    suggestions: [
      'Check module name and version',
      'Verify module is available on cluster',
      'Contact administrator about module availability'
    ],
    retryable: false
  },
  
  // Validation errors
  VALIDATION_FAILED: {
    code: 'VAL_001',
    category: 'Validation' as ErrorCategory,
    message: 'Validation failed',
    suggestions: [
      'Check input parameters',
      'Review validation errors',
      'Correct invalid fields and try again'
    ],
    retryable: false
  },
  
  // Generic/Unknown errors
  UNKNOWN_ERROR: {
    code: 'UNK_001',
    category: 'Unknown' as ErrorCategory,
    message: 'An unexpected error occurred',
    suggestions: [
      'Try the operation again',
      'Check the logs for more details',
      'Contact support if the issue persists'
    ],
    retryable: true
  }
};

// Error builder for creating consistent errors
export class ErrorBuilder {
  static create(
    template: typeof CONNECTION_ERRORS[keyof typeof CONNECTION_ERRORS],
    details?: string,
    context?: ErrorContext
  ): ConnectionError {
    return {
      category: template.category,
      code: template.code,
      message: template.message,
      details,
      retryable: template.retryable,
      suggestions: template.suggestions,
      timestamp: new Date().toISOString(),
      context
    };
  }
  
  static fromError(error: Error, category: ErrorCategory = 'Unknown'): ConnectionError {
    return {
      category,
      code: 'UNK_001',
      message: error.message,
      details: error.stack,
      retryable: category !== 'Authentication' && category !== 'Permission',
      suggestions: ['Check the error details', 'Try again'],
      timestamp: new Date().toISOString()
    };
  }
  
  static toUserError(error: ConnectionError): UserError {
    return {
      title: this.getCategoryTitle(error.category),
      message: error.message,
      suggestions: error.suggestions || [],
      retryable: error.retryable
    };
  }
  
  private static getCategoryTitle(category: ErrorCategory): string {
    const titles: Record<ErrorCategory, string> = {
      Network: 'Connection Problem',
      Authentication: 'Authentication Failed',
      Timeout: 'Operation Timed Out',
      Permission: 'Access Denied',
      Configuration: 'Configuration Error',
      Validation: 'Validation Error',
      FileOperation: 'File Operation Failed',
      Unknown: 'Unexpected Error'
    };
    return titles[category];
  }
}

// Default error recovery strategy
export class DefaultErrorRecoveryStrategy implements ErrorRecoveryStrategy {
  private maxRetries = 3;
  private baseDelay = 1000;
  
  canRecover(error: ConnectionError): boolean {
    return error.retryable && error.category !== 'Authentication';
  }
  
  getRecoveryActions(error: ConnectionError): RecoveryAction[] {
    const actions: RecoveryAction[] = [];
    
    if (error.retryable) {
      actions.push({
        type: 'retry',
        label: 'Retry Operation',
        description: 'Try the operation again',
        automated: true,
        execute: async () => {
          // Implementation will be in the service layer
        }
      });
    }
    
    if (error.category === 'Network' || error.category === 'Timeout') {
      actions.push({
        type: 'reconnect',
        label: 'Reconnect',
        description: 'Establish a new connection',
        automated: false,
        execute: async () => {
          // Implementation will be in the service layer
        }
      });
    }
    
    if (error.category === 'Authentication' && error.code === 'AUTH_002') {
      actions.push({
        type: 'refresh',
        label: 'Refresh Session',
        description: 'Refresh your authentication session',
        automated: true,
        execute: async () => {
          // Implementation will be in the service layer
        }
      });
    }
    
    return actions;
  }
  
  shouldRetry(error: ConnectionError, attemptCount: number): boolean {
    return error.retryable && attemptCount < this.maxRetries;
  }
  
  getRetryDelay(attemptCount: number): number {
    // Exponential backoff with jitter
    const delay = this.baseDelay * Math.pow(2, attemptCount);
    const jitter = Math.random() * 0.3 * delay;
    return Math.min(delay + jitter, 30000); // Max 30 seconds
  }
  
  getMaxRetries(): number {
    return this.maxRetries;
  }
}