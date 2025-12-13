# UI Development Guide

**Component system, design patterns, and Svelte implementation.**

> See project README for project overview.
>
> **Related Docs:**
>
> - [ARCHITECTURE.md](ARCHITECTURE.md) - System architecture
> - [CONTRIBUTING.md](CONTRIBUTING.md) - Development standards

## Design System

### CSS Variables (`src/lib/styles/app.css`)

All styles use `--namd-*` prefixed variables:

**Colors:**

- `--namd-bg-*`: Backgrounds (primary, secondary, muted)
- `--namd-text-*`: Text (primary, secondary, muted)
- `--namd-primary-*`: Primary actions
- `--namd-success/warning/error/info-*`: Status colors (bg, fg variants)
- `--namd-border`: Border colors

**Layout:**

- `--namd-spacing-*`: xs, sm, md, lg, xl, 2xl (0.25rem to 3rem)
- `--namd-font-size-*`: xs, base, lg, xl, 2xl
- `--namd-font-weight-*`: normal, medium, semibold, bold
- `--namd-border-radius-*`: sm, base, lg
- `--namd-shadow-*`: md, lg
- `--namd-z-*`: dropdown, modal, popover, tooltip

**Theme Support:**
`[data-theme="dark"]` selector overrides variables. Toggle via `uiStore.setTheme('dark')`.

### Component Classes

**Buttons** (`.namd-button`):

- Variants: `--primary`, `--secondary`, `--destructive`, `--ghost`, `--outline`
- Sizes: `--sm`, `--lg`

**Forms** (`.namd-field-group`, `.namd-label`, `.namd-input`):

- `.namd-input` - All inputs, textareas, selects
- `.namd-input.error` - Error state
- `.namd-error-text` - Error messages

**Cards** (`.namd-card`):

- `.namd-card-header`, `.namd-card-content`

**File Lists** (`.namd-file-list`):

- `.namd-file-item`, `.namd-file-content`, `.namd-file-name`
- `.namd-file-progress-bg`, `.namd-file-progress-text` (upload progress)
- `.namd-file-list-empty` (empty states)

**Badges** (`.namd-badge`):

- Variants: `--success`, `--error`, `--warning`, `--info`

**Utility:**

- `.namd-text-truncate` - Overflow ellipsis for long text

**Never:**

- ❌ Create custom button styles
- ❌ Create custom form input styles
- ❌ Use inline styles or arbitrary colors
- ✅ Always use design system variables and classes

## Component Primitives

### Dialog (Base Modal)

**Location:** `src/lib/components/ui/Dialog.svelte`

The ONLY base modal component. All modals use this.

```svelte
<Dialog open={isOpen} size="md" onClose={handleClose}>
  <svelte:fragment slot="header">
    <h2 class="dialog-title">Title Here</h2>
  </svelte:fragment>

  <svelte:fragment slot="body">
    <!-- Content -->
  </svelte:fragment>

  <svelte:fragment slot="footer">
    <button class="namd-button namd-button--secondary" on:click={handleClose}>Cancel</button>
    <button class="namd-button namd-button--primary" on:click={handleSave}>Save</button>
  </svelte:fragment>
</Dialog>
```

**Features:** Backdrop, ESC key, click-outside-to-close, focus management

**Sizes:** `sm` (400px), `md` (600px), `lg` (800px)

### EditDialog (Form Wrapper)

**Location:** `src/lib/components/ui/EditDialog.svelte`

Standard wrapper for add/edit forms. Use this instead of base Dialog for consistency.

```svelte
<EditDialog {isOpen} title="Edit Entity" {onSave} {onClose}>
  <svelte:fragment slot="form">
    <div class="namd-field-group">
      <label class="namd-label">Name *</label>
      <input class="namd-input" bind:value={entity.name} />
    </div>
  </svelte:fragment>
</EditDialog>
```

**Provides:** Consistent header, Cancel/Save buttons, form container with spacing

### ConfirmDialog (Confirmations)

