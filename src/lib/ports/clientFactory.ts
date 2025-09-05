import type { ICoreClient } from './coreClient';
import { TauriCoreClient } from './coreClient-tauri';
import { MockCoreClient } from './coreClient-mock';

/**
 * Client factory for creating the appropriate IPC client
 * Uses environment detection to determine whether to use mock or production client
 */
export class CoreClientFactory {
  private static instance: ICoreClient | null = null;

  /**
   * Get the core client instance (singleton)
   * @param forceMock - Force mock client even in production mode
   */
  static getClient(forceMock: boolean = false): ICoreClient {
    if (this.instance === null) {
      // Determine if we should use mock client
      const useMock =
        forceMock ||
        import.meta.env.DEV ||
        import.meta.env.MODE === 'test' ||
        typeof window !== 'undefined' &&
          window.location.hostname === 'localhost';

      if (useMock) {
        console.log('🔧 Using mock core client for development');
        this.instance = new MockCoreClient();
      } else {
        console.log('🚀 Using production Tauri core client');
        this.instance = new TauriCoreClient();
      }
    }

    return this.instance;
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