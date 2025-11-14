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
  - Jobs (default view, shows badge with total job count)
  - Create Job (disabled when disconnected)
  - Templates
  - Settings (database management)
- **Breadcrumbs**: Secondary navigation for drilling into details
  - Example: `Jobs > Job Details`
  - Example: `Templates > Edit Template`
- **Connection Status**: Top-right dropdown for SSH management

---

## Page Specifications

### 1. Jobs Table (Main View)

#### Header Area
- **Page Title**: "Jobs"
- **Sync Status**: `Last synced: 5 minutes ago [Sync Now] [Auto-sync: ☐ every _5_ min]`
  - Gray text when disconnected: "Offline - showing cached data from [timestamp]"
- **Actions**: [Create Job] button (disabled when disconnected)

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
- Overview | Input Files | Output Files | SLURM Logs

[Tab Content Area]
- Content varies by selected tab

[Action Buttons Area]
- [Sync Results from Scratch] (only visible when job completed)
- [Delete Job] (shows confirmation with checkbox options)
```

#### Tab Contents
- **Overview**: Simulation progress, resource allocation, job information, template configuration values
- **Input Files**: List of uploaded input files with individual download buttons and bulk download option
- **Output Files**: List of output files with file sizes, individual download buttons, and bulk download option
- **SLURM Logs**: stdout and stderr output viewers

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
`Jobs > Create Job`

#### Tab Interface (3 tabs)
The create job page uses a 3-tab interface for organizing job configuration:

**Tab 1: Resources**
- **Resource Presets**: Pill-style buttons for common configurations
  - Small, Medium, Large, GPU presets
  - Each shows specs: cores, memory, wall time
  - Selected preset is highlighted
- **Manual Configuration** (collapsible details section)
  - Cores: [number input] (1-1024) *
  - Memory: [text input] (e.g., "32GB") *
  - Wall Time: [text input] (HH:MM:SS format) *
  - Partition: [dropdown] *
  - QOS: [dropdown] *
- **Validation Bar**: Real-time display showing:
  - Validation status (valid/invalid with icon)
  - Cost estimate in SU (Service Units)
  - Queue time estimate
  - Expandable issues/warnings list
- **Actions**: [Preview SLURM Script] button

**Tab 2: Configure**
- **Job Information**
  - Job Name: [text input] * (unique identifier)
- **Template Selection**
  - Template dropdown with description display
- **Dynamic Form** (generated from selected template):
  - **Input Files Section** (if template has file variables)
    - File upload fields with browse buttons
    - Shows allowed extensions
    - Help text per field
  - **Simulation Parameters Section** (if template has parameter variables)
    - Dynamic form fields based on variable types:
      - Number inputs (with min/max constraints)
      - Text inputs
      - Checkbox for boolean values
    - Help text per field
  - Field order matches template text order
- **Actions**: [Validate Configuration], [Preview NAMD Configuration]

**Tab 3: Review**
- **Validation Summary**: Shows count of validation errors if any
- **Resource Summary**: Displays selected partition, QOS, cores, memory, wall time
- **Configuration Summary**: Job name, template ID, template parameter values
- **Input Files**: List of files to upload with animated progress bars
- **Actions**: [Back to Jobs], [Create Job] (disabled if validation errors exist)

#### Form Validation
- **Backend Validation**: Debounced validation runs automatically on input changes
- **Required Fields**: Marked with asterisk (*)
- **Inline Errors**: Red border and error text below fields
- **Real-time Feedback**: Validation bar updates as configuration changes
- **Submit Prevention**: Create Job button disabled when validation errors exist

---

### 4. Templates Page

#### Breadcrumb
`Templates`

#### Page Layout
```
[Page Header]
- Title: "Simulation Templates"
- [+ Create Template] button

[Template Grid]
- Card-based grid layout (auto-fill, min 300px width)
- Each card shows:
  - Template name (header)
  - Badge: "Built-in" (blue) or "Custom" (green)
  - Description text
  - Action buttons: [Edit] [Duplicate] [Delete]
```

#### Template Cards
- **Built-in Templates**: Badge indicator, improved contrast with secondary background
- **Custom Templates**: Standard styling
- **Hover State**: Elevated shadow effect
- **Empty State**: Centered message encouraging template creation

#### Actions
- **Create**: Opens template editor in create mode
- **Edit**: Opens template editor in edit mode with template data
- **Duplicate**: Creates copy with "_copy_timestamp" ID and "(Copy)" name suffix
- **Delete**: Shows confirmation dialog (unified ConfirmDialog component)

---

### 5. Settings Page

#### Breadcrumb
`Settings`

#### Page Layout
```
[Page Header]
- Title: "Settings"

