# Developer Guidelines

## Overview
This document establishes coding patterns and architectural principles for NAMDRunner development. These guidelines emerge from lessons learned during Phase 1 implementation and refactoring.

## Related Documentation
- [`svelte-patterns-guide.md`](svelte-patterns-guide.md) - Svelte-specific UI patterns
- [`testing-spec.md`](testing-spec.md) - Testing infrastructure and strategies
- [`technical-spec.md`](technical-spec.md) - Technology stack and setup
- [`.claude/agents/review-refactor.md`](../.claude/agents/review-refactor.md) - Code review agent using these guidelines

## Core Architectural Principles

### 1. Clean Architecture First
**Single Responsibility**: Each service, function, and module should have one clear purpose.

**No Thin Wrappers**: Avoid functions that just delegate to other functions without adding value.
```typescript
// ❌ Avoid thin wrappers
export const ServiceFactories = {
  createPathResolver(): PathResolver {
    const container = getServiceContainer();
    return container.get<PathResolver>('pathResolver');
  }
};

// ✅ Use services directly
const pathResolver = serviceContainer.get<PathResolver>('pathResolver');
```

**No Redundant Fallbacks**: Each operation should have one clear code path.
```typescript
// ❌ Avoid redundant fallbacks
function getJobPath(username: string, jobId: string): string {
  const result = pathResolver.getJobPath(username, jobId);
  if (!result.success) {
    console.warn('Path resolution failed, using fallback');
    return `/projects/${username}/jobs/${jobId}`; // Hardcoded fallback
  }
  return result.data;
}

// ✅ Proper error handling
function getJobPath(username: string, jobId: string): string {
  const result = pathResolver.getJobPath(username, jobId);
  if (!result.success) {
    throw new Error(`Failed to resolve job path: ${result.error.message}`);
  }
  return result.data;
}
```

### 2. Dependency Injection
**Constructor Injection**: All services should receive dependencies through constructor parameters.

```typescript
// ✅ Proper dependency injection
export class DirectoryManager {
  constructor(
    private sshConnection: SSHConnection,
    private sftpConnection: SFTPConnection,
    private pathResolver: PathResolver
  ) {}
}

// ✅ Use service container for composition
const container = createServiceContainer();
container.register('pathResolver', () => new PathResolver());
const directoryManager = container.get<DirectoryManager>('directoryManager');
```

**Mock Dependencies**: Tests should inject mock dependencies, not create their own service containers.
```typescript
// ✅ Explicit dependency injection in tests
describe('DirectoryManager', () => {
  let directoryManager: DirectoryManager;
  let mockSSH: MockSSHConnection;
  let pathResolver: PathResolver;

  beforeEach(() => {
    mockSSH = new MockSSHConnection();
    pathResolver = new PathResolver();
    directoryManager = new DirectoryManager(mockSSH, mockSFTP, pathResolver);
  });
});
```

### 3. Result<T> Error Handling
**Consistent Return Types**: All operations that can fail should return `Result<T>`.

```typescript
// ✅ Result pattern for error handling
async function validateConnection(config: ConnectionConfig): Promise<Result<ValidationResult>> {
  try {
    const result = await performValidation(config);
    return { success: true, data: result };
  } catch (error) {
    return { 
      success: false, 
      error: toConnectionError(error, 'Validation')
    };
  }
}
```

**Error Chaining**: Use utilities for complex operations.
```typescript
// ✅ Chain multiple Result operations (TypeScript)
async function setupWorkspace(username: string): Promise<Result<WorkspaceResult>> {
  const basePathResult = await pathResolver.getUserBasePath(username);
  return chainResults(basePathResult, async (basePath) => {
    return createWorkspaceStructure(basePath);
  });
}
```

