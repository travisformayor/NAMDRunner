# React Mockup Analysis and Svelte Component Architecture

## Overview

This document analyzes the React mockup implementation in `docs/design_mockup/` to guide the Svelte component architecture.

## Visual Reference

For visual context of the UI patterns analyzed below, see mockup screenshots in: `docs/design_mockup/mockup_screenshots/`

Available screenshots:
- `main-view.png` - Primary jobs table with sidebar navigation
- `job-details-view.png` - Job detail page with tabbed interface
- `connected-dropdown.png` - Connection status dropdown (connected state)
- `disconnected-dropdown.png` - Connection status dropdown (login form)
- `ssh-console-open.png` - Collapsible SSH console panel

## React Mockup Component Inventory

### Application Structure
The React mockup follows a clean, hierarchical structure with these main sections:

#### Core Layout Components
- **App.tsx** - Root component with routing and global state
- **layout/Sidebar.tsx** - Navigation sidebar with view switching
- **layout/ConnectionStatus.tsx** - SSH connection dropdown with form
- **layout/Breadcrumbs.tsx** - Navigation breadcrumb trail
- **layout/SSHConsole.tsx** - Collapsible footer console

#### Jobs Management
- **jobs/JobsView.tsx** - Jobs page container with header/actions
- **jobs/JobTable.tsx** - Sortable table with row selection
- **jobs/JobStatusBadge.tsx** - Status pill components
- **jobs/SyncStatus.tsx** - Sync controls and timestamp display

