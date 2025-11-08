# NAMD Commands Reference

> **📚 For cluster-specific details**, see [`alpine-cluster-reference.md`](alpine-cluster-reference.md)

This document provides a complete reference of NAMD configuration patterns, command execution, and file management using the template-based configuration system.

## Table of Contents
- [Template System Overview](#template-system-overview)
- [Built-in Templates](#built-in-templates)
- [Template Structure](#template-structure)
- [Command Execution](#command-execution)
- [File Organization](#file-organization)
- [Parameter Validation](#parameter-validation)
- [Common Workflows](#common-workflows)
- [Error Handling](#error-handling)

## Template System Overview

NAMDRunner uses a flexible template system for NAMD configuration generation. Templates define:
- NAMD config structure with `{{variable}}` placeholders
- Variable types (FileUpload, Number, Text, Boolean)
- Constraints and defaults for each variable

Templates are stored in the database and rendered at job creation time. Built-in templates are embedded in `src-tauri/templates/`. All variables in a template must have values provided - if a `{{variable}}` placeholder exists in the template, it must be filled.

## Built-in Templates

### 1. Vacuum Optimization (`vacuum_optimization_v1`)

Structure optimization in vacuum with large periodic box, PME disabled, and elastic network model (ENM) restraints. Ideal for initial DNA origami structure prediction.

**Key Features:**
- PME disabled (vacuum simulation)
- Large periodic box (default 1000Å)
- Low Langevin damping (0.1) for fast relaxation
- Large pairlist margin (30Å) for vacuum simulations
- ENM restraints via extra bonds file

**Inputs:**
- Structure file (PSF)
- Coordinates file (PDB)
- Force field parameters (2 files)
- Extra bonds file (ENM restraints)

**Typical Use:** Energy minimization of DNA origami structures before solvation

### 2. Explicit Solvent NPT (`explicit_solvent_npt_v1`)

Constant pressure/temperature (NPT) simulation in explicit solvent with PME electrostatics. For equilibration and production runs of solvated systems.

**Key Features:**
- PME enabled (PME grid spacing: 1.5Å)
- NPT ensemble (Langevin piston at 1.01325 bar)
- Higher Langevin damping (5.0) for equilibration
- ENM restraints via extra bonds file
- Typical cell dimensions: 124×114×323Å

**Inputs:**
- Structure file (PSF) - solvated system
- Coordinates file (PDB) - solvated system
- Force field parameters (2 files)
- Cell dimensions (from solvated system)
- Extra bonds file (restraints)

**Typical Use:** Equilibration and production runs of solvated molecular systems

## Template Structure

Templates are JSON files with the following structure:

```json
{
  "id": "template_id",
  "name": "Human-Readable Name",
  "description": "Brief description of template purpose",
  "namd_config_template": "NAMD config with {{placeholders}}",
  "variables": {
    "variable_name": {
      "key": "variable_name",
      "label": "Display Label",
      "var_type": { /* Type definition */ },
      "help_text": "Help text for user"
    }
  },
  "is_builtin": true,
  "created_at": "ISO timestamp",
  "updated_at": "ISO timestamp"
}
```

### Variable Types

**FileUpload** - User uploads a file:
```json
{
  "FileUpload": {
    "extensions": [".psf", ".pdb"]
  }
}
```
- Files are automatically prefixed with `input_files/` when rendered
- Multiple extensions can be specified

**Number** - Numeric value with constraints:
```json
{
  "Number": {
    "min": 200.0,
    "max": 400.0,
    "default": 300.0
  }
}
```
- Integers rendered without decimals (e.g., `10000` not `10000.0`)
- Floats rendered with decimals (e.g., `2.5`)

**Text** - String value:
```json
{
  "Text": {
    "default": "default_value"
  }
}
```

**Boolean** - True/false converted to NAMD yes/no:
```json
{
  "Boolean": {
    "default": true
  }
}
```
- Rendered as `yes` (true) or `no` (false)

### Template Rendering

Templates use `{{variable_name}}` placeholders. The renderer:
1. Validates all variables have values provided
2. Substitutes each `{{variable}}` with its value
3. Applies type-specific formatting (file paths, yes/no, number precision)
4. Detects unreplaced variables and reports errors

**Example:**
```
Template: "temperature {{temperature}}\nstructure {{structure_file}}"
Values:   { "temperature": 310.5, "structure_file": "my_system.psf" }
Result:   "temperature 310.5\nstructure input_files/my_system.psf"
```

## Command Execution

### Module Loading (Alpine Cluster)

NAMDRunner uses hardcoded module loading for NAMD 3.0.1 on Alpine:

```bash
# Commands reference
# Discover available NAMD versions
module avail namd 2>&1 | grep namd

# Get module dependencies
module spider namd/3.0.1_cpu


# Load required modules for NAMD execution
module purge
module load gcc/14.2.0
module load openmpi/5.0.6
module load namd/3.0.1_cpu
```

### NAMD Execution Command

**Future Work**: Execution patterns and lodaded module versions will be configurable in a Settings page. Work after that will add module spider command results to Settings UI as well.

Alpine cluster uses OpenMPI-compiled NAMD 3.0.1:

```bash
# Execute NAMD with MPI (OpenMPI handles CPU affinity automatically, do not use +setcpuaffinity)
mpirun -np $SLURM_NTASKS namd3 config.namd > namd_output.log
```

**Important Notes:**
- `+setcpuaffinity` and `+pemap` flags are **incompatible** with OpenMPI-compiled NAMD
- Alpine uses OpenMPI, so these flags should **NOT** be used
- OpenMPI and SLURM handle CPU affinity automatically
- Only use `+setcpuaffinity` with ibverbs/native Charm++ builds (not MPI)
- Always redirect output to log file for debugging

### NAMD Execution in SLURM Job Scripts

**For complete SLURM job script templates**, see [slurm-commands-reference.md#complete-slurm-job-script-template](slurm-commands-reference.md#complete-slurm-job-script-template)

The execution line that goes in your SLURM job script:

```bash
# Execute NAMD with MPI (OpenMPI version - Alpine cluster)
mpirun -np $SLURM_NTASKS namd3 config.namd > namd_output.log
```

**Command breakdown:**
- `mpirun -np $SLURM_NTASKS` - Launch MPI with SLURM-allocated tasks
- `namd3` - NAMD 3.x binary (from module load)
- `config.namd` - NAMD configuration file (see templates below)
- `> namd_output.log` - Redirect output to log file

**Alpine-specific notes:**
- Do NOT use `+setcpuaffinity` or `+pemap` flags with Alpine's OpenMPI-compiled NAMD
- OpenMPI and SLURM automatically handle CPU affinity and task placement
- Manual CPU affinity flags cause "CmiSetCPUAffinity failed" errors due to conflicts with SLURM cgroups
- See [alpine-cluster-reference.md#mpi-execution-commands](alpine-cluster-reference.md#mpi-execution-commands) for cluster-specific MPI patterns

## File Organization

### Input Files from Researcher

#### Structure Files
- **`.psf` (Protein Structure File)**
  - Contains topology: atoms, bonds, angles, dihedrals
  - Defines molecular system connectivity
  - Created using VMD or similar tools

- **`.pdb` (Protein Data Bank File)**
  - Contains atomic coordinates (x, y, z positions)
  - Initial structure of molecular system
  - Must match PSF topology

#### Parameter Files
- **`.prm` (CHARMM Parameter Files)**
  - Force field parameters for molecular interactions
  - Common files:
    - `par_all36_na.prm` - Nucleic acid parameters
    - `par_water_ions_cufix.prm` - Water and ion parameters
  - May need multiple files for complex systems

#### Additional Input Files (Template-Dependent)
- **`.str` (Stream Files)** - Additional parameter definitions
- **`.exb` (Extra Bonds File)** - ENM restraints or other bonded constraints
- **Previous Simulation Outputs** (for restart templates):
  - `.coor` - Binary coordinate file
  - `.vel` - Binary velocity file
  - `.xsc` - Extended system configuration

### Directory Structure Pattern
```
/projects/$USER/namdrunner_jobs/
└── {job_id}/
    ├── job_info.json           # Job metadata
    ├── config.namd             # Rendered from template (in job root)
    ├── job.sbatch              # Generated SLURM script (in job root)
    ├── input_files/            # Uploaded input files
    │   ├── {user_filename}.pdb
    │   ├── {user_filename}.psf
    │   ├── {user_filename}.prm
    │   └── {user_filename}.exb
    └── outputs/                # After job completion
        ├── {job_name}_{slurm_job_id}.out
        ├── {job_name}_{slurm_job_id}.err
        ├── namd_output.log
        ├── {output_prefix}.dcd
        ├── {output_prefix}.xst
        └── {output_prefix}.restart.*

/scratch/alpine/$USER/namdrunner_jobs/
└── {job_id}/                   # Working directory during execution
    ├── [all files copied from projects]
    └── outputs/                # Generated output files
        ├── {output_prefix}.dcd
        ├── {output_prefix}.xst
        └── {output_prefix}.restart.*
```

### File Naming Conventions
- **NAMD config**: `config.namd` (rendered from template, in job root)
- **SLURM script**: `job.sbatch` (generated, in job root)
- **NAMD log**: `namd_output.log` (in working directory)
- **SLURM stdout**: `{job_name}_{slurm_job_id}.out` (in outputs/)
- **SLURM stderr**: `{job_name}_{slurm_job_id}.err` (in outputs/)
- **Trajectory**: `outputs/{output_prefix}.dcd` (user-defined prefix)
- **Extended system**: `outputs/{output_prefix}.xst`
- **Restart files**: `outputs/{output_prefix}.restart.coor/vel/xsc`

## Parameter Validation

### Template-Based Validation

Parameter validation is defined in template variable definitions:

```json
{
  "temperature": {
    "key": "temperature",
    "label": "Temperature (K)",
    "var_type": {
      "Number": {
        "min": 200.0,
        "max": 400.0,
        "default": 300.0
      }
    },
    "help_text": "Simulation temperature in Kelvin"
  }
}
```

Validation checks:
- **All variables must have values** - Every variable in the template must be provided
- **Number ranges** - Values must be within min/max bounds
- **File extensions** - Uploaded files must match allowed extensions
- **Type correctness** - Values must match variable type (number, text, boolean, file)

### Typical Parameter Ranges (Built-in Templates)

| Parameter | Min | Max | Default | Unit |
|-----------|-----|-----|---------|------|
| temperature | 200 | 400 | 300 | K |
| timestep | 1.0 | 4.0 | 2.0 | fs |
| steps | 100 | 100000000 | 4800 | - |
| langevin_damping | 0.01 | 10.0 | varies | - |
| pme_grid_spacing | 0.5 | 3.0 | 1.5 | Å |
| cell dimensions | 10/100 | 500/2000 | varies | Å |
| output frequencies | 100 | 100000 | varies | steps |

### Resource Requirements by System Size
| Atom Count | Recommended Cores | Memory | Typical Walltime |
|------------|------------------|---------|------------------|
| < 10,000   | 4-8             | 4-8GB   | 2-4 hours       |
| 10,000-50,000 | 16-32        | 16-32GB | 8-12 hours      |
| 50,000-150,000 | 32-64        | 32-64GB | 12-24 hours     |
| > 150,000  | 64-128          | 64-128GB| 24+ hours       |

### File Naming (Template System)

The template system automatically handles file naming:

**Template Variable (FileUpload type):**
```json
{
  "structure_file": {
    "key": "structure_file",
    "label": "Structure File (PSF)",
    "var_type": {
      "FileUpload": {
        "extensions": [".psf"]
      }
    },
    "help_text": "NAMD structure file defining molecular topology"
  }
}
```

**Template Placeholder:**
```tcl
structure {{structure_file}}
```

**User uploads:** `my_dna_origami.psf`

**Rendered output:**
```tcl
structure input_files/my_dna_origami.psf
```

The renderer automatically:
- Uses the actual uploaded filename
- Prepends `input_files/` directory
- Maintains user's original file naming

## Common Workflows

### Typical Simulation Workflow

1. **Select Template** - Choose appropriate template (vacuum_optimization_v1 or explicit_solvent_npt_v1)
2. **Upload Files** - Upload input files (PSF, PDB, parameters, restraints)
3. **Configure Variables** - Fill in template variables (temperature, steps, cell dimensions, etc.)
4. **Preview Config** - Review rendered NAMD config before submission
5. **Submit Job** - Create job and submit to SLURM
6. **Monitor Progress** - Check job status and logs
7. **Download Results** - Retrieve trajectory and restart files

### Resource Allocation Strategy
1. **Test runs**: 4 cores, 4GB, 10 minutes
2. **Small systems**: 24 cores, 16GB, 4 hours
3. **Production runs**: 48 cores, 32GB, 24 hours
4. **Large systems**: 64 cores, 64GB, extended walltime (single-node limit)

### Template Selection Guide

**For vacuum optimization:**
- Use `vacuum_optimization_v1` template
- Ideal for initial structure relaxation
- Has ENM restraints file
- Set execution_command to "minimize" for minimization or "run" for dynamics

**For solvated systems:**
- Use `explicit_solvent_npt_v1` template
- For equilibration and production
- Has cell dimensions from solvation
- Has extra bonds file for restraints

### Multi-Stage Workflows (Future Enhancement)

Multi-stage workflows (e.g., gradual release of restraints) are not yet implemented. Current approach:
- Create separate jobs for each stage manually
- Use appropriate template for each stage
- Manage restart files between stages manually

## Error Handling

### Common NAMD Errors

#### Simulation Instability
```
Error: Periodic cell has become too small for original patch grid!
```
**Cause**: System collapse or extreme conditions
**Solution**: Reduce timestep, check restraints, verify initial structure

#### Missing Files
```
FATAL ERROR: Unable to read PDB file structure.pdb
```
**Cause**: File path issues or missing input files
**Solution**: Verify file paths, check SFTP uploads, use absolute paths

#### PME Grid Issues
```
Error: PME grid size X is not even
```
**Cause**: Automatic PME grid calculation failed
**Solution**: Manually set PME grid dimensions or adjust spacing

#### Memory Exhaustion
```
FATAL ERROR: Memory allocation failed
```
**Cause**: Insufficient memory allocation
**Solution**: Increase SLURM memory request, reduce system size, or optimize settings

### Runtime Error Detection Patterns
Monitor NAMD log for these critical patterns:
- `FATAL ERROR:` - Immediate failure
- `Warning:` - Potential issues
- `ERROR:` - Critical problems
- Energy spikes > 1000% - System instability
- Temperature spikes > 500K - Simulation problems

### Recovery Strategies
1. **For timeout errors**: Use restart template with remaining steps
2. **For memory errors**: Increase memory allocation, reduce output frequency
3. **For instability**: Reduce timestep, add restraints, check initial structure
4. **For missing files**: Verify SFTP sync, check file paths

### NAMD Error Message Mapping

**User-friendly error messages for common NAMD failures:**

| Error Pattern | Category | User Message | Recommended Action |
|---------------|----------|--------------|-------------------|
| `FATAL ERROR: ...` | Simulation | Simulation failed | Check NAMD output log for details |
| `Periodic cell too small for original patch grid` | Configuration | System instability detected | Reduce timestep or check initial structure |
| `Unable to read PDB file` | FileSystem | Input file not found | Verify file upload succeeded |
| `Memory allocation failed` | Resources | Insufficient memory | Increase SLURM memory request (e.g., from 32GB to 64GB) |
| `PME grid size X is not even` | Configuration | PME grid misconfigured | Manually set PME grid or adjust spacing |
| `Warning: Atoms moving too fast` | Simulation | System instability | Reduce timestep, add restraints, or check initial conditions |

**Best practice:** Provide actionable guidance. Instead of showing raw NAMD errors, map them to clear messages with next steps.

## Testing Templates

### Example Template Values (vacuum_optimization_v1)

```json
{
  "structure_file": "dna_origami.psf",
  "coordinates_file": "dna_origami.pdb",
  "parameters_file": "par_all36_na.prm",
  "parameters_file_2": "par_water_ions_cufix.prm",
  "extrabonds_file": "dna_restraints.exb",
  "output_name": "vacuum_opt",
  "temperature": 300,
  "timestep": 2,
  "langevin_damping": 0.1,
  "margin": 30,
  "cell_x": 1000,
  "cell_y": 1000,
  "cell_z": 1000,
  "xst_freq": 4800,
  "output_energies_freq": 4800,
  "dcd_freq": 4800,
  "restart_freq": 48000,
  "execution_command": "minimize",
  "steps": 4800
}
```

### Example Rendered Config (vacuum_optimization_v1)

```tcl
#############################################################
## NAMD CONFIGURATION - VACUUM OPTIMIZATION              ##
#############################################################
# Generated by NAMDRunner
# Template: Vacuum Optimization v1

#############################################################
## INPUT FILES                                            ##
#############################################################
structure          input_files/dna_origami.psf
coordinates        input_files/dna_origami.pdb

#############################################################
## OUTPUT FILES                                           ##
#############################################################
outputName         outputs/vacuum_opt
XSTfile            outputs/vacuum_opt.xst
DCDfile            outputs/vacuum_opt.dcd

# ... (Force field, PME, temperature control sections) ...

#############################################################
## EXECUTION                                               ##
#############################################################
temperature        300
minimize           4800
```

### NAMD Execution Output (Example)

```
Charm++> Running on MPI version: 3.1
Info: NAMD 3.0.1 for Linux-x86_64
Info: Running on 24 processors, 1 physical node
Info: CPU topology information available
ETITLE: TS  BOND  ANGLE  DIHED  IMPRP  ELECT  VDW  BOUNDARY  MISC  KINETIC  TOTAL  TEMP
ENERGY: 0  123456.78  789012.34  345678.90  12.34  -9876543.21  567890.12  0  0  876543.21  -8765432.10  298.15
TIMING: 0  CPU: 0.12s, 0.12s/step  Wall: 0.12s
```

## Best Practices

### Template Creation
1. **Define clear variable names** - Use descriptive keys that match NAMD conventions
2. **Set appropriate constraints** - Define realistic min/max ranges for Number types
3. **Provide helpful descriptions** - Write clear help_text for each variable
4. **Use sensible defaults** - Default values should work for typical cases
5. **Validate template syntax** - Preview rendered configs before saving templates

### Job Configuration
1. **Preview before submission** - Always review rendered config
2. **Use actual file names** - Template system preserves uploaded filenames
3. **Set appropriate output frequencies** - Balance detail vs storage based on system size
4. **Choose correct template** - Vacuum vs explicit solvent, minimization vs dynamics
5. **Verify file uploads** - Ensure all files are uploaded

### Performance Optimization
1. **Scale resources with atom count** - Use resource allocation guidelines
2. **Optimize PME grid spacing** - 1.5Å typical for explicit solvent
3. **Balance output frequency** - Higher frequency for short equilibration, lower for production
4. **Monitor memory usage** - Increase allocation if jobs fail with memory errors
5. **Single-node execution** - Current limit is 64 cores (amilan partition)

### File Management
1. **Use descriptive output prefixes** - Make files easy to identify later
2. **Verify file transfers** - Check that files uploaded successfully before submission
3. **Archive large trajectories** - DCD files can be very large for long simulations
4. **Keep restart files** - Essential for continuing failed/timeout jobs
5. **Clean up scratch space** - Remove old job directories periodically

## Implementation Reference

### Template System Architecture

**Components:**
- `src-tauri/src/templates/types.rs` - Template and variable type definitions
- `src-tauri/src/templates/renderer.rs` - Template rendering engine
- `src-tauri/src/templates/validation.rs` - Value validation
- `src-tauri/templates/*.json` - Built-in template definitions
- `src-tauri/src/commands/templates.rs` - IPC commands for template operations

**Key Functions:**
- `render_template(template, values)` - Render template with values
- `validate_values(template, values)` - Validate values against template constraints
- `list_templates()` - Get all available templates
- `get_template(id)` - Load specific template definition
- `preview_namd_config(template_id, values)` - Preview rendered config

### Database Schema

Templates are stored in SQLite with JSON serialization:

```sql
CREATE TABLE templates (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    namd_config_template TEXT NOT NULL,
    variables TEXT NOT NULL,  -- JSON serialized HashMap
    is_builtin INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

This reference documents the template-based NAMD configuration system for NAMDRunner on Alpine cluster.