```rust
// ✅ Error chaining with anyhow
use anyhow::{Result, Context};

impl ConnectionManager {
    pub async fn setup_job_workspace(&self, username: &str, job_id: &str) -> Result<WorkspaceInfo> {
        let project_dir = format!("/projects/{}/namdrunner_jobs/{}", username, job_id);

        // Chain operations with context for better error messages
        self.create_directory(&project_dir)
            .await
            .context("Failed to create project directory")?;

        self.create_directory(&format!("{}/inputs", project_dir))
            .await
            .context("Failed to create inputs directory")?;

        self.create_directory(&format!("{}/outputs", project_dir))
            .await
            .context("Failed to create outputs directory")?;

        Ok(WorkspaceInfo {
            project_dir,
            inputs_dir: format!("{}/inputs", project_dir),
            outputs_dir: format!("{}/outputs", project_dir),
        })
    }
}
```

**No Silent Failures**: Never suppress errors with console.warn().
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
    return { 
      success: false, 
      error: toConnectionError(error, 'Validation')
    };
  }
}
```

## Service Development Patterns

### 1. Service Layer Principles
**Single Responsibility**: Each service should have one clear purpose and handle one domain.

**Dependency Injection**: Services should receive dependencies through constructor parameters, not create them.

**Error Boundary**: Services should handle their own errors and return meaningful error types.

For complete SSH/Network operations implementation patterns, see [`docs/api-spec.md`](api-spec.md) SSH/Network Operations section.

### 2. Path Management
**Use PathResolver**: All path operations must go through the centralized PathResolver service.

```typescript
// ✅ Centralized path resolution
class JobService {
  constructor(private pathResolver: PathResolver) {}
  
  getJobPaths(username: string, jobId: string): Result<JobPaths> {
    return this.pathResolver.getJobPaths(username, jobId);
  }
}

// ❌ Direct path construction
function getJobDir(username: string, jobId: string): string {
  return `/projects/${username}/namdrunner_jobs/${jobId}`;
}
```

**Path Security**: Always validate paths for user isolation.
```typescript
// ✅ Path security validation
function validateJobAccess(path: string, username: string): boolean {
  return pathResolver.isPathAllowed(path, username);
}
```

### 2. State Management
**Observable Patterns**: Use state machines for complex state management.

```typescript
// ✅ State machine with observers
class ConnectionManager {
  private stateMachine: ConnectionStateMachine;
  
  constructor() {
    this.stateMachine = new ConnectionStateMachine();
    this.stateMachine.onStateChange((state) => {
      this.notifyUI(state);
    });
  }
}
```

**State Validation**: Always validate state transitions.
```typescript
// ✅ Validated state transitions
const transitionResult = stateMachine.transitionTo('Connected', 'Authentication successful');
if (!transitionResult.success) {
  throw new Error(`Invalid transition: ${transitionResult.error.message}`);
}
```

### 3. Session Management
**Security First**: Never persist credentials, clear memory on disconnect.

```typescript
// ✅ Secure session handling
class SessionManager {
  private currentSession: SessionInfo | null = null;
  
