import { writable, derived } from 'svelte/store';
import type { Job } from '../types/api';

export type View = 'jobs' | 'create';

interface UIState {
  currentView: View;
  selectedJobId: string | null;
  consoleOpen: boolean;
  sidebarCollapsed: boolean;
  theme: 'light' | 'dark';
}

interface BreadcrumbItem {
  label: string;
  onClick?: () => void;
}

// Initialize UI state
const initialState: UIState = {
  currentView: 'jobs',
  selectedJobId: null,
  consoleOpen: false,
  sidebarCollapsed: false,
  theme: 'light'
};

// Create the main UI store
function createUIStore() {
  const { subscribe, set, update } = writable<UIState>(initialState);

  return {
    subscribe,
    setView: (view: View) => update(state => ({
      ...state,
      currentView: view,
      selectedJobId: null // Clear selection when changing views
    })),
    selectJob: (jobId: string | null) => update(state => ({
      ...state,
      selectedJobId: jobId
    })),
    toggleConsole: () => update(state => ({
      ...state,
      consoleOpen: !state.consoleOpen
    })),
    toggleSidebar: () => update(state => ({
      ...state,
      sidebarCollapsed: !state.sidebarCollapsed
    })),
    setTheme: (theme: 'light' | 'dark') => {
      update(state => ({ ...state, theme }));
      // Apply theme to document
      if (theme === 'dark') {
        document.documentElement.setAttribute('data-theme', 'dark');
      } else {
        document.documentElement.removeAttribute('data-theme');
      }
    },
    reset: () => set(initialState)
  };
}

export const uiStore = createUIStore();

// Derived stores for convenience
export const currentView = derived(uiStore, $ui => $ui.currentView);
export const selectedJobId = derived(uiStore, $ui => $ui.selectedJobId);
export const consoleOpen = derived(uiStore, $ui => $ui.consoleOpen);
export const sidebarCollapsed = derived(uiStore, $ui => $ui.sidebarCollapsed);
export const theme = derived(uiStore, $ui => $ui.theme);

// Breadcrumb generation based on current state
export const breadcrumbs = derived(
  [uiStore],
  ([$ui]) => {
    const items: BreadcrumbItem[] = [];

    // Always start with Jobs
    items.push({
      label: 'Jobs',
      onClick: $ui.currentView !== 'jobs' || $ui.selectedJobId ?
        () => uiStore.setView('jobs') : undefined
    });

    // Add current view specifics
    if ($ui.currentView === 'create') {
      items.push({ label: 'Create New Job' });
    } else if ($ui.currentView === 'jobs' && $ui.selectedJobId) {
      items.push({ label: 'Job Details' });
    }

    return items;
  }
);