[Database Management Section]
- Database Location: /full/path/to/namdrunner.db (read-only display)
- Database Size: 2.3 MB (formatted display)
- Action buttons:
  [Backup Database] - Opens save dialog, creates copy
  [Restore Database] - Warning dialog → file dialog → replaces DB
  [Reset Database] - Warning dialog → deletes and recreates DB
```

#### Database Operations
- **Backup**: Opens OS save dialog, uses SQLite Backup API for safe online backup
- **Restore**: Shows warning with ConfirmDialog (destructive style), opens file dialog, validates backup, replaces database, reinitializes connection
- **Reset**: Shows warning with ConfirmDialog (destructive style), deletes database file, recreates schema, reloads stores
- **Post-Operation**: AlertDialog shows success/error messages, all stores reload automatically

#### Database Paths
- **Production Linux**: `~/.local/share/namdrunner/namdrunner.db`
- **Production Windows**: `%APPDATA%\namdrunner\namdrunner.db`
- **Development**: `./namdrunner_dev.db` (project root)

---

### 6. Template Editor Page

#### Breadcrumb
`Templates > Create Template` or `Templates > Edit Template`

#### Form Structure

**Template Metadata**
- Template ID: [text input] * (disabled in edit mode, lowercase/underscores only)
- Template Name: [text input] *
- Description: [textarea] (optional, 3 rows)

**NAMD Configuration Template**
- Large textarea (20 rows, monospace font) *
- Uses `{{variable_name}}` syntax for variables
- Auto-detects variables on text change (debounced 500ms)
- Help text explains variable syntax

**Template Variables** (Auto-detected section)
- Shows list of detected variables from template text
- Each variable shows:
  - Label (human-readable)
  - Variable key (monospace, gray background)
  - Type badge (uppercase, colored: Number/Text/Boolean/FileUpload)
  - [Edit] button
- Variables ordered by first appearance in template
- Empty state message if no variables detected

**Variable Editor** (Modal)
Opens when editing a variable to configure:
- Variable key (editable, triggers re-indexing in parent)
- Display label
- Variable type selection (Number/Text/Boolean/FileUpload)
- Type-specific configuration:
  - Number: min, max, default
  - Text: default value
  - Boolean: default (checkbox)
  - FileUpload: allowed extensions (array)
- Required checkbox
- Help text (optional)

**Form Actions**
- Left side: [Delete Template] (edit mode only, red button)
- Right side: [Cancel] [Test Template] [Save Template]
- Test Template: Shows preview modal with sample values substituted
- Save: Creates or updates template, navigates back to templates list

#### Auto-Variable Detection
Variables are automatically detected from template text using regex pattern:
- Pattern: `{{[a-zA-Z_][a-zA-Z0-9_]*}}`
- New variables get default Text type with smart label (capitalize, replace underscores)
- Removed variables deleted from metadata
- Existing variable metadata preserved on re-detection

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

### Common UI Components

NAMDRunner uses a composition-based modal system with a single primitive component and specialized wrappers.

**Dialog** (Primitive Component)
- The only base modal component in the system
- Located: `src/lib/components/ui/Dialog.svelte`
- Features: Backdrop overlay, escape key handler, click-outside to close, z-index management
- Props: `open`, `size` (sm/md/lg), `onClose`, `showCloseButton`
- Slots: `header`, `body` (default), `footer`
- All other modals use Dialog internally via composition

**AlertDialog** (Replaces native `alert()`)
- Wrapper around Dialog for simple notifications
- Located: `src/lib/components/ui/AlertDialog.svelte`
- Props: `open`, `title`, `message`, `variant`, `onClose`
- Variants: `success`, `error`, `warning`, `info` (with colored icons)
- Used for: Settings page notifications, operation feedback
- Single OK button to dismiss

**ConfirmDialog** (Confirmation Dialogs)
- Wrapper around Dialog for confirmation actions
- Located: `src/lib/components/ui/ConfirmDialog.svelte`
- Props: `isOpen`, `title`, `message`, `confirmText`, `cancelText`, `confirmStyle`, `onConfirm`, `onCancel`
- confirmStyle: `'destructive'` (red) or `'primary'` (blue)
- Used for: template deletion, job deletion, database operations
- Two buttons: Cancel (secondary) and Confirm (primary/destructive)

**PreviewModal**
- Wrapper around Dialog for displaying code/text previews
- Located: `src/lib/components/ui/PreviewModal.svelte`
- Used for: SLURM script preview, NAMD config preview, template testing
- Props: `isOpen`, `title`, `content`, `onClose`
- Content displayed in monospace font with `--namd-code-bg` background

---

## CSS Design System

### CSS Variables and Theming

NAMDRunner uses CSS custom properties (variables) for consistent theming and easy dark mode support. All variables use the `--namd-*` prefix and are defined in `src/lib/styles/app.css`.

**Core Color Categories:**
- `--namd-bg-*`: Background colors (primary, secondary, muted)
- `--namd-text-*`: Text colors (primary, secondary, muted)
- `--namd-primary-*`: Primary action colors and variants (light: `#2563eb`, dark: `#3b82f6`)
- `--namd-secondary-*`: Secondary action colors
- `--namd-accent-*`: Accent colors for hover states
- `--namd-success/warning/error/info-*`: Status colors with background/foreground variants
- `--namd-sidebar-*`: Sidebar-specific colors including active states
- `--namd-border*`: Border colors and shadows
- `--namd-code-bg`: Background for code/monospace content
- `--namd-input-disabled-bg`: Disabled input background

