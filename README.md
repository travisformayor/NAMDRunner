# NAMDRunner

A desktop application for managing NAMD molecular dynamics simulations on SLURM HPC clusters.

## What is NAMDRunner?

NAMDRunner simplifies the process of running molecular dynamics simulations on remote computing clusters. It provides a user-friendly interface to:

- Connect securely to your HPC cluster
- Configure NAMD simulation parameters
- Submit and monitor SLURM jobs
- Track simulation progress and retrieve results
- Manage multiple projects and simulations

## Key Features

- **Secure SSH Connection** - Authentication with no credential storage
- **Template-Based Configuration** - Pre-configured templates for common simulation types
- **Real-Time Job Monitoring** - Track job status directly from your desktop
- **Local Database** - SQLite-based caching for offline access to job history
- **Cross-Platform** - Runs on Windows, macOS, and Linux

## Installation

### Windows
Download the latest `.exe` installer from the Releases page and run it.

### macOS
Download the `.dmg` file from the Releases page, open it, and drag NAMDRunner to your Applications folder.

### Linux
Download the `.AppImage` file from the Releases page, make it executable, and run:
```bash
chmod +x NAMDRunner-*.AppImage
./NAMDRunner-*.AppImage
```

## Quick Start

1. **Launch NAMDRunner** from your applications menu or desktop
2. **Connect to Cluster** - Enter your cluster hostname and credentials
3. **Configure Simulation** - Set up your NAMD parameters
4. **Submit Job** - Upload files and submit to SLURM queue
5. **Monitor Progress** - Track job status and download results when complete

## Manual Server Management

To manually check on job status or manage files directly on the cluster:

### Using tmux for multiple screens
```bash
# Start a new tmux session
tmux
# Create new screen tab (ctrl+b, then c)
# Move to next/previous tab (ctrl+b, then n or p)
# Split screen horizontally (ctrl+b, then ")
# Split screen vertically (ctrl+b, then %)
# Switch between panes (ctrl+b, then arrow keys)
```

### Checking Job Directories
```bash
# Check scratch workspace (active jobs, 90-day purge)
cd /scratch/alpine/<username>/namdrunner_jobs/
pwd # confirm location with 'print working directory'
ls -lah

# Check projects directory (permanent storage)
cd /projects/<username>/namdrunner_jobs/
pwd # confirm location with 'print working directory'
ls -lah
```

### Monitoring Jobs
```bash
# load slurm if monitoring tools below dont work
module load slurm/alpine

# Watch active jobs (updates every 10 seconds)
watch -n 10 squeue -u <username>

# Watch recent job history (last 24 hours)
watch -n 10 sacct -u <username> --starttime=now-1days

# One-time status checks (no watch)
squeue -u <username>
sacct -u <username> --starttime=now-7days

# Detailed info for specific job
scontrol show job <job_id>
```

### Useful SLURM Commands
```bash
# Cancel a specific job
scancel <job_id>

# Cancel all your jobs
scancel -u <username>

# Check partition availability
sinfo -p amilan

# Check your job priorities
sprio -u <username>
```

### Checking Job Output
```bash
# View live output (while job is running)
tail -f /scratch/alpine/<username>/namdrunner_jobs/<job_id>/<job_name>_*.out

# Check error logs
tail -f /scratch/alpine/<username>/namdrunner_jobs/<job_id>/<job_name>_*.err

# Monitor NAMD progress
tail -f /scratch/alpine/<username>/namdrunner_jobs/<job_id>/namd_output.log
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
