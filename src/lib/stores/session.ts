import { writable, derived } from 'svelte/store';
import type { ConnectionState, SessionInfo } from '../types/api';
import { CoreClientFactory } from '../ports/clientFactory';

// Session state store
interface SessionState {
  connectionState: ConnectionState;
  sessionInfo: SessionInfo | null;
  lastError: string | null;
  isConnecting: boolean;
}

const initialState: SessionState = {
  connectionState: 'Disconnected',
  sessionInfo: null,
  lastError: null,
  isConnecting: false,
};

// Create the writable store
const sessionStore = writable<SessionState>(initialState);

// Derived stores for common use cases
export const connectionState = derived(sessionStore, ($session) => $session.connectionState);
export const isConnected = derived(sessionStore, ($session) => $session.connectionState === 'Connected');
export const isConnecting = derived(sessionStore, ($session) => $session.isConnecting);
export const sessionInfo = derived(sessionStore, ($session) => $session.sessionInfo);
export const lastError = derived(sessionStore, ($session) => $session.lastError);

// Actions for managing session state
export const sessionActions = {
  // Connect to cluster
  async connect(host: string, username: string, password: string): Promise<boolean> {
    sessionStore.update((state) => ({
      ...state,
      isConnecting: true,
      lastError: null,
    }));

    try {
      const result = await CoreClientFactory.getClient().connect({ host, username, password });
      
      if (result.success) {
        sessionStore.update((state) => ({
          ...state,
          connectionState: 'Connected',
          sessionInfo: result.sessionInfo || null,
          isConnecting: false,
          lastError: null,
        }));
        return true;
      } else {
        sessionStore.update((state) => ({
          ...state,
          connectionState: 'Disconnected',
          sessionInfo: null,
          isConnecting: false,
          lastError: result.error || 'Connection failed',
        }));
        return false;
      }
    } catch (error) {
      sessionStore.update((state) => ({
        ...state,
        connectionState: 'Disconnected',
        sessionInfo: null,
        isConnecting: false,
        lastError: error instanceof Error ? error.message : 'Unknown error occurred',
      }));
      return false;
    }
  },

  // Disconnect from cluster
  async disconnect(): Promise<boolean> {
    try {
      const result = await CoreClientFactory.getClient().disconnect();
      
      sessionStore.update((state) => ({
        ...state,
        connectionState: 'Disconnected',
        sessionInfo: null,
        lastError: result.error || null,
      }));
      
      return result.success;
    } catch (error) {
      sessionStore.update((state) => ({
        ...state,
        lastError: error instanceof Error ? error.message : 'Disconnect failed',
      }));
      return false;
    }
  },

  // Check connection status
  async checkStatus(): Promise<void> {
    try {
      const result = await CoreClientFactory.getClient().getConnectionStatus();
      
      sessionStore.update((state) => ({
        ...state,
        connectionState: result.state,
        sessionInfo: result.sessionInfo || null,
      }));
    } catch (error) {
      sessionStore.update((state) => ({
        ...state,
        lastError: error instanceof Error ? error.message : 'Status check failed',
      }));
    }
  },

  // Clear error
  clearError(): void {
    sessionStore.update((state) => ({
      ...state,
      lastError: null,
    }));
  },

  // Reset session to initial state
  reset(): void {
    sessionStore.set(initialState);
  },

  // Mock connected state for UI testing
  mockConnected(): void {
    sessionStore.update((state) => ({
      ...state,
      connectionState: 'Connected' as const,
      sessionInfo: {
        host: 'login.rc.colorado.edu',
        username: 'jsmith',
        homeDirectory: '/home/jsmith',
        workingDirectory: '/projects/jsmith/namdrunner_jobs',
        slurmVersion: '23.02.7',
        modules: ['gcc/14.2.0', 'openmpi/4.1.4', 'namd/3.0']
      },
      isConnecting: false,
      lastError: null,
    }));
  },
};

// Export the store for subscription
export const session = { subscribe: sessionStore.subscribe };