# Task: Phase 7.1 - Template System Refactor

## Objective
Replace hardcoded NAMD configuration with a flexible template system where templates are stored in the database and users can create, edit, and manage simulation templates through a UI.

## Context
- **Starting state**: NAMD config uses hardcoded `NAMDConfig` struct with fixed fields (temperature, timestep, etc.). File types are auto-detected. ConfigurationTab.svelte has hardcoded form fields. Demo mode has hardcoded test data.
- **Delivered state**: Templates stored in database define both NAMD config structure and form UI. Jobs reference template + values. Users can manage templates via Templates page. File uploads are form fields assigned to template variables.
- **Foundation**: SQLite database, IPC command pattern, dynamic UI rendering (ClusterConfig pattern)
- **Dependencies**: Phase 6.7 completed (cellBasisVector, extrabonds support)
- **Testing approach**: Unit tests for template rendering, validation logic, database operations. No tests for framework code or external libraries.

## Why We're Doing This

### The Problem with Current Approach

**Hardcoded Assumptions Everywhere**:
- `NAMDConfig` struct has fixed fields - can't support new simulation types without Rust changes
- File type auto-detection makes assumptions about file roles
- Demo mode expects specific field names
- ConfigurationTab.svelte hardcodes form fields
- Adding parameters requires changes across: Rust struct, frontend form, validation, script generation

**Scalability Issues**:
- Different simulation types (vacuum vs explicit solvent vs implicit solvent) need completely different parameter sets
- Can't support user-defined workflows
- Every new parameter = code changes in multiple files
- No way for users to create templates for their specific use cases

**Tech Debt We're Avoiding**:
- We're NOT patching the current system with conditional logic
- We're NOT maintaining backwards compatibility with existing jobs (will delete old DB)
- We're NOT keeping NAMDConfig + adding templates on top
- We're doing a clean refactor with the right architecture from the start

### The Solution: Templates as Data

**Core Principle**: Template = NAMD config text with `{{variables}}` + variable definitions (types, validation, labels)

**Benefits**:
- Backend doesn't need to know field names - just renders template with values
- Adding new simulation types = create template in UI, no code changes
- Users can create custom templates for their workflows
- File handling is explicit - user assigns files to variables
- Form is generated from template definition - one source of truth

## What Gets Deleted (No Mercy Refactor)

### Delete Entire Modules
- ✗ `src-tauri/src/demo/` - Demo mode entirely
- ✗ `src-tauri/src/types/namd.rs` or wherever `NAMDConfig` struct lives
- ✗ `src/lib/components/create-job/ConfigurationTab.svelte` - Hardcoded form
- ✗ Any file type detection utilities

### Delete from Existing Files
- ✗ All `NAMDConfig` struct references
- ✗ All `InputFile`/`OutputFile` tracking (files become template values)
- ✗ File type auto-detection logic
- ✗ Demo mode initialization
- ✗ Hardcoded validation for specific fields (temperature range, etc.)
- ✗ Script generator code that expects specific NAMDConfig fields

### Database Changes
- ✗ Remove NAMDConfig fields from jobs table
- ✗ No migration needed - will delete old database entirely
- ✗ No version detection code - fresh start

## No Migration, No Backwards Compatibility

**Critical Understanding**: We are NOT maintaining backwards compatibility. Here's why and what this means:

**Why No Backwards Compatibility**:
- App has not been released to users
- All existing jobs are test data with no value
- Clean refactor is better than migration code that runs once then becomes dead weight
- Avoiding tech debt from day one

**What This Means for Implementation**:
- ✗ **DO NOT** write database migration code
- ✗ **DO NOT** write code to detect database versions
- ✗ **DO NOT** write code to convert old jobs to new format
- ✗ **DO NOT** keep old NAMDConfig fields "just in case"
- ✓ **DO** delete old database before running new app (manual step, document in progress log)
- ✓ **DO** create fresh schema with templates table
- ✓ **DO** assume all jobs use template system from day one

