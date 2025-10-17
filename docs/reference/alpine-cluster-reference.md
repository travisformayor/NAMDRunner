# Alpine Cluster Reference Guide

This document provides the authoritative reference for Alpine cluster integration in NAMDRunner. All cluster-specific implementation details should reference this guide.

> **Source**: CU Research Computing (CURC) Alpine cluster documentation
> **Last Updated**: 2025-01-15
> **Target Cluster**: CURC Alpine (RHEL 8.4/8.8, SLURM scheduler)
> **For SSH/SFTP implementation patterns**, see [`../SSH.md`](../SSH.md)

## Cluster Overview

**Alpine** is CU Research Computing's primary cluster for NAMDRunner integration:
- **Architecture**: AMD Milan processors, HDR InfiniBand fabric
- **Scheduler**: SLURM workload manager
- **Module System**: Environment Modules for software management
- **File Systems**: `/home/$USER` (2GB), `/projects/$USER` (250GB), `/scratch/alpine/$USER` (10TB, 90-day purge)

## Hardware Partitions

| Partition | Description | Nodes | Cores/Node | RAM/Core | Max Walltime | GPU Type | GPU Count |
|-----------|-------------|-------|------------|----------|--------------|----------|-----------|
| `amilan` | General compute (default) | 374+ | 32/48/64 | 3.75 GB | 24H (7D with long QoS) | None | 0 |
| `amilan128c` | High-core compute | 16+ | 128 | 2.01 GB | 24H (7D with long QoS) | None | 0 |
| `amem` | High-memory | 22+ | 48/64/128 | 16-21.5 GB | 4H (7D with mem QoS) | None | 0 |
| `aa100` | NVIDIA GPU compute | 10+ | 64 | 3.75 GB | 24H (7D with long QoS) | NVIDIA A100 | 3 |
| `ami100` | AMD GPU compute | 8+ | 64 | 3.75 GB | 24H (7D with long QoS) | AMD MI100 | 3 |
| `al40` | NVIDIA L40 GPU | 2+ | 64 | 3.75 GB | 24H (7D with long QoS) | NVIDIA L40 | 3 |

### Testing Partitions (Development Use)
| Partition | Purpose | Max Runtime | Max Jobs/User | Resource Limits |
|-----------|---------|-------------|---------------|-----------------|
| `atesting` | CPU testing | 1 hour | 5 | 2 nodes, 16 cores total |
| `atesting_a100` | GPU testing (MIG) | 1 hour | 5 | 1 GPU instance, 10 cores |
| `atesting_mi100` | GPU testing | 1 hour | 5 | 3 GPUs, 64 cores |
| `acompile` | Code compilation | 12 hours | - | 1 node, 4 cores |

## Quality of Service (QoS)

| QoS | Description | Max Walltime | Max Jobs/User | Node Limits | Valid Partitions |
|-----|-------------|--------------|---------------|-------------|------------------|
| `normal` | Default QoS | 1 day | 1000 | 128 | amilan, amilan128c, aa100, ami100, al40 |
| `long` | Extended runtime | 7 days | 200 | 20 | amilan, amilan128c, aa100, ami100, al40 |
| `mem` | High-memory jobs | 7 days | 1000 | 12 | amem only (requires 256GB+ RAM) |
| `testing` | Testing partitions | 1 hour | 5 | 2 | atesting, atesting_a100, atesting_mi100 |
| `compile` | Compilation jobs | 12 hours | - | 1 | acompile |

## Resource Allocation Rules

### Default Recommendations for NAMD Jobs
- **Small jobs**: `amilan`, 24 cores, 16GB memory, 4 hours
- **Production runs**: `amilan`, 48 cores, 32GB memory, 24 hours  
- **Large simulations**: `amilan128c`, 128 cores, 64GB memory, 7 days (long QoS)
- **GPU acceleration**: `aa100`, 64 cores + 1-3 GPUs, 24 hours

