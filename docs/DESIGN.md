# UI/UX Design Guide

## Project Overview
NAMDRunner is a desktop application for managing NAMD molecular dynamics simulations on SLURM HPC clusters. The UI should be clean, functional, and straightforward - prioritizing clarity and reliability over visual flourish. Think traditional desktop application patterns that scientists will find familiar and trustworthy.

## Design Philosophy
- **Clean and Simple**: No fancy animations or complex interactions - just clear, readable interfaces
- **Functional First**: Every element should have a clear purpose
- **Desktop Native Feel**: Classic desktop app patterns (sidebar navigation, tables, standard forms)
- **Information Density**: Show relevant data efficiently without feeling cramped
- **Consistent Feedback**: Clear status indicators, loading states, and error messages

---

## Application Structure

### Main Layout
```
┌─────────────────────────────────────────────────────────────┐
│ [Sidebar] │  [Breadcrumb Navigation]          [Connection ▼]│
│           │─────────────────────────────────────────────────│
│ Jobs      │                                                 │
│ Create    │           Main Content Area                     │
│ Settings  │                                                 │
│           │                                                 │
│           │                                                 │
│           │                                                 │
│           │                                                 │
│           │─────────────────────────────────────────────────│
│           │ [Logs Panel - Collapsed by default]             │
└─────────────────────────────────────────────────────────────┘
```

### Navigation Structure
- **Left Sidebar**: Primary navigation between main sections
  - Jobs (default view)
  - Create Job
  - Settings (future)
- **Breadcrumbs**: Secondary navigation for drilling into details
  - Example: `Jobs > job_001_simulation`
- **Connection Status**: Top-right dropdown for SSH management

---

## Page Specifications

### 1. Jobs Table (Main View)

#### Header Area
- **Page Title**: "Jobs"
- **Sync Status**: `Last synced: 5 minutes ago [Sync Now] [Auto-sync: ☐ every _5_ min]`
  - Gray text when disconnected: "Offline - showing cached data from [timestamp]"
- **Actions**: [Create New Job] button (disabled when disconnected)

#### Table Structure
**Columns** (sortable by clicking headers):
- Job Name
- Status (colored badge/pill)
- Runtime (format: "02:15:30")
- Wall Time (format: "4h total / 45m left")
- Created Date
- Submitted Date
- Job ID

**Default Sort**: Creation time, newest first

**Row Interaction**: Click anywhere on row to navigate to Job Detail page

**Status Badges** (pill-shaped with text + color):
- `CREATED` - Gray
- `PENDING` - Yellow/Amber
- `RUNNING` - Blue
- `COMPLETED` - Green
- `FAILED` - Red
- `CANCELLED` - Dark Gray

---

### 2. Job Detail Page

#### Breadcrumb
`Jobs > [job_name]`

#### Page Layout
```
[Job Summary Card]
- Job Name, ID, Status Badge
- Created/Submitted/Completed timestamps
- SLURM Job ID

[Tab Navigation]
- Overview | SLURM Logs | Input Files | Output Files | Configuration

[Tab Content Area]
- Content varies by selected tab

[Action Buttons Area]
- [Sync Results from Scratch] (only visible when job completed)
- [Delete Job] (shows confirmation with checkbox options)
```

#### Tab Contents
- **Overview**: Summary statistics, resource usage
- **SLURM Logs**: stdout and stderr output viewers
- **Input Files**: List with file names, sizes, types
- **Output Files**: List with download buttons for each file
- **Configuration**: NAMD and SLURM parameters used

#### Delete Confirmation
Modal dialog:
```
Delete Job: [job_name]?
☑ Also delete files (project and scratch folders)
[Cancel] [Delete]
```

---

### 3. Create Job Page

#### Breadcrumb
`Jobs > Create New Job`

#### Form Structure (Single page with sections)

**Section 1: SLURM Resource Allocation**
- Cores: [number input] *
- Memory: [text input] GB *
- Wall Time: [text input] format: HH:MM:SS *
- Partition: [dropdown] (default: "amilan")
- QOS: [dropdown] (default: "normal")

**Section 2: Input Files**
```
┌─────────────────────────────────┐
│  Drag & drop files here or      │
│  [Click to Browse]              │
│                                 │
│  Accepted: .pdb, .psf, .prm     │
└─────────────────────────────────┘

Uploaded Files:
- structure.pdb [x]
- structure.psf [x]
- parameters.prm [x]
```

**Section 3: NAMD Configuration**
- Job Name: [text input] *
- Simulation Steps: [number input] *
- Temperature (K): [number input] *
- Timestep (fs): [number input] *
- Output Name: [text input] *
- DCD Frequency: [number input] (optional)
- Restart Frequency: [number input] (optional)

**Actions**
[Cancel] [Create Job]

#### Form Validation
- **Required Fields**: Marked with asterisk (*), show inline error if empty on submit
- **Type Validation**: Number fields show error for non-numeric input
- **Format Validation**: Wall time must match HH:MM:SS format
- **Inline Errors**: Appear below fields in red text
  - Example: "This field is required"
  - Example: "Please enter a valid number"
  - Example: "Wall time must be in HH:MM:SS format"

