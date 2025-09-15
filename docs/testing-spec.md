# Testing and Debugging Guide for NAMDRunner

## Testing Strategy

### Overview
NAMDRunner uses a pragmatic testing approach focused on business logic validation while preserving fast development workflows. Testing strategy balances coverage with simplicity, avoiding complex test infrastructure.

## Dual-Purpose Testing Strategy

NAMDRunner uses **TWO DISTINCT** testing approaches, each serving dual purposes:

### 1. UI Testing (Playwright + Vite) - `tests/ui/`
**Purpose**: Fast UI development and debugging with mock backend  
**Target**: Web browser testing of Svelte UI via `http://localhost:1420`  
**Backend**: Mock client only - no Rust backend involved  
**Speed**: Very fast (no build required)  

**Dual Purpose Usage**:
- **Static Tests**: Fixed regression tests and TDD for UI components
- **Agent Investigation**: Ad-hoc debugging scripts for autonomous development

**Use when**:
- Daily UI development and iteration
- Agent-first debugging with visual feedback and screenshots
- Testing component logic, form validation, and UI workflows
- Quick "does this dialog work?" checks during development
- Debugging UI issues without Rust backend complexity

### 2. E2E Testing (WebdriverIO + tauri-driver) - `tests/e2e/`
**Purpose**: Complete desktop application testing with full stack  
**Target**: Built Tauri desktop binary with complete Rust backend  
**Backend**: Full TypeScript ↔ Rust IPC integration testing  
**Speed**: Slower (requires Tauri build, but debug logs provide good feedback)  

**Dual Purpose Usage**:
- **Static Tests**: Comprehensive integration test suites for release validation  
- **Agent Investigation**: Testing and debugging TypeScript ↔ Rust IPC boundary

**Use when**:
- Backend changes that affect IPC boundary
- Release validation and integration testing
- Testing complete user workflows end-to-end with real desktop app
- Verifying that TypeScript frontend properly communicates with Rust backend

**⚠️ Key Distinction**: UI tests target web browsers via Vite dev server. E2E tests target the built desktop application via WebDriver.

### 3-Tier Test Architecture

NAMDRunner uses a **3-tier test structure** with distinct purposes and execution environments:

#### Tier 1: Frontend Unit Tests (`src/lib/test/`)
- **Purpose**: TypeScript/Svelte frontend logic testing
- **Tool**: Vitest
- **Command**: `npm run test`
- **Scope**: Pure functions, stores, UI logic, client-side validation
- **Mock Level**: Mock external dependencies (no real HTTP/IPC calls)
- **Speed**: Very fast (< 1s)

#### Tier 2: Backend Unit Tests (`src-tauri/src/`)
- **Purpose**: Rust SSH service business logic testing
- **Tool**: Cargo test
- **Command**: `cargo test`  
- **Scope**: Command parsing, error mapping, file path handling, credential management
- **Mock Level**: Simple mocked ssh2 responses (return values only, no network operations)
- **Speed**: Fast (< 10s)
- **Focus**: Test our logic, not ssh2 crate functionality

#### Tier 3: Integration & E2E Tests (`tests/`)
- **Purpose**: Full system integration and user workflow testing
- **Tools**: Playwright (UI), WebdriverIO (E2E)
- **Commands**: `npm run test:ui`, `npm run test:e2e`
- **Scope**: User workflows, IPC boundary, complete desktop app testing
- **Mock Level**: Can use real or mock backends depending on test goals
- **Speed**: Medium (UI: ~30s) to Slow (E2E: ~2-5min)

### Testing Level Guidelines

#### When to Use Each Tier:
- **Tier 1**: Business logic, data transformations, UI state management, connection state logic
- **Tier 2**: SSH service business logic (command parsing, error mapping, path handling) 
- **Tier 3**: User workflows, IPC boundary testing, complete feature validation

#### SSH/SFTP Testing Strategy:
- **Tier 1**: Continue using existing mock client (`coreClient-mock.ts`) for UI testing
- **Tier 2**: Test SSH service logic with canned ssh2 responses, no real connections
- **Tier 3**: Manual validation - developers occasionally test against real clusters
- **No Test Servers**: Avoid complexity of SSH test infrastructure or integration environments

