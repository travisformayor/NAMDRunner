# Python Implementation Reference

> **📚 For current specifications**, see the `docs/` directory:
> - [`docs/API.md`](../API.md) - IPC interfaces, data schemas, and SLURM integration patterns
> - [`docs/CONTRIBUTING.md#testing-strategy`](../CONTRIBUTING.md#testing-strategy) - Testing strategies and error handling

This document consolidates all lessons learned from 18 months of Python/CustomTkinter NAMDRunner development. These insights should guide the Tauri implementation while avoiding past mistakes.

## Table of Contents
- [Architecture Patterns](#architecture-patterns)
- [Data Management](#data-management)
- [SLURM Integration](#slurm-integration)
- [Features Implemented](#features-implemented)
- [Development Process](#development-process)
- [Performance Insights](#performance-insights)
- [What to Avoid](#what-to-avoid)

## Architecture Patterns

### Manager Separation Pattern ✅
The Python version successfully separated concerns into distinct managers:
- **SSHManager**: Handled all SSH/SFTP operations
- **JobManager**: Managed job lifecycle and metadata
- **CacheManager**: SQLite operations and offline state

**Why it worked:**
- Clean boundaries made unit testing straightforward
- Each manager could be mocked independently
- Easy to reason about where functionality belonged
- Prevented circular dependencies

**Recommendation for Tauri:** Keep this separation pattern in Rust modules.

### Offline-First with Cache ✅
- Local SQLite database served as source of truth for UI
- Remote operations updated cache asynchronously
- Users could view jobs without active connection

**Benefits discovered:**
- UI remained responsive during network operations
- Reduced unnecessary SLURM queries
- Worked well on unreliable connections
- Simplified state management

### Mock Mode for Development ✅
The `NAMDRUNNER_DEV=true` environment flag enabled complete offline development:
```python
if os.getenv("NAMDRUNNER_DEV"):
    return MockSSHManager()
else:
    return RealSSHManager()
```

**Why this was critical:**
- Developers could work without cluster access
- UI development didn't require SSH setup
- Tests ran consistently without network
- Demo mode for presentations

### UI-as-Data Testing Pattern ✅
Captured widget states as JSON for deterministic testing:
```python
def capture_ui_state():
    return {
        "job_table": [row.to_dict() for row in table],
        "buttons": {name: widget.enabled for name, widget in buttons},
        "status": status_label.text
    }
```

This enabled ~160 comprehensive unit tests that gave confidence during refactoring.

## Data Management

### SQLite Schema Evolution
The Python version went through several schema iterations:

**Initial (single jobs):**
- Simple jobs table with basic fields
- Worked well for Phase 1

**Multi-stage addition (complex):**
- Added job_groups, job_stages tables
- UUID-based group IDs
- Became overly complex for the use case

**Lessons:**
- Start simple, evolve based on real needs
- Version your schema from day one
- Keep migration paths simple

### JSON Metadata Pattern ✅
Each job had a `job_info.json` file on the cluster:
```json
{
  "schema_version": "1.0",
  "job_id": "job_001",
  "status": "COMPLETED",
  // ... full metadata
}
```

**Why this worked:**
- Schema versioning prevented breaking changes
- Human-readable for debugging
- Easy to migrate between versions
- Worked as backup if SQLite corrupted

### Directory Structure
Used two-directory pattern that worked reliably:
```
/projects/$USER/namdrunner_jobs/  # Persistent storage
/scratch/$USER/namdrunner_jobs/   # Execution directory
```

**Key insight:** Keep these paths configurable - hardcoding caused issues when cluster paths changed.

## SLURM Integration

### SSH Command Pattern ✅
The most reliable pattern discovered:
```python
def execute_command(ssh_client, command):
    full_command = f"source /etc/profile && module load slurm/alpine && {command}"
    stdin, stdout, stderr = ssh_client.exec_command(full_command)
    return stdout.read().decode(), stderr.read().decode()
```

**Critical discoveries:**
- Always source profile and load modules
- Parse both stdout AND stderr
- Handle command timeouts gracefully
- Batch commands when possible

### Status Checking Strategy
Combined `squeue` (active jobs) and `sacct` (completed jobs):
- `squeue` for PENDING/RUNNING states
- `sacct` for COMPLETED/FAILED states
- Cache results for 30 seconds minimum

### Performance Optimizations
- **Batch SLURM queries**: One SSH connection for multiple queries
- **Background polling**: Non-blocking UI updates
- **Status caching**: Avoid repeated queries for same job
- **Connection pooling**: Reuse SSH connections when possible

### Error Handling Patterns
**Retry with exponential backoff for:**
- SSH connection failures
- Network timeouts
- Module loading issues

**Fail immediately for:**
- Invalid credentials
- Incorrect partition names
- Insufficient resources
- Disk space issues

## Features Implemented

### Core Features (Working in Python)
- ✅ SSH connection via username/password
- ✅ Offline/Online modes with manual connection
- ✅ Create NAMD simulation jobs
- ✅ Upload PDB, PSF, parameter files
- ✅ Generate NAMD configs from templates
- ✅ Submit jobs to SLURM
- ✅ Track job status
- ✅ View job outputs
- ✅ Local SQLite cache

### NAMD-Specific Features
- ✅ Three job types: Structure Optimization, Multi-Stage Equilibration, General
- ✅ Parameter validation (temperature, timestep ranges)
- ✅ Template-based config generation (Jinja2)
- ✅ Support for DNA origami simulations

### UI Components
- ✅ Main dashboard with job table
- ✅ Job creation dialog with file upload
- ✅ Resource allocation controls
- ✅ Tabbed output viewer
- ✅ Color-coded status badges
- ✅ Connection status indicator

### Missing Features (Never Implemented)
- ❌ Job discovery (only tracked created jobs)
- ❌ Multi-stage workflows (critical for real NAMD)
- ❌ Restart capabilities
- ❌ Job modification after submission
- ❌ Batch job operations

## Development Process

### Task Management Success
- **One task at a time** discipline prevented scope creep
- **Comprehensive documentation** helped context switching
- **Task templates** ensured consistency
- **Milestone-based planning** kept focus

### Testing Strategy
- **~160 unit tests** provided refactoring confidence
- **Mock SSH** enabled offline testing
- **UI-as-Data** pattern made UI testing deterministic
- **Integration tests** with real cluster (manual)

## Performance Insights

### Bottlenecks Discovered
- **SFTP uploads**: Large PDB files (>50MB) took 2-3 minutes
- **Network latency**: ~200ms overhead per SSH command
- **SLURM queries**: `sacct` could take 5-10 seconds when busy
- **UI blocking**: Synchronous operations froze interface

### Solutions That Worked
- **Background threads** for all SSH operations
- **Progress indicators** for long operations
- **Chunked file uploads** with progress callbacks
- **Status caching** with 30-second TTL

## What to Avoid

### Technology Pitfalls
- **CustomTkinter limitations**: Hard to create modern UI
- **PyInstaller complexity**: Fragile packaging, 150MB+ executables
- **Python threading**: Complex SSH concurrency issues
- **GIL limitations**: Performance bottlenecks

### Architecture Anti-Patterns
- **Complex agent hierarchies**: Overcomplicated documentation
- **Scattered configuration**: Settings in multiple files
- **Hardcoded values**: Module versions, paths, timeouts
- **Overly generic abstractions**: YAGNI principle applies

### Process Mistakes
- **No early Windows testing**: Discovered issues late
- **Skipped integration tests**: Manual testing insufficient
- **Ignored performance early**: Retrofitting was painful
- **Documentation scattered**: Multiple sources of truth

## Critical Success Factors

1. **Maintain simplicity**: Scientists need reliability over features
2. **Preserve workflows**: Users have muscle memory for patterns
3. **Prioritize compatibility**: Must work with existing cluster
4. **Focus on core features**: Perfect basics before advanced features
5. **Document everything**: Future maintainers need clarity

## Key Recommendations for Tauri

### Architecture
- Keep manager separation pattern (proven to work)
- Use Rust's type system to enforce boundaries
- Implement async from the start
- Build mock mode early

### Data Management
- Start with simple schema, evolve carefully
- Version everything from day one
- Keep JSON metadata for debugging
- Make paths configurable

### UI Development
- Leverage Svelte's reactivity
- Implement loading states everywhere
- Use progressive disclosure for complex forms
- Test on Windows early and often

### Testing Strategy
- Port UI-as-Data pattern to Svelte
- Use Rust's built-in testing
- Maintain mock mode
- Automate integration tests if possible

## Summary

The Python implementation taught us that **simplicity, reliability, and clear separation of concerns** are more important than advanced features. The Tauri version should:

1. **Keep what worked**: Manager pattern, offline-first, mock mode
2. **Fix what didn't**: Better performance, modern UI, smaller executables
3. **Add what's missing**: Job discovery, multi-stage workflows, batch operations
4. **Avoid complexity**: Simple is better than clever

Remember: Scientists need a tool that works reliably for months without maintenance. Every design decision should prioritize stability and clarity over elegance or advanced features.