---

## Component Specifications

### Connection Status Dropdown

**Collapsed State** (normal view):
```
[● Connected] ▼
```
- Green dot + "Connected"
- Yellow dot + "Connecting..."
- Red dot + "Disconnected"
- Gray dot + "Connection Expired"

**Expanded State** (dropdown overlay):
```
┌────────────────────────┐
│ ● Connected            │
│ Host: cluster.edu      │
│ User: <username>       │
│ Since: 10:30 AM        │
│ [Disconnect]           │
└────────────────────────┘
```

**Disconnected Expanded State**:
```
┌────────────────────────┐
│ ○ Disconnected         │
│ Host: [___________]    │
│ Username: [________]   │
│ Password: [________]   │
│ [Connect]              │
└────────────────────────┘
```

### Logs Panel (Footer)

**Collapsed**: Single line showing `[↑ Logs]`

**Expanded** (overlays bottom 1/3 of screen):
```
┌─────────────────────────────────────────────┐
│ Logs            [Copy All] [Clear] [↓ Hide] │
│─────────────────────────────────────────────│
│ $ module load slurm/alpine                  │
│ $ squeue -u <username> --format=...         │
│ 12345678|simulation_001|R|01:30:45|...      │
│ $ sbatch job.sbatch                         │
│ Submitted batch job 12345679                │
│ _                                           │
└─────────────────────────────────────────────┘
```

---

## CSS Design System

#### Naming Convention
**Consistent Naming**: Use `namd-*` prefix for all custom CSS classes.

```css
/* ✅ Consistent naming in app.css */
.namd-button { /* base styles */ }
.namd-button--outline { /* variant */ }
.namd-status-badge { /* component */ }
.namd-status-badge--running { /* state */ }
.namd-file-type-badge { /* component */ }
.namd-file-type-structure { /* variant */ }
```

#### Centralized Styling
**Define reusable styles in `app.css`**, not component files.

```svelte
<!-- ✅ Use centralized classes -->
<span class="namd-status-badge namd-status-badge--{statusClass}">
  {status}
</span>

<!-- ❌ Component-specific styles -->
<style>
  .status-badge { /* duplicate styles */ }
</style>
```

#### Component Class Examples
```css
/* Status badges */
.namd-status-badge {
  padding: 0.25rem 0.5rem;
  border-radius: 9999px;
  font-size: 0.875rem;
  font-weight: 500;
}

.namd-status-badge--running {
  background-color: #dbeafe;
  color: #1d4ed8;
}

.namd-status-badge--completed {
  background-color: #d1fae5;
  color: #065f46;
}

/* Form fields */
.namd-field-group {
  margin-bottom: 1rem;
}

.namd-label {
  display: block;
  margin-bottom: 0.25rem;
  font-weight: 500;
}

.namd-input {
  width: 100%;
  padding: 0.5rem;
  border: 1px solid #d1d5db;
  border-radius: 0.375rem;
}

.namd-input.error {
  border-color: #ef4444;
}

.namd-error-text {
  color: #ef4444;
  font-size: 0.875rem;
  margin-top: 0.25rem;
}

/* Tabs */
.namd-tabs-nav {
  display: flex;
  border-bottom: 1px solid #e5e7eb;
}

.namd-tabs-nav--grid {
  display: grid;
}

.namd-tabs-nav--grid-5 {
  grid-template-columns: repeat(5, 1fr);
}

.namd-tab-button {
  padding: 0.75rem 1rem;
  background: transparent;
  border: none;
  border-bottom: 2px solid transparent;
  cursor: pointer;
}

.namd-tab-button.active {
  border-bottom-color: #3b82f6;
  color: #3b82f6;
}

.namd-tab-content {
  padding: 1rem;
}

.namd-tab-panel {
  /* Panel-specific styles */
}
```

### Form Validation

#### Validation Architecture
**Frontend validation** provides immediate user feedback for basic UX concerns only. **Backend validation** in Rust is authoritative for all business rules, security, and cluster constraints.

#### Frontend Validation (UX Only)
- **Required Field Indicators**: Asterisk (*) next to label
- **Immediate Feedback**: Basic format and required field checks
- **Error Display**: Show backend validation errors from Tauri commands
- **Error Styling**: Red text, red border on invalid fields

#### Implementation Pattern
```svelte
<!-- Frontend: UX feedback only -->
<script lang="ts">
  export let value: string = '';
  export let required: boolean = false;

  let error: string = '';
  let backendError: string = ''; // From Tauri command response

  function validateUX() {
    // Only basic UX validation - NOT business rules
    if (required && !value.trim()) {
      error = 'This field is required';
      return false;
    }
    error = '';
    return true;
  }

  // Display backend validation errors
  $: displayError = backendError || error;
</script>

<div class="namd-field-group">
  <label class="namd-label">
    Field Label {#if required}*{/if}
  </label>
  <input
    class="namd-input"
    class:error={displayError}
    bind:value
    on:blur={validateUX}
  />
  {#if displayError}
    <span class="namd-error-text">{displayError}</span>
  {/if}
</div>
```