**Implementation Impact**:
- Database initialization just creates tables, no migration logic
- JobInfo struct has only new fields (template_id, template_values), no legacy fields
- No dual code paths ("if old job use NAMDConfig, if new job use template")
- Simpler, cleaner codebase from the start

## Implementation Plan

**Important**: Everything below must be completed. There are no optional items. The steps are ordered to minimize rework - follow this sequence.

### Step 1: Database Foundation

- [ ] **Template Database Schema & Storage**
  - [ ] Create `templates` table with: id (TEXT PRIMARY KEY), name (TEXT), description (TEXT), namd_config_template (TEXT), variables (TEXT as JSON), created_at (TEXT), updated_at (TEXT)
  - [ ] Modify `jobs` table: add `template_id` TEXT, `template_values` TEXT (as JSON), REMOVE all NAMDConfig fields
  - [ ] Create `src-tauri/templates/` folder for default template JSON files
  - [ ] Implement DB initialization: on first run, load templates from `src-tauri/templates/*.json` into DB
  - [ ] Write template CRUD functions: create_template(), get_template(), list_templates(), update_template(), delete_template()
  - [ ] **CRITICAL**: No migration logic, no version detection, no backwards compatibility code

### Step 2: Data Structures & Types

- [ ] **Template Data Structures**
  - [ ] Define `Template` struct: id, name, description, namd_config_template (String), variables (HashMap<String, VariableDefinition>), created_at, updated_at
  - [ ] Define `VariableDefinition` struct: key, label, var_type (VariableType enum), required (bool), help_text (Option<String>)
  - [ ] Define `VariableType` enum: Number{min: Option<f64>, max: Option<f64>, default: Option<f64>}, Text{default: Option<String>}, Boolean{default: bool}, FileUpload{extensions: Vec<String>}
  - [ ] No conditional visibility, no required_if, no custom validators - keep it simple

### Step 3: Template Rendering Engine

- [ ] **Template Renderer**
  - [ ] Implement `render_template(template: &Template, values: &HashMap<String, Value>) -> Result<String>`
  - [ ] Use Handlebars or simple string replacement for `{{variable}}` substitution (decide based on whether we need `{{#if}}` logic)
  - [ ] Handle file paths: if variable is FileUpload type, prepend "input_files/" to filename when rendering
  - [ ] Handle type conversion: Boolean → "yes"/"no", Number → string with proper formatting, Text as-is
  - [ ] Error if template has unreplaced variables after rendering (validation failure)

### Step 4: JobInfo Refactor (The Big Delete)

- [ ] **JobInfo Refactor**
  - [ ] Add fields to JobInfo struct: `template_id: String`, `template_values: HashMap<String, Value>`
  - [ ] Remove from JobInfo: all NAMDConfig fields, input_files: Vec<InputFile>, output_files: Vec<OutputFile>
  - [ ] **CRITICAL DELETE**: Remove NAMDConfig struct entirely from codebase
  - [ ] **CRITICAL DELETE**: Remove demo mode module entirely (`src-tauri/src/demo/`)
  - [ ] Update job creation automation to use template rendering instead of hardcoded config generation
  - [ ] Update script generator: remove NAMDConfig parameter, accept rendered config string directly

### Step 5: Default Templates & IPC

- [ ] **Default Templates**
  - [ ] Create `vacuum_optimization_v1.json` based on `examples/origamiTutorial/step2/hextube.namd`
    - Variables: structure_file (PSF), coordinates_file (PDB), parameters_file (PRM), extrabonds_file (EXB optional), temperature, timestep, steps, langevin_damping, cell_x/y/z, margin
    - NAMD template: Full config text with PME=no, large box (1000Å), low damping (0.1)
  - [ ] Create `explicit_solvent_npt_v1.json` based on `examples/origamiTutorial/step3/equil_min.namd`
    - Variables: structure_file, coordinates_file, parameters_file, extrabonds_file (optional), temperature, timestep, steps, cell_x/y/z, pme_grid_spacing, langevin_damping, langevin_piston_target
    - NAMD template: Full config text with PME=yes, NPT ensemble, system-specific box
  - [ ] Store both JSON files in `src-tauri/templates/` folder
  - [ ] Verify DB initialization loads them on first run (test with fresh DB)

