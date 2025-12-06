import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { DatabaseInfo, DatabaseOperationData, ApiResult } from '../types/api';

// Settings store state
interface SettingsState {
  databaseInfo: DatabaseInfo | null;
  loading: boolean;
  error: string | null;
}

const initialState: SettingsState = {
  databaseInfo: null,
  loading: false,
  error: null,
};

// Create settings store
function createSettingsStore() {
  const { subscribe, update } = writable<SettingsState>(initialState);

  return {
    subscribe,

    async loadDatabaseInfo(): Promise<void> {
      update(state => ({ ...state, loading: true, error: null }));

      try {
        const result = await invoke<ApiResult<DatabaseInfo>>('get_database_info');

        if (result.success && result.data) {
          const { path, size_bytes, job_count } = result.data;
          update(state => ({
            ...state,
            databaseInfo: {
              path,
              size_bytes,
              job_count,
            },
            loading: false,
            error: null,
          }));
        } else {
          update(state => ({
            ...state,
            loading: false,
            error: result.error || 'Failed to load database information',
          }));
        }
      } catch (error) {
        update(state => ({
          ...state,
          loading: false,
          error: `Error loading database information: ${error}`,
        }));
      }
    },

    async restoreDatabase(): Promise<ApiResult<DatabaseOperationData>> {
      const result = await invoke<ApiResult<DatabaseOperationData>>('restore_database');

      if (result.success && result.data) {
        // Reload database info after restore
        await this.loadDatabaseInfo();
      }

      return result;
    },

    async resetDatabase(): Promise<ApiResult<DatabaseOperationData>> {
      const result = await invoke<ApiResult<DatabaseOperationData>>('reset_database');

      if (result.success && result.data) {
        // Reload database info after reset
        await this.loadDatabaseInfo();
      }

      return result;
    },
  };
}

// Export store instance
export const settingsStore = createSettingsStore();

// Derived stores for convenience
export const databaseInfo = derived(settingsStore, $store => $store.databaseInfo);
export const settingsLoading = derived(settingsStore, $store => $store.loading);
export const settingsError = derived(settingsStore, $store => $store.error);
