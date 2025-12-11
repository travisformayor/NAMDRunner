import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { createStore, isConnectionError, invokeWithErrorHandling } from './storeFactory';
import type { ApiResult } from '../types/api';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

// Mock session actions
vi.mock('./session', () => ({
  sessionActions: {
    markExpired: vi.fn(),
  },
}));

import { invoke } from '@tauri-apps/api/core';
import { sessionActions } from './session';

describe('isConnectionError', () => {
  it('should detect timeout errors', () => {
    expect(isConnectionError('Connection timeout')).toBe(true);
    expect(isConnectionError('Request timed out')).toBe(true);
    expect(isConnectionError('SSH timeout after 30s')).toBe(true);
  });

  it('should detect not connected errors', () => {
    expect(isConnectionError('Not connected to cluster')).toBe(true);
    expect(isConnectionError('not connected')).toBe(true);
    expect(isConnectionError('Connection not established')).toBe(true);
  });

  it('should detect connection failure errors', () => {
    expect(isConnectionError('Connection failed')).toBe(true);
    expect(isConnectionError('Connection refused')).toBe(true);
    expect(isConnectionError('Connection reset')).toBe(true);
  });

  it('should detect broken pipe errors', () => {
    expect(isConnectionError('Broken pipe')).toBe(true);
    expect(isConnectionError('broken pipe detected')).toBe(true);
  });

  it('should detect network errors', () => {
    expect(isConnectionError('Network unreachable')).toBe(true);
    expect(isConnectionError('Network error occurred')).toBe(true);
  });

  it('should detect SSH errors', () => {
    expect(isConnectionError('SSH connection lost')).toBe(true);
    expect(isConnectionError('ssh handshake failed')).toBe(true);
  });

  it('should be case insensitive', () => {
    expect(isConnectionError('CONNECTION TIMEOUT')).toBe(true);
    expect(isConnectionError('Network Error')).toBe(true);
    expect(isConnectionError('SSH Failed')).toBe(true);
  });

  it('should not detect unrelated errors', () => {
    expect(isConnectionError('Template not found')).toBe(false);
    expect(isConnectionError('Invalid job ID')).toBe(false);
    expect(isConnectionError('Database error')).toBe(false);
    expect(isConnectionError('Validation failed')).toBe(false);
  });

  it('should handle empty strings', () => {
    expect(isConnectionError('')).toBe(false);
  });
});