**Layout Variables:**
- `--namd-border-radius*`: Border radius tokens (sm, base, lg)
- `--namd-spacing-*`: Spacing scale (xs, sm, md, lg, xl, 2xl)
- `--namd-font-size-*`: Typography scale
- `--namd-font-weight-*`: Font weights
- `--namd-shadow-*`: Box shadow tokens
- `--namd-z-*`: Z-index layers (dropdown, modal, popover, tooltip)

**Dark Theme:**
Uses `[data-theme="dark"]` selector to override variables. Toggle via `uiStore.setTheme()`. All colors are defined for both light and dark themes to ensure proper contrast and readability.

#### Naming Convention
**Consistent Naming**: Use `namd-*` prefix for all custom CSS classes.

```css
/* ✅ Consistent naming in app.css */
.namd-button { /* base styles */ }
.namd-button--outline { /* variant */ }
.namd-status-badge { /* component */ }
.namd-status-badge--running { /* state */ }
```

#### Centralized Styling Approach
**CSS Variables** are defined in `app.css` and referenced in component styles.

**Form Inputs**: All text inputs, textareas, and select elements use the `.namd-input` class. No component-specific input styles exist.

```svelte
<!-- Component uses CSS variables -->
<style>
  .card {
    background: var(--namd-bg-primary);
    border: 1px solid var(--namd-border);
    border-radius: var(--namd-border-radius);
    padding: var(--namd-spacing-md);
  }
</style>
```

Component-specific styles are allowed but should use CSS variables for colors, spacing, and other themeable values.

#### Button System

NAMDRunner uses a centralized button system defined in `app.css`. All components use these classes instead of custom button styles.

**Base Button Class:**
```css
.namd-button {
  /* Base styles: flex layout, padding, border-radius, transitions */
}
```

**Button Variants:**
- `.namd-button--primary` - Primary actions (blue background)
- `.namd-button--secondary` - Secondary actions (gray background with border)
- `.namd-button--destructive` - Dangerous actions (red background)
- `.namd-button--ghost` - Transparent background, text-colored
- `.namd-button--outline` - Transparent background with border

**Button Sizes:**
- `.namd-button--sm` - Small buttons (reduced padding and font size)
- `.namd-button--lg` - Large buttons (increased padding and font size)

**Usage Pattern:**
```svelte
<button class="namd-button namd-button--primary">Create</button>
<button class="namd-button namd-button--secondary">Cancel</button>
<button class="namd-button namd-button--destructive">Delete</button>
<button class="namd-button namd-button--sm namd-button--outline">Edit</button>
```

