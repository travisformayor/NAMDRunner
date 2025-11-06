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
- ✅ `stores/ui.ts` - Added template-edit view and navigation
- ✅ `stores/jobs.ts` - Connection failure detection in all operations
- ✅ `components/pages/TemplatesPage.svelte` - Unified template list with badges
- ✅ `components/pages/TemplateEditorPage.svelte` - Full-page template editor
- ✅ `components/templates/TemplateEditor.svelte` - Template form with auto-variable detection
- ✅ `components/templates/VariableEditor.svelte` - Variable metadata editor
- ✅ `components/create-job/CreateJobTabs.svelte` - 3-tab interface (Resources, Configure, Review)
- ✅ `components/create-job/ResourcesTab.svelte` - Preset pills + validation + cost estimation
- ✅ `components/create-job/ConfigureTab.svelte` - Job name + DynamicJobForm wrapper
- ✅ `components/create-job/ReviewTab.svelte` - Validation summary + file upload progress
- ✅ `components/create-job/DynamicJobForm.svelte` - Dynamic form from template variables
- ✅ `components/pages/CreateJobPage.svelte` - Tab-based job creation
- ✅ `components/ui/ConfirmDialog.svelte` - Unified confirmation dialog (reused across app)
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

**Variable Types**:
- **Number**: min/max/default constraints, rendered as numeric string
- **Text**: default value, rendered as-is
- **Boolean**: default value, rendered as "yes"/"no"
- **FileUpload**: file extensions filter, filename extracted and prepended with "input_files/"

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
- Tab 1: Resources - Preset pills (Small/Medium/Large/GPU), manual config (collapsible), real-time validation/cost
- Tab 2: Configure - Job name + template selector + dynamic form (template variables auto-generated)
- Tab 3: Review - Validation summary, resource review, template values, file upload progress, submit button
- Uses global `namd-tabs` CSS for consistency with job details

**Connection Failure Detection**:
- Pattern-matches errors from all job operations (sync, create, submit, delete)
- Automatically transitions state to 'Expired' on timeout/connection errors
- Zero extra network calls (uses existing operation failures)
- User sees "Connection Expired" status and can reconnect

**Offline Support**:
- Jobs loaded from database on app startup
- Cached jobs visible when disconnected
- All action buttons disabled offline (Submit, Delete, Download, Create Job, Sync)

**Theme & Accessibility**:
- Centralized CSS variables for all colors (light + dark themes)
- Hover states maintain readable contrast in both themes
- Unified ConfirmDialog component (replaces duplicate modal implementations)

**UX Improvements**:
- Sync time updates every 10 seconds (reactive timer, not just on navigation)
- Variable ordering follows template text (not alphabetical)
- Template card buttons aligned to bottom (flexbox)
- All templates editable (no "View" vs "Edit" distinction)

### Remaining Work

**End-to-End Verification** (User Testing):
- [ ] Test complete job lifecycle with templates
- [ ] Verify template CRUD operations
- [ ] Test resource presets and validation
- [ ] Test connection timeout detection
- [ ] Verify offline mode behavior

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
- ✅ Unit tests for validation logic (9 tests - required fields, ranges, types, extensions)
- ✅ No demo mode, file auto-detection, or backwards compatibility code
- ✅ Clean codebase following NAMDRunner standards
- ✅ **137 tests passing**, zero warnings

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
- [ ] End-to-end testing verification (user testing)
- [ ] Run code review with review-refactor agent
- [ ] Update documentation (ARCHITECTURE.md, DB.md, API.md)
- [ ] Archive task to `tasks/completed/`

## Current Status

**Phase**: Implementation complete - ready for user testing

**What's Complete**:
- Template system fully functional (create, edit, delete, duplicate templates)
- Auto-variable detection with position-based ordering (template text order preserved)
- Job creation with 3-tab interface (Resources with presets, Configure with templates, Review with validation)
- Resource presets, real-time validation, and cost estimation
- Job details display template information (Overview, SLURM Logs, Output Files)
- Connection timeout detection (auto-transitions to Expired state on failures)
- Offline mode with cached data (jobs visible, action buttons disabled)
- Unified confirmation dialogs (ConfirmDialog component)
- Theme consistency (centralized CSS variables, readable hover states)
- 137 tests passing, zero warnings

**Blockers**: None

**Next**: User end-to-end testing → Code review → Documentation → Archive
