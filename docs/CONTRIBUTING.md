# Contributing to NAMDRunner

**Developer quick start and coding standards.**

> See project README for project overview.
>
> **Related Docs:**
>
> - [ARCHITECTURE.md](ARCHITECTURE.md) - System design and structure
> - [DESIGN.md](DESIGN.md) - UI patterns
> - [API.md](API.md) - IPC interfaces

## Development Setup

### Prerequisites

- **Node.js LTS** (via nvm recommended)
- **Rust toolchain** (via rustup.rs)
- **Git**

### First-Time Setup

> Follow the official Tauri v2 documentation for platform prerequisites: <https://v2.tauri.app/start/>

**Linux/Fedora:**

```bash
# Tauri system dependencies
sudo dnf install -y webkit2gtk4.1-devel openssl-devel curl wget file libappindicator-gtk3-devel librsvg2-devel

# Install Rust and Node
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
nvm install --lts && nvm use --lts

# Clone and setup
git clone https://github.com/yourusername/namdrunner.git
cd namdrunner
npm install
npm run tauri dev  # Smoke test
```

**Windows:**

```powershell
# Prerequisites:
# 1. Rust from https://rustup.rs (MSVC toolchain)
# 2. Node.js LTS from https://nodejs.org
# 3. Visual Studio Build Tools with Desktop C++ workload

git clone https://github.com/yourusername/namdrunner.git
cd namdrunner
npm install
npm run tauri build
```

### Development Commands

```bash
# Frontend
npm run dev               # Svelte dev server
npm run build             # Build static frontend
npm run check             # TypeScript + svelte-check
npm run lint              # ESLint + Prettier

# Backend (in src-tauri/)
cargo test                # Rust unit tests
cargo clippy              # Rust lint
cargo check               # Fast compile check

# Full app
npm run tauri dev         # Run with hot reload
npm run tauri build       # Production build
```

## Core Coding Principles

### 1. Backend-First Architecture

**ALL business logic, validation, and calculations in Rust backend:**

- ✅ Resource validation (cores, memory, partition limits)
- ✅ Calculations (cost estimation, queue times)
- ✅ Cluster configuration (partitions, QoS)
- ✅ Template rendering and validation
- ✅ File operations (upload, download)

**Frontend is PURE presentation layer:**

- ✅ Display logic (formatting, icons, colors)
- ✅ User interaction (clicks, form inputs)
- ✅ State caching (reactive stores)
- ❌ NO validation logic
- ❌ NO calculations
- ❌ NO business rules

### 2. Logging System (Backend Only)

**All logging uses backend macros** - Frontend has no logging system:

```rust
// In Rust backend - these appear in app logs panel
log_info!(category: "Jobs", message: "Job created", details: "ID: {}", job_id);
log_error!(category: "SSH", message: "Connection failed", details: "{}", error);
log_debug!(category: "Validation", message: "Checking resources");

// User-facing success/error messages - add show_toast
log_info!(category: "Jobs", message: "Job submitted successfully", show_toast: true);
log_error!(category: "Connection", message: "Authentication failed", show_toast: true);
```

### 3. Error Handling

**Backend: ApiResult<T> pattern**

```rust
#[tauri::command(rename_all = "snake_case")]
pub async fn create_job(params: CreateJobParams) -> ApiResult<JobInfo> {
    match execute_job_creation(params).await {
        Ok(job_info) => {
            log_info!(category: "Jobs", message: "Job created", show_toast: true);
            ApiResult::success(job_info)
        }
        Err(e) => {
            log_error!(category: "Jobs", message: "Failed to create job", details: "{}", e, show_toast: true);
            ApiResult::error(e.to_string())
        }
    }
}
```

**Frontend: Simple result checking**

```typescript
const result = await invoke<ApiResult<JobInfo>>('create_job', { params });
if (result.success && result.data) {
    jobsStore.addJob(result.data);
    // Backend already toasted success
}
// Backend already toasted error - no frontend error handling needed
```

### 4. Command Architecture

**ALL Tauri commands in `commands/` module:**

```
commands/
├── app.rs          # Application initialization, logging
├── cluster.rs      # Cluster configuration commands
├── connection.rs   # SSH connection management
├── jobs.rs         # Job lifecycle commands
├── files.rs        # File operations
├── templates.rs    # Template CRUD
├── database.rs     # Database management
└── validation.rs   # Validation command wrappers
```

**Commands are thin adapters:**

```rust
// Command wrapper in commands/cluster.rs
#[tauri::command(rename_all = "snake_case")]
pub fn get_cluster_capabilities() -> ClusterCapabilities {
    cluster::get_cluster_capabilities()  // Calls business logic
}
```

**Business logic stays in respective modules:**

- `cluster.rs` - Cluster capabilities business logic
- `validation/job.rs` - Resource validation business logic
- `automations/` - Job lifecycle workflows

## Backend Development Patterns

### Module Organization

