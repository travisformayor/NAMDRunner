/**
 * Settings Store - REFACTORED EXAMPLE using storeFactory
 *
 * This is a demonstration of how settings.ts can be refactored to use the store factory.
 * Compare with settings.ts to see the reduction in boilerplate.
 */

import { derived } from 'svelte/store';
import type { DatabaseInfo, DatabaseOperationData, ApiResult } from '../types/api';
import { createStore, invokeWithErrorHandling } from './storeFactory';

// Settings store using factory
const settingsStoreBase = createStore<DatabaseInfo | null>({
  initialData: null,
  loadCommand: 'get_database_info',
});

// Add custom methods for database operations
export const settingsStore = {
  ...settingsStoreBase,

  // loadDatabaseInfo is now just an alias for the built-in load()
  loadDatabaseInfo: async (): Promise<void> => {
    await settingsStoreBase.load();
  },

  // Custom method: restore database
  restoreDatabase: async (): Promise<ApiResult<DatabaseOperationData>> => {
    const result = await invokeWithErrorHandling<DatabaseOperationData>(
      'restore_database'
    );

    if (result.success && result.data) {
      // Reload database info after restore
      await settingsStoreBase.load();
      return { success: true, data: result.data };
    }

    return { success: false, error: result.error || 'Restore failed' };
  },

  // Custom method: reset database
  resetDatabase: async (): Promise<ApiResult<DatabaseOperationData>> => {
    const result = await invokeWithErrorHandling<DatabaseOperationData>(
      'reset_database'
    );

    if (result.success && result.data) {
      // Reload database info after reset
      await settingsStoreBase.load();
      return { success: true, data: result.data };
    }

    return { success: false, error: result.error || 'Reset failed' };
  },
};

// Derived stores for convenience - access the inner state
export const databaseInfo = derived(settingsStore, ($store) => $store.data);
export const settingsLoading = derived(settingsStore, ($store) => $store.loading);
export const settingsError = derived(settingsStore, ($store) => $store.error);

/**
 * COMPARISON with original settings.ts:
 *
 * BEFORE (settings.ts):
 * - 90 lines of code
 * - Manual state management with update() calls
 * - Manual loading/error state handling
 * - Repetitive try/catch blocks
 * - ApiResult handling duplicated in each method
 *
 * AFTER (settings.refactored.ts):
 * - 65 lines of code (28% reduction)
 * - Store factory handles state management
 * - Built-in load() method with loading/error states
 * - Single invokeWithErrorHandling utility for custom methods
 * - ApiResult handling centralized
 *
 * KEY BENEFITS:
 * - Less code to write and maintain
 * - Consistent patterns across all stores
 * - Connection error handling available (can enable if needed)
 * - Built-in reset(), setData(), clearError() methods
 * - Type-safe with minimal boilerplate
 */