### Resource Limits
- **Maximum cores**: 128 per node (amilan128c partition)
- **Maximum memory**: 256GB per node (amem high-memory nodes)
- **Maximum walltime**: 24 hours (normal), 7 days (long/mem QoS)
- **GPU limits**: 15 MI100, 21 A100, 6 L40 total across all user jobs

### Memory Specification

**SLURM Memory Units:**
- Bare numbers (no unit) = **MEGABYTES**
- `--mem=64` → 64 **MB** (will cause OOM failures!)
- `--mem=64GB` → 64 **GB** (correct)
- `--mem=65536` → 64 GB (65536 MB, correct but not recommended)

**Always append unit suffix:**
```bash
#SBATCH --mem=32GB   # Correct
#SBATCH --mem=32     # WRONG - only 32 MB!
```

**Common mistake:** Forgetting unit suffix causes ALL jobs to fail with Out of Memory errors.

### Billing Weights
- **CPU partitions**: 1.0 SU per core-hour
- **GPU partitions**: 1.0 SU per core-hour + 108.2 SU per GPU-hour
- **Example**: 64 cores + 3 GPUs for 1 hour = (64 × 1.0) + (3 × 108.2) = 389 SUs

## Module Environment

### Essential Module Loading Sequence
```bash
# For all SLURM operations (login nodes only)
module load slurm/alpine

# For NAMD job execution (in job scripts - MUST be in order)
module purge
module load gcc/14.2.0       # Base compiler (required first)
module load openmpi/5.0.6    # Requires gcc/14.2.0
module load namd/3.0.1_cpu   # Requires gcc/14.2.0 + openmpi/5.0.6
```

### Critical Module Notes
- **Never load `slurm/alpine` on compute nodes** - login nodes only
- **Always `module purge` in job scripts** to ensure clean environment
- **Module loading order matters** - openmpi requires gcc, namd requires both
- **Module versions are cluster-specific** - verify availability on compute nodes
- **Modules only visible on compute nodes** - not available on login nodes
- **Source environment**: Use `source /etc/profile` before module commands in scripts

### Interactive Node Access
```bash
# Start interactive session for testing/debugging (preferred method)
sinteractive --partition=amilan --time=00:10:00 --ntasks=1 --nodes=1 --qos=normal

# Once in interactive session, modules are available:
module spider gcc       # List available versions
module spider openmpi   # Check dependencies
module spider namd      # Verify installation
```

## SLURM Command Patterns

### Command Execution Strategy
All SLURM commands must be executed with proper module loading:

```rust
// Rust command wrapper pattern
let full_command = format!("source /etc/profile && module load slurm/alpine && {}", command);
```

This ensures the SLURM environment is properly loaded before executing any scheduler commands.

### Job Submission
```bash
# Submit job script (returns job ID)
cd /scratch/alpine/$USER/namdrunner_jobs/<job_id>/
module load slurm/alpine && sbatch job.sbatch

# Expected response pattern
"Submitted batch job 12345678"

# Parse Job ID using regex
Extract using regex: /Submitted batch job (\d+)/
```

### Status Monitoring
```bash
# Active jobs (running/pending)
module load slurm/alpine && squeue -u $USER --format="%i|%j|%t|%M|%L|%D|%C|%m|%P|%Z" --noheader

# Example output format:
# 12345678|job_001|R|01:30:45|22:29:15|1|48|32GB|amilan|/scratch/alpine/username/namdrunner_jobs/job_001

# Completed jobs (last 7 days)
module load slurm/alpine && sacct -u $USER --starttime=$(date -d '7 days ago' +%Y-%m-%d) --format=JobID,JobName,State,ExitCode,Start,End,Elapsed,WorkDir --parsable2 --noheader

# Example output format:
# 12345678|job_001|COMPLETED|0:0|2025-01-15T10:00:00|2025-01-15T11:00:00|01:00:00|/scratch/alpine/username/namdrunner_jobs/job_001
```

### Response Parsing Patterns

