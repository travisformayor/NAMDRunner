# Svelte Implementation Guide for NAMDRunner

This comprehensive guide documents how to implement NAMDRunner's UI using Svelte, translating React mockup patterns into idiomatic Svelte code while leveraging the framework's strengths.

## Overview

This guide translates the React mockup implementation (originally in `docs/design_mockup/`) into proper Svelte patterns and best practices. The goal is to achieve the same user experience while leveraging Svelte's compile-time optimizations, built-in reactivity, and simpler state management.

## Visual Reference

Refer to mockup screenshots for UI context in the design specifications and mockup materials. These screenshots show the design system in action across different states and views:
- Main jobs table with sidebar navigation
- Job detail page with tabbed interface
- Connection status dropdown (connected/disconnected states)
- Collapsible SSH console panel

## Core Architectural Patterns

### 1. Store-Based State Management

**React Pattern (Props Drilling)**
```tsx
// React: Data flows down through props
function App() {
  const [connectionState, setConnectionState] = useState('disconnected');
  const [jobs, setJobs] = useState([]);

  return (
    <Sidebar connectionState={connectionState} />
    <JobsView jobs={jobs} connectionState={connectionState} />
  );
}
```

**Svelte Pattern (Stores)**
```typescript
// stores/session.ts
export const connectionState = writable<ConnectionState>('disconnected');

// stores/jobs.ts
export const jobs = writable<Job[]>([]);

// Any component can access these stores
```

```svelte
<!-- AppSidebar.svelte -->
<script>
  import { connectionState } from '$lib/stores/session';
  // Reactive - automatically updates when store changes
  $: canCreateJob = $connectionState === 'connected';
</script>

<button disabled={!canCreateJob}>Create Job</button>
```

### 2. Reactive Statements vs useEffect

**React Pattern**
```tsx
function JobsView({ connectionState, jobs }) {
  const [filteredJobs, setFilteredJobs] = useState([]);

  useEffect(() => {
    if (connectionState === 'connected') {
      setFilteredJobs(jobs.filter(job => job.status !== 'CANCELLED'));
    } else {
      setFilteredJobs([]);
    }
  }, [connectionState, jobs]);

  return <JobTable jobs={filteredJobs} />;
}
```

**Svelte Pattern**
```svelte
<!-- JobsPage.svelte -->
<script>
  import { connectionState } from '$lib/stores/session';
  import { jobs } from '$lib/stores/jobs';

  // Reactive statement - automatically recalculates when dependencies change
  $: filteredJobs = $connectionState === 'connected'
    ? $jobs.filter(job => job.status !== 'CANCELLED')
    : [];
</script>

<JobTable jobs={filteredJobs} />
```

### 3. Form Handling with Two-Way Binding

**React Pattern (Controlled Components)**
```tsx
function ResourceForm({ config, onChange }) {
  const handleCoresChange = (e) => {
    onChange({
      ...config,
      cores: parseInt(e.target.value)
    });
  };

  return (
    <input
      value={config.cores}
      onChange={handleCoresChange}
      type="number"
    />
  );
}
```

**Svelte Pattern (Two-Way Binding)**
```svelte
<!-- ResourceSection.svelte -->
<script>
  export let config;

  // Validation happens automatically when config changes
  $: errors = validateResourceConfig(config);

  // Can also have reactive side effects
  $: if (config.cores > 256) {
    console.warn('High core count detected');
  }
</script>

<input bind:value={config.cores} type="number" />

{#if errors.cores}
  <p class="error">{errors.cores}</p>
{/if}
```

### 4. Event Handling and Component Communication

**React Pattern (Callback Props)**
```tsx
function JobTable({ jobs, onJobSelect }) {
  return (
    <tbody>
      {jobs.map(job => (
        <tr key={job.id} onClick={() => onJobSelect(job.id)}>
          <td>{job.name}</td>
        </tr>
      ))}
    </tbody>
  );
}
```

