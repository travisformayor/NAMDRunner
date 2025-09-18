# Reference Documentation

> **📚 For current specifications**, see the `docs/` directory:
> - [`docs/API.md`](../API.md) - IPC interfaces, data schemas, and command specifications
> - [`docs/SSH.md`](../SSH.md) - SSH/SFTP connection management, security patterns, and implementation details
> - [`docs/reference/alpine-cluster-reference.md`](alpine-cluster-reference.md) - CURC Alpine cluster details
> - [`docs/CONTRIBUTING.md#testing-strategy`](../CONTRIBUTING.md#testing-strategy) - Testing strategies and error handling

This directory contains reference materials from the Python NAMDRunner implementation that inform the Tauri version development.

## File Organization

### Core Reference Documents

- **`python-implementation-reference.md`** - Comprehensive lessons from 18 months of Python development
  - Architecture patterns that worked (manager separation, offline-first, mock mode)
  - Data management insights (SQLite, JSON, file sync)
  - SLURM integration lessons (SSH patterns, performance, error handling)
  - Complete feature list from Python implementation
  - Development process insights
  - Anti-patterns and pitfalls to avoid

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
1. Check `docs/` for current specifications (authoritative)
2. Review `python-implementation-reference.md` for relevant lessons
3. Use `slurm-commands-reference.md` for SLURM integration patterns
4. Use `namd-commands-reference.md` for NAMD configuration and workflows
5. Apply insights but adapt to Rust/Tauri's strengths

### When debugging issues:
1. Check if Python encountered similar problems
2. Look for solutions in `python-implementation-reference.md`
3. Verify SLURM commands against `slurm-commands-reference.md`
4. Consider if Rust/Tauri offers better approaches

### Key principles from Python experience:
- **Simplicity wins** - Scientists need reliability over features
- **Mock mode is essential** - Build it from day one
- **Performance matters** - Pagination and lazy loading are critical
- **Clear errors beat edge case handling** - Users appreciate knowing what went wrong

## Important Notes

- These are **lessons and insights**, not requirements
- The Tauri implementation should **improve upon** these patterns
- Focus on the **principles** behind the patterns, not literal implementation
- Some Python-specific issues (GIL, CustomTkinter limitations) don't apply to Rust/Tauri

## What's NOT Here

We've intentionally excluded:
- **UI patterns** - Svelte is completely different from CustomTkinter
- **Deployment/packaging** - Tauri handles this differently than PyInstaller
- **Security details** - Already covered in `docs/` specifications
- **Python-specific workarounds** - Not relevant to Rust

The goal is to learn from Python development experience while building something better with modern tools.