#### squeue Field Mappings
The `squeue` format string `%i|%j|%t|%M|%L|%D|%C|%m|%P|%Z` produces fields:
- `%i` - Job ID (SLURM job ID)
- `%j` - Job Name  
- `%t` - State (PD=Pending, R=Running, CG=Completing)
- `%M` - Time Used (HH:MM:SS)
- `%L` - Time Left (HH:MM:SS)
- `%D` - Number of Nodes
- `%C` - Number of CPUs
- `%m` - Memory Requested
- `%P` - Partition
- `%Z` - Working Directory

#### sacct Field Mappings
The `sacct` format `JobID,JobName,State,ExitCode,Start,End,Elapsed,WorkDir` produces:
- **JobID** - SLURM job ID
- **JobName** - Job name from script
- **State** - Final state (COMPLETED, FAILED, CANCELLED, TIMEOUT)
- **ExitCode** - Exit code (format: signal:status)
- **Start** - Start timestamp (ISO 8601)
- **End** - End timestamp (ISO 8601)  
- **Elapsed** - Runtime duration (HH:MM:SS)
- **WorkDir** - Working directory path

#### NAMDRunner Job Identification
Identify our jobs by working directory pattern:
```bash
/scratch/alpine/$USER/namdrunner_jobs/*
```

### Status Code Mappings
| SLURM State | NAMDRunner Status | Description |
|-------------|-------------------|-------------|
| `PD` | `PENDING` | Waiting for resources |
| `R` | `RUNNING` | Executing on compute nodes |
| `CG` | `RUNNING` | Completing (cleanup phase) |
| `COMPLETED` | `COMPLETED` | Finished successfully |
| `FAILED` | `FAILED` | Non-zero exit code |
| `CANCELLED` | `CANCELLED` | User cancelled |
| `TIMEOUT` (`TO`) | `FAILED` | Exceeded walltime limit |
| `OUT_OF_MEMORY` (`OOM`) | `FAILED` | Insufficient memory allocation |
| `NODE_FAIL` (`NF`) | `FAILED` | Compute node failure |
| `BOOT_FAIL` (`BF`) | `FAILED` | Node failed to boot |
| `DEADLINE` (`DL`) | `FAILED` | Job deadline exceeded |
| `PREEMPTED` (`PR`) | `FAILED` | Job preempted by higher priority |

## MPI Best Practices for Alpine

### Node Calculation
**Critical**: Always explicitly specify `--nodes` for optimal MPI performance and task placement.

**Calculation Formula:**
```
nodes = ceiling(cores / cores_per_node)
```

**Alpine Core Counts per Node:**
- `amilan`: 64 cores/node (most common)
- `amilan` (some nodes): 32 or 48 cores/node
- `amilan128c`: 128 cores/node
- `amem`: 48, 64, or 128 cores/node

**Examples:**
- 48 cores → `--nodes=1` (single node, 48/64 = 0.75)
- 64 cores → `--nodes=1` (single node, exactly one full node)
- 96 cores → `--nodes=2` (multi-node, 96/64 = 1.5, round up)
- 128 cores → `--nodes=2` on amilan OR `--nodes=1` on amilan128c

**For NAMDRunner (single-node MVP):**
- Always use `--nodes=1` for Phase 6
- Core count limited to 64 (one full amilan node)
- Multi-node support deferred to Phase 7+

### OpenMPI Requirements

**Environment Export (CRITICAL):**
OpenMPI requires `SLURM_EXPORT_ENV=ALL` when jobs are scheduled from login nodes.

```bash
export SLURM_EXPORT_ENV=ALL
```

**Why?** Ensures proper MPI initialization and environment propagation to compute nodes.

### MPI Execution Commands

**Recommended:** Use `mpirun` or `mpiexec` (not `srun`)

```bash
# Preferred method for Alpine
mpirun -np $SLURM_NTASKS <application>

# Alternative (standardized across MPI distributions)
mpiexec -np $SLURM_NTASKS <application>
```

