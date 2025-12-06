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

**Current Problem**:

- Cluster configuration hardcoded in `cluster.rs` module (970 lines containing partitions, QoS, memory limits, core limits, billing rates)
- Infrastructure exists for profile switching (lazy_static ACTIVE_CLUSTER with RwLock)
- If cluster admin renames partition (e.g., "amilan" → "amilan-ucb") or changes limits, app breaks
- User has no way to fix without code changes and rebuilding

**Current Architecture** (as of Phase 7 complete):

- **Cluster Module** (`src-tauri/src/cluster.rs`): Single source of truth for cluster configuration
  - `alpine_profile()` - Returns hardcoded Alpine cluster configuration
  - `get_cluster_capabilities()` - Returns capabilities for frontend (already has #[tauri::command])
  - `get_partition_limits()`, `calculate_job_cost()`, `suggest_qos()`, `estimate_queue_time()` - Business logic functions
  - Direct registration: Commands call cluster module directly (no wrapper layer after cleanup)
- **NO** `commands/cluster.rs` file exists (thin wrappers deleted during code cleanup)
- **Validation Module** (`src-tauri/src/validation/job_validation.rs`): Resource validation using cluster config
- **Frontend Store** (`src/lib/stores/clusterConfig.ts`): Caches capabilities, invokes cluster commands directly

**Implementation**:

- [ ] **Database Schema** (`cluster_settings` table - key/value store pattern):
  - [ ] `key TEXT PRIMARY KEY` - Setting identifier (e.g., 'billing_rate_cpu', 'partition_amilan_max_cores')
  - [ ] `value TEXT NOT NULL` - Setting value (JSON for complex types)
  - [ ] `updated_at TEXT NOT NULL` - Timestamp
  - [ ] Alternative: Structured tables (`cluster_partitions`, `cluster_qos`) - decide based on query patterns
  - [ ] Initialize table with current hardcoded values on first run via migration
  - [ ] Schema supports multiple cluster profiles (store profile_id in key, e.g., 'alpine.billing_rate_cpu')

- [ ] **Backend Implementation**:
  - [ ] Keep `cluster.rs` as business logic module (calculations, validations)
  - [ ] Add database loading: `load_cluster_settings_from_db()` called during app initialization
  - [ ] Merge DB settings with hardcoded defaults: DB overrides hardcoded values
  - [ ] Add Tauri commands (register directly in cluster.rs with #[tauri::command]):
    - [ ] `get_editable_cluster_config()` - Returns current configuration for Settings UI
    - [ ] `update_partition(partition_config)` - Updates single partition in DB
    - [ ] `update_qos(qos_config)` - Updates single QoS option in DB
    - [ ] `update_billing_rates(cpu_rate, gpu_rate)` - Updates billing configuration
    - [ ] `reset_cluster_config_to_defaults()` - Clears DB settings, reverts to hardcoded defaults
  - [ ] **NO** `commands/cluster.rs` file - register commands directly from cluster module
  - [ ] Update `alpine_profile()` to merge DB settings with hardcoded defaults on load

- [ ] **Settings Page UI** (extend existing SettingsPage.svelte):
  - [ ] Add "Cluster Configuration" section with tabs
  - [ ] **Partitions Tab**: Card-based list of partitions with inline edit capability
    - [ ] Each partition card: name, description, max cores, max memory, max walltime, default QoS
    - [ ] Edit button opens inline form with validation
    - [ ] Save button calls `update_partition()` command
  - [ ] **QoS Options Tab**: Card-based list with inline edit
    - [ ] Each QoS card: name, description, compatible partitions (chips/tags display)
    - [ ] Edit form with multi-select for compatible partitions
  - [ ] **Billing Rates Tab**: Simple form for CPU/GPU rates
  - [ ] **Reset to Defaults** button with ConfirmDialog (uses existing Dialog.svelte primitive)
  - [ ] Use existing design system (namd-button classes, CSS variables)

- [ ] **Migration Strategy**:
  - [ ] Database migration in `database/mod.rs`: Create `cluster_settings` table if not exists
  - [ ] On app startup in `main.rs`: If table empty, populate with `alpine_profile()` values
  - [ ] Load settings from DB → merge with defaults → set active profile in ACTIVE_CLUSTER
  - [ ] Existing validation and calculation functions automatically use updated values (no code changes needed)

**Why**: Cluster admins change configuration over time. User-editable settings prevent app breakage and eliminate need for code changes/rebuilds.

**Architecture Notes**:

- Cluster module remains single source of truth for business logic
- Database provides persistence layer for user customizations
- No thin wrapper layer - Settings UI invokes cluster commands directly
- Hardcoded defaults serve as fallback when DB settings not present
- Profile switching infrastructure already in place via lazy_static RwLock

#### Milestone 8.2: App/Module Discovery and Management

**Goal**: Allow users to search for cluster applications (e.g., NAMD), discover prerequisites, and pin specific versions for use in jobs. Replace hardcoded module versions in SLURM scripts with user-selected configurations.

**Current Problem**:

- Module versions hardcoded in job script generation (namd/3.0.1_cpu, gcc/14.2.0, openmpi/5.0.6)
- If cluster admin updates NAMD or changes prerequisite versions, all jobs fail
- User must know correct module names and load order (requires SSH + manual `module spider` commands)
- No support for multiple NAMD versions (e.g., CPU-optimized vs GPU-optimized)

**Implementation**:

**Backend - Module Discovery** (`commands/cluster_apps.rs`):

- [ ] `search_cluster_apps(search_term: string) -> ApiResult<Vec<AppSearchResult>>`
  - [ ] Run `module avail <search_term> 2>&1` via SSH
  - [ ] Parse output to extract available module names/versions
  - [ ] Return list: `[{ name: "namd", version: "3.0.1_cpu", full_name: "namd/3.0.1_cpu" }, ...]`

- [ ] `pin_cluster_app(module_name: string) -> ApiResult<PinnedApp>`
  - [ ] Run `module spider <module_name>` to get immediate prerequisites
  - [ ] **Recursively** run `module spider` on each prerequisite to build full dependency tree
  - [ ] Build directed acyclic graph (DAG) of all dependencies
  - [ ] Perform topological sort to determine correct load order (deepest dependencies first)
  - [ ] Detect circular dependencies (return error if found)
  - [ ] Validate all modules in chain exist on cluster
  - [ ] Store in DB: app_name, version, full_module_name, load_order (array), dependency_tree (JSON)
  - [ ] Return pinned app data to UI

- [ ] `get_pinned_apps() -> ApiResult<Vec<PinnedApp>>`
  - [ ] Return all pinned apps from DB for display in UI and job creation

- [ ] `unpin_cluster_app(id: string) -> ApiResult<()>`
  - [ ] Delete pinned app from DB
  - [ ] Check if any jobs reference this app (warn user but allow deletion)

- [ ] `validate_pinned_app(id: string) -> ApiResult<ValidationResult>`
  - [ ] Test that `module purge && module load <load_order>` succeeds
  - [ ] Useful for user to verify configuration after cluster changes

**Backend - Module Parsing Utilities** (`ssh/module_parser.rs`):

- [ ] Parse `module avail` output (format varies by cluster, needs robust regex)
- [ ] Parse `module spider` output to extract dependencies
- [ ] Handle edge cases: missing modules, version conflicts, circular deps
- [ ] Unit tests for various output formats

**Database Schema** (`pinned_apps` table):

- [ ] `id` TEXT PRIMARY KEY
- [ ] `app_name` TEXT NOT NULL (e.g., "namd")
- [ ] `version` TEXT NOT NULL (e.g., "3.0.1_cpu")
- [ ] `full_module_name` TEXT NOT NULL (e.g., "namd/3.0.1_cpu")
- [ ] `display_name` TEXT (user-friendly label, e.g., "NAMD 3.0.1 (CPU-optimized)")
- [ ] `load_order` TEXT NOT NULL (JSON array: `["gcc/14.2.0", "openmpi/5.0.6", "namd/3.0.1_cpu"]`)
- [ ] `dependency_tree` TEXT (JSON for full DAG, useful for debugging)
- [ ] `pinned_at` TEXT (timestamp)
- [ ] `last_validated_at` TEXT (timestamp, nullable)

**Job Metadata Breaking Change**:

- [ ] Update `JobInfo` struct to include `app_module` field:

  ```rust
  pub struct JobInfo {
      // ... existing fields
      pub app_module: AppModule,
  }

  pub struct AppModule {
      pub app_name: String,
      pub version: String,
      pub full_module_name: String,
      pub display_name: String,
  }
  ```

- [ ] Update `job_info.json` format on cluster to include app_module
- [ ] Update SLURM script generation to use `load_order` from selected pinned app

**Settings Page UI** (new "Applications" section):

- [ ] **App Search**:
  - [ ] Search input field: "Search for cluster applications (e.g., 'namd', 'gcc', 'python')"
  - [ ] Search button triggers `search_cluster_apps()` command
  - [ ] Display results in list with "Pin" button for each result
  - [ ] Show loading state during search (SSH command can take 1-2 seconds)

- [ ] **Pinned Apps List**:
  - [ ] Card-based display of all pinned apps
  - [ ] Each card shows: display_name, full_module_name, load_order preview, pinned date
  - [ ] Actions: Edit display name, Validate (test load order), Unpin (delete)
  - [ ] Expand card to show full dependency tree and load order
  - [ ] "Validate" button runs SSH test and shows success/failure

- [ ] **Pin Dialog** (when user clicks "Pin" on search result):
  - [ ] Show loading: "Discovering prerequisites for <module>..."
  - [ ] On success: Show full load order, prompt for display name
  - [ ] On error: Show which module failed (e.g., "Prerequisite gcc/14.2.0 not found")
  - [ ] Confirm button saves to DB

**Job Creation Integration** (`CreateJobPage.svelte` - Resources tab):

- [ ] Add "Application" dropdown below partition/QoS selection
- [ ] Load pinned apps from DB on page mount
- [ ] Dropdown options: display_name (e.g., "NAMD 3.0.1 (CPU-optimized)")
- [ ] Default to first pinned app or show "No applications pinned" message
- [ ] If no apps pinned, show link to Settings page
- [ ] Store selected app ID in job draft state

**Job Details Integration** (`JobDetailPage.svelte` - Overview tab):

- [ ] Display app_module information in metadata section
- [ ] Show: "Application: NAMD 3.0.1 (CPU-optimized) [namd/3.0.1_cpu]"
- [ ] Show full load order in expandable section

**Implementation Notes**:

- **No auto-refresh**: User manually searches and re-pins when cluster changes (acceptable workflow)
- **Discovery is expensive**: SSH commands take time, only run on explicit user action
- **Multiple pinned apps expected**: Users may pin CPU and GPU versions, or different major versions
- **Validation optional but recommended**: User can test pinned app still works after cluster changes
- **Circular dependency handling**: Detect and reject during pin operation (rare but possible)

**Why**: Cluster admins update software versions regularly. User-managed module configuration eliminates hardcoded dependencies and allows app to adapt to cluster changes without code modifications.

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
