# NAMDRunner System Architecture

**System design, module structure, and architectural overview.**

> See project README for project overview.

## Documentation Map

- **[CONTRIBUTING.md](CONTRIBUTING.md)**: Development setup, coding standards, and testing strategy.
- **[DESIGN.md](DESIGN.md)**: UI components, design system, and Svelte patterns.
- **[API.md](API.md)**: IPC command interfaces and data contracts.
- **[DB.md](DB.md)**: Database schemas, storage patterns, and JSON metadata.
- **[SSH.md](SSH.md)**: SSH/SFTP connection management and file operations.
- **[AUTOMATIONS.md](AUTOMATIONS.md)**: Job lifecycle automation workflows.
- **[reference/](reference/)**: SLURM commands, NAMD configuration, and cluster specifics.

## Technology Stack

- **Frontend**: Svelte 4 + TypeScript (Vite build)
- **Backend**: Rust + Tauri v2 (No bundled Node.js/Python)
- **Database**: SQLite (`rusqlite`) - Local cache and template storage
- **SSH/SFTP**: `ssh2` crate (libssh2) - Password-authenticated connections
- **State Management**: Svelte Stores (pure local cache of backend state)
- **Styling**: Vanilla CSS variables (`--namd-*`)
- **Testing**: Vitest (Frontend), Cargo Test (Backend)

## Module Structure

### Frontend (Svelte/TypeScript)

```
src/
├── lib/
│   ├── types/                   # TypeScript type definitions
│   │   ├── api.ts               # Core API types matching Rust types
│   │   ├── template.ts          # Template types matching Rust types
│   │   └── cluster.ts           # Cluster capability types (matches backend)
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
│   │   │   ├── ReviewTab.svelte         # Final review before submission
│   │   │   └── ValidationDisplay.svelte # Real-time validation feedback UI
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
│   │   │   ├── ConfirmDialog.svelte      # Confirmation/alert dialogs (with variant support)
│   │   │   └── PreviewModal.svelte       # Template preview modal
│   │   └── AppShell.svelte      # Main application shell
│   ├── types/                   # TypeScript types
│   │   ├── api.ts
│   │   └── template.ts
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
│       └── setup.ts             # Test setup configuration
├── routes/
│   ├── +layout.ts              # Layout configuration
│   └── +page.svelte            # Main application interface
└── app.html
```

### Backend (Rust)

```
src-tauri/
├── src/
│   ├── main.rs                  # Application entry point
│   ├── lib.rs                   # Tauri app configuration and command registration
│   ├── commands/                # ALL Tauri IPC command handlers (thin adapters)
│   │   ├── mod.rs              # Command module exports
│   │   ├── connection.rs       # SSH connection lifecycle commands
│   │   ├── cluster.rs          # Cluster configuration commands
│   │   ├── jobs.rs             # Job lifecycle commands (create/submit/sync/delete/complete)
│   │   ├── files.rs            # File management commands
│   │   ├── templates.rs        # Template management commands (list/get/create/update/delete/export/import/validate)
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
│   │   └── renderer.rs         # Template rendering with variable substitution
│   ├── ssh/                    # SSH/SFTP service implementation
│   │   ├── mod.rs              # SSH module exports
│   │   ├── connection.rs       # Low-level SSH connection handling
│   │   ├── manager.rs          # Connection lifecycle and directory management
│   │   ├── commands.rs         # SSH command execution and parsing
│   │   ├── sftp.rs             # File transfer operations
│   │   ├── metadata.rs         # Job metadata upload utilities
│   │   ├── paths.rs            # Path construction and validation utilities
│   │   ├── directory_structure.rs # Standard job directory structure definitions
│   │   ├── errors.rs           # SSH error mapping and classification
│   │   └── test_utils.rs       # Development utilities
│   ├── slurm/                  # SLURM integration
│   │   ├── mod.rs              # SLURM module exports
│   │   ├── commands.rs         # SLURM command builder and patterns
│   │   ├── script_generator.rs # SLURM script generation (no NAMD config - uses templates)
│   │   └── status.rs           # Job status synchronization
│   ├── database/               # Data persistence layer
│   │   ├── mod.rs              # Database with jobs and templates tables
│   ├── types/                  # Rust type definitions
│   │   ├── mod.rs              # Type module exports
│   │   ├── core.rs             # Core domain types (JobInfo, SessionInfo, ApiResult, JobStatus, ConnectionState, etc.)
│   │   ├── commands.rs         # Command parameter and result types
│   │   └── response_data.rs    # Response data structures for IPC commands
│   ├── validation/             # Business logic validation
│   │   ├── mod.rs              # Validation module exports
│   │   ├── job.rs              # Resource and business logic validation
│   │   └── template.rs         # Template value validation
│   ├── security/               # Security and input sanitization
│   │   ├── mod.rs              # Security module exports
│   │   ├── credentials.rs      # Secure password handling (SecurePassword)
│   │   ├── input.rs            # Input sanitization and validation
│   │   └── shell.rs            # Shell escaping utilities
│   ├── logging.rs              # Rust-to-Frontend logging bridge system
│   ├── retry.rs                # Exponential backoff retry implementation
│   └── cluster.rs              # Cluster capabilities and configuration (business logic)
├── templates/                   # Embedded template JSON files
│   ├── vacuum_optimization_v1.json
│   └── explicit_solvent_npt_v1.json
├── Cargo.toml                  # Dependencies (ssh2, secstr, rusqlite, anyhow, chrono)
└── tauri.conf.json            # Tauri configuration
```

## Data Flow

1. **User Action** (UI) -> **Svelte Store**
2. **IPC Call** (`invoke`) -> **Tauri Command** (Rust)
3. **Command Handler** -> **Business Logic** (Validation/Automation)
4. **Integration Layer** -> **SSH/DB/File System**
5. **Response** -> **Tauri Command** -> **UI Update**

## Build & Deployment

- **Targets**: Windows, Linux, macOS
- **Build System**: GitHub Actions (Windows, Linux)
- **Artifacts**: Portable `.exe` (Windows), AppImage/Deb (Linux)
- **CI/CD**: Automatic builds on push, release creation on tag