**Svelte Pattern (Custom Events)**
```svelte
<!-- JobsTable.svelte -->
<script>
  import { createEventDispatcher } from 'svelte';

  export let jobs;

  const dispatch = createEventDispatcher();

  function selectJob(jobId) {
    dispatch('jobSelect', { jobId });
  }
</script>

<tbody>
  {#each jobs as job (job.id)}
    <tr on:click={() => selectJob(job.id)}>
      <td>{job.name}</td>
    </tr>
  {/each}
</tbody>
```

```svelte
<!-- JobsPage.svelte -->
<script>
  import { selectedJobId } from '$lib/stores/jobs';

  function handleJobSelect(event) {
    $selectedJobId = event.detail.jobId;
  }
</script>

<JobsTable {jobs} on:jobSelect={handleJobSelect} />
```

### 5. Conditional Rendering and Lists

**React Pattern**
```tsx
function JobDetailView({ job }) {
  if (!job) {
    return <div>Select a job to view details</div>;
  }

  return (
    <div>
      <h2>{job.name}</h2>
      {job.logs && (
        <div>
          <h3>Logs</h3>
          <pre>{job.logs}</pre>
        </div>
      )}
    </div>
  );
}
```

**Svelte Pattern**
```svelte
<!-- JobDetailPage.svelte -->
<script>
  export let job;
</script>

{#if !job}
  <div>Select a job to view details</div>
{:else}
  <div>
    <h2>{job.name}</h2>

    {#if job.logs}
      <div>
        <h3>Logs</h3>
        <pre>{job.logs}</pre>
      </div>
    {/if}
  </div>
{/if}
```

### 6. Modal and Popup Management

**React Pattern (State-Based Modals)**
```tsx
function JobDetailView({ job }) {
  const [showDeleteModal, setShowDeleteModal] = useState(false);

  return (
    <>
      <button onClick={() => setShowDeleteModal(true)}>Delete</button>

      {showDeleteModal && (
        <DeleteJobModal
          job={job}
          onClose={() => setShowDeleteModal(false)}
          onConfirm={() => {/* handle delete */}}
        />
      )}
    </>
  );
}
```

**Svelte Pattern (Store-Based Modals)**
```typescript
// stores/modals.ts
export const modals = writable({
  deleteJob: false,
  connectionError: false
});

export const modalData = writable({});

export function openDeleteJobModal(job) {
  modalData.set({ job });
  modals.update(m => ({ ...m, deleteJob: true }));
}
```

```svelte
<!-- JobDetailPage.svelte -->
<script>
  import { openDeleteJobModal } from '$lib/stores/modals';

  export let job;
</script>

<button on:click={() => openDeleteJobModal(job)}>Delete</button>
```

```svelte
<!-- App.svelte - Root level modal management -->
<script>
  import { modals } from '$lib/stores/modals';
  import DeleteJobModal from '$lib/components/modals/DeleteJobModal.svelte';
</script>

<!-- Main app content -->
<main>...</main>

<!-- Global modals -->
{#if $modals.deleteJob}
  <DeleteJobModal />
{/if}
```

## Component-Specific Patterns

### 1. Connection Status Dropdown

**React Pattern (useState + Popover)**
```tsx
function ConnectionStatus() {
  const [host, setHost] = useState('');
  const [username, setUsername] = useState('');
  const [isOpen, setIsOpen] = useState(false);

  return (
    <Popover open={isOpen} onOpenChange={setIsOpen}>
      {/* Popover content */}
    </Popover>
  );
}
```