> **For complete module structure**, see [`docs/ARCHITECTURE.md#module-structure`](ARCHITECTURE.md#backend-module-structure)

**Key modules:**

- `commands/` - All Tauri IPC commands (thin adapters)
- `automations/` - Job lifecycle workflows
- `validation/` - Business logic validation (job.rs, template.rs)
- `security/` - Input sanitization (input.rs, shell.rs, credentials.rs)
- `ssh/` - SSH/SFTP operations and path utilities
- `cluster.rs` - Cluster capabilities (business logic)

### Security - Input Sanitization

**Always sanitize user input before use:**

```rust
use crate::security::input;

// Sanitize job ID
let clean_id = input::sanitize_job_id(&user_input)?;

// Sanitize username
let clean_username = input::sanitize_username(&username)?;
```

**Never use user input directly in:**

- Shell commands (use `security::shell::escape_parameter()`)
- File paths (use `ssh::paths::project_directory()`)
- SLURM commands (sanitize first)

### Database Access

**Use `with_database()` wrapper for all operations:**

```rust
use crate::database::with_database;

// Save job
with_database(|db| db.save_job(&job_info))?;

// Load job
let job = with_database(|db| db.load_job(&job_id))?
    .ok_or_else(|| anyhow!("Job not found"))?;
```

### IPC Commands

**All commands use snake_case parameters:**

```rust
#[tauri::command(rename_all = "snake_case")]  // ALWAYS include this
pub async fn submit_job(job_id: String) -> ApiResult<JobInfo> {
    // Frontend sends: { job_id: "test_001" }
    // NOT: { jobId: "test_001" }
}
```

## Frontend Development Patterns

### Backend-First Design

**Stores are pure caches:**

```typescript
// ✅ Store just caches backend data
export const clusterConfig = writable<ClusterCapabilities | null>(null);

export async function loadClusterConfig() {
    const result = await invoke('get_cluster_capabilities');
    clusterConfig.set(result);
}

// ❌ NO validation or business logic in stores
```

### Design System Usage

**Use centralized `--namd-*` CSS variables:**

```svelte
<!-- ✅ Use design system classes -->
<button class="namd-button namd-button--primary">Save</button>
<div class="namd-field-group">
  <label class="namd-label">Name</label>
  <input class="namd-input" />
</div>

<!-- ❌ Never create custom button/form styles -->
<button class="my-custom-btn">Save</button>
```

### Dialog Components

**Use unified dialog primitives:**

- `Dialog.svelte` - Base modal (header/body/footer slots)
- `EditDialog.svelte` - Edit forms with Cancel/Save buttons
- `ConfirmDialog.svelte` - Confirmations and alerts

```svelte
<!-- ✅ Use EditDialog for forms -->
<EditDialog {isOpen} title="Edit Partition" {onSave} {onClose}>
  <svelte:fragment slot="form">
    <!-- Form fields here -->
  </svelte:fragment>
</EditDialog>
```

## Security Requirements

### Core Principles

1. **No Credential Persistence** - Passwords exist only in memory during active sessions
2. **Input Sanitization** - Always sanitize before use:

   ```rust
   use crate::security::input;
   let clean_id = input::sanitize_job_id(&user_input)?;
   ```

3. **Shell Escaping** - Always escape parameters:

   ```rust
   use crate::security::shell;
   let safe_param = shell::escape_parameter(&user_input);
   ```

4. **Path Safety** - Use centralized path utilities:

   ```rust
   use crate::ssh::paths;
   let project_dir = paths::project_directory(&username, &job_id)?;
   ```

5. **Secure Memory** - Use `SecurePassword` for credentials (auto-clears on drop)
6. **Never Log Secrets** - No passwords, credentials, or sensitive data in logs

## Testing Strategy

### 3-Tier Testing Architecture

**Tier 1 (Frontend):** TypeScript/Svelte unit tests

- UI logic, stores, display formatting
- Mock Tauri invoke with Vitest: `vi.mock('@tauri-apps/api/core')`

**Tier 2 (Backend):** Rust unit tests

- Business logic: validation, parsing, rendering
- Security: input sanitization, path safety, shell escaping
- Test what we build, not external libraries (ssh2, rusqlite)

**Tier 3 (Integration):** Manual testing + E2E

- Full workflows with real cluster connection
- Template-based job creation
- File uploads/downloads

### What to Test

- ✅ Security validation
- ✅ Business logic
- ✅ Command parsing
- ✅ Error classification

### Testing Commands

```bash
# Frontend
npm test                # Vitest (watch mode)
npm run test:run        # Vitest (run once)

# Backend
cargo test              # All Rust tests
cargo clippy            # Linting

# Before completing work
npm run check           # TypeScript check
```

## Anti-Patterns to Avoid

### Critical Anti-Patterns

1. **Thin Wrappers** - Functions that only delegate to other functions
2. **Business Logic in Frontend** - All validation/calculations belong in Rust
3. **Silent Failures** - Always surface errors to user
4. **Hardcoded Values** - Use database/config for cluster-specific data
5. **Custom CSS** - Use `--namd-*` design system variables
6. **console.log(), println!() or eprintln!()** - Use backend `log_info!()` macros only

### Code Quality Standards

1. **Warnings as Errors**: Investigate and fix build and linting warnings, don't ignore or suppress
2. **Fail-Fast Architecture**: No silent fallbacks. Return errors explicitly. Let users know when something's wrong
3. **From Scratch Refactors**: Ask: "If building from scratch, how would I solve this?". Prefer refactoring over patching.

## Quick Reference

### Backend Logging

```rust
log_info!(category: "Category", message: "Message", details: "Detail: {}", value);
log_error!(category: "Category", message: "Error", details: "{}", error, show_toast: true);
```

### Security Sanitization

```rust
use crate::security::input;
use crate::security::shell;

let clean_id = input::sanitize_job_id(&input)?;
let safe_param = shell::escape_parameter(&input);
```

### Database Access

```rust
use crate::database::with_database;

with_database(|db| db.save_job(&job_info))?;
let job = with_database(|db| db.load_job(&job_id))?.ok_or(...)?;
```

### IPC Parameters

```rust
#[tauri::command(rename_all = "snake_case")]  // ALWAYS include
pub fn my_command(job_id: String, partition_id: String) -> ApiResult<T>
```

### Frontend Design System

```svelte
<button class="namd-button namd-button--primary">Save</button>
<div class="namd-field-group">
  <label class="namd-label">Field</label>
  <input class="namd-input" />
</div>
```