#### UI Testing Features (tests/ui/)
- **✅ Playwright** for rapid UI testing via Vite dev server
- **✅ Agent debug toolkit** with autonomous screenshots and error monitoring
- **✅ Xvfb support** for headless environments and CI
- **✅ Console logs, JS errors, and network failure monitoring**
- **✅ Fast iteration** - no build step required
- **Target**: Web UI at `http://localhost:1420` with mock backend (preserves fast development workflow)

#### E2E Testing Features (tests/e2e/)
- **✅ WebdriverIO + tauri-driver** for desktop application automation
- **✅ WebKitWebDriver** for native Linux WebDriver support
- **✅ Comprehensive debug logging** with session tracking
- **✅ Automatic Tauri build** as part of test setup
- **✅ Full IPC boundary testing** between TypeScript and Rust
- **Target**: Built Tauri desktop binary with complete backend integration

### Testing Configuration

#### Vitest Setup
```json
// vitest.config.js
export default {
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['src/lib/fixtures/setup.ts']
  }
}
```

#### Rust Test Configuration
```rust
// In Cargo.toml
[dev-dependencies]
tokio = { version = "1", features = ["test-util"] }
tempfile = "3"
```

#### UI Testing Setup (Playwright + Vite) - `tests/ui/`
```bash
# Quick start for UI development and agent debugging
Xvfb :99 -screen 0 1280x720x24 &  # Virtual display for SSH/CI environments
export DISPLAY=:99
npm run dev                        # Start Vite dev server (localhost:1420)
npm run test:ui                    # Agent debugging toolkit with screenshots
```

#### E2E Testing Setup (WebdriverIO + tauri-driver) - `tests/e2e/`
```bash
# Prerequisites (one-time setup)
cargo install tauri-driver --locked     # WebDriver for Tauri apps
which WebKitWebDriver                   # Verify WebKit driver exists

# Run E2E tests (automatically builds Tauri binary)
export DISPLAY=:99                      # Virtual display if needed
npm run test:e2e                        # Complete desktop app testing
```

**Key Implementation Notes**:
- E2E tests automatically build the Tauri binary before running
- WebdriverIO capabilities are simplified to work with tauri-driver
- Debug logging provides detailed session information for troubleshooting

## Mock Data Patterns

### SLURM Command Responses
Based on proven Python implementation patterns:

```rust
// Mock squeue output
const MOCK_SQUEUE_RUNNING: &str = "12345678|test_job|R|00:15:30|01:44:30|1|24|16GB|amilan|/scratch/alpine/testuser/namdrunner_jobs/test_job";

// Mock sacct output  
const MOCK_SACCT_COMPLETED: &str = "12345678|test_job|COMPLETED|0:0|2025-01-15T10:00:00|2025-01-15T11:00:00|01:00:00|/scratch/alpine/testuser/namdrunner_jobs/test_job";

// Mock sbatch response
const MOCK_SBATCH_SUCCESS: &str = "Submitted batch job 12345678";
```

### Job Lifecycle States
- **CREATED** - Job exists locally but not submitted
- **PENDING** - Submitted to SLURM, waiting for resources  
- **RUNNING** - Executing on cluster
- **COMPLETED** - Finished successfully
- **FAILED** - Job failed with error
- **CANCELLED** - User or system cancelled job

### Mock Implementation Pattern
```rust
// Trait abstraction for testing
trait SlurmClient {
    async fn submit_job(&self, script_path: &str) -> Result<String, SlurmError>;
    async fn get_job_status(&self, job_id: &str) -> Result<JobStatus, SlurmError>;
    async fn get_running_jobs(&self, username: &str) -> Result<Vec<JobInfo>, SlurmError>;
}

// Production implementation
struct RemoteSlurmClient { /* real SSH connection */ }

// Mock implementation for testing
struct MockSlurmClient { /* fixture responses */ }
```

## Development Testing Workflow

### Daily Testing Commands

#### Tier 1: Frontend Unit Tests (Run Frequently)
```bash
npm run test            # Vitest for TypeScript/Svelte components
                       # Fast feedback on business logic and UI state
```

