# Task: Phase 6.7 - Template Type 2 NAMD Configuration Support

## Objective
Rebuild NAMD configuration generation from tutorial source of truth, fixing all critical bugs, eliminating code smells, and establishing clean architecture with backend-validated file types and extracted template constants.

## Context
- **Starting state**: Phase 6.6 complete - SLURM automation working, but NAMD config has critical bugs and code smells
- **Delivered state**: Users can run tutorial workflows (explicit solvent equilibration) with clean, maintainable code
- **Source of truth**: `examples/origamiTutorial/step3/equil_min.namd` - All parameter values come from here
- **Foundation**: Complete rewrite of script generation, not patches on existing messy code
- **Dependencies**: Phase 6.6 complete
- **Testing approach**: Manual cluster testing with actual tutorial files

## Critical Issues & Code Smells

### Issues Blocking Job Execution
1. **Missing cellBasisVector** - No periodic boundaries, ALL PME jobs fail
2. **Missing execution_mode** - Cannot run minimization (always "run", never "minimize")
3. **Wrong output frequencies** - Uses dcd_freq for xstFreq/outputEnergies/outputPressure

### Code Quality Issues
4. **Giant format!() template** - 100+ line string with positional `{}`, unmaintainable
5. **Magic numbers everywhere** - No constants, unclear meanings (what is "1.01325"?)
6. **Duplicated unwrap_or()** - Same `unwrap_or(9600)` repeated 5 times
7. **File type detection in frontend** - Validation logic doesn't belong in UI
8. **No extrabonds support** - Missing .exb/.enm.extra file type
9. **Hardcoded PME/NPT** - Cannot do vacuum or NVT simulations
10. **Unclear optional fields** - Why is `dcd_freq` Option if we always unwrap?

## Implementation Plan

### Step 1: Establish Backend File Type Validation

**Move all file type detection logic to backend where it belongs**

#### Task 1.1: Backend File Type Detection

**File**: `src-tauri/src/types/core.rs`

Add Exb variant and detection logic:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NAMDFileType {
    #[serde(rename = "pdb")]
    Pdb,
    #[serde(rename = "psf")]
    Psf,
    #[serde(rename = "prm")]
    Prm,
    #[serde(rename = "exb")]
    Exb,
    #[serde(rename = "other")]
    Other,
}

impl NAMDFileType {
    /// Detect file type from filename (source of truth for type detection)
    pub fn from_filename(filename: &str) -> Self {
        let lower = filename.to_lowercase();
        let ext = lower.split('.').last().unwrap_or("");

        match ext {
            "pdb" => Self::Pdb,
            "psf" => Self::Psf,
            "prm" => Self::Prm,
            "exb" => Self::Exb,
            _ if lower.ends_with(".enm.extra") => Self::Exb,
            _ if lower.contains("extrabonds") => Self::Exb,
            _ => Self::Other,
        }
    }
}
```

**Checklist**:
- [ ] Add Exb variant to enum
- [ ] Implement from_filename() method
- [ ] Handle .exb, .enm.extra, and "extrabonds" in filename
- [ ] Add unit tests for detection logic

#### Task 1.2: Backend File Type Detection Command

**File**: `src-tauri/src/commands/files.rs`

Add new command:
```rust
#[tauri::command]
pub fn detect_file_type(filename: String) -> Result<NAMDFileType, String> {
    Ok(NAMDFileType::from_filename(&filename))
}
```

**File**: `src-tauri/src/lib.rs`

Register command:
```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands
    commands::files::detect_file_type,
])
```

**Checklist**:
- [ ] Add detect_file_type command
- [ ] Register in invoke_handler
- [ ] Test command returns correct types

#### Task 1.3: Frontend Calls Backend for Detection

**File**: `src/lib/types/api.ts`

Update InputFile type:
```typescript
export interface InputFile {
  name: string;
  local_path: string;
  remote_name?: string;
  file_type?: 'pdb' | 'psf' | 'prm' | 'exb' | 'other';  // Add 'exb'
  size?: number;
  uploaded_at?: string;
}
```

**File**: `src/lib/components/pages/CreateJobPage.svelte`

Remove frontend detection, call backend:
```typescript
// DELETE detectFileType() function entirely

