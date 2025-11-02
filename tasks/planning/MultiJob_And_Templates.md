# NAMD Job Chains and Templates - Design Document

> **Status**: Design Phase - Not Yet Implemented
> **Target Phase**: Phase 7+
> **Last Updated**: 2025-01-18

## Table of Contents

### Part 1: Background and Core Design
1. [Executive Summary](#1-executive-summary)
2. [Background: Understanding NAMD Workflows](#2-background-understanding-namd-workflows)
3. [Core Design Decisions](#3-core-design-decisions)
4. [Simulation Templates System](#4-simulation-templates-system)
5. [Data Model Design](#5-data-model-design)
6. [File Handling Architecture](#6-file-handling-architecture)
7. [UI/UX Design](#7-uiux-design)

### Part 2: Implementation Details
8. [Backend Implementation Strategy](#8-backend-implementation-strategy)
9. [NAMD Config Templates](#9-namd-config-templates)
10. [Tutorial Reference Guide](#10-tutorial-reference-guide)
11. [Open Questions & Future Decisions](#11-open-questions--future-decisions)
12. [Integration with Existing Architecture](#12-integration-with-existing-architecture)
13. [Success Criteria](#13-success-criteria)
14. [Appendices](#14-appendices)

---

## 1. Executive Summary

### What This Document Covers

This document defines the design for **Job Chains** and **NAMD Configuration Templates**, two interconnected features that enable multi-stage molecular dynamics workflows in NAMDRunner. These features address critical gaps discovered during analysis of the NAMD DNA origami tutorial.

### Key Decisions Made

1. **Job Chains as Unified Mechanism**: "Restart after timeout" and "next equilibration stage" are the same operation - creating a new job that continues from a parent job's outputs
2. **Jobs as Self-Contained Islands**: Each job in a chain copies all necessary files from its parent, ensuring resilience to deletion and scratch purges
3. **Template-Driven Dynamic Forms**: NAMD configuration UI is generated dynamically based on backend-provided template definitions
4. **Two Continuation Types**: Checkpoint restart (same simulation) vs next stage (new simulation phase)
5. **Focus on Computationally Intensive Jobs**: NAMDRunner will initially support Step 3 (solvated equilibration) templates; Step 2 (vacuum optimization) can be run locally by users

### What This Version Supports

**Tutorial has two workflow types**:
1. **Step 2: Vacuum Optimization** (~40 ps, computationally trivial, runs on laptop)
2. **Step 3: Solvated Equilibration** (4.8 ns per stage × multiple stages, computationally expensive, requires cluster)

**Current scope**: NAMDRunner will support **Step 3 workflows** - the computationally intensive multi-stage equilibration jobs that need HPC resources. Users can run Step 2 locally before uploading results to cluster.

### Prerequisites for Implementation

**Must complete first**:
- Category 1 issues from tutorial analysis (cellBasisVector, extrabonds support, configurable PME/NPT)
- Review existing codebase patterns for dynamic configuration (clusterConfig system)
- Understand current file operations and SFTP capabilities

**Related Issues**:
- Issue #9: Restart capability (multi-stage workflows)
- Issue #10: Simulation type presets (vacuum vs explicit solvent)
- Issue #11: Box dimension auto-detection
- Issue #12: Multi-stage workflow templates

---

## 2. Background: Understanding NAMD Workflows

### 2.1 Tutorial Analysis

**Primary Resource**: `examples/origamiTutorial/origamiprotocols_0_markdown.md`

**What to Read First** (Discovery Process):

1. **Section 3.2: Building an Optimized Atomistic Structure** (lines 265-425)
   - Understand vacuum optimization workflow
   - See how extrabonds restraints work
   - Note: PME is disabled, large box (1000Å), low damping (0.1)
   - Key file: `step2/hextube.namd`

2. **Section 3.3: MD Simulations in Explicit MgCl₂ Solution** (lines 426-609)
   - Understand multi-stage equilibration workflow
   - Progressive restraint reduction (k=0.5 → 0.1 → 0.01 → 0)
   - Each stage runs 4.8 ns, uses previous stage's restart files
   - Key files: `step3/equil_min.namd`, `step3/equil_k0.5.namd`, `step3/equil_k0.1.namd`

3. **Compare Stage Configurations**:
   - `step2/hextube.namd` vs `step3/equil_min.namd` - Vacuum vs explicit solvent differences
   - `step3/equil_min.namd` vs `step3/equil_k0.5.namd` - Fresh start vs restart differences

**Critical Observations**:
- Line 49 of `step2/hextube.namd`: `PME no` (vacuum simulation)
- Line 66 of `step3/equil_min.namd`: `PME yes` (periodic boundaries required)
- Lines 17-39 of `step3/equil_k0.5.namd`: Restart mechanism with timestep recovery

### 2.2 How NAMD Simulations Actually Work

#### Single-Stage Simulations (Simple Case)

```
User → Upload inputs (PDB, PSF, PRM) → Create NAMD config → Submit to SLURM → Job runs → Download outputs
```

**Required Files**:
- `.pdb` - Atomic coordinates
- `.psf` - Molecular topology
- `.prm` - Force field parameters

**Generated Outputs**:
- `.dcd` - Trajectory file
- `.restart.coor/vel/xsc` - Checkpoint files for continuation

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

#### Why Periodic Boundaries and PME Matter

**The Error We Encountered**:
```
FATAL ERROR: PME requires periodic boundary conditions.
```

**Root Cause**: PME (Particle Mesh Ewald) computes long-range electrostatics using periodic boundary conditions. Without `cellBasisVector` definitions, NAMD cannot initialize PME.

**Two Simulation Types**:

1. **Vacuum Optimization** (no PME):
   ```tcl
   PME                 no
   cellBasisVector1    1000 0 0  # Large box
   cellBasisVector2    0 1000 0  # prevents periodic
   cellBasisVector3    0 0 1000  # interactions
   ```

2. **Explicit Solvent** (requires PME):
   ```tcl
   PME                 yes
   PMEGridSpacing      1.5
   cellBasisVector1    124 0.0 0.0  # Actual system size
   cellBasisVector2    0.0 114 0.0  # from solvation
   cellBasisVector3    0.0 0.0 323  # box
   ```

### 2.3 The Realization: Job Chains

#### Initial Assumption (Wrong)

We initially thought there were two separate features:
- **Restart**: Continue failed/timed-out job (technical recovery)
- **Multi-Stage**: Progressive equilibration workflow (scientific methodology)

#### The Insight (Correct)

**They are the same mechanism!** All of these scenarios are identical:

**Scenario A: Timeout Recovery**
```
Job: Equilibration k=0.5 (24 cores, 4 hours, TIMEOUT)
  └─> Continue: Equilibration k=0.5 (48 cores, 8 hours, COMPLETED)
```

**Scenario B: Out of Memory Recovery**
```
Job: Equilibration k=0.1 (48 cores, 32GB, OOM)
  └─> Continue: Equilibration k=0.1 (48 cores, 64GB, COMPLETED)
```

**Scenario C: Next Equilibration Stage**
```
Job: Equilibration k=0.5 (COMPLETED)
  └─> Continue: Equilibration k=0.1 (new restraints, COMPLETED)
```

**Scenario D: Production Run**
```
Job: Equilibration k=0.01 (COMPLETED)
  └─> Continue: Production run (no restraints, new output name)
```

**Common Pattern**: Take Job N's outputs → Use as Job N+1's inputs → Submit new SLURM job

#### Real-World HPC Scenarios

**Why jobs fail/need continuation**:
- Walltime timeout (didn't finish in allocated time)
- Out of memory (system needs more RAM)
- Node failure (hardware crash)
- QoS preemption (higher priority job needs resources)
- Scientific decision (preliminary results suggest parameter adjustment)
- Workflow progression (next stage in multi-stage protocol)

**Why this matters**:
- Scientists constantly restart with tweaks
- Multi-stage workflows are the norm in MD simulations
- Failures are expected, not exceptional
- Need to insert stages mid-workflow (forgot a step, results show need more equilibration)

#### Tutorial's Two-Template Pattern

After analyzing the tutorial workflow, we identified **two distinct simulation template types**:

**Type 1: Vacuum Optimization (Step 2)**
- Purpose: Quick initial structural relaxation
- Computational cost: ~40 ps (~10 minutes on laptop)
- Key parameters: PME=no, large box (1000Å), langevinDamping=0.1, margin=30
- Example: `examples/origamiTutorial/step2/hextube.namd`
- **Decision**: Users run this locally - no cluster needed

**Type 2: Explicit Solvent Equilibration (Step 3)**
- Purpose: Multi-stage equilibration with progressive restraint reduction
- Computational cost: 4.8 ns per stage × multiple stages (~hours on HPC)
- Key parameters: PME=yes, NPT ensemble, cellBasisVector from solvated system, langevinDamping=5
- Stages differ only in: restraint strength (k=0.5 → 0.1 → 0.01 → 0) and minimize vs run
- Examples: `step3/equil_min.namd`, `step3/equil_k0.5.namd`, `step3/equil_k0.1.namd`
- **Decision**: This is what NAMDRunner supports - computationally intensive HPC jobs

**Implication for Implementation**:
- NAMDRunner needs **one flexible template** for Type 2 (solvated equilibration/production)
- Template must support: minimize vs run, different extrabonds files, configurable cell dimensions
- Multi-stage workflows handled via job chains, not separate template types
- Users upload pre-optimized structures from local Step 2 runs

#### Why Jobs Must Be Self-Contained

**Scratch Storage Reality**:
- `/scratch/alpine/` auto-purges after 90 days
- Not guaranteed to survive job completion
- Fast local storage optimized for job execution

**Project Storage Reality**:
- `/projects/` is persistent but not guaranteed (not our server)
- Could be affected by quota limits, server issues, user actions
- Better than scratch but still not bulletproof

**User Workflow Reality**:
- User downloads important outputs
- May delete old jobs to free space
- Shares jobs with collaborators who may not have parent jobs

**Therefore**: Each job in a chain must be an **independent island** with all necessary files copied locally.

#### The Unified Job Chain Concept

**Definition**: A job chain is a sequence of jobs where each child job continues from its parent's outputs.

**Properties**:
1. Each link in chain is a full NAMDRunner job (own ID, metadata, files)
2. Child job copies parent's outputs to its own input_files/
3. Parent-child relationship tracked via `parent_job_id` field
4. Chain can be interrupted (parent deletion doesn't break child)
5. Same mechanism for all continuation types (timeout, next stage, retry with changes)

**Example Chain**:
```
Job A: DNA_Minimization (COMPLETED)
  └─> Job B: DNA_k0.5 (TIMEOUT after 2h)
      └─> Job B-retry: DNA_k0.5_cont (COMPLETED with 8h walltime)
          └─> Job C: DNA_k0.1 (OOM with 32GB)
              └─> Job C-retry: DNA_k0.1_retry (COMPLETED with 64GB)
                  └─> Job D: DNA_Production (RUNNING)
```

Every arrow is the same relationship: "Uses restart files from parent"

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

#### Critical: Original Input Files Never Change

**NAMD's File Behavior**:
- Original input files (`.pdb`, `.psf`, `.prm`, `.exb`) are **read-only** to NAMD
- NAMD **never modifies** these files during execution
- NAMD only **creates new files** with different names (e.g., `output.coor`, `output.restart.coor`)
- Parent job's original inputs remain pristine and unchanged

**What This Means for Job Chains**:
1. **User's local copies always valid**: The files user uploaded for Job 1 can be reused for any child job without re-upload
2. **Parent's input_files/ directory unchanged**: Original inputs copied from parent are still in their original form
3. **Only outputs become new inputs**: The `.coor`, `.vel`, `.xsc` files generated by parent become inputs to child
4. **No tracking of "modified inputs"**: No need to distinguish "original" vs "modified" input files - originals never change

**File Flow Example**:
```
User uploads:
  - hextube.pdb (original structure)
  - hextube.psf (topology)

Job 1 execution:
  - Reads: hextube.pdb (unchanged)
  - Writes: equil_min.coor, equil_min.restart.coor (new files)

Job 2 execution:
  - Reads: hextube.pdb (still unchanged, copied from Job 1)
  - Reads: equil_min.restart.coor (Job 1's output, now Job 2's input)
  - Writes: equil_k0.5.coor, equil_k0.5.restart.coor (new files)

Job 3 execution:
  - Reads: hextube.pdb (still unchanged, copied from Job 2)
  - Reads: equil_k0.5.restart.coor (Job 2's output, now Job 3's input)
  - Writes: equil_k0.1.coor, equil_k0.1.restart.coor (new files)
```

**Simplification for Implementation**:
- No need to worry about "input file versioning"
- No need to track which inputs were "modified by job"
- Simply copy original inputs + copy selected outputs = complete child job setup

#### Why This Decision

**✅ Advantages**:
1. **Resilient to parent deletion**: Child survives if parent removed
2. **Survives scratch purge**: Parent's scratch may be deleted, child unaffected
3. **Shareable**: Can share child job independently
4. **Simple paths**: Config uses `input_files/filename.pdb` (no relative path complexity)
5. **Clear audit trail**: Each job has complete record of its inputs
6. **No brittle dependencies**: No symlinks, no relative paths across jobs

**❌ Cost**:
- Large files duplicated (restart files can be 500MB-1GB each)
- Disk space increases with chain length

**Why Cost is Acceptable**:
- Disk space is not a constraint (TB-scale project storage)
- Typical user has < 20 jobs (few GB total)
- Users can delete old jobs when done
- **Reliability > Optimization**: Scientists need workflows that work reliably over minimizing disk usage

#### Implementation Pattern

**During job continuation automation**:
```rust
// Copy parent outputs to child inputs
let parent_output_dir = format!("{}/outputs", parent_job.project_dir);
let child_input_dir = format!("{}/input_files", child_job.project_dir);

// Use cluster-side rsync (no local download needed)
connection_manager.sync_directory_rsync(
    &format!("{}/", parent_output_dir),  // Trailing / = copy contents
    &child_input_dir
).await?;

// Also copy original parent inputs
let parent_input_dir = format!("{}/input_files", parent_job.project_dir);
connection_manager.sync_directory_rsync(
    &format!("{}/", parent_input_dir),
    &child_input_dir
).await?;
```

**Metadata Transfer**:
```rust
// Convert parent OutputFile metadata to child InputFile metadata
for output_file in parent_job.output_files {
    child_input_files.push(InputFile {
        name: output_file.name,
        size: Some(output_file.size),  // Copy size, no re-stat needed
        uploaded_at: Some(chrono::Utc::now().to_rfc3339()),
        file_type: detect_type(&output_file.name),
        // ... other fields
    });
}
```

### 3.2 Job Chains Over In-Place Restart

#### Alternative Considered: In-Place Restart

**Concept**: Same job, update metadata to indicate restart, regenerate config/script, resubmit same job.

**How it would work**:
```
Job X (attempt 1): Submitted, TIMEOUT
  └─> Update metadata: restart_count = 1
  └─> Regenerate config.namd (using existing restart files)
  └─> Resubmit job.sbatch
  └─> New SLURM job ID assigned
```

#### Problems with In-Place Approach

**Problem 1: Metadata Complexity**

Current structure (simple):
```rust
pub struct JobInfo {
    pub slurm_job_id: Option<String>,  // Single SLURM job
    pub submitted_at: Option<String>,  // Single submission
    pub completed_at: Option<String>,  // Single completion
    pub status: JobStatus,             // Single status
}
```

In-place restart requires (complex):
```rust
pub struct JobInfo {
    pub slurm_job_ids: Vec<String>,    // Multiple submissions
    pub submission_history: Vec<SubmissionRecord>,
    pub restart_count: u32,
    pub current_stage: String,
}

struct SubmissionRecord {
    pub slurm_job_id: String,
    pub submitted_at: String,
    pub completed_at: Option<String>,
    pub status: JobStatus,
    pub namd_config_snapshot: NAMDConfig,  // What config ran
}
```

**Problem 2: Status Synchronization Complexity**

Simple (current):
```rust
let status = query_squeue(job.slurm_job_id)?;
job.status = status;
```

Complex (in-place):
```rust
// Which status do we show?
// - Latest submission only?
// - Combined status of all attempts?
// - Per-attempt status list?

for submission in job.submission_history {
    let status = query_squeue(submission.slurm_job_id)?;
    submission.status = status;
}
job.status = compute_aggregate_status(&job.submission_history)?;
```

**Problem 3: Which Config is Active?**

When user views job detail page:
- Show original config?
- Show current running config?
- Show all configs in timeline?

When user wants to modify and restart again:
- Edit "current" config?
- Create new from "latest" config?

**Problem 4: File Naming Collisions**

Tutorial uses different output names per stage:
```tcl
# Stage 1
outputName         equil_min
# Stage 2
outputName         equil_k0.5
```

In-place restart needs collision prevention:
```
outputs/
├── equil_k0.5_attempt1.coor
├── equil_k0.5_attempt1.restart.coor
├── equil_k0.5_attempt2.coor
├── equil_k0.5_attempt2.restart.coor
```

Which restart files for next restart? Latest? How does system know?

**Problem 5: Immutability Principle Violation**

Current architecture treats jobs as **immutable after creation**:
- Job created → metadata locked
- Job submitted → SLURM config locked
- Job completed → outputs preserved forever

Benefits:
- Simple audit trail
- Easy debugging (config never changes)
- Reproducibility (exact config that ran is preserved)

In-place restart breaks all of this.

**Problem 6: UI Complexity**

Job list shows:
- "Job X - Restart 3/5"?
- "Job X - Running Stage 4"?
- How to filter by status when one job has multiple statuses?

Job detail needs:
- Timeline of all submissions
- Accordion for each attempt's config
- Separate logs for each attempt
- Status indicator for each attempt

Much more complex than current simple job detail view.

#### Why Separate Jobs is Superior

**✅ Advantages**:
1. **Clean separation**: Each job = one NAMD run = one SLURM submission
2. **Simple 1:1 mapping**: Easy to understand and debug
3. **No architecture changes**: Fits perfectly with existing JobInfo structure
4. **Flexible resources**: Stage 1 (8 cores, 2h), Stage 2 (32 cores, 12h) - can't do with in-place
5. **Partial execution**: Submit stage 1, wait, review, then decide to proceed
6. **Error recovery**: Stage 2 fails? Fix and resubmit just stage 2
7. **UI remains simple**: Job list is just a list, job detail is single job

**❌ Only Disadvantage**:
- More jobs in database (but this is actually clearer for user)

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

#### Why This Distinction Matters

**Different File Selection**:
```
Parent job outputs/:
├── output.dcd                    # Trajectory (not needed)
├── output.restart.coor           # For checkpoint restart
├── output.restart.vel            # For checkpoint restart
├── output.restart.xsc            # For checkpoint restart
├── output.coor                   # For next stage
├── output.vel                    # For next stage
├── output.xsc                    # For next stage
└── namd_output.log
```

**Checkpoint restart** copies: `output.restart.*`
**Next stage** copies: `output.{coor,vel,xsc}` (final, not restart)

**Different Config Generation**:

Checkpoint restart:
```tcl
# Continue same simulation from checkpoint
bincoordinates     input_files/output.restart.coor
binvelocities      input_files/output.restart.vel
extendedSystem     input_files/output.restart.xsc
firsttimestep      [get_first_ts input_files/output.restart.xsc]
# DO NOT set temperature
```

Next stage:
```tcl
# Start new simulation using previous final coords
bincoordinates     input_files/output.coor
binvelocities      input_files/output.vel
extendedSystem     input_files/output.xsc
firsttimestep      0  # Reset time series
# DO set temperature (reinitialize velocities)
temperature        300
```

#### Implementation

**Data Model**:
```rust
#[derive(Serialize, Deserialize)]
pub enum ContinuationType {
    CheckpointRestart,  // Use .restart.* files
    NextStage,          // Use final .* files
}

pub struct JobInfo {
    // ... existing fields ...
    pub parent_job_id: Option<String>,
    pub continuation_type: Option<ContinuationType>,
}
```

**UI Distinction**:
```svelte
<RadioGroup bind:value={continuationType}>
  <Radio value="checkpoint">
    <strong>Checkpoint Restart</strong>
    <p class="help">Continue interrupted simulation (timeout/OOM/failure)</p>
  </Radio>
  <Radio value="next_stage">
    <strong>Next Stage</strong>
    <p class="help">Start next phase with modified parameters</p>
  </Radio>
</RadioGroup>
```

---

## 4. Simulation Templates System

### 4.1 The Template Problem

#### Observation from Tutorial

Different simulation types require **completely different parameter combinations**:

**Vacuum Optimization** (step2/hextube.namd):
```tcl
PME                 no
cellBasisVector1    1000 0 0     # Large box
langevinDamping     0.1          # Low damping
fullElectFrequency  3
margin              30
```

**Explicit Solvent NPT** (step3/equil_min.namd):
```tcl
PME                 yes
PMEGridSpacing      1.5
cellBasisVector1    124 0.0 0.0  # Actual system size
langevinDamping     5            # High damping
langevinPiston      on           # Pressure control
fullElectFrequency  2
```

**Problem**: New users don't know these combinations. If we just provide all parameters as a flat form, users will create invalid configurations (e.g., PME=yes with 1000Å box).

#### Why Not Hardcode UI Fields

**Current approach** (won't scale):
```svelte
<!-- ConfigurationTab.svelte -->
<label>Temperature (K)</label>
<input type="number" bind:value={namdConfig.temperature} />

<label>Timestep (fs)</label>
<input type="number" bind:value={namdConfig.timestep} />

<!-- ... 20 more hardcoded fields ... -->
```

**Problems**:
1. Adding new parameters requires editing component
2. Different templates need different fields
3. Can't hide/show fields conditionally
4. Can't provide template-specific help text
5. Can't validate interdependent parameters

### 4.2 Template-Driven Dynamic Forms Solution

#### Core Concept

**Backend defines UI structure**, frontend renders dynamically.

**Template Definition** (in Rust):
```rust
pub struct NAMDTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub use_case: String,
    pub form_sections: Vec<FormSection>,
}

pub struct FormSection {
    pub title: String,
    pub description: Option<String>,
    pub fields: Vec<FieldDefinition>,
}

pub struct FieldDefinition {
    pub key: String,              // "use_pme"
    pub label: String,            // "Use PME Electrostatics"
    pub field_type: FieldType,
    pub default_value: Value,
    pub required: bool,
    pub help_text: Option<String>,
    pub visible_when: Option<VisibilityCondition>,
}
```

**Frontend renders based on definition**:
```svelte
{#each template.form_sections as section}
  <fieldset>
    <legend>{section.title}</legend>
    {#each section.fields as field}
      {#if shouldShowField(field, formValues)}
        <DynamicField {field} bind:value={formValues[field.key]} />
      {/if}
    {/each}
  </fieldset>
{/each}
```

#### Field Types System

```rust
pub enum FieldType {
    Boolean,
    Number {
        min: Option<f64>,
        max: Option<f64>,
        step: Option<f64>,
        unit: Option<String>,  // "K", "fs", "Å"
    },
    Text {
        pattern: Option<String>,
        max_length: Option<usize>,
    },
    Select {
        options: Vec<SelectOption>,
    },
    Dimensions {  // Special: X, Y, Z inputs
        min: Option<f64>,
        max: Option<f64>,
    },
    FileUpload {
        accepted_extensions: Vec<String>,
    },
}

pub struct SelectOption {
    pub value: String,
    pub label: String,
    pub description: Option<String>,
}
```

#### Conditional Visibility

**Example**: Show "margin" parameter only when PME is disabled

```rust
FieldDefinition {
    key: "margin",
    label: "Pairlist Margin",
    field_type: Number { min: 0, max: 50, step: 5, unit: None },
    default_value: json!(30),
    visible_when: Some(VisibilityCondition {
        field: "use_pme",
        operator: Equals,
        value: json!(false),
    }),
}
```

**Frontend evaluation**:
```typescript
function shouldShowField(field: FieldDefinition, formValues: Record<string, any>): boolean {
  if (!field.visible_when) return true;

  const { field: dependentField, operator, value } = field.visible_when;
  const actualValue = formValues[dependentField];

  switch (operator) {
    case 'Equals':
      return actualValue === value;
    case 'NotEquals':
      return actualValue !== value;
    // ... other operators
  }
}
```

### 4.3 Parameter Validation Rules

#### Problem

Each parameter has validation rules that depend on:
1. Physical constraints (temperature > 0K)
2. NAMD software limits (timestep 0.1-4.0 fs)
3. Cluster resources (memory, walltime)
4. Interdependent parameters (PME requires cellBasisVector)

**Users need to specify validation rules per template parameter.**

#### Proposed Validation System

**Per-Field Validation**:
```rust
pub struct FieldValidation {
    pub rules: Vec<ValidationRule>,
    pub custom_validator: Option<String>,  // Function name for complex validation
}

pub enum ValidationRule {
    Range { min: f64, max: f64, message: String },
    MinValue { min: f64, message: String },
    MaxValue { max: f64, message: String },
    Pattern { regex: String, message: String },
    RequiredIf { field: String, equals: Value, message: String },
    OneOf { values: Vec<Value>, message: String },
}
```

**Example**: Temperature validation
```rust
FieldDefinition {
    key: "temperature",
    validation: Some(FieldValidation {
        rules: vec![
            ValidationRule::Range {
                min: 200.0,
                max: 400.0,
                message: "Temperature must be in biological range (200-400 K)".to_string(),
            },
        ],
        custom_validator: None,
    }),
}
```

**Example**: Interdependent validation (PME requires cellBasisVector)
```rust
FieldDefinition {
    key: "cell_basis_vector",
    validation: Some(FieldValidation {
        rules: vec![
            ValidationRule::RequiredIf {
                field: "use_pme",
                equals: json!(true),
                message: "Cell basis vectors required when PME is enabled".to_string(),
            },
        ],
    }),
}
```

#### Validation Execution

**Frontend** (immediate feedback):
```typescript
function validateField(field: FieldDefinition, value: any): string | null {
  for (const rule of field.validation.rules) {
    switch (rule.type) {
      case 'Range':
        if (value < rule.min || value > rule.max) {
          return rule.message;
        }
        break;
      // ... other rules
    }
  }
  return null;  // Valid
}
```

**Backend** (authoritative):
```rust
pub fn validate_template_values(
    template_id: &str,
    field_values: &HashMap<String, Value>
) -> ValidationResult {
    let template = get_template(template_id)?;
    let mut issues = Vec::new();

    for field in template.field_definitions {
        if let Some(validation) = field.validation {
            for rule in validation.rules {
                if !check_rule(&rule, &field_values[&field.key]) {
                    issues.push(rule.message);
                }
            }
        }
    }

    ValidationResult {
        is_valid: issues.is_empty(),
        issues,
        warnings: vec![],
        suggestions: vec![],
    }
}
```

### 4.4 Template Examples from Tutorial

**See Section 9 for complete template definitions derived from tutorial files.**

The tutorial reveals **two distinct template types** for DNA origami workflows:

#### Template Type 1: Vacuum Optimization (Step 2)

**Purpose**: Initial structural relaxation before solvation

**Characteristics**:
- No periodic boundaries (PME disabled)
- Large simulation box: cellBasisVector 1000Å × 1000Å × 1000Å
- Low friction: langevinDamping = 0.1
- Large margin: 30Å (prevents atoms from crossing box boundaries)
- Uses extrabonds for structure stabilization
- Short duration: ~40 ps (21,600 steps at 2fs timestep)
- Computational cost: ~10 minutes on laptop

**Key Parameters**:
```tcl
PME                 no
cellBasisVector1    1000  0    0
cellBasisVector2    0     1000 0
cellBasisVector3    0     0    1000
langevinDamping     0.1
margin              30
extraBondsFile      hextube.exb
run                 21600
```

**Example**: `examples/origamiTutorial/step2/hextube.namd`

**NAMDRunner Support Status**: ❌ **Not implemented** - Users run this locally before uploading results to cluster. Computationally trivial, doesn't need HPC resources.

#### Template Type 2: Explicit Solvent Equilibration/Production (Step 3)

**Purpose**: Multi-stage equilibration in explicit solvent with progressive restraint reduction

**Characteristics**:
- Periodic boundaries enabled (PME yes)
- NPT ensemble (constant pressure/temperature)
- System-specific box dimensions from solvation (e.g., 124Å × 114Å × 323Å for hextube)
- High friction: langevinDamping = 5 (appropriate for aqueous solution)
- Uses extrabonds with variable restraint strength (k=0.5 → 0.1 → 0.01 → 0)
- Long duration per stage: 4.8 ns (2,400,000 steps at 2fs timestep)
- Computational cost: Hours per stage on HPC cluster

**Key Parameters** (common to all stages):
```tcl
PME                      yes
PMEGridSpacing           1.5
cellBasisVector1         124  0    0    # From solvated system
cellBasisVector2         0    114  0
cellBasisVector3         0    0    323
langevinDamping          5
langevinPiston           on
langevinPistonTarget     1.01325
```

**Stage-Specific Variations**:
- **Stage 1 (Minimization)**: `minimize 4800` instead of `run`
- **Stage 2-4**: Different extrabonds files (`k0.5.enm.extra`, `k0.1.enm.extra`, `k0.01.enm.extra`)
- **Stage 5 (Production)**: No extrabonds restraints

**Examples**: `step3/equil_min.namd`, `step3/equil_k0.5.namd`, `step3/equil_k0.1.namd`

**NAMDRunner Support Status**: ✅ **This is what we implement** - Single flexible template that handles all Step 3 variations.

#### Implementation Strategy

**For Current Single-Job-Type App**:

NAMDRunner needs **one flexible explicit solvent template** that supports:
- Configurable cellBasisVector (user provides or extracted from .xsc file)
- Configurable langevinDamping
- PME enable/disable toggle
- NPT enable/disable toggle
- Extrabonds file selection (optional)
- Execution mode: minimize vs run
- Output frequencies (dcdfreq, restartfreq, etc.)

**Templates NOT needed initially**:
- ~~Vacuum Optimization~~ - Users handle Step 2 locally before uploading to cluster

**Future Template Extensions** (when supporting more simulation types):
- NVT ensemble (constant volume instead of NPT)
- Implicit solvent simulations
- Different force fields (AMBER, OPLS)
- Advanced sampling (steered MD, umbrella sampling, replica exchange)

**Key Insight**: The "multi-stage" aspect of DNA origami workflows comes from **job chains** (each stage continues from previous outputs), not from needing multiple template types. One flexible template + job chain mechanism = complete workflow support.

---

## 5. Data Model Design

### 5.1 Extensions to JobInfo

**Current Structure** (`src-tauri/src/types/core.rs`):
```rust
pub struct JobInfo {
    pub job_id: String,
    pub job_name: String,
    pub status: JobStatus,
    pub slurm_job_id: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub submitted_at: Option<String>,
    pub completed_at: Option<String>,
    pub project_dir: Option<String>,
    pub scratch_dir: Option<String>,
    pub error_info: Option<String>,
    pub slurm_stdout: Option<String>,
    pub slurm_stderr: Option<String>,
    pub namd_config: NAMDConfig,
    pub slurm_config: SlurmConfig,
    pub input_files: Vec<InputFile>,
    pub output_files: Option<Vec<OutputFile>>,
    pub remote_directory: String,
}
```

**Required Extensions**:
```rust
pub struct JobInfo {
    // ... all existing fields unchanged ...

    // NEW: Template tracking
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

**Why These Fields are Optional**:
- `template_id`: Required for new jobs, but allows backward compatibility with existing jobs
- `parent_job_id`: Only for continued jobs, None for fresh jobs
- `continuation_type`: Only present when parent_job_id is Some
- `root_job_id`: Convenience for UI, can be computed by traversing parent chain
- `chain_depth`: Convenience for UI, can be computed from parent chain

**Backward Compatibility**:
- All new fields are `Option<T>` or have defaults
- Existing jobs in database continue to work
- New jobs populate new fields
- No database migration needed (document-store pattern)

### 5.2 NAMDConfig Extensions

**Current Structure**:
```rust
pub struct NAMDConfig {
    pub steps: u32,
    pub temperature: f64,
    pub timestep: f64,
    pub outputname: String,
    pub dcd_freq: Option<u32>,
    pub restart_freq: Option<u32>,
}
```

**Required Extensions** (from Category 1 issues):
```rust
pub struct NAMDConfig {
    // Existing fields unchanged
    pub steps: u32,
    pub temperature: f64,
    pub timestep: f64,
    pub outputname: String,
    pub dcd_freq: Option<u32>,
    pub restart_freq: Option<u32>,

    // NEW: Periodic boundaries (Issue #1 - CRITICAL)
    pub cell_basis_vector_x: Option<f64>,
    pub cell_basis_vector_y: Option<f64>,
    pub cell_basis_vector_z: Option<f64>,

    // NEW: PME configuration (Issue #2)
    pub use_pme: Option<bool>,              // Default: true
    pub pme_grid_spacing: Option<f64>,      // Default: 1.5

    // NEW: Pressure control (Issue #3)
    pub pressure_control: Option<bool>,     // Default: true (NPT)
    pub target_pressure: Option<f64>,       // Default: 1.01325 bar
    pub piston_period: Option<f64>,         // Default: 1000
    pub piston_decay: Option<f64>,          // Default: 500

    // NEW: Advanced parameters (Issues #4-7)
    pub langevin_damping: Option<f64>,      // Default: 5.0
    pub margin: Option<u32>,                // Only for vacuum (no PME)
    pub full_elect_frequency: Option<u32>,  // Default: 2
    pub wrap_all: Option<bool>,             // Default: false
    pub wrap_water: Option<bool>,           // Default: false

    // NEW: Minimization
    pub minimize_steps: Option<u32>,        // If set, run minimization before dynamics
}
```

**Template-Driven Values**:
These fields will be populated from template defaults, but users can override. Template system (Section 4) manages which fields are shown/hidden based on simulation type.

### 5.3 Template Data Structure

**Backend Definition** (`src-tauri/src/templates.rs`):
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NAMDTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: TemplateCategory,
    pub use_case: String,
    pub form_sections: Vec<FormSection>,
    pub default_values: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemplateCategory {
    Optimization,      // Vacuum/minimization
    Equilibration,     // Restrained equilibration
    Production,        // Unrestrained production
    Custom,            // User-defined
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormSection {
    pub title: String,
    pub description: Option<String>,
    pub fields: Vec<FieldDefinition>,
    pub collapsible: bool,
    pub collapsed_by_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDefinition {
    pub key: String,
    pub label: String,
    pub field_type: FieldType,
    pub default_value: serde_json::Value,
    pub required: bool,
    pub help_text: Option<String>,
    pub validation: Option<FieldValidation>,
    pub visible_when: Option<VisibilityCondition>,
}

// FieldType, FieldValidation, VisibilityCondition defined in Section 4.3
```

**Frontend TypeScript Types** (`src/lib/types/template.ts`):
```typescript
export interface NAMDTemplate {
  id: string;
  name: string;
  description: string;
  category: 'Optimization' | 'Equilibration' | 'Production' | 'Custom';
  use_case: string;
  form_sections: FormSection[];
  default_values: Record<string, any>;
}

export interface FormSection {
  title: string;
  description?: string;
  fields: FieldDefinition[];
  collapsible: boolean;
  collapsed_by_default: boolean;
}

export interface FieldDefinition {
  key: string;
  label: string;
  field_type: FieldType;
  default_value: any;
  required: boolean;
  help_text?: string;
  validation?: FieldValidation;
  visible_when?: VisibilityCondition;
}

// Complete type definitions mirror Rust structure
```

### 5.4 Continuation Metadata

**CreateJobParams Extension**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJobParams {
    pub job_name: String,
    pub template_id: String,              // NEW
    pub namd_config: NAMDConfig,
    pub slurm_config: SlurmConfig,
    pub input_files: Vec<InputFile>,

    // NEW: Continuation support
    pub parent_job_id: Option<String>,
    pub continuation_type: Option<ContinuationType>,
    pub restart_output_prefix: Option<String>,  // e.g., "equil_k0.5", "output"
}
```

**Frontend CreateJobParams**:
```typescript
export interface CreateJobParams {
  job_name: string;
  template_id: string;
  namd_config: NAMDConfig;
  slurm_config: SlurmConfig;
  input_files: InputFile[];
  parent_job_id?: string;
  continuation_type?: 'CHECKPOINT_RESTART' | 'NEXT_STAGE';
  restart_output_prefix?: string;
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

**Child Job Result**:
```
/projects/$USER/namdrunner_jobs/child_job_id/
├── input_files/
│   ├── structure.pdb                  # FROM parent input_files/
│   ├── structure.psf                  # FROM parent input_files/
│   ├── parameters.prm                 # FROM parent input_files/
│   ├── restraints.exb                 # FROM parent input_files/ (if present)
│   ├── equil_k0.5.restart.coor        # FROM parent outputs/ (checkpoint)
│   ├── equil_k0.5.restart.vel         # FROM parent outputs/ (checkpoint)
│   └── equil_k0.5.restart.xsc         # FROM parent outputs/ (checkpoint)
├── scripts/
│   ├── config.namd                    # NEW (generated with restart block)
│   └── job.sbatch                     # NEW
└── outputs/                           # Empty (filled during execution)
```

### 6.2 File Discovery Strategy

#### Prefix-Based File Matching (Tutorial Pattern)

**Tutorial uses prefix convention**:
```tcl
# Stage 2 config
outputName         equil_k0.5

# Output files created:
# - equil_k0.5.dcd
# - equil_k0.5.restart.coor
# - equil_k0.5.restart.vel
# - equil_k0.5.restart.xsc

# Stage 3 references by prefix
set input          equil_k0.5
bincoordinates     $input.coor
```

**Implementation**:
```rust
pub fn find_restart_files(
    parent_job: &JobInfo,
    prefix: &str
) -> Result<Vec<String>> {
    let mut files = Vec::new();

    let required_extensions = vec!["restart.coor", "restart.vel", "restart.xsc"];

    for ext in required_extensions {
        let filename = format!("{}.{}", prefix, ext);

        // Check if file exists in parent outputs
        if let Some(output_files) = &parent_job.output_files {
            if output_files.iter().any(|f| f.name == filename) {
                files.push(filename);
            } else {
                return Err(anyhow!("Required restart file not found: {}", filename));
            }
        } else {
            return Err(anyhow!("Parent job has no output files"));
        }
    }

    Ok(files)
}
```

#### Convention for Restart Files

**Standard naming** (enforced by NAMD):
- Checkpoint files: `{outputName}.restart.{coor,vel,xsc}`
- Final coordinates: `{outputName}.{coor,vel,xsc}`
- Trajectory: `{outputName}.dcd`

**NAMDRunner enforces**:
- `outputName` in NAMD config must be simple (no paths, no special characters)
- Validated during job creation
- Enables reliable file discovery

#### User Selection vs Automatic Detection

**Phase 1: Automatic (Prefix-Based)**
- User provides output prefix from parent (e.g., "output", "equil_k0.5")
- System finds matching `.restart.{coor,vel,xsc}` or `.{coor,vel,xsc}`
- Validates all three files exist
- Copies to child input_files/

**Phase 2: Manual Selection (Future)**
- UI shows parent's output files
- User checks which files to copy
- Useful for non-standard naming or selective file copying
- Advanced feature for power users

### 6.3 File Copying During Continuation

#### Cluster-Side Copy (No Local Intermediary)

**Existing Infrastructure**: NAMDRunner already supports cluster-to-cluster file operations via `sync_directory_rsync()` method (see Section 12.2 Integration Research).

**Implementation**:
```rust
// In job creation automation
pub async fn copy_parent_files_to_child(
    connection_manager: &ConnectionManager,
    parent_job: &JobInfo,
    child_project_dir: &str,
    continuation_type: &ContinuationType,
    restart_prefix: &str,
) -> Result<Vec<InputFile>> {
    let parent_project_dir = parent_job.project_dir.as_ref()
        .ok_or_else(|| anyhow!("Parent job has no project directory"))?;

    let child_input_dir = format!("{}/input_files", child_project_dir);

    // Step 1: Copy original input files from parent
    let parent_input_dir = format!("{}/input_files", parent_project_dir);
    connection_manager.sync_directory_rsync(
        &format!("{}/", parent_input_dir),  // Trailing / = copy contents
        &child_input_dir
    ).await?;

    // Step 2: Copy continuation files from parent outputs
    let parent_output_dir = format!("{}/outputs", parent_project_dir);
    let files_to_copy = match continuation_type {
        ContinuationType::CheckpointRestart => {
            // Copy .restart.* files
            vec![
                format!("{}.restart.coor", restart_prefix),
                format!("{}.restart.vel", restart_prefix),
                format!("{}.restart.xsc", restart_prefix),
            ]
        }
        ContinuationType::NextStage => {
            // Copy final coordinate files
            vec![
                format!("{}.coor", restart_prefix),
                format!("{}.vel", restart_prefix),
                format!("{}.xsc", restart_prefix),
            ]
        }
    };

    // Copy individual files (rsync or cp)
    for filename in files_to_copy {
        let src = format!("{}/{}", parent_output_dir, filename);
        let dst = format!("{}/{}", child_input_dir, filename);

        // Use cluster-side cp command
        let cp_cmd = shell::safe_cp(&src, &dst);
        connection_manager.execute_command(&cp_cmd, Some(timeouts::FILE_COPY)).await?;
    }

    // Step 3: Build InputFile metadata records
    let mut child_input_files = Vec::new();

    // Original inputs
    for parent_input in &parent_job.input_files {
        child_input_files.push(InputFile {
            name: parent_input.name.clone(),
            local_path: "".to_string(),  // No local path for copied files
            remote_name: Some(parent_input.name.clone()),
            file_type: parent_input.file_type.clone(),
            size: parent_input.size,  // Copy size from parent
            uploaded_at: Some(chrono::Utc::now().to_rfc3339()),
        });
    }

    // Continuation files
    for filename in files_to_copy {
        // Find size from parent output metadata
        let size = parent_job.output_files.as_ref()
            .and_then(|outputs| outputs.iter().find(|f| f.name == filename))
            .map(|f| f.size);

        child_input_files.push(InputFile {
            name: filename.clone(),
            local_path: "".to_string(),
            remote_name: Some(filename),
            file_type: Some(determine_file_type(&filename)),
            size,
            uploaded_at: Some(chrono::Utc::now().to_rfc3339()),
        });
    }

    Ok(child_input_files)
}

fn determine_file_type(filename: &str) -> NAMDFileType {
    if filename.ends_with(".coor") || filename.ends_with(".vel") || filename.ends_with(".xsc") {
        NAMDFileType::Other  // Or new type: NAMDFileType::Checkpoint
    } else {
        NAMDFileType::Other
    }
}
```

**Key Points**:
- No download to local machine (cluster-side operations only)
- Metadata (size) copied from parent (no re-stat needed)
- Timestamp set to current time (when copy occurred)
- All files end up in child's `input_files/` directory

---

## 7. UI/UX Design

### 7.1 Creating Fresh Jobs

**Current Flow** (CreateJobPage.svelte):
1. Resources Tab → Select partition, cores, memory, walltime, QoS
2. Configuration Tab → Enter NAMD parameters
3. Files Tab → Upload input files
4. Review Tab → Validate and submit

**Enhanced Flow with Templates**:
1. **Template Selection** (NEW)
   - Dropdown: "Vacuum Optimization", "Equilibration NPT", "Production", "Custom"
   - Description card shows use case
   - Example: "Vacuum Optimization - For initial DNA origami structure relaxation"

2. **Resources Tab** (unchanged)
   - May pre-fill from template recommendations (future)

3. **Configuration Tab** (DYNAMIC)
   - Form fields generated from selected template
   - Parameters pre-filled from template defaults
   - User can override any value
   - Conditional fields (e.g., margin only shows when PME=off)

4. **Files Tab** (unchanged)
   - Upload PDB, PSF, PRM files
   - Upload extrabonds (.exb) if template supports

5. **Review Tab** (enhanced)
   - Shows template name
   - Validation checks template-specific rules

**UI Mockup** (Template Selector):
```svelte
<div class="template-selector">
  <label>Simulation Type</label>
  <select bind:value={selectedTemplateId} on:change={loadTemplate}>
    <option value="">-- Select Template --</option>
    {#each $availableTemplates as template}
      <option value={template.id}>{template.name}</option>
    {/each}
  </select>

  {#if selectedTemplate}
    <div class="template-description">
      <p class="use-case">{selectedTemplate.use_case}</p>
      <p class="description">{selectedTemplate.description}</p>
    </div>
  {/if}
</div>
```

**UI Mockup** (Dynamic Configuration Tab):
```svelte
{#if selectedTemplate}
  <div class="dynamic-config-form">
    {#each selectedTemplate.form_sections as section}
      <fieldset class:collapsed={section.collapsed_by_default}>
        <legend>{section.title}</legend>
        {#if section.description}
          <p class="section-description">{section.description}</p>
        {/if}

        {#each section.fields as field}
          {#if shouldShowField(field, formValues)}
            <DynamicField
              definition={field}
              bind:value={formValues[field.key]}
              error={fieldErrors[field.key]}
              on:change={() => validateField(field)}
            />
          {/if}
        {/each}
      </fieldset>
    {/each}
  </div>
{/if}
```

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
   - If next stage: Can change template
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

**UI Mockup** (Continuation Dialog):
```svelte
<dialog open={showContinuationDialog}>
  <h2>Continue from: {parentJob.job_name}</h2>

  <div class="continuation-type-selector">
    <RadioGroup bind:value={continuationType}>
      <Radio value="checkpoint">
        <strong>Checkpoint Restart</strong>
        <p>Continue same simulation from last checkpoint</p>
        <ul class="file-preview">
          <li>{parentJob.namd_config.outputname}.restart.coor</li>
          <li>{parentJob.namd_config.outputname}.restart.vel</li>
          <li>{parentJob.namd_config.outputname}.restart.xsc</li>
        </ul>
      </Radio>

      <Radio value="next_stage">
        <strong>Next Stage</strong>
        <p>Start new stage with modified parameters</p>
        <ul class="file-preview">
          <li>{parentJob.namd_config.outputname}.coor</li>
          <li>{parentJob.namd_config.outputname}.vel</li>
          <li>{parentJob.namd_config.outputname}.xsc</li>
        </ul>
      </Radio>
    </RadioGroup>
  </div>

  {#if continuationType === 'next_stage'}
    <div class="template-selector">
      <label>Template (optional: change simulation type)</label>
      <select bind:value={newTemplateId}>
        <option value={parentJob.template_id}>
          Keep parent template: {getTemplateName(parentJob.template_id)}
        </option>
        {#each $availableTemplates as template}
          <option value={template.id}>{template.name}</option>
        {/each}
      </select>
    </div>
  {/if}

  <div class="job-name-input">
    <label>New Job Name</label>
    <input type="text" bind:value={newJobName} />
  </div>

  <!-- Dynamic form appears here (pre-filled from parent) -->

  <div class="dialog-actions">
    <button on:click={cancelContinuation}>Cancel</button>
    <button on:click={createContinuation} class="primary">
      Create Continuation Job
    </button>
  </div>
</dialog>
```

### 7.3 Job Chain Visualization

**Job List Enhancement** (Tree View):
```
Jobs List:

  ☑ DNA_Minimization                    COMPLETED  2 days ago
    └─ ☑ DNA_k0.5                       TIMEOUT    2 days ago
       └─ ☑ DNA_k0.5_retry              COMPLETED  2 days ago
          └─ ☐ DNA_k0.1                 RUNNING    Now

  ☑ Protein_Equilibration               COMPLETED  1 week ago
    └─ ☐ Protein_Production             RUNNING    1 hour ago
```

**Visual Indicators**:
- Indentation shows parent-child relationship
- Icons: ☑ completed, ☐ running, ⚠ failed, ⏸ timeout
- Clicking parent expands/collapses children
- Hover shows "Parent: X, Depth: 2"

**Job Detail Enhancement** (Chain Timeline):
```
Job Chain Timeline:

  DNA_Minimization ──┬── DNA_k0.5 ──┬── DNA_k0.5_retry ──┬── DNA_k0.1
  (COMPLETED)        │   (TIMEOUT)  │   (COMPLETED)       │   (RUNNING)
                     │              │                     │
                  Failed after 2h  Continued with 8h   Next stage k=0.1

  [You are here: DNA_k0.1]

  Actions:
  - View parent: DNA_k0.5_retry
  - View root: DNA_Minimization
  - Continue this job (when complete)
```

### 7.4 File Selection Interface

**When Creating Continuation** (Advanced Mode):
```svelte
<div class="file-selector">
  <h3>Files to Copy from Parent</h3>

  <div class="file-categories">
    <details open>
      <summary>Original Inputs (always copied)</summary>
      <ul class="file-list">
        {#each parentJob.input_files as file}
          <li class="auto-selected">
            <span class="filename">{file.name}</span>
            <span class="filesize">{formatSize(file.size)}</span>
            <span class="filetype">{file.file_type}</span>
          </li>
        {/each}
      </ul>
    </details>

    <details open>
      <summary>
        Restart Files (
        {continuationType === 'checkpoint' ? 'checkpoint' : 'final coordinates'}
        )
      </summary>
      <ul class="file-list">
        {#each autoSelectedRestartFiles as file}
          <li class="auto-selected">
            <input type="checkbox" checked disabled />
            <span class="filename">{file.name}</span>
            <span class="filesize">{formatSize(file.size)}</span>
          </li>
        {/each}
      </ul>
    </details>

    <details>
      <summary>Other Output Files (optional)</summary>
      <ul class="file-list">
        {#each otherOutputFiles as file}
          <li>
            <input type="checkbox" bind:checked={file.selected} />
            <span class="filename">{file.name}</span>
            <span class="filesize">{formatSize(file.size)}</span>
          </li>
        {/each}
      </ul>
    </details>
  </div>

  <div class="total-size">
    Total size to copy: {formatSize(totalSize)}
  </div>
</div>
```

**Simple Mode** (Phase 1):
- Auto-select files based on continuation type
- Show preview list (read-only)
- User just confirms

**Advanced Mode** (Phase 2):
- User can check/uncheck files
- Useful for non-standard workflows
- Validation ensures required files selected

---

**END OF PART 1**

*Continue to Part 2 for Sections 8-15 and Appendices...*

## 8. Backend Implementation Strategy

### 8.1 Template Management

**Phase 1: Hardcoded Rust Templates**

Templates defined as compile-time data structures:

```rust
// src-tauri/src/templates/mod.rs
pub mod definitions;

lazy_static! {
    static ref TEMPLATE_REGISTRY: HashMap<String, NAMDTemplate> = {
        let mut templates = HashMap::new();
        templates.insert("vacuum_optimization".to_string(), definitions::vacuum_optimization());
        templates.insert("explicit_solvent_npt".to_string(), definitions::explicit_solvent_npt());
        templates.insert("minimization".to_string(), definitions::minimization());
        templates.insert("equilibration".to_string(), definitions::equilibration());
        templates
    };
}

pub fn get_template(template_id: &str) -> Option<&NAMDTemplate> {
    TEMPLATE_REGISTRY.get(template_id)
}

pub fn get_all_templates() -> Vec<&NAMDTemplate> {
    TEMPLATE_REGISTRY.values().collect()
}
```

**IPC Command**:
```rust
// src-tauri/src/commands/templates.rs
#[tauri::command]
pub fn get_template_capabilities() -> ApiResult<TemplateCapabilities> {
    let templates = crate::templates::get_all_templates();
    ApiResult::success(TemplateCapabilities {
        templates: templates.into_iter().cloned().collect(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[tauri::command]
pub fn get_template(template_id: String) -> ApiResult<NAMDTemplate> {
    match crate::templates::get_template(&template_id) {
        Some(template) => ApiResult::success(template.clone()),
        None => ApiResult::error(format!("Template '{}' not found", template_id)),
    }
}
```

**Phase 2: Database-Stored Templates** (Future)
- User-created templates saved to SQLite
- Template CRUD operations
- Template versioning (maybe not needed based on requirements)
- Template sharing/export

### 8.2 Job Creation Automation Extension

**Current**: `execute_job_creation_with_progress()` in `job_creation.rs`

**Extension Points**:

**1. After validation (line ~30)**:
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

**2. After directory creation (line ~73)**:
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

**3. Config generation (line ~195)**:
```rust
// Enhanced: Pass parent job info to script generator
let namd_config_content = SlurmScriptGenerator::generate_namd_config_with_continuation(
    &job_info,
    params.parent_job_id.as_deref(),
    params.continuation_type.as_ref(),
)?;
```

**New Helper Function**:
```rust
fn validate_parent_for_continuation(
    parent: &JobInfo,
    continuation_type: &Option<ContinuationType>
) -> Result<()> {
    // Ensure parent has completed (for next stage) or failed/timeout (for checkpoint)
    match continuation_type {
        Some(ContinuationType::NextStage) => {
            if parent.status != JobStatus::Completed {
                return Err(anyhow!(
                    "Cannot start next stage from incomplete parent job. Parent status: {:?}",
                    parent.status
                ));
            }
        }
        Some(ContinuationType::CheckpointRestart) => {
            // Can restart from any terminal state
            if !matches!(parent.status, JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled | JobStatus::Timeout) {
                return Err(anyhow!(
                    "Cannot restart from non-terminal parent job. Parent status: {:?}",
                    parent.status
                ));
            }
        }
        None => {} // Fresh job, no validation needed
    }

    // Ensure parent has output files
    if parent.output_files.is_none() || parent.output_files.as_ref().unwrap().is_empty() {
        return Err(anyhow!("Parent job has no output files available"));
    }

    Ok(())
}
```

### 8.3 Script Generation Changes

**Current**: `generate_namd_config()` in `script_generator.rs` (lines 81-218)

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

    // Insert execution block before final "run" command
    let config = format!(
        r#"
{existing_config}

#############################################################
## EXECUTION SCRIPT                                        ##
#############################################################

{execution_block}

# Production run
run {steps}
"#,
        existing_config = basic_config,
        execution_block = execution_block,
        steps = job_info.namd_config.steps,
    );

    Ok(config)
}

fn generate_continuation_block(
    job_info: &JobInfo,
    continuation_type: &ContinuationType,
) -> Result<String> {
    // Find restart files in input_files
    let restart_prefix = job_info.namd_config.restart_output_prefix
        .as_deref()
        .unwrap_or("output");

    let block = match continuation_type {
        ContinuationType::CheckpointRestart => {
            // Use .restart.* files, recover timestep
            format!(r#"
# Restart from checkpoint
set input          input_files/{prefix}
bincoordinates     $input.restart.coor
binvelocities      $input.restart.vel
extendedSystem     $input.restart.xsc

# Recover timestep from restart file
proc get_first_ts {{ xscfile }} {{
  set fd [open $xscfile r]
  gets $fd
  gets $fd
  gets $fd line
  set ts [lindex $line 0]
  close $fd
  return $ts
}}
set firsttime [get_first_ts $input.restart.xsc]
firsttimestep $firsttime

# NOTE: Do NOT set temperature when restarting
# Velocities are loaded from restart.vel file
"#,
                prefix = restart_prefix
            )
        }
        ContinuationType::NextStage => {
            // Use final coordinates, reset timestep
            format!(r#"
# Continue from previous stage
bincoordinates     input_files/{prefix}.coor
binvelocities      input_files/{prefix}.vel
extendedSystem     input_files/{prefix}.xsc

# Reset timestep for new stage
firsttimestep      0

# Reinitialize velocities at target temperature
temperature        {temperature}
"#,
                prefix = restart_prefix,
                temperature = job_info.namd_config.temperature
            )
        }
    };

    Ok(block)
}

fn generate_fresh_start_block(job_info: &JobInfo) -> String {
    format!(r#"
# Fresh start
temperature        {temperature}
firsttimestep      0
"#,
        temperature = job_info.namd_config.temperature
    )
}
```

### 8.4 Validation Requirements

**Template Validation**:
```rust
pub fn validate_template_configuration(
    template_id: &str,
    field_values: &HashMap<String, serde_json::Value>,
) -> ValidationResult {
    let template = get_template(template_id)
        .ok_or_else(|| anyhow!("Template '{}' not found", template_id))?;

    let mut issues = Vec::new();
    let mut warnings = Vec::new();

    // Validate each field according to template rules
    for field in &template.field_definitions {
        if let Some(value) = field_values.get(&field.key) {
            // Check required
            if field.required && value.is_null() {
                issues.push(format!("{} is required", field.label));
                continue;
            }

            // Run validation rules
            if let Some(validation) = &field.validation {
                for rule in &validation.rules {
                    if let Some(error) = check_validation_rule(rule, value) {
                        issues.push(format!("{}: {}", field.label, error));
                    }
                }
            }
        } else if field.required {
            issues.push(format!("{} is required", field.label));
        }
    }

    // Check interdependent parameters
    check_interdependent_rules(&template, field_values, &mut issues, &mut warnings);

    ValidationResult {
        is_valid: issues.is_empty(),
        issues,
        warnings,
        suggestions: vec![],
    }
}

fn check_interdependent_rules(
    template: &NAMDTemplate,
    values: &HashMap<String, serde_json::Value>,
    issues: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    // Example: PME requires cellBasisVector
    if let Some(use_pme) = values.get("use_pme").and_then(|v| v.as_bool()) {
        if use_pme {
            let has_cell = values.get("cell_basis_vector_x").is_some()
                && values.get("cell_basis_vector_y").is_some()
                && values.get("cell_basis_vector_z").is_some();

            if !has_cell {
                issues.push("Cell basis vectors are required when PME is enabled".to_string());
            }
        }
    }

    // Example: Warn if box is very large with PME
    if let Some(use_pme) = values.get("use_pme").and_then(|v| v.as_bool()) {
        if use_pme {
            if let Some(x) = values.get("cell_basis_vector_x").and_then(|v| v.as_f64()) {
                if x > 500.0 {
                    warnings.push("Large box size (>500Å) with PME may be inefficient. Consider vacuum simulation.".to_string());
                }
            }
        }
    }
}
```

---

## 9. NAMD Config Templates

### 9.1 Tutorial-Derived Templates

Based on analysis of tutorial files in `examples/origamiTutorial/`.

#### Template 1: Vacuum Optimization

**Source**: `step2/hextube.namd`
**Use Case**: Initial structure optimization without solvent

**Key Parameters**:
```tcl
PME                 no           # No periodic electrostatics
cellBasisVector1    1000 0 0     # Large box prevents periodic images
langevinDamping     0.1          # Low friction for fast relaxation
fullElectFrequency  3            # Higher frequency for vacuum
margin              30           # Large pairlist margin
extraBonds          on           # Use extrabonds for structural stability
```

**Template Definition**:
```rust
pub fn vacuum_optimization() -> NAMDTemplate {
    NAMDTemplate {
        id: "vacuum_optimization".to_string(),
        name: "Vacuum Optimization".to_string(),
        description: "Initial structure relaxation in vacuum with extrabonds restraints".to_string(),
        category: TemplateCategory::Optimization,
        use_case: "Use for DNA origami initial structure optimization or when starting from idealized atomic models".to_string(),

        form_sections: vec![
            FormSection {
                title: "Simulation Duration".to_string(),
                fields: vec![
                    FieldDefinition {
                        key: "steps".to_string(),
                        label: "Simulation Steps".to_string(),
                        field_type: FieldType::Number {
                            min: Some(1000.0),
                            max: Some(100000000.0),
                            step: Some(1000.0),
                            unit: Some("steps".to_string()),
                        },
                        default_value: json!(96000000),  // 40 ps at 2fs/step from tutorial
                        required: true,
                        help_text: Some("Number of MD steps. Tutorial uses 96M steps (192 ns at 2fs/step)".to_string()),
                        validation: Some(FieldValidation {
                            rules: vec![
                                ValidationRule::MinValue {
                                    min: 1000.0,
                                    message: "Must run at least 1000 steps".to_string(),
                                },
                            ],
                            custom_validator: None,
                        }),
                        visible_when: None,
                    },
                    FieldDefinition {
                        key: "timestep".to_string(),
                        label: "Timestep (fs)".to_string(),
                        field_type: FieldType::Number {
                            min: Some(0.1),
                            max: Some(4.0),
                            step: Some(0.1),
                            unit: Some("fs".to_string()),
                        },
                        default_value: json!(2.0),
                        required: true,
                        help_text: Some("Integration timestep. 2fs is standard with rigidBonds".to_string()),
                        validation: None,
                        visible_when: None,
                    },
                ],
                description: None,
                collapsible: false,
                collapsed_by_default: false,
            },
            FormSection {
                title: "Periodic Boundaries".to_string(),
                fields: vec![
                    FieldDefinition {
                        key: "use_pme".to_string(),
                        label: "Use PME Electrostatics".to_string(),
                        field_type: FieldType::Boolean,
                        default_value: json!(false),  // NO PME for vacuum
                        required: true,
                        help_text: Some("Disabled for vacuum simulations".to_string()),
                        validation: None,
                        visible_when: None,
                    },
                    FieldDefinition {
                        key: "cell_basis_vector_x".to_string(),
                        label: "Box Size X (Å)".to_string(),
                        field_type: FieldType::Number {
                            min: Some(50.0),
                            max: Some(2000.0),
                            step: Some(10.0),
                            unit: Some("Å".to_string()),
                        },
                        default_value: json!(1000.0),  // Large box
                        required: true,
                        help_text: Some("Large box prevents periodic interactions. Tutorial uses 1000Å".to_string()),
                        validation: None,
                        visible_when: None,
                    },
                    // Y and Z similar...
                ],
                description: Some("Large simulation box to prevent periodic boundary interactions".to_string()),
                collapsible: false,
                collapsed_by_default: false,
            },
            FormSection {
                title: "Temperature Control".to_string(),
                fields: vec![
                    FieldDefinition {
                        key: "temperature".to_string(),
                        label: "Temperature (K)".to_string(),
                        field_type: FieldType::Number {
                            min: Some(200.0),
                            max: Some(400.0),
                            step: Some(10.0),
                            unit: Some("K".to_string()),
                        },
                        default_value: json!(300.0),
                        required: true,
                        help_text: Some("Target temperature for simulation".to_string()),
                        validation: Some(FieldValidation {
                            rules: vec![
                                ValidationRule::Range {
                                    min: 200.0,
                                    max: 400.0,
                                    message: "Temperature must be in biological range (200-400 K)".to_string(),
                                },
                            ],
                            custom_validator: None,
                        }),
                        visible_when: None,
                    },
                    FieldDefinition {
                        key: "langevin_damping".to_string(),
                        label: "Langevin Damping".to_string(),
                        field_type: FieldType::Number {
                            min: Some(0.01),
                            max: Some(10.0),
                            step: Some(0.1),
                            unit: None,
                        },
                        default_value: json!(0.1),  // Low for vacuum
                        required: true,
                        help_text: Some("Friction coefficient. Low value (0.1) for faster relaxation in vacuum".to_string()),
                        validation: None,
                        visible_when: None,
                    },
                ],
                description: Some("Langevin dynamics for temperature control".to_string()),
                collapsible: false,
                collapsed_by_default: false,
            },
            FormSection {
                title: "Advanced Parameters".to_string(),
                fields: vec![
                    FieldDefinition {
                        key: "margin".to_string(),
                        label: "Pairlist Margin".to_string(),
                        field_type: FieldType::Number {
                            min: Some(0.0),
                            max: Some(50.0),
                            step: Some(5.0),
                            unit: Some("Å".to_string()),
                        },
                        default_value: json!(30),  # Large for vacuum
                        required: false,
                        help_text: Some("Extra space for pairlist. Large value (30) for vacuum simulations".to_string()),
                        validation: None,
                        visible_when: Some(VisibilityCondition {
                            field: "use_pme".to_string(),
                            operator: ConditionOperator::Equals,
                            value: json!(false),
                        }),
                    },
                    FieldDefinition {
                        key: "full_elect_frequency".to_string(),
                        label: "Full Electrostatics Frequency".to_string(),
                        field_type: FieldType::Number {
                            min: Some(1.0),
                            max: Some(10.0),
                            step: Some(1.0),
                            unit: Some("steps".to_string()),
                        },
                        default_value: json!(3),
                        required: false,
                        help_text: Some("How often to compute full electrostatics. Tutorial uses 3 for vacuum".to_string()),
                        validation: None,
                        visible_when: None,
                    },
                ],
                description: Some("Advanced simulation parameters (usually don't need to change)".to_string()),
                collapsible: true,
                collapsed_by_default: true,
            },
        ],

        default_values: {
            let mut defaults = HashMap::new();
            defaults.insert("steps".to_string(), json!(96000000));
            defaults.insert("temperature".to_string(), json!(300.0));
            defaults.insert("timestep".to_string(), json!(2.0));
            defaults.insert("use_pme".to_string(), json!(false));
            defaults.insert("cell_basis_vector_x".to_string(), json!(1000.0));
            defaults.insert("cell_basis_vector_y".to_string(), json!(1000.0));
            defaults.insert("cell_basis_vector_z".to_string(), json!(1000.0));
            defaults.insert("langevin_damping".to_string(), json!(0.1));
            defaults.insert("margin".to_string(), json!(30));
            defaults.insert("full_elect_frequency".to_string(), json!(3));
            defaults.insert("outputname".to_string(), json!("output"));
            defaults.insert("dcd_freq".to_string(), json!(9600));
            defaults.insert("restart_freq".to_string(), json!(9600));
            defaults
        },
    }
}
```

#### Template 2: Explicit Solvent NPT

**Source**: `step3/equil_k0.5.namd`
**Use Case**: Equilibration with pressure control in explicit solvent

**Key Parameters**:
```tcl
PME                 yes          # Periodic electrostatics required
PMEGridSpacing      1.5
cellBasisVector1    124 0.0 0.0  # Actual system size from solvation
langevinDamping     5            # Higher friction in solvent
langevinPiston      on           # NPT ensemble with pressure control
fullElectFrequency  2
extraBonds          on           # ENM restraints + MGHH restraints
```

**Template Definition**: (Similar structure to vacuum, key differences below)

```rust
pub fn explicit_solvent_npt() -> NAMDTemplate {
    NAMDTemplate {
        // ... metadata ...

        form_sections: vec![
            // Duration section (same)
            FormSection {
                title: "Periodic Boundaries".to_string(),
                fields: vec![
                    FieldDefinition {
                        key: "use_pme".to_string(),
                        default_value: json!(true),  // REQUIRED for solvent
                        help_text: Some("Required for explicit solvent simulations".to_string()),
                        // Could make this field read-only/disabled in UI
                    },
                    FieldDefinition {
                        key: "pme_grid_spacing".to_string(),
                        label: "PME Grid Spacing (Å)".to_string(),
                        field_type: FieldType::Number {
                            min: Some(0.8),
                            max: Some(2.0),
                            step: Some(0.1),
                            unit: Some("Å".to_string()),
                        },
                        default_value: json!(1.5),
                        required: true,
                        help_text: Some("Grid spacing for PME. Tutorial uses 1.5Å".to_string()),
                        visible_when: Some(VisibilityCondition {
                            field: "use_pme".to_string(),
                            operator: ConditionOperator::Equals,
                            value: json!(true),
                        }),
                    },
                    FieldDefinition {
                        key: "cell_basis_vector_x".to_string(),
                        default_value: json!(null),  // Must be provided/detected
                        required: true,
                        help_text: Some("System size from solvation box. Can be auto-detected from PDB CRYST1 record".to_string()),
                        validation: Some(FieldValidation {
                            rules: vec![
                                ValidationRule::RequiredIf {
                                    field: "use_pme".to_string(),
                                    equals: json!(true),
                                    message: "Cell basis vectors required when PME enabled".to_string(),
                                },
                            ],
                            custom_validator: None,
                        }),
                    },
                    // Y, Z similar...
                ],
            },
            FormSection {
                title: "Temperature & Pressure Control".to_string(),
                fields: vec![
                    FieldDefinition {
                        key: "temperature".to_string(),
                        default_value: json!(300.0),
                        // ... same as vacuum ...
                    },
                    FieldDefinition {
                        key: "langevin_damping".to_string(),
                        default_value: json!(5.0),  // HIGHER for solvent
                        help_text: Some("Friction coefficient. Higher value (5.0) for explicit solvent".to_string()),
                    },
                    FieldDefinition {
                        key: "pressure_control".to_string(),
                        label: "Enable Pressure Control (NPT)".to_string(),
                        field_type: FieldType::Boolean,
                        default_value: json!(true),
                        required: true,
                        help_text: Some("NPT ensemble with constant pressure. Disable for NVT (constant volume)".to_string()),
                    },
                    FieldDefinition {
                        key: "target_pressure".to_string(),
                        label: "Target Pressure (bar)".to_string(),
                        field_type: FieldType::Number {
                            min: Some(0.5),
                            max: Some(2.0),
                            step: Some(0.01),
                            unit: Some("bar".to_string()),
                        },
                        default_value: json!(1.01325),  // 1 atm
                        required: false,
                        help_text: Some("Target pressure for NPT ensemble. 1.01325 bar = 1 atm".to_string()),
                        visible_when: Some(VisibilityCondition {
                            field: "pressure_control".to_string(),
                            operator: ConditionOperator::Equals,
                            value: json!(true),
                        }),
                    },
                ],
            },
            // Advanced section with full_elect_frequency=2, no margin field
        ],
    }
}
```

### 9.2 Template Parameters Catalog

**Complete parameter mapping** from tutorial to NAMDRunner templates:

| Parameter | Vacuum Opt | Explicit NPT | Minimization | Production | Notes |
|-----------|------------|--------------|--------------|------------|-------|
| **PME** | no | yes | yes | yes | Periodic electrostatics |
| **PMEGridSpacing** | - | 1.5 | 1.5 | 1.5 | Only when PME=yes |
| **cellBasisVector** | 1000x1000x1000 | Actual size | Actual size | Actual size | Large box for vacuum |
| **temperature** | 300 | 300 | 300 | 310 | Can vary |
| **timestep** | 2.0 | 2.0 | 2.0 | 2.0 | Standard with rigidBonds |
| **langevinDamping** | 0.1 | 5 | 5 | 5 | Low for vacuum, high for solvent |
| **langevinPiston** | off | on | on | on | Pressure control for NPT |
| **margin** | 30 | - | - | - | Only for vacuum |
| **fullElectFrequency** | 3 | 2 | 2 | 2 | Higher for vacuum |
| **minimize** | 0 | 4800 | 4800 | 0 | Minimization before dynamics |
| **extraBonds** | yes (.exb) | yes (ENM+MGHH) | no | no | Restraints for structure |
| **wrapAll/wrapWater** | off/off | off/off | off/off | off/off | Tutorial keeps unwrapped |
| **outputname** | hextube | equil_k0.5 | equil_min | production | Varies by stage |
| **dcd_freq** | 4800 | 9600 | 1200 | 5000 | Output frequency varies |
| **restart_freq** | 48000 | 9600 | 1200 | 10000 | Checkpoint frequency |

**Key Insights**:
1. Vacuum vs solvent has **different parameter sets**
2. Damping coefficient changes 50x (0.1 → 5.0)
3. PME + cellBasisVector are coupled
4. Extrabonds usage varies by stage
5. Output frequencies vary widely

### 9.3 Restart Configuration Blocks

**From Tutorial** (`step3/equil_k0.5.namd` lines 17-39):

```tcl
set input          equil_min
bincoordinates     $input.coor
binvelocities      $input.vel
extendedSystem     $input.xsc

# NOTE: Do not set the initial velocity temperature if you
# have also specified a .vel restart file!
set temperature    300
#temperature         $temperature

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
#set firsttime      0
firsttimestep $firsttime
```

**Critical Pattern**:
1. **Load binary files**: `bincoordinates`, `binvelocities`, `extendedSystem`
2. **Do NOT set temperature**: Commented out to preserve velocities from restart file
3. **Recover timestep**: Read from `.xsc` file's third line, first column
4. **Set firsttimestep**: Continues time series from previous run

**NAMDRunner Implementation** (in script generator):

```rust
// Checkpoint restart block
let restart_block = format!(r#"
# Restart from checkpoint
set input          input_files/{prefix}
bincoordinates     $input.restart.coor
binvelocities      $input.restart.vel
extendedSystem     $input.restart.xsc

# Recover timestep from restart file
proc get_first_ts {{ xscfile }} {{
  set fd [open $xscfile r]
  gets $fd
  gets $fd
  gets $fd line
  set ts [lindex $line 0]
  close $fd
  return $ts
}}
set firsttime [get_first_ts $input.restart.xsc]
firsttimestep $firsttime

# NOTE: Do NOT set temperature when restarting
# Velocities are loaded from restart.vel file
"#, prefix = restart_prefix);
```

**Next Stage Block** (different):
```rust
// Next stage continuation (uses final coords, not restart)
let continuation_block = format!(r#"
# Continue from previous stage
bincoordinates     input_files/{prefix}.coor
binvelocities      input_files/{prefix}.vel
extendedSystem     input_files/{prefix}.xsc

# Reset timestep for new stage
firsttimestep      0

# Reinitialize velocities at new temperature
temperature        {temperature}
"#, prefix = restart_prefix, temperature = namd_config.temperature);
```

---

## 10. Tutorial Reference Guide

### 10.1 Essential Sections to Read

**Location**: `examples/origamiTutorial/origamiprotocols_0_markdown.md` or `.pdf`

**Reading Order** (Discovery Process):

1. **Section 1: Introduction** (lines 1-111)
   - Understand DNA origami method
   - Why MD simulations are needed
   - Overview of tutorial scope

2. **Section 3.2: Building an Optimized Atomistic Structure** (lines 265-425)
   - **START HERE for hands-on understanding**
   - Download hextube.tar.gz from ENRG MD web server
   - Understand vacuum optimization workflow
   - **Key file**: `step2/hextube.namd` (lines 1-92)
   - Note use of extrabonds for structural stability (lines 71-72)
   - Note PME=no for vacuum (line 49)
   - Note large cellBasisVector 1000Å (lines 79-81)

3. **Section 3.3: MD Simulations in Explicit MgCl₂ Solution** (lines 426-609)
   - Solvation process with Mg²⁺ ions
   - Multi-stage equilibration protocol
   - **Key files**:
     - `step3/equil_min.namd` - Minimization (lines 1-118)
     - `step3/equil_k0.5.namd` - Strong restraints (lines 1-128)
     - Compare these to see restart mechanism
   - Progressive restraint reduction (k=0.5 → 0.1 → 0.01 → 0)
   - Note PME=yes for explicit solvent (line 66 of equil_min.namd)
   - Note restart block (lines 17-39 of equil_k0.5.namd)

4. **Section 4: Notes** (lines 820-935)
   - Common errors and troubleshooting
   - Minimization requirements
   - NPT ensemble considerations
   - Water bubble issues

### 10.2 Critical Files to Examine

**File Tree**:
```
examples/origamiTutorial/
├── origamiprotocols_0_markdown.md    # Main guide
├── origamiprotocols_0.pdf            # PDF version
├── step1/                            # caDNAno design (not critical for NAMDRunner)
├── step2/
│   ├── hextube.namd                  # VACUUM OPTIMIZATION - Read this!
│   ├── hextube.psf
│   ├── hextube.pdb
│   └── hextube.exb                   # Extrabonds file
├── step3/
│   ├── equil_min.namd                # MINIMIZATION - Read this!
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

**Critical Comparison**:

| Aspect | step2/hextube.namd | step3/equil_min.namd | step3/equil_k0.5.namd |
|--------|-------------------|---------------------|----------------------|
| **Simulation Type** | Vacuum optimization | Minimization in solvent | Equilibration with restraints |
| **PME** | no (line 49) | yes (line 66) | yes (line 72) |
| **Cell Basis** | 1000Å (lines 79-81) | 124x114x323Å (lines 35-37) | 124x114x323Å (lines 41-43) |
| **Langevin Damping** | 0.1 (line 55) | 5 (line 76) | 5 (line 82) |
| **Pressure Control** | no | yes (lines 81-87) | yes (lines 87-93) |
| **Extrabonds** | yes (.exb, lines 71-72) | yes (ENM+MGHH, lines 104-107) | yes (ENM+MGHH, lines 114-117) |
| **Minimize Steps** | 4800 (line 85) | 4800 (line 113) | 0 (commented, line 123) |
| **Run Steps** | 96M (line 91) | 0 (commented, line 114) | 2.4M (line 124) |
| **Restart From** | none (fresh) | none (fresh) | equil_min (lines 17-39) |
| **Temperature Set** | yes (line 84) | yes (line 20) | NO (commented, line 26) |
| **Timestep Recovery** | no | no | YES (lines 28-39) |

**Key Insight from Comparison**: Notice how `equil_k0.5.namd` differs from `equil_min.namd`:
- Loads restart files (lines 17-21)
- Does NOT set temperature (line 26 commented)
- Recovers timestep from .xsc file (lines 28-39)
- This is the **canonical restart pattern**

### 10.3 Key Insights from Tutorial

**1. PME Requires Periodic Boundaries**
- Error we encountered: "FATAL ERROR: PME requires periodic boundary conditions"
- Solution: Always set `cellBasisVector1/2/3` when `PME yes`
- Vacuum simulations use `PME no` + large box (1000Å) to avoid periodic images

**2. Different Damping for Vacuum vs Solvent**
- Vacuum: `langevinDamping 0.1` - Low friction for fast relaxation
- Solvent: `langevinDamping 5` - Higher friction matches viscosity
- 50x difference! Not arbitrary.

**3. Restart Files Naming Convention**
- NAMD generates: `{outputName}.restart.{coor,vel,xsc}`
- Also generates: `{outputName}.{coor,vel,xsc}` (final coordinates)
- Restart from checkpoint: Use `.restart.*` files
- Start new stage: Use final `.{coor,vel,xsc}` files

**4. Timestep Recovery is Critical**
- XSC file format: 3 header lines, then data
- Third line, first column = timestep
- Must read and set `firsttimestep` when restarting
- Ensures continuous time series across stages

**5. Temperature Initialization Rules**
- Fresh start: Set `temperature $temp` to initialize velocities
- Restart: Do NOT set temperature (velocities from .vel file)
- Next stage: Set temperature to reinitialize at new value
- Tutorial comments clearly indicate this (line 23-24 of equil_k0.5.namd)

**6. Progressive Restraint Reduction**
- DNA origami needs gradual equilibration
- Strong restraints (k=0.5) → Medium (k=0.1) → Weak (k=0.01) → None (k=0)
- Each stage runs ~4.8 ns (2.4M steps at 2fs)
- Without this, structure breaks during equilibration

**7. Extrabonds Serve Two Purposes**
- DNA-DNA repulsion (imitate explicit solvent in vacuum)
- Structural stability (enforce base pairing, crossover integrity)
- Different extrabonds files for different stages
- ENM = Elastic Network Model restraints

**8. Output Naming Matters**
- Each stage has unique `outputName` (equil_min, equil_k0.5, etc.)
- Enables easy file discovery for next stage
- NAMDRunner should enforce simple names (no paths, no special chars)

---

## 11. Open Questions & Future Decisions

### 11.1 User-Created Templates

**Goal**: Allow users to save custom configurations as templates for reuse.

**Open Questions**:

**Q1: Template Storage**
- Store in SQLite (local only)?
- Upload to cluster (share across machines)?
- Both with sync?

**Q2: Template Scope**
- User-local templates only?
- Lab/team templates (shared folder)?
- Public template repository?

**Q3: Template Editing**
- Edit existing templates?
- Or always create new from current settings?
- Template versioning (probably NOT needed per requirements)?

**Q4: Template Export/Import**
- JSON export for sharing?
- Import from file?
- Template validation on import?

**Proposed Approach** (Phase 1):
1. **Create from current job**: "Save as Template" button on job detail page
2. **Store in SQLite**: Local templates table
3. **User-local only**: No sharing in Phase 1
4. **No editing**: Can delete and recreate
5. **JSON export/import**: For manual sharing

**Schema**:
```sql
CREATE TABLE user_templates (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    category TEXT,
    created_at TEXT,
    template_data TEXT NOT NULL  -- JSON serialized NAMDTemplate
);
```

### 11.2 Naming Conventions for Continued Jobs

**Problem**: What should child jobs be named automatically?

**Options**:

**A: Suffix numbering**
- Parent: `DNA_Minimization`
- Child 1: `DNA_Minimization_2`
- Child 2: `DNA_Minimization_3`
- Pro: Simple, clear sequence
- Con: Doesn't indicate purpose (restart vs next stage)

**B: Continuation suffix**
- Parent: `DNA_Minimization`
- Checkpoint: `DNA_Minimization_cont`
- Next stage: `DNA_Minimization_k0.5`
- Pro: Purpose is clear
- Con: User must provide stage name

**C: Timestamp suffix**
- Parent: `DNA_Minimization`
- Child: `DNA_Minimization_20250118`
- Pro: Always unique
- Con: Not human-readable

**D: User prompt**
- Always ask user to name child job
- Pre-fill suggestion based on type
- Pro: Maximum control
- Con: Extra step

**Proposed**: **Option B with fallback to A**
- Checkpoint restart: Auto-suggest `{parent}_cont`
- Next stage: Prompt user for stage name, suggest based on parent
- User can always override

### 11.3 Template Field Validation - Custom Validators

**Problem**: Some validations are too complex for simple rules.

**Example**: Validate that cellBasisVector dimensions match uploaded PDB file

**Proposed Solution**:
```rust
pub enum ValidationRule {
    // ... existing rules ...

    CustomValidator {
        function_name: String,
        message: String,
    },
}

// Registry of custom validator functions
type CustomValidatorFn = fn(&serde_json::Value, &HashMap<String, serde_json::Value>) -> Option<String>;

lazy_static! {
    static ref CUSTOM_VALIDATORS: HashMap<String, CustomValidatorFn> = {
        let mut validators = HashMap::new();
        validators.insert("validate_box_vs_pdb".to_string(), validate_box_vs_pdb as CustomValidatorFn);
        validators.insert("validate_pme_grid_dimensions".to_string(), validate_pme_grid_dimensions as CustomValidatorFn);
        validators
    };
}

fn validate_box_vs_pdb(
    value: &serde_json::Value,
    all_values: &HashMap<String, serde_json::Value>
) -> Option<String> {
    // Load PDB, check CRYST1 record
    // Compare with cellBasisVector values
    // Return error message if mismatch
    None  // or Some("error message")
}
```

**Usage in Template**:
```rust
FieldDefinition {
    key: "cell_basis_vector_x",
    validation: Some(FieldValidation {
        rules: vec![
            ValidationRule::CustomValidator {
                function_name: "validate_box_vs_pdb".to_string(),
                message: "Box dimensions don't match PDB file".to_string(),
            },
        ],
    }),
}
```

**Open Question**: Is this overengineering? Start with simple rules only?

---

## 12. Integration with Existing Architecture

### 12.1 Automation System Integration

**Current System** (from `AUTOMATIONS.md`):
- Job Creation Automation: `execute_job_creation_with_progress()`
- Job Submission Automation: `execute_job_submission_with_progress()`
- Status Sync: `sync_job_status()`
- Job Completion: `execute_job_completion_with_progress()`

**Integration Points**:

**1. Job Creation Enhancement**:
```rust
// src-tauri/src/automations/job_creation.rs

pub async fn execute_job_creation_with_progress(
    app_handle: AppHandle,
    params: CreateJobParams,  // NOW includes parent_job_id, continuation_type, template_id
    progress_callback: impl Fn(&str),
) -> Result<(String, JobInfo)> {
    // ... existing validation ...

    // NEW: If continuation, copy parent files
    if let Some(parent_id) = &params.parent_job_id {
        progress_callback("Loading parent job...");
        let parent = with_database(|db| db.load_job(parent_id))?;

        progress_callback("Copying files from parent job...");
        let copied_files = copy_parent_files_to_child(
            &connection_manager,
            &parent,
            &project_dir,
            &params.continuation_type.unwrap(),
            &params.restart_output_prefix.unwrap_or("output".to_string()),
        ).await?;

        input_files_with_metadata.extend(copied_files);
    }

    // ... rest of existing automation ...
}
```

**2. Progress Events**:
```rust
// NEW events to emit:
progress_callback("Validating parent job for continuation...");
progress_callback("Copying original input files from parent...");
progress_callback("Copying restart files from parent...");
progress_callback("Generating restart configuration...");
```

**3. Validation Integration**:
```rust
// Extend existing validation in job_creation.rs
if let Some(parent_id) = &params.parent_job_id {
    validate_parent_for_continuation(&parent, &params.continuation_type)?;
}

// Template validation before config generation
validate_template_configuration(
    &params.template_id,
    &namd_config_as_map,
)?;
```

### 12.2 File Operations Integration

**Research Summary** (from subagent investigation):

**Existing Capabilities**:
1. ✅ **Cluster-to-cluster copy**: `sync_directory_rsync()` in `manager.rs` (lines 390-424)
   ```rust
   pub async fn sync_directory_rsync(
       &self,
       source: &str,
       destination: &str,
   ) -> Result<()>
   ```
   - Uses SSH `rsync -az` command (cluster-side, no local download)
   - Already used for job submission (project→scratch) and completion (scratch→project)

2. ✅ **File metadata tracking**: `list_files_with_metadata()` in `sftp.rs` (lines 283-320)
   - Returns `Vec<OutputFile>` with name, size, modified_at
   - Used in job completion automation

3. ✅ **Individual file upload**: `upload_file()` in `sftp.rs` (lines 98-190)
   - With progress tracking
   - Used in job creation for user file uploads

**Integration for Job Continuation**:
```rust
// NEW utility function in job_creation.rs or separate module

async fn copy_parent_files_to_child(
    connection_manager: &ConnectionManager,
    parent_job: &JobInfo,
    child_project_dir: &str,
    continuation_type: ContinuationType,
    restart_prefix: &str,
) -> Result<Vec<InputFile>> {
    let parent_project = parent_job.project_dir.as_ref().unwrap();
    let child_input = format!("{}/input_files", child_project_dir);

    // Copy original inputs using rsync (entire directory)
    connection_manager.sync_directory_rsync(
        &format!("{}/input_files/", parent_project),  // Trailing / = contents
        &child_input,
    ).await?;

    // Copy specific restart/continuation files
    let parent_outputs = format!("{}/outputs", parent_project);
    let files_to_copy = match continuation_type {
        ContinuationType::CheckpointRestart => {
            vec![
                format!("{}.restart.coor", restart_prefix),
                format!("{}.restart.vel", restart_prefix),
                format!("{}.restart.xsc", restart_prefix),
            ]
        }
        ContinuationType::NextStage => {
            vec![
                format!("{}.coor", restart_prefix),
                format!("{}.vel", restart_prefix),
                format!("{}.xsc", restart_prefix),
            ]
        }
    };

    // Copy individual files using shell cp
    for filename in &files_to_copy {
        let src = format!("{}/{}", parent_outputs, filename);
        let dst = format!("{}/{}", child_input, filename);
        let cp_cmd = shell::safe_cp(&src, &dst);
        connection_manager.execute_command(&cp_cmd, Some(timeouts::FILE_COPY)).await?;
    }

    // Build InputFile metadata from parent
    build_input_file_metadata(parent_job, &files_to_copy)
}
```

**Key Integration Points**:
- Uses existing `sync_directory_rsync()` - no new SFTP operations needed
- Uses existing `shell::safe_cp()` for individual files
- Metadata copied from parent OutputFile to child InputFile
- No local download required (all cluster-side)

### 12.3 Database Schema Additions

**Current Schema** (`src-tauri/src/database/mod.rs`):
```sql
CREATE TABLE IF NOT EXISTS jobs (
    job_id TEXT PRIMARY KEY,
    data TEXT NOT NULL  -- JSON blob of JobInfo
);
```

**No Schema Changes Needed!** Document-store pattern means new fields in `JobInfo` automatically stored in JSON blob.

**New Fields in JobInfo** (from Section 5.1):
```rust
pub struct JobInfo {
    // ... all existing fields ...

    pub template_id: String,
    pub parent_job_id: Option<String>,
    pub continuation_type: Option<ContinuationType>,
    pub root_job_id: Option<String>,
    pub chain_depth: u32,
}
```

**Query Considerations**:

SQLite can query JSON fields:
```sql
-- Find all children of a job
SELECT * FROM jobs
WHERE json_extract(data, '$.parent_job_id') = 'job_parent_123';

-- Find all jobs in a chain (root)
SELECT * FROM jobs
WHERE json_extract(data, '$.root_job_id') = 'job_root_001'
ORDER BY json_extract(data, '$.chain_depth');

-- Find all jobs using a template
SELECT * FROM jobs
WHERE json_extract(data, '$.template_id') = 'vacuum_optimization';
```

**Index Creation** (optional, for performance):
```sql
CREATE INDEX IF NOT EXISTS idx_parent_job
ON jobs(json_extract(data, '$.parent_job_id'));

CREATE INDEX IF NOT EXISTS idx_template
ON jobs(json_extract(data, '$.template_id'));
```

### 12.4 UI Components Architecture

**Research Summary** (from subagent investigation):

**Current Dynamic Configuration Pattern** (ClusterConfig):
1. **Backend provides data**: `get_cluster_capabilities()` returns `ClusterCapabilities` struct
2. **Frontend store caches**: `clusterConfig.ts` writable store initialized at app start
3. **Derived stores filter**: `partitions`, `qosOptions` derived from main store
4. **Components iterate**: `PartitionSelector.svelte` iterates `$partitions` array
5. **Dynamic rendering**: Cards/options generated from backend data

**Template System Integration** (follows same pattern):

**1. Backend Template Provider**:
```rust
// src-tauri/src/commands/templates.rs
#[tauri::command]
pub fn get_template_capabilities() -> ApiResult<TemplateCapabilities> {
    let templates = crate::templates::get_all_templates();
    ApiResult::success(TemplateCapabilities {
        templates: templates.into_iter().cloned().collect(),
    })
}
```

**2. Frontend Template Store**:
```typescript
// src/lib/stores/templateConfig.ts
import { writable, derived } from 'svelte/store';
import type { TemplateCapabilities, NAMDTemplate } from '../types/template';

const templateConfigStore = writable<TemplateCapabilities | null>(null);

export async function initTemplateConfig(): Promise<void> {
  const result = await CoreClientFactory.getClient().getTemplateCapabilities();
  if (result.success && result.data) {
    templateConfigStore.set(result.data);
  }
}

export const availableTemplates = derived(
  templateConfigStore,
  $config => $config?.templates ?? []
);

export const getTemplate = derived(
  templateConfigStore,
  $config => (templateId: string) =>
    $config?.templates.find(t => t.id === templateId)
);
```

**3. Dynamic Form Component** (NEW):
```svelte
<!-- src/lib/components/create-job/DynamicConfigForm.svelte -->
<script lang="ts">
  import { availableTemplates, getTemplate } from '../../stores/templateConfig';
  import DynamicField from './DynamicField.svelte';

  export let selectedTemplateId: string;
  export let fieldValues: Record<string, any> = {};

  $: template = $getTemplate(selectedTemplateId);
  $: if (template) {
    initializeDefaultValues(template);
  }

  function initializeDefaultValues(template: NAMDTemplate) {
    fieldValues = { ...template.default_values, ...fieldValues };
  }

  function shouldShowField(field: FieldDefinition): boolean {
    if (!field.visible_when) return true;
    const { field: depField, operator, value } = field.visible_when;
    return evaluateCondition(fieldValues[depField], operator, value);
  }
</script>

{#if template}
  <div class="dynamic-form">
    {#each template.form_sections as section}
      <fieldset>
        <legend>{section.title}</legend>
        {#if section.description}
          <p class="description">{section.description}</p>
        {/if}

        {#each section.fields as field}
          {#if shouldShowField(field)}
            <DynamicField
              definition={field}
              bind:value={fieldValues[field.key]}
            />
          {/if}
        {/each}
      </fieldset>
    {/each}
  </div>
{/if}
```

**4. Field Renderer Component** (NEW):
```svelte
<!-- src/lib/components/create-job/DynamicField.svelte -->
<script lang="ts">
  import type { FieldDefinition } from '../../types/template';

  export let definition: FieldDefinition;
  export let value: any;

  $: fieldType = definition.field_type;
</script>

<div class="field-group">
  <label for={definition.key}>
    {definition.label}
    {#if definition.required}*{/if}
  </label>

  {#if fieldType.type === 'Boolean'}
    <input
      type="checkbox"
      id={definition.key}
      bind:checked={value}
    />
  {:else if fieldType.type === 'Number'}
    <input
      type="number"
      id={definition.key}
      bind:value
      min={fieldType.min}
      max={fieldType.max}
      step={fieldType.step}
    />
    {#if fieldType.unit}
      <span class="unit">{fieldType.unit}</span>
    {/if}
  {:else if fieldType.type === 'Select'}
    <select id={definition.key} bind:value>
      {#each fieldType.options as option}
        <option value={option.value}>{option.label}</option>
      {/each}
    </select>
  {:else if fieldType.type === 'Dimensions'}
    <div class="dimensions-grid">
      <input type="number" bind:value={value.x} placeholder="X (Å)" />
      <input type="number" bind:value={value.y} placeholder="Y (Å)" />
      <input type="number" bind:value={value.z} placeholder="Z (Å)" />
    </div>
  {/if}

  {#if definition.help_text}
    <span class="help-text">{definition.help_text}</span>
  {/if}
</div>
```

**Integration into CreateJobPage**:
```svelte
<!-- Modify src/lib/components/pages/CreateJobPage.svelte -->

<script>
  import DynamicConfigForm from '../create-job/DynamicConfigForm.svelte';

  let selectedTemplateId = 'explicit_solvent_npt';
  let namdFieldValues = {};

  // When form submitted, merge template values with CreateJobParams
  function buildCreateJobParams() {
    return {
      job_name,
      template_id: selectedTemplateId,
      namd_config: buildNAMDConfigFromFields(namdFieldValues),
      slurm_config,
      input_files,
      // continuation fields if applicable
    };
  }
</script>

<!-- Configuration Tab -->
<DynamicConfigForm
  bind:selectedTemplateId
  bind:fieldValues={namdFieldValues}
/>
```

**Component Reuse**: Same `DynamicConfigForm` used for:
- Fresh job creation
- Job continuation (pre-filled with parent values)
- User template creation (save current form as template)

---

## 13. Success Criteria

**This feature will be considered successful when**:

### Functional Requirements
1. ✅ **Users can create multi-stage DNA origami workflows** without command line
2. ✅ **Jobs can be continued after timeout/failure** without losing progress
3. ✅ **Template system guides correct parameter selection** (vacuum vs solvent)
4. ✅ **Job chains are visualized** clearly in UI (parent-child relationships)
5. ✅ **Files are correctly copied** between parent and child jobs
6. ✅ **Restart blocks preserve timestep continuity** across stages
7. ✅ **Users can save custom templates** for reuse

### Technical Requirements
1. ✅ **Each job is self-contained** (survives parent deletion)
2. ✅ **No cluster-side file references** between jobs (copied, not linked)
3. ✅ **Template validation prevents invalid configurations** (PME without cellBasisVector)
4. ✅ **Backend validation is authoritative** (frontend shows errors from backend)
5. ✅ **Metadata accurately tracks** template_id, parent_job_id, continuation_type
6. ✅ **Database queries can find chains** (all children of parent, all jobs in tree)

### User Experience Requirements
1. ✅ **"Continue Job" button provides clear options** (checkpoint vs next stage)
2. ✅ **Form is pre-filled from parent** when continuing
3. ✅ **Template descriptions guide users** to correct simulation type
4. ✅ **File copying progress is visible** during job creation
5. ✅ **Job chains are visually distinct** in job list (indentation/icons)
6. ✅ **Error messages are actionable** ("PME requires box dimensions" not "Config invalid")

### Documentation Requirements
1. ✅ **Tutorial reference integrated** into template help text
2. ✅ **Parameter explanations** included in field descriptions
3. ✅ **Common workflows documented** (DNA origami 5-stage equilibration)
4. ✅ **Troubleshooting guide** for restart issues

### Testing Requirements
1. ✅ **Full 5-stage DNA origami workflow** runs successfully
2. ✅ **Checkpoint restart** preserves timestep and velocities
3. ✅ **Next stage** correctly reinitializes parameters
4. ✅ **Parent deletion** doesn't break child jobs
5. ✅ **Template validation** catches PME/box mismatches
6. ✅ **User templates** can be created, saved, and reused

---

## 14. Appendices

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
│   ├── equil_k0.5.namd                  # ⭐ CRITICAL: Restart example
│   ├── equil_k0.1.namd                  # Medium restraints
│   ├── equil_k0.01.namd                 # Weak restraints
│   ├── equil_k0.namd                    # Production (no restraints)
│   ├── hextube_MGHH_WI.psf              # Solvated structure
│   ├── hextube_MGHH_WI.pdb
│   ├── hextube_MGHH_WI_k0.5.enm.extra   # ENM restraints (k=0.5)
│   ├── hextube_MGHH_WI_k0.1.enm.extra   # ENM restraints (k=0.1)
│   ├── hextube_MGHH_WI_k0.01.enm.extra  # ENM restraints (k=0.01)
│   ├── mghh_extrabonds                  # Mg hexahydrate restraints
│   ├── mk_extra.sh                      # Script to generate ENM files
│   ├── par_all36_na.prm
│   ├── par_water_ions_cufix.prm
│   ├── add_mgh_ver2_3.tcl               # Add Mg ions script
│   ├── solIon.tcl                       # Solvation script
│   ├── mghh_extrabonds.pl               # Generate Mg restraints
│   └── save_pdb.tcl
│
└── step4/                                # Analysis scripts
    ├── grepBoxTrace.sh
    ├── measureRMSD.sh
    ├── countBrokenBps.tcl
    ├── AlignWrap.tcl
    ├── measureCharge.sh
    ├── writePDB.tcl
    └── pdb2chickenwire.pl
```

**Key Files with Line References**:

| File | Critical Lines | What to Learn |
|------|---------------|---------------|
| `step2/hextube.namd` | 49 (PME no), 55 (damping 0.1), 79-81 (large box), 71-72 (extrabonds) | Vacuum simulation pattern |
| `step3/equil_min.namd` | 66 (PME yes), 76 (damping 5), 35-37 (box size), 81-87 (pressure) | Minimization in solvent |
| `step3/equil_k0.5.namd` | 17-21 (load restart), 26 (NO temperature!), 28-39 (timestep recovery) | ⭐ Restart mechanism |
| `origamiprotocols_0_markdown.md` | 265-425 (Section 3.2), 426-609 (Section 3.3) | Workflow understanding |

### Appendix B: NAMD Parameter Reference

**Complete parameter table** from tutorial analysis:

| Parameter | Type | Range | Default | Tutorial Vacuum | Tutorial Solvent | Notes |
|-----------|------|-------|---------|----------------|------------------|-------|
| **Structure & Topology** |
| structure | file | .psf | required | hextube.psf | hextube_MGHH_WI.psf | Molecular topology |
| coordinates | file | .pdb | required | hextube.pdb | hextube_MGHH_WI.pdb | Atomic positions |
| **Force Field** |
| paraTypeCharmm | bool | - | on | on | on | Use CHARMM format |
| parameters | file | .prm | required | par_all36_na.prm | par_all36_na.prm | Force field params |
| exclude | enum | - | scaled1-4 | scaled1-4 | scaled1-4 | Exclusion rule |
| 1-4scaling | float | 0-1 | 1.0 | 1.0 | 1.0 | 1-4 interaction scale |
| switching | bool | - | on | on | on | Smoothing at cutoff |
| switchdist | float | Å | 8 | 8 | 8 | Start smoothing |
| cutoff | float | Å | 10 | 10 | 10 | Interaction cutoff |
| pairlistdist | float | Å | 12 | 12 | 12 | Pairlist distance |
| margin | int | Å | - | 30 | - | Pairlist extra space |
| **Periodic Boundaries** |
| PME | bool | - | no | **no** | **yes** | Periodic electrostatics |
| PMEGridSpacing | float | Å | 1.0 | - | 1.5 | PME grid spacing |
| cellBasisVector1 | vector | Å | required | 1000 0 0 | 124 0 0 | Box dimension X |
| cellBasisVector2 | vector | Å | required | 0 1000 0 | 0 114 0 | Box dimension Y |
| cellBasisVector3 | vector | Å | required | 0 0 1000 | 0 0 323 | Box dimension Z |
| **Integration** |
| timestep | float | fs | 1.0 | 2.0 | 2.0 | Integration timestep |
| rigidBonds | enum | - | none | all | all | Constrain bonds |
| nonbondedFreq | int | steps | 1 | 1 | 1 | Nonbonded frequency |
| fullElectFrequency | int | steps | 2 | **3** | **2** | Full electrostatics |
| stepspercycle | int | steps | 10 | 12 | 12 | Steps per cycle |
| **Temperature Control** |
| temperature | float | K | 300 | 300 | 300 | Initial temperature |
| langevin | bool | - | off | on | on | Langevin dynamics |
| langevinDamping | float | 1/ps | 5 | **0.1** | **5** | Friction coefficient |
| langevinTemp | float | K | 300 | 300 | 300 | Target temperature |
| langevinHydrogen | bool | - | off | off | off | Couple hydrogens |
| **Pressure Control** |
| langevinPiston | bool | - | off | **off** | **on** | NPT ensemble |
| langevinPistonTarget | float | bar | 1.01325 | - | 1.01325 | Target pressure |
| langevinPistonPeriod | float | fs | 100 | - | 1000 | Oscillation period |
| langevinPistonDecay | float | fs | 50 | - | 500 | Damping timescale |
| langevinPistonTemp | float | K | 300 | - | 300 | Piston temperature |
| **Output** |
| outputName | string | - | output | hextube | equil_k0.5 | Output file prefix |
| binaryoutput | bool | - | yes | - | yes | Binary restart files |
| xstFreq | int | steps | 1000 | 4800 | 9600 | XST output frequency |
| outputEnergies | int | steps | 1000 | 4800 | 9600 | Energy output |
| dcdfreq | int | steps | 1000 | 4800 | 9600 | Trajectory output |
| restartfreq | int | steps | 1000 | 48000 | 9600 | Restart file output |
| outputPressure | int | steps | 1000 | - | 9600 | Pressure output |
| **Wrapping** |
| wrapAll | bool | - | off | off | off | Wrap coordinates |
| wrapWater | bool | - | off | off | off | Wrap water only |
| **Restraints** |
| extraBonds | bool | - | off | on | on | Enable extrabonds |
| extraBondsFile | file | .exb | - | hextube.exb | hextube_MGHH_WI_k0.5.enm.extra | Restraints file |
| **Execution** |
| minimize | int | steps | 0 | 4800 | 4800 | Minimization steps |
| run | int | steps | 0 | 96000000 | 2400000 | MD simulation steps |
| firsttimestep | int | steps | 0 | 0 | [from xsc] | Starting timestep |
| **Restart** |
| bincoordinates | file | .coor | - | - | equil_min.coor | Load coordinates |
| binvelocities | file | .vel | - | - | equil_min.vel | Load velocities |
| extendedSystem | file | .xsc | - | - | equil_min.xsc | Load box/pressure |

**Parameter Dependencies**:
- `PME yes` → REQUIRES `cellBasisVector1/2/3`
- `langevinPiston on` → REQUIRES `langevinPistonTarget/Period/Decay/Temp`
- `rigidBonds all` → ENABLES larger `timestep` (2fs instead of 1fs)
- `extraBonds on` → REQUIRES `extraBondsFile`
- Restart → REQUIRES `bincoordinates`, `binvelocities`, `extendedSystem`
- Restart → MUST NOT set `temperature` (velocities from file)
- Restart → SHOULD set `firsttimestep` from .xsc file

### Appendix C: Glossary

**NAMD Terminology**:
- **PME** (Particle Mesh Ewald): Method for computing long-range electrostatic interactions in periodic systems
- **Langevin Dynamics**: Thermostat method using friction and random forces
- **NPT Ensemble**: Constant Number of particles, Pressure, and Temperature
- **NVT Ensemble**: Constant Number of particles, Volume, and Temperature
- **Extrabonds**: Harmonic restraints between atoms (springs)
- **ENM** (Elastic Network Model): Network of springs enforcing overall structure
- **MGHH**: Magnesium Hexahydrate - Mg²⁺ ion with 6 coordinated water molecules
- **Minimization**: Energy minimization to remove bad contacts
- **Equilibration**: Gradual relaxation to stable state
- **Production**: Main simulation phase after equilibration
- **Timestep**: Integration time step (typically 1-2 femtoseconds)
- **Trajectory**: Time series of atomic coordinates (.dcd file)
- **Restart files**: Binary checkpoint files (.coor, .vel, .xsc)

**NAMDRunner Concepts**:
- **Job Chain**: Sequence of jobs where each child continues from parent
- **Parent Job**: Job that provides restart files to child
- **Child Job**: Job that continues from parent's outputs
- **Root Job**: First job in chain (no parent)
- **Chain Depth**: Number of parent hops to root (0 = root, 1 = first child, etc.)
- **Checkpoint Restart**: Continue same simulation from interruption
- **Next Stage**: Start new simulation phase from previous final state
- **Template**: Pre-configured set of NAMD parameters for specific simulation type
- **Self-Contained Job**: Job with all necessary files copied locally (survives parent deletion)

**HPC/SLURM Terms**:
- **Walltime**: Maximum time job can run before being killed
- **Partition**: Group of cluster nodes with specific capabilities
- **QoS** (Quality of Service): Resource limits and priorities
- **Scratch Storage**: Fast temporary storage, auto-purged after 90 days
- **Project Storage**: Persistent storage, survives job completion
- **Node**: Individual compute server in cluster
- **Core**: CPU core (can run one process/thread)
- **MPI**: Message Passing Interface for parallel computing
- **SLURM**: Cluster workload manager
- **sbatch**: SLURM command to submit batch jobs
- **squeue**: SLURM command to check job queue status

---

**END OF DOCUMENT**

For questions or clarifications about this design, refer to:
- Tutorial files in `examples/origamiTutorial/`
- Current architecture docs in `docs/`
- NAMDRunner source code in `src-tauri/` and `src/`