#### Component Class Examples
```css
/* Status badges */
.namd-badge {
  padding: var(--namd-spacing-xs) var(--namd-spacing-sm);
  border-radius: 9999px;
  font-size: var(--namd-font-size-xs);
}

.namd-badge--success {
  background-color: var(--namd-success-bg);
  color: var(--namd-success-fg);
}

/* Form fields */
.namd-field-group {
  margin-bottom: 1rem;
}

.namd-label {
  display: block;
  margin-bottom: 0.25rem;
  font-weight: var(--namd-font-weight-medium);
}

.namd-input {
  width: 100%;
  padding: 0.5rem;
  border: 1px solid var(--namd-border);
  border-radius: var(--namd-border-radius-sm);
  background: var(--namd-bg-primary);
  color: var(--namd-text-primary);
}

.namd-input.error {
  border-color: var(--namd-error-border);
  background-color: var(--namd-error-bg);
}

.namd-input:disabled {
  background-color: var(--namd-input-disabled-bg);
}

.namd-error-text {
  color: var(--namd-error-fg);
  font-size: var(--namd-font-size-xs);
  margin-top: 0.25rem;
}

/* File Lists */
.namd-file-list {
  display: flex;
  flex-direction: column;
  gap: var(--namd-spacing-sm);
}

.namd-file-item {
  position: relative;
  display: flex;
  align-items: center;
  padding: var(--namd-spacing-sm) var(--namd-spacing-md);
  background-color: var(--namd-bg-primary);
  border: 1px solid var(--namd-border);
  border-radius: var(--namd-border-radius-sm);
}

.namd-file-content {
  display: flex;
  align-items: center;
  gap: var(--namd-spacing-sm);
  flex: 1;
}

.namd-file-name {
  flex: 1;
  font-family: var(--namd-font-mono);
}

.namd-file-metadata {
  color: var(--namd-text-secondary);
  font-size: var(--namd-font-size-sm);
}

.namd-file-progress-bg {
  /* Animated background for upload progress */
  position: absolute;
  top: 0;
  left: 0;
  height: 100%;
  background-color: var(--namd-primary-bg);
  transition: width 0.3s ease;
}

.namd-file-progress-text {
  margin-left: auto;
  font-weight: var(--namd-font-weight-semibold);
  color: var(--namd-primary);
}

.namd-file-list-empty {
  color: var(--namd-text-secondary);
  font-style: italic;
  padding: var(--namd-spacing-lg);
  text-align: center;
  background-color: var(--namd-bg-muted);
  border-radius: var(--namd-border-radius-sm);
  border: 1px dashed var(--namd-border);
}

.namd-file-list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--namd-spacing-md);
}

.namd-file-list-error {
  padding: var(--namd-spacing-sm) var(--namd-spacing-md);
  background-color: var(--namd-error-bg);
  color: var(--namd-error);
  border-radius: var(--namd-border-radius-sm);
  border: 1px solid var(--namd-error);
  margin-bottom: var(--namd-spacing-md);
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

NAMDRunner uses Svelte stores for global state management instead of prop drilling.

#### Core Stores

**`stores/ui.ts`** - View navigation and UI state
```typescript
interface UIState {
  currentView: 'jobs' | 'create' | 'templates' | 'template-edit' | 'settings';
  selectedJobId: string | null;
  selectedTemplateId: string | null;
  templateEditorMode: 'create' | 'edit';
  consoleOpen: boolean;
  theme: 'light' | 'dark';
}

// Key methods
uiStore.setView(view)              // Navigate between pages
uiStore.selectJob(job_id)          // Select job for detail view
uiStore.editTemplate(id, mode)     // Open template editor
uiStore.toggleConsole()            // Show/hide logs panel
uiStore.setTheme(theme)            // Toggle dark mode

// Derived stores
export const currentView = derived(uiStore, $ui => $ui.currentView);
export const breadcrumbs = derived(uiStore, ...); // Auto-generated breadcrumbs
```

**`stores/session.ts`** - SSH connection state
```typescript
interface ConnectionState {
  status: 'disconnected' | 'connecting' | 'connected' | 'expired';
  host?: string;
  username?: string;
  connectedSince?: Date;
}

// Connection transitions to 'expired' on SSH errors
export const isConnected = derived(session, $s => $s.status === 'connected');
```

**`stores/jobs.ts`** - Job management with offline support
```typescript
// Cached jobs loaded from SQLite
jobsStore.loadJobs()          // Load from local DB
jobsStore.syncJobs()          // Sync with cluster (requires connection)
jobsStore.createJob(params)   // Create new job
jobsStore.deleteJob(id)       // Delete job and optionally files

// Derived stores
export const jobCounts = derived(jobs, ...);  // Count by status
export const selectedJob = derived(...);       // Current job details

// Features:
// - Auto-detects connection failures (transitions to Expired state)
// - Offline mode: shows cached jobs, disables actions requiring connection
```

**`stores/templateStore.ts`** - Template management
```typescript
templateStore.loadTemplates()                    // List all templates
templateStore.loadTemplate(id)                   // Get full template
templateStore.createTemplate(template)           // Create new
templateStore.updateTemplate(id, template)       // Update existing
templateStore.deleteTemplate(id)                 // Delete template
templateStore.validateTemplateValues(id, values) // Validate user input