**Svelte Pattern (Stores + Native Details/Summary)**
```svelte
<!-- ConnectionDropdown.svelte -->
<script>
  import { connectionState, connectionInfo } from '$lib/stores/session';

  let isOpen = false;
  let credentials = { host: '', username: '', password: '' };

  // Reactive status info
  $: statusInfo = getStatusInfo($connectionState);

  async function handleConnect() {
    // Use action from store
    await sessionActions.connect(credentials);
  }
</script>

<details bind:open={isOpen}>
  <summary>
    <div class="status-indicator status-indicator--{$connectionState}"></div>
    {statusInfo.label}
  </summary>

  <div class="dropdown-content">
    {#if $connectionState === 'connected'}
      <!-- Connected state UI -->
    {:else}
      <!-- Connection form -->
      <input bind:value={credentials.host} placeholder="Host" />
      <input bind:value={credentials.username} placeholder="Username" />
      <input bind:value={credentials.password} type="password" placeholder="Password" />
      <button on:click={handleConnect}>Connect</button>
    {/if}
  </div>
</details>
```

### 2. Sortable Table with State

**React Pattern**
```tsx
function JobTable({ jobs }) {
  const [sortField, setSortField] = useState('createdDate');
  const [sortDirection, setSortDirection] = useState('desc');

  const sortedJobs = useMemo(() => {
    return [...jobs].sort((a, b) => {
      // Sort logic
    });
  }, [jobs, sortField, sortDirection]);

  return (
    <table>
      <thead>
        <tr>
          <th onClick={() => handleSort('name')}>Name</th>
        </tr>
      </thead>
      <tbody>
        {sortedJobs.map(job => (
          <JobRow key={job.id} job={job} />
        ))}
      </tbody>
    </table>
  );
}
```

**Svelte Pattern**
```svelte
<!-- JobsTable.svelte -->
<script>
  export let jobs;

  let sortField = 'createdDate';
  let sortDirection = 'desc';

  // Reactive sorted jobs
  $: sortedJobs = [...jobs].sort((a, b) => {
    const aValue = a[sortField];
    const bValue = b[sortField];

    if (typeof aValue === 'string') {
      const result = aValue.localeCompare(bValue);
      return sortDirection === 'asc' ? result : -result;
    }

    return sortDirection === 'asc' ? aValue - bValue : bValue - aValue;
  });

  function handleSort(field) {
    if (sortField === field) {
      sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
    } else {
      sortField = field;
      sortDirection = 'asc';
    }
  }
</script>

<table>
  <thead>
    <tr>
      <th on:click={() => handleSort('name')}>
        Name
        {#if sortField === 'name'}
          <span class="sort-indicator sort-indicator--{sortDirection}"></span>
        {/if}
      </th>
    </tr>
  </thead>
  <tbody>
    {#each sortedJobs as job (job.id)}
      <JobRow {job} />
    {/each}
  </tbody>
</table>
```

### 3. Multi-Step Form with Validation

**React Pattern**
```tsx
function CreateJobView() {
  const [step, setStep] = useState(1);
  const [formData, setFormData] = useState({});
  const [errors, setErrors] = useState({});

  const validateStep = (stepNumber) => {
    // Validation logic
  };

  const nextStep = () => {
    if (validateStep(step)) {
      setStep(step + 1);
    }
  };
}
```

**Svelte Pattern**
```svelte
<!-- CreateJobPage.svelte -->
<script>
  import { createJobForm } from '$lib/stores/forms';
  import { derived } from 'svelte/store';

  // Use a store for complex form state
  const form = createJobForm();

  // Derived validation
  const validation = derived(form, ($form) => validateForm($form));

  let currentStep = 1;

  // Reactive step validation
  $: canAdvance = $validation.steps[currentStep - 1].isValid;

  function nextStep() {
    if (canAdvance) {
      currentStep++;
    }
  }
</script>

<div class="form-steps">
  {#if currentStep === 1}
    <ResourceSection bind:config={$form.resources} errors={$validation.resources} />
  {:else if currentStep === 2}
    <FileUploadSection bind:files={$form.files} errors={$validation.files} />
  {:else if currentStep === 3}
    <NAMDConfigSection bind:config={$form.namd} errors={$validation.namd} />
  {/if}

  <div class="form-actions">
    {#if currentStep > 1}
      <button on:click={() => currentStep--}>Back</button>
    {/if}

    {#if currentStep < 3}
      <button on:click={nextStep} disabled={!canAdvance}>Next</button>
    {:else}
      <button on:click={submitForm} disabled={!$validation.isValid}>Create Job</button>
    {/if}
  </div>
</div>
```

