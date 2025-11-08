# Task: Phase 7.1 - Template System Refactor

## Objective
Replace hardcoded NAMD configuration with a flexible template system where templates are stored in the database and users can create, edit, and manage simulation templates through a UI.

## Context
- **Original system**: NAMD config used hardcoded `NAMDConfig` struct with fixed fields. File types were auto-detected. ConfigurationTab had hardcoded form fields.
- **New system**: Templates stored in database define NAMD config structure and form UI. Jobs reference template + values. Users manage templates via Templates page. Files are form fields assigned to template variables.
- **Foundation**: SQLite database, IPC command pattern, dynamic UI rendering
- **Testing approach**: Unit tests for template rendering and validation logic only. No tests for framework code.

## Implementation Summary

### Core Architecture

**Template System**:
- Templates define NAMD config as text with `{{variable}}` placeholders
- Variable definitions specify type (Number, Text, Boolean, FileUpload), constraints, and UI labels
- Templates stored in database, embedded defaults compiled into binary
- Jobs store `template_id` + `template_values` (HashMap<String, Value>)

**Data Flow**:
```
Template + Values → Validate → Render → Upload Files → Generate SLURM Script → Submit
```

### What Was Deleted

**Backend**:
- ✅ `NAMDConfig` struct and all related types (ExecutionMode, CellBasisVector, InputFile)
- ✅ Demo mode module (`src-tauri/src/demo/`)
- ✅ `commands/system.rs` (demo mode commands)
- ✅ File type auto-detection utilities
- ✅ `namd_constants.rs` (hardcoded NAMD parameters)
- ✅ Hardcoded NAMD config generation in script_generator.rs

**Frontend**:
- ✅ `ConfigurationTab.svelte` (hardcoded form)
- ✅ `CreateJobTabs.svelte` (old tab system)
- ✅ Demo mode toggle from ConnectionDropdown
- ✅ All demo mode logic from stores (clientFactory, jobs, session)
- ✅ `mockJobData.ts` and in-app mock data
- ✅ Job detail tabs for NAMDConfig (ConfigurationTab, InputFilesTab)

### What Was Built

**Backend (`src-tauri/src/`)**:
- ✅ `templates/mod.rs` - Template module
- ✅ `templates/types.rs` - Template, VariableDefinition, VariableType structs
- ✅ `templates/renderer.rs` - Template rendering with variable substitution
- ✅ `templates/validation.rs` - Type checking and constraint validation
- ✅ `commands/templates.rs` - 6 IPC commands (list, get, create, update, delete, validate)
- ✅ `database/mod.rs` - Template CRUD, embedded template loading
- ✅ Default templates embedded using `include_str!` macro

**Default Templates (Embedded in Binary)**:
- ✅ `vacuum_optimization_v1.json` - Based on origami tutorial step 2, 16 variables
- ✅ `explicit_solvent_npt_v1.json` - Based on origami tutorial step 3, 17 variables

**Frontend (`src/lib/`)**:
- ✅ `types/template.ts` - TypeScript types matching Rust backend
- ✅ `stores/templateStore.ts` - Template state management
- ✅ `stores/ui.ts` - Template-edit view, navigation, breadcrumbs
- ✅ `stores/jobs.ts` - Connection failure detection in all operations
- ✅ `utils/logger.ts` - Centralized logging utility (window.appLogger)
- ✅ `utils/template-utils.ts` - Shared template functions (extraction, labels, samples)
- ✅ `utils/template-utils.test.ts` - Frontend unit tests (17 tests)
- ✅ `components/pages/TemplatesPage.svelte` - Unified template list with is_builtin badges
- ✅ `components/pages/TemplateEditorPage.svelte` - Full-page template editor
- ✅ `components/templates/TemplateEditor.svelte` - Template form with auto-variable detection
- ✅ `components/templates/VariableEditor.svelte` - Variable metadata editor (required fields, dropdown for Boolean)
- ✅ `components/create-job/CreateJobTabs.svelte` - 3-tab interface with debounced validation
- ✅ `components/create-job/ResourcesTab.svelte` - Preset pills, validation, cost, preview button
- ✅ `components/create-job/ConfigureTab.svelte` - Job name, DynamicJobForm, preview button
- ✅ `components/create-job/ReviewTab.svelte` - Validation summary, file upload progress
- ✅ `components/create-job/DynamicJobForm.svelte` - Dynamic form with template text ordering
- ✅ `components/pages/CreateJobPage.svelte` - Tab-based job creation
- ✅ `components/ui/ConfirmDialog.svelte` - Unified confirmation dialog
- ✅ `components/ui/PreviewModal.svelte` - Reusable preview modal (NAMD config, SLURM script)
- ✅ `components/layout/LogsPanel.svelte` - Renamed from SSHConsolePanel

