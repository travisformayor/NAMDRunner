import type { ICoreClient } from './coreClient';
import { TauriCoreClient } from './coreClient-tauri';

/**
 * Client factory for creating the Tauri IPC client
 * Simple singleton pattern
 */
export class CoreClientFactory {
  private static instance: ICoreClient | null = null;

  /**
   * Get the core client instance (singleton)
   */
  static getClient(): ICoreClient {
    if (this.instance === null) {
      this.instance = new TauriCoreClient();
    }
    return this.instance;
  }

  /**
   * Reset the client instance (useful for testing)
   */
  static reset(): void {
    this.instance = null;
  }
}

// Convenience export
export const coreClient = CoreClientFactory.getClient();