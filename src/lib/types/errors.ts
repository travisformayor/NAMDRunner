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
export interface ConnectionError extends Error {
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
    customMessage?: string,
    context?: ErrorContext
  ): ConnectionError {
    const error = new Error(customMessage || template.message) as ConnectionError;
    error.name = 'ConnectionError';
    error.category = template.category;
    error.code = template.code;
    error.message = customMessage || template.message;
    if (customMessage) {
      error.details = template.message;
    }
    error.retryable = template.retryable;
    error.suggestions = template.suggestions;
    error.timestamp = new Date().toISOString();
    if (context) {
      error.context = context;
    }
    return error;
  }
  
  static fromError(error: Error, category: ErrorCategory = 'Unknown'): ConnectionError {
    const connError = error as ConnectionError;
    connError.name = 'ConnectionError';
    connError.category = category;
    connError.code = 'UNK_001';
    connError.message = error.message;
    if (error.stack) {
      connError.details = error.stack;
    }
    connError.retryable = category !== 'Authentication' && category !== 'Permission';
    connError.suggestions = ['Check the error details', 'Try again'];
    connError.timestamp = new Date().toISOString();
    return connError;
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