#### Tier 2: Backend Unit Tests (Run Less Frequently)
```bash
cargo test              # Rust SSH service business logic tests
                       # Tests command parsing, error mapping, path handling
                       # Uses simple mocked ssh2 responses, no real connections
```

#### Tier 3: Integration Testing (As Needed)
```bash
# Fast UI development & agent debugging
export DISPLAY=:99      # Virtual display for SSH environments
npm run dev             # Start Vite dev server (required for UI testing)
npm run test:ui         # Playwright agent debugging toolkit

# Full desktop application testing (slower)
npm run test:e2e        # WebdriverIO + tauri-driver (auto-builds Tauri app)
```

#### Complete Test Suite (CI/Release)
```bash
# Run all three tiers
npm run test && cargo test && npm run test:ui && npm run test:e2e
```

#### Code Quality (Daily)
```bash
npm run lint            # ESLint + Prettier
cargo clippy            # Rust linting
```

**✅ Current Implementation Notes**: 
- UI testing is very fast - no build step required
- E2E testing automatically builds Tauri binary with comprehensive debug logging
- Both approaches provide detailed debugging output for troubleshooting

### Development Mode Setup
```bash
# Frontend development with mocks (default)
npm run dev                    # Uses mock client automatically

# Production testing with real SSH (manual)
# Set environment variable to use real SSH service
export NAMDRUNNER_PRODUCTION=true
npm run tauri dev             # Uses real SSH service via Rust

# Verify which client is being used
grep -r "clientFactory" src/lib/ports/
```

### Phase 2 SSH Testing Strategy

#### Mock Client Preservation (Phase 1 Foundation)
- **Frontend development**: Continue using `coreClient-mock.ts` for fast UI iteration
- **TypeScript unit tests**: All existing tests use mocks, no changes needed  
- **Agent debugging**: UI testing toolkit remains fully functional with mocks
- **Development workflow**: `npm run dev` continues to work offline with mock backend

#### Rust SSH Service Testing (Phase 2 Addition)
- **Business logic focus**: Test command parsing, error mapping, file path handling
- **Simple mocks**: Return canned ssh2 responses, no real network operations
- **No test servers**: Avoid SSH test infrastructure complexity
- **Manual validation**: Developers occasionally test against real clusters

#### Dual Implementation Support
- **Client factory**: `clientFactory.ts` chooses mock vs real based on environment
- **Same interfaces**: Both implementations use identical TypeScript contracts
- **Independent testing**: Mock and real implementations tested separately
- **Good enough coverage**: Test our logic, not ssh2 crate functionality

## CI Testing Pipeline

### Linux CI Job
1. Run unit tests (Vitest + Cargo)
2. Run fast UI tests with agent debug toolkit
3. Build Tauri application (`--debug` for testing)
4. Install `tauri-driver` and WebKit prerequisites
5. Run WebdriverIO E2E tests under Xvfb
6. Upload screenshots, test results, and logs as artifacts

### Windows CI Job  
1. Build portable `.exe` file
2. Publish as release artifact
3. No desktop E2E testing required

### macOS Support
- Manual smoke builds supported
- No automated desktop E2E on macOS

## Debugging Guide

### Build and Development Issues

#### Tauri Build Problems
```bash
# Clean everything and rebuild
cargo clean
rm -rf node_modules package-lock.json target/
npm ci

# Check Rust toolchain
rustup show
rustc --version
cargo --version

# Verbose build output
npm run tauri dev -- --verbose

# Check Tauri prerequisites
npx tauri info
```

#### TypeScript/Svelte Issues
```bash
# Check TypeScript compilation
npm run check

# Svelte language server issues
rm -rf .svelte-kit/
npm run dev

# Check ESLint configuration
npm run lint -- --debug
```

#### Rust Compilation Issues
```bash
# Verbose output
RUST_BACKTRACE=1 cargo build --verbose

# Check clippy warnings
cargo clippy -- -D warnings

# Format code
cargo fmt --check
```

### Application Debugging

#### Tauri IPC Debugging
```typescript
// Frontend debugging
console.log('Calling Tauri command:', commandName, args);
try {
  const result = await invoke(commandName, args);
  console.log('Tauri command result:', result);
} catch (error) {
  console.error('Tauri command error:', error);
}
```

