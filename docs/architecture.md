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
- [Data Models & Interfaces](#data-models--interfaces)
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
* **SQLite** via `rusqlite` with comprehensive schema and status history tracking. See [`docs/DB.md`](DB.md) for complete database patterns.
* **Logging Bridge** - Real-time Rust-to-Frontend logging system for SSH debugging and progress tracking.
* **Security Validation** - Comprehensive input sanitization, path safety, and secure password handling.
* **Templating** (NAMD `.conf` + Slurm scripts) via `tera` or `handlebars` - **Phase 5+**: Templates stored as configurable data in database.
  > **For NAMD configuration patterns**, see [`docs/reference/namd-commands-reference.md`](reference/namd-commands-reference.md)

### SLURM Integration
NAMDRunner integrates with SLURM workload managers for job submission and monitoring on HPC clusters.

> **For SLURM command patterns and implementation details**, see [`docs/reference/slurm-commands-reference.md`](reference/slurm-commands-reference.md)

### Why This Stack
* **Security/stability**: Rust core, minimal attack surface, no secrets on disk.
* **Maintainability**: typed boundaries (TS ↔ Rust), clear module seams, small binary.
* **UI velocity**: Svelte's component model is simple, predictable, and testable.

### Database Architecture

NAMDRunner uses SQLite for local data persistence with job lifecycle management and status tracking.

**Key Features:**
- **Local SQLite cache** for job metadata and status history
- **Thread-safe global database** with singleton pattern
- **Atomic operations** integrated with automation chains
- **Schema versioning** for development iteration

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
* **Local**: SQLite cache with job metadata, timestamps, status history. See [`docs/DB.md`](DB.md) for complete schema definitions.
* **Remote** (cluster filesystem): JSON metadata files under project/job folders; job scratch directories contain runtime outputs
* **Single-writer rule**: The application writes JSON metadata; jobs write only inside their scratch/results directories

#### Directory Structure
```
/projects/$USER/namdrunner_jobs/     # Persistent storage
└── {job_id}/
    ├── job_info.json               # Job metadata
    ├── input_files/                # User input files
    ├── config.namd                 # Generated NAMD config
    └── job.sbatch                  # Generated SLURM script

/scratch/alpine/$USER/namdrunner_jobs/  # Execution workspace
└── {job_id}/                       # Working directory during execution
    ├── [copied input files]
    ├── output.dcd                  # Trajectory files
    └── restart.*                   # Restart files
```

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

**Implementation Pattern:**
- Simple async functions with progress callbacks
- Consistent `Result<T>` error handling
- Direct integration with Tauri command system

> **For complete automation implementation details, patterns, and integration examples**, see [`docs/AUTOMATIONS.md`](AUTOMATIONS.md)

### Architectural Overview

**Clean Separation of Concerns**:
- **Frontend**: Svelte components with TypeScript, reactive stores, and comprehensive IPC client
  > **For UI/UX design patterns**, see [`docs/DESIGN.md`](DESIGN.md)
- **Backend**: Rust command handlers with SSH/SFTP services and security validation
- **IPC Layer**: Strongly-typed communication layer between frontend and backend
- **Demo Mode Integration**: User-selectable mode switching with persistent preference
- **Development**: Mock implementations enable offline development

**Demo Mode Architecture**: The application supports seamless switching between demo mode (rich mock data for demonstrations) and real mode (full cluster integration) through a user toggle in the connection dropdown. Mode preference persists across sessions via localStorage and synchronizes with the backend for consistent behavior.

### Module Structure

#### Frontend (Svelte/TypeScript)

```
src/
├── lib/
│   ├── ports/                   # IPC communication layer
│   │   ├── coreClient.ts        # IPC interface definition
│   │   ├── coreClient-tauri.ts  # Production Tauri implementation
│   │   ├── coreClient-mock.ts   # Mock for offline development
│   │   └── clientFactory.ts     # Smart client selection with demo mode support
│   ├── types/                   # TypeScript type definitions
│   │   ├── api.ts               # Core API types matching Rust types
│   │   ├── connection.ts        # Connection state and session types
│   │   ├── errors.ts            # Error handling types
│   │   └── errorUtils.ts        # Error utility functions
│   ├── services/                # Frontend business logic services
│   │   ├── connectionState.ts   # Observable connection state management
│   │   ├── sessionManager.ts    # Session lifecycle and validation
│   │   ├── pathResolver.ts      # Centralized path generation
│   │   ├── serviceContainer.ts  # Service container
│   │   ├── sftp.ts              # SFTP service wrapper
│   │   ├── ssh.ts               # SSH service wrapper
│   │   └── index.ts             # Service exports
│   ├── stores/                  # Reactive state management
│   │   ├── session.ts           # Session state with reactive updates
│   │   ├── jobs.ts              # Job state management with real-time updates
│   │   ├── ui.ts                # UI state and preferences
│   │   └── session.test.ts      # Store unit tests
│   ├── components/              # Svelte UI components
│   │   ├── layout/              # Layout and infrastructure components
│   │   │   ├── AppHeader.svelte     # Application header with connection status
│   │   │   ├── AppSidebar.svelte    # Navigation sidebar
│   │   │   ├── ConnectionDropdown.svelte # Connection management with demo mode toggle
│   │   │   └── SSHConsolePanel.svelte    # SSH command logging and debugging interface
│   │   ├── create-job/          # Job creation workflow components
│   │   │   ├── CreateJobTabs.svelte     # Tabbed job creation interface
│   │   │   ├── ResourcesTab.svelte      # Resource allocation configuration
│   │   │   └── CompactQosSelector.svelte # QoS selection component
│   │   ├── job-detail/          # Job details and management components
│   │   ├── jobs/                # Job listing and management components
│   │   ├── pages/               # Page-level components
│   │   ├── ui/                  # Reusable UI components
│   │   │   ├── Progress.svelte      # Progress indicators
│   │   │   ├── FormField.svelte     # Form input components
│   │   │   └── ResourceUsage.svelte # Resource usage displays
│   │   └── AppShell.svelte      # Main application shell
│   ├── data/                    # Static data and configuration
│   ├── styles/                  # Global styles and themes
│   ├── utils/                   # Utility functions
│   │   └── file-helpers.ts      # File handling utilities
│   └── test/                    # Testing infrastructure
│       ├── fixtures/            # Test data and scenarios
│       │   ├── testDataManager.ts # Test data management
│       │   ├── sessionFixtures.ts # Session test data
│       │   ├── jobFixtures.ts     # Job test data
│       │   ├── fileFixtures.ts    # File test data
│       │   └── slurmFixtures.ts   # SLURM test data
│       ├── services/            # Unit tests for business logic
│       │   ├── connectionState.test.ts
│       │   └── sessionManager.test.ts
│       ├── utils/               # Test utilities
│       │   └── connectionMocks.ts # Mock connection utilities
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
│   ├── commands/                # Tauri IPC command handlers
│   │   ├── mod.rs              # Command module exports
│   │   ├── connection.rs       # Connection management and SSH/SFTP commands
│   │   ├── jobs.rs             # Job lifecycle commands (create/submit/sync/delete/complete)
│   │   └── files.rs            # File management commands
│   ├── automations/            # Job lifecycle automation system
│   │   ├── mod.rs              # Automation module exports
│   │   ├── job_creation.rs     # Job creation automation with progress tracking
│   │   ├── job_submission.rs   # Job submission automation
│   │   ├── job_completion.rs   # Job completion and results preservation
│   │   └── progress.rs         # Progress tracking utilities
│   ├── ssh/                    # SSH/SFTP service implementation
│   │   ├── mod.rs              # SSH module exports
│   │   ├── connection.rs       # Low-level SSH connection handling
│   │   ├── manager.rs          # Connection lifecycle and directory management
│   │   ├── commands.rs         # SSH command execution and parsing
│   │   ├── sftp.rs             # File transfer operations
│   │   ├── errors.rs           # SSH error mapping and classification
│   │   └── test_utils.rs       # Development utilities
│   ├── slurm/                  # SLURM integration
│   │   ├── mod.rs              # SLURM module exports
│   │   ├── commands.rs         # SLURM command builder and patterns
│   │   ├── script_generator.rs # SLURM script generation
│   │   └── status.rs           # Job status synchronization
│   ├── database/               # Data persistence layer
│   │   ├── mod.rs              # Database module with SQLite schema and operations
│   │   └── helpers.rs          # Database helper functions
│   ├── types/                  # Rust type definitions
│   │   ├── mod.rs              # Type module exports
│   │   ├── core.rs             # Core domain types (JobInfo, SessionInfo, ApiResult)
│   │   └── commands.rs         # Command parameter and result types
│   ├── logging.rs              # Rust-to-Frontend logging bridge system
│   ├── retry.rs                # Exponential backoff retry implementation
│   ├── validation.rs           # Input sanitization and path safety
│   ├── security.rs             # Secure password handling with SecStr
│   ├── mode_switching.rs       # Demo/real mode switching patterns
│   ├── mock_state.rs           # Mock state management for development
│   ├── security_tests.rs       # Security validation tests (29 tests)
│   └── integration_tests.rs    # Integration test suite
├── Cargo.toml                  # Dependencies (ssh2, secstr, rusqlite, anyhow, chrono)
└── tauri.conf.json            # Tauri configuration
```

## Data Models & Interfaces

### Data Models

#### TypeScript Type System
Core types defined in `src/lib/types/api.ts`:

```typescript
// Core state types
type ConnectionState = 'Disconnected' | 'Connecting' | 'Connected' | 'Expired';
type JobStatus = 'CREATED' | 'PENDING' | 'RUNNING' | 'COMPLETED' | 'FAILED' | 'CANCELLED';

// Job management types
interface CreateJobParams {
  jobName: string;
  namdConfig: NAMDConfig;
  slurmConfig: SlurmConfig;
  inputFiles: InputFile[];
}

interface JobInfo {
  jobId: JobId;
  jobName: string;
  status: JobStatus;
  slurmJobId?: SlurmJobId;
  createdAt: Timestamp;
  updatedAt?: Timestamp;
  // ... additional fields
}
```

#### Rust Type System

NAMDRunner maintains strict type safety between TypeScript frontend and Rust backend:

**Core Types:**
- **`ApiResult<T>`** - Consistent return type for all Tauri commands
- **`JobInfo`** - Complete job lifecycle information with all status fields
- **`SessionInfo`** - SSH connection state and authentication data
- **`NAMDConfig`** - NAMD simulation parameters
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

NAMDRunner provides 18 Tauri commands organized into three main categories:
- **Connection Management**: SSH connectivity, mode switching, command execution
- **Job Lifecycle**: Create, submit, monitor, sync, complete, cleanup operations
- **File Management**: Upload, download, listing, and file operation commands

**Key Features:**
- **Consistent `ApiResult<T>` return type** for uniform error handling
- **Real-time progress tracking** via Tauri event system
- **Demo/real mode switching** for development and production use
- **Comprehensive automation** with job completion and results preservation

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
- **SSH/SFTP Integration** - Complete secure cluster connectivity and file operations
  > **For SSH/SFTP security patterns and connection management**, see [`docs/SSH.md`](SSH.md)
  - Connection management: `src-tauri/src/ssh/manager.rs::ConnectionManager` (singleton with retry logic)
  - Authentication: `src-tauri/src/ssh/connection.rs::SSHConnection::connect()` (password-only, SecurePassword)
  - File operations: `src-tauri/src/ssh/sftp.rs::SFTPOperations` (upload/download with progress tracking)
  - Directory operations: `src-tauri/src/ssh/manager.rs::create_directory()` (recursive with validation)
  - Command execution: `src-tauri/src/ssh/commands.rs::CommandExecutor` (SLURM integration)
  - Error handling: `src-tauri/src/ssh/errors.rs` (comprehensive categorization and retry logic)
  - **Note**: All SSH/SFTP/directory operations are handled by Rust backend infrastructure
- **Job Automation System** - Complete lifecycle automation with progress tracking
  > **For complete automation implementation details, patterns, and integration examples**, see [`docs/AUTOMATIONS.md`](AUTOMATIONS.md)
  - Job creation: `src-tauri/src/automations/job_creation.rs::execute_job_creation_with_progress()`
  - Job submission: `src-tauri/src/automations/job_submission.rs::execute_job_submission_with_progress()`
  - Job completion: `src-tauri/src/automations/job_completion.rs::execute_job_completion_with_progress()`
- **Database Layer** - SQLite persistence with status history tracking
  > **For complete database schemas, implementation patterns, and data management details**, see [`docs/DB.md`](DB.md)
  - Main database: `src-tauri/src/database/mod.rs::NAMDRunnerDatabase`
  - Helpers: `src-tauri/src/database/helpers.rs`
- **Security Validation** - Input sanitization and path safety throughout
  > **For security requirements and implementation patterns**, see [`docs/CONTRIBUTING.md#security-requirements`](CONTRIBUTING.md#security-requirements)
  - Input validation: `src-tauri/src/validation.rs::input` module
  - Path safety: `src-tauri/src/validation.rs::paths` module
  - Shell escaping: `src-tauri/src/validation.rs::shell` module

**Frontend (TypeScript/Svelte):**
- **IPC Communication** - Strongly-typed commands with consistent error handling
  > **For detailed IPC interfaces and command specifications**, see [`docs/API.md`](API.md)
  - Main client: `src/lib/ports/coreClient.ts::CoreClient` interface
  - Tauri implementation: `src/lib/ports/coreClient-tauri.ts`
  - Mock implementation: `src/lib/ports/coreClient-mock.ts`
- **Demo Mode Support** - Seamless switching between mock and real cluster operations
  - Client factory: `src/lib/ports/clientFactory.ts::createClient()`
- **Reactive State Management** - Real-time job status updates and progress tracking
  - Job state: `src/lib/stores/jobs.ts`
  - Session state: `src/lib/stores/session.ts`
- **Component Architecture** - Focused, composable UI components
  > **For UI/UX design patterns and component specifications**, see [`docs/DESIGN.md`](DESIGN.md)
  - App shell: `src/lib/components/AppShell.svelte`
  - Layout components: `src/lib/components/layout/` directory
  - Page components: `src/lib/components/pages/` directory

> **For complete development patterns, coding standards, and implementation guidelines**, see [`docs/CONTRIBUTING.md`](CONTRIBUTING.md)
> **For SSH/SFTP security patterns and connection management**, see [`docs/SSH.md`](SSH.md)

## Development Environment

NAMDRunner supports dual-mode development with comprehensive tooling:
- **Demo/Real Mode Switching** for offline development and production use
- **Hot reload development** with `npm run tauri dev`
- **Cross-platform builds** targeting Windows, Linux, and macOS

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
- **SSH Console Buffer**: Global store with configurable size limit for debugging

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