// REPLACE file selection handler:
async function handleFileSelection(selectedFiles: SelectedFile[]) {
  const filesWithTypes = await Promise.all(
    selectedFiles.map(async (file) => ({
      name: file.name,
      local_path: file.path,
      size: file.size,
      file_type: await invoke<'pdb' | 'psf' | 'prm' | 'exb' | 'other'>(
        'detect_file_type',
        { filename: file.name }
      ),
    }))
  );

  uploadedFiles = [...uploadedFiles, ...filesWithTypes];
}
```

**Checklist**:
- [ ] Update InputFile type with 'exb'
- [ ] Delete frontend detectFileType() function
- [ ] Call backend detect_file_type for each file
- [ ] Test file type detection works end-to-end

#### Task 1.4: Update FilesTab Accepted Extensions

**File**: `src/lib/components/create-job/FilesTab.svelte`

```svelte
const acceptedExtensions = [".pdb", ".psf", ".prm", ".exb"];
```

**Checklist**:
- [ ] Add .exb to accepted extensions

---

### Step 2: Clean Up NAMDConfig Structure

**Remove unclear optionals, establish clear defaults**

#### Task 2.1: Redesign NAMDConfig with Clear Semantics

**File**: `src-tauri/src/types/core.rs`

Replace existing NAMDConfig:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NAMDConfig {
    // Basic simulation parameters (always required)
    pub outputname: String,
    pub steps: u32,
    pub temperature: f64,
    pub timestep: f64,

    // Execution mode
    pub execution_mode: ExecutionMode,

    // Output frequencies (if None, use tutorial defaults)
    pub output_freq: Option<u32>,    // For DCD, XST, energies, pressure (default: 1200)
    pub restart_freq: Option<u32>,   // For restart checkpoints (default: 1200)

    // Periodic boundaries (required when use_pme=true)
    pub cell_basis_x: Option<f64>,
    pub cell_basis_y: Option<f64>,
    pub cell_basis_z: Option<f64>,

    // Physics simulation type
    pub use_pme: bool,               // Particle Mesh Ewald (default: true)
    pub use_npt: bool,               // NPT ensemble (default: true)

    // Advanced parameters (if None, use tutorial defaults)
    pub langevin_damping: Option<f64>,      // Default: 5.0
    pub margin: Option<f64>,                // Only for vacuum (PME=false)
    pub full_elect_frequency: Option<u32>,  // Default: 2
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionMode {
    Minimize,
    Run,
}
```

**Checklist**:
- [ ] Replace old NAMDConfig with new structure
- [ ] Add ExecutionMode enum
- [ ] Clear semantics: None = use tutorial default
- [ ] Remove old dcd_freq field (replaced by output_freq)

#### Task 2.2: Frontend Type Updates

**File**: `src/lib/types/api.ts`

```typescript
export type ExecutionMode = 'minimize' | 'run';

export interface NAMDConfig {
  // Basic simulation parameters
  outputname: string;
  steps: number;
  temperature: number;
  timestep: number;

  // Execution mode
  execution_mode: ExecutionMode;

  // Output frequencies
  output_freq?: number;
  restart_freq?: number;

  // Periodic boundaries
  cell_basis_x?: number;
  cell_basis_y?: number;
  cell_basis_z?: number;

  // Physics simulation type
  use_pme: boolean;
  use_npt: boolean;

  // Advanced parameters
  langevin_damping?: number;
  margin?: number;
  full_elect_frequency?: number;
}
```

**Checklist**:
- [ ] Add ExecutionMode type
- [ ] Update NAMDConfig interface
- [ ] Remove old dcd_freq field

---

### Step 3: Rebuild Script Generator From Tutorial

**Complete rewrite: modular, constants extracted, tutorial as source of truth**

See full implementation in separate document due to length. Key changes:

**File**: `src-tauri/src/slurm/script_generator.rs`

Restructure:
1. Extract `namd_constants` module with all tutorial values
2. Replace giant format!() with modular section builders
3. Implement dedicated builder functions:
   - `build_header_section()`
   - `build_input_section()`
   - `build_temperature_section()`
   - `build_cell_basis_section()`
   - `build_parameters_section()`
   - `build_force_field_section()`
   - `build_integrator_section()`
   - `build_pme_section()`
   - `build_langevin_section()`
   - `build_npt_section()`
   - `build_output_section()`
   - `build_extrabonds_section()`
   - `build_execution_section()`
4. Add validation function
5. Compose sections at end

**Constants from tutorial (lines documented)**:
```rust
mod namd_constants {
    // Output frequencies (tutorial line 89-93)
    pub const DEFAULT_OUTPUT_FREQ: u32 = 1200;
    pub const DEFAULT_RESTART_FREQ: u32 = 1200;

    // Langevin dynamics (tutorial line 76)
    pub const DEFAULT_LANGEVIN_DAMPING: f64 = 5.0;

    // PME parameters (tutorial line 68)
    pub const PME_GRID_SPACING: f64 = 1.5;

    // NPT ensemble (tutorial lines 83-85)
    pub const PRESSURE_TARGET: f64 = 1.01325;  // bar = 1 atm
    pub const PISTON_PERIOD: u32 = 1000;       // fs
    pub const PISTON_DECAY: u32 = 500;         // fs

    // Force field parameters (tutorial lines 51-56)
    pub const EXCLUDE_RULE: &str = "scaled1-4";
    pub const SCALING_1_4: f64 = 1.0;
    pub const SWITCH_DIST: f64 = 8.0;
    pub const CUTOFF: f64 = 10.0;
    pub const PAIRLIST_DIST: f64 = 12.0;

    // Integration parameters (tutorial lines 61-63)
    pub const NONBONDED_FREQ: u32 = 1;
    pub const STEPS_PER_CYCLE: u32 = 12;
    pub const DEFAULT_FULL_ELECT_FREQ: u32 = 2;
}
```