- [ ] **Template IPC Commands**
  - [ ] `list_templates() -> Vec<TemplateSummary>` - Return id, name, description for template list UI
  - [ ] `get_template(id: String) -> Template` - Full template definition for rendering dynamic form
  - [ ] `create_template(template: Template) -> Result<()>` - Create new user template in DB
  - [ ] `update_template(id: String, template: Template) -> Result<()>` - Edit existing template
  - [ ] `delete_template(id: String) -> Result<()>` - Remove template (error if jobs exist using it)
  - [ ] `validate_template_values(template_id: String, values: HashMap<String, Value>) -> ValidationResult` - Check all required vars present, correct types

### Step 6: Template Management UI

- [ ] **Templates Page (New Sidebar Section)**
  - [ ] Add "Templates" to sidebar navigation (between Jobs and Settings)
  - [ ] Create `TemplatesPage.svelte`: List all templates with Create/Edit/Delete buttons
  - [ ] Show built-in templates separately from user-created (visual separation)
  - [ ] Implement delete confirmation dialog (warn if jobs exist using template, show count)
  - [ ] "Duplicate" button to copy template as starting point for customization

- [ ] **Template Editor UI**
  - [ ] Create `TemplateEditor.svelte`: Form for template metadata (name, description fields)
  - [ ] Textarea for NAMD config template - plain text is fine, syntax highlighting optional
  - [ ] List of variables with Add/Edit/Delete controls for each variable
  - [ ] Create `VariableEditor.svelte`: Form to edit single variable (key, label, type selector, required checkbox, help_text, type-specific options like min/max for numbers)
  - [ ] "Test Template" button: Render preview with sample values to verify template syntax
  - [ ] Save button creates/updates template in DB via IPC

- [ ] **Dynamic Job Creation Form**
  - [ ] Create `DynamicJobForm.svelte`: This completely replaces ConfigurationTab.svelte
  - [ ] Template selector dropdown at top (loads from list_templates IPC)
  - [ ] When template selected, fetch full definition via get_template IPC and render form
  - [ ] Generate form sections from template variables: Files section (all FileUpload vars), Parameters section (Number/Text/Boolean vars)
  - [ ] FileUpload variables → `<input type="file">` with accept attribute from extensions
  - [ ] Number variables → `<input type="number" min={} max={}>` with unit label display
  - [ ] Boolean variables → `<input type="checkbox">`
  - [ ] Text variables → `<input type="text">`
  - [ ] No conditional visibility logic - all variables for selected template are always shown
  - [ ] Submit: call validate_template_values IPC, then create_job with template_id + template_values

### Step 7: Validation & Integration

- [ ] **Validation Implementation**
  - [ ] Required field checking: Error if required variable missing from values
  - [ ] Type checking: Number is actually numeric, FileUpload has file selected, etc.
  - [ ] Range validation: Number within min/max if specified in variable definition
  - [ ] File extension validation: FileUpload filename matches accepted extensions
  - [ ] Show validation errors in UI form (red borders on invalid fields, error messages below)
  - [ ] Backend validation is authoritative - frontend validation is for UX only

- [ ] **Job Creation Integration**
  - [ ] Update job creation automation to use new template system
  - [ ] File upload: Store uploaded files in job's `input_files/` directory, record filename in template_values
  - [ ] Generate config.namd by calling render_template with template + values
  - [ ] Generate job.sbatch with rendered config (script generator receives rendered string)
  - [ ] Store job in DB with template_id + template_values (as JSON)
  - [ ] Remove old InputFile/OutputFile metadata tracking code - files are now just template values