  async clearSession(): Promise<Result<void>> {
    this.currentSession = null;
    // Force garbage collection if available
    if (global.gc) {
      global.gc();
    }
    return { success: true, data: undefined };
  }
}
```

## Testing Principles

### Core Testing Principles
**Test Business Logic**: Focus on testing your code's logic, not language features or external libraries.

**Mock Dependencies**: Every external dependency should be mockable for isolated testing.

**Integration Testing**: Test how services work together, not just individual units.

**Avoid Anti-Patterns**: Don't test mock configurations, language features, or trivial assignments.

For complete testing strategies, anti-patterns, and implementation examples, see [`docs/testing-spec.md`](testing-spec.md).

## Feature Behavior Investigation Process
*Systematic methodology for validating feature completeness*

### 1. Question Expected Behavior First
**Don't Assume Implementation is Correct**: Always question how features *should* work before validating how they *do* work.

```typescript
// ✅ Question-driven investigation process
const investigationQuestions = [
  // Job Directory Management
  "When creating a job: Does it create project folders immediately or defer until submission?",
  "When submitting: Does it properly create scratch directories without duplicating project setup?",
  "When deleting: Does it clean up both project AND scratch directories?",
  "When syncing/reloading: Does it avoid recreating existing directories?",

  // Connection Lifecycle
  "When app starts: Does it properly restore 'Disconnected' state vs trying to resume old sessions?",
  "When connection expires: Does it gracefully handle mid-operation failures?",
  "When reconnecting: Does it properly validate the new session vs assuming it's still good?",

  // Error Handling
  "When network hiccups: Do we retry appropriately vs failing immediately?",
  "When commands timeout: Do we clean up SSH channels properly?",
  "When SLURM returns errors: Do we parse them into user-friendly messages?",
];
```

### 2. Trace Actual Code Flows
**Follow the Execution Path**: Trace through actual code to see what really happens vs what's intended.

```rust
// ✅ Systematic code flow investigation
async fn investigate_job_creation_flow() -> InvestigationResult {
    // 1. Trace job creation command
    let creation_flow = trace_command_flow("create_job", CreateJobParams {
        job_name: "test_job".to_string(),
        // ... other params
    }).await;

    // 2. Check what actually gets called
    let actual_calls = creation_flow.get_function_calls();

    // 3. Identify gaps between expected and actual
    let expected_calls = vec![
        "create_job_metadata",
        "create_project_directories",    // ❌ Missing!
        "setup_directory_structure",    // ❌ Missing!
        "store_job_in_database",
    ];

    let missing_calls = expected_calls.iter()
        .filter(|call| !actual_calls.contains(call))
        .collect();

    InvestigationResult {
        expected_behavior: "Job creation should create remote directories",
        actual_behavior: "Job creation only stores directory paths",
        gap_identified: !missing_calls.is_empty(),
        missing_implementations: missing_calls,
    }
}
```

### 3. Validate Interface vs Implementation
**Check That Interfaces Are Fully Implemented**: Ensure all defined interfaces have complete implementations.

```rust
// ✅ Interface completeness check
trait FileOperations {
    async fn upload_file(&self, local: &str, remote: &str) -> Result<()>;
    async fn download_file(&self, remote: &str, local: &str) -> Result<()>;
    async fn file_exists(&self, remote: &str) -> Result<bool>;        // ✅ Implemented
    async fn delete_file(&self, remote: &str) -> Result<()>;
}

impl FileOperations for RealSFTPService {
    async fn upload_file(&self, local: &str, remote: &str) -> Result<()> {
        // ❌ Investigation found: No existence check before upload!
        // ❌ Investigation found: No resume capability for partial uploads!
        self.sftp.create_file(remote).await?;
        self.sftp.transfer_bytes(local, remote).await
    }

    async fn file_exists(&self, remote: &str) -> Result<bool> {
        // ✅ Method exists and is implemented
        self.sftp.stat(remote).await.map(|_| true).or(Ok(false))
    }
}

// Investigation reveals gap: upload_file() doesn't use file_exists()!
```

### 4. Identify Integration Points
**Check That Tools Are Wired Into Workflows**: Building utilities isn't enough - they must be integrated.

```rust
// ✅ Integration point investigation
#[derive(Debug)]
struct IntegrationGap {
    tool_exists: bool,
    tool_works: bool,
    integrated_in_workflow: bool,
    gap_description: String,
}