**Validation**:
```rust
fn validate_namd_config(config: &NAMDConfig) -> Result<()> {
    if config.use_pme {
        if config.cell_basis_x.is_none() ||
           config.cell_basis_y.is_none() ||
           config.cell_basis_z.is_none() {
            return Err(anyhow!("PME requires cell dimensions"));
        }
    }

    if config.use_pme && config.margin.is_some() {
        return Err(anyhow!("Margin only valid for vacuum (PME=false)"));
    }

    Ok(())
}
```

**Checklist**:
- [ ] Extract namd_constants module
- [ ] Implement modular section builders
- [ ] Replace giant format!() with composition
- [ ] Add validation function
- [ ] Use constants instead of magic numbers
- [ ] Test each section builder independently

---

### Step 4: Update Frontend UI

**Add all required controls with clear labels**

**File**: `src/lib/components/create-job/ConfigurationTab.svelte`

Sections:
1. Basic Configuration (name, output, mode, steps, temp, timestep)
2. Simulation Type (PME checkbox, NPT checkbox)
3. Periodic Cell Dimensions (conditional on PME, 3 inputs)
4. Output & Advanced Parameters (frequencies, damping, margin, full_elect_freq)

**Checklist**:
- [ ] Add execution mode dropdown
- [ ] Add PME/NPT checkboxes with defaults
- [ ] Add conditional cell dimensions section
- [ ] Rename dcd_freq to output_freq
- [ ] Add all advanced parameters with placeholders
- [ ] Show margin only when PME disabled
- [ ] Add helpful field descriptions

---

## Success Criteria

### Functional Success
- [ ] User uploads tutorial files, generates valid NAMD config
- [ ] Config uses actual uploaded filenames (no hardcoding)
- [ ] All tutorial parameter values match source of truth
- [ ] PME jobs work with cell dimensions
- [ ] Minimize mode works for first stage
- [ ] Multiple extrabonds files load correctly
- [ ] Vacuum simulations work (PME=false)

### Code Quality Success
- [ ] No magic numbers - all extracted to constants
- [ ] No giant format!() - modular section builders
- [ ] No duplicate unwrap_or() - single variable per value
- [ ] File type detection in backend only
- [ ] Clear separation: validation in backend, display in frontend
- [ ] Tutorial as documented source of truth
- [ ] Each section builder testable independently

### Technical Success
- [ ] All unit tests passing
- [ ] Manual cluster test with tutorial files succeeds
- [ ] No code smells remain
- [ ] Clean, maintainable architecture

## Key Technical Decisions

### Backend Validation Only
All validation logic (file types, config validation) lives in backend. Frontend is pure display/input with no business logic.

### Tutorial as Source of Truth
All NAMD parameter values come from `examples/origamiTutorial/step3/equil_min.namd`. Constants documented with line references.

### Modular Template Generation
Each config section has dedicated builder function. Sections combined at end. Easy to test, modify, and understand.

### No Backwards Compatibility
Clean slate - old jobs can be deleted. No migration code, no serde tricks.

### No Enterprise Patterns
Direct implementation, no layers of abstraction, no "future-proofing" beyond good structure.

## References
- **Source of truth**: `examples/origamiTutorial/step3/equil_min.namd`
- **Current code**: `src-tauri/src/slurm/script_generator.rs` (complete rewrite)
- **Types**: `src-tauri/src/types/core.rs` (NAMDConfig, NAMDFileType)
- **UI**: `src/lib/components/create-job/ConfigurationTab.svelte`

## Progress Log
[Date] - Phase 6.7 plan created with integrated code smell fixes and complete rewrite approach

## Completion Process
After implementation and testing:
- [ ] Test with tutorial files from `examples/origamiTutorial/step3/`
- [ ] Submit job to cluster, verify successful execution
- [ ] Run code review with `.claude/agents/review-refactor.md`
- [ ] Archive to `tasks/completed/phase-6-7-template-type-2-namd-config-fixes.md`
- [ ] Update `tasks/roadmap.md` to mark Phase 6.7 complete
