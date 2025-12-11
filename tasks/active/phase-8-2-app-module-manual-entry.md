# Task: Phase 8.2 - Cluster Application Module Management (Manual Entry)

## Objective
Allow users to configure and pin cluster application module load sequences via manual entry in Settings, replacing hardcoded module versions in SLURM job scripts with user-selected configurations.

## Context
- **Starting state**: Module versions hardcoded in `slurm/script_generator.rs` (gcc/14.2.0, openmpi/5.0.6, namd/3.0.1_cpu). If cluster admin updates modules, all jobs fail. Users cannot configure multiple app versions (e.g., CPU vs GPU).
- **Delivered state**: Users manually enter module load order in Settings after running `module spider` commands themselves. Jobs copy module load order at creation time (island pattern). No hardcoded module dependencies remain.
- **Foundation**: Phase 8.1 user-editable cluster config pattern (JSON document store, Settings page sections, validation patterns). Existing input sanitization infrastructure (`validation/input.rs`).
- **Dependencies**: None (Phase 7 complete)
- **Testing approach**: Unit tests for validation/sanitization functions, integration tests for CRUD operations, manual E2E testing for full workflow. No complex SSH mocking needed (as outlined in `docs/CONTRIBUTING.md#testing-strategy`).

## Implementation Plan

### Blockers

- [ ] **Database Schema & Types**
  - [ ] Add `pinned_apps` table to `database/mod.rs` schema (pure document store: `id TEXT PRIMARY KEY, data TEXT NOT NULL`)
  - [ ] Create `PinnedApp` Rust struct in `types/core.rs` with fields: `id`, `display_name`, `main_module`, `load_order: Vec<String>`, `pinned_at`
  - [ ] Add `#[serde(default)]` to all fields for clean deserialization
  - [ ] Create matching TypeScript `PinnedApp` interface in `src/lib/types/api.ts`
  - [ ] Add database CRUD methods: `save_pinned_app()`, `load_pinned_app()`, `load_all_pinned_apps()`, `delete_pinned_app()`

- [ ] **JobInfo Breaking Change**
  - [ ] Add `module_load_order: Vec<String>` field to `JobInfo` struct in `types/core.rs` with `#[serde(default)]`
  - [ ] Add `pinned_app_id: String` field to `CreateJobParams` struct in `types/commands.rs` (lines 29-35)
  - [ ] Update TypeScript `CreateJobParams` interface in `src/lib/types/api.ts` with `pinned_app_id: string`
  - [ ] Update TypeScript `JobInfo` interface in `src/lib/types/api.ts` with `module_load_order: string[]`
  - [ ] Update `create_job_info()` in `automations/job_creation.rs` to accept `module_load_order` parameter
  - [ ] Update all test fixtures (`slurm/script_generator.rs` line 158-185 and others) with default module_load_order

- [ ] **Input Validation**
  - [ ] Add `sanitize_module_name()` to `validation/input.rs` following `sanitize_job_id()` pattern
  - [ ] Strict validation: allow only `a-zA-Z0-9._/-` characters
  - [ ] Reject module names containing `..` (path traversal prevention)
  - [ ] Add comprehensive unit tests with valid/invalid module name cases

### Core Functionality

- [ ] **Backend Commands**
  - [ ] Create `commands/pinned_apps.rs` following `commands/templates.rs` structure
  - [ ] Implement `create_pinned_app_manual` command:
    - Parameters: `display_name: String`, `load_order: Vec<String>`
    - Validate display_name not empty
    - Filter empty lines from load_order (silently remove whitespace-only lines)
    - Validate at least one module remains after filtering
    - Sanitize each module name using `validation::input::sanitize_module_name()`
    - Extract `main_module` from last item in load_order (target application)
    - Generate UUID for id, set pinned_at to `chrono::Utc::now().to_rfc3339()`
    - Save using `with_database(|db| db.save_pinned_app(&app))`
    - Return `ApiResult<String>` (app ID)
  - [ ] Implement `list_pinned_apps` command (no summary type needed - full objects)
  - [ ] Implement `delete_pinned_app` command (no reference checking - jobs are islands)
  - [ ] Add `load_pinned_app_or_fail()` helper to `commands/helpers.rs` following `load_template_or_fail()` pattern
  - [ ] Add `pub mod pinned_apps;` to `commands/mod.rs`
  - [ ] Register commands in `lib.rs` invoke handler: `create_pinned_app_manual`, `list_pinned_apps`, `delete_pinned_app`