## Store Patterns for Complex State

### 1. Session Store with Actions
```typescript
// stores/session.ts
import { writable, derived } from 'svelte/store';

interface ConnectionState {
  status: 'disconnected' | 'connecting' | 'connected' | 'expired';
  host?: string;
  username?: string;
  connectedSince?: Date;
  lastError?: string;
}

function createSessionStore() {
  const { subscribe, set, update } = writable<ConnectionState>({
    status: 'disconnected'
  });

  return {
    subscribe,
    connect: async (host: string, username: string, password: string) => {
      update(s => ({ ...s, status: 'connecting' }));

      try {
        // Call Tauri backend
        await invoke('ssh_connect', { host, username, password });
        set({
          status: 'connected',
          host,
          username,
          connectedSince: new Date()
        });
      } catch (error) {
        set({
          status: 'disconnected',
          lastError: error.message
        });
      }
    },
    disconnect: () => {
      invoke('ssh_disconnect');
      set({ status: 'disconnected' });
    },
    checkStatus: async () => {
      const status = await invoke('ssh_status');
      update(s => ({ ...s, status }));
    }
  };
}

export const session = createSessionStore();

// Derived stores
export const isConnected = derived(session, $session => $session.status === 'connected');
export const connectionError = derived(session, $session => $session.lastError);
```

### 2. Jobs Store with Sync Logic
```typescript
// stores/jobs.ts
import { writable, derived } from 'svelte/store';

function createJobsStore() {
  const { subscribe, set, update } = writable<Job[]>([]);

  return {
    subscribe,
    load: async () => {
      const jobs = await invoke('get_jobs');
      set(jobs);
    },
    sync: async () => {
      const updatedJobs = await invoke('sync_jobs');
      set(updatedJobs);
    },
    create: async (jobData: CreateJobRequest) => {
      const newJob = await invoke('create_job', jobData);
      update(jobs => [...jobs, newJob]);
      return newJob;
    },
    delete: async (jobId: string) => {
      await invoke('delete_job', { jobId });
      update(jobs => jobs.filter(j => j.id !== jobId));
    }
  };
}

export const jobs = createJobsStore();
export const selectedJobId = writable<string | null>(null);

// Derived stores
export const selectedJob = derived(
  [jobs, selectedJobId],
  ([$jobs, $selectedJobId]) =>
    $selectedJobId ? $jobs.find(j => j.id === $selectedJobId) : null
);

export const jobsByStatus = derived(jobs, $jobs => {
  return {
    running: $jobs.filter(j => j.status === 'RUNNING'),
    pending: $jobs.filter(j => j.status === 'PENDING'),
    completed: $jobs.filter(j => j.status === 'COMPLETED'),
    failed: $jobs.filter(j => j.status === 'FAILED')
  };
});
```

## Design System Implementation

### CSS Custom Properties Setup

