import { writable } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';

export interface Toast {
  id: string;
  message: string;
  variant: 'success' | 'error' | 'warning' | 'info';
}

interface AppLogMessage {
  level: string;
  category: string;
  message: string;
  details?: string;
  show_toast: boolean;
  timestamp: string;
}

/**
 * Map log level to toast variant
 */
function mapLevel(level: string): Toast['variant'] {
  switch (level.toLowerCase()) {
    case 'error':
      return 'error';
    case 'warn':
      return 'warning';
    case 'info':
      return 'success';
    case 'debug':
      return 'info';
    default:
      return 'info';
  }
}

/**
 * Toast notification store
 * Automatically listens for app-log events from backend and displays toasts
 */
function createToastStore() {
  const { subscribe, update } = writable<Toast[]>([]);

  // Listen for app-log events from backend
  listen<AppLogMessage>('app-log', (event) => {
    const log = event.payload;

    // Only show toast if explicitly flagged
    if (log.show_toast) {
      const toast: Toast = {
        id: globalThis.crypto.randomUUID(),
        message: log.message,
        variant: mapLevel(log.level),
      };

      // Add toast to store
      update((toasts) => {
        // Limit to max 5 toasts - remove oldest if exceeding
        const newToasts = [...toasts, toast];
        if (newToasts.length > 5) {
          newToasts.shift(); // Remove oldest
        }
        return newToasts;
      });

      // Auto-dismiss after 4 seconds
      setTimeout(() => {
        update((toasts) => toasts.filter((t) => t.id !== toast.id));
      }, 4000);
    }
  });

  return {
    subscribe,
    dismiss: (id: string) => {
      update((toasts) => toasts.filter((t) => t.id !== id));
    },
  };
}

export const toasts = createToastStore();