- [ ] **Script Generator Update**
  - [ ] Update `build_module_loads()` in `slurm/script_generator.rs` (lines 117-123):
    - Change signature: `fn build_module_loads(job_info: &JobInfo) -> Result<String>`
    - Return error if `job_info.module_load_order.is_empty()`
    - Delete hardcoded module strings
    - Iterate over `job_info.module_load_order` generating `module load` commands
  - [ ] Update call site at line 31: `Self::build_module_loads(job_info)?`
  - [ ] Update tests (lines 271-278): test with specific module_load_order, test empty array (should error)

- [ ] **Job Creation Integration**
  - [ ] Update `execute_job_creation_with_progress()` in `automations/job_creation.rs`:
    - Validate `params.pinned_app_id` not empty, return error if missing
    - Load pinned app: `helpers::load_pinned_app_or_fail(&params.pinned_app_id, "Job Creation")?`
    - Extract: `let module_load_order = pinned_app.load_order.clone();`
    - Pass module_load_order to `create_job_info()`

- [ ] **Frontend Store**
  - [ ] Create `src/lib/stores/pinnedAppsStore.ts` using `storeFactory.ts` `createStore` pattern
  - [ ] Use `createStore<PinnedApp[]>` with `loadCommand: 'list_pinned_apps'`
  - [ ] Export `loadPinnedApps()` and `deletePinnedApp(id)` functions
  - [ ] Add store tests following `storeFactory.test.ts` pattern (mock invoke, test load/delete)
  - [ ] Initialize store in `AppShell.svelte` on mount

- [ ] **Settings Page UI**
  - [ ] Add "Cluster Apps" section to `SettingsPage.svelte` between About and Database sections
  - [ ] Create manual entry form:
    - Display name input (`.namd-input` class)
    - Module load order textarea (`.namd-input` class, multiline)
    - Always-visible help text with example:
      ```
      To find modules: SSH to cluster and run `module spider namd` then `module spider namd/3.0.1_cpu`.
      Enter modules in load order (dependencies first, target app last), one per line.

      Example:
      gcc/14.2.0
      openmpi/5.0.6
      namd/3.0.1_cpu
      ```
    - Save button (`.namd-button .namd-button--primary`)
    - Clear button (`.namd-button .namd-button--secondary`)
  - [ ] Client-side validation: display_name not empty, at least one non-empty line in textarea
  - [ ] Show inline error messages below fields when validation fails
  - [ ] Pinned apps list display (card-based, following `TemplatesPage.svelte` pattern):
    - Use `.namd-card`, `.namd-card-header`, `.namd-card-content` classes
    - Show display_name (bold), main_module (code font), load_order (numbered list or arrow-separated)
    - Delete button only (`.namd-button .namd-button--destructive`) - no editing, delete and recreate
    - Use existing `ConfirmDialog.svelte` for delete confirmation
  - [ ] Show "No cluster applications pinned yet" when list empty

- [ ] **Job Creation UI Integration**
  - [ ] Add app selector dropdown to `ResourcesTab.svelte` after QoS selector
  - [ ] Load `$pinnedAppsStore` on mount
  - [ ] Show display_name for each app in dropdown
  - [ ] If no apps pinned: show disabled dropdown with message and link to Settings
  - [ ] Bind to `selectedAppId: string` local state
  - [ ] Add validation: check `selectedAppId` not empty before submission, show error "Application selection required"
  - [ ] Pass `pinned_app_id: selectedAppId` in CreateJobParams to backend
  - [ ] Update Review tab to show selected app: "Application: {display_name} ({main_module})"