```rust
// Backend command debugging
use log::{debug, info, warn, error};

#[tauri::command]
async fn connect_to_cluster(
    host: String,
    username: String,
    password: String
) -> Result<String, String> {
    debug!("Attempting to connect to: {}", host);
    
    // Your implementation here
    
    info!("Successfully connected to cluster");
    Ok("Connected".to_string())
}

// Enable logging in main.rs
fn main() {
    env_logger::init();
    
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            connect_to_cluster
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### SSH and Network Debugging

#### SSH Connection Issues
```bash
# Test SSH connection manually
ssh -v username@login.rc.colorado.edu

# Network connectivity
ping login.rc.colorado.edu
nslookup login.rc.colorado.edu
```

#### SLURM Command Debugging
```bash
# Test SLURM commands manually
ssh username@login.rc.colorado.edu "module load slurm/alpine && squeue -u $USER"

# Check module availability
ssh username@login.rc.colorado.edu "module avail slurm"

# Debug SLURM job submission
ssh username@login.rc.colorado.edu "cd /scratch/alpine/$USER && sbatch --test-only job.sbatch"
```

### Database Debugging

#### Rust SQLite Debugging
```rust
use log::debug;

// In your database operations
debug!("Executing SQL: {}", sql);
match conn.execute(sql, params) {
    Ok(changes) => {
        debug!("SQL executed successfully, {} rows affected", changes);
        Ok(changes)
    }
    Err(e) => {
        error!("SQL execution failed: {}", e);
        Err(e)
    }
}
```

## Error Handling Strategy

### Error Categories
NAMDRunner uses a structured error handling approach with specific categories:

```typescript
interface NAMDRunnerError {
  category: 'Network' | 'Authentication' | 'Validation' | 'FileSystem' | 'SLURM' | 'Internal';
  message: string;
  details?: string;
  retryable: boolean;
}
```

### Error Category Examples

#### Network Errors
```typescript
const NETWORK_ERROR: NAMDRunnerError = {
  category: 'Network',
  message: 'Failed to connect to cluster',
  details: 'Connection timed out after 30 seconds',
  retryable: true
};
```

#### Authentication Errors
```typescript
const AUTH_ERROR: NAMDRunnerError = {
  category: 'Authentication',
  message: 'SSH authentication failed',
  details: 'Invalid username or password',
  retryable: true
};
```

#### Validation Errors
```typescript
const VALIDATION_ERROR: NAMDRunnerError = {
  category: 'Validation', 
  message: 'Invalid NAMD parameters',
  details: 'Temperature must be between 200 and 400 Kelvin',
  retryable: false
};
```

#### SLURM Errors
```typescript
const SLURM_ERROR: NAMDRunnerError = {
  category: 'SLURM',
  message: 'Job submission failed',
  details: 'Invalid partition name specified',
  retryable: false
};
```

#### FileSystem Errors
```typescript
const FILESYSTEM_ERROR: NAMDRunnerError = {
  category: 'FileSystem',
  message: 'Failed to upload input files',
  details: 'Permission denied accessing remote directory',
  retryable: true
};
```

#### Internal Errors
```typescript
const INTERNAL_ERROR: NAMDRunnerError = {
  category: 'Internal',
  message: 'Database operation failed',
  details: 'SQLite connection timeout',
  retryable: true
};
```

## Error Message Reference

### Common Tauri Errors
```
Error: "failed to resolve component"
Solution: Check component import paths and Svelte configuration

Error: "command not found"  
Solution: Verify command is registered in main.rs invoke_handler

Error: "failed to serialize/deserialize"
Solution: Check that all IPC types implement Serialize/Deserialize
```

### Common Rust Errors
```
Error: "borrowed value does not live long enough"
Solution: Review lifetime annotations and ownership

Error: "cannot borrow as mutable"
Solution: Check borrow checker rules, consider RefCell or Mutex

Error: "failed to connect" (SSH)
Solution: Check network connectivity and credentials
```

### Common SLURM Errors
```
Error: "Invalid partition name"
Solution: Check available partitions with "sinfo"

Error: "sbatch: error: Batch job submission failed: Invalid partition name specified"
Solution: Check partition name, use "sinfo" to list available partitions

Error: "sbatch: error: Batch job submission failed: Access denied"
Solution: Check user permissions and authentication

