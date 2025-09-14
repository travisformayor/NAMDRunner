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

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