**Location:** `src/lib/components/ui/ConfirmDialog.svelte`

Replaces `window.confirm()` and `alert()`. Use for all confirmations.

```svelte
<ConfirmDialog
  isOpen={showDeleteDialog}
  title="Delete Job?"
  message="This action cannot be undone."
  confirmText="Delete"
  confirmStyle="destructive"
  onConfirm={handleDelete}
  onCancel={() => showDeleteDialog = false}
/>

<!-- Alert mode (no cancel button) -->
<ConfirmDialog
  isOpen={showAlert}
  title="Success"
  message="Job created successfully"
  variant="success"
  showCancel={false}
  confirmText="OK"
  onConfirm={() => showAlert = false}
/>
```

**Variants:** `success`, `error`, `warning`, `info` (adds colored icon)

**Never use:** `window.confirm()`, `window.alert()`, `console.log()`

### PreviewModal (Code Display)

**Location:** `src/lib/components/ui/PreviewModal.svelte`

For displaying code/config previews.

```svelte
<PreviewModal
  isOpen={showPreview}
  title="SLURM Script Preview"
  content={scriptContent}
  onClose={() => showPreview = false}
/>
```

**Content:** Displayed in monospace with syntax-appropriate background

## Svelte Patterns

### Store-Based State

**Backend data caches** - Stores hold data from Tauri commands:

```typescript
// stores/jobs.ts
export const jobs = writable<JobInfo[]>([]);

export async function loadJobs() {
  const result = await invoke<ApiResult<JobInfo[]>>('get_all_jobs');
  if (result.success && result.data) {
    jobs.set(result.data);
  }
}
```

**Derived stores** for computed values:

```typescript
export const completedJobs = derived(jobs, $jobs =>
  $jobs.filter(j => j.status === 'COMPLETED')
);
```

**Core stores:**

- `ui.ts` - Navigation, theme, console state
- `session.ts` - Connection state
- `jobs.ts` - Job list and operations
- `templateStore.ts` - Template management
- `clusterConfig.ts` - Cluster capabilities (partitions, QoS, presets)
- `settings.ts` - Database management

### Reactive Patterns

**Reactive statements** (`$:`):

```svelte
<script>
  import { connectionState } from '$lib/stores/session';

  $: canCreate = $connectionState === 'connected';
  $: filteredJobs = $jobs.filter(j => j.status !== 'CANCELLED');
</script>

<button disabled={!canCreate}>Create Job</button>
```

**Two-way binding:**

```svelte
<input bind:value={config.cores} type="number" />
```

**Event dispatchers:**

```svelte
<script>
  import { createEventDispatcher } from 'svelte';
  const dispatch = createEventDispatcher();
</script>

<button on:click={() => dispatch('select', { id: item.id })}>Select</button>
```

## Forms & Validation

### Form Structure

**Standard pattern:**

```svelte
<div class="namd-field-group">
  <label class="namd-label" for="field-id">
    Label Text {#if required}<span class="required">*</span>{/if}
  </label>
  <input
    class="namd-input"
    id="field-id"
    bind:value={entity.field}
    class:error={errors.field}
    placeholder="Example value"
  />
  {#if errors.field}
    <span class="namd-error-text">{errors.field}</span>
  {/if}
</div>
```

**For textareas:**

```svelte
<textarea class="namd-input" bind:value={text} rows="3"></textarea>
```

**For selects:**

```svelte
<select class="namd-input" bind:value={selected}>
  {#each options as option}
    <option value={option.value}>{option.label}</option>
  {/each}
</select>
```

**For checkboxes:**

```svelte
<label class="namd-label">
  <input type="checkbox" bind:checked={entity.enabled} />
  Enable feature
</label>
```

**For fieldsets (checkbox groups):**

```svelte
<fieldset class="namd-field-group">
  <legend class="namd-label">Options *</legend>
  <div class="checkbox-group">
    {#each options as option}
      <label>
        <input type="checkbox" value={option.id} />
        {option.label}
      </label>
    {/each}
  </div>
</fieldset>
```

