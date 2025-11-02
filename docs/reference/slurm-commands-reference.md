# SLURM Commands Reference

> **📚 For cluster-specific details**, see [`alpine-cluster-reference.md`](alpine-cluster-reference.md)
> **🔐 For SSH/SFTP patterns**, see [`../SSH.md`](../SSH.md)
> **⚙️ For dynamic configuration**, see Phase 5 Settings Page architecture

This document provides a complete reference of SLURM commands and patterns. **Note**: Starting in Phase 5, resource limits and partition information will be discoverable via the Settings page rather than hardcoded.

## Table of Contents
- [Module Loading](#module-loading)
- [Job Submission](#job-submission)
- [Status Checking](#status-checking)
- [Job Management](#job-management)
- [Resource Limits](#resource-limits)
- [Error Messages](#error-messages)
- [Mock Data for Testing](#mock-data-for-testing)

## Module Loading

### Required Module Sequence
Always load modules before any SLURM commands:

```bash
# Basic SLURM operations
source /etc/profile && module load slurm/alpine

# For NAMD job execution
module purge
module load gcc/14.2.0
module load openmpi/5.0.6
module load namd/3.0.1_cpu
```

**Critical:** The `source /etc/profile` is essential for SSH connections.

## Job Submission

### Complete SLURM Job Script Template

This template brings together SLURM directives, Alpine cluster requirements, MPI best practices, and NAMD execution. Each section is annotated with references to detailed documentation.

```bash
#!/bin/bash

#############################################################
## SLURM RESOURCE DIRECTIVES                              ##
#############################################################
# Basic job identification
#SBATCH --job-name={{ job_name }}
#SBATCH --output={{ job_name }}_%j.out
#SBATCH --error={{ job_name }}_%j.err

# Alpine cluster partition and QoS
# See: alpine-cluster-reference.md#hardware-partitions
#SBATCH --partition=amilan
#SBATCH --qos=normal

# Node allocation (critical for MPI performance)
# See: alpine-cluster-reference.md#node-calculation
#SBATCH --nodes={{ nodes }}          # nodes = ceiling(cores / 64) for amilan
#SBATCH --ntasks={{ num_cores }}     # total MPI tasks

# Resource allocation
#SBATCH --time={{ walltime }}        # format: HH:MM:SS or DD-HH:MM:SS
#SBATCH --mem={{ memory }}GB         # CRITICAL: Must include "GB" unit (bare numbers = MB, causes OOM)

# MPI requirements for Alpine
# See: alpine-cluster-reference.md#mpi-best-practices
#SBATCH --constraint=ib              # Force Infiniband-enabled nodes

#############################################################
## ENVIRONMENT INITIALIZATION                             ##
#############################################################
# Initialize module system (required for SSH connections)
source /etc/profile

# Load Alpine-specific software stack
# See: alpine-cluster-reference.md#module-environment
module purge
module load gcc/14.2.0
module load openmpi/5.0.6
module load namd/3.0.1_cpu

# OpenMPI requirement when submitting from login nodes (not needed for interactive jobs)
# See: alpine-cluster-reference.md#openmpi-requirements
export SLURM_EXPORT_ENV=ALL

#############################################################
## JOB EXECUTION                                          ##
#############################################################
# Navigate to working directory
cd {{ working_dir }}

# Execute NAMD with MPI
# See: namd-commands-reference.md#command-execution
# Note: Use actual uploaded file names in config, not generic names like "structure.psf"
# NAMDRunner extracts names from InputFile.name field based on file_type
# Do NOT use +setcpuaffinity with OpenMPI-compiled NAMD (Alpine uses OpenMPI)
# OpenMPI handles CPU affinity automatically - manual affinity flags cause binding conflicts
mpirun -np $SLURM_NTASKS namd3 {{ namd_config }} > {{ namd_log }}
#      │             │     │               │
#      │             │     │               └─ NAMD output log
#      │             │     └─ NAMD config file
#      │             └─ Number of MPI tasks (from SLURM)
#      └─ MPI launcher (recommended over srun on Alpine)
```

### Template Variable Reference

| Variable | Description | Example | Documentation |
|----------|-------------|---------|---------------|
| `{{ job_name }}` | Job identifier | `my-namd-job` | SLURM standard |
| `{{ nodes }}` | Number of nodes | `1` (single-node only app version) | [alpine-cluster-reference.md#node-calculation](alpine-cluster-reference.md#node-calculation) |
| `{{ num_cores }}` | Total MPI tasks | `48` | [alpine-cluster-reference.md#resource-allocation-rules](alpine-cluster-reference.md#resource-allocation-rules) |
| `{{ walltime }}` | Maximum runtime | `24:00:00` | [alpine-cluster-reference.md#quality-of-service-qos](alpine-cluster-reference.md#quality-of-service-qos) |
| `{{ memory }}` | Memory in GB (unit added by template) | `32` → `32GB` |  |
| `{{ working_dir }}` | Job working directory | `/scratch/alpine/user/job_001` | [alpine-cluster-reference.md#directory-structure-requirements](alpine-cluster-reference.md#directory-structure-requirements) |
| `{{ namd_config }}` | NAMD config file | `config.namd` | [namd-commands-reference.md#configuration-templates](namd-commands-reference.md#configuration-templates) |
| `{{ namd_log }}` | NAMD output log | `namd_output.log` | [namd-commands-reference.md#file-organization](namd-commands-reference.md#file-organization) |

### Critical Requirements Checklist

Before generating a job script, verify:
- ✅ **Memory includes `GB` unit** - Bare numbers interpreted as MB (64 = 64MB causes OOM, not 64GB)
- ✅ **`--nodes` calculated correctly** - `ceiling(cores/64)` for amilan partition (Phase 6: always nodes=1, max 64 cores)
- ✅ **`--constraint=ib` included** - Required for MPI jobs on Alpine
- ✅ **`export SLURM_EXPORT_ENV=ALL` present** - Required for OpenMPI when jobs submitted from login nodes
- ✅ **`source /etc/profile` before modules** - Required for SSH connections to initialize module system
- ✅ **NAMD config uses actual uploaded file names** - Extract from InputFile.name (e.g., "hextube.psf" not "structure.psf")

**Phase 6.6 Fixes Applied:**
- Memory unit bug fixed in `script_generator.rs::generate_namd_script()` (appends "GB" if missing)
- File naming bug fixed (uses `input_files.iter().find(|f| f.file_type == Psf).name`)
- OpenMPI export added to template
- Node calculation implemented (Phase 6: single-node only, validates cores ≤ 64)

### Submit Command
```bash
# Navigate to job directory and submit
cd /scratch/alpine/$USER/namdrunner_jobs/<job_id>/ && sbatch job.sbatch

# With module loading
source /etc/profile && module load slurm/alpine && cd /path/to/job && sbatch job.sbatch
```

### Parse Submission Output
```
Submitted batch job 12345678
```
Extract job ID with regex: `/Submitted batch job (\d+)/`

## Status Checking

### Active Jobs (PENDING/RUNNING)
```bash
# Command
squeue -u $USER --format="%i|%j|%t|%M|%L|%D|%C|%m|%P|%Z" --noheader

# Example Output
12345678|namd_job|R|01:30:45|22:29:15|1|48|32GB|amilan|/scratch/alpine/user/namdrunner_jobs/job_001
12345679|namd_job|PD|0:00:00|24:00:00|1|24|16GB|amilan|/scratch/alpine/user/namdrunner_jobs/job_002
```

**Format fields:**
- `%i` - Job ID
- `%j` - Job Name  
- `%t` - State (PD=Pending, R=Running, CG=Completing)
- `%M` - Time Used (HH:MM:SS)
- `%L` - Time Left (HH:MM:SS)
- `%D` - Number of Nodes
- `%C` - Number of CPUs
- `%m` - Memory
- `%P` - Partition
- `%Z` - Working Directory

### Completed Jobs (Last 7 Days)
```bash
# Command
sacct -u $USER --starttime=$(date -d '7 days ago' +%Y-%m-%d) \
  --format=JobID,JobName,State,ExitCode,Start,End,Elapsed,WorkDir \
  --parsable2 --noheader

# Example Output  
12345678|namd_job|COMPLETED|0:0|2025-01-15T10:35:00|2025-01-15T12:00:00|01:25:00|/scratch/alpine/user/namdrunner_jobs/job_001
12345677|namd_job|FAILED|1:0|2025-01-14T14:20:00|2025-01-14T14:22:30|00:02:30|/scratch/alpine/user/namdrunner_jobs/job_000
```

### Job States
- **PD** - Pending (waiting for resources)
- **R** - Running
- **CG** - Completing (cleaning up)
- **CD** - Completed
- **F** - Failed
- **CA** - Cancelled
- **TO** - Timeout
- **NF** - Node Failure
- **PR** - Preempted

### Full Job Information
```bash
# Detailed job info
scontrol show job <job_id>

# Job accounting details
sacct -j <job_id> --format=ALL
```

### Batch Status Queries

When querying status for multiple jobs, batch queries are significantly more efficient than individual queries:

```bash
# Efficient: Single query for multiple jobs (comma-separated IDs)
squeue --job 12345,12346,12347,12348 --Format=jobid,state --noheader

# Inefficient: Multiple individual queries
squeue --job 12345 --Format=jobid,state --noheader
squeue --job 12346 --Format=jobid,state --noheader
squeue --job 12347 --Format=jobid,state --noheader
```

**Performance Comparison:**
- 10 jobs with individual queries: 10 SSH commands
- 10 jobs with batch query: 1 SSH command

**Two-Stage Query Pattern:**

For jobs that may have completed, use a two-stage approach:

1. **Stage 1**: Query `squeue` for all jobs (running/pending show up)
2. **Stage 2**: Query `sacct` for jobs not found in `squeue` (completed jobs)

```bash
# Stage 1: Check active jobs
squeue --job 12345,12346,12347 --Format=jobid,state --noheader

# Stage 2: Check completed jobs (not in squeue results)
sacct -j 12346,12347 --format=JobID,State --noheader
```

**When to Use Batch Queries:**
- ✅ Syncing status for multiple jobs (job sync automation)
- ✅ Dashboard updates showing many jobs
- ✅ Periodic polling of job status
- ❌ Single job status check (no benefit)
- ❌ Interactive user queries (adds complexity)

## Job Management

### Cancel Job

Basic cancellation:
```bash
# Cancel single job
scancel <job_id>

# Cancel all user jobs
scancel -u $USER

# Cancel with state filter
scancel -u $USER --state=PENDING
```

**Job Cancellation Workflow:**

When canceling jobs programmatically (e.g., during job deletion), use conditional cancellation:

```rust
// Only cancel if job is actively running or pending
if matches!(job_info.status, JobStatus::Pending | JobStatus::Running) {
    if let Some(slurm_job_id) = &job_info.slurm_job_id {
        slurm_sync.cancel_job(slurm_job_id).await?;
    }
}
```

**Why conditional?**
- Completed/failed jobs have no SLURM state to cancel
- Avoids unnecessary SSH commands
- More explicit about intent

**Idempotent Cancellation:**

Use `--quiet` flag to avoid errors when job is already completed:

```bash
# Silent success if job already finished
scancel --quiet <job_id>
```

**Error Handling:**

- `scancel` without `--quiet` returns non-zero exit code for completed jobs
- With `--quiet`, it returns success (exit code 0) regardless of job state
- Use `--quiet` for idempotent operations (e.g., cleanup scripts)
- Don't use `--quiet` when you need to detect "job not found" errors

**Integration with Delete Job:**

When user deletes a job via UI:
1. Check job status (Pending/Running/Completed)
2. If Pending or Running: call `scancel` to prevent orphaned cluster jobs
3. Delete from local database
4. Optionally delete remote files

This prevents the critical bug where deleted jobs continue consuming cluster resources.

### Hold/Release Jobs
```bash
# Hold a job (prevent from starting)
scontrol hold <job_id>

# Release a held job
scontrol release <job_id>
```

### Modify Pending Jobs
```bash
# Change time limit
scontrol update JobId=<job_id> TimeLimit=10:00:00

# Change partition
scontrol update JobId=<job_id> Partition=amilan

# Note: Only pending jobs can be modified
```

## Resource Limits (Dynamic Discovery)

**Phase 5+**: Resource limits will be discovered automatically via Settings page using:

```bash
# Query partition information
sinfo --partition=amilan --format="%n %c %m %t %O" --noheader

# Get detailed partition limits
scontrol show partition amilan

# Query QOS information
sacctmgr show qos format=name,priority,maxtres,maxwall --noheader
```

**Legacy Reference** (Alpine Cluster - amilan partition):
| Resource | Minimum | Maximum | Default |
|----------|---------|---------|---------|
| Cores | 1 | 64 | 1 |
| Memory | 1GB | 256GB | 2GB/core |
| Walltime | 00:01:00 | 24:00:00 | 01:00:00 |
| Nodes | 1 | 4 | 1 |

### Common Allocations
```bash
# Test job (quick validation)
#SBATCH --ntasks=4 --mem=4GB --time=00:10:00

# Small simulation
#SBATCH --ntasks=24 --mem=16GB --time=04:00:00

# Production run
#SBATCH --ntasks=48 --mem=32GB --time=24:00:00
```

### QOS Options
- **normal** - Standard priority (default)
- **long** - Extended walltime (up to 7 days, limited availability)
- **high** - Higher priority (requires special access)

## Error Messages

### Common Submission Errors
```bash
# Invalid partition
"sbatch: error: Batch job submission failed: Invalid partition name specified"

# Insufficient resources
"sbatch: error: Batch job submission failed: Requested node configuration is not available"

# Invalid QOS
"sbatch: error: Batch job submission failed: Invalid qos specification"

# Over memory limit
"sbatch: error: Batch job submission failed: Job violates accounting/QOS policy"

# Authentication failure
"sbatch: error: Batch job submission failed: Access denied"
```

### Runtime Errors
```bash
# Out of memory
"slurmstepd: error: Detected 1 oom-kill event(s)"

# Exceeded walltime
"CANCELLED due to DUE TO TIME LIMIT"

# Node failure
"NODE_FAIL"

# Job preempted
"PREEMPTED"
```

### SSH/Connection Errors
```bash
# Connection timeout
"ssh: connect to host login.rc.colorado.edu port 22: Connection timed out"

# Authentication failure
"Permission denied (publickey,password)"

# Module not found
"bash: module: command not found"
# Fix: source /etc/profile
```

### SLURM Error Messages and Solutions

**User-friendly error mapping:**

| SLURM Error | Category | User Message | Actionable Guidance |
|-------------|----------|--------------|---------------------|
| `Invalid partition name specified` | Configuration | Invalid partition selected | Check available partitions for your account |
| `Requested node configuration not available` | Resources | Resource limits exceeded | Reduce cores or memory request |
| `Job violates accounting/QOS policy` | Authorization | Quota or permission issue | Contact cluster administrator |
| `Access denied` | Authentication | Authentication failure | Verify credentials and account status |
| `sbatch: error: Batch job submission failed` | Submission | Job submission failed | Check SLURM script syntax and resource requests |

**Retry behavior:**
- **Retry with backoff**: Transient SLURM scheduler errors, temporary module loading failures
- **Fail immediately**: Invalid partition names, insufficient resource quotas, authentication failures

## SLURM Integration Best Practices

### Command Execution Pattern

All SLURM commands require proper environment initialization:

```bash
source /etc/profile && module load slurm/alpine && <command>
```

**Critical requirements:**
- Always initialize environment (`source /etc/profile`)
- Load required modules before every command
- Parse both stdout AND stderr (errors may appear in either)
- Handle command timeouts gracefully
- Batch related commands when possible to reduce SSH round trips

### Status Caching Strategy

**Cache SLURM query results** to avoid excessive scheduler load:
- Minimum cache TTL: 30-60 seconds
- Cache active job status (`squeue` results)
- Cache completed job status (`sacct` results)
- Invalidate cache on explicit user refresh

**Performance impact:**
- Without caching: Every UI update queries SLURM (can be 10+ queries/second)
- With caching: Queries limited to 1-2 per minute during normal operation
- Reduces scheduler load and improves UI responsiveness

**Implementation pattern:**
```rust
// Cache structure
struct StatusCache {
    last_update: Instant,
    ttl: Duration,
    cached_statuses: HashMap<String, JobStatus>,
}

// Check cache before querying
if cache.is_valid() {
    return cache.get(job_id);
}
// Query SLURM and update cache
let status = query_slurm(job_id);
cache.update(job_id, status);
```

## Directory Patterns

### NAMDRunner Job Identification
Jobs are identified by working directory pattern:
```
/scratch/alpine/$USER/namdrunner_jobs/*
```

### Standard Structure
```
/projects/$USER/namdrunner_jobs/
└── job_001/
    ├── job_info.json
    ├── input_files/
    ├── config.namd
    ├── job.sbatch
    └── outputs/

/scratch/alpine/$USER/namdrunner_jobs/
└── job_001/
    ├── [all job files]
    ├── namd_output.log
    └── *.dcd (trajectory files)
```

## Mock Data for Testing

### Mock squeue Response
```
12345678|test_job|R|00:15:30|01:44:30|1|24|16GB|amilan|/scratch/alpine/testuser/namdrunner_jobs/test_job
```

### Mock sacct Response
```
12345678|test_job|COMPLETED|0:0|2025-01-15T10:00:00|2025-01-15T11:00:00|01:00:00|/scratch/alpine/testuser/namdrunner_jobs/test_job
```

### Mock sbatch Response
```
Submitted batch job 12345678
```

### Mock Error Responses
```bash
# Resource unavailable
"sbatch: error: Batch job submission failed: Requested node configuration is not available"

# Invalid partition
"sbatch: error: Batch job submission failed: Invalid partition name specified"
```

## Best Practices

### Command Execution
1. **Always load modules first** - Critical for command availability
2. **Use full paths** - Avoid relative path issues
3. **Parse both stdout and stderr** - Errors may appear in either
4. **Check exit codes** - Non-zero indicates failure
5. **Handle timeouts** - Network operations can hang

### Performance
1. **Batch queries** - Combine multiple status checks
2. **Cache results** - 30-second minimum cache time
3. **Use --noheader** - Easier parsing without headers
4. **Limit time ranges** - Don't query entire history

### Error Handling
1. **Retry transient failures** - Network issues are common
2. **Fail fast on auth errors** - Don't retry bad credentials
3. **Log full error context** - Include command and full output
4. **Provide clear messages** - Help users understand issues

## Important Notes

- **90-day scratch purge**: Files in `/scratch` deleted after 90 days
- **Queue wait times**: Jobs may be PENDING for hours during busy periods
- **Maintenance windows**: Cluster unavailable during scheduled maintenance
- **Module versions**: May change during cluster updates
- **Network latency**: Each SSH command has ~200ms overhead minimum

This reference is based on proven patterns from production use with the CURC Alpine cluster.