Error: "sbatch: error: Batch job submission failed: Requested node configuration is not available"
Solution: Reduce resource requirements or check cluster availability

Error: "ssh: connect to host login.rc.colorado.edu port 22: Connection timed out"
Solution: Check network connectivity to cluster

Error: "Permission denied (publickey,password)"
Solution: Verify username and password for SSH authentication
```

### SLURM Error Recovery Patterns
```rust
// Example error handling in Rust
match submit_job(&job_script).await {
    Ok(job_id) => Ok(job_id),
    Err(e) if e.contains("Invalid partition") => {
        Err(NAMDRunnerError {
            category: "SLURM",
            message: "Invalid partition specified",
            details: "Use 'sinfo' to check available partitions",
            retryable: false,
        })
    },
    Err(e) if e.contains("Connection timed out") => {
        Err(NAMDRunnerError {
            category: "Network",
            message: "Network connection failed",
            details: e,
            retryable: true,
        })
    },
    Err(e) => {
        Err(NAMDRunnerError {
            category: "Internal",
            message: "Unexpected error",
            details: e,
            retryable: false,
        })
    }
}
```

## Testing Best Practices

### From Python Implementation Lessons
**What Worked Well:**
- Mock mode enabled rapid development
- Comprehensive unit tests caught regressions
- UI-as-Data pattern was reliable and fast
- Module isolation made testing easier

**What Could Be Improved:**
- More integration testing with real cluster
- Error injection testing
- User workflow validation

**Avoid These Patterns:**
- Don't skip mocking external dependencies
- Don't rely only on manual testing
- Don't test UI and business logic together
- Don't ignore error paths in tests

### Tauri-Specific Guidelines
- Prefer accessible roles/names or stable `data-testid`s over brittle CSS selectors
- Use wait-for-condition patterns instead of fixed sleeps
- Keep golden screenshots in repo; changes require human review
- Test IPC boundaries thoroughly with both success and failure cases

## Logging Configuration

### Development Logging Setup
```rust
// In main.rs
use log::LevelFilter;

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Debug)
        .init();
        
    // Rest of main function
}
```

## Step-by-Step Bug Investigation
1. **Reproduce the issue** with minimal steps
2. **Check logs** for error messages
3. **Verify environment** (network, permissions, dependencies)
4. **Isolate the problem** (frontend vs backend vs network)
5. **Test with mock data** to eliminate external factors
6. **Add debug logging** to narrow down the issue
7. **Create minimal test case** to verify fix

## Testing Anti-Patterns to Avoid
*Critical anti-patterns that waste development time*

### Don't Test Language Features
Avoid testing obvious programming language capabilities.

```rust
// ❌ Anti-pattern - testing language features
#[test]
fn test_string_operations() {
    let test_string = "hello world";
    assert_eq!(test_string.len(), 11); // Tests String::len(), not our code
}

#[test]
fn test_struct_field_assignment() {
    let info = RemoteFileInfo {
        name: "test.txt".to_string(),
        size: 1024,
    };
    assert_eq!(info.name, "test.txt"); // Tests struct assignment, not our logic
}

// ✅ Test business logic instead
#[test]
fn test_file_size_formatting() {
    let formatter = FileSizeFormatter::new();
    assert_eq!(formatter.format_bytes(1024), "1.0 KB");
    assert_eq!(formatter.format_bytes(1048576), "1.0 MB");
}
```

### Don't Test Mock Return Values
Avoid testing that mocks return what you configured them to return.

```rust
// ❌ Anti-pattern - testing mock configuration
#[test]
fn test_mock_returns_configured_value() {
    let mut mock_ssh = MockSSHManager::new();
    mock_ssh.expect_execute_command()
        .returning(|_| Ok(CommandResult { stdout: "success".to_string(), .. }));

    let result = mock_ssh.execute_command("test").unwrap();
    assert_eq!(result.stdout, "success"); // Just tests mock setup
}

