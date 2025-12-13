# NAMDRunner Development Roadmap

**Current Status**: Phase 7 Complete ✅ | **Next**: Phase 8 - Settings Page Configuration

**Architecture Reference**: See [`docs/ARCHITECTURE.md`](../docs/ARCHITECTURE.md) for current implementation details.

---

## Completed Phases

### Phase 1: Foundation ✅

Tauri v2 + Svelte scaffold, IPC interfaces, mock infrastructure, and SSH/SFTP patterns.

- Milestone 1.1: Project Scaffold → [task plan](tasks/completed/phase1-milestone1.1-foundation.md)
- Milestone 1.2: Mock Infrastructure → [task plan](tasks/completed/phase1-milestone1.2-mock-infrastructure.md)
- Milestone 1.3: Connection Foundation → [task plan](tasks/completed/phase1-milestone1.3-connection-foundation.md)
- Milestone 1.4: Cleanup & Refactoring (no separate task plan)

### Phase 2: Core Backend ✅

SSH/SFTP implementation, job directory lifecycle, retry logic, SLURM integration, and SQLite persistence.

- Milestone 2.1: SSH/SFTP Implementation → [task plan](tasks/completed/phase2-milestone2.1-ssh-sftp-implementation.md)
- Milestone 2.2: Critical Fixes & Enhancements → [task plan](tasks/completed/phase2-milestone2.2-ssh-sftp-critical-fixes.md)
- Milestone 2.3: Job Status Synchronization → [task plan](tasks/completed/phase2-milestone2.3-job-status-synchronization.md)
- Milestone 2.4: Cleanup & Refactoring (no separate task plan)

### Phase 3: Frontend Development ✅

Complete UI implementation matching React mockup: application shell, jobs management, job creation workflow, connection interface, and theme support.

- Full phase: [task plan](tasks/completed/phase3-ui-implementation.md)

### Phase 4: SLURM Integration ✅

Real job submission via sbatch, status tracking via squeue/sacct, and database persistence across restarts.

- Full phase: [task plan](tasks/completed/phase4-slurm-job-submission.md)

### Phase 5: File Operations & Results ✅

Real SFTP upload/download, results browsing, log aggregation, and job cleanup with ~20% code reduction.

- Full phase: [task plan](tasks/completed/phase5-file-operations-results-management.md)

### Phase 6: Integration & Polish ✅

End-to-end workflow completion: UI-backend integration, automation chains, SLURM log caching, job lifecycle reliability, NAMD config fixes, and pragmatic testing.

- Milestone 6.1: UI Integration & Connection Stability → [task plan](tasks/completed/phase-6-1-ui-backend-integration.md)
- Milestone 6.2: Job Automation Verification → [task plan](tasks/completed/phase-6-2-automation-verification.md)
- Milestone 6.3: Code Quality & Refactoring → [task plan](tasks/completed/phase-6-3-code-quality-refactoring.md)
- Milestone 6.4: Frontend-Backend Integration → [task plan](tasks/completed/phase-6-4-frontend-backend-integration.md)
- Milestone 6.5: Infrastructure Cleanup → [task plan](tasks/completed/phase-6-5-code-quality-infrastructure-cleanup.md)
- Milestone 6.6: Job Lifecycle Reliability → [task plan](tasks/completed/phase-6-6-job-lifecycle-reliability-bug-fixes.md)
- Milestone 6.7: Template Type 2 NAMD Config → [task plan](tasks/completed/phase-6-7-template-type-2-namd-config-fixes.md)
- Milestone 6.8: Pragmatic Testing → [task plan](tasks/completed/phase-6-8-pragmatic-testing.md)
- Milestone 6.9: Production Readiness (deferred to future work)

### Phase 7: Template System & Settings ✅

Template-based job creation replacing hardcoded NAMD config, plus Settings page with database management and theme unification.

- Milestone 7.1: Template System Refactor → [task plan](tasks/completed/phase-7-1-template-system-refactor.md)
- Milestone 7.2: Settings Page & Database Management → [task plan](tasks/completed/phase-7-2-db-settings-page-and-theming.md)

