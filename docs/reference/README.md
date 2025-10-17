# Reference Documentation

> **For feature specifications**, see the `docs/` directory:

## File Organization

### Core Reference Documents

- **`agent-development-tools.md`** - Tooling and infrastructure for AI-assisted development
  - Testing infrastructure setup and usage
  - Debugging tools and workflows
  - Mock data management patterns
  - Development environment configuration

- **`alpine-cluster-reference.md`** - Complete Alpine cluster configuration and resource information
  - Partition specifications (amilan, ami100, atesting, etc) with core counts and memory limits
  - QoS options (normal, long, mem) with walltime limits
  - MPI execution patterns and Infiniband configuration
  - Node calculation formulas for optimal resource allocation
  - Memory unit specifications (SLURM uses MB by default, must append 'GB')
  - Module loading sequences for NAMD 3.0.1_cpu

- **`slurm-commands-reference.md`** - Complete SLURM command reference for Alpine cluster
  - All working SLURM commands with exact syntax
  - Module loading sequences
  - Job submission and status checking patterns
  - Resource limits and QOS options
  - Error messages and handling
  - Mock data for testing

- **`namd-commands-reference.md`** - Complete NAMD configuration and execution reference
  - Production-ready NAMD configuration templates
  - Multi-stage DNA origami workflow patterns
  - Command execution patterns for Alpine cluster
  - File organization and parameter validation
  - Error handling and troubleshooting
  - Mock data and test configurations

## How to Use These Files

### When starting a new feature:
1. Check `docs/` for current specifications and architecture
2. Use `alpine-cluster-reference.md` for cluster-specific configuration (partitions, QoS, resources)
3. Use `slurm-commands-reference.md` for SLURM integration patterns and commands
4. Use `namd-commands-reference.md` for NAMD configuration templates and workflows

### Key principles
- **Simplicity wins** - Scientists need reliability over features
- **Demo mode is essential** - Build it from day one
- **Clear errors beat edge case handling** - Users appreciate knowing what went wrong
