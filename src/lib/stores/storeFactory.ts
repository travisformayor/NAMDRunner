/**
 * Store Factory
 *
 * Generic factory for creating Svelte stores with standard patterns:
 * - Loading/error/data state management
 * - ApiResult<T> handling
 * - Connection error detection and session expiry
 * - Standard load/refresh/reset methods
 *
 * Eliminates duplication across stores (jobs, settings, templates, etc.)
 */

import { writable, type Writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { ApiResult } from '../types/api';
import { sessionActions } from './session';

/**
 * Standard store state shape
 */
export interface StoreState<T> {
  data: T;
  loading: boolean;
  error: string | null;
}

/**
 * Configuration for store factory
 */
export interface StoreConfig<T> {
  /** Initial data value */
  initialData: T;

  /** IPC command to load data */
  loadCommand?: string;

  /** Enable connection error detection and session expiry */
  enableConnectionHandling?: boolean;

  /** Custom initial state (overrides standard state + initialData) */
  initialState?: StoreState<T>;
}

/**
 * Options for load operations
 */
export interface LoadOptions {
  /** Show loading state during operation */
  showLoading?: boolean;

  /** Clear error state before operation */
  clearError?: boolean;
}

/**
 * Store actions returned by factory
 */
export interface StoreActions<T> {
  /** Subscribe to store (Svelte store contract) */
  subscribe: Writable<StoreState<T>>['subscribe'];

  /** Load data using configured load command */
  load: (options?: LoadOptions) => Promise<boolean>;

  /** Refresh data (alias for load with default options) */
  refresh: () => Promise<boolean>;

  /** Reset to initial state */
  reset: () => void;

  /** Set data directly */
  setData: (data: T) => void;

  /** Set error state */
  setError: (error: string | null) => void;

  /** Clear error state */
  clearError: () => void;

  /** Update state directly */
  update: Writable<StoreState<T>>['update'];
}

/**
 * Helper: Detect if error indicates connection failure
 * Exported for testing and direct use
 */
export function isConnectionError(errorMessage: string): boolean {
  const msg = errorMessage.toLowerCase();
  return (
    msg.includes('timeout') ||
    msg.includes('timed out') ||
    msg.includes('not connected') ||
    msg.includes('connection') ||
    msg.includes('disconnect') ||
    msg.includes('broken pipe') ||
    msg.includes('network') ||
    msg.includes('ssh')
  );
}

/**
 * Handle API result and update store state
 * Returns true if successful, false otherwise
 */
function handleApiResult<T>(
  result: ApiResult<T>,
  update: Writable<StoreState<T>>['update'],
  config: StoreConfig<T>
): boolean {
  if (result.success && result.data !== undefined) {
    update((state) => ({
      ...state,
      data: result.data as T,
      loading: false,
      error: null,
    }));
    return true;
  } else {
    const errorMsg = result.error || 'Operation failed';

    // Handle connection errors if enabled
    if (config.enableConnectionHandling && isConnectionError(errorMsg)) {
      sessionActions.markExpired(errorMsg);
    }

    update((state) => ({
      ...state,
      loading: false,
      error: errorMsg,
    }));
    return false;
  }
}

/**
 * Handle exceptions during API calls
 */
function handleException<T>(
  error: unknown,
  update: Writable<StoreState<T>>['update'],
  config: StoreConfig<T>
): void {
  const errorMsg = error instanceof Error ? error.message : String(error);

  // Handle connection errors if enabled
  if (config.enableConnectionHandling && isConnectionError(errorMsg)) {
    sessionActions.markExpired(errorMsg);
  }

  update((state) => ({
    ...state,
    loading: false,
    error: errorMsg,
  }));
}

/**
 * Create a store with standard loading/error/data patterns
 *
 * @example Basic usage
 * ```ts
 * const settingsStore = createStore({
 *   initialData: null as DatabaseInfo | null,
 *   loadCommand: 'get_database_info'
 * });
 * ```
 *
 * @example With connection handling
 * ```ts
 * const jobsStore = createStore({
 *   initialData: [] as JobInfo[],
 *   loadCommand: 'get_all_jobs',
 *   enableConnectionHandling: true
 * });
 * ```
 */
export function createStore<T>(
  config: StoreConfig<T>
): StoreActions<T> {
  const initialState: StoreState<T> =
    config.initialState ||
    ({
      data: config.initialData,
      loading: false,
      error: null,
    } as StoreState<T>);

  const { subscribe, update, set } = writable<StoreState<T>>(initialState);

  return {
    subscribe,
    update,

    async load(options: LoadOptions = {}): Promise<boolean> {
      const { showLoading = true, clearError: shouldClearError = true } = options;

      if (!config.loadCommand) {
        console.warn('Store has no loadCommand configured');
        return false;
      }

      // Update loading/error state
      update((state) => ({
        ...state,
        loading: showLoading,
        error: shouldClearError ? null : state.error,
      }));

      try {
        const result = await invoke<ApiResult<T>>(config.loadCommand);
        return handleApiResult(result, update, config);
      } catch (error) {
        handleException(error, update, config);
        return false;
      }
    },

    async refresh(): Promise<boolean> {
      return this.load({ showLoading: true, clearError: true });
    },

    reset(): void {
      set(initialState);
    },

    setData(data: T): void {
      update((state) => ({
        ...state,
        data,
        error: null,
      }));
    },

    setError(error: string | null): void {
      update((state) => ({
        ...state,
        error,
      }));
    },

    clearError(): void {
      update((state) => ({
        ...state,
        error: null,
      }));
    },
  };
}

/**
 * Invoke a command and handle ApiResult with connection error detection
 *
 * Utility for custom store methods that don't fit the standard load pattern.
 * Handles ApiResult extraction, connection errors, and returns typed data.
 *
 * @example
 * ```ts
 * const result = await invokeWithErrorHandling<JobInfo>(
 *   'get_job_status',
 *   { job_id: '123' },
 *   { enableConnectionHandling: true }
 * );
 * ```
 */
export async function invokeWithErrorHandling<T>(
  command: string,
  args?: Record<string, unknown>,
  options: { enableConnectionHandling?: boolean } = {}
): Promise<{ success: boolean; data?: T; error?: string }> {
  try {
    const result = await invoke<ApiResult<T>>(command, args);

    if (result.success && result.data !== undefined) {
      return { success: true, data: result.data };
    } else {
      const errorMsg = result.error || 'Operation failed';

      // Handle connection errors if enabled
      if (options.enableConnectionHandling && isConnectionError(errorMsg)) {
        sessionActions.markExpired(errorMsg);
      }

      return { success: false, error: errorMsg };
    }
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : String(error);

    // Handle connection errors if enabled
    if (options.enableConnectionHandling && isConnectionError(errorMsg)) {
      sessionActions.markExpired(errorMsg);
    }

    return { success: false, error: errorMsg };
  }
}
