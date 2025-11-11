import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { DatabaseInfo, DatabaseInfoResult, DatabaseOperationResult } from '../types/api';
import { logger } from '../utils/logger';

interface SettingsState {
  databaseInfo: DatabaseInfo | null;
  isLoading: boolean;
}

const initialState: SettingsState = {
  databaseInfo: null,
  isLoading: false,
};

function createSettingsStore() {
  const { subscribe, set, update } = writable<SettingsState>(initialState);

  return {
    subscribe,

    async loadDatabaseInfo() {
      logger.debug('Settings', 'Loading database info');
      update(state => ({ ...state, isLoading: true }));

      const result = await invoke<DatabaseInfoResult>('get_database_info');

      if (result.success && result.path && result.size_bytes !== undefined) {
        update(state => ({
          ...state,
          databaseInfo: {
            path: result.path!,
            size_bytes: result.size_bytes!,
          },
          isLoading: false,
        }));
      } else {
        logger.error('Settings', `Failed to load database info: ${result.error}`);
        update(state => ({ ...state, isLoading: false }));
      }
    },

    async backupDatabase(): Promise<DatabaseOperationResult> {
      logger.debug('Settings', 'Starting backup');
      const result = await invoke<DatabaseOperationResult>('backup_database');

      if (result.success) {
        logger.debug('Settings', result.message || 'Backup successful');
      } else if (result.error !== 'Backup cancelled') {
        logger.error('Settings', result.error || 'Backup failed');
      }

      return result;
    },

    async restoreDatabase(): Promise<DatabaseOperationResult> {
      logger.debug('Settings', 'Starting restore');
      const result = await invoke<DatabaseOperationResult>('restore_database');

      if (result.success) {
        logger.debug('Settings', result.message || 'Restore successful');
        // Reload database info after restore
        await settingsStore.loadDatabaseInfo();
      } else if (result.error !== 'Restore cancelled') {
        logger.error('Settings', result.error || 'Restore failed');
      }

      return result;
    },

    async resetDatabase(): Promise<DatabaseOperationResult> {
      logger.debug('Settings', 'Resetting database');
      const result = await invoke<DatabaseOperationResult>('reset_database');

      if (result.success) {
        logger.debug('Settings', result.message || 'Reset successful');
        // Reload database info after reset
        await settingsStore.loadDatabaseInfo();
      } else {
        logger.error('Settings', result.error || 'Reset failed');
      }

      return result;
    },
  };
}

export const settingsStore = createSettingsStore();
