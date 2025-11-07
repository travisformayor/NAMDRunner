import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { sessionActions, connectionState, isConnected } from './session';
import { CoreClientFactory } from '../ports/clientFactory';
import type { ICoreClient, ConnectResult, DisconnectResult } from '../ports/coreClient';

// Mock the CoreClientFactory
vi.mock('../ports/clientFactory', () => ({
  CoreClientFactory: {
    getClient: vi.fn(),
    reset: vi.fn()
  }
}));

describe('Session Store', () => {
  let mockClient: ICoreClient;

  beforeEach(() => {
    // Reset session state
    sessionActions.reset();

    // Create fresh mock client for each test
    mockClient = {
      connect: vi.fn(),
      disconnect: vi.fn(),
      getConnectionStatus: vi.fn(),
      createJob: vi.fn(),
      submitJob: vi.fn(),
      getJobStatus: vi.fn(),
      getAllJobs: vi.fn(),
      syncJobs: vi.fn(),
      deleteJob: vi.fn(),
      refetchLogs: vi.fn(),
      uploadFile: vi.fn(),
      downloadAllOutputs: vi.fn(),
      listFiles: vi.fn(),
      getClusterCapabilities: vi.fn(),
      validateResourceAllocation: vi.fn(),
      calculateJobCost: vi.fn(),
      estimateQueueTimeForJob: vi.fn(),
      suggestQosForPartition: vi.fn()
    };

    // Mock CoreClientFactory to return our mock client
    vi.mocked(CoreClientFactory.getClient).mockReturnValue(mockClient);
  });

  it('should initialize with disconnected state', () => {
    expect(get(connectionState)).toBe('Disconnected');
    expect(get(isConnected)).toBe(false);
  });

  it('should handle successful connection', async () => {
    const successResult: ConnectResult = {
      success: true,
      session_info: {
        host: 'test.host',
        username: 'testuser',
        connected_at: new Date().toISOString()
      },
      error: null
    };
    vi.mocked(mockClient.connect).mockResolvedValue(successResult);

    const success = await sessionActions.connect('test.host', 'testuser', 'testpass');

    expect(success).toBe(true);
    expect(get(connectionState)).toBe('Connected');
    expect(get(isConnected)).toBe(true);
    expect(mockClient.connect).toHaveBeenCalledWith({
      host: 'test.host',
      username: 'testuser',
      password: 'testpass'
    });
  });

  it('should handle connection failure', async () => {
    const failureResult: ConnectResult = {
      success: false,
      session_info: null,
      error: 'Authentication failed'
    };
    vi.mocked(mockClient.connect).mockResolvedValue(failureResult);

    const success = await sessionActions.connect('invalid.host', 'testuser', 'testpass');

    expect(success).toBe(false);
    expect(get(connectionState)).toBe('Disconnected');
    expect(get(isConnected)).toBe(false);
  });

  it('should handle disconnection', async () => {
    // First connect
    const connectResult: ConnectResult = {
      success: true,
      session_info: {
        host: 'test.host',
        username: 'testuser',
        connected_at: new Date().toISOString()
      },
      error: null
    };
    vi.mocked(mockClient.connect).mockResolvedValue(connectResult);

    const connectSuccess = await sessionActions.connect('test.host', 'testuser', 'testpass');
    expect(connectSuccess).toBe(true);
    expect(get(connectionState)).toBe('Connected');

    // Then disconnect
    const disconnectResult: DisconnectResult = {
      success: true,
      error: null
    };
    vi.mocked(mockClient.disconnect).mockResolvedValue(disconnectResult);

    const disconnectSuccess = await sessionActions.disconnect();

    expect(disconnectSuccess).toBe(true);
    expect(get(connectionState)).toBe('Disconnected');
    expect(get(isConnected)).toBe(false);
    expect(mockClient.disconnect).toHaveBeenCalled();
  });

  it('should clear errors', async () => {
    // Trigger an error
    const failureResult: ConnectResult = {
      success: false,
      session_info: null,
      error: 'Connection error'
    };
    vi.mocked(mockClient.connect).mockResolvedValue(failureResult);
    await sessionActions.connect('invalid.host', 'testuser', 'testpass');

    // Clear the error
    sessionActions.clearError();

    // Error should be cleared but connection state unchanged
    expect(get(connectionState)).toBe('Disconnected');
  });
});