**Why not srun?** CURC recommends `mpirun`/`mpiexec` for simplicity and reliability. `srun` can have execution issues.

### CPU Affinity with OpenMPI on Alpine

Alpine allows using OpenMPI for MPI applications (including NAMD). OpenMPI automatically manages CPU affinity in coordination with SLURM's cgroup-based resource allocation.

**For NAMD specifically:**
- **Do NOT use** `+setcpuaffinity` or `+pemap` flags
- These flags cause "CmiSetCPUAffinity failed" errors
- OpenMPI + SLURM handle task placement automatically
- Manual affinity conflicts with SLURM's CPU allocation

**Correct NAMD execution:**
```bash
# Correct - let OpenMPI handle affinity
mpirun -np $SLURM_NTASKS namd3 config.namd

# WRONG - manual affinity causes conflicts
mpirun -np $SLURM_NTASKS namd3 +setcpuaffinity +pemap 0-23 config.namd  # Will fail!
```

**When CPU affinity flags ARE appropriate:**
- Non-MPI builds (ibverbs/native Charm++ builds only)
- Full node allocation (all CPUs on node)
- Never with OpenMPI-compiled applications on Alpine

### Infiniband Constraint

**Always use `--constraint=ib`** to force jobs onto Infiniband-enabled nodes.

```bash
#SBATCH --constraint=ib
```

**Why?** MPI communication requires high-speed interconnect. Not all Alpine nodes have Infiniband.

### MPI Limitations on Alpine

- **Maximum cores per job:** 4,096 (64 nodes × 64 cores)
- **Chassis restriction:** MPI jobs cannot span chassis boundaries
- **Infiniband requirement:** Use `--constraint=ib` for MPI jobs

## Job Script Generation

