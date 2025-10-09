import { writable, derived } from 'svelte/store';
import type { ConnectionState, SessionInfo } from '../types/api';
import { CoreClientFactory } from '../ports/clientFactory';
import { jobsStore } from './jobs';

// Session state store
interface SessionState {
  connectionState: ConnectionState;
  session_info: SessionInfo | null;
  lastError: string | null;
  isConnecting: boolean;
}

const initialState: SessionState = {
  connectionState: 'Disconnected',
  session_info: null,
  lastError: null,
  isConnecting: false,
};

// Create the writable store
const sessionStore = writable<SessionState>(initialState);

// Derived stores for common use cases
export const connectionState = derived(sessionStore, ($session) => $session.connectionState);
export const isConnected = derived(sessionStore, ($session) => $session.connectionState === 'Connected');
export const isConnecting = derived(sessionStore, ($session) => $session.isConnecting);
export const sessionInfo = derived(sessionStore, ($session) => $session.session_info);
export const lastError = derived(sessionStore, ($session) => $session.lastError);

// Actions for managing session state
export const sessionActions = {
  // Connect to cluster
  async connect(host: string, username: string, password: string): Promise<boolean> {
    // Starting connection attempt

    sessionStore.update((state) => ({
      ...state,
      isConnecting: true,
      lastError: null,
    }));

    try {
      // Calling CoreClientFactory.getClient().connect()
      const result = await CoreClientFactory.getClient().connect({ host, username, password });
      // Connection attempt completed
      
      if (result.success) {
        sessionStore.update((state) => ({
          ...state,
          connectionState: 'Connected',
          session_info: result.session_info || null,
          isConnecting: false,
          lastError: null,
        }));

        // Sync jobs after successful connection (but not in demo mode)
        const currentMode = CoreClientFactory.getUserMode();
        if (currentMode !== 'demo') {
          await jobsStore.sync();
        }
        // In demo mode, keep the existing cached demo jobs

        return true;
      } else {
        // Connection failed
        sessionStore.update((state) => ({
          ...state,
          connectionState: 'Disconnected',
          session_info: null,
          isConnecting: false,
          lastError: result.error || 'Connection failed',
        }));
        return false;
      }
    } catch (error) {
      // Connection threw exception
      sessionStore.update((state) => ({
        ...state,
        connectionState: 'Disconnected',
        session_info: null,
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
        session_info: null,
        lastError: result.error || null,
      }));

      // Clear jobs when disconnecting, but keep demo jobs if in demo mode
      const currentMode = CoreClientFactory.getUserMode();
      if (currentMode === 'demo') {
        // Keep demo jobs when disconnecting in demo mode
        jobsStore.loadDemoJobs();
      } else {
        // Clear all jobs when disconnecting in real mode
        jobsStore.reset();
      }

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
        session_info: result.session_info || null,
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
};

// Export the store for subscription
export const session = { subscribe: sessionStore.subscribe };