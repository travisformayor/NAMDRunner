# NAMDRunner Documentation

This directory contains the complete NAMDRunner project documentation. Use this guide to find the information you need.

## 🚀 Quick Start

### What is NAMDRunner?
Desktop app for NAMD molecular dynamics on SLURM clusters. Built with Tauri v2 (Rust) + Svelte (TypeScript).

### Setup (5 minutes)
```bash
# Prerequisites: Node.js LTS, Rust toolchain
git clone [repo]
cd namdrunner
npm install

# Launch development app
npm run tauri dev
```

### First Development Task
```bash
# Run tests to verify setup
npm test && cargo test

# Make a small change to verify hot reload works
# Edit src/App.svelte and see changes in app
```

## 📋 Core Documentation

### [`CONTRIBUTING.md`](CONTRIBUTING.md)
**New to the project? Start here.** This guide covers everything you need to be a productive developer on this project, including:
- Development environment setup and platform requirements
- Development principles and our testing philosophy
- Build configuration and deployment constraints
- Code quality standards and performance guidelines

### [`ARCHITECTURE.md`](ARCHITECTURE.md)
**How does this app work?** Read this for a high-level overview of the system design, technology stack, and end-to-end workflows.
- "How this project is built and works"
- Project overview and design principles
- End-to-end workflow and data placement strategy
- Technology choices and module structure
- Security architecture and current implementation

### [`DESIGN.md`](DESIGN.md)
**Building UI?** This is the specification for UI/UX design, component architecture, and page workflows.
- Design philosophy and patterns
- Component architecture
- Page specifications and workflows
- Implementation guidelines

### [`API.md`](API.md)
**Adding a command?** This defines the contract between the frontend and backend, including IPC interfaces, data schemas, and error handling.
- IPC command interfaces
- SLURM integration patterns
- Error handling patterns
- Security implementation

### [`DB.md`](DB.md)
**Touching the database?** This document covers the SQLite schema, JSON metadata formats, and data validation rules.
- SQLite database design
- JSON metadata schemas
- Data validation rules
- Migration and backup strategies

### [`SSH.md`](SSH.md)
**Working with the cluster?** This details SSH/SFTP connection management, security patterns, and file operations.
- Connection lifecycle and state management
- SFTP file operations and optimization
- Security patterns and credential handling
- Error handling and troubleshooting

## 📁 Reference Materials

### [`reference/`](reference/)
**Lookup tables and system details**
- [`agent-development-tools.md`](reference/agent-development-tools.md) - Tooling setup to enable autonomous development for AI assistants
- [`alpine-cluster-reference.md`](reference/alpine-cluster-reference.md) - Complete Alpine cluster information reference (partitions, QoS, resources, etc)
- [`slurm-commands-reference.md`](reference/slurm-commands-reference.md) - SLURM command patterns
- [`namd-commands-reference.md`](reference/namd-commands-reference.md) - NAMD configuration templates
- [`svelte-implementation-guide.md`](reference/svelte-implementation-guide.md) - Svelte patterns and component architecture
- [`python-implementation-reference.md`](reference/python-implementation-reference.md) - Lessons from Python version

## 📝 Documentation Principles

- **One topic, one file** - Each major topic has a single authoritative source
- **Traditional structure** - Familiar open source conventions (CONTRIBUTING.md, etc.)
- **Quick onboarding** - Fast path to productivity for new developers
- **Zero redundancy** - No duplicate information to maintain
- **Task-oriented** - Clear paths for specific developer needs