### Step 8: Error Handling & Testing

- [ ] **Error Handling**
  - [ ] Template not found error → Clear error message with template ID
  - [ ] Missing required variables → List all missing variable names in error
  - [ ] Invalid template syntax (unreplaced {{variables}}) → Error before job creation with variable list
  - [ ] File upload failures → Specific error message per file (name + reason)
  - [ ] Template deletion blocked if jobs exist → Show count of dependent jobs, prevent deletion

- [ ] **End-to-End Verification**
  - [ ] Delete old database (manual step - document in progress log)
  - [ ] Start app, verify templates table created and default templates loaded
  - [ ] Create job using "Explicit Solvent NPT" template
  - [ ] Upload files via form fields (PSF, PDB, PRM, EXB)
  - [ ] Fill parameter fields (temperature, timestep, cell dimensions, etc.)
  - [ ] Submit job and verify config.namd generated correctly with all variables replaced
  - [ ] Verify job submits to SLURM and runs (if cluster available)
  - [ ] Create custom template via Templates page UI
  - [ ] Create job using custom template and verify it works
  - [ ] Edit template via UI and verify changes immediately reflect in job creation form

## Success Criteria

### Functional Success
- [ ] User can create job using "Explicit Solvent NPT" template with all files as form fields
- [ ] User can create custom template via Templates page UI
- [ ] User can edit existing template and changes reflect in job creation form
- [ ] Job creation generates correct config.namd from template + values
- [ ] Template system works end-to-end: create template → create job → submit → runs on cluster

### Technical Success
- [ ] No hardcoded NAMDConfig references remain in codebase
- [ ] Template renderer handles all variable types correctly (files, numbers, bools, text)
- [ ] Database initialization loads default templates on fresh install
- [ ] Validation prevents invalid jobs (missing files, out-of-range values)
- [ ] All template operations (CRUD) work via IPC commands

### Quality Success
- [ ] Unit tests for template renderer (variable substitution, file path handling, type conversion)
- [ ] Unit tests for validation logic (required fields, ranges, types)
- [ ] Database template operations tested
- [ ] Code follows NAMDRunner testing philosophy (business logic only, no framework tests)
- [ ] No demo mode, no file auto-detection, no backwards compatibility code

## Key Technical Decisions

### Why Template-as-Data (Not Rust Structs)
- **Reasoning**: Templates stored as data (DB + JSON) allow runtime modification without code changes. Users can create templates via UI. Backend is generic renderer, not field-aware.
- **Alternatives considered**: Keep NAMDConfig struct, add template layer on top → Rejected because creates dual system with tech debt
- **Trade-offs**: Lose compile-time type safety for template variables, gain runtime flexibility and user extensibility

### Why Files as Form Fields (Not Separate Upload Tab)
- **Reasoning**: Cleaner UX - file upload is just another form input. Explicit assignment (variable name = role) instead of auto-detection. One form, all inputs.
- **Alternatives considered**: Separate upload area + dropdown assignment → Rejected as extra step, harder to understand
- **Trade-offs**: Rethink tabs in job creation UI, but simpler mental model for users

### Why Database Storage (Not JSON Files Only)
- **Reasoning**: DB allows user-created templates, modification via UI, querying (e.g., "find templates used by jobs"). Built-in templates seeded from JSON on first run.
- **Alternatives considered**: JSON files only → Rejected because can't support user templates without file system writes
- **Trade-offs**: DB adds complexity, but necessary for user template management feature

### Why No Conditional Visibility
- **Reasoning**: Different simulation needs = different templates, not conditional logic in one template. Keeps template system simple. Vacuum opt template vs explicit solvent template instead of one template with if/else.
- **Alternatives considered**: `visible_when` field for conditional fields → Rejected as unnecessary complexity
- **Trade-offs**: More templates (2-3x), but each template is simpler and clearer

