import type { Result } from './connection';
import type { ConnectionError } from './errors';
import { ErrorBuilder, CONNECTION_ERRORS } from './errors';

/**
 * Utilities for converting between different error patterns
 * Ensures consistent error handling across all application layers
 */

/**
 * Convert an Error to a ConnectionError
 */
export function toConnectionError(
  error: Error | string, 
  category: ConnectionError['category'] = 'Unknown'
): ConnectionError {
  if (typeof error === 'string') {
    return ErrorBuilder.create({
      code: 'ERR_GENERIC',
      category,
      message: error,
      retryable: false,
      suggestions: ['Try the operation again', 'Check the system logs']
    });
  }
  
  return ErrorBuilder.fromError(error, category);
}

/**
 * Convert success/error object to Result pattern
 */
export function toResult<T>(successErrorResult: {
  success: boolean;
  data?: T;
  error?: string;
}): Result<T> {
  if (successErrorResult.success && successErrorResult.data !== undefined) {
    return { success: true, data: successErrorResult.data };
  }

  const error = toConnectionError(successErrorResult.error || 'Operation failed');
  return { success: false, error };
}

/**
 * Convert Result pattern to success/error object
 */
export function fromResult<T>(result: Result<T>): {
  success: boolean;
  data?: T;
  error?: string;
} {
  if (result.success) {
    return { success: true, data: result.data };
  }

  return {
    success: false,
    error: result.error.message
  };
}

/**
 * Wrap a function that might throw with Result pattern
 */
export function wrapWithResult<T, Args extends any[]>(
  fn: (...args: Args) => T | Promise<T>,
  errorCategory: ConnectionError['category'] = 'Unknown'
) {
  return async (...args: Args): Promise<Result<T>> => {
    try {
      const result = await fn(...args);
      return { success: true, data: result };
    } catch (error) {
      return { 
        success: false, 
        error: toConnectionError(error as Error, errorCategory) 
      };
    }
  };
}

/**
 * Chain multiple Result operations
 */
export function chainResults<T, U>(
  result: Result<T>,
  transform: (data: T) => Result<U> | Promise<Result<U>>
): Promise<Result<U>> {
  if (!result.success) {
    return Promise.resolve({ success: false, error: result.error });
  }
  
  return Promise.resolve(transform(result.data));
}

/**
 * Collect multiple results into a single result with array data
 */
export function collectResults<T>(results: Result<T>[]): Result<T[]> {
  const data: T[] = [];
  
  for (const result of results) {
    if (!result.success) {
      return { success: false, error: result.error };
    }
    data.push(result.data);
  }
  
  return { success: true, data };
}

/**
 * Execute multiple async operations and collect their results
 */
export async function collectAsyncResults<T>(
  operations: Promise<Result<T>>[]
): Promise<Result<T[]>> {
  try {
    const results = await Promise.all(operations);
    return collectResults(results);
  } catch (error) {
    return { 
      success: false, 
      error: toConnectionError(error as Error, 'Unknown') 
    };
  }
}

/**
 * Retry a Result-returning operation with exponential backoff
 */
export async function retryWithResult<T>(
  operation: () => Promise<Result<T>>,
  maxRetries: number = 3,
  baseDelay: number = 1000,
  enableDelay: boolean = true
): Promise<Result<T>> {
  let lastError: ConnectionError | null = null;
  
  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    const result = await operation();
    
    if (result.success) {
      return result;
    }
    
    lastError = result.error;
    
    // Don't retry if error is not retryable
    if (!result.error.retryable || attempt === maxRetries) {
      break;
    }
    
    // Exponential backoff with jitter (can be disabled for tests)
    if (enableDelay) {
      const delay = baseDelay * Math.pow(2, attempt);
      const jitter = Math.random() * 0.3 * delay;
      await new Promise(resolve => setTimeout(resolve, delay + jitter));
    }
  }
  
  return { 
    success: false, 
    error: lastError || ErrorBuilder.create(CONNECTION_ERRORS.UNKNOWN_ERROR)
  };
}

/**
 * Validate and normalize error responses from different layers
 */
export class ErrorNormalizer {
  /**
   * Normalize IPC command result to consistent format
   */
  static normalizeIPCResult<T>(ipcResult: any): Result<T> {
    // Handle Tauri command results
    if (typeof ipcResult === 'object' && ipcResult !== null) {
      if ('success' in ipcResult) {
        return toResult(ipcResult);
      }
      
      // Handle direct data responses
      return { success: true, data: ipcResult };
    }
    
    return { 
      success: false, 
      error: ErrorBuilder.create(CONNECTION_ERRORS.UNKNOWN_ERROR, 'Invalid IPC response format')
    };
  }
  
  /**
   * Normalize network/HTTP errors
   */
  static normalizeNetworkError(error: any): ConnectionError {
    if (error.code === 'ENOTFOUND') {
      return ErrorBuilder.create(CONNECTION_ERRORS.NETWORK_UNREACHABLE);
    }
    
    if (error.code === 'ETIMEDOUT') {
      return ErrorBuilder.create(CONNECTION_ERRORS.CONNECTION_TIMEOUT);
    }
    
    if (error.code === 'ECONNREFUSED') {
      return ErrorBuilder.create(CONNECTION_ERRORS.CONNECTION_REFUSED);
    }
    
    return toConnectionError(error, 'Network');
  }
  
  /**
   * Normalize validation errors
   */
  static normalizeValidationError(error: any): ConnectionError {
    return toConnectionError(error, 'Validation');
  }
}