### Key Implementation Details

**Template Loading (Lazy, On-Demand)**:
- Templates embedded in binary using `include_str!` at compile time
- Loaded on first `list_templates` call (when frontend is ready for logs)
- Atomic flag prevents duplicate loading
- Works identically in dev and production

**Template Rendering**:
- Simple regex-based variable substitution
- FileUpload variables: prepends "input_files/" to filename during rendering
- Type conversion: Boolean → "yes"/"no", Number → string, Text → as-is
- Errors if unreplaced variables remain after rendering

**Variable Types** (All Constraints Required):
- **Number**: min (required), max (required), default (required) - rendered as numeric string
- **Text**: default (required) - rendered as-is
- **Boolean**: default (required, dropdown UI) - rendered as "yes"/"no"
- **FileUpload**: extensions list - filename prepended with "input_files/", validated for empty and extension match

**Auto-Variable Detection**:
- Variables automatically detected from template text using regex
- Tracks first occurrence position for ordering (template text order preserved)
- Duplicate variables handled correctly (same `{{var}}` = one entry)
- Variable list updates as user types (500ms debounce)
- Adding/removing `{{var}}` in template automatically adds/removes variable
- Existing metadata preserved when variable still exists
- New variables default to Text type with auto-generated label (snake_case → Title Case)
- Display order follows template text order (not alphabetical)

