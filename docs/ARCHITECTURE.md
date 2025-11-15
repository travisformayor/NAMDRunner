# NAMDRunner System Architecture

**How the system currently works and is structured** - This document explains the NAMDRunner system design, component architecture, business requirements, and technology choices.

> **For development practices and coding standards**, see [`docs/CONTRIBUTING.md`](CONTRIBUTING.md)
> **For SSH/SFTP connection patterns and security**, see [`docs/SSH.md`](SSH.md)
> **For UI/UX design patterns**, see [`docs/DESIGN.md`](DESIGN.md)
> **For database schemas and data management**, see [`docs/DB.md`](DB.md)
> **For adding commands or API changes**, see [`docs/API.md`](API.md)
> **For job automation architecture and workflow patterns**, see [`docs/AUTOMATIONS.md`](AUTOMATIONS.md)
> **For SLURM/NAMD command patterns**, see [`docs/reference/`](reference/) directory

## Table of Contents
- [Project Overview](#project-overview)
  - [Purpose](#purpose)
  - [Core Design Principles](#core-design-principles)
  - [Success Criteria](#success-criteria)
    - [User Experience Goals](#user-experience-goals)
    - [Technical Goals](#technical-goals)
- [Architecture Principles & Constraints](#architecture-principles--constraints)
  - [Security Requirements](#security-requirements)
    - [Security Anti-Patterns to Avoid](#security-anti-patterns-to-avoid)
  - [Architecture Constraints](#architecture-constraints)
  - [Fundamental System Requirements](#fundamental-system-requirements)
  - [Code Quality Standards](#code-quality-standards)
- [Technology Stack](#technology-stack)
  - [Shell/Runtime](#shellruntime)
  - [Frontend](#frontend)
  - [Backend (App Core)](#backend-app-core)
  - [SLURM Integration](#slurm-integration)
  - [Why This Stack](#why-this-stack)
  - [Application Lifecycle](#application-lifecycle)
  - [Architectural Patterns](#architectural-patterns)
  - [Database Architecture](#database-architecture)
- [System Design](#system-design)
  - [End-to-End Workflow](#end-to-end-workflow)
    - [Primary Workflow (Job Lifecycle)](#primary-workflow-job-lifecycle)
  - [Data Placement Strategy](#data-placement-strategy)
    - [Storage Architecture](#storage-architecture)
    - [Directory Structure](#directory-structure)
  - [Automation Architecture](#automation-architecture)
    - [Job Lifecycle Automation](#job-lifecycle-automation)
    - [Progress Tracking System](#progress-tracking-system)
    - [Automation Chain Implementation](#automation-chain-implementation)
  - [Architectural Overview](#architectural-overview)
  - [Module Structure](#module-structure)
    - [Frontend (Svelte/TypeScript)](#frontend-sveltetypescript)
    - [Backend (Rust)](#backend-rust)
    - [Frontend Architecture](#frontend-architecture)
- [Data Models & Interfaces](#data-models--interfaces)
  - [Type-Safe IPC Boundary](#type-safe-ipc-boundary)
  - [Data Models](#data-models)
    - [TypeScript Type System](#typescript-type-system)
    - [Rust Type System](#rust-type-system)
  - [IPC Command Interface](#ipc-command-interface)
    - [Available Commands](#available-commands)
- [Implementation Architecture](#implementation-architecture)
  - [Rust Development Patterns](#rust-development-patterns)
  - [Security Implementation](#security-implementation)
  - [SSH/SFTP Integration](#sshsftp-integration)
  - [Clean Architecture Patterns](#clean-architecture-patterns)
    - [Dependency Injection](#dependency-injection)
    - [Repository Pattern Implementation](#repository-pattern-implementation)
    - [Validation Pattern Implementation](#validation-pattern-implementation)
    - [SLURM Command Builder](#slurm-command-builder)
    - [Async Pattern Optimization](#async-pattern-optimization)
  - [System Architecture Patterns](#system-architecture-patterns)
    - [Data Flow](#data-flow)
    - [Error Handling](#error-handling)
- [Development Environment](#development-environment)
- [Build & Deployment](#build--deployment)
  - [Target Platforms & Build Matrix](#target-platforms--build-matrix)
    - [Primary Targets](#primary-targets)
    - [CI/CD Strategy](#cicd-strategy)
  - [Packaging & Delivery](#packaging--delivery)
    - [Distribution Artifacts](#distribution-artifacts)
    - [Build Commands](#build-commands)
- [Status & Planning](#status--planning)
  - [Open Items (Affect Implementation Details)](#open-items-affect-implementation-details)
    - [Pending Decisions](#pending-decisions)
  - [Next Steps](#next-steps)

## Project Overview

NAMDRunner is a **local desktop application** that runs on a researcher's PC to prepare, submit, and track **NAMD** simulations on a remote **SLURM** cluster. The application provides a secure, type-safe interface between a Rust backend and TypeScript frontend, with comprehensive SSH/SFTP integration for cluster operations.

### Purpose
A desktop application for scientists to manage NAMD molecular dynamics simulations without command-line complexity. The application is designed for **fellow researchers** (not paying customers) and prioritizes stability and low-maintenance operation over enterprise features.

### Core Design Principles
- **Local-only operation** - No hosted services or local HTTP servers (avoids CORS entirely)
- **SSH password-only authentication** - No SSH keys (cluster requirement)
- **No credential persistence** - Credentials exist only in memory during active sessions
- **Minimal cluster footprint** - No cluster-resident applications or databases
- **Type-safe boundaries** - Strict TypeScript ↔ Rust contracts

### Success Criteria

#### User Experience Goals
* Scientists can submit NAMD jobs without command line complexity
* Jobs don't mysteriously fail due to our tool
* Tool works reliably for months without maintenance
* Scientists can reopen the tool and see past submitted jobs
* Scientists can restart failed/incomplete jobs with different resources
* New developers can understand and modify code

#### Technical Goals
* Portable Windows executable that "just works"
* No credential persistence (security requirement)
* Offline mode with local cache
* Type-safe boundaries between frontend and backend
* Clean architecture without unnecessary abstractions

## Architecture Principles & Constraints

### Security Requirements
- **No Credential Persistence**: Passwords exist only in memory during active sessions
- **Input Sanitization**: Comprehensive validation prevents injection attacks and path traversal
- **Secure Memory**: SecStr-based password handling with automatic cleanup
- **Type-Safe IPC**: Strongly-typed communication prevents data corruption
- **Minimal Permissions**: Tauri configured with minimal required permissions

#### Security Anti-Patterns to Avoid
* Storing credentials in any form
* Logging secrets or passwords
* Printing raw command lines with secrets
* Over-broad Tauri permissions

### Architecture Constraints
* Adding hosted services or local HTTP servers (re-introduces CORS)
* Switching to SSH keys (cluster disallows)
* Installing servers or databases on the cluster
* Desktop E2E testing on macOS (no WKWebView WebDriver support)

### Fundamental System Requirements
- **Local-only desktop app** - No hosted backend or local HTTP servers
- **SSH auth = username/password only** - No SSH keys support
- **Never persist credentials** - Re-authentication required when sessions expire
- **No cluster-resident application** - Only files and SLURM CLI interactions
- **Local cache DB** - SQLite only
- **Strict typing and code quality** - Required for all code

## Technology Stack

### Shell/Runtime
* **Tauri v2** (desktop shell using system WebView). The frontend is built to **static assets** (HTML/CSS/JS) and **embedded** into the binary; no Node runtime ships, and there's no local server. This avoids CORS, keeps the footprint small, and minimizes maintenance.

### Frontend
* **Svelte + TypeScript**
  * Compile-time reactivity → small output and fewer "mystery re-renders."
  * Single-file components and explicit `$:` derivations → easier for juniors to read and reason about.
  * Solid unit/component testing story (Vitest + Svelte Testing Library).
  > **For Svelte patterns and implementation guidelines**, see [`docs/DESIGN.md#svelte-implementation-patterns`](DESIGN.md#svelte-implementation-patterns)

### Backend (App Core)
* **Rust** with **Tauri commands** as the IPC boundary from UI.
* **Job Automation System** - Complete lifecycle automation with progress tracking and real-time UI feedback.
  > **For automation architecture details**, see [`docs/AUTOMATIONS.md`](AUTOMATIONS.md)
* **SSH/SFTP** via Rust `ssh2` (libssh2). Password and keyboard-interactive auth match cluster policies.
* **SQLite** via `rusqlite` using simple document-store pattern for job caching. See [`docs/DB.md`](DB.md) for complete database patterns.
* **Logging Bridge** - Real-time Rust-to-Frontend logging system for SSH debugging and progress tracking.
* **Security Validation** - Comprehensive input sanitization, path safety, and secure password handling.
* **Template System** - Simple regex-based template rendering with variable substitution. Templates stored in database with embedded defaults.
  > **For NAMD configuration patterns**, see [`docs/reference/namd-commands-reference.md`](reference/namd-commands-reference.md)

### SLURM Integration
NAMDRunner integrates with SLURM workload managers for job submission and monitoring on HPC clusters.

> **For SLURM command patterns and implementation details**, see [`docs/reference/slurm-commands-reference.md`](reference/slurm-commands-reference.md)

### Why This Stack
* **Security/stability**: Rust core, minimal attack surface, no secrets on disk.
* **Maintainability**: typed boundaries (TS ↔ Rust), clear module seams, small binary.
* **UI velocity**: Svelte's component model is simple, predictable, and testable.

### Application Lifecycle

NAMDRunner follows a structured initialization sequence in `src-tauri/src/lib.rs`:

1. **Logging initialization** - First, before Tauri builder (`logging::init_logging()`)
2. **Tauri builder** - Plugin registration and app configuration
3. **Setup hook** - Critical initialization requiring `AppHandle`:
   - Set logging bridge to frontend (`logging::set_app_handle()`)
   - Resolve database path (`database::get_database_path(app.handle())`)
   - Initialize database with platform-specific path
4. **Command registration** - All 30+ IPC commands registered in `invoke_handler!`
5. **Template loading** - Default templates loaded lazily on first `list_templates` call (ensures frontend ready for logs)

**Why `.setup()` hook for database:**
- Tauri `AppHandle` only available after builder construction
- `app_data_dir()` API requires `AppHandle` for platform-specific path resolution
- Ensures proper directory creation before database initialization

### Architectural Patterns

**Manager Separation:**
NAMDRunner maintains strict separation of concerns across subsystems:
- **SSH/Connection Manager**: All remote operations and connection lifecycle
- **Job Manager**: Job lifecycle, metadata, and SLURM interactions
- **Database/Cache**: Local state persistence and offline capabilities

**Benefits:**
- Clean boundaries enable independent testing
- Each subsystem can be mocked separately
- Clear ownership prevents circular dependencies
- Easy to reason about where functionality belongs

**Offline-First Design:**
Local SQLite database serves as source of truth for UI, with remote operations updating asynchronously.

**Key benefits:**
- UI remains responsive during network operations
- Reduces unnecessary cluster queries
- Works reliably on unstable connections
- Simplifies state management

**Sync pattern:** User-controlled explicit operations (not automatic background sync). Users understand and control when data moves between local and remote.

**Testing Architecture:**
Frontend tests use Vitest mocking for isolated component testing:
- Tests mock dependencies using `vi.mock()` for type-safe test doubles
- Consistent testing with controlled test data
- No standalone mock client infrastructure required

### Database Architecture

NAMDRunner uses SQLite for local data persistence with platform-specific user data directory paths.

**Database Initialization:**
- **Platform-specific paths** using Tauri's `app_data_dir()` API
  - Linux: `~/.local/share/namdrunner/namdrunner.db`
  - Windows: `%APPDATA%\namdrunner\namdrunner.db`
  - Development: `./namdrunner_dev.db`
- **Setup hook initialization** - Database created in `.setup()` hook where `AppHandle` is available
- **Path resolution** - `database::get_database_path(app_handle)` centralizes all path logic

**Key Features:**
- **Local SQLite cache** for job metadata (document-store pattern)
- **Thread-safe global database** with singleton pattern
- **Atomic operations** integrated with automation chains
- **Zero-migration schema** using JSON serialization
- **Template storage** - Normalized schema with JSON variable definitions

> **For complete database schemas, implementation patterns, and data management details**, see [`docs/DB.md`](DB.md)

## System Design

### End-to-End Workflow

#### Primary Workflow (Job Lifecycle)
1. **Connect via SSH** - Password authentication, session lives in memory only
2. **Job Creation** - Build NAMD `.conf` from templates; user attaches input files; automation creates project directory
3. **Job Submission** - Automation creates scratch directory, copies files, submits to SLURM, captures JobID
4. **Status Monitoring** - Continuous synchronization with SLURM queue status
5. **Job Completion** - Automation detects completion, preserves critical results from scratch to project directory
6. **Results Access** - Browse preserved results, download outputs as needed
7. **Cleanup** - Safe removal of project and scratch directories when no longer needed

**Automation Integration:** Each workflow step leverages the automation system for progress tracking, error handling, and atomic operations.

### Data Placement Strategy

#### Storage Architecture
* **Local**: SQLite cache storing complete job metadata as JSON. See [`docs/DB.md`](DB.md) for schema details.
* **Remote** (cluster filesystem): JSON metadata files under project/job folders; job scratch directories contain runtime outputs
* **Single-writer rule**: The application writes JSON metadata; jobs write only inside their scratch/results directories

#### Directory Structure

Jobs use a standard two-subdirectory structure (`input_files/`, `outputs/`) centrally defined in `ssh/directory_structure.rs`. Job scripts (config.namd, job.sbatch) reside in the job root directory.

```
{job_id}/
├── job_info.json
├── config.namd                 # Generated NAMD config (in job root)
├── job.sbatch                  # Generated SLURM script (in job root)
├── input_files/                # User-uploaded files
├── outputs/                    # NAMD output files
├── namd_output.log
├── {job_name}_{slurm_id}.out
└── {job_name}_{slurm_id}.err
```

**Locations:**
- `/projects/$USER/namdrunner_jobs/{job_id}/` - Persistent storage
- `/scratch/alpine/$USER/namdrunner_jobs/{job_id}/` - Execution workspace (rsync mirrored)

### Automation Architecture

NAMDRunner implements a comprehensive job lifecycle automation system that orchestrates the complete workflow from job creation through completion and cleanup.

#### Job Lifecycle Automation

**Five Core Automation Chains:**
1. **Job Creation** - Project setup, file validation, directory creation
2. **Job Submission** - Scratch workspace setup, SLURM submission
3. **Status Synchronization** - Real-time SLURM queue monitoring
4. **Job Completion** - Results preservation from scratch to project storage
5. **Job Cleanup** - Safe directory removal with security validation

#### Key Automation Features

**Architecture Principles:**
- **Progress Tracking** - Real-time UI feedback via Tauri events
- **Atomic Operations** - Complete success or clean failure for each step
- **Workflow Separation** - Clear boundaries between automation stages
- **Security Validation** - Input sanitization and path safety throughout
- **Metadata-at-Boundaries** - Server metadata updated only at lifecycle boundaries (creation, submission, completion), not during execution

#### Metadata-at-Boundaries Principle

NAMDRunner updates server metadata (`job_info.json`) only at job lifecycle boundaries, not during execution:

- **Job Creation**: Metadata written to project directory (job parameters, initial status)
- **Job Submission**: Metadata updated with SLURM job ID and submission timestamp
- **During Execution**: Local database updated only (metadata remains at submission state)
- **Job Completion**: Rsync scratch→project FIRST, then metadata updated with final status

**Rationale:**
- Prevents rsync conflicts during job execution
- Keeps metadata updates predictable and atomic
- Metadata snapshot represents job configuration at submission time
- Final metadata update after rsync ensures consistency

**Implementation:** Server metadata updates restricted to `job_creation.rs`, `job_submission.rs`, and `job_completion.rs` automation modules.

**Implementation Pattern:**
- Simple async functions with progress callbacks
- Consistent `Result<T>` error handling
- Direct integration with Tauri command system
- Backend-owned workflows (single backend call per user action)

> **For complete automation implementation details, patterns, and integration examples**, see [`docs/AUTOMATIONS.md`](AUTOMATIONS.md)

### Architectural Overview

**Clean Separation of Concerns**:
- **Frontend**: Svelte components with TypeScript - pure UI layer with no business logic or validation
  - Reactive stores cache backend data for instant UI updates
  - All validation happens in backend (single source of truth)
  - Dynamic form generation from template definitions
  > **For UI/UX design patterns**, see [`docs/DESIGN.md`](DESIGN.md)
- **Backend**: Layered architecture with clear separation of concerns
  - **Commands layer** (`commands/`): Thin IPC adapters that validate input, call business logic, and wrap results in ApiResult
  - **Automations layer** (`automations/`): Business logic for job lifecycle workflows with progress tracking
  - **Core services**: Cluster capabilities (`cluster.rs`), validation (`validation/`), templates (`templates/`), SSH/SFTP (`ssh/`)
  - Cluster capabilities (partitions, QOS, billing) in `cluster.rs`
  - Resource validation in `validation/job_validation.rs`
  - Input sanitization and security validation in `validation.rs`
  - Template rendering and validation in `templates/` module
  - SSH/SFTP operations with full console logging
  - Metadata management in `ssh/metadata.rs`
- **IPC Layer**: Strongly-typed communication layer between frontend and backend
- **Development**: Self-contained mock client enables offline development

### Module Structure

#### Frontend (Svelte/TypeScript)

```
src/
├── lib/
│   ├── types/                   # TypeScript type definitions
│   │   ├── api.ts               # Core API types matching Rust types
│   │   ├── template.ts          # Template types matching Rust types
│   │   ├── connection.ts        # Connection state and session types
│   │   └── errors.ts            # Error handling types
│   ├── stores/                  # Reactive state management (caches backend data)
│   │   ├── session.ts           # Session state with reactive updates
│   │   ├── jobs.ts              # Job state management with real-time updates
│   │   ├── templateStore.ts     # Template state management
│   │   ├── clusterConfig.ts     # Cluster capabilities cache (from backend)
│   │   ├── settings.ts          # Database info and management operations
│   │   ├── ui.ts                # UI state, navigation, and view preferences
│   │   └── session.test.ts      # Store unit tests
│   ├── components/              # Svelte UI components
│   │   ├── layout/              # Layout and infrastructure components
│   │   │   ├── AppHeader.svelte     # Application header with connection status
│   │   │   ├── AppSidebar.svelte    # Navigation sidebar
│   │   │   ├── ConnectionDropdown.svelte # Connection management
│   │   │   └── LogsPanel.svelte          # Application and SSH command logging and debugging interface
│   │   ├── create-job/          # Job creation workflow components
│   │   │   ├── CreateJobTabs.svelte     # 3-tab job creation interface
│   │   │   ├── ResourcesTab.svelte      # Resource allocation configuration
│   │   │   ├── ConfigureTab.svelte      # Template selection and configuration
│   │   │   ├── DynamicJobForm.svelte    # Dynamic form from template variables
│   │   │   └── ReviewTab.svelte         # Final review before submission
│   │   ├── templates/           # Template management components
│   │   │   ├── TemplateEditor.svelte    # Template creation/editing
│   │   │   └── VariableEditor.svelte    # Variable definition editor
│   │   ├── job-detail/          # Job details and management components (4 tabs)
│   │   │   ├── JobTabs.svelte           # Tab navigation and routing
│   │   │   ├── tabs/OverviewTab.svelte  # Job overview and metadata
│   │   │   ├── tabs/InputFilesTab.svelte  # Input files listing
│   │   │   ├── tabs/OutputFilesTab.svelte # Output files listing
│   │   │   └── tabs/SlurmLogsTab.svelte   # SLURM logs display
│   │   ├── jobs/                # Job listing and management components
│   │   ├── pages/               # Page-level components
│   │   │   ├── TemplatesPage.svelte      # Template management page
│   │   │   ├── TemplateEditorPage.svelte # Template editor page
│   │   │   └── SettingsPage.svelte       # Settings and database management
│   │   ├── ui/                  # Reusable UI components
│   │   │   ├── Dialog.svelte             # Base modal primitive
│   │   │   ├── AlertDialog.svelte        # Success/Error/Warning notifications
│   │   │   ├── ConfirmDialog.svelte      # Confirmation dialog
│   │   │   └── PreviewModal.svelte       # Template preview modal
│   │   └── AppShell.svelte      # Main application shell
│   ├── types/                   # Additional TypeScript types
│   │   └── cluster.ts           # Cluster capability types (matches backend)
│   ├── styles/                  # Global styles and themes
│   ├── utils/                   # Utility functions
│   │   ├── file-helpers.ts      # File display formatting utilities
│   │   └── template-utils.ts    # Template rendering and validation utilities
│   └── test/                    # Testing infrastructure
│       ├── fixtures/            # Test data and scenarios
│       │   ├── testDataManager.ts # Test data management
│       │   ├── jobFixtures.ts     # Job test data
│       │   └── slurmFixtures.ts   # SLURM test data
│       └── setup.ts             # Test setup configuration
│           > **For testing tools and debugging infrastructure**, see [`docs/reference/agent-development-tools.md`](reference/agent-development-tools.md)
├── routes/
│   ├── +layout.ts              # Layout configuration
│   └── +page.svelte            # Main application interface
└── app.html
```

#### Backend (Rust)

```
src-tauri/
├── src/
│   ├── main.rs                  # Application entry point
│   ├── lib.rs                   # Tauri app configuration and command registration
│   ├── commands/                # Tauri IPC command handlers (thin adapters)
│   │   ├── mod.rs              # Command module exports
│   │   ├── connection.rs       # SSH connection lifecycle commands
│   │   ├── cluster.rs          # Cluster configuration and validation commands
│   │   ├── jobs.rs             # Job lifecycle commands (create/submit/sync/delete/complete)
│   │   ├── files.rs            # File management commands
│   │   ├── templates.rs        # Template management commands (list/get/create/update/delete/validate)
│   │   ├── database.rs         # Database management commands (info/backup/restore/reset)
│   │   └── helpers.rs          # Reusable command helpers (4 functions: require_connection, load_job_or_fail, get_cluster_username, load_template_or_fail)
│   ├── automations/            # Job lifecycle automation system
│   │   ├── mod.rs              # Automation module exports
│   │   ├── job_creation.rs     # Job creation automation with progress tracking
│   │   ├── job_submission.rs   # Job submission automation
│   │   ├── job_completion.rs   # Job completion and results preservation
│   │   ├── job_deletion.rs     # Job deletion automation
│   │   ├── job_sync.rs         # Job status synchronization
│   │   ├── common.rs           # Shared automation helpers (7 functions: save_job_to_database, require_connection_with_username, require_project_dir, etc.)
│   │   ├── errors.rs           # Automation error types and handling
│   │   └── progress.rs         # Progress tracking types and utilities
│   ├── templates/              # Template system
│   │   ├── mod.rs              # Template module exports
│   │   ├── types.rs            # Template and variable definition types
│   │   ├── renderer.rs         # Template rendering with variable substitution
│   │   └── validation.rs       # Template value validation
│   ├── ssh/                    # SSH/SFTP service implementation
│   │   ├── mod.rs              # SSH module exports
│   │   ├── connection.rs       # Low-level SSH connection handling
│   │   ├── manager.rs          # Connection lifecycle and directory management
│   │   ├── commands.rs         # SSH command execution and parsing
│   │   ├── sftp.rs             # File transfer operations
│   │   ├── metadata.rs         # Job metadata upload utilities
│   │   ├── directory_structure.rs # Standard job directory structure definitions
│   │   ├── errors.rs           # SSH error mapping and classification
│   │   └── test_utils.rs       # Development utilities
│   ├── slurm/                  # SLURM integration
│   │   ├── mod.rs              # SLURM module exports
│   │   ├── commands.rs         # SLURM command builder and patterns
│   │   ├── script_generator.rs # SLURM script generation (no NAMD config - uses templates)
│   │   └── status.rs           # Job status synchronization
│   ├── database/               # Data persistence layer
│   │   └── mod.rs              # Database with jobs and templates tables
│   ├── types/                  # Rust type definitions
│   │   ├── mod.rs              # Type module exports
│   │   ├── core.rs             # Core domain types (JobInfo, SessionInfo, ApiResult, JobStatus, ConnectionState, etc.)
│   │   ├── commands.rs         # Command parameter and result types
│   │   └── response_data.rs    # Response data structures for IPC commands
│   ├── validation/             # Validation system
│   │   ├── mod.rs              # Input sanitization and path safety
│   │   └── job_validation.rs   # Resource and business logic validation
│   ├── logging.rs              # Rust-to-Frontend logging bridge system
│   ├── retry.rs                # Exponential backoff retry implementation
│   ├── security.rs             # Secure password handling with SecStr
│   └── cluster.rs              # Cluster capabilities and configuration
├── templates/                   # Embedded template JSON files
│   ├── vacuum_optimization_v1.json
│   └── explicit_solvent_npt_v1.json
├── Cargo.toml                  # Dependencies (ssh2, secstr, rusqlite, anyhow, chrono)
└── tauri.conf.json            # Tauri configuration
```

### Frontend Architecture

**Backend-First Design:**
NAMDRunner implements a backend-first architecture where all business logic, validation, and cluster configuration lives in Rust, while the frontend is a pure UI layer using reactive stores for state caching.

> **For frontend state management patterns and component implementation details**, see [`docs/CONTRIBUTING.md#frontend-development-standards`](CONTRIBUTING.md#frontend-development-standards)

**Key Architecture Principles:**
- **Stores as pure caches** - No business logic, workflow orchestration, or validation. Stores receive complete state from backend and reactively update UI.
- **Store factory pattern** - Stores use factory functions (e.g., `createJobsStore()`) for encapsulation and testability
- **Backend is source of truth** - All validation, calculations, and workflow orchestration happen in Rust. Frontend makes single backend calls.
- **Type-safe IPC boundary** - Consistent snake_case contracts between TypeScript and Rust
- **Single backend calls** - No multi-step orchestration (e.g., sync returns complete job list including discovered jobs)
- **Presentational components** - UI components handle display logic only

**Design System:**
NAMDRunner uses a centralized design system with comprehensive theming support.

- **Centralized CSS variables** - All colors defined in `app.css` with light/dark theme support
- **Unified component patterns** - Single primitive components extended through composition
  - `Dialog.svelte` - Base modal primitive (escape key, click-outside, z-index management)
  - `AlertDialog.svelte` - Success/Error/Warning/Info notifications
  - `ConfirmDialog.svelte` - Confirmation dialogs with destructive action support
  - Button system: `.namd-button` classes (primary, secondary, destructive variants)
- **Theme consistency** - All interactive elements use `--namd-*` CSS variables (no hardcoded colors)

**Frontend Responsibilities:**
- Reactive state caching via Svelte stores with factory pattern ([`jobs.ts`](../src/lib/stores/jobs.ts), [`session.ts`](../src/lib/stores/session.ts), [`templateStore.ts`](../src/lib/stores/templateStore.ts), [`settings.ts`](../src/lib/stores/settings.ts))
- UI presentation and user interaction
- Progress event handling from backend
- Display formatting (status badges, file icons, etc.)
- **Single backend call pattern** - One IPC call per user action (no orchestration)

**Frontend Does NOT:**
- Validate user inputs (backend validates)
- Implement business logic (backend owns all rules)
- Perform calculations (backend is source of truth)
- Execute file operations directly (backend handles SSH/SFTP)
- **Orchestrate multi-step workflows** (backend returns complete results)
- **Make business decisions** (e.g., when to trigger discovery)

## Data Models & Interfaces

### Type-Safe IPC Boundary

All IPC communication uses **snake_case convention** for consistency between TypeScript and Rust, eliminating the need for case conversion layers and improving searchability across the codebase.

```rust
// Rust (native snake_case)
#[derive(Serialize, Deserialize)]
pub struct JobInfo {
    pub job_id: String,
    pub job_name: String,
    pub slurm_job_id: Option<String>,
    pub created_at: String,
}
```

```typescript
// TypeScript (matches Rust exactly)
interface JobInfo {
    job_id: string;
    job_name: string;
    slurm_job_id: string | null;
    created_at: string;
}
```

**Benefits:**
- Type errors caught at compile time
- Consistent naming across frontend and backend
- No manual type conversions required
- Improved code searchability

### Data Models

#### TypeScript Type System
Core types defined in `src/lib/types/api.ts`:

```typescript
// Core state types
type ConnectionState = 'Disconnected' | 'Connecting' | 'Connected' | 'Expired';
type JobStatus = 'CREATED' | 'PENDING' | 'RUNNING' | 'COMPLETED' | 'FAILED' | 'CANCELLED';

// Job management types (IPC communication uses snake_case)
interface CreateJobParams {
  job_name: string;
  template_id: string;                     // Template to use for job
  template_values: Record<string, any>;    // Variable values for template
  slurm_config: SlurmConfig;
}

interface JobInfo {
  job_id: JobId;
  job_name: string;
  status: JobStatus;
  slurm_job_id?: SlurmJobId;
  template_id: string;                     // Template used for this job
  template_values: Record<string, any>;    // Variable values at submission
  input_files?: string[];                  // List of uploaded input filenames
  created_at: Timestamp;
  updated_at?: Timestamp;
  // ... additional fields
}

// Template system types
interface Template {
  id: string;                              // Unique identifier
  name: string;                            // Display name
  description: string;                     // User-facing description
  namd_config_template: string;            // NAMD config with {{variables}}
  variables: Record<string, VariableDefinition>;
  is_builtin: boolean;                     // Embedded default templates
  created_at: Timestamp;
  updated_at: Timestamp;
}

interface VariableDefinition {
  key: string;                             // Variable name in template
  label: string;                           // UI display label
  var_type: VariableType;                  // Number | Text | Boolean | FileUpload
  help_text?: string;                      // Optional help text for users
}
```

#### Template System Architecture

NAMDRunner uses a **template-based configuration system** where NAMD simulation parameters are defined through templates stored in the database:

**Core Concepts:**
- **Templates as data** - NAMD configurations stored in database, not hardcoded
- **Variable substitution** - `{{variable}}` placeholders replaced with user values
- **All variables required** - Every variable defined in template must have a value (no optional variables)
- **Type-safe rendering** - Variable types control value formatting (Boolean→"yes"/"no", FileUpload→"input_files/filename")
- **Embedded defaults** - Two built-in templates loaded on first run via `include_str!` macro
- **Jobs reference templates** - JobInfo stores `template_id` + `template_values` instead of full NAMD config

**Template Rendering Flow:**
1. User selects template and fills in variable values
2. Frontend validates all variables have values
3. Backend validates values match types and constraints (missing any variable causes error)
4. Backend renders template by replacing `{{key}}` with formatted values
5. Rendered NAMD config written to `config.namd` in job root directory on cluster

**Default Templates:**
- `vacuum_optimization_v1` - Large periodic box, PME disabled, ENM restraints
- `explicit_solvent_npt_v1` - NPT ensemble, PME enabled, pressure control

**Implementation:**
- Backend: `src-tauri/src/templates/` module (types, renderer, validation)
- Database: Templates table with JSON variable definitions
- Commands: 7 IPC commands for template management (list/get/create/update/delete/validate/preview)

#### Rust Type System

NAMDRunner maintains strict type safety between TypeScript frontend and Rust backend:

**Core Types:**
- **`ApiResult<T>`** - Consistent return type for all Tauri commands
- **`JobInfo`** - Complete job lifecycle with template_id + template_values
- **`SessionInfo`** - SSH connection state and authentication data
- **`Template`** - Template definition with NAMD config and variables
- **`SlurmConfig`** - SLURM resource allocation settings

**Type Safety Features:**
- **Serde attributes** ensure consistent serialization between TS and Rust
- **Strongly typed enums** for job status and connection states
- **Optional fields** for lifecycle progression (submitted_at, completed_at)

> **For complete type definitions, serde attributes, and API contracts**, see [`docs/API.md`](API.md)

### IPC Command Interface

The application provides a complete set of commands for job management and cluster operations.

> **For detailed IPC interfaces and command specifications**, see [`docs/API.md`](API.md)

#### Available Commands

NAMDRunner provides 30+ Tauri commands organized into five main categories:
- **Connection Management** (3 commands): SSH connectivity and session management
- **Job Lifecycle** (9 commands): Create, submit, monitor, sync, complete, cleanup operations
- **File Management** (6 commands): Upload, download, listing, and file operations
- **Template Management** (7 commands): List, get, create, update, delete, validate, preview templates
- **Database Management** (4 commands): Info, backup, restore, reset operations

**Key Features:**
- **Consistent return types** for uniform error handling
- **Real-time progress tracking** via Tauri event system
- **Template-based job configuration** with dynamic validation
- **Comprehensive automation** with job completion and results preservation
- **Database management** with online backup using SQLite Backup API

> **For complete command interfaces, parameter types, and response schemas**, see [`docs/API.md`](API.md)

## Implementation Approach

### Development Philosophy

NAMDRunner follows **direct code patterns** and **progressive enhancement** principles:
- Start simple, add abstraction only when proven necessary
- Direct function calls over wrapper layers
- Type-safe boundaries between frontend and backend
- Security-first design with comprehensive input validation

### Key Implementation Areas

**Backend (Rust):**
- **Commands Layer** - Thin IPC adapters with minimal logic
  - Commands pattern: Validate input → Call automation/business logic → Wrap result in ApiResult
  - No business logic in commands (moved to `automations/` modules)
  - Helper functions in `commands/helpers.rs` for common patterns (connection checks, database loading)
  - Example refactoring: `commands/jobs.rs` reduced from 590 to 210 lines by extracting workflows to `automations/`
- **Template System** - Template-based NAMD configuration with database storage
  - Template types: `src-tauri/src/templates/types.rs` (Template, VariableDefinition, VariableType)
  - Template rendering: `src-tauri/src/templates/renderer.rs::render_template()` (regex-based variable substitution)
  - Template validation: `src-tauri/src/templates/validation.rs::validate_values()` (type checking, required fields)
  - Template commands: `src-tauri/src/commands/templates.rs` (7 IPC commands)
  - Default templates: Embedded JSON files loaded via `include_str!` macro on first `list_templates` call
  - Variable types: Number (min/max/default), Text (default), Boolean (default, rendered as "yes"/"no"), FileUpload (extensions, prepends "input_files/")
- **Database Management** - User-accessible database operations
  - Database commands: `src-tauri/src/commands/database.rs` (4 IPC commands: info, backup, restore, reset)
  - Online backup: SQLite Backup API for safe backups while app running
  - Connection management: `database::reinitialize_database()` safely closes and reopens connections
  - Path resolution: `database::get_database_path()` centralizes platform-specific path logic
- **SSH/SFTP Integration** - Complete secure cluster connectivity and file operations
  > **For SSH/SFTP security patterns and connection management**, see [`docs/SSH.md`](SSH.md)
  - Connection management: `src-tauri/src/ssh/manager.rs::ConnectionManager` (singleton with retry logic)
  - Authentication: `src-tauri/src/ssh/connection.rs::SSHConnection::connect()` (password-only, SecurePassword)
  - File operations: `src-tauri/src/ssh/sftp.rs::SFTPOperations` (upload/download with progress tracking)
  - Metadata management: `src-tauri/src/ssh/metadata.rs::upload_job_metadata()` (centralized job metadata uploads)
  - Directory operations: `src-tauri/src/ssh/manager.rs::create_directory()` (recursive with validation)
  - Command execution: `src-tauri/src/ssh/commands.rs::CommandExecutor` (SLURM integration)
  - Error handling: `src-tauri/src/ssh/errors.rs` (comprehensive categorization and retry logic)
  - **Note**: All SSH/SFTP/directory operations are handled by Rust backend infrastructure
- **Job Automation System** - Complete lifecycle automation with progress tracking
  > **For complete automation implementation details, patterns, and integration examples**, see [`docs/AUTOMATIONS.md`](AUTOMATIONS.md)
  - Job creation: `src-tauri/src/automations/job_creation.rs::execute_job_creation_with_progress()`
  - Job submission: `src-tauri/src/automations/job_submission.rs::execute_job_submission_with_progress()`
  - Job completion: `src-tauri/src/automations/job_completion.rs::execute_job_completion_with_progress()`
- **Database Layer** - SQLite document-store for jobs and templates
  > **For complete database patterns and data management details**, see [`docs/DB.md`](DB.md)
  - Main database: `src-tauri/src/database/mod.rs::JobDatabase`
  - Jobs table: JSON document storage for JobInfo
  - Templates table: Normalized schema with JSON variable definitions
- **Validation System** - Comprehensive input sanitization and business logic validation
  > **For security requirements and implementation patterns**, see [`docs/CONTRIBUTING.md#security-requirements`](CONTRIBUTING.md#security-requirements)
  - Input validation: `src-tauri/src/validation/mod.rs::input` module (sanitization, path traversal prevention)
  - Path safety: `src-tauri/src/validation/mod.rs::paths` module
  - Shell escaping: `src-tauri/src/validation/mod.rs::shell` module
  - Resource validation: `src-tauri/src/validation/job_validation.rs::validate_resource_allocation()` (cluster limits, QoS rules)
  - Template validation: `src-tauri/src/templates/validation.rs` (variable type checking)

**Frontend (TypeScript/Svelte):**
- **IPC Communication** - Direct Tauri invoke with strongly-typed commands
  > **For detailed IPC interfaces and command specifications**, see [`docs/API.md`](API.md)
  - Direct usage: `invoke<T>(command, params)` from `@tauri-apps/api/core`
  - No abstraction layer: Stores call invoke directly for type safety and simplicity
  - Test mocking: Vitest `vi.mock('@tauri-apps/api/core')` for isolated component testing
- **Reactive State Management** - UI state caching backend data
  - Job state: `src/lib/stores/jobs.ts` (caches job list, handles real-time updates)
  - Template state: `src/lib/stores/templateStore.ts` (template management)
  - Session state: `src/lib/stores/session.ts` (connection status, session info)
  - Cluster config: `src/lib/stores/clusterConfig.ts` (caches backend cluster capabilities)
  - Settings state: `src/lib/stores/settings.ts` (database info, backup/restore/reset operations)
- **Component Architecture** - Focused, composable UI components
  > **For UI/UX design patterns and component specifications**, see [`docs/DESIGN.md`](DESIGN.md)
  - App shell: `src/lib/components/AppShell.svelte`
  - Job creation: `src/lib/components/create-job/` (3-tab workflow with dynamic forms)
  - Template management: `src/lib/components/templates/` (template editor, variable editor)
  - Settings page: `src/lib/components/pages/SettingsPage.svelte` (database management UI)
  - UI primitives: `src/lib/components/ui/` (Dialog, AlertDialog, ConfirmDialog, PreviewModal)
  - Layout components: `src/lib/components/layout/` directory
  - Page components: `src/lib/components/pages/` directory

> **For complete development patterns, coding standards, and implementation guidelines**, see [`docs/CONTRIBUTING.md`](CONTRIBUTING.md)
> **For SSH/SFTP security patterns and connection management**, see [`docs/SSH.md`](SSH.md)

## Development Environment

NAMDRunner supports comprehensive development tooling:
- **Hot reload development** with `npm run tauri dev`
- **Cross-platform builds** targeting Windows, Linux, and macOS
- **Unit testing** with Vitest and Svelte Testing Library

> **For complete development setup, tooling, and workflow details**, see [`docs/CONTRIBUTING.md`](CONTRIBUTING.md)

## Build & Deployment

### Target Platforms

**Primary Distribution:**
- **Windows** - Portable `.exe` via GitHub Actions (primary user target)
- **Linux** - Primary development environment
- **macOS** - Local builds for validation only

**Build Strategy:**
- **Cross-platform Tauri builds** with GitHub Actions CI/CD
- **Portable executables** with embedded frontend assets
- **Static linking** for dependency-free distribution

> **For complete build setup, CI/CD workflows, and deployment procedures**, see [`docs/CONTRIBUTING.md`](CONTRIBUTING.md)


## Status & Planning

### Open Items (Affect Implementation Details)

#### Pending Decisions
* **SLURM Version Compatibility**: Confirm JSON output support; plan formatted fallbacks
* **Scratch Purge Cadence**: Determine when to copy back results from scratch directories
* **Module Versions**: Make gcc/NAMD module versions configurable (not hardcoded)
* **Resource Limits**: Document cluster-specific partition limits and QoS specifications
  > **For cluster-specific configurations and resource limits**, see [`docs/reference/alpine-cluster-reference.md`](reference/alpine-cluster-reference.md)

### Next Steps

See [`tasks/roadmap.md`](../tasks/roadmap.md) for planned features and development timeline.

## Technical Implementation Considerations

### Frontend State Management Architecture
- **Connection State**: Global Svelte store managing SSH session lifecycle
- **Job List**: Cached in store, synced with backend via periodic refresh
- **Form State**: Local to components using Svelte's reactive binding
- **Logs Panel Buffer**: Global store with configurable size limit for debugging

### Performance Optimization Patterns
- **Lazy loading** for job details and log content (loaded on-demand)
- **Debounced** form validation to reduce computation overhead
- **Throttled** SSH console updates to prevent UI blocking

### Accessibility Implementation
- **Keyboard navigation** support for all interactive elements
- **ARIA labels** for screen readers on status indicators and controls
- **Focus management** for modals and popup interactions
- **Color-blind friendly** status indicators (combine icons with color coding)

### UX Implementation Requirements
- **Explicit connection controls** with visible session state indicators
- **Clear job status** with last-polled timestamp display
- **Non-blocking status refresh** with dismissible error banners and retry options