// ✅ Test how your code uses the mock
#[test]
fn test_command_output_parsing() {
    let mut mock_ssh = MockSSHManager::new();
    mock_ssh.expect_execute_command()
        .returning(|_| Ok(CommandResult {
            stdout: "Submitted batch job 12345678".to_string(),
            exit_code: 0,
            ..
        }));

    let service = SlurmService::new(mock_ssh);
    let job_id = service.submit_job(&job_config).unwrap();
    assert_eq!(job_id, "12345678"); // Tests our parsing logic
}
```

### Focus on Business Logic
Test the logic you wrote, not external library behavior.

```rust
// ✅ Business logic testing examples
#[test]
fn test_error_categorization() {
    let ssh_error = SSHError::NetworkError("Connection refused".to_string());
    let categorized = map_ssh_error(&ssh_error);

    assert_eq!(categorized.category, "Network");
    assert!(categorized.retryable);
    assert!(categorized.suggestions.contains(&"Check your network connection".to_string()));
}

#[test]
fn test_path_sanitization() {
    assert_eq!(sanitize_job_id("valid_job_123"), Ok("valid_job_123".to_string()));
    assert!(sanitize_job_id("../../../etc").is_err());
    assert!(sanitize_job_id("job; rm -rf /").is_err());
}

#[test]
fn test_retry_logic_respects_max_attempts() {
    let mut attempt_count = 0;
    let result = retry_with_backoff(3, Duration::from_millis(1), || {
        attempt_count += 1;
        Err(anyhow::anyhow!("Network error"))
    });

    assert!(result.is_err());
    assert_eq!(attempt_count, 3);
}
```

### Network/SSH Testing Strategy
Never make real network calls in unit tests.

```rust
// ✅ SSH testing with mocks
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssh::test_utils::MockConnectionManager;

    #[tokio::test]
    async fn test_directory_creation_workflow() {
        let mut mock_manager = MockConnectionManager::new();

        // Test the sequence of calls our code makes
        mock_manager.expect_create_directory()
            .with(eq("/projects/testuser/namdrunner_jobs/job_001"))
            .times(1)
            .returning(|_| Ok(()));

        mock_manager.expect_create_directory()
            .with(eq("/projects/testuser/namdrunner_jobs/job_001/inputs"))
            .times(1)
            .returning(|_| Ok(()));

        let service = JobService::new(mock_manager);
        let result = service.setup_job_workspace("testuser", "job_001").await;

        assert!(result.is_ok());
        // Mock manager automatically verifies all expectations were met
    }
}
```

## Feature Behavior Investigation Process
*Systematic methodology for validating feature completeness*

### Question Expected Behavior First
Always question how features *should* work before validating implementation.

**Key Questions:**
- Does the feature handle the complete lifecycle (create → use → cleanup)?
- Are error cases properly handled with appropriate retry logic?
- Do interfaces match their actual implementations?

### Trace and Validate Implementation
Follow code execution paths to identify gaps between intended and actual behavior.

```rust
// ✅ Validate implementation completeness
trait FileOperations {
    async fn upload_file(&self, local: &str, remote: &str) -> Result<()>;
    async fn file_exists(&self, remote: &str) -> Result<bool>;
}

// Check: Does upload_file use file_exists for optimization?
// Check: Are all trait methods actually implemented?
```

### Check Integration and Security
Verify tools are integrated and validate input handling.

```rust
// ✅ Verify complete integration
pub async fn submit_job(job_id: &str) -> SubmitJobResult {
    if let Some(scratch_dir) = &job_info.scratch_dir {
        connection_manager.create_directory(scratch_dir).await?;
    }
}

#[test]
fn test_dangerous_inputs() {
    let inputs = vec!["../../../etc", "job; rm -rf /", "job\x00hidden"];
    for input in inputs {
        assert!(create_job_with_name(&input).is_err());
    }
}
```

### Document Findings
Turn investigation results into actionable roadmap items.

```markdown
### Typical Issues Found
- **Missing Lifecycle Steps**: Features store state but don't perform required actions
- **Retry Logic Gaps**: Errors marked retryable without implementation
- **Input Validation Missing**: No sanitization for user-provided data
```

## Related Documentation

For architectural principles and service development patterns, see [`docs/developer-guidelines.md`](developer-guidelines.md).

For SSH/Network operations and security implementation details, see [`docs/api-spec.md`](api-spec.md).

For Rust development patterns and backend configuration, see [`docs/technical-spec.md`](technical-spec.md).