- [ ] **Default Seeding**
  - [ ] Create `src-tauri/default_apps/alpine_namd_cpu.json` with NAMD 3.0.1 CPU default for Alpine cluster
  - [ ] JSON structure: `{ "id": "alpine_namd_cpu_default", "display_name": "NAMD 3.0.1 CPU (Alpine)", "main_module": "namd/3.0.1_cpu", "load_order": [...], "pinned_at": "..." }`
  - [ ] Create `ensure_default_pinned_apps_loaded()` in `database/mod.rs` following `ensure_default_templates_loaded()` pattern (lines 328-344)
  - [ ] Use `static PINNED_APPS_LOADED: AtomicBool = AtomicBool::new(false);`
  - [ ] Add embed macro: `const DEFAULT_APPS: &str = include_str!("../default_apps/alpine_namd_cpu.json");`
  - [ ] Call seeding function during app initialization (same location as template seeding)

### Enhancements

- [ ] **CSS Documentation**
  - [ ] Document required CSS classes (already exist in `app.css`):
    - `.namd-input` - form inputs and textareas
    - `.namd-button`, `.namd-button--primary`, `.namd-button--secondary`, `.namd-button--destructive` - buttons
    - `.namd-field-group` - form field containers
    - `.namd-card`, `.namd-card-header`, `.namd-card-content` - card layouts
  - [ ] No new CSS needed - use existing design system

## Success Criteria

### Functional Success
- [ ] User can manually enter module load order in Settings
- [ ] Display name and module list saved to database
- [ ] Pinned apps appear in list with delete action
- [ ] Job creation dropdown shows pinned apps
- [ ] Job creation blocked when no apps pinned (with helpful message)
- [ ] Selected app's load_order copied to job at creation time
- [ ] SLURM script contains correct module load commands in correct order
- [ ] Deleting pinned app does not affect existing jobs (island pattern verified)
- [ ] Default NAMD app seeds on first run

### Technical Success
- [ ] No hardcoded module versions in script generator (all dynamic from job metadata)
- [ ] Jobs store complete module_load_order (no references to pinned apps)
- [ ] Module name sanitization prevents shell injection
- [ ] Empty lines filtered silently from textarea input
- [ ] main_module auto-extracted from last load_order item
- [ ] All validation happens in backend (display name, module names, load order not empty)

### Quality Success
- [ ] All validation functions have unit tests
- [ ] Store operations have tests with mocked invoke
- [ ] Build passes with no warnings
- [ ] Code follows existing patterns (storeFactory, CRUD commands, helpers)
- [ ] No duplicate validation or sanitization logic (centralized in validation/input.rs)

## Key Technical Decisions

### Why Manual Entry Over Auto-Discovery
- **Reasoning**: Auto-discovery requires parsing `module spider` output (fragile), SSH command execution (complex), recursive dependency resolution (DAG + topological sort), and connection state management. Manual entry is 75% less implementation work, more stable (no parsing), and aligns with "scientists need reliability over features" philosophy.
- **Alternatives considered**: Auto-discovery with `module -t spider` commands (rejected: 50+ tasks, fragile parsing, network dependencies)
- **Trade-offs**: Users spend 30 seconds running module spider themselves vs weeks of development time and ongoing maintenance burden for auto-discovery

### Why Extract main_module from Last Load Order Item
- **Reasoning**: Module spider output format lists dependencies first, target application last. This mirrors how users naturally think about load order.
- **Integration**: User enters exactly what they see from `module -t spider` output, we extract target app automatically
- **Validation**: Ensure load_order not empty before extraction

### Why No Editing (Delete and Recreate Only)
- **Reasoning**: Simpler UI, no edit form state management, no partial update logic. Jobs copy load_order at creation (island pattern), so editing wouldn't affect existing jobs anyway.
- **Trade-offs**: User must delete and recreate to change app, but module configs rarely change

### Why Filter Empty Lines Silently
- **Reasoning**: User-friendly - forgives trailing newlines and whitespace from copy/paste. Error on empty lines would frustrate users.
- **Implementation**: `load_order.lines().map(|s| s.trim()).filter(|s| !s.is_empty())`

### Why No PinnedAppSummary Type
- **Reasoning**: PinnedApp is small (~200 bytes: id, display_name, main_module, load_order with ~3-5 modules, pinned_at). 10 apps = 2KB. No performance benefit from summary type, only added complexity.
- **Alternatives considered**: Separate summary type for list views (rejected: unnecessary abstraction)

## Integration with Existing Code

### Leverage Existing Patterns

