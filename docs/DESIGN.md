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
│           │ [SSH Console - Collapsed by default]            │
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
│ User: jsmith           │
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

### SSH Console (Footer)

**Collapsed**: Single line showing `[↑ SSH Console]`

**Expanded** (overlays bottom 1/3 of screen):
```
┌─────────────────────────────────────────────┐
│ SSH Console     [Copy All] [Clear] [↓ Hide] │
│─────────────────────────────────────────────│
│ $ module load slurm/alpine                  │
│ $ squeue -u jsmith --format=...             │
│ 12345678|simulation_001|R|01:30:45|...      │
│ $ sbatch job.sbatch                         │
│ Submitted batch job 12345679                │
│ _                                           │
└─────────────────────────────────────────────┘
```

---

## Future Enhancement Planning

### Multi-Stage Job Groups (Post-MVP)

The current single-job table design will evolve to support job groups:

**Table Enhancement**:
```
▼ Multi-Stage Simulation Group        GROUP      3/5 Complete    ...
  ├─ Stage 1: Minimization           COMPLETED   00:45:00        ...
  ├─ Stage 2: Heating                COMPLETED   01:30:00        ...
  ├─ Stage 3: Equilibration           RUNNING     02:15:30        ...
  ├─ Stage 4: Production 1            PENDING     --              ...
  └─ Stage 5: Production 2            PENDING     --              ...
```

- Parent rows show aggregate status
- Expandable to show individual stage jobs
- Visual distinction between groups and single jobs (indent, icon, or background)

---

## Visual Design Guidelines

### Typography
- Use system fonts for native feel
- Clear hierarchy: Headers > Subheaders > Body > Caption
- Monospace font for: Job IDs, SSH console, log outputs

### Colors
- **Primary Actions**: Blue
- **Destructive Actions**: Red
- **Success States**: Green
- **Warning/Pending**: Amber/Yellow
- **Neutral/Disabled**: Gray shades
- **Background**: Light gray or white
- **Borders/Dividers**: Light gray

### CSS Design System

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

### Spacing & Layout
- Consistent padding/margins throughout
- Clear visual grouping of related elements
- Adequate whitespace between sections
- No cramped interfaces, but efficient use of space

### Loading & Feedback
- **Loading States**: Simple spinner or progress bar
- **Success Toasts**: Green, auto-dismiss after 3 seconds
- **Error Toasts**: Red, require manual dismissal
- **Long Operations**: Show progress with cancel option

### Form Validation
- **Required Field Indicators**: Asterisk (*) next to label
- **Inline Validation**: Show errors below fields as user types or on blur
- **Error Styling**: Red text, red border on invalid fields
- **Success Feedback**: Optional green checkmark for valid fields
- **Validation Types**:
  - Required field validation
  - Type validation (numbers only, etc.)
  - Format validation (time formats, etc.)
  - Range validation (min/max values)

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

## Implementation Priorities

### MVP Focus
1. Core job table with status tracking
2. Basic job creation flow with validation
3. Connection management
4. Job detail viewing
5. SLURM log viewing

### Post-MVP Enhancements
1. Multi-stage job groups
2. Bulk operations
3. Advanced filtering/search
4. Job templates
5. Settings/preferences

---

## UX Requirements

* Explicit **Connect/Disconnect/Reconnect** controls and visible session state.
* Clear job status with last-polled timestamp.
* Non-blocking status refresh; errors as dismissible banners with retry.

### Job Restart UI Flow (Phase 6+)
* **"Restart Job" button** appears on completed/failed jobs with detected checkpoint files.
* **Restart wizard** allows researcher to:
  - Review original job parameters
  - Modify resource allocation (cores, memory, walltime)
  - See checkpoint file status and validation
  - Preview restart job configuration
* **Restart job tracking** shows connection to parent job and restart lineage.
* **Progress indication** displays completed vs remaining steps across restart chain.

---

## Technical Considerations

### State Management
- **Connection State**: Global Svelte store
- **Job List**: Cached in store, synced with backend
- **Form State**: Local to components
- **SSH Console Buffer**: Global store with size limit

### Performance
- **Lazy loading** for job details and logs
- **Debounced** form validation
- **Throttled** SSH console updates

### Accessibility
- Keyboard navigation support
- ARIA labels for screen readers
- Focus management for modals
- Color-blind friendly status indicators (use icons + color)

---

This specification provides the foundation for creating a clean, functional UI that scientists will find reliable and easy to use. The design emphasizes clarity and utility while maintaining flexibility for future enhancements.