### Validation Architecture

**Frontend:** UX feedback only (required fields, basic format checks)
**Backend:** All business rules, security, cluster constraints

```svelte
<script>
  let frontendErrors = {};  // UX validation
  let backendErrors = {};   // From ApiResult.error or ValidationResult.field_errors

  function validateUX() {
    frontendErrors = {};
    if (!name.trim()) frontendErrors.name = 'Name is required';
    if (cores <= 0) frontendErrors.cores = 'Must be > 0';
    return Object.keys(frontendErrors).length === 0;
  }

  async function handleSave() {
    if (!validateUX()) return;

    const result = await invoke('save_entity', { entity });
    if (!result.success) {
      backendErrors = result.field_errors || {};
    }
  }

  $: displayErrors = { ...frontendErrors, ...backendErrors };
</script>
```

## Common UI Patterns

### Add/Edit/Delete Pattern

**State management:**

```svelte
<script>
  let showDialog = false;
  let editing = null;  // null = add mode, object = edit mode
  let errors = {};

  function handleAdd() {
    editing = { name: '', /* defaults */ };
    errors = {};
    showDialog = true;
  }

  function handleEdit(item) {
    editing = { ...item };  // Clone to avoid mutating original
    errors = {};
    showDialog = true;
  }

  async function handleSave() {
    // Validation, then save
    const result = await saveEntity(editing);
    if (result.success) {
      showDialog = false;
      editing = null;
    }
  }
</script>

<button on:click={handleAdd}>Add</button>

{#each items as item}
  <button on:click={() => handleEdit(item)}>Edit</button>
  <button on:click={() => handleDelete(item)}>Delete</button>
{/each}

<EditDialog {isOpen} title={editing ? 'Edit' : 'Add'} {onSave} {onClose}>
  <!-- Form fields -->
</EditDialog>
```

### Collapsible Sections

Use native `<details>` element:

```svelte
<details>
  <summary class="section-summary">Section Title</summary>
  <div class="section-content">
    <!-- Content -->
  </div>
</details>

<style>
  .section-summary {
    cursor: pointer;
    font-weight: var(--namd-font-weight-semibold);
    user-select: none;
  }

  .section-summary:hover {
    color: var(--namd-primary);
  }
</style>
```

### Card Grids

**Standard pattern for lists:**

```svelte
<div class="card-grid">
  {#each items as item}
    <div class="namd-card">
      <div class="namd-card-header">
        <div class="item-info">
          <h4>{item.title}</h4>
          <code class="item-name">{item.id}</code>
        </div>
        <div class="item-actions">
          <button class="namd-button namd-button--secondary namd-button--sm" on:click={() => handleEdit(item)}>
            Edit
          </button>
          <button class="namd-button namd-button--destructive namd-button--sm" on:click={() => handleDelete(item)}>
            Delete
          </button>
        </div>
      </div>
      <div class="namd-card-content">
        <p>{item.description}</p>
        <div class="item-meta">
          <span>Meta 1</span>
          <span>Meta 2</span>
        </div>
      </div>
    </div>
  {/each}
</div>

<style>
  .card-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: var(--namd-spacing-lg);
  }

  .item-meta span {
    background: var(--namd-bg-muted);
    padding: var(--namd-spacing-xs) var(--namd-spacing-sm);
    border-radius: var(--namd-border-radius-sm);
    font-size: var(--namd-font-size-xs);
  }
</style>
```

### File Lists

**Use `.namd-file-list` pattern:**

```svelte
{#if files.length > 0}
  <div class="namd-file-list">
    {#each files as file}
      <div class="namd-file-item">
        <div class="namd-file-content">
          <span class="namd-file-name">{file.name}</span>
          <span class="namd-file-metadata">{formatBytes(file.size)}</span>
        </div>
        <button class="namd-button namd-button--sm" on:click={() => download(file)}>
          Download
        </button>
      </div>
    {/each}
  </div>
{:else}
  <div class="namd-file-list-empty">No files available</div>
{/if}
```