```css
/* src/app.css */
:root {
  /* Theme Colors */
  --namd-bg-primary: #ffffff;
  --namd-bg-secondary: #f8fafc;
  --namd-text-primary: #1f2937;
  --namd-text-secondary: #6b7280;
  --namd-text-muted: #9ca3af;

  /* Interactive Elements */
  --namd-primary: #030213;
  --namd-primary-fg: #ffffff;
  --namd-secondary: #f3f4f6;
  --namd-secondary-fg: #1f2937;

  /* Status Colors */
  --namd-success: #10b981;
  --namd-success-bg: #ecfdf5;
  --namd-warning: #f59e0b;
  --namd-warning-bg: #fffbeb;
  --namd-error: #ef4444;
  --namd-error-bg: #fef2f2;
  --namd-info: #3b82f6;
  --namd-info-bg: #eff6ff;

  /* Layout */
  --namd-border: rgba(0, 0, 0, 0.1);
  --namd-border-radius: 0.625rem;
  --namd-sidebar-width: 12rem;

  /* Typography */
  --namd-font-sans: ui-sans-serif, system-ui, sans-serif;
  --namd-font-mono: ui-monospace, 'SF Mono', Monaco, monospace;
  --namd-font-size-base: 0.875rem;
  --namd-font-weight-normal: 400;
  --namd-font-weight-medium: 500;

  /* Spacing */
  --namd-spacing-xs: 0.25rem;
  --namd-spacing-sm: 0.5rem;
  --namd-spacing-md: 1rem;
  --namd-spacing-lg: 1.5rem;
  --namd-spacing-xl: 2rem;
}

/* Dark theme */
@media (prefers-color-scheme: dark) {
  :root {
    --namd-bg-primary: #0f172a;
    --namd-bg-secondary: #1e293b;
    --namd-text-primary: #f1f5f9;
    --namd-text-secondary: #cbd5e1;
    --namd-text-muted: #64748b;

    --namd-primary: #f1f5f9;
    --namd-primary-fg: #1e293b;
    --namd-secondary: #334155;
    --namd-secondary-fg: #f1f5f9;

    --namd-border: rgba(255, 255, 255, 0.1);
  }
}
```

### Component Architecture

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
│   │   ├── job-create/
│   │   │   ├── CreateJobPage.svelte
│   │   │   ├── ResourceSection.svelte
│   │   │   ├── FileUploadSection.svelte
│   │   │   └── NAMDConfigSection.svelte
│   │   └── common/
│   │       ├── Button.svelte
│   │       ├── Input.svelte
│   │       ├── Modal.svelte
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
```

## Performance Best Practices

### 1. Use Keyed Each Blocks
```svelte
<!-- Good: Keyed each block -->
{#each jobs as job (job.id)}
  <JobRow {job} />
{/each}

<!-- Bad: Unkeyed each block -->
{#each jobs as job}
  <JobRow {job} />
{/each}
```

### 2. Optimize Expensive Computations
```svelte
<script>
  import { onMount } from 'svelte/core';

  export let largeDataset;

  let processedData = [];

  // Process data only when needed
  $: if (largeDataset.length > 0) {
    processedData = expensiveProcessing(largeDataset);
  }

  // Or use derived store for shared expensive computations
  const processedStore = derived(jobs, $jobs => expensiveProcessing($jobs));
</script>
```

### 3. Component Lifecycle
```svelte
<script>
  import { onMount, onDestroy, beforeUpdate, afterUpdate } from 'svelte/core';

  let timer;

  onMount(() => {
    // Initialize component
    timer = setInterval(syncJobs, 30000);
  });

  onDestroy(() => {
    // Cleanup
    clearInterval(timer);
  });

  beforeUpdate(() => {
    // Before DOM updates
  });

  afterUpdate(() => {
    // After DOM updates
  });
</script>
```

## Key Architectural Differences from React

### 1. No Virtual DOM
- Direct DOM manipulation means more efficient updates
- No need for React keys or reconciliation concerns
- Component lifecycle is simpler

### 2. Compile-Time Optimizations
- Svelte compiler optimizes reactivity at build time
- No runtime framework overhead
- Smaller bundle size

### 3. Built-in State Management
- Stores are first-class citizens
- No need for external state management libraries
- Automatic subscription/unsubscription

### 4. CSS Scoping
- Component-scoped CSS by default
- No need for CSS-in-JS libraries
- Can use CSS custom properties for theming

### 5. Form Handling
- Two-way binding eliminates controlled component boilerplate
- Built-in form validation helpers
- Easier to manage form state

This guide provides the foundation for implementing NAMDRunner using proper Svelte patterns, ensuring both code quality and maintainability while achieving the desired user experience from the original React mockup.