---

## Active Development

### Phase 8: Settings Page - Cluster & App Configuration

User-configurable cluster settings and application module management.

**Context**: Currently cluster configuration (partitions, QoS, resource limits) and application modules (NAMD versions, prerequisites) are hardcoded in Rust. If cluster admins rename partitions, change limits, or update module versions, the app breaks. This phase makes all cluster-specific configuration user-editable to future-proof the application.

**Breaking Changes**: New database tables (`cluster_configs`, `pinned_apps`), job metadata schema changes to include `app_module` field. No backwards compatibility needed (app not yet released, user will delete old database).

#### Milestone 8.1: User-Editable Cluster Configuration

**Goal**: Move hardcoded cluster settings from Rust constants to database, allowing users to edit partitions, QoS options, and resource limits via Settings page.

**Why**: Cluster admins change configuration over time. User-editable settings prevent app breakage and eliminate need for code changes/rebuilds.

**Implementation**: See detailed task plan in `tasks/completed/phase-8-1-user-editable-cluster-config.md`

#### Milestone 8.2: App/Module Discovery and Management

**Goal**: Allow users to search for cluster applications (e.g., NAMD), discover prerequisites, and pin specific versions for use in jobs. Replace hardcoded module versions in SLURM scripts with user-selected configurations.

**Why**: Cluster admins update software versions regularly. User-managed module configuration eliminates hardcoded dependencies and allows app to adapt to cluster changes without code modifications.

**Implementation**: See detailed task plan in `tasks/active/phase-8-2-app-module-manual-entry.md`

#### Phase 8 Complete When

- User can edit cluster configuration (partitions, QoS, limits) via Settings page
- User can search, pin, and manage cluster applications with automatic prerequisite discovery
- All jobs use user-selected apps with dynamic module loading
- Zero hardcoded cluster or module configuration remaining in codebase

---

## Future Work

Features planned for post-Phase 8 development. Priorities determined by user feedback.

### Production Readiness

- [ ] Installation documentation
- [ ] User guide (template-based job workflow)
- [ ] Final documentation completeness check and cleanup

### Request Rate Limiting & Queue Management

**Goal:** Prevent cluster abuse and provide graceful degradation under load

**Current State:** Mutex serialization provides implicit rate limiting (one request at a time). Single SSH connection physically prevents parallel spam. Adequate for current usage.

**When Needed:** If users report accidental DOS of cluster or app becomes unresponsive under load

**Approach:** Token bucket rate limiter wrapping existing ConnectionManager mutex, request deduplication, queue depth limits

### Job Chaining / Multi-Stage Workflows

**Note**: Design in progress at [tasks/planning/Job_Chaining.md](../tasks/planning/Job_Chaining.md)

**Core Concept**: "Restart after timeout" and "next equilibration stage" are the same mechanism - creating new job that continues from parent job's outputs. Jobs are self-contained islands (each copies necessary files from parent).

**When Needed:** Users need to run multi-stage simulations (minimization → equilibration → production) or restart jobs that hit walltime limits

**Approach:** Parent-child job relationships, file propagation system, chain visualization UI

### Multi-Cluster Support

**Goal:** Support users with accounts on multiple clusters

**Dependencies:** Phase 8.1 cluster configuration must be complete first

**Approach:**

- Multiple cluster profiles in `cluster_configs` table
- Profile switcher in connection UI
- Profile-specific pinned apps
- Migration of connection management to support profile selection

### Automation Builder

**Goal:** Visual workflow designer for complex job automation patterns

**Dependencies:** Builds on existing Phase 6 automation framework

**Approach:**

- Serializable automation steps (already implemented in Rust)
- Drag-and-drop workflow canvas
- Automation template library
- Parameter sweep automation
- Community template marketplace

### UI/UX Enhancements

- Bulk operations (multi-select job management)
- Advanced filtering/search (by status, date, resources, templates)
- User preferences (default values, UI behavior)
- Export/import jobs and templates
- Job comparison and diff tools