#### Validation Responsibilities
- **Frontend**: Required fields, basic formats, immediate UX feedback
- **Backend (Rust)**: Business rules, security validation, cluster limits, data integrity
- **Error Flow**: Backend validation errors displayed in frontend UI

> **For complete backend validation patterns**, see [`docs/CONTRIBUTING.md#security-requirements`](CONTRIBUTING.md#security-requirements)

---

## Svelte Implementation Patterns

### Store-Based State Management

NAMDRunner uses Svelte stores for global state management instead of prop drilling:

```typescript
// stores/session.ts
import { writable, derived } from 'svelte/store';

interface ConnectionState {
  status: 'disconnected' | 'connecting' | 'connected' | 'expired';
  host?: string;
  username?: string;
  connectedSince?: Date;
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
        await invoke('ssh_connect', { host, username, password });
        set({ status: 'connected', host, username, connectedSince: new Date() });
      } catch (error) {
        set({ status: 'disconnected' });
      }
    },
    disconnect: () => {
      invoke('ssh_disconnect');
      set({ status: 'disconnected' });
    }
  };
}

export const session = createSessionStore();
export const isConnected = derived(session, $session => $session.status === 'connected');
```

```typescript
// stores/jobs.ts
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
    }
  };
}

export const jobs = createJobsStore();
export const selectedJobId = writable<string | null>(null);
export const selectedJob = derived(
  [jobs, selectedJobId],
  ([$jobs, $selectedJobId]) =>
    $selectedJobId ? $jobs.find(j => j.id === $selectedJobId) : null
);
```

### Component Reactive Patterns

**Reactive Statements for Data Processing:**
```svelte
<script>
  import { connectionState } from '$lib/stores/session';
  import { jobs } from '$lib/stores/jobs';

  // Reactive data filtering
  $: filteredJobs = $connectionState === 'connected'
    ? $jobs.filter(job => job.status !== 'CANCELLED')
    : [];

  // Reactive validation
  $: canCreateJob = $connectionState === 'connected';
</script>

<button disabled={!canCreateJob}>Create Job</button>
```

**Two-Way Binding for Forms:**
```svelte
<script>
  export let config;

  // Reactive validation
  $: errors = validateResourceConfig(config);
  $: isValid = !Object.keys(errors).length;
</script>

<input bind:value={config.cores} type="number" />
{#if errors.cores}
  <p class="namd-error-text">{errors.cores}</p>
{/if}
```

**Event Handling Between Components:**
```svelte
<!-- JobsTable.svelte -->
<script>
  import { createEventDispatcher } from 'svelte';

  export let jobs;
  const dispatch = createEventDispatcher();

  function selectJob(job_id) {
    dispatch('jobSelect', { job_id });
  }
</script>

{#each jobs as job (job.job_id)}
  <tr on:click={() => selectJob(job.job_id)}>
    <td>{job.job_name}</td>
  </tr>
{/each}
```

```svelte
<!-- JobsPage.svelte -->
<script>
  import { selectedJobId } from '$lib/stores/jobs';

  function handleJobSelect(event) {
    $selectedJobId = event.detail.job_id;
  }
</script>

<JobsTable {jobs} on:jobSelect={handleJobSelect} />
```

### Component Lifecycle

```svelte
<script>
  import { onMount, onDestroy } from 'svelte';

  let timer;

  onMount(() => {
    // Initialize component
    timer = setInterval(syncJobs, 30000);
  });

  onDestroy(() => {
    // Cleanup
    clearInterval(timer);
  });
</script>
```

---

## Svelte Component Architecture

### Recommended Component Structure
```
components/
├── layout/
│   ├── Sidebar.svelte
│   ├── ConnectionStatus.svelte
│   ├── Breadcrumbs.svelte
│   └── SSHConsole.svelte
├── jobs/
│   ├── JobTable.svelte
│   ├── JobTableRow.svelte
│   ├── JobStatusBadge.svelte
│   └── SyncStatus.svelte
├── job-detail/
│   ├── JobSummaryCard.svelte
│   ├── JobTabs.svelte
│   └── FileList.svelte
├── job-create/
│   ├── ResourceForm.svelte
│   ├── FileUploadArea.svelte
│   └── NAMDConfigForm.svelte
└── common/
    ├── Button.svelte
    ├── Toast.svelte
    ├── Modal.svelte
    ├── FormInput.svelte
    ├── FormError.svelte
    └── NumberInput.svelte
```

### Component Guidelines
- Start with larger, page-level components
- Extract reusable pieces as patterns emerge
- Keep component props simple and typed
- Use Svelte stores for shared state (connection status, job list)
- Form components should handle their own validation display

---

This design guide provides the essential patterns and specifications needed to implement NAMDRunner's UI consistently with the project's goals of clarity, reliability, and maintainability.

> **For technical implementation details**, see [`docs/ARCHITECTURE.md#technical-implementation-considerations`](ARCHITECTURE.md#technical-implementation-considerations)
> **For future enhancements and roadmap**, see [`tasks/roadmap.md`](../tasks/roadmap.md)