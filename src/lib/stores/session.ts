import { writable, derived } from 'svelte/store';
import type {
  ConnectionState,
  SessionInfo,
  ConnectResult,
  DisconnectResult,
  ConnectionStatusResult
} from '../types/api';
import { invoke } from '@tauri-apps/api/core';
import { jobsStore } from './jobs';
import { logger } from '../utils/logger';

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
    logger.debug('SSH', `Starting connection attempt to ${host} as ${username}`);

    sessionStore.update((state) => ({
      ...state,
      isConnecting: true,
      lastError: null,
    }));

    try {
      const result = await invoke<ConnectResult>('connect_to_cluster', {
        params: { host, username, password }
      });

      logger.debug('SSH', `Connection result: ${result.success ? 'SUCCESS' : 'FAILED'}`);

      if (result.success) {
        logger.debug('SSH', 'Connection established successfully');

        sessionStore.update((state) => ({
          ...state,
          connectionState: 'Connected',
          session_info: result.session_info || null,
          isConnecting: false,
          lastError: null,
        }));

        // Sync jobs after successful connection
        await jobsStore.sync();

        return true;
      } else {
        logger.error('SSH', `Connection failed: ${result.error || 'Unknown error'}`);

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
      logger.error('SSH', `Connection exception: ${error instanceof Error ? error.message : 'Unknown error'}`);

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
      const result = await invoke<DisconnectResult>('disconnect');

      sessionStore.update((state) => ({
        ...state,
        connectionState: 'Disconnected',
        session_info: null,
        lastError: result.error || null,
      }));

      // Keep cached jobs when disconnecting - allows offline viewing
      // Jobs remain in store for offline access

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
      const result = await invoke<ConnectionStatusResult>('get_connection_status');

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

  // Mark connection as expired (called when connection failure detected)
  markExpired(error: string): void {
    sessionStore.update((state) => ({
      ...state,
      connectionState: 'Expired',
      lastError: error,
    }));
  },

  // Reset session to initial state
  reset(): void {
    sessionStore.set(initialState);
  },
};

// Export the store for subscription
export const session = { subscribe: sessionStore.subscribe };