**Required Fields Logic**:
- All variables implicitly required (no optional checkbox)
- Rationale: Every `{{variable}}` must be filled for template to render
- Default values provide good UX (user doesn't fill repetitive values)
- FileUpload variables must have file selected (no default)
- If user wants optional behavior, they duplicate template and remove the variable

**Serde Format**:
- Uses **externally-tagged** enum format: `{"FileUpload": {"extensions": [...]}}`
- Matches JSON template file structure

**Page-Based Navigation (Not Modals)**:
- Template editor is full page with breadcrumbs (Templates > Create/Edit Template)
- Delete confirmation uses small modal dialog (appropriate for confirmation)
- Consistent with Jobs and Create Job pages

**Delete Template Functionality**:
- Delete button on all templates in list view (built-in and custom)
- Delete button in template editor (edit mode only, left side of action bar)
- Backend prevents deletion if jobs reference template (shows count in error)
- Users can delete built-in templates if not needed

**Create Job Flow (3-Tab Interface)**:
- Tab 1: Resources - Preset pills, manual config (collapsible), real-time validation/cost, SLURM script preview
- Tab 2: Configure - Job name, template selector, dynamic form with template text ordering, NAMD config preview
- Tab 3: Review - Validation summary, resource/template values, file upload progress with animated bars
- Debounced backend validation (500ms) updates errors in real-time as user types
- Preview buttons call backend to render actual NAMD config and SLURM script
- Uses global `namd-tabs` CSS for consistency

**Connection Failure Detection**:
- Pattern-matches errors from all job operations (sync, create, submit, delete)
- Automatically transitions state to 'Expired' on timeout/connection errors
- Zero extra network calls (uses existing operation failures)
- User sees "Connection Expired" status and can reconnect

**Offline Support**:
- Jobs loaded from database on app startup
- Cached jobs visible when disconnected
- All action buttons disabled offline (Submit, Delete, Download, Create Job, Sync)

**Centralized Logging**:
- Created logger utility with consistent API (debug, error, command, output)
- Renamed window.sshConsole → window.appLogger throughout codebase
- All components use logger utility (no console.error or direct window access)
- Backend uses info_log!/error_log! macros (emits Tauri events)

**Template Built-in Detection**:
- Added is_builtin boolean field to Template schema (replaces fragile .includes('_v1') check)
- Database column: is_builtin INTEGER NOT NULL
- Embedded templates: is_builtin = true
- User-created/edited templates: is_builtin = false
- Frontend uses proper flag instead of naming convention

**Theme & Accessibility**:
- Centralized CSS variables for all colors (light + dark themes)
- Hover states maintain readable contrast in both themes (primary-hover-fg, sidebar-active-hover)
- Unified ConfirmDialog and PreviewModal components

**UX & Code Quality**:
- Sync time updates every 10 seconds (reactive timer)
- Variable ordering follows template text via shared extraction utility
- Template card buttons aligned to bottom (flexbox)
- Removed setTimeout anti-pattern (errors cleared on next operation, matches app pattern)
- Extracted duplicate template utilities (extraction, label generation, sample values)

## Success Criteria

### Functional Requirements ✅ (Complete)
- ✅ User can see 2 built-in templates on fresh install
- ✅ User can create custom templates via Templates page
- ✅ Dynamic form generates from template variables
- ✅ Job creation uses template system end-to-end
- ✅ Template editor auto-detects variables and allows editing metadata
- ✅ Delete template available in list and editor (with job count protection)
- ✅ Job details display template information properly (Overview, SLURM Logs, Output Files)

### Technical Requirements ✅ (Complete)
- ✅ Zero NAMDConfig references in codebase
- ✅ Zero demo mode code
- ✅ Template renderer handles all variable types
- ✅ Database loads embedded defaults on first run
- ✅ Validation prevents invalid jobs (type checking, required fields, ranges)
- ✅ All template CRUD operations work via IPC
- ✅ Zero compiler warnings (Rust + frontend)

### Quality Requirements ✅ (Complete)
- ✅ Unit tests for template renderer (5 tests - variable substitution, types, errors)
- ✅ Unit tests for template validation (9 tests - required fields, ranges, types, extensions)
- ✅ Unit tests for template utilities (17 tests - extraction, label generation, sample values)
- ✅ No demo mode, file auto-detection, or backwards compatibility code
- ✅ Clean codebase following NAMDRunner standards
- ✅ **159 tests passing** (137 Rust + 22 frontend), zero warnings

## Key Technical Decisions (Final)

**Templates Embedded in Binary**:
- Using `include_str!` macro to embed JSON at compile time
- No runtime file system access needed
- Works identically in dev and production

**Lazy Template Loading**:
- Loads on first `list_templates` call (not app startup)
- Ensures frontend is ready to receive logs
- Atomic flag prevents duplicate loading

**Page-Based Navigation**:
- Template editor is full page (not modal)
- Consistent with Jobs and Create Job pages
- Better UX for large forms

**Auto-Variable Detection (Not Manual Add/Delete)**:
- Variables parsed from template text, not manually added
- Adding `{{var}}` in template = variable appears in list
- Removing from template = variable removed from list
- User only configures metadata (type, label, constraints)
- Simpler mental model: template text is source of truth

**All Variables Required (No Optional Checkbox)**:
- Every variable must have value for template to render
- Implicit requirement eliminates unnecessary UI checkbox
- Default values provide good UX without making fields optional
- If user wants optional behavior, duplicate template and remove variable

**Simple Variable Types**:
- No conditional visibility or complex validators
- Different simulation types = different templates
- Keeps template system maintainable

**No Migration or Backwards Compatibility**:
- App not released to users
- Clean refactor better than migration code
- All test data expendable

## Completion Checklist

Before archiving this task:
- [x] Complete job detail view refactors (deleted broken tabs, OverviewTab shows template data)
- [x] Implement template editor variable management (auto-detection with position ordering)
- [x] Delete commented test code (broken validation tests removed)
- [x] Write unit tests for core template logic (renderer: 5 tests, validation: 9 tests)
- [x] Restore resource presets and validation (3-tab Create Job interface)
- [x] Unified confirmation dialogs (ConfirmDialog component reused)
- [x] Theme consistency (centralized CSS variables, hover states readable)
- [x] Connection failure detection (auto-transition to Expired)
- [x] Offline mode support (cached jobs, buttons disabled)
- [ ] Update documentation (ARCHITECTURE.md, DB.md, API.md)

## Current Status

**Phase**: Implementation complete - ready for user testing

**What's Complete**:
- Template system fully functional (create, edit, delete, duplicate with is_builtin flag)
- Auto-variable detection with shared utilities (extraction, label generation, sample values)
- Job creation with 3-tab interface (Resources, Configure, Review) and debounced validation
- Preview features (NAMD config and SLURM script rendered by backend)
- Resource presets, real-time validation, cost estimation
- Job details display template information (Overview, SLURM Logs, Output Files)
- Connection timeout detection (auto-transitions to Expired on operation failures)
- Offline mode with cached data (loaded on startup, action buttons disabled)
- Centralized logging (logger utility, window.appLogger, no console.error)
- Code quality (no setTimeout anti-patterns, extracted duplicate code, proper error handling)
- Theme consistency (centralized CSS variables with readable hover states)
- 159 tests passing (137 Rust + 22 frontend), zero warnings

**Blockers**: None

**Next**: User end-to-end testing → Code review → Documentation → Archive