### Why No Backwards Compatibility
- **Reasoning**: App not released, no production users. Clean refactor better than migration code that runs once then becomes dead weight.
- **Alternatives considered**: Migrate old jobs to template system → Rejected because test data has no value
- **Trade-offs**: Lose test jobs (acceptable), avoid migration code forever (huge win)

### Why Handlebars/String Replacement (Not Code Generation)
- **Reasoning**: Templates are text with holes. Simple string replacement or lightweight templating (Handlebars) is sufficient. No need for complex code generation.
- **Alternatives considered**: Generate Rust code from template → Rejected as overengineered
- **Trade-offs**: Less type safety, but matches mental model (template = text file)

## Integration with Existing Code

### Leverage Existing Patterns
- **Use ClusterConfig pattern**: Template store follows same pattern as clusterConfig.ts (writable store, derived stores, initialize on app start)
- **Follow IPC command conventions**: Template commands match job commands structure (list, get, create, update, delete)
- **Apply validation patterns**: Reuse validation approach from job creation (frontend for UX, backend authoritative)
- **Use automation system**: Template rendering integrated into existing job creation automation chain

### Where to Hook In

**Database** (`src-tauri/src/db.rs`):
```rust
// ADD: Template CRUD functions
fn create_template(template: &Template) -> Result<()>
fn get_template(id: &str) -> Result<Template>
fn list_templates() -> Result<Vec<Template>>
fn update_template(id: &str, template: &Template) -> Result<()>
fn delete_template(id: &str) -> Result<()>

// MODIFY: Database initialization
fn initialize_database() {
    // Existing: Create tables
    // ADD: Load templates from src-tauri/templates/*.json
}
```

**Job Creation** (`src-tauri/src/automations/job_creation.rs`):
```rust
// MODIFY: Replace NAMDConfig generation
pub async fn execute_job_creation_with_progress(...) {
    // OLD: Generate config from hardcoded NAMDConfig struct
    // NEW: Render template with template_values
    let template = get_template(&params.template_id)?;
    let config_content = render_template(&template, &params.template_values)?;
}
```

**Script Generator** (`src-tauri/src/slurm/script_generator.rs`):
```rust
// MODIFY: Remove NAMDConfig references
// OLD: fn generate_namd_config(namd_config: &NAMDConfig) -> String
// NEW: Receive rendered config string directly, no generation needed
// Job creation automation already rendered it via template
```

**Frontend Job Creation** (`src/lib/components/pages/CreateJobPage.svelte`):
```svelte
<!-- REPLACE: ConfigurationTab with DynamicJobForm -->
<!-- OLD: <ConfigurationTab bind:namdConfig /> -->
<!-- NEW: <DynamicJobForm bind:templateId bind:templateValues /> -->
```

### New Files to Create

**Backend**:
- `src-tauri/src/templates/mod.rs` - Template module exports
- `src-tauri/src/templates/types.rs` - Template, VariableDefinition structs
- `src-tauri/src/templates/renderer.rs` - Template rendering logic
- `src-tauri/src/templates/validation.rs` - Template value validation
- `src-tauri/src/commands/templates.rs` - IPC commands
- `src-tauri/templates/vacuum_optimization_v1.json` - Default template
- `src-tauri/templates/explicit_solvent_npt_v1.json` - Default template

**Frontend**:
- `src/lib/components/pages/TemplatesPage.svelte` - Template list UI
- `src/lib/components/templates/TemplateEditor.svelte` - Edit template
- `src/lib/components/templates/VariableEditor.svelte` - Edit variable definition
- `src/lib/components/create-job/DynamicJobForm.svelte` - Dynamic form renderer
- `src/lib/stores/templateStore.ts` - Template state management
- `src/lib/types/template.ts` - TypeScript types for templates

## Template JSON Format

Example structure for default templates:

