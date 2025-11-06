# Contributing to NAMDRunner

**How we expect you to write code and why** - This document covers development setup, coding standards, testing strategies, and best practices for contributing to NAMDRunner.

> **For system architecture and design principles**, see [`docs/ARCHITECTURE.md`](ARCHITECTURE.md)
> **For SSH/SFTP connection patterns and security**, see [`docs/SSH.md`](SSH.md)
> **For UI/UX components and design patterns**, see [`docs/DESIGN.md`](DESIGN.md)
> **For database schemas and data management**, see [`docs/DB.md`](DB.md)
> **For IPC interfaces and API contracts**, see [`docs/API.md`](API.md)
> **For SLURM/NAMD command patterns**, see [`docs/reference/`](reference/) directory

## Table of Contents
- [Development Setup](#development-setup)
  - [Prerequisites](#prerequisites)
  - [First-Time Setup](#first-time-setup)
  - [Development Commands](#development-commands)
- [VM Development Environment (Optional)](#vm-development-environment-optional)
  - [Platform](#platform)
  - [MacOS Host Setup (Outside the VM)](#macos-host-setup-outside-the-vm)
  - [VM Setup (Linux/Fedora)](#vm-setup-linuxfedora)
  - [Rust Builds](#rust-builds)
- [Developer Standards & Project Philosophy](#developer-standards--project-philosophy)
  - [Quick Start - Top 7 Critical Rules](#quick-start---top-7-critical-rules)
  - [Core Architectural Principles](#core-architectural-principles)
    - [1. Direct Code Patterns](#1-direct-code-patterns)
    - [2. Result<T> Error Handling](#2-result<t>-error-handling)
  - [Anti-Patterns to Avoid](#anti-patterns-to-avoid)
    - [Critical Anti-Patterns (From NAMDRunner Experience)](#critical-anti-patterns-from-namdrunner-experience)
    - [General Anti-Patterns](#general-anti-patterns)
  - [Security Requirements](#security-requirements)
    - [Core Security Principles](#core-security-principles)
    - [Path Security & Input Validation](#path-security--input-validation)
    - [Command Injection Prevention](#command-injection-prevention)
    - [Connection Lifecycle Management](#connection-lifecycle-management)
  - [Service Development Patterns](#service-development-patterns)
    - [Dependency Injection](#dependency-injection)
    - [Path Management](#path-management)
    - [State Management](#state-management)
  - [Performance Guidelines](#performance-guidelines)
    - [Integration Best Practices](#integration-best-practices)
      - [Command Reliability](#command-reliability)
      - [Interaction Optimization](#interaction-optimization)
      - [Error Recovery](#error-recovery)
    - [Retry Logic Implementation](#retry-logic-implementation)
    - [Error Mapping for User Experience](#error-mapping-for-user-experience)
    - [Async Operations with Blocking Libraries](#async-operations-with-blocking-libraries)
  - [Build Configuration](#build-configuration)
    - [TypeScript Configuration](#typescript-configuration)
    - [Rust Quality Tools](#rust-quality-tools)
- [Testing Strategy](#testing-strategy)
  - [NAMDRunner Testing Philosophy](#namdrunner-testing-philosophy)
  - [3-Tier Testing Architecture](#3-tier-testing-architecture)
  - [What We Test](#what-we-test)
  - [What We Don't Test](#what-we-dont-test)
  - [Testing Commands](#testing-commands)
  - [Why This Works](#why-this-works)
  - [Mock Development Philosophy](#mock-development-philosophy)
  - [Repository Structure](#repository-structure)
- [UX Requirements](#ux-requirements)
- [References](#references)
- [Related Documentation](#related-documentation)

## Development Setup

### Prerequisites
- **Node.js LTS** (via nvm recommended)
- **Rust toolchain** (via rustup.rs)
- **Git**

### First-Time Setup

> Follow the official Tauri v2 documentation for platform prerequisites: https://v2.tauri.app/start/

#### Linux/Fedora
```bash
# Tauri system dependencies
sudo dnf check-update
sudo dnf install -y webkit2gtk4.1-devel openssl-devel curl wget file libappindicator-gtk3-devel librsvg2-devel libxdo-devel
sudo dnf group install -y "C Development Tools and Libraries"

# Install Rust
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
source ~/.cargo/env

# Install Node.js via nvm
nvm install --lts && nvm use --lts

# Clone and setup project
git clone https://github.com/yourusername/namdrunner.git
cd namdrunner
npm install

# Smoke test
npm run tauri dev
```

#### macOS

```bash
# Install Dev tools
xcode-select --install
# Install Node/Rust with Homebrew
brew install node rust

# Clone repo
git clone https://github.com/yourusername/namdrunner.git
cd namdrunner
npm install
```

#### Windows

```powershell
# Prerequisites for Windows development
# 1. Install Rust from https://rustup.rs (MSVC toolchain)
# 2. Install Node.js LTS from https://nodejs.org
# 3. Install Visual Studio Build Tools with Desktop C++ workload
#    OR Visual Studio 2022 Community with Desktop C++ workload
# 4. WebView2 runtime (usually pre-installed on Windows 10+)
# 5. VBSCRIPT optional feature (for MSI installers - enabled by default)

# Clone and setup project
git clone https://github.com/yourusername/namdrunner.git
cd namdrunner
npm install

# Test cross-platform build
npm run tauri build
```

**Note for Windows Development**: The Rust code includes cross-platform path handling that automatically detects Windows vs Unix environments. Windows builds use static OpenSSL linking to eliminate runtime dependencies.

**MSI Installer Requirements**: Building MSI packages requires the VBSCRIPT optional feature to be enabled. This is enabled by default on most Windows installations. If you encounter "failed to run light.exe" errors, see `docs/WINDOWS_BUILD.md` for troubleshooting steps.

### Development Commands
```bash
# Frontend development
npm run dev               # Svelte dev server (Vite)
npm run build             # Build static frontend
npm run preview           # Preview built assets
npm run check             # svelte-kit sync + svelte-check
npm run check:watch       # svelte-check --watch
npm run lint              # ESLint + Prettier (check)
npm run lint:fix          # ESLint --fix + Prettier --write

# Tests
npm run test              # Vitest unit tests
npm run test:vitest-ui    # Vitest UI
npm run test:run          # Vitest run (CI-friendly)
npm run test:ui           # UI testing toolkit (under Xvfb)
npm run test:e2e          # WebdriverIO E2E (under Xvfb)

# Rust (executed in `src-tauri/`)
cargo test                # Rust unit tests
cargo clippy              # Rust lint
cargo check               # Fast compile check

# Full Tauri application
npm run tauri dev         # Run app with hot reload
npm run tauri build       # Build release binary
```

### Cross-Platform Build Information

NAMDRunner supports building on multiple platforms with automatic CI/CD:

#### Linux Builds (Primary Development Platform)
- **Native builds**: AppImage (portable) and .deb packages
- **CI/CD**: Automatic builds on Ubuntu latest
- **E2E Testing**: Full WebdriverIO testing with Xvfb
- **Agent Testing**: UI testing with Playwright

#### Windows Builds (GitHub Actions)
- **Bundle formats**: MSI installer and NSIS executable
- **CI/CD**: Automatic builds on Windows latest
- **Static linking**: OpenSSL vendored for dependency-free distribution
- **Cross-platform paths**: Rust code handles Windows and Unix paths

#### macOS Builds (Future)
- **Bundle formats**: .dmg and .app bundle
- **Status**: Ready for implementation when needed


## Cross-Platform Development

> See [`docs/WINDOWS_BUILD.md`](WINDOWS_BUILD.md) for Windows build information

## Developer Standards & Project Philosophy

### Quick Start - Top 7 Critical Rules

1. **No Thin Wrappers**: Don't create functions that just delegate to other functions
2. **Direct Error Handling**: Use `Result<T>` patterns, never suppress errors with `console.warn()`
3. **No Repository Pattern**: Use direct database calls with `with_database()`
4. **Security First**: Always sanitize user input, never log credentials
5. **Simple Mocks**: Predictable test behavior over complex simulation
6. **Balance DRY with Simplicity**: Reduce redundancy and centralize common patterns, but avoid over-abstraction that creates unnecessary complexity
7. **Easy to Reason About**: Write code that is clear and understandable when reading it

### Core Architectural Principles

### 1. Progressive Enhancement
**Start Simple**: Begin with the simplest solution that works, add abstraction only when you have 3+ use cases.

```typescript
// ✅ Start with simple utility functions
export function parseMemoryString(memory: string): number {
  // Direct implementation
}

// ✅ Add abstraction when pattern emerges across multiple components
export const memoryUtils = {
  parse: parseMemoryString,
  format: formatMemory,
  validate: validateMemoryInput
};

// ❌ Don't create abstraction prematurely
class MemoryConfigurationManagerFactory {
  // Over-engineered from the start
}
```

**Prefer Composition**: Build complex functionality by combining simple, focused utilities.
```typescript
// ✅ Composable utilities
const validation = validateResourceRequest(cores, memoryGB, walltimeHours, partition, qos);
const cost = calculateJobCost(cores, walltimeHours, hasGpu, gpuCount);
const queue = estimateQueueTime(cores, partition);

// ❌ Monolithic service
class CompleteJobValidator {
  // Everything in one place
}
```

### 2. Direct Code Patterns
- Avoid thin wrapper functions that only delegate to other functions
- Use direct database calls with `with_database()` instead of repository patterns
- Call automation functions directly in Tauri command handlers
- Functions should add value, not just delegate

### 3. Result<T> Error Handling
**Consistent Return Types**: All operations that can fail return `Result<T>`.
```typescript
// ✅ Result pattern
async function validateConnection(config: ConnectionConfig): Promise<Result<ValidationResult>> {
  try {
    const result = await performValidation(config);
    return { success: true, data: result };
  } catch (error) {
    return { success: false, error: toConnectionError(error, 'Validation') };
  }
}
```

**Error Chaining in Rust**: Use `anyhow` with context.
```rust
use anyhow::{Result, Context};

impl ConnectionManager {
    pub async fn setup_workspace(&self, username: &str, job_id: &str) -> Result<WorkspaceInfo> {
        let project_dir = format!("/projects/{}/namdrunner_jobs/{}", username, job_id);

        self.create_directory(&project_dir)
            .await
            .context("Failed to create project directory")?;

        Ok(WorkspaceInfo { project_dir })
    }
}
```

**Never Suppress Errors**: Don't use console.warn() or hardcoded fallbacks.
```typescript
// ❌ Silent failure
async function createJob(params: CreateJobParams): Promise<string> {
  try {
    const result = await client.createJob(params);
    return result.job_id;
  } catch (error) {
    console.warn('Job creation failed, using placeholder');
    return `job_${Date.now()}`;
  }
}

// ✅ Proper error handling
async function createJob(params: CreateJobParams): Promise<Result<string>> {
  try {
    const result = await client.createJob(params);
    return { success: true, data: result.job_id };
  } catch (error) {
    return { success: false, error: toConnectionError(error, 'JobCreation') };
  }
}
```

## UI Development Principles

### Core UI Patterns
- **Utility-First Design**: Create focused utility functions that serve multiple components
- **Single Source of Truth**: Centralize configuration and data definitions
- **Component Composition**: Build reusable, focused components
- **Reactive Data Flow**: Use Svelte's reactive statements with utility functions
- **Consistent Design System**: Follow unified styling patterns across all components

### Connection-Aware UI
- **Disable destructive actions when disconnected**: Delete job, sync, file downloads
- **Confirmation dialogs for data loss**: Warn users before permanent deletions
- **Clear workflow expectations**: Inform users when operations are multi-step (e.g., "Create Job" uploads files but doesn't submit to SLURM yet)

> **For complete UI component patterns, design system usage, and Svelte implementation examples**, see [`docs/DESIGN.md`](DESIGN.md)

## Service Development Patterns

### 1. Utility Function Design
**Pure Functions**: Utility functions should be pure, predictable, and side-effect free.

```typescript
// ✅ Pure utility function
export function parseMemoryString(memory: string): number {
  if (!memory) return 0;
  const cleanMemory = memory.toString().toLowerCase().replace(/\s+/g, '');
  const match = cleanMemory.match(/^(\d+(?:\.\d+)?)([a-z]*)/);
  // ... conversion logic
  return value;
}

// ❌ Impure function with side effects
export function parseMemoryString(memory: string): number {
  console.log('Parsing memory:', memory); // Side effect
  if (!memory) {
    showErrorMessage('Memory is required'); // Side effect
    return 0;
  }
  // ...
}
```

**Focused Responsibility**: Each utility should have one clear, well-defined purpose.
```typescript
// ✅ Focused utilities
export function getFileIcon(type: string): string { /* ... */ }
export function getTypeLabel(type: string): string { /* ... */ }
export function getTypeColor(type: string): string { /* ... */ }

// ❌ Mixed responsibilities
export function handleFileType(type: string, action: 'icon' | 'label' | 'color'): string {
  // One function doing multiple things
}
```

**Configuration Centralization**: Keep all related configuration in one place.
```rust
// ✅ Single source of truth
// In cluster.rs (backend)
pub fn get_cluster_capabilities() -> ClusterCapabilities {
  ClusterCapabilities {
    partitions: get_all_partitions(),
    qos_options: get_all_qos_options(),
    // ...
  }
}

// Frontend caches backend data in stores/clusterConfig.ts
// for reactive UI access

// ❌ Scattered configuration
// In Component A: const limits = { amilan: { maxCores: 64 } };
// In Component B: const partitionLimits = { amilan: { maxCores: 64 } };
```

### 2. Frontend Stores Architecture

**Stores are pure caching layers** with no business logic. All business logic lives in the Rust backend.

#### Store Pattern
```typescript
import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/tauri';

// ✅ Store as pure cache
export const clusterConfig = writable<ClusterCapabilities | null>(null);

export async function loadClusterCapabilities() {
    const capabilities = await invoke('get_cluster_capabilities');
    clusterConfig.set(capabilities);
}

// ❌ Store with business logic
export async function loadClusterCapabilities() {
    const capabilities = await invoke('get_cluster_capabilities');
    // Validation logic - belongs in backend!
    if (capabilities.partitions.length === 0) {
        throw new Error('No partitions available');
    }
    clusterConfig.set(capabilities);
}
```

#### Backend-First Design Principle

**All business logic in Rust backend:**
- ✅ Validation (resource limits, NAMD parameters)
- ✅ Calculations (cost estimation, queue times)
- ✅ Cluster configuration (partitions, QoS, limits)
- ✅ File operations (uploads, downloads, metadata)
- ✅ SLURM commands (sbatch, squeue, sacct, scancel)

**Frontend stores only cache:**
- ✅ Reactive state management
- ✅ IPC call wrapping
- ✅ Event handling (progress updates)
- ❌ No validation logic
- ❌ No business rules
- ❌ No calculations

#### Component Patterns
```svelte
<script lang="ts">
    import { jobs } from '$lib/stores/jobs';
    import { clusterConfig } from '$lib/stores/clusterConfig';

    // ✅ Subscribe to stores reactively
    $: jobList = $jobs;
    $: capabilities = $clusterConfig;

    // ✅ Presentational logic (UI concerns)
    function getStatusBadgeClass(status: JobStatus): string {
        return status === 'RUNNING' ? 'badge-success' : 'badge-default';
    }
</script>

<button on:click={() => jobs.createJob(params)}>
    Create Job
</button>
```

**Components handle:**
- ✅ Display logic (formatting, colors, icons)
- ✅ User interaction (button clicks, form inputs)
- ✅ Navigation (routing)
- ❌ Validation (backend only)
- ❌ Calculations (backend only)
- ❌ File operations (backend only)

For high-level architecture overview, see [docs/ARCHITECTURE.md#frontend-architecture](ARCHITECTURE.md#frontend-architecture)

### Anti-Patterns to Avoid

#### Critical Anti-Patterns (From NAMDRunner Experience)
- **Repository Pattern with Single Implementation**: JobRepository wrapping database calls
- **Validation Traits Wrapping Functions**: ValidateId trait wrapping sanitize_job_id
- **Intermediate Business Logic Functions**: Functions only calling execute_with_mode
- **Unused Macros**: Defined but never used (mode_switch! macro)
- **Complex Mock State**: Random error simulation vs predictable testing

#### General Anti-Patterns
- **False Backward Compatibility**: Claims of compatibility when no legacy code exists
- **Over-Engineering**: Creating abstractions before you need them (YAGNI principle)
- **Mixed Concerns**: UI logic in business logic, networking in data persistence

#### Frontend-Backend Separation Anti-Patterns
- **Business Logic in Frontend**: ALL validation, resource calculations, and cluster configuration belongs in Rust backend
- **Frontend Validation**: Frontend has NO validation logic - it's purely a display layer that calls backend for all validation
- **Stub Implementations**: Functions marked with `// TODO: implement` or using mock data must be completed before PR
- **Calculation Functions in UI Layer**: Cost estimation, queue time, resource validation belong in backend
- **Connection State Not Checked**: UI actions must disable when disconnected from server

#### UI-Specific Anti-Patterns
- **CSS Duplication**: Use centralized `namd-*` classes instead of duplicating styles across components
- **Hardcoded Styling**: Use CSS custom properties and design system classes, not hardcoded colors
- **Over-Complex Component APIs**: Keep component interfaces focused and simple

> **For detailed UI patterns, design system usage, and component examples**, see [`docs/DESIGN.md`](DESIGN.md)

### Security Requirements

**Security is a core requirement for NAMDRunner** - all code must follow secure patterns for credential handling, input validation, and system interactions.

### Server Interaction Best Practices

When implementing server operations, follow these guidelines for good cluster citizenship:

- **Use standardized timeouts** - Import timeout constants from `crate::config::timeouts`
- **Validate all inputs** - Use `crate::validation::input::sanitize_job_id()` and path validation before server operations
- **Don't spam the cluster** - Batch operations when possible, respect rate limits in retry logic
- **Handle connection failures gracefully** - Always check `connection_manager.is_connected()` before operations
- **Clean up resources** - Use existing retry patterns from `crate::retry::patterns`
- **Provide actionable errors** - Use user-friendly error messages that guide users to solutions

> **For complete security requirements, implementation patterns, and examples**, see [`docs/SSH.md#security-patterns`](SSH.md#security-patterns)

#### Command Injection Prevention

**CRITICAL**: Always use the centralized command building functions for shell operations:

```rust
// ✅ Correct: Use safe command builder
use crate::validation::shell;
let cmd = shell::build_command_safely("mkdir {} && cd {}", &[dir_name, dir_name])?;

// ✅ Correct: Escape individual parameters
let safe_param = shell::escape_parameter(&user_input);

// ❌ NEVER: Direct string concatenation with user input
let cmd = format!("mkdir {}", user_input); // VULNERABLE TO INJECTION
```

**Required Functions**:
- `shell::build_command_safely(template, params)` - Template-based command building
- `shell::escape_parameter(param)` - Individual parameter escaping
- Located in `src-tauri/src/validation.rs`

#### Logs Panel Debugging

**For SSH/SLURM operations**, log important events to the logs panel for user debugging:

```typescript
// Frontend: Add to logs panel (visible to users)
if (typeof window !== 'undefined' && window.sshConsole) {
  window.sshConsole.addDebug(`[JOBS] Job creation failed: ${error}`);
  window.sshConsole.addCommand(`sbatch job.sbatch`); // Show commands being run
}
```

```rust
// Backend: Use tagged console logs (captured by logs panel)
println!("[SLURM] Submitting job: {}", job_name);
```

**Logs Panel captures**:
- Tagged console logs: `[SSH]`, `[SLURM]`, `[CONNECTION]`, `[JOBS]`
- Backend Rust logs via Tauri events
- User-visible debugging without production noise

**Essential Security Principles**:
- Never log or persist credentials - memory only during sessions
- Validate and sanitize all user input before use
- Prevent directory traversal and command injection attacks
- Use secure memory handling for sensitive data
- Clean up connections and clear credentials properly

### 2. Service Architecture
**Direct Dependencies**: Services use direct imports rather than complex dependency injection.
**Mock Testing**: Use mock implementations at the service boundary level.

> **For SSH/SFTP service patterns and testing approaches**, see [`docs/SSH.md#testing--development`](SSH.md#testing--development)

### 3. Path Management
Use centralized path validation functions from `validation::paths` module. Never construct paths directly. See `src-tauri/src/validation.rs` for safe path utilities like `project_directory()` and `scratch_directory()`. For job directory structure, use `ssh::JobDirectoryStructure` constants.

### 4. State Management
Use state machines for complex state management with validated transitions.

### Performance Guidelines

- **Async Operations**: All I/O must be async to avoid blocking UI
- **Retry with Backoff**: Implement exponential backoff for network operations
- **Resource Cleanup**: Always clean up connections and clear memory

### Integration Best Practices

#### Command Reliability
1. Always load modules before SLURM commands
2. Use full paths for working directories
3. Check command exit codes
4. Parse stderr for error messages
5. Handle network timeouts gracefully

#### Interaction Optimization
1. Batch SLURM queries when possible
2. Cache job status to avoid repeated queries
3. Use background threads for long operations
4. Limit concurrent connections

#### Error Recovery
1. Retry failed commands with exponential backoff
2. Validate connection before SLURM commands
3. Handle partial failures in batch operations
4. Provide clear error messages to users

### Error Handling & Recovery

**All error handling patterns and implementations are consolidated in SSH.md** to avoid duplication across documentation.

> **For complete error handling, retry logic, and async patterns**, see [`docs/SSH.md#error-handling`](SSH.md#error-handling)

**Essential Development Principles**:
- Use `Result<T>` patterns consistently for all operations that can fail
- Implement exponential backoff for retryable network operations
- Convert technical errors to actionable user messages
- Handle blocking operations properly with `spawn_blocking`

### Build Configuration

### TypeScript Configuration
```json
{
  "compilerOptions": {
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "exactOptionalPropertyTypes": true
  }
}
```

**Optional Property Convention**: With `exactOptionalPropertyTypes: true`, follow these patterns:
- **`Result<void>` returns**: Use `{ success: true, data: undefined }` - this is correct for void types
- **Optional object properties**: Use conditional assignment: `if (value !== undefined) obj.prop = value;`
- Never assign `undefined` directly to optional properties (e.g., `prop?: string`) - either conditionally assign or omit the property

### Rust Quality Tools
- `clippy` with `-D warnings` (deny all warnings)
- `rustfmt` for consistent formatting
- `cargo-audit` for security scanning

### CI/CD and Cross-Platform Builds

#### GitHub Actions Workflow
The CI pipeline (`.github/workflows/ci.yml`) currently includes:

- **Basic Linting**: ESLint and TypeScript checking (`npm run lint`, `npm run check`)
- **Rust Checks**: Formatting (`cargo fmt --check`) and Clippy (`cargo clippy -D warnings`)
- **Windows Builds**: Automated MSI and NSIS installer generation for Windows deployment target
- **Build Artifacts**: Automated artifact upload with 30-day retention

**Note**: The CI workflow is currently limited to Windows builds with basic linting. Full test suite runs locally only:
- Frontend unit tests: Run `npm test` locally (Vitest with jsdom)
- Backend unit tests: Run `cargo test` locally in `src-tauri/`
- E2E tests: Run manually using test scripts in `tests/ui/`

#### Cross-Platform Support
- **Rust Code**: Uses `cfg!(windows)` conditionals for platform-specific paths
- **Dependencies**: OpenSSL static linking on Windows, bundled SQLite
- **Tauri Configuration**: Platform-specific bundle settings in `tauri.conf.json`
- **Primary Platform**: Development on Linux, deployment target is Windows

#### Deployment
- **Windows**: Automated builds via GitHub Actions (primary deployment target)
- **Linux**: Development and testing platform only
- **Release Process**: See `docs/WINDOWS_BUILD.md` for configuration

For platform support, build requirements, and system constraints, see [`docs/ARCHITECTURE.md#architecture-principles--constraints`](ARCHITECTURE.md#architecture-principles--constraints).

## Summary: Key Development Principles

1. **Progressive Enhancement**: Start simple, add complexity only when proven necessary
2. **Single Source of Truth**: Centralize configuration, utilities, and styling
3. **Utility-First Design**: Create focused, reusable functions and components
4. **Consistent Patterns**: Use unified naming conventions and design system
5. **Clean Separation**: Keep UI, business logic, and data operations separate
6. **Security First**: Never log credentials, validate all paths, clear sensitive data

This guideline document should be treated as living documentation that evolves with the project. All new code should follow these patterns, and existing code should be refactored to match these standards when possible.

**Remember**: We're building a focused tool for scientists, not an enterprise platform. Keep it simple, reliable, and maintainable.

## Testing Strategy

### NAMDRunner Testing Philosophy

**Core Principle: Business Logic Focus**
Test our logic, not external libraries. Focus on what NAMDRunner does, not how ssh2 or other crates work.

### 3-Tier Testing Architecture
- **Tier 1 (Frontend)**: TypeScript/Svelte unit tests - UI logic, stores, state management
- **Tier 2 (Backend)**: Rust unit tests - command parsing, error mapping, path handling, credential management, validation
- **Tier 3 (Integration)**: Full workflows via Playwright/WebdriverIO

### What We Test
✅ **Security validation** - malicious inputs, path traversal, credential safety (backend only)
✅ **Resource validation** - cluster limits, QoS rules, partition constraints (backend only)
✅ **File path handling** - directory generation, safety checks
✅ **Command parsing** - SLURM output parsing, job state mapping
✅ **Error classification** - which errors are retryable vs fatal
✅ **User workflows** - complete job lifecycle (create → submit → delete)
✅ **Frontend display logic** - utility functions, state management, display formatting
✅ **UI interactions** - button click handlers, form submissions, state changes (without full E2E)
✅ **Component behavior** - component state changes, event handling, prop validation

### What We Don't Test
❌ **External crate functionality** - ssh2 connections, SFTP implementations
❌ **Mock performance** - testing how fast our mocks run
❌ **"Stress Tests"** - unit tests should be fast and focused, not simulate production load
❌ **Fake Delays** - simulating server response times or file upload delays adds no value and slows down test feedback
❌ **Implementation details** - internal state consistency
❌ **Infrastructure complexity** - no SSH test servers or integration environments
❌ **AppHandle-dependent tests** - tests requiring Tauri's AppHandle should be avoided; test business logic directly instead
❌ **Test-Only Code** - If only tests use a function/method, it's dead code. Delete both the code AND the tests.

> **For SSH/SFTP testing patterns and mock infrastructure**, see [`docs/SSH.md#testing--development`](SSH.md#testing--development)

### Testing Tauri Commands

When testing Tauri commands that require `AppHandle`:
1. **Extract business logic** into separate functions that don't depend on AppHandle
2. **Test the logic directly** without involving Tauri infrastructure
3. **Use test wrapper functions** that bypass AppHandle requirements
4. **Rely on manual/E2E testing** for the full integration with Tauri

Example:
```rust
// Instead of testing this directly (requires AppHandle):
#[tauri::command]
pub async fn create_job(app: AppHandle, params: CreateJobParams) -> CreateJobResult {
    // ... uses app for database, events, etc.
}

// Test the business logic separately:
pub fn validate_job_params(params: &CreateJobParams) -> Result<()> {
    // Business logic that can be tested without AppHandle
}
```

### Testing Commands

**⏱️ Note on Test Startup Time**: The unit test suite (`npm test`) takes approximately 15-20 seconds to initialize due to jsdom environment setup. This is normal behavior - the tests are not hanging.

#### Quick start for UI development and agent debugging
```bash
Xvfb :99 -screen 0 1280x720x24 &  # Virtual display for SSH/CI environments
export DISPLAY=:99

# Start Vite dev server (takes 1-3 minutes on first start)
npm run dev &                      # Run in background
sleep 120                          # Wait for server startup

# Verify server is ready
curl -s http://localhost:1420 > /dev/null && echo "Server ready" || echo "Server not ready"

# Run headless UI tests (for SSH environments)
node tests/ui/headless-visual-check.js  # Custom headless script
# OR
npm run test:ui                    # Standard debug toolkit (non-headless)
```

#### Standard testing commands
```bash
# Frontend testing
npm test                # Vitest unit tests (watch mode)
npm run test:run        # Vitest unit tests (run once)
npm run test:ui         # UI testing with Playwright

# Backend testing
cargo test              # Rust unit tests
cargo clippy            # Rust linting

# Full integration testing
npm run test:e2e        # Desktop E2E testing (Linux)
```

**Before Completing Code Work**: Always run `npm run test:run` and `npm run check` to verify unit tests pass and types are correct. The production build command (`npm run build:check`) automatically runs these checks before building.

### Why This Works
Scientists need reliability over performance. A desktop app that safely handles credentials and prevents security vulnerabilities is more valuable than one optimized for millisecond performance differences.

**Most valuable testing insights:**
- **Integration testing finds more bugs than unit tests** - SSH operations and cluster behavior have many edge cases
- **Mock mode enables offline testing** - No cluster required for development
- **Test error conditions explicitly** - Invalid input, network failures, etc.

### Development Process Best Practices

**Task discipline:**
- **One task at a time** prevents scope creep
- **Comprehensive documentation** helps with context switching
- **Clear completion criteria** prevents "90% done" syndrome

**Testing approach:**
- Mock mode for offline development and automated testing
- Periodic integration tests with real cluster
- Focus on business logic, not external library internals

**Documentation priorities:**
- **Command reference docs** with exact syntax (most valuable)
- **Cluster-specific quirks** and workarounds
- **Working example** scripts that are proven to function
- **Error message catalog** with solutions

**Avoid:**
- Excessive architecture diagrams (code is the truth)
- Theoretical design patterns without clear benefit
- Premature optimization documentation

### Mock Development Philosophy

**Simple, predictable mocks for fast development feedback** - avoid random behavior and complex simulation in favor of deterministic responses that enable reliable debugging.

> **For complete mock implementations and testing patterns**, see [`docs/SSH.md#testing--development`](SSH.md#testing--development)

### Repository Structure
For a detailed explanation of the directory layout and what each folder contains, **see [`docs/ARCHITECTURE.md`](ARCHITECTURE.md)**.

## UX Requirements

For UI/UX requirements and design specifications, see [`docs/DESIGN.md`](DESIGN.md).

## References

- [WebDriver | Tauri](https://v2.tauri.app/develop/tests/webdriver/) - Tauri WebDriver testing documentation
- [Prerequisites | Tauri](https://v2.tauri.app/start/prerequisites/) - Tauri development prerequisites
- [Visual comparisons | Playwright](https://playwright.dev/docs/test-snapshots) - Playwright screenshot testing
- [tauri-apps/webdriver-example](https://github.com/tauri-apps/webdriver-example) - Official Tauri WebDriver example
- [WebdriverIO | Tauri](https://v2.tauri.app/develop/tests/webdriver/example/webdriverio/) - WebdriverIO with Tauri guide
- [Playwright](https://playwright.dev/) - Web testing framework
- [Continuous Integration | Tauri](https://v2.tauri.app/develop/tests/webdriver/ci/) - Tauri CI setup for testing
- [How to Run Your Tests Headless with Xvfb](https://elementalselenium.com/tips/38-headless) - Xvfb configuration for CI

## Related Documentation

- [`docs/API.md`](API.md): IPC interfaces, command specifications, and API contracts.
- [`docs/DB.md`](DB.md): Database schemas, data validation, and persistence patterns.
- [`docs/reference/slurm-commands-reference.md`](reference/slurm-commands-reference.md): Job script template, template variable reference, critical requirements checklist.
- [`docs/reference/alpine-cluster-reference.md`](reference/alpine-cluster-reference.md): MPI best practices (node calculation, OpenMPI requirements), hardware partitions, QoS/resource limits, module loading sequences for Alpine.
- [`docs/reference/namd-commands-reference.md`](reference/namd-commands-reference.md): NAMD config file templates (.namd files), parameter validation, resource estimation, and related NAMD requirements.
