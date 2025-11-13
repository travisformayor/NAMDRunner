import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { DatabaseInfo, DatabaseOperationData, ApiResult } from '../types/api';
import { logger } from '../utils/logger';

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
      logger.debug('Settings', 'Loading database info');
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
        logger.error('Settings', `Failed to load database info: ${result.error}`);
        update(state => ({ ...state, loading: false }));
      }
    },

    async backupDatabase() {
      logger.debug('Settings', 'Starting backup');
      const result = await invoke<ApiResult<DatabaseOperationData>>('backup_database');

      if (result.success && result.data) {
        logger.debug('Settings', result.data.message || 'Backup successful');
      } else if (result.error !== 'Backup cancelled') {
        logger.error('Settings', result.error || 'Backup failed');
      }

      return result;
    },

    async restoreDatabase() {
      logger.debug('Settings', 'Starting restore');
      const result = await invoke<ApiResult<DatabaseOperationData>>('restore_database');

      if (result.success && result.data) {
        logger.debug('Settings', result.data.message || 'Restore successful');
        // Reload database info after restore
        await settingsStore.loadDatabaseInfo();
      } else if (result.error !== 'Restore cancelled') {
        logger.error('Settings', result.error || 'Restore failed');
      }

      return result;
    },

    async resetDatabase() {
      logger.debug('Settings', 'Resetting database');
      const result = await invoke<ApiResult<DatabaseOperationData>>('reset_database');

      if (result.success && result.data) {
        logger.debug('Settings', result.data.message || 'Reset successful');
        // Reload database info after reset
        await settingsStore.loadDatabaseInfo();
      } else {
        logger.error('Settings', result.error || 'Reset failed');
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