// Stores
export const templates = writable<TemplateSummary[]>([]);  // List view
export const templatesLoading = writable(false);
export const templatesError = writable<string | null>(null);
```

**`stores/clusterConfig.ts`** - Cluster configuration
```typescript
// Pre-loaded cluster metadata
export const partitions = writable<Partition[]>([]);       // Available partitions
export const allQosOptions = writable<QosOption[]>([]);    // QOS options
export const jobPresets = writable<JobPreset[]>([]);       // Resource presets

// Helper functions
validateResourceRequest(cores, memory, walltime, partition, qos)
calculateJobCost(cores, walltimeHours, hasGpu, gpuCount)
estimateQueueTime(cores, partition)
walltimeToHours(walltime)  // Parse HH:MM:SS to hours
```

**`stores/settings.ts`** - Settings and database management
```typescript
// Database information
settingsStore.loadDatabaseInfo()  // Get path and size
settingsStore.backupDatabase()    // OS save dialog, create backup
settingsStore.restoreDatabase()   // OS file dialog, replace database
settingsStore.resetDatabase()     // Delete and recreate database

// Stores
export const databaseInfo = writable<DatabaseInfo | null>(null);
export const isLoading = writable(false);

// Features:
// - Auto-reloads database info after restore/reset
// - Integrates with logger for user feedback
// - Post-operation store reloading (jobs, templates)
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

### Component Architecture

```
components/
├── layout/
│   ├── AppSidebar.svelte          # Main navigation with job count badges
│   ├── ConnectionDropdown.svelte   # SSH connection status/controls
│   └── LogsPanel.svelte           # Collapsible SSH logs footer
├── pages/
│   ├── JobsPage.svelte            # Jobs table with sync controls
│   ├── CreateJobPage.svelte       # Create job 3-tab interface
│   ├── JobDetailPage.svelte       # Job detail tabs
│   ├── TemplatesPage.svelte       # Template grid with actions
│   ├── TemplateEditorPage.svelte  # Template create/edit form
│   └── SettingsPage.svelte        # Database management
├── create-job/
│   ├── CreateJobTabs.svelte       # 3-tab container with validation
│   ├── ResourcesTab.svelte        # Presets + manual config + validation
│   ├── ConfigureTab.svelte        # Job name + template + dynamic form
│   ├── ReviewTab.svelte           # Summary + file upload progress
│   └── DynamicJobForm.svelte      # Auto-generated form from template
├── templates/
│   ├── TemplateEditor.svelte      # Template CRUD form
│   └── VariableEditor.svelte      # Variable metadata editor modal
├── job-detail/
│   ├── JobTabs.svelte             # Tab container for job details (4 tabs)
│   └── tabs/
│       ├── OverviewTab.svelte     # Progress, resources, template values
│       ├── InputFilesTab.svelte   # Input file downloads with bulk download
│       ├── OutputFilesTab.svelte  # Output file downloads with bulk download
│       └── SlurmLogsTab.svelte    # stdout/stderr viewers
├── jobs/
│   ├── JobsTable.svelte           # Table with sortable columns
│   └── SyncControls.svelte        # Sync status/controls
└── ui/
    ├── Dialog.svelte              # Base modal primitive (composition root)
    ├── AlertDialog.svelte         # Notification dialog (replaces alert())
    ├── ConfirmDialog.svelte       # Confirmation dialogs with Cancel/Confirm
    ├── PreviewModal.svelte        # Code/text preview display
    └── FormField.svelte           # Reusable form field wrapper
```

### Component Guidelines
- Start with larger, page-level components
- Extract reusable pieces as patterns emerge
- Keep component props simple and typed
- Use Svelte stores for shared state (connection status, job list)
- Form components should handle their own validation display
- Use CSS variables for all themeable values (colors, spacing, etc.)

---

## Key Features and Patterns

### Template System

**Core Concept:** Job configuration is driven by templates instead of hardcoded forms.

**Variable Detection:**
- Variables parsed from template text using `{{variable_name}}` syntax
- Auto-detection runs on template text changes (debounced 500ms)
- Variables ordered by first occurrence in template text
- Metadata preserved when template text changes

**Variable Types:**
- `Number`: min, max, default values
- `Text`: default string value
- `Boolean`: default true/false
- `FileUpload`: allowed file extensions array