#### Job Detail Views
- **job-detail/JobDetailView.tsx** - Tabbed detail page
- **job-detail/JobSummaryCard.tsx** - Job overview card
- **job-detail/JobTabs.tsx** - Tab navigation
- **job-detail/DeleteJobDialog.tsx** - Confirmation modal
- **job-detail/tabs/** - Individual tab content components

#### Job Creation Workflow
- **job-create/CreateJobView.tsx** - Multi-section form container
- **job-create/ResourceForm.tsx** - SLURM resource allocation
- **job-create/FileUploadArea.tsx** - Drag & drop file upload
- **job-create/NAMDConfigForm.tsx** - NAMD parameters form

#### UI Foundation
- **ui/** - 50+ shadcn/ui components (buttons, inputs, dialogs, etc.)

### Key React Patterns Observed

#### State Management
- **Local useState** for component state (form data, UI state)
- **Props drilling** for data flow between parent/child components
- **Callback props** for event handling (onJobSelect, onStateChange)
- **Controlled components** for all form inputs

#### UI Patterns
- **Popover-based dropdowns** for connection status
- **Table sorting** with chevron indicators
- **Modal dialogs** for destructive actions
- **Form validation** with inline error display
- **Loading states** with disabled buttons and loading text

#### Data Flow
- **Top-down data flow** from App.tsx to child components
- **Event bubbling** for user interactions back to parent
- **Mock data** embedded in App.tsx for demonstration

## Svelte Component Architecture Design

### Component Tree Structure

```
App.svelte (Root)
├── lib/
│   ├── components/
│   │   ├── layout/
│   │   │   ├── AppSidebar.svelte
│   │   │   ├── ConnectionDropdown.svelte
│   │   │   ├── BreadcrumbNav.svelte
│   │   │   └── SSHConsolePanel.svelte
│   │   ├── jobs/
│   │   │   ├── JobsPage.svelte
│   │   │   ├── JobsTable.svelte
│   │   │   ├── StatusBadge.svelte
│   │   │   └── SyncControls.svelte
│   │   ├── job-detail/
│   │   │   ├── JobDetailPage.svelte
│   │   │   ├── JobSummary.svelte
│   │   │   ├── DetailTabs.svelte
│   │   │   └── tabs/
│   │   │       ├── OverviewTab.svelte
│   │   │       ├── LogsTab.svelte
│   │   │       ├── InputFilesTab.svelte
│   │   │       ├── OutputFilesTab.svelte
│   │   │       └── ConfigTab.svelte
│   │   ├── job-create/
│   │   │   ├── CreateJobPage.svelte
│   │   │   ├── ResourceSection.svelte
│   │   │   ├── FileUploadSection.svelte
│   │   │   └── NAMDConfigSection.svelte
│   │   └── common/
│   │       ├── Button.svelte
│   │       ├── Input.svelte
│   │       ├── Modal.svelte
│   │       ├── Tooltip.svelte
│   │       └── FormField.svelte
│   ├── stores/
│   │   ├── session.ts (connection state)
│   │   ├── jobs.ts (jobs data)
│   │   ├── ui.ts (view state, modals)
│   │   └── console.ts (SSH console)
│   └── types/
│       ├── job.ts
│       ├── connection.ts
│       └── ui.ts
└── routes/
    ├── +layout.svelte (main app layout)
    └── +page.svelte (route-based views)
```

### Svelte-Native Patterns

#### Store-Based State Management
Instead of React's props drilling, use Svelte stores:

```typescript
// stores/session.ts
export const connectionState = writable<ConnectionState>('disconnected');
export const connectionInfo = writable<ConnectionInfo | null>(null);

// stores/jobs.ts
export const allJobs = writable<Job[]>([]);
export const selectedJob = writable<Job | null>(null);

// stores/ui.ts
export const currentView = writable<'jobs' | 'create' | 'detail'>('jobs');
export const showDeleteModal = writable(false);
```

#### Reactive Statements
Replace React useEffect with Svelte's reactive statements:

```svelte
<script>
  import { connectionState, allJobs } from '$lib/stores';

  // Reactive derived values
  $: connectedJobs = $connectionState === 'connected' ? $allJobs : [];
  $: canCreateJob = $connectionState === 'connected';

  // Reactive side effects
  $: if ($connectionState === 'connected') {
    syncJobsFromServer();
  }
</script>
```

#### Component Communication
- **Stores** for global state (connection, jobs list)
- **Props** for parent → child data passing
- **Events** for child → parent communication
- **Context** for deeply nested component trees

#### Form Handling
Replace React controlled components with Svelte's two-way binding:

```svelte
<script>
  let resourceConfig = {
    cores: 128,
    memory: '512',
    wallTime: '04:00:00'
  };

  // Validation
  $: errors = validateResourceConfig(resourceConfig);
</script>

<input bind:value={resourceConfig.cores} type="number" />
<input bind:value={resourceConfig.memory} />
<input bind:value={resourceConfig.wallTime} pattern="\\d{2}:\\d{2}:\\d{2}" />
```

### Component Responsibilities

#### Layout Components
- **AppSidebar.svelte**: Navigation menu, active view highlighting
- **ConnectionDropdown.svelte**: SSH connection form/status, popover behavior
- **BreadcrumbNav.svelte**: Dynamic breadcrumb generation
- **SSHConsolePanel.svelte**: Collapsible console with command history

#### Data Components
- **JobsTable.svelte**: Sorting, filtering, row selection
- **StatusBadge.svelte**: Status-based styling and icons
- **SyncControls.svelte**: Manual/auto sync controls

#### Form Components
- **CreateJobPage.svelte**: Multi-step form orchestration
- **ResourceSection.svelte**: SLURM parameter validation
- **FileUploadSection.svelte**: Drag & drop with file validation
- **NAMDConfigSection.svelte**: Scientific parameter forms

### Store Architecture

#### Session Store (`stores/session.ts`)
```typescript
interface ConnectionState {
  status: 'disconnected' | 'connecting' | 'connected' | 'expired';
  host?: string;
  username?: string;
  connectedSince?: Date;
  lastError?: string;
}

export const session = writable<ConnectionState>({
  status: 'disconnected'
});

export const sessionActions = {
  connect: async (host: string, username: string, password: string) => {},
  disconnect: () => {},
  checkStatus: async () => {}
};
```

#### Jobs Store (`stores/jobs.ts`)
```typescript
export const jobs = writable<Job[]>([]);
export const selectedJobId = writable<string | null>(null);

// Derived stores
export const selectedJob = derived(
  [jobs, selectedJobId],
  ([$jobs, $selectedJobId]) =>
    $selectedJobId ? $jobs.find(j => j.id === $selectedJobId) : null
);

export const jobsByStatus = derived(jobs, ($jobs) =>
  groupBy($jobs, 'status')
);
```

#### UI Store (`stores/ui.ts`)
```typescript
export const currentView = writable<'jobs' | 'create' | 'detail'>('jobs');
export const showConsole = writable(false);
export const breadcrumbs = writable<BreadcrumbItem[]>([]);

// Modal states
export const modals = writable({
  deleteJob: false,
  connectionError: false
});
```

### Key Architectural Differences from React

#### 1. No Virtual DOM
- Direct DOM manipulation means more efficient updates
- No need for React keys or reconciliation concerns
- Component lifecycle is simpler

#### 2. Compile-Time Optimizations
- Svelte compiler optimizes reactivity at build time
- No runtime framework overhead
- Smaller bundle size

#### 3. Built-in State Management
- Stores are first-class citizens
- No need for external state management libraries
- Automatic subscription/unsubscription

#### 4. CSS Scoping
- Component-scoped CSS by default
- No need for CSS-in-JS libraries
- Can use CSS custom properties for theming

#### 5. Form Handling
- Two-way binding eliminates controlled component boilerplate
- Built-in form validation helpers
- Easier to manage form state

This architecture preserves the excellent UX patterns from the React mockup while leveraging Svelte's strengths for a more maintainable and performant implementation.