fn investigate_directory_management() -> Vec<IntegrationGap> {
    vec![
        IntegrationGap {
            tool_exists: true,        // ✅ sftp_create_directory() exists
            tool_works: true,         // ✅ Function works when called
            integrated_in_workflow: false, // ❌ Job creation doesn't call it!
            gap_description: "Directory creation tool exists but job creation workflow doesn't use it".to_string(),
        },
        IntegrationGap {
            tool_exists: true,        // ✅ Connection retry logic config exists
            tool_works: false,       // ❌ No actual retry implementation
            integrated_in_workflow: false, // ❌ Error handling doesn't retry
            gap_description: "Retry configuration exists but no retry mechanism implemented".to_string(),
        },
    ]
}
```

### 5. Security and Input Validation Review
**Assume Malicious Input**: Test how the system handles dangerous inputs and edge cases.

```rust
// ✅ Security-focused investigation
#[test]
fn investigate_path_security() {
    let dangerous_inputs = vec![
        "../../../etc/passwd",          // Directory traversal
        "job; rm -rf /",               // Command injection
        "../../../../proc/version",     // System information leakage
        "job\x00hidden",               // Null byte injection
        "very_long_name".repeat(1000), // Buffer overflow attempt
    ];

    for input in dangerous_inputs {
        let result = create_job_with_name(&input);

        // Investigation questions:
        // - Does the system sanitize this input?
        // - Does it validate against path traversal?
        // - Does it prevent command injection?
        // - Does it handle edge cases gracefully?

        assert!(result.is_err(), "System should reject dangerous input: {}", input);
    }
}
```

### 6. Performance and Resource Investigation
**Check Resource Management**: Investigate memory usage, connection cleanup, and resource leaks.

```rust
// ✅ Resource investigation methodology
async fn investigate_connection_cleanup() -> ResourceInvestigation {
    let initial_connections = count_active_connections().await;

    // Perform operations that should clean up after themselves
    let _conn1 = connect_to_cluster().await?;
    let mid_connections = count_active_connections().await;

    disconnect().await?;
    let final_connections = count_active_connections().await;

    ResourceInvestigation {
        connections_created: mid_connections - initial_connections,
        connections_cleaned: mid_connections - final_connections,
        potential_leak: final_connections > initial_connections,
        investigation_notes: vec![
            "Check if SSH sessions are properly closed".to_string(),
            "Verify password memory is cleared".to_string(),
            "Ensure SFTP channels are released".to_string(),
        ],
    }
}
```

### 7. Document Findings and Create Action Items
**Convert Investigation into Roadmap**: Turn findings into concrete development tasks.

```markdown
## Investigation Results Summary

### Critical Issues Found (Must Fix)
- **Job Directory Management Gap**: Job creation stores paths but doesn't create directories
- **Missing Retry Logic**: Errors marked retryable but no retry implementation
- **Path Security Vulnerability**: No input sanitization in path construction

### Action Items
1. **Add to Milestone 2.2**: Implement missing directory creation in job lifecycle
2. **Add to Milestone 2.2**: Implement exponential backoff retry mechanism
3. **Add to Milestone 2.2**: Add path sanitization and validation utilities

### Testing Improvements Needed
- Remove anti-pattern tests that test language features
- Add business logic tests for retry mechanisms
- Add security tests for path sanitization
```

## Feature Completeness Validation
*Ensuring implementations meet requirements*

### 1. Lifecycle Implementation Checklist
**Complete CRUD Operations**: Don't just build individual functions, implement the complete workflow.

```rust
// ❌ Incomplete lifecycle - only creates paths, doesn't create actual directories
pub fn create_job(params: CreateJobParams) -> CreateJobResult {
    let job_info = JobInfo {
        project_dir: Some(format!("/projects/{}/jobs/{}", username, job_id)),
        scratch_dir: Some(format!("/scratch/{}/jobs/{}", username, job_id)),
        // ... other fields
    };
    // Missing: Actually create the directories on remote system!
}

// ✅ Complete lifecycle implementation
pub async fn create_job(params: CreateJobParams) -> CreateJobResult {
    // 1. Create job metadata
    let job_info = JobInfo { /* ... */ };

    // 2. Create actual directories on remote system
    let project_dir = format!("/projects/{}/jobs/{}", username, job_id);
    connection_manager.create_directory(&project_dir).await?;

    // 3. Set up directory structure
    connection_manager.create_directory(&format!("{}/inputs", project_dir)).await?;
    connection_manager.create_directory(&format!("{}/outputs", project_dir)).await?;

    // 4. Store job in database/state
    state.jobs.insert(job_id.clone(), job_info);

    CreateJobResult { success: true, job_id: Some(job_id) }
}
```

**Deletion Cleanup**: If you create resources, you must clean them up.
```rust
// ✅ Complete deletion with cleanup
pub async fn delete_job(job_id: String, delete_remote: bool) -> DeleteJobResult {
    // 1. Remove from local state
    let job_info = state.jobs.remove(&job_id)?;

    // 2. Clean up remote resources if requested
    if delete_remote {
        if let Some(project_dir) = job_info.project_dir {
            connection_manager.delete_directory_recursive(&project_dir).await?;
        }
        if let Some(scratch_dir) = job_info.scratch_dir {
            connection_manager.delete_directory_recursive(&scratch_dir).await?;
        }
    }

    DeleteJobResult { success: true }
}
```

### 2. Error Handling Completeness
**Implement Retry Logic**: If errors are marked `retryable`, implement actual retry mechanism.

```rust
// ❌ Incomplete - marks errors as retryable but no retry implementation
pub struct ConnectionError {
    pub retryable: bool,  // Flag exists...
    // ... other fields
}

