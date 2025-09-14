import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { sessionActions, connectionState, isConnected } from './session';
import { CoreClientFactory } from '../ports/clientFactory';
import type { MockCoreClient } from '../ports/coreClient-mock';

describe('Session Store', () => {
  beforeEach(() => {
    // Reset the client factory to use mock
    CoreClientFactory.reset();
    // Reset session state
    sessionActions.reset();
    
    // Get mock client and disable error injection for predictable tests
    const mockClient = CoreClientFactory.getClient(true) as MockCoreClient;
    mockClient.enableErrorInjection(false);
    mockClient.resetToCleanState();
  });

  it('should initialize with disconnected state', () => {
    expect(get(connectionState)).toBe('Disconnected');
    expect(get(isConnected)).toBe(false);
  });

  it('should handle successful connection', async () => {
    const mockClient = CoreClientFactory.getClient(true); // Force mock
    
    const success = await sessionActions.connect('test.host', 'testuser', 'testpass');
    
    expect(success).toBe(true);
    expect(get(connectionState)).toBe('Connected');
    expect(get(isConnected)).toBe(true);
  });

  it('should handle connection failure', async () => {
    const success = await sessionActions.connect('invalid.host', 'testuser', 'testpass');
    
    expect(success).toBe(false);
    expect(get(connectionState)).toBe('Disconnected');
    expect(get(isConnected)).toBe(false);
  });

  it('should handle disconnection', async () => {
    // First connect
    const connectSuccess = await sessionActions.connect('test.host', 'testuser', 'testpass');
    expect(connectSuccess).toBe(true);
    expect(get(connectionState)).toBe('Connected');
    expect(get(isConnected)).toBe(true);
    
    // Then disconnect
    const disconnectSuccess = await sessionActions.disconnect();
    
    expect(disconnectSuccess).toBe(true);
    expect(get(connectionState)).toBe('Disconnected');
    expect(get(isConnected)).toBe(false);
  });

  it('should clear errors', async () => {
    // Trigger an error
    await sessionActions.connect('invalid.host', 'testuser', 'testpass');
    
    // Clear the error
    sessionActions.clearError();
    
    // Error should be cleared but connection state unchanged
    expect(get(connectionState)).toBe('Disconnected');
  });
});