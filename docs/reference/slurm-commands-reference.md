# SLURM Commands Reference

> **📚 For cluster-specific details**, see `docs/cluster-guide.md`
> **🔧 For implementation patterns**, see `python-implementation-reference.md`
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

### SLURM Script Template
```bash
#!/bin/bash
#SBATCH --job-name={{ job_name }}
#SBATCH --output={{ job_name }}_%j.out
#SBATCH --error={{ job_name }}_%j.err
#SBATCH --partition=amilan
#SBATCH --nodes=1
#SBATCH --ntasks={{ num_cores }}
#SBATCH --time={{ walltime }}
#SBATCH --mem={{ memory }}
#SBATCH --qos=normal
#SBATCH --constraint=ib

module purge
module load gcc/14.2.0
module load openmpi/5.0.6
module load namd/3.0.1_cpu

cd {{ working_dir }}

mpirun -np $SLURM_NTASKS namd3 +setcpuaffinity +pemap 0-{{ num_cores - 1 }} {{ namd_config }} > {{ namd_log }}
```

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

## Job Management

### Cancel Job
```bash
# Cancel single job
scancel <job_id>

# Cancel all user jobs
scancel -u $USER

# Cancel with state filter
scancel -u $USER --state=PENDING
```

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