- **Use `ValidationResult`**: From `validation/job_validation.rs` lines 10-48 - reuse exact type for any validation commands
- **Use `ApiResult<T>`**: From `types/core.rs` lines 5-50 - all commands return this
- **Use `with_database()`**: From `database/mod.rs` - synchronous DB access wrapper
- **Use `Dialog.svelte`**: From `components/ui/Dialog.svelte` - for any modal dialogs needed
- **Use `ConfirmDialog.svelte`**: From `components/ui/ConfirmDialog.svelte` - for delete confirmation
- **Follow CRUD command pattern**: From `commands/templates.rs` - list/get/create/delete structure
- **Follow helper pattern**: From `commands/helpers.rs` - `load_*_or_fail()` functions
- **Use storeFactory**: From `stores/storeFactory.ts` - `createStore` for all new stores
- **Use existing CSS classes**: From `app.css` - `.namd-button`, `.namd-input`, `.namd-card`, `.namd-field-group`

### Where to Hook In

```rust
// Database - add to existing schema initialization
database::mod.rs::initialize_schema() // Add: pinned_apps table creation

// Types - extend existing type definitions
types::core.rs::JobInfo // Add: module_load_order field
types::commands.rs::CreateJobParams // Add: pinned_app_id field

// Validation - extend existing input validation
validation::input.rs // Add: sanitize_module_name() function

// Script generation - modify existing function
slurm::script_generator.rs::build_module_loads() // Change: use job_info.module_load_order instead of hardcoded

// Job creation - integrate pinned app lookup
automations::job_creation.rs::execute_job_creation_with_progress() // Add: load pinned app and copy load_order

// New command module (follow templates.rs pattern)
commands::pinned_apps.rs::create_pinned_app_manual()
commands::pinned_apps.rs::list_pinned_apps()
commands::pinned_apps.rs::delete_pinned_app()

// Frontend store (follow templateStore.ts pattern)
stores::pinnedAppsStore.ts // New: using createStore factory
```

### Phase Dependencies (Critical Ordering)

1. Phase 1 (Database & Types) → Phase 2 (JobInfo Changes)
2. Phase 2 (JobInfo Changes) → Phase 5 (Script Generator) - script generator needs module_load_order field
3. Phase 4 (Backend Commands) → Phase 6 (Job Creation Integration) - job creation needs load_pinned_app_or_fail helper
4. Phase 7 (Frontend Store) → Phase 8-9 (UI) - UI needs store subscriptions

## References

- **NAMDRunner patterns**: `docs/CONTRIBUTING.md` (testing strategy), `docs/ARCHITECTURE.md` (system design)
- **Phase 8.1 pattern**: `tasks/active/phase-8-1-user-editable-cluster-config.md` - same JSON document store, Settings page sections, validation approach
- **Validation pattern**: `src-tauri/src/validation/input.rs` - input sanitization, `src-tauri/src/validation/job_validation.rs` - ValidationResult type
- **CRUD commands pattern**: `src-tauri/src/commands/templates.rs` - complete CRUD with ApiResult
- **Helper pattern**: `src-tauri/src/commands/helpers.rs` - load_*_or_fail functions
- **Store factory**: `src/lib/stores/storeFactory.ts` - createStore pattern, `src/lib/stores/templateStore.ts` - usage example
- **Dialog components**: `src/lib/components/ui/Dialog.svelte`, `ConfirmDialog.svelte`
- **Settings page**: `src/lib/components/pages/SettingsPage.svelte` - section structure
- **Design system**: `src/lib/styles/app.css` - all `.namd-*` classes

## Progress Log

[2025-12-07] - Task plan created from extensive design sessions. Manual entry approach chosen over auto-discovery for stability and simplicity. All decisions locked in. Ready for implementation.

## Completion Process

After implementation and testing:
- [ ] Run code review using `.claude/agents/review-refactor.md`
- [ ] Implement recommended refactoring improvements
- [ ] Update and archive task to `tasks/completed/phase-8-2-app-module-manual-entry.md`
- [ ] Update `tasks/roadmap.md` progress
- [ ] Update `docs/ARCHITECTURE.md` with implementation details (jobs are islands pattern, module_load_order field)

## Open Questions

None - all decisions locked in during design phase.