```json
{
  "id": "explicit_solvent_npt_v1",
  "name": "Explicit Solvent NPT",
  "description": "Constant pressure/temperature simulation in explicit solvent with PME electrostatics",
  "namd_config_template": "# NAMD Configuration\nstructure          input_files/{{structure_file}}\ncoordinates        input_files/{{coordinates_file}}\nparameters         input_files/{{parameters_file}}\n\ntemperature        {{temperature}}\ntimestep           {{timestep}}\n\nPME                yes\nPMEGridSpacing     {{pme_grid_spacing}}\ncellBasisVector1   {{cell_x}} 0 0\ncellBasisVector2   0 {{cell_y}} 0\ncellBasisVector3   0 0 {{cell_z}}\n\nlangevin           on\nlangevinTemp       {{temperature}}\nlangevinDamping    {{langevin_damping}}\n\nlangevinPiston     on\nlangevinPistonTarget {{langevin_piston_target}}\n\n{{#if extrabonds_file}}\nextraBondsFile     input_files/{{extrabonds_file}}\n{{/if}}\n\nrun                {{steps}}\n",
  "variables": {
    "structure_file": {
      "label": "Structure File (PSF)",
      "var_type": {
        "FileUpload": {
          "extensions": [".psf"]
        }
      },
      "required": true,
      "help_text": "NAMD structure file defining molecular topology"
    },
    "coordinates_file": {
      "label": "Coordinates (PDB)",
      "var_type": {
        "FileUpload": {
          "extensions": [".pdb"]
        }
      },
      "required": true,
      "help_text": "Initial atomic coordinates"
    },
    "temperature": {
      "label": "Temperature (K)",
      "var_type": {
        "Number": {
          "min": 200.0,
          "max": 400.0,
          "default": 300.0
        }
      },
      "required": true,
      "help_text": "Simulation temperature in Kelvin"
    },
    "cell_x": {
      "label": "Cell X Dimension (Å)",
      "var_type": {
        "Number": {
          "min": 10.0,
          "max": 1000.0,
          "default": null
        }
      },
      "required": true,
      "help_text": "Periodic box X dimension from solvated system"
    }
  }
}
```

## References
- **NAMDRunner patterns**:
  - `docs/CONTRIBUTING.md#testing-strategy` - Testing philosophy
  - `docs/ARCHITECTURE.md` - System design patterns
  - `docs/DB.md` - Database schema conventions
- **Implementation files**:
  - `src-tauri/src/db.rs` - Database operations
  - `src-tauri/src/automations/job_creation.rs` - Job creation flow
  - `src/lib/stores/clusterConfig.ts` - Store pattern to follow
- **Tutorial files**:
  - `examples/origamiTutorial/step2/hextube.namd` - Vacuum template source
  - `examples/origamiTutorial/step3/equil_min.namd` - Explicit solvent template source
- **Planning docs**:
  - `tasks/planning/Templates.md` - Template system design (original plan, now superseded by this refactor)

## Progress Log
2025-11-01 - Task plan created based on template system refactor discussion. Decision: Clean refactor, delete NAMDConfig, files as form fields, templates in DB, no backwards compatibility.

## Completion Process
After implementation and testing:
- [ ] Run code review using `.claude/agents/review-refactor.md`
- [ ] Implement recommended refactoring improvements
- [ ] Update and archive task to `tasks/completed/phase-7-1-template-system-refactor.md`
- [ ] Update `tasks/roadmap.md` - Mark Phase 7.1 complete
- [ ] Update `docs/ARCHITECTURE.md` - Document template system architecture
- [ ] Update `docs/DB.md` - Document templates table schema
- [ ] Update `docs/API.md` - Document template IPC commands

## Open Questions
- [ ] Handlebars vs simple string replacement? (Handlebars if we need `{{#if}}`, simple replacement if not)
- [ ] Should built-in templates be editable or read-only? (Probably allow duplicate → edit, but not direct edit)
- [ ] Template export/import feature? (Not critical for v1, but nice to have for sharing)
- [ ] Validation error UX: Inline per field or summary at top? (Inline is standard, do that)
