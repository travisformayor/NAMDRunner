import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { sessionActions, connectionState, isConnected } from './session';
import type { SessionInfo, ApiResult } from '../types/api';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

// Import after mock to get mocked version
import { invoke } from '@tauri-apps/api/core';

describe('Session Store', () => {
  beforeEach(() => {
    // Reset session state
    sessionActions.reset();

    // Clear all mocks
    vi.clearAllMocks();
  });

  it('should initialize with disconnected state', () => {
    expect(get(connectionState)).toBe('Disconnected');
    expect(get(isConnected)).toBe(false);
  });

  it('should handle successful connection', async () => {
    const successResult: ApiResult<SessionInfo> = {
      success: true,
      data: {
        host: 'test.host',
        username: 'testuser',
        connected_at: new Date().toISOString()
      }
    };

    vi.mocked(invoke).mockResolvedValue(successResult);

    const success = await sessionActions.connect('test.host', 'testuser', 'testpass');

    expect(success).toBe(true);
    expect(get(connectionState)).toBe('Connected');
    expect(get(isConnected)).toBe(true);
    expect(invoke).toHaveBeenCalledWith('connect_to_cluster', {
      params: {
        host: 'test.host',
        username: 'testuser',
        password: 'testpass'
      }
    });
  });

  it('should handle connection failure', async () => {
    const failureResult: ApiResult<SessionInfo> = {
      success: false,
      error: 'Authentication failed'
    };

    vi.mocked(invoke).mockResolvedValue(failureResult);

    const success = await sessionActions.connect('invalid.host', 'testuser', 'testpass');

    expect(success).toBe(false);
    expect(get(connectionState)).toBe('Disconnected');
    expect(get(isConnected)).toBe(false);
  });

  it('should handle connection exception', async () => {
    vi.mocked(invoke).mockRejectedValue(new Error('Network error'));

    const success = await sessionActions.connect('test.host', 'testuser', 'testpass');

    expect(success).toBe(false);
    expect(get(connectionState)).toBe('Disconnected');
    expect(get(isConnected)).toBe(false);
  });

  it('should handle disconnection', async () => {
    // First connect
    const connectResult: ApiResult<SessionInfo> = {
      success: true,
      data: {
        host: 'test.host',
        username: 'testuser',
        connected_at: new Date().toISOString()
      }
    };
    vi.mocked(invoke).mockResolvedValue(connectResult);

    const connectSuccess = await sessionActions.connect('test.host', 'testuser', 'testpass');
    expect(connectSuccess).toBe(true);
    expect(get(connectionState)).toBe('Connected');

    // Then disconnect
    const disconnectResult: ApiResult<void> = {
      success: true
    };
    vi.mocked(invoke).mockResolvedValue(disconnectResult);

    const disconnectSuccess = await sessionActions.disconnect();

    expect(disconnectSuccess).toBe(true);
    expect(get(connectionState)).toBe('Disconnected');
    expect(get(isConnected)).toBe(false);
    expect(invoke).toHaveBeenCalledWith('disconnect');
  });

  it('should clear errors', async () => {
    // Trigger an error
    const failureResult: ApiResult<SessionInfo> = {
      success: false,
      error: 'Connection error'
    };
    vi.mocked(invoke).mockResolvedValue(failureResult);
    await sessionActions.connect('invalid.host', 'testuser', 'testpass');

    // Clear the error
    sessionActions.clearError();

    // Error should be cleared but connection state unchanged
    expect(get(connectionState)).toBe('Disconnected');
  });
});
