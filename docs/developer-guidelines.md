# Developer Guidelines

## Overview
This document establishes coding patterns and architectural principles for NAMDRunner development. These guidelines emerge from lessons learned during Phase 1 implementation and refactoring.

## Related Documentation
- [`svelte-patterns-guide.md`](svelte-patterns-guide.md) - Svelte-specific UI patterns
- [`testing-spec.md`](testing-spec.md) - Testing infrastructure and strategies
- [`technical-spec.md`](technical-spec.md) - Technology stack and setup
- [`.claude/agents/review-refactor.md`](../.claude/agents/review-refactor.md) - Code review agent using these guidelines

## Core Architectural Principles

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

### 2. Clean Architecture First
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

### 2. CSS Design System
**Consistent Naming**: Use `namd-*` prefix for all custom CSS classes.

```css
/* ✅ Consistent naming in app.css */
.namd-button { /* base styles */ }
.namd-button--outline { /* variant */ }
.namd-status-badge { /* component */ }
.namd-status-badge--running { /* state */ }
.namd-file-type-badge { /* component */ }
.namd-file-type-structure { /* variant */ }
```

**Centralized Styling**: Define reusable styles in `app.css`, not component files.
```svelte
<!-- ✅ Use centralized classes -->
<span class="namd-status-badge namd-status-badge--{statusClass}">
  {status}
</span>

<!-- ❌ Component-specific styles -->
<style>
  .status-badge { /* duplicate styles */ }
</style>
```

### 3. Component Composition
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

### 4. Tab and Layout Systems
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

### 2. TypeScript Usage
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

### 1. UI Duplication and Inconsistency
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

### 2. False Backward Compatibility
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
**Separation of Concerns**: Keep UI, business logic, and data operations separate.

```typescript
// ❌ Mixed concerns in component
class JobManager {
  async createJob(params: CreateJobParams): Promise<void> {
    const job = new Job(params); // Business logic
    showNotification('Job created!'); // UI logic
    await fetch('/api/jobs', { method: 'POST', body: JSON.stringify(job) }); // Network
  }
}

// ✅ Separated concerns
// Business logic in services
class JobService {
  async createJob(params: CreateJobParams): Promise<Result<Job>> {
    return { success: true, data: new Job(params) };
  }
}

// UI logic in components
// Network logic in separate API layer
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

**Note**: Detailed build configuration is documented in [`technical-spec.md`](technical-spec.md). Key quality requirements:

- **TypeScript**: Strict mode enabled, no `any` types
- **Linting**: ESLint + Prettier with pre-commit hooks
- **Rust**: Clippy with warnings denied, security auditing enabled
- **Error Handling**: User-friendly messages, no internal details exposed

## Summary: Key Development Principles

1. **Progressive Enhancement**: Start simple, add complexity only when proven necessary
2. **Single Source of Truth**: Centralize configuration, utilities, and styling
3. **Utility-First Design**: Create focused, reusable functions and components
4. **Consistent Patterns**: Use unified naming conventions and design system
5. **Clean Separation**: Keep UI, business logic, and data operations separate
6. **Security First**: Never log credentials, validate all paths, clear sensitive data

This guideline document should be treated as living documentation that evolves with the project. All new code should follow these patterns, and existing code should be refactored to match these standards when possible.

**Remember**: We're building a focused tool for scientists, not an enterprise platform. Keep it simple, reliable, and maintainable.