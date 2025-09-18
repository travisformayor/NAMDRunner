# Contributing to NAMDRunner

**How we expect you to write code and why** - This document covers development setup, coding standards, testing strategies, and best practices for contributing to NAMDRunner.

> **For system architecture and design principles**, see [`ARCHITECTURE.md`](ARCHITECTURE.md)
> **For SSH/SFTP connection patterns and security**, see [`SSH.md`](SSH.md)
> **For UI/UX components and design patterns**, see [`DESIGN.md`](DESIGN.md)
> **For database schemas and data management**, see [`DB.md`](DB.md)
> **For IPC interfaces and API contracts**, see [`API.md`](API.md)
> **For SLURM/NAMD command patterns**, see [`reference/`](reference/) directory

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
  - [Quick Start - Top 5 Critical Rules](#quick-start---top-5-critical-rules)
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
# Install Rust from https://rustup.rs (MSVC)
# Install Node.js LTS from https://nodejs.org
# Ensure Visual Studio Build Tools / Desktop C++ are present for native deps

git clone https://github.com/yourusername/namdrunner.git
cd namdrunner
npm install
```

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

# Full Tauri application
npm run tauri dev         # Run app with hot reload
npm run tauri build       # Build release binary
```


## VM Development Environment (Optional)

For developers using a Fedora VM environment (e.g., UTM on macOS).

### Platform
* **Fedora 38 ARM64** (UTM VM) for development
* **Workspace**: `/media/share/<repo-worktree>` mounted from the host (synced with host machine)

### Host Setup (Outside the VM)
1. **Port forwarding with socat**:
   ```bash
   socat TCP-LISTEN:2222,fork,reuseaddr TCP:<vm ip address>:22
   ```

2. **SSH config**:
   ```ssh
   Host fedora-vm
     HostName 127.0.0.1
     Port 2222
     User fedora
     IdentityFile ~/.ssh/utm_ed25519
   ```

## Host vs. VM Builds (Rust)

**Goals**

* Keep **sources** on the shared mount (e.g., `/media/share/<REPO>`).
* On **Fedora VM**: send heavy I/O to VM disk.
* On **macOS host**: use normal project-local folders.

### Rust (Cargo `target/`)

**Fedora VM (zsh) — add to `~/.zshrc`:**

```zsh
# Use VM-local disk for Cargo artifacts
export CARGO_TARGET_DIR="$HOME/.cargo-target/namdrunner"
export CARGO_INCREMENTAL=0
```

Setup folder once:

```bash
mkdir -p "$HOME/.cargo-target/namdrunner"
```

**macOS host:**
No configuration needed. Ensure `CARGO_TARGET_DIR` is **not** set on macOS so Cargo writes to `./target` in the repo.

**Verify**

```bash
# On VM: artifacts at ~/.cargo-target/namdrunner
ls -1 "$HOME/.cargo-target/namdrunner" | head

# On Mac: artifacts in ./target
test -d target && echo "macOS using ./target ✅"
```

## Developer Standards & Project Philosophy

### Quick Start - Top 5 Critical Rules

1. **No Thin Wrappers**: Don't create functions that just delegate to other functions
2. **Direct Error Handling**: Use `Result<T>` patterns, never suppress errors with `console.warn()`
3. **No Repository Pattern**: Use direct database calls with `with_database()`
4. **Security First**: Always sanitize user input, never log credentials
5. **Simple Mocks**: Predictable test behavior over complex simulation

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
**Avoid Thin Wrappers**: Functions should add value, not just delegate.
```typescript
// ❌ Thin wrapper
const ServiceFactories = {
  createPathResolver(): PathResolver {
    return container.get<PathResolver>('pathResolver');
  }
};

// ✅ Direct usage
const pathResolver = serviceContainer.get<PathResolver>('pathResolver');
```

**No Repository Pattern**: Use direct database calls.
```rust
// ❌ Repository wrapper
trait JobRepository {
    fn save_job(&self, job: &JobInfo) -> Result<()>;
}

// ✅ Direct database calls
with_database(|db| db.save_job(&job_info))
```

**No Intermediate Business Logic**: Call `execute_with_mode` directly in commands.
```rust
// ❌ Unnecessary wrapper
async fn create_job_business_logic(params: CreateJobParams) -> CreateJobResult {
    execute_with_mode(create_job_mock(params.clone()), create_job_real(params)).await
}

// ✅ Direct in command handler
#[tauri::command]
pub async fn create_job(params: CreateJobParams) -> CreateJobResult {
    execute_with_mode(create_job_mock(params.clone()), create_job_real(params)).await
}
```

### 2. Result<T> Error Handling
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
function sanitizeJobId(jobId: string): string {
  try {
    return pathResolver.sanitizeJobId(jobId);
  } catch (error) {
    console.warn('Sanitization failed, using fallback');
    return `job_${Date.now()}`;
  }
}

// ✅ Proper error handling
function sanitizeJobId(jobId: string): Result<string> {
  try {
    const sanitized = pathResolver.sanitizeJobId(jobId);
    return { success: true, data: sanitized };
  } catch (error) {
    return { success: false, error: toConnectionError(error, 'Validation') };
  }
}
```

## UI Development Patterns

### 1. Utility-First Component Design
**Centralized Utilities**: Create focused utility functions that serve multiple components.

```typescript
// ✅ Focused utility functions in utils/file-helpers.ts
export function getFileIcon(type: string): string { /* ... */ }
export function getTypeLabel(type: string): string { /* ... */ }
export function parseMemoryString(memory: string): number { /* ... */ }

// ✅ Use in components
import { getFileIcon, getTypeLabel } from '../../utils/file-helpers';
```

**Single Source of Truth**: Centralize configuration and data definitions.
```typescript
// ✅ Centralized in data/cluster-config.ts
export const PARTITIONS: PartitionSpec[] = [/* ... */];
export function validateResourceRequest(cores, memory, walltime, partition, qos) { /* ... */ }

// ❌ Duplicate definitions across components
const partitionLimits = { amilan: { maxCores: 64 } }; // In Component A
const limits = { amilan: { maxCores: 64 } }; // In Component B
```

### 2. Component Composition
**Reusable Components**: Create focused, composable components.

```svelte
<!-- ✅ FormField.svelte - Reusable form component -->
<script lang="ts">
  export let label: string;
  export let id: string;
  export let type: 'text' | 'number' | 'email' = 'text';
  export let value: string | number;
  export let error: string = '';
</script>

<div class="namd-field-group">
  <label class="namd-label" for={id}>{label}</label>
  <input class="namd-input" class:error {id} {type} bind:value />
  {#if error}<span class="namd-error-text">{error}</span>{/if}
</div>
```

**Reactive Data Flow**: Use Svelte's reactive statements with utility functions.
```svelte
<script lang="ts">
  import { validateResourceRequest, parseMemoryString } from '../../utils/helpers';

  export let cores: number;
  export let memory: string;

  // ✅ Reactive validation using utilities
  $: memoryGB = parseMemoryString(memory);
  $: validation = validateResourceRequest(cores, memoryGB, walltime, partition, qos);
</script>
```

### 3. Tab and Layout Systems
**Unified Tab System**: Use consistent tab styling across all implementations.

```svelte
<!-- ✅ Consistent tab pattern -->
<nav class="namd-tabs-nav namd-tabs-nav--grid namd-tabs-nav--grid-5">
  {#each tabs as tab}
    <button class="namd-tab-button" class:active={activeTab === tab.id}>
      {tab.label}
    </button>
  {/each}
</nav>
<div class="namd-tab-content">
  <div class="namd-tab-panel">
    <!-- Tab content -->
  </div>
</div>
```

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
```typescript
// ✅ Single source of truth
// In cluster-config.ts
export const PARTITIONS: PartitionSpec[] = [/* ... */];
export function validateResourceRequest(cores, memory, walltime, partition, qos) { /* ... */ }

// ❌ Scattered configuration
// In Component A: const limits = { amilan: { maxCores: 64 } };
// In Component B: const partitionLimits = { amilan: { maxCores: 64 } };
```

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

#### UI-Specific Anti-Patterns
**CSS Duplication**: Don't define the same styles in multiple components.

```svelte
<!-- ❌ Duplicate badge styles across components -->
<!-- Component A -->
<style>
  .status-badge { padding: 0.25rem 0.5rem; border-radius: 9999px; }
  .status-running { background-color: #dbeafe; color: #1d4ed8; }
</style>

<!-- Component B -->
<style>
  .status-indicator { padding: 0.25rem 0.5rem; border-radius: 9999px; }
  .running { background-color: #dbeafe; color: #1d4ed8; }
</style>

<!-- ✅ Use centralized classes -->
<span class="namd-status-badge namd-status-badge--running">Running</span>
```

**Hardcoded Styling**: Don't use hardcoded colors or Tailwind classes without the framework.
```svelte
<!-- ❌ Hardcoded styles -->
<div class="bg-blue-500 text-white px-4 py-2">Content</div>
<div style="background-color: #3b82f6; color: white;">Content</div>

<!-- ✅ Use CSS custom properties -->
<div class="namd-button namd-button--primary">Content</div>
```

**Over-Complex Component APIs**: Keep component interfaces focused and simple.
```svelte
<!-- ❌ Over-complex API -->
<FormField
  {label} {id} {type} {value} {placeholder} {required} {error}
  {min} {max} {step} {disabled} {readonly} {autocomplete}
  {validation} {transform} {formatter} {parser}
  onInput={handleInput} onBlur={handleBlur} onFocus={handleFocus}
/>

<!-- ✅ Focused, simple API -->
<FormField {label} {id} {type} {value} {error} {required} />
```

### Security Requirements

#### Core Security Principles
- **Secure credential handling**: Always use SecStr for passwords with automatic memory cleanup
- **No credential persistence**: Passwords exist only in memory during active sessions
- **Safe logging**: Never log credentials, passwords, or sensitive configuration
- **Input sanitization**: Validate and sanitize all user input before use
- **Path safety**: Prevent directory traversal and injection attacks

> **For complete security implementation patterns and examples**, see [`SSH.md#security-patterns`](SSH.md#security-patterns)

#### Path Security & Input Validation
**Essential Principles**:
- Never use user input directly in path construction or shell commands
- Always sanitize and validate input before use
- Prevent directory traversal attacks (`../`, null bytes, etc.)
- Use allow-lists for valid characters (alphanumeric, `_`, `-`)
- Validate path length limits and component restrictions

> **For complete input validation implementations**, see [`SSH.md#input-validation`](SSH.md#input-validation)

#### Command Injection Prevention
**Essential Principles**:
- Always escape shell parameters when executing remote commands
- Sanitize filenames and command arguments
- Use parameter validation before shell execution
- Never use user input directly in command construction

> **For complete command injection prevention patterns**, see [`SSH.md#security-patterns`](SSH.md#security-patterns)

#### Connection Lifecycle Management
**Essential Principles**:
- Always clean up connections properly
- Clear credentials from memory on disconnect
- Validate SSH connection before SLURM operations
- Handle connection expiration gracefully

> **For complete SSH connection lifecycle patterns**, see [`SSH.md#connection-management`](SSH.md#connection-management)

### 2. Dependency Injection
**Constructor Injection**: Services receive dependencies through constructor.
```typescript
export class DirectoryManager {
  constructor(
    private sshConnection: SSHConnection,
    private pathResolver: PathResolver
  ) {}
}
```

**Mock Dependencies in Tests**: Don't create service containers in tests.

> **For SSH mock patterns in tests**, see [`SSH.md#testing--development`](SSH.md#testing--development)

### 3. Path Management
Use PathResolver for all path operations. Never construct paths directly.

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

### Retry Logic Implementation
**Essential Principles**:
- Implement exponential backoff for retryable operations
- Distinguish between retryable and non-retryable errors
- Use appropriate timeout limits and maximum attempts
- Add jitter to prevent thundering herd effects

> **For complete retry implementations and patterns**, see [`SSH.md#retry-strategies`](SSH.md#retry-strategies)

### Error Mapping for User Experience
**Essential Principles**:
- Convert technical errors to actionable user messages
- Categorize errors by type (Network, Authentication, Permission, etc.)
- Provide recovery suggestions for each error category
- Maintain error context throughout the system

> **For complete error mapping patterns**, see [`SSH.md#error-handling`](SSH.md#error-handling)

### Async Operations with Blocking Libraries
**Essential Principles**:
- Use `spawn_blocking` for CPU-intensive blocking operations
- Avoid blocking the async runtime with synchronous operations
- Handle ssh2 and other blocking libraries properly
- Maintain async interface boundaries for UI responsiveness

> **For complete async/blocking integration patterns**, see [`SSH.md#async-patterns`](SSH.md#async-patterns)

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

### Rust Quality Tools
- `clippy` with `-D warnings` (deny all warnings)
- `rustfmt` for consistent formatting
- `cargo-audit` for security scanning

For platform support, build requirements, and system constraints, see [`ARCHITECTURE.md#architecture-principles--constraints`](ARCHITECTURE.md#architecture-principles--constraints).

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
- **Tier 1 (Frontend)**: TypeScript/Svelte unit tests - UI logic, stores, client-side validation
- **Tier 2 (Backend)**: Rust unit tests - command parsing, error mapping, path handling, credential management
- **Tier 3 (Integration)**: Full workflows via Playwright/WebdriverIO

### What We Test
✅ **Security validation** - malicious inputs, path traversal, credential safety
✅ **File path handling** - directory generation, safety checks
✅ **Command parsing** - SLURM output parsing, job state mapping
✅ **Error classification** - which errors are retryable vs fatal
✅ **User workflows** - complete job lifecycle (create → submit → delete)

### What We Don't Test
❌ **External crate functionality** - ssh2 connections, SFTP implementations
❌ **Mock performance** - testing how fast our mocks run
❌ **Implementation details** - internal state consistency
❌ **Infrastructure complexity** - no SSH test servers or integration environments

> **For SSH/SFTP testing patterns and mock infrastructure**, see [`SSH.md#testing--development`](SSH.md#testing--development)

### Testing Commands

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
npm test                # Vitest unit tests
npm run test:ui         # UI testing with Playwright

# Backend testing
cargo test              # Rust unit tests
cargo clippy            # Rust linting

# Full integration testing
npm run test:e2e        # Desktop E2E testing (Linux)
```

### Why This Works
Scientists need reliability over performance. A desktop app that safely handles credentials and prevents security vulnerabilities is more valuable than one optimized for millisecond performance differences.

### Mock Development Philosophy
**Core Principles**:
- **Simple and predictable**: Mocks should behave consistently, not randomly
- **Fast feedback**: Fixed delays and deterministic responses
- **Easy debugging**: Predictable behavior helps identify real issues
- **Development workflow**: Enable offline development without external dependencies

**Mock Guidelines**:
- Use environment-based switching (`USE_MOCK_SSH=true`)
- Keep mock responses simple and deterministic
- Provide comprehensive fixture data for testing
- Maintain separate mock implementations for each service

> **For complete mock implementations and patterns**, see [`SSH.md#testing--development`](SSH.md#testing--development)

### Repository Structure
```
/src/                     # Svelte + TS components (no Tauri in here)
/src/lib/ports/coreClient.ts
/src/lib/ports/coreClient-tauri.ts
/src/lib/ports/coreClient-mock.ts
/src/lib/domain/          # pure logic (parsers, mapping, validation)
/src/lib/fixtures/        # deterministic UI fixtures for tests
/src-tauri/               # Rust: ssh/sftp/slurm/sqlite/templating + commands
/tests/                   # Vitest + Svelte Testing Library
/tests/e2e/               # Linux WebDriver specs (real desktop)
/tests/ui/                # UI testing with agent debug toolkit
/ci/                      # workflows, scripts (xvfb, deps)
```

## UX Requirements

For UI/UX requirements and design specifications, see [DESIGN.md](DESIGN.md).

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

For IPC interfaces, command specifications, and API contracts, see [`API.md`](API.md).

For database schemas, data validation, and persistence patterns, see [`DB.md`](DB.md).

For SLURM command patterns and cluster-specific details, see [`reference/slurm-commands-reference.md`](reference/slurm-commands-reference.md) and [`reference/alpine-cluster-reference.md`](reference/alpine-cluster-reference.md).
