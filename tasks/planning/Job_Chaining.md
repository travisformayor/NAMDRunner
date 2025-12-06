# NAMD Job Chains - Design Document

> **Status**: Design Phase - Not Yet Implemented
> **Target Phase**: Phase 7+
> **Last Updated**: 2025-12-05

## Table of Contents

### Part 1: Background and Core Design
1. [Executive Summary](#1-executive-summary)
2. [Background: Understanding NAMD Workflows](#2-background-understanding-namd-workflows)
3. [Core Design Decisions](#3-core-design-decisions)
4. [Integration with Template System](#4-integration-with-template-system)
5. [Data Model Design](#5-data-model-design)
6. [File Handling Architecture](#6-file-handling-architecture)
7. [UI/UX Design](#7-uiux-design)

### Part 2: Implementation Details
8. [Backend Implementation Strategy](#8-backend-implementation-strategy)
9. [Tutorial Reference Guide](#9-tutorial-reference-guide)
10. [Open Questions & Future Decisions](#10-open-questions--future-decisions)
11. [Integration with Existing Architecture](#11-integration-with-existing-architecture)
12. [Success Criteria](#12-success-criteria)
13. [Appendices](#13-appendices)

---

## 1. Executive Summary

### What This Document Covers

This document defines the design for **Job Chains** and how they enable multi-stage molecular dynamics workflows in NAMDRunner.

> **Note**: The **Template System** has been implemented. For details on templates, see the codebase at `src-tauri/src/templates/`. This document focuses on job chaining mechanisms and how they interact with the existing template system.

### Key Decisions Made

1. **Job Chains as Unified Mechanism**: "Restart after timeout" and "next equilibration stage" are the same operation - creating a new job that continues from a parent job's outputs.
2. **Jobs as Self-Contained Islands**: Each job in a chain copies all necessary files from its parent, ensuring resilience to deletion and scratch purges.
3. **Two Continuation Types**: Checkpoint restart (same simulation) vs next stage (new simulation phase).

### Prerequisites for Implementation

**Must complete first**:
- Review existing codebase patterns for dynamic configuration (clusterConfig system)
- Understand current file operations and SFTP capabilities

**Related Issues**:
- Issue #9: Restart capability (multi-stage workflows)
- Issue #12: Multi-stage workflow templates

---

## 2. Background: Understanding NAMD Workflows

### 2.1 Tutorial Analysis

**Primary Resource**: `examples/origamiTutorial/origamiprotocols_0_markdown.md`

**Critical Observations**:
- Line 49 of `step2/hextube.namd`: `PME no` (vacuum simulation)
- Line 66 of `step3/equil_min.namd`: `PME yes` (periodic boundaries required)
- Lines 17-39 of `step3/equil_k0.5.namd`: Restart mechanism with timestep recovery

### 2.2 How NAMD Simulations Actually Work

#### Multi-Stage Equilibration Workflows (DNA Origami Example)

**Tutorial Pattern**:
```
Stage 1: Minimization (4800 steps)
  └─> outputs: equil_min.coor, equil_min.vel, equil_min.xsc

Stage 2: Strong restraints k=0.5 (4.8 ns)
  └─> reads: equil_min.restart.coor/vel/xsc
  └─> outputs: equil_k0.5.restart.coor/vel/xsc

Stage 3: Medium restraints k=0.1 (4.8 ns)
  └─> reads: equil_k0.5.restart.coor/vel/xsc
  └─> outputs: equil_k0.1.restart.coor/vel/xsc

... (continues through k=0.01 and k=0)
```

**Key Insight**: Each stage is a separate NAMD run that continues from previous checkpoint files.

#### Restart/Checkpoint Mechanisms

**Critical NAMD Configuration** (from tutorial `step3/equil_k0.5.namd`):

```tcl
# Restart from previous stage
set input          equil_min
bincoordinates     $input.coor
binvelocities      $input.vel
extendedSystem     $input.xsc

# CRITICAL: Do NOT set temperature when restarting
# Velocities come from restart file
set temperature    300
#temperature         $temperature  <- COMMENTED OUT!

# Get correct timestep from restart file
proc get_first_ts { xscfile } {
  set fd [open $xscfile r]
  gets $fd
  gets $fd
  gets $fd line
  set ts [lindex $line 0]
  close $fd
  return $ts
}
set firsttime [get_first_ts $input.restart.xsc]
firsttimestep $firsttime
```

**Critical Rules**:
1. Load binary coordinates, velocities, extended system from previous stage
2. **Never** set `temperature` directive when restarting (velocities already initialized)
3. Read correct timestep from `.xsc` file to maintain continuous time series
4. Restart files use `.restart.{coor,vel,xsc}` naming convention

### 2.3 The Realization: Job Chains

#### The Insight (Correct)

**They are the same mechanism!** All of these scenarios are identical:

**Scenario A: Timeout Recovery**
```
Job: Equilibration k=0.5 (24 cores, 4 hours, TIMEOUT)
  └─> Continue: Equilibration k=0.5 (48 cores, 8 hours, COMPLETED)
```

**Scenario C: Next Equilibration Stage**
```
Job: Equilibration k=0.5 (COMPLETED)
  └─> Continue: Equilibration k=0.1 (new restraints, COMPLETED)
```

**Common Pattern**: Take Job N's outputs → Use as Job N+1's inputs → Submit new SLURM job

#### Why Jobs Must Be Self-Contained

**Scratch Storage Reality**:
- `/scratch/alpine/` auto-purges after 90 days
- Not guaranteed to survive job completion

**Therefore**: Each job in a chain must be an **independent island** with all necessary files copied locally.

#### The Unified Job Chain Concept

**Definition**: A job chain is a sequence of jobs where each child job continues from its parent's outputs.

**Properties**:
1. Each link in chain is a full NAMDRunner job (own ID, metadata, files)
2. Child job copies parent's outputs to its own input_files/
3. Parent-child relationship tracked via `parent_job_id` field
4. Chain can be interrupted (parent deletion doesn't break child)
5. Same mechanism for all continuation types (timeout, next stage, retry with changes)

---

## 3. Core Design Decisions

### 3.1 Jobs as Self-Contained Islands

#### Decision

**Every job in a chain copies all necessary files from its parent**, rather than referencing files in parent's directory.

#### Files Copied During Continuation

**From Parent to Child**:
1. **Original input files**: `.pdb`, `.psf`, `.prm` files (needed for NAMD config structure/parameters references)
2. **Restart checkpoint files**: `.restart.{coor,vel,xsc}` (for checkpoint restart)
3. **Final output coordinates**: Final `.coor/.vel/.xsc` (for next stage continuation)
4. **Extrabonds files**: `.exb` files if present (DNA restraints)

**Child Job Directory After Copy**:
```
/projects/$USER/namdrunner_jobs/job_child_id/
├── input_files/
│   ├── structure.pdb              # COPIED from parent input_files/ (unchanged)
│   ├── structure.psf              # COPIED from parent input_files/ (unchanged)
│   ├── parameters.prm             # COPIED from parent input_files/ (unchanged)
│   ├── equil_k0.5.restart.coor    # COPIED from parent outputs/
│   ├── equil_k0.5.restart.vel     # COPIED from parent outputs/
│   └── equil_k0.5.restart.xsc     # COPIED from parent outputs/
├── scripts/
│   ├── config.namd                # NEW - references local input_files/
│   └── job.sbatch                 # NEW
└── outputs/                       # Empty, will be filled during execution
```

### 3.2 Job Chains Over In-Place Restart

#### Decision

We use **separate jobs** for each step in the chain, rather than modifying and restarting the same job in-place.

**✅ Advantages**:
1. **Clean separation**: Each job = one NAMD run = one SLURM submission
2. **Simple 1:1 mapping**: Easy to understand and debug
3. **No architecture changes**: Fits perfectly with existing JobInfo structure
4. **Flexible resources**: Stage 1 (8 cores, 2h), Stage 2 (32 cores, 12h) - can't do with in-place
5. **Partial execution**: Submit stage 1, wait, review, then decide to proceed
6. **Error recovery**: Stage 2 fails? Fix and resubmit just stage 2
7. **UI remains simple**: Job list is just a list, job detail is single job

### 3.3 Two Types of Continuation

#### Decision

Job continuation has **two distinct modes** that differ in which files they use and how configs are generated:

**1. Checkpoint Restart** (same simulation, continue from interruption)
- **Use case**: Job timed out, ran out of memory, node failed
- **Files needed**: `.restart.{coor,vel,xsc}` from parent
- **NAMD config**: Includes restart block with timestep recovery
- **Parameters**: Usually keep same (maybe adjust walltime/memory)
- **Scientific meaning**: Resume interrupted simulation

**2. Next Stage** (new simulation phase)
- **Use case**: Completed equilibration stage, starting next stage
- **Files needed**: Final `.{coor,vel,xsc}` (not restart files)
- **NAMD config**: Fresh start with new parameters (but reads previous final state)
- **Parameters**: Changed (restraint strength, output name, etc.)
- **Scientific meaning**: Progress to next phase of protocol

---

## 4. Integration with Template System

The Template System is already implemented in NAMDRunner (see `src-tauri/src/templates/`). Job Chains interact with templates in the following ways:

1.  **Template Inheritance**: When creating a child job, it typically defaults to using the same template as the parent job.
2.  **Parameter Persistence**: The values entered for the parent job (stored in `template_values`) are pre-filled for the child job.
3.  **Stage-Specific Variations**: For "Next Stage" continuation, the user can modify the template values (e.g., changing `extrabonds_file` from `k0.5.enm` to `k0.1.enm` or changing `run_type` from `minimize` to `run`).

**Job Chain with Templates**:
```
Job 1: Template=explicit_solvent_npt, template_values={mode: "minimize", ...}
  └─> Job 2: Same template, template_values={mode: "run", extrabonds: "k0.5.enm.extra", ...}
      └─> Job 3: Same template, template_values={extrabonds: "k0.1.enm.extra", ...}
```

---

## 5. Data Model Design

### 5.1 Extensions to JobInfo

**Required Extensions**:
```rust
pub struct JobInfo {
    // ... all existing fields unchanged ...

    // Template tracking
    pub template_id: String,

    // NEW: Job chain support
    pub parent_job_id: Option<String>,
    pub continuation_type: Option<ContinuationType>,

    // FUTURE: Chain metadata (optional, for UI convenience)
    pub root_job_id: Option<String>,     // Top of chain (for finding entire chain)
    pub chain_depth: u32,                // 0 = root, 1 = first child, etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContinuationType {
    #[serde(rename = "CHECKPOINT_RESTART")]
    CheckpointRestart,
    #[serde(rename = "NEXT_STAGE")]
    NextStage,
}
```

### 5.2 Continuation Metadata

**CreateJobParams Extension**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJobParams {
    pub job_name: String,
    pub template_id: String,
    pub namd_config: NAMDConfig,
    pub slurm_config: SlurmConfig,
    pub input_files: Vec<InputFile>,

    // NEW: Continuation support
    pub parent_job_id: Option<String>,
    pub continuation_type: Option<ContinuationType>,
    pub restart_output_prefix: Option<String>,  // e.g., "equil_k0.5", "output"
}
```

---

## 6. File Handling Architecture

### 6.1 What Files Get Copied

**During Job Continuation** (Parent → Child):

**Always Copied**:
1. **Original input files** from parent `input_files/`:
   - `.pdb` - Structure coordinates
   - `.psf` - Topology
   - `.prm` - Force field parameters
   - `.exb` - Extrabonds restraints (if present)

2. **Continuation files** from parent `outputs/` (depends on continuation type):
   - **Checkpoint Restart**: `{prefix}.restart.{coor,vel,xsc}`
   - **Next Stage**: `{prefix}.{coor,vel,xsc}` (final, not restart)

**Not Copied**:
- `.dcd` - Trajectory files (too large, not needed for continuation)
- `.log` - Log files (informational only)
- `.out/.err` - SLURM logs (already cached in database)

### 6.2 File Discovery Strategy

**Prefix-Based File Matching**:
- User provides output prefix from parent (e.g., "output", "equil_k0.5")
- System finds matching `.restart.{coor,vel,xsc}` or `.{coor,vel,xsc}`
- Validates all three files exist
- Copies to child input_files/

### 6.3 File Copying During Continuation

**Cluster-Side Copy (No Local Intermediary)**:
- Use `sync_directory_rsync()` for directories.
- Use `cp` command for individual files.
- No download to local machine.

---

## 7. UI/UX Design

### 7.1 Creating Fresh Jobs

Fresh jobs are created using the existing **Template System**. Users select a template (e.g., "Equilibration") and fill out the dynamic form. This is already covered by the Template System implementation.

### 7.2 Continuing Existing Jobs

**Entry Point**: Job detail page has "Continue Job" button

**Flow**:
1. **Click "Continue Job"** → Opens continuation dialog

2. **Select Continuation Type**:
   ```
   ○ Checkpoint Restart
     Continue interrupted simulation (timeout/OOM/failure)
     Keeps same parameters, may adjust resources

   ○ Next Stage
     Start next phase with modified parameters
     Change NAMD settings, different output name
   ```

3. **Template Selection** (for Next Stage only):
   - If checkpoint restart: Use parent's template (locked)
   - If next stage: Can change template (usually keeps same)
   - Shows parent's template as default

4. **Pre-filled Form**:
   - Job name: Auto-suggests `{parent_name}_cont` or `{parent_name}_k0.1`
   - Resources: Copied from parent (user can adjust)
   - NAMD params: Copied from parent (user can modify for next stage)
   - Files: Auto-selected from parent outputs (user sees preview)

5. **File Selection**:
   - Shows parent's output files
   - Auto-selects restart files based on continuation type
   - User can verify/adjust
   - Displays file sizes

6. **Create Job**:
   - Runs continuation automation
   - Copies files from parent
   - Creates new job record
   - Shows in job list as child of parent

### 7.3 Job Chain Visualization

**Job List Enhancement** (Tree View):
```
Jobs List:

  ☑ DNA_Minimization                    COMPLETED  2 days ago
    └─ ☑ DNA_k0.5                       TIMEOUT    2 days ago
       └─ ☑ DNA_k0.5_retry              COMPLETED  2 days ago
          └─ ☐ DNA_k0.1                 RUNNING    Now
```

**Job Detail Enhancement** (Chain Timeline):
```
Job Chain Timeline:

  DNA_Minimization ──┬── DNA_k0.5 ──┬── DNA_k0.5_retry ──┬── DNA_k0.1
  (COMPLETED)        │   (TIMEOUT)  │   (COMPLETED)       │   (RUNNING)
                     │              │                     │
                  Failed after 2h  Continued with 8h   Next stage k=0.1

  [You are here: DNA_k0.1]
```

---

## 8. Backend Implementation Strategy

### 8.1 Job Creation Automation Extension

**Current**: `execute_job_creation_with_progress()` in `job_creation.rs`

**Extension Points**:

**1. After validation**:
```rust
// NEW: Handle continuation if parent_job_id provided
if let Some(parent_id) = &params.parent_job_id {
    progress_callback("Loading parent job...");
    let parent_job = with_database(|db| db.load_job(parent_id))?
        .ok_or_else(|| anyhow!("Parent job not found"))?;

    // Validate parent can be continued from
    validate_parent_for_continuation(&parent_job, &params.continuation_type)?;
}
```

**2. After directory creation**:
```rust
// NEW: Copy parent files if this is a continuation
if let Some(parent_id) = &params.parent_job_id {
    progress_callback("Copying files from parent job...");

    let parent_input_files = copy_parent_files_to_child(
        &connection_manager,
        &parent_job,
        &project_dir,
        &params.continuation_type.as_ref().unwrap(),
        &params.restart_output_prefix.as_ref().unwrap_or(&"output".to_string()),
    ).await?;

    // Add copied files to input_files list
    input_files_with_metadata.extend(parent_input_files);
}
```

**3. Config generation**:
```rust
// Enhanced: Pass parent job info to script generator
let namd_config_content = SlurmScriptGenerator::generate_namd_config_with_continuation(
    &job_info,
    params.parent_job_id.as_deref(),
    params.continuation_type.as_ref(),
)?;
```

### 8.2 Script Generation Changes

**Enhancement**: Add restart/continuation block generation

```rust
pub fn generate_namd_config_with_continuation(
    job_info: &JobInfo,
    parent_job_id: Option<&str>,
    continuation_type: Option<&ContinuationType>,
) -> Result<String> {
    // ... existing code for basic config ...

    // NEW: Add restart/continuation block before "run" command
    let execution_block = if let Some(continuation_type) = continuation_type {
        generate_continuation_block(job_info, continuation_type)?
    } else {
        generate_fresh_start_block(job_info)
    };

    // ... combine config ...
}

fn generate_continuation_block(
    job_info: &JobInfo,
    continuation_type: &ContinuationType,
) -> Result<String> {
    // ... implementation details for CheckpointRestart vs NextStage ...
    // CheckpointRestart: Use .restart.* files, recover timestep, NO temperature
    // NextStage: Use final files, reset timestep, SET temperature
}
```

---

## 9. Tutorial Reference Guide

### 9.1 Essential Sections to Read

**Location**: `examples/origamiTutorial/origamiprotocols_0_markdown.md`

**Key Insights from Tutorial**:

1. **Restart Files Naming Convention**:
   - NAMD generates: `{outputName}.restart.{coor,vel,xsc}`
   - Also generates: `{outputName}.{coor,vel,xsc}` (final coordinates)
   - Restart from checkpoint: Use `.restart.*` files
   - Start new stage: Use final `.{coor,vel,xsc}` files

2. **Timestep Recovery is Critical**:
   - Must read and set `firsttimestep` when restarting
   - Ensures continuous time series across stages

3. **Temperature Initialization Rules**:
   - Fresh start: Set `temperature $temp` to initialize velocities
   - Restart: Do NOT set temperature (velocities from .vel file)
   - Next stage: Set temperature to reinitialize at new value

---

## 10. Open Questions & Future Decisions

### 10.1 Naming Conventions for Continued Jobs

**Problem**: What should child jobs be named automatically?

**Proposed**: **Option B with fallback to A**
- Checkpoint restart: Auto-suggest `{parent}_cont`
- Next stage: Prompt user for stage name, suggest based on parent
- User can always override

---

## 11. Integration with Existing Architecture

### 11.1 Automation System Integration

**Integration Points**:
1. **Job Creation Enhancement**: Add file copying logic.
2. **Progress Events**: Emit events for copying files.
3. **Validation**: Validate parent job status and files.

### 11.2 File Operations Integration

**Existing Capabilities**:
- `sync_directory_rsync()`: Cluster-to-cluster copy.
- `shell::safe_cp()`: Individual file copy.

**Integration for Job Continuation**:
- Use `sync_directory_rsync` to copy original `input_files` from parent.
- Use `cp` to copy specific restart files from parent `outputs`.

---

## 12. Success Criteria

**This feature will be considered successful when**:

### Functional Requirements
1. ✅ **Users can create multi-stage DNA origami workflows** without command line
2. ✅ **Jobs can be continued after timeout/failure** without losing progress
3. ✅ **Job chains are visualized** clearly in UI (parent-child relationships)
4. ✅ **Files are correctly copied** between parent and child jobs
5. ✅ **Restart blocks preserve timestep continuity** across stages

### Technical Requirements
1. ✅ **Each job is self-contained** (survives parent deletion)
2. ✅ **No cluster-side file references** between jobs (copied, not linked)
3. ✅ **Metadata accurately tracks** parent_job_id, continuation_type
4. ✅ **Database queries can find chains** (all children of parent, all jobs in tree)

### User Experience Requirements
1. ✅ **"Continue Job" button provides clear options** (checkpoint vs next stage)
2. ✅ **Form is pre-filled from parent** when continuing
3. ✅ **File copying progress is visible** during job creation
4. ✅ **Job chains are visually distinct** in job list (indentation/icons)

---

## 13. Appendices

### Appendix A: Tutorial File Locations

**Complete file tree** of `examples/origamiTutorial/`:

```
origamiTutorial/
├── origamiprotocols_0_markdown.md       # Main tutorial guide (1118 lines)
├── origamiprotocols_0.pdf               # PDF version
│
├── step1/                                # caDNAno design
│   └── hextube.json                     # DNA origami design file
│
├── step2/                                # Vacuum optimization
│   ├── hextube.namd                     # ⭐ CRITICAL: Vacuum simulation config
│   ├── hextube.psf                      # Structure topology
│   ├── hextube.pdb                      # Atomic coordinates
│   ├── hextube.exb                      # Extrabonds restraints
│   └── charmm36.nbfix/                  # Force field files
│       ├── par_all36_na.prm
│       └── par_water_ions_na.prm
│
├── step3/                                # Explicit solvent equilibration
│   ├── equil_min.namd                   # ⭐ CRITICAL: Minimization config
│   ├── equil_k0.5.namd               # RESTART EXAMPLE - Read this!
│   ├── equil_k0.1.namd               # Another restart (similar)
│   ├── equil_k0.01.namd              # Another restart (similar)
│   ├── equil_k0.namd                 # Production run (no restraints)
│   ├── hextube_MGHH_WI.psf
│   ├── hextube_MGHH_WI.pdb
│   ├── hextube_MGHH_WI_k0.5.enm.extra  # ENM restraints
│   └── mghh_extrabonds               # Mg²⁺ hexahydrate restraints
└── step4/                            # Analysis scripts (not critical)
```
