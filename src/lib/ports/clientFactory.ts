import type { ICoreClient } from './coreClient';
import { TauriCoreClient } from './coreClient-tauri';
import { MockCoreClient } from './coreClient-mock';

export type AppMode = 'demo' | 'real';

/**
 * Client factory for creating the appropriate IPC client
 * Supports user mode preference with localStorage persistence
 */
export class CoreClientFactory {
  private static instance: ICoreClient | null = null;
  private static readonly MODE_STORAGE_KEY = 'namdrunner-app-mode';

  /**
   * Get the user's preferred mode from localStorage
   */
  static getUserMode(): AppMode {
    if (typeof window === 'undefined') return 'real';
    const stored = window.localStorage.getItem(this.MODE_STORAGE_KEY);
    return (stored as AppMode) || 'real';
  }

  /**
   * Set the user's preferred mode and persist to localStorage
   */
  static async setUserMode(mode: AppMode): Promise<void> {
    if (typeof window !== 'undefined') {
      window.localStorage.setItem(this.MODE_STORAGE_KEY, mode);
    }

    // Sync mode with backend
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('set_app_mode', { isDemo: mode === 'demo' });
      // Backend mode synced successfully
    } catch (error) {
      // Failed to sync mode with backend - continuing with current mode
    }

    // Reset instance to force recreating with new mode
    this.reset();
  }

  /**
   * Get the core client instance (singleton)
   * @param forceMock - Force mock client even in production mode (for testing)
   */
  static getClient(forceMock: boolean = false): ICoreClient {
    if (this.instance === null) {
      const userMode = this.getUserMode();

      // Sync mode with backend on first access
      this.syncModeWithBackend(userMode);

      // Determine if we should use mock client
      const useMock = this.shouldUseMock(userMode, forceMock);

      if (useMock) {
        // Using demo mode (mock client)
        this.instance = new MockCoreClient();
      } else {
        // Using real mode (Tauri client)
        this.instance = new TauriCoreClient();
      }
    } else {
      // Reusing existing client instance
    }

    return this.instance;
  }

  /**
   * Sync mode with backend (fire and forget)
   */
  private static async syncModeWithBackend(mode: AppMode): Promise<void> {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('set_app_mode', { isDemo: mode === 'demo' });
      // Initial backend mode synced
    } catch (error) {
      // Failed to sync initial mode with backend
    }
  }

  /**
   * Determine whether to use mock client based on user preference
   */
  private static shouldUseMock(userMode: AppMode, forceMock: boolean): boolean {
    // Force mock overrides everything (for testing)
    if (forceMock) return true;

    // Simple binary choice: demo mode on or off
    return userMode === 'demo';
  }

  /**
   * Reset the client instance (useful for testing)
   */
  static reset(): void {
    this.instance = null;
  }

  /**
   * Check if we're currently using the mock client
   */
  static isMockClient(): boolean {
    return this.instance instanceof MockCoreClient;
  }
}

// Convenience export
export const coreClient = CoreClientFactory.getClient();