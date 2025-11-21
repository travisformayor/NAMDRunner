import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { DatabaseInfo, DatabaseOperationData, ApiResult } from '../types/api';

// Settings store state
interface SettingsState {
  databaseInfo: DatabaseInfo | null;
  loading: boolean;
}

const initialState: SettingsState = {
  databaseInfo: null,
  loading: false,
};

// Create settings store
function createSettingsStore() {
  const { subscribe, update } = writable<SettingsState>(initialState);

  return {
    subscribe,

    // Load database information
    async loadDatabaseInfo(): Promise<void> {
      update(state => ({ ...state, loading: true }));

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
        }));
      } else {
        update(state => ({ ...state, loading: false }));
      }
    },

    async backupDatabase() {
      const result = await invoke<ApiResult<DatabaseOperationData>>('backup_database');

      return result;
    },

    async restoreDatabase() {
      const result = await invoke<ApiResult<DatabaseOperationData>>('restore_database');

      if (result.success && result.data) {
        // Reload database info after restore
        await settingsStore.loadDatabaseInfo();
      }

      return result;
    },

    async resetDatabase() {
      const result = await invoke<ApiResult<DatabaseOperationData>>('reset_database');

      if (result.success && result.data) {
        // Reload database info after reset
        await settingsStore.loadDatabaseInfo();
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