// ✅ Complete - implement the retry logic
pub async fn connect_with_retry(params: ConnectParams) -> Result<ConnectionInfo> {
    let mut attempts = 0;
    let max_attempts = 3;
    let mut delay = Duration::from_millis(1000);

    loop {
        match connection_manager.connect(params.clone()).await {
            Ok(info) => return Ok(info),
            Err(e) => {
                let conn_error = map_ssh_error(&e);

                attempts += 1;
                if !conn_error.retryable || attempts >= max_attempts {
                    return Err(e);
                }

                // Exponential backoff
                tokio::time::sleep(delay).await;
                delay *= 2;
            }
        }
    }
}
```

### 3. Security Input Validation
**Sanitize All User Inputs**: Never trust user input in path construction or command execution.

```rust
// ❌ Dangerous - direct user input in path construction
pub fn create_job_directory(username: &str, job_name: &str) -> String {
    format!("/projects/{}/jobs/{}", username, job_name)
    // Risk: username="../../../etc" or job_name="important_file; rm -rf /"
}

// ✅ Secure - sanitize all inputs
pub fn create_job_directory(username: &str, job_name: &str) -> Result<String> {
    let clean_username = sanitize_path_component(username)?;
    let clean_job_name = sanitize_path_component(job_name)?;

    validate_no_traversal(&clean_username)?;
    validate_no_traversal(&clean_job_name)?;

    Ok(format!("/projects/{}/jobs/{}", clean_username, clean_job_name))
}

fn sanitize_path_component(input: &str) -> Result<String> {
    // Remove dangerous characters
    let cleaned = input
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect::<String>();

    if cleaned.is_empty() {
        return Err(anyhow::anyhow!("Invalid path component"));
    }

    Ok(cleaned)
}
```

### 4. Integration Points Verification
**Wire Tools Into Workflows**: Building utilities is not enough - they must be used in actual workflows.

```rust
// ❌ Tool exists but not integrated into workflow
// We have: sftp_create_directory() function
// But: Job creation doesn't call it!

// ✅ Tools integrated into workflows
pub async fn submit_job(job_id: &str) -> SubmitJobResult {
    // 1. Get job info
    let job_info = get_job_info(job_id)?;

    // 2. Create scratch directory (using our tool!)
    if let Some(scratch_dir) = &job_info.scratch_dir {
        connection_manager.create_directory(scratch_dir).await?;
    }

    // 3. Upload job files (using our SFTP tools!)
    for file in &job_info.input_files {
        sftp_upload_file(&file.local_path, &file.remote_path).await?;
    }

    // 4. Submit to SLURM
    let slurm_job_id = submit_to_slurm(&job_info).await?;

    SubmitJobResult { success: true, slurm_job_id: Some(slurm_job_id) }
}
```

## Code Quality Standards

### 1. TypeScript Usage
**Strict Types**: Use strict TypeScript configuration, avoid `any`.

```typescript
// ✅ Proper typing
interface JobCreationParams {
  username: string;
  jobName: string;
  namdConfig: NAMDConfig;
  slurmConfig: SLURMConfig;
}

function createJob(params: JobCreationParams): Promise<Result<JobInfo>> {
  // Implementation
}

// ❌ Avoid any types
function createJob(params: any): Promise<any> {
  // Avoid this
}
```

**Interface Segregation**: Create focused interfaces, not monolithic ones.
```typescript
// ✅ Focused interfaces
interface ConnectionManager {
  connect(params: ConnectParams): Promise<Result<SessionInfo>>;
  disconnect(): Promise<Result<void>>;
  getStatus(): ConnectionState;
}