describe('createStore', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should create store with initial data', () => {
    const store = createStore({
      initialData: [] as string[],
    });

    const state = get(store);
    expect(state.data).toEqual([]);
    expect(state.loading).toBe(false);
    expect(state.error).toBe(null);
  });


  it('should setData correctly', () => {
    const store = createStore({
      initialData: [] as string[],
    });

    store.setData(['item1', 'item2']);

    const state = get(store);
    expect(state.data).toEqual(['item1', 'item2']);
    expect(state.loading).toBe(false);
    expect(state.error).toBe(null);
  });

  it('should setError correctly', () => {
    const store = createStore({
      initialData: null,
    });

    store.setError('Test error');

    const state = get(store);
    expect(state.error).toBe('Test error');
  });

  it('should clearError correctly', () => {
    const store = createStore({
      initialData: null,
    });

    store.setError('Existing error');
    store.clearError();

    const state = get(store);
    expect(state.error).toBe(null);
  });

  it('should reset to initial state', () => {
    const store = createStore({
      initialData: 'initial',
    });

    // Modify state
    store.setData('modified');
    store.setError('Some error');

    // Reset
    store.reset();

    const state = get(store);
    expect(state.data).toBe('initial');
    expect(state.loading).toBe(false);
    expect(state.error).toBe(null);
  });

  it('should load data successfully', async () => {
    vi.mocked(invoke).mockResolvedValue({
      success: true,
      data: { value: 'test data' },
      // error omitted
    } as ApiResult<{ value: string }>);

    const store = createStore({
      initialData: null as { value: string } | null,
      loadCommand: 'test_command',
    });

    const success = await store.load();

    expect(success).toBe(true);
    expect(invoke).toHaveBeenCalledWith('test_command');

    const state = get(store);
    expect(state.data).toEqual({ value: 'test data' });
    expect(state.loading).toBe(false);
    expect(state.error).toBe(null);
  });

  it('should handle load errors', async () => {
    vi.mocked(invoke).mockResolvedValue({
      success: false,
      // data omitted
      error: 'Load failed',
    } as ApiResult<string>);

    const store = createStore({
      initialData: '',
      loadCommand: 'test_command',
    });

    const success = await store.load();

    expect(success).toBe(false);

    const state = get(store);
    expect(state.error).toBe('Load failed');
    expect(state.loading).toBe(false);
  });

  it('should handle connection errors with session expiry', async () => {
    vi.mocked(invoke).mockResolvedValue({
      success: false,
      // data omitted
      error: 'Connection timeout',
    } as ApiResult<string>);

    const store = createStore({
      initialData: '',
      loadCommand: 'test_command',
    });

    await store.load();

    expect(sessionActions.markExpired).toHaveBeenCalledWith('Connection timeout');
  });

  it('should not mark session expired for non-connection errors', async () => {
    vi.mocked(invoke).mockResolvedValue({
      success: false,
      // data omitted
      error: 'Template not found',
    } as ApiResult<string>);

    const store = createStore({
      initialData: '',
      loadCommand: 'test_command',
    });

    await store.load();

    expect(sessionActions.markExpired).not.toHaveBeenCalled();
  });

  it('should handle load exceptions', async () => {
    vi.mocked(invoke).mockRejectedValue(new Error('Network failure'));

    const store = createStore({
      initialData: '',
      loadCommand: 'test_command',
    });

    const success = await store.load();

    expect(success).toBe(false);

    const state = get(store);
    expect(state.error).toBe('Network failure');
    expect(state.loading).toBe(false);
  });

  it('should require loadCommand when calling load', async () => {
    const store = createStore({
      initialData: '',
    });

    const success = await store.load();

    expect(success).toBe(false);
    // Note: Store logs warning but doesn't set error state
  });

  it('should handle showLoading option', async () => {
    vi.mocked(invoke).mockImplementation(
      () =>
        new Promise((resolve) =>
          setTimeout(
            () =>
              resolve({
                success: true,
                data: 'test',
                // error omitted
              } as ApiResult<string>),
            10
          )
        )
    );

    const store = createStore({
      initialData: '',
      loadCommand: 'test_command',
    });

    const loadPromise = store.load({ showLoading: true });

    // Check loading state is true during operation
    const duringLoad = get(store);
    expect(duringLoad.loading).toBe(true);

    await loadPromise;

    const afterLoad = get(store);
    expect(afterLoad.loading).toBe(false);
  });

  it('should support refresh as alias for load', async () => {
    vi.mocked(invoke).mockResolvedValue({
      success: true,
      data: 'refreshed',
      // error omitted
    } as ApiResult<string>);

    const store = createStore({
      initialData: '',
      loadCommand: 'test_command',
    });

    await store.refresh();

    const state = get(store);
    expect(state.data).toBe('refreshed');
  });
});

describe('invokeWithErrorHandling', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should return data on success', async () => {
    vi.mocked(invoke).mockResolvedValue({
      success: true,
      data: { value: 42 },
      // error omitted
    } as ApiResult<{ value: number }>);

    const result = await invokeWithErrorHandling<{ value: number }>(
      'test_command',
      { param: 'value' }
    );

    expect(result.success).toBe(true);
    expect(result.data).toEqual({ value: 42 });
    expect(result.error).toBeUndefined();
  });

  it('should return error on failure', async () => {
    vi.mocked(invoke).mockResolvedValue({
      success: false,
      // data omitted
      error: 'Operation failed',
    } as ApiResult<string>);

    const result = await invokeWithErrorHandling<string>('test_command');

    expect(result.success).toBe(false);
    expect(result.error).toBe('Operation failed');
  });

  it('should detect connection errors', async () => {
    vi.mocked(invoke).mockResolvedValue({
      success: false,
      // data omitted
      error: 'SSH connection lost',
    } as ApiResult<string>);

    const result = await invokeWithErrorHandling<string>(
      'test_command',
      {}
    );

    expect(result.success).toBe(false);
    expect(sessionActions.markExpired).toHaveBeenCalledWith('SSH connection lost');
  });

  it('should handle exceptions gracefully', async () => {
    vi.mocked(invoke).mockRejectedValue(new Error('Unexpected error'));

    const result = await invokeWithErrorHandling<string>('test_command');

    expect(result.success).toBe(false);
    expect(result.error).toBe('Unexpected error');
  });

});
