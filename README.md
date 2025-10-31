# NAMDRunner

NAMDRunner is a desktop application for running and managing NAMD molecular dynamics simulations on SLURM HPC clusters. Built with Tauri v2 (Rust) and Svelte (TypeScript), it simplifies the simulation workflow by providing a user-friendly interface to:

- Connect securely to your HPC cluster
- Configure NAMD simulation parameters
- Submit and monitor SLURM jobs
- Track simulation progress and retrieve results
- Manage multiple simulations

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

## Development Quick Start

### Setup
```bash
# Prerequisites: Node.js LTS, Rust toolchain
git clone [repo]
cd namdrunner
npm install

# Launch development app
npm run tauri dev
```

## Manual Server Management

To manually check on job status or manage files directly on the cluster:

```bash
# (Optional) Start a tmux session to have multiple windows
tmux
# Split screen horizontally (ctrl+b, then ")
# Split screen vertically (ctrl+b, then %)
# Switch between panes (ctrl+b, then arrow keys)

# == Check Job Files
# Projects directory (permanent storage)
cd /projects/<username>/namdrunner_jobs/
pwd # confirm location with 'print working directory'
ls -lah

# Scratch directory (temp storage for running jobs, 90-day purge)
cd /scratch/alpine/<username>/namdrunner_jobs/
pwd # confirm location with 'print working directory'
ls -lah

# == Check Job Status
# Load slurm (optional: if the commands below dont work)
module load slurm/alpine

# Watch active jobs (updates every 10 seconds)
watch -n 15 squeue -u <username>

# Watch recent job history (last 24 hours)
watch -n 15 sacct -u <username> --starttime=now-1days

# One-time status checks (no watch)
squeue -u <username>
sacct -u <username> --starttime=now-7days

# Detailed info for specific job
scontrol show job <job_id>

# == Monitor Job Output and Logs
# View live output (while job is running)
tail -f /scratch/alpine/<username>/namdrunner_jobs/<job_id>/<job_name>_*.out

# Monitor NAMD progress
tail -f /scratch/alpine/<username>/namdrunner_jobs/<job_id>/namd_output.log

# Check error logs
tail -f /scratch/alpine/<username>/namdrunner_jobs/<job_id>/<job_name>_*.err

# == Additional SLURM Commands
# Cancel a specific job
scancel <job_id>

# Cancel all your jobs
scancel -u <username>

# Check partition availability
sinfo -p amilan

# Check your job priorities
sprio -u <username>
```


## License

This project is licensed under the MIT License - see [LICENSE](LICENSE).