**For complete SLURM job script templates**, see [slurm-commands-reference.md#complete-slurm-job-script-template](slurm-commands-reference.md#complete-slurm-job-script-template)

The SLURM reference provides a fully-annotated template integrating Alpine requirements, MPI best practices, and NAMD execution.

### Alpine-Specific Job Script Requirements

When generating job scripts for Alpine, ensure these Alpine-specific elements are included:

**Memory specification:**
```bash
#SBATCH --mem=32GB   # Must include GB unit
```

**Node allocation:**
```bash
#SBATCH --nodes=1    # For single-node jobs (≤64 cores on amilan)
```

**Infiniband constraint:**
```bash
#SBATCH --constraint=ib   # Force Infiniband nodes for MPI
```

**Environment initialization:**
```bash
source /etc/profile       # Required for SSH connections
export SLURM_EXPORT_ENV=ALL   # Required for OpenMPI
```

**Module loading sequence:**
```bash
module purge
module load gcc/14.2.0
module load openmpi/5.0.6
module load namd/3.0.1_cpu
```

### Partition-Specific Considerations

**Standard CPU jobs (amilan):**
- Use `--partition=amilan --qos=normal`
- 64 cores/node (most common)
- Up to 24 hours walltime (7 days with `--qos=long`)

**High-memory jobs (amem):**
- Use `--partition=amem --qos=mem`
- Minimum 256GB memory required
- Up to 7 days walltime

**GPU jobs (aa100):**
- Use `--partition=aa100 --qos=normal`
- Add `--gres=gpu:N` (N = 1-3)
- Billing: 1 SU/core-hour + 108.2 SU/GPU-hour

## Directory Structure Requirements

### NAMDRunner Working Pattern
```
/projects/$USER/namdrunner_jobs/
└── {job_id}/
    ├── job_info.json           # Job metadata
    ├── input_files/            # User-uploaded input files
    │   ├── structure.pdb
    │   ├── structure.psf
    │   └── parameters.prm
    ├── scripts/                # Generated job scripts
    │   ├── config.namd
    │   └── job.sbatch
    ├── outputs/                # NAMD output files (after job completion)
    │   ├── sim.dcd             # Trajectory
    │   ├── sim.coor            # Restart files
    │   ├── sim.vel
    │   └── sim.xsc
    ├── namd_output.log         # NAMD console output
    ├── {job_name}_{slurm_job_id}.out  # SLURM stdout
    └── {job_name}_{slurm_job_id}.err  # SLURM stderr

/scratch/alpine/$USER/namdrunner_jobs/
└── {job_id}/                   # Working directory during execution (rsync mirror)
    └── (same structure as project directory)
```

### Job Identification Pattern
NAMDRunner jobs are identified by working directory pattern:
```
/scratch/alpine/$USER/namdrunner_jobs/*
```

## Error Handling Patterns

### Common SLURM Errors

#### Error Message Patterns
Common SLURM errors and their patterns:
- `Invalid partition`: Check partition name
- `Invalid qos specification`: User not authorized for QOS
- `Requested node configuration is not available`: Resource limits exceeded
- `Unable to allocate resources`: Queue full or maintenance

#### Example SLURM Error Messages
```bash
# Resource unavailable
"sbatch: error: Batch job submission failed: Invalid partition name specified"

# Authentication issues
"sbatch: error: Batch job submission failed: Access denied"

# Resource limits exceeded
"sbatch: error: Batch job submission failed: Requested node configuration is not available"
```

#### Error Resolution Table
| Error Message | Cause | Resolution |
|---------------|-------|------------|
| `Invalid partition name specified` | Wrong partition for allocation | Check user's available partitions |
| `Requested node configuration is not available` | Resource limits exceeded | Reduce cores/memory request |
| `Job exceeded walltime limit` | Job ran too long | Increase walltime or optimize |
| `Access denied` | No valid allocation | Verify user has active allocation |

### Network and Module Errors  
| Error Pattern | Cause | Resolution |
|---------------|-------|------------|
| `Connection timed out` | Network issues | Retry with exponential backoff |
| `Module command not found` | Missing environment | Source `/etc/profile` first |
| `Permission denied (publickey)` | SSH auth failure | Verify password authentication |

### Best Practices for Error Recovery
- **Retry transient network failures** with exponential backoff
- **Validate SSH connection** before SLURM commands
- **Parse both stdout and stderr** for complete error information  
- **Handle authentication timeouts** gracefully with re-connection prompts
- **Log command failures** without exposing credentials

## Storage and Purge Policies

### File System Usage
- **Home directory** (`/home/$USER`): 2GB limit, permanent storage
- **Project directory** (`/projects/$USER`): 250GB limit, permanent storage  
- **Scratch directory** (`/scratch/alpine/$USER`): 10TB limit, **90-day purge**

### Critical Purge Warning
Files in `/scratch/alpine/` are **automatically deleted after 90 days**. NAMDRunner must:
- Copy important results back to `/projects/` before purge
- Monitor scratch usage and warn users of approaching purge dates
- Design workflows assuming scratch files are temporary

## Implementation Notes for NAMDRunner

### Required Capabilities by Phase
- **Phase 1.3**: SSH connection, module loading patterns
- **Phase 2.1**: SFTP file transfer, directory creation
- **Phase 2.2**: Job discovery, status sync
- **Phase 4.1**: Job submission, template generation
- **Phase 4.2**: Status parsing, result retrieval

### Security Requirements
- **Never log passwords** or credentials in any form
- **Use password authentication only** (no SSH keys)
- **Clear credentials from memory** on disconnect
- **Validate all file paths** to prevent directory traversal

### Performance Considerations
- **Batch SLURM commands** when possible to reduce SSH overhead
- **Cache job status** for 30-60 seconds to avoid excessive queries
- **Limit concurrent SSH connections** to 3 maximum
- **Use background threads** for long-running operations

### Compatibility Notes
- **Module versions may change** - make configurable in application
- **Partition availability varies** by allocation - query user's access
- **QoS permissions differ** by user group - handle gracefully
- **Resource limits can change** - validate before submission

---

*This guide reflects Alpine cluster capabilities as of September 2025. For the most current information, consult [CURC documentation](https://curc.readthedocs.io/).*