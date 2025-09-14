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
// ✅ Chain multiple Result operations
async function setupWorkspace(username: string): Promise<Result<WorkspaceResult>> {
  const basePathResult = await pathResolver.getUserBasePath(username);
  return chainResults(basePathResult, async (basePath) => {
    return createWorkspaceStructure(basePath);
  });
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

### 1. Path Management
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

## Testing Patterns

### 1. Unit Testing
**Mock Dependencies**: Every dependency should be mockable.

```typescript
// ✅ Comprehensive mocking
describe('DirectoryManager', () => {
  let mockSSH: jest.Mock<SSHConnection>;
  let mockSFTP: jest.Mock<SFTPConnection>;
  let mockPathResolver: jest.Mock<PathResolver>;
  
  beforeEach(() => {
    mockSSH = createMockSSH();
    mockSFTP = createMockSFTP();
    mockPathResolver = createMockPathResolver();
  });
});
```

**Test Real Behavior**: Don't just test mocks, test actual service behavior.
```typescript
// ✅ Test actual behavior
it('should create job directory structure', async () => {
  const result = await directoryManager.setupJobWorkspace('testuser', 'job_001');
  
  expect(result.success).toBe(true);
  expect(result.data.jobDir).toBe('/projects/testuser/namdrunner_jobs/job_001');
  expect(mockSFTP.createDirectory).toHaveBeenCalledWith(
    '/projects/testuser/namdrunner_jobs/job_001'
  );
});
```

### 2. Integration Testing
**Service Interaction**: Test how services work together.

```typescript
// ✅ Integration test
describe('Service Integration', () => {
  it('should handle full job creation flow', async () => {
    const container = createMockServiceContainer();
    const jobService = container.get<JobService>('jobService');
    
    const result = await jobService.createAndSetupJob({
      username: 'testuser',
      jobName: 'test_job',
      namdConfig: mockConfig
    });
    
    expect(result.success).toBe(true);
    // Verify all services were called correctly
  });
});
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

## Security Guidelines

### 1. Credential Handling
**Never Log Credentials**: Passwords and sensitive data must never appear in logs.

```typescript
// ✅ Safe logging
logger.info('Connecting to cluster', { 
  host: config.host, 
  username: config.username 
  // Never log password
});

// ❌ Dangerous logging
logger.info('Connection attempt', { config }); // May contain password
```

**Memory Cleanup**: Clear sensitive data from memory when done.
```typescript
// ✅ Secure cleanup
class ConnectionManager {
  async disconnect(): Promise<void> {
    this.credentials = null;
    if (global.gc) {
      global.gc();
    }
  }
}
```

### 2. Path Security
**Validate All Paths**: Never trust user-provided paths without validation.

```typescript
// ✅ Path validation
function accessFile(username: string, filePath: string): Result<void> {
  if (!pathResolver.isPathAllowed(filePath, username)) {
    return {
      success: false,
      error: ErrorBuilder.create({
        code: 'SEC_001',
        category: 'Permission',
        message: 'Access denied: Path is outside user directory'
      })
    };
  }
  // Proceed with file access
}
```

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