**Dynamic Form Generation:**
- `DynamicJobForm` component generates form fields from template variables
- Fields organized into sections: Input Files, Simulation Parameters
- Field order matches template text order (not alphabetical)
- Form values initialized from variable defaults
- File paths stored during configuration, uploaded during job creation

### Connection State Management

**Connection States:**
- `disconnected`: No active connection
- `connecting`: Connection attempt in progress
- `connected`: Active SSH session
- `expired`: Connection lost due to error

**Auto-Detection:**
- Jobs store detects SSH failures during sync operations
- Automatically transitions session to `expired` state
- UI shows "Connection Expired" status
- Prompts user to reconnect

**Offline Mode:**
- Jobs loaded from SQLite cache always available
- Sync controls show last sync timestamp
- Action buttons disabled when disconnected
- Create Job page shows connection warning

### Real-Time Validation

**Debounced Backend Validation:**
- Runs automatically 500ms after input changes
- Validates all job configuration fields together
- Returns field-specific error messages
- Errors displayed inline with red borders and text

**Validation Display:**
- Resources tab: Validation bar with status, cost estimate, queue time
- Configure tab: Inline errors on individual fields
- Review tab: Summary banner showing error count
- Create Job button disabled when errors exist

### File Management Pattern

NAMDRunner uses a unified `.namd-file-list` CSS pattern for all file operations (upload and download contexts).

**File Upload (Create Job Flow):**
1. User selects files via Tauri dialog in Configure tab
2. File paths stored in template values
3. Review tab extracts file variables and shows list with `.namd-file-list`
4. During job creation, files uploaded via SFTP
5. Progress events update UI with animated `.namd-file-progress-bg`
6. Percentage displayed in `.namd-file-progress-text`

**File Download (Job Details Tabs):**
- **InputFilesTab**: Lists uploaded input files tracked in `job.input_files`
  - Individual download buttons per file
  - Bulk "Download All" button for ZIP archive
- **OutputFilesTab**: Lists output files from `job.output_files`
  - Shows file sizes with `.namd-file-metadata`
  - Individual and bulk download options
  - Empty state messages based on job status

**Unified Pattern Classes:**
- `.namd-file-list` - Container for file items
- `.namd-file-item` - Individual file row
- `.namd-file-content` - File icon, name, metadata layout
- `.namd-file-icon` - File type icon
- `.namd-file-name` - Filename in monospace
- `.namd-file-metadata` - File size or status text
- `.namd-file-action` - Action button container
- `.namd-file-progress-bg` - Animated upload progress background
- `.namd-file-progress-text` - Progress percentage text
- `.namd-file-error` - Error message below file item
- `.namd-file-list-empty` - Empty state message
- `.namd-file-list-header` - Header with title and bulk actions
- `.namd-file-list-error` - Bulk operation error message

---

## Design System Architecture

### Modal System

NAMDRunner uses a composition-based modal architecture with a single primitive:

**Primitive:**
- `Dialog.svelte` - The only base modal component
- Features: Backdrop, escape key, click-outside, z-index, size variants
- Slots: header, body, footer

**Wrappers:**
- `AlertDialog.svelte` - Simple notifications (success/error/warning/info)
- `ConfirmDialog.svelte` - Confirmation actions with Cancel/Confirm buttons
- `PreviewModal.svelte` - Code/text preview with monospace display

All modals use Dialog internally via composition. No duplicate modal code exists.

### Button System

All buttons use centralized `.namd-button` classes from `app.css`:
- Variants: `--primary`, `--secondary`, `--destructive`, `--ghost`, `--outline`
- Sizes: `--sm`, `--lg`
- No component-specific button styles exist

### Theme System

All colors defined as CSS variables in `app.css`:
- Light and dark themes fully supported
- Primary color: `#2563eb` (light), `#3b82f6` (dark)
- Components use variables exclusively (no hardcoded colors)
- Special variables: `--namd-code-bg`, `--namd-input-disabled-bg`, `--namd-error-*`

---

This design guide provides the essential patterns and specifications needed to implement NAMDRunner's UI consistently with the project's goals of clarity, reliability, and maintainability.

> **For technical implementation details**, see [`docs/ARCHITECTURE.md#technical-implementation-considerations`](ARCHITECTURE.md#technical-implementation-considerations)
> **For future enhancements and roadmap**, see [`tasks/roadmap.md`](../tasks/roadmap.md)