# NAMDRunner Documentation

This directory contains the complete NAMDRunner project documentation. Use this guide to find the information you need.

## Core Documentation

### [`docs/CONTRIBUTING.md`](CONTRIBUTING.md)
**New to the project? Start here.** This guide covers everything you need to be a productive developer on this project, including:
- Development environment setup and platform requirements
- Development principles and our testing philosophy
- Build configuration and deployment constraints
- Code quality standards and performance guidelines

### [`docs/ARCHITECTURE.md`](ARCHITECTURE.md)
**How does this app work?** Read this for a high-level overview of the system design, technology stack, and end-to-end workflows.
- "How this project is built and works"
- Project overview and design principles
- End-to-end workflow and data placement strategy
- Technology choices and module structure
- Security architecture and current implementation

### [`docs/API.md`](API.md)
**Adding a command?** This defines the contract between the frontend and backend, including IPC interfaces, data schemas, and error handling.
- IPC command interfaces
- SLURM integration patterns
- Error handling patterns
- Security implementation

### [`docs/AUTOMATIONS.md`](AUTOMATIONS.md)
**Working on job lifecycle automation?** This document covers the complete automation architecture for job creation, submission, monitoring, completion, and cleanup.
- Job lifecycle automation chains
- Progress tracking system
- Atomic operations and error handling
- Real-time UI feedback via Tauri events

### [`docs/DB.md`](DB.md)
**Touching the database?** This document covers the SQLite schema, JSON metadata formats, and data validation rules.
- SQLite database design
- JSON metadata schemas
- Data validation rules
- Migration and backup strategies

### [`docs/DESIGN.md`](DESIGN.md)
**Building UI?** This is the specification for UI/UX design, component architecture, and page workflows.
- Design philosophy and patterns
- Component architecture
- Page specifications and workflows
- Implementation guidelines

### [`docs/SSH.md`](SSH.md)
**Working with the cluster?** This details SSH/SFTP connection management, security patterns, and file operations.
- Connection lifecycle and state management
- SFTP file operations and optimization
- Security patterns and credential handling
- Error handling and troubleshooting

## Reference Materials

### [`docs/reference/`](reference/)
**Commands and configuration details**
- [`agent-development-tools.md`](reference/agent-development-tools.md) - Tooling setup to enable autonomous development for AI assistants
- [`alpine-cluster-reference.md`](reference/alpine-cluster-reference.md) - Complete Alpine cluster information (partitions, QoS, resources, MPI patterns)
- [`namd-commands-reference.md`](reference/namd-commands-reference.md) - NAMD configuration templates and multi-stage workflow patterns
- [`slurm-commands-reference.md`](reference/slurm-commands-reference.md) - SLURM command patterns and integration best practices

## Supplementary Documentation

### Platform-Specific Guides
- [`VM_SETUP.md`](VM_SETUP.md) - Virtual machine development environment setup (optional, geared towards macOS users)
- [`WINDOWS_BUILD.md`](WINDOWS_BUILD.md) - Windows-specific build configuration, MSI packaging, and troubleshooting

## Documentation Principles

- **One topic, one file** - Each major topic has a single authoritative source
- **Traditional structure** - Familiar open source conventions (CONTRIBUTING.md, etc.)
- **Quick onboarding** - Fast path to productivity for new developers
- **Zero redundancy** - No duplicate information to maintain
- **Task-oriented** - Clear paths for specific developer needs
