import { writable, derived } from 'svelte/store';
import { logger } from '../utils/logger';

export type View = 'jobs' | 'create' | 'templates' | 'template-edit' | 'settings';

interface UIState {
  currentView: View;
  selectedJobId: string | null;
  selectedTemplateId: string | null;
  templateEditorMode: 'create' | 'edit';
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
  selectedTemplateId: null,
  templateEditorMode: 'create',
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
      logger.debug('UIStore', `Setting view: ${view}`);
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
    editTemplate: (template_id: string | null, mode: 'create' | 'edit' = 'edit') => update(state => ({
      ...state,
      currentView: 'template-edit',
      selectedTemplateId: template_id,
      templateEditorMode: mode
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
export const selectedTemplateId = derived(uiStore, $ui => $ui.selectedTemplateId);
export const templateEditorMode = derived(uiStore, $ui => $ui.templateEditorMode);
export const consoleOpen = derived(uiStore, $ui => $ui.consoleOpen);
export const sidebarCollapsed = derived(uiStore, $ui => $ui.sidebarCollapsed);
export const theme = derived(uiStore, $ui => $ui.theme);

// Breadcrumb generation based on current state
export const breadcrumbs = derived(
  [uiStore],
  ([$ui]) => {
    const items: BreadcrumbItem[] = [];

    // Determine top-level page (Jobs, Templates, or Settings)
    if ($ui.currentView === 'settings') {
      // Settings page
      items.push({
        label: 'Settings',
        onClick: undefined
      });
    } else if ($ui.currentView === 'templates' || $ui.currentView === 'template-edit') {
      // Templates hierarchy
      const onClick = ($ui.currentView !== 'templates') ?
        () => uiStore.setView('templates') : undefined;

      items.push({
        label: 'Templates',
        onClick
      });

      // Add template sub-pages
      if ($ui.currentView === 'template-edit') {
        items.push({
          label: $ui.templateEditorMode === 'create' ? 'Create Template' : 'Edit Template',
          onClick: undefined
        });
      }
    } else {
      // Jobs hierarchy (default)
      const onClick = ($ui.currentView !== 'jobs' || $ui.selectedJobId) ?
        () => uiStore.setView('jobs') : undefined;

      items.push({
        label: 'Jobs',
        onClick
      });

      // Add job sub-pages
      if ($ui.currentView === 'create') {
        items.push({ label: 'Create Job', onClick: undefined });
      } else if ($ui.currentView === 'jobs' && $ui.selectedJobId) {
        items.push({ label: 'Job Details', onClick: undefined });
      }
    }

    return items;
  }
);