interface SessionValidator {
  validateSession(session: SessionInfo): boolean;
  isExpired(session: SessionInfo): boolean;
}
```

### 2. Error Messages
**User-Friendly Messages**: Error messages should help users understand what went wrong and what to do.

```typescript
// ✅ Helpful error messages
ErrorBuilder.create({
  code: 'CONN_001',
  category: 'Network',
  message: 'Unable to connect to cluster',
  retryable: true,
  suggestions: [
    'Check your network connection',
    'Verify the cluster hostname is correct',
    'Ensure you are connected to the university VPN if required'
  ]
});
```

### 3. Documentation
**Self-Documenting Code**: Code should be clear enough to understand without extensive comments.

```typescript
// ✅ Clear, self-documenting code
async function setupUserWorkspace(username: string): Promise<Result<DirectorySetupResult>> {
  const basePathResult = await pathResolver.getUserBasePath(username);
  if (!basePathResult.success) {
    return basePathResult;
  }
  
  const workspaceResult = await createWorkspaceDirectories(basePathResult.data);
  return workspaceResult;
}
```

**Interface Documentation**: Document public interfaces and complex business logic.
```typescript
/**
 * Validates connection to SLURM cluster with comprehensive checks
 * Tests SSH connectivity, SFTP operations, module system, and SLURM access
 * 
 * @param config - Connection configuration with host, username, password
 * @returns Detailed validation result with recommendations for failures
 */
interface ConnectionValidator {
  runFullValidation(config: ConnectionConfig): Promise<Result<ValidationResult>>;
}
```

## Anti-Patterns to Avoid

### 1. False Backward Compatibility
**No Compatibility Claims**: Don't create "backward compatible" interfaces when no legacy code exists.

```typescript
// ❌ False backward compatibility
export const PathUtils = {
  getJobPath: (username: string, jobId: string) => {
    // "Backward compatible" wrapper that never had a forward compatibility
  }
};

// ✅ Direct interface usage
// Use PathResolver methods directly
```

### 2. Over-Engineering
**YAGNI Principle**: Don't create abstractions until you actually need them.

```typescript
// ❌ Over-engineered abstraction
interface ServiceFactory<T> {
  create(): T;
  createWithConfig(config: any): T;
  createSingleton(): T;
}

// ✅ Simple, direct approach
const pathResolver = new PathResolver();
```

### 3. Mixed Concerns
**Separation of Concerns**: Don't mix UI logic with business logic, or networking with data persistence.

```typescript
// ❌ Mixed concerns
class JobManager {
  async createJob(params: CreateJobParams): Promise<void> {
    // Business logic
    const job = new Job(params);
    
    // UI logic (shouldn't be here)
    showNotification('Job created!');
    
    // Network logic (should be separate)
    await fetch('/api/jobs', { method: 'POST', body: JSON.stringify(job) });
  }
}

// ✅ Separated concerns
class JobService {
  async createJob(params: CreateJobParams): Promise<Result<Job>> {
    // Only business logic
    return { success: true, data: new Job(params) };
  }
}
```

### 4. Feature Incompleteness Anti-Patterns
*Critical anti-patterns that lead to incomplete features*

**Building Tools Without Integration**: Don't create utilities that aren't wired into actual workflows.

```rust
// ❌ Anti-pattern - Tool exists but not integrated
// We have: sftp_create_directory() function
// But: Job creation workflow doesn't call it!

pub async fn create_job(params: CreateJobParams) -> CreateJobResult {
    let job_info = JobInfo {
        project_dir: Some(format!("/projects/{}/jobs/{}", username, job_id)),
        // Missing: Actually create the directories!
    };
}