**With upload progress:**

```svelte
<div class="namd-file-item">
  <div class="namd-file-progress-bg" style="width: {progress}%"></div>
  <div class="namd-file-content">
    <span class="namd-file-name">{file.name}</span>
    <span class="namd-file-progress-text">{progress}%</span>
  </div>
</div>
```

### Status Badges

```svelte
<span class="namd-badge namd-badge--{variant}">{status}</span>
```

**Variants:** `success`, `error`, `warning`, `info`, `pending`

### Tab Navigation

**Pattern:**

```svelte
<script>
  let activeTab = 'overview';
</script>

<nav class="tabs">
  <button class:active={activeTab === 'overview'} on:click={() => activeTab = 'overview'}>
    Overview
  </button>
  <button class:active={activeTab === 'details'} on:click={() => activeTab = 'details'}>
    Details
  </button>
</nav>

<div class="tab-content">
  {#if activeTab === 'overview'}
    <OverviewTab />
  {:else if activeTab === 'details'}
    <DetailsTab />
  {/if}
</div>
```

## Adding New Features

### Adding a New Page

1. **Create component** in `src/lib/components/pages/MyPage.svelte`
2. **Add route** in `uiStore.ts` if needed
3. **Add sidebar item** in `AppSidebar.svelte`
4. **Update breadcrumbs** in `AppHeader.svelte`

### Adding a New Modal

**For simple confirmations/alerts:**
Use `ConfirmDialog.svelte` (already exists)

**For forms:**
Use `EditDialog.svelte` wrapper with form content in slot

**Never:**
Create a new modal component from scratch

### Adding a New Form

**Pattern:**

```svelte
<EditDialog {isOpen} title="Add Entity" {onSave} {onClose}>
  <svelte:fragment slot="form">
    {#if entity}
      <!-- Field groups here -->
      <div class="namd-field-group">
        <label class="namd-label">Field</label>
        <input class="namd-input" bind:value={entity.field} />
      </div>
    {/if}
  </svelte:fragment>
</EditDialog>
```

### Adding Backend Integration

**Call Tauri command, update store:**

```svelte
<script>
  import { invoke } from '@tauri-apps/api/core';
  import { myStore } from '$lib/stores/myStore';

  async function handleAction() {
    const result = await invoke<ApiResult<T>>('my_command', { params });
    if (result.success && result.data) {
      myStore.updateData(result.data);
      // Backend already shows toast if needed
    }
    // Backend already shows error toast if failed
  }
</script>
```

**Remember:**

- Frontend does NOT validate business rules
- Frontend does NOT perform calculations
- Backend handles all toasts via `log_info!(..., show_toast: true)`
- No `console.log()` (invisible in production app)

## Best Practices

### Component Organization

```
components/
├── layout/       # App shell, sidebar, header
├── pages/        # Top-level page components
├── [feature]/    # Feature-specific components (create-job/, job-detail/, etc.)
├── ui/           # Reusable primitives (Dialog, buttons, badges)
└── shared/       # Shared utilities
```

**Extract when:**

- Component used in 2+ places
- Logical grouping emerges (job-detail tabs, create-job tabs)

**Don't extract:**

- One-off components
- Page-specific layouts

### State Management

**Use stores for:**

- Data from backend (jobs, templates, config)
- Global UI state (theme, navigation, connection status)

**Use component state for:**

- Form inputs
- Modal open/closed
- Local UI state

**Never:**

- Put business logic in stores
- Duplicate backend data
- Create store for every component

### Styling

**Do:**

- Use design system variables
- Use existing `.namd-*` classes
- Add component-specific classes when needed
- Follow BEM-like naming (`.component__element--modifier`)

**Don't:**

- Hardcode colors or spacing
- Create custom button/form styles
- Use inline styles
- Duplicate existing component classes
