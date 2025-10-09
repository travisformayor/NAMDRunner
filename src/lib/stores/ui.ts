import { writable, derived } from 'svelte/store';

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
  onClick: (() => void) | undefined;
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
    setView: (view: View) => {
      if (typeof window !== 'undefined' && window.sshConsole) window.sshConsole.addDebug(`[UIStore] Setting view: ${view}`);
      update(state => ({
        ...state,
        currentView: view,
        selectedJobId: null // Clear selection when changing views
      }));
    },
    selectJob: (job_id: string | null) => update(state => ({
      ...state,
      selectedJobId: job_id
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
    const onClick = ($ui.currentView !== 'jobs' || $ui.selectedJobId) ?
      () => uiStore.setView('jobs') : undefined;

    items.push({
      label: 'Jobs',
      onClick
    });

    // Add current view specifics
    if ($ui.currentView === 'create') {
      items.push({ label: 'Create New Job', onClick: undefined });
    } else if ($ui.currentView === 'jobs' && $ui.selectedJobId) {
      items.push({ label: 'Job Details', onClick: undefined });
    }

    return items;
  }
);