// ✅ Properly integrated workflow
pub async fn create_job(params: CreateJobParams) -> CreateJobResult {
    let job_info = JobInfo { /* ... */ };

    // Actually use the tools we built
    let project_dir = format!("/projects/{}/jobs/{}", username, job_id);
    connection_manager.create_directory(&project_dir).await?;

    job_info
}
```

**Incomplete Error Handling**: Don't mark errors as retryable without implementing retry logic.

```rust
// ❌ Anti-pattern - Retryable flag without implementation
pub struct ConnectionError {
    pub retryable: bool,  // Flag exists but no retry logic!
}

// ✅ Complete implementation
pub async fn connect_with_retry(params: ConnectParams) -> Result<ConnectionInfo> {
    let mut attempts = 0;
    let max_attempts = 3;

    loop {
        match connection_manager.connect(params.clone()).await {
            Ok(info) => return Ok(info),
            Err(e) => {
                let conn_error = map_ssh_error(&e);
                attempts += 1;

                if !conn_error.retryable || attempts >= max_attempts {
                    return Err(e);
                }

                // Actually implement the retry logic
                tokio::time::sleep(Duration::from_millis(1000 * attempts)).await;
            }
        }
    }
}
```

**Security Assumptions**: Don't assume user input is safe without validation.

```rust
// ❌ Anti-pattern - Direct user input usage
pub fn create_job_path(username: &str, job_name: &str) -> String {
    format!("/projects/{}/jobs/{}", username, job_name)
    // Risk: username="../../../etc" or job_name="dangerous; rm -rf /"
}

// ✅ Always validate and sanitize
pub fn create_job_path(username: &str, job_name: &str) -> Result<String> {
    let clean_username = sanitize_path_component(username)?;
    let clean_job_name = sanitize_path_component(job_name)?;
    validate_no_traversal(&clean_username)?;
    validate_no_traversal(&clean_job_name)?;
    Ok(format!("/projects/{}/jobs/{}", clean_username, clean_job_name))
}
```

For Rust-specific development patterns including error chaining, memory management, module organization, async/blocking integration, and IPC type safety, see [`docs/technical-spec.md`](technical-spec.md) Backend section.

For security implementation guidelines including credential handling, path validation, input sanitization, and memory security, see [`docs/api-spec.md`](api-spec.md) Security Implementation Guidelines section.

## Performance Guidelines

### 1. Async Operations
**Non-Blocking**: All I/O operations must be async to avoid blocking the UI.

```typescript
// ✅ Async I/O operations
async function uploadFiles(files: FileUpload[]): Promise<Result<UploadResult[]>> {
  const uploadPromises = files.map(file => uploadSingleFile(file));
  const results = await Promise.all(uploadPromises);
  return collectResults(results);
}
```

### 2. Error Recovery
**Retry with Backoff**: Implement exponential backoff for network operations.

```typescript
// ✅ Retry with exponential backoff
async function connectWithRetry(config: ConnectionConfig): Promise<Result<SessionInfo>> {
  return retryWithResult(
    () => establishConnection(config),
    maxRetries: 3,
    baseDelay: 1000
  );
}
```

## Build Configuration Standards

### TypeScript Configuration
**Strict Mode Required**: Enable all strict type checking options for maximum safety.

```json
// tsconfig.json
{
  "compilerOptions": {
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "exactOptionalPropertyTypes": true,
    "noImplicitOverride": true
  }
}
```

**Linting and Formatting**:
- ESLint with `@typescript-eslint` parser
- Prettier for consistent formatting
- Svelte ESLint plugin for component linting
- Pre-commit hooks to enforce standards

### Rust Configuration
**Minimum Supported Rust Version (MSRV)**: Pin at project start for stability.

```toml
# Cargo.toml
[package]
rust-version = "1.70"  # Pin MSRV
```

**Quality Tools**:
- `clippy` with `-D warnings` flag (deny all warnings)
- `rustfmt` for consistent formatting
- `cargo-audit` for security vulnerability scanning
- `cargo-deny` for dependency hygiene
- All tools enforced in CI pipeline

**Error Handling**:
- One error type per module for clarity
- IPC boundaries return typed, user-friendly messages
- Use `thiserror` for error derivation
- Never expose internal error details to users

This guideline document should be treated as living documentation that evolves with the project. All new code should follow these patterns, and existing code should be refactored to match these standards when possible.