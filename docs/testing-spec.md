# Testing Specification - NAMDRunner

## NAMDRunner Testing Philosophy

### Core Principle: Business Logic Focus
Test our logic, not external libraries. Focus on what NAMDRunner does, not how ssh2 or other crates work.

### 3-Tier Architecture
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

### Why This Works
Scientists need reliability over performance. A desktop app that safely handles credentials and prevents security vulnerabilities is more valuable than one optimized for millisecond performance differences.

**Key Insight**: We're building a business application, not a systems library. Test the business rules that keep scientists' data and credentials safe.

## Test Commands

### Daily Development
```bash
npm run test            # Frontend unit tests (Vitest)
cargo test              # Backend unit tests (Rust)
npm run test:ui         # UI testing with Playwright
```

### Complete Validation
```bash
npm run test && cargo test && npm run test:ui && npm run test:e2e
```

## Testing Patterns

### Security Validation (Critical)
Test that malicious inputs are rejected:

```rust
#[test]
fn test_malicious_input_rejection() {
    let dangerous_inputs = vec![
        "../../../etc/passwd",      // Directory traversal
        "job; rm -rf /",           // Command injection
        "job\x00hidden",           // Null byte injection
    ];

    for input in dangerous_inputs {
        assert!(input::sanitize_job_id(input).is_err());
    }
}
```

### SSH Operation Mocking
Never make real network calls. Use mocks for all SSH/SFTP operations:

```rust
#[tokio::test]
async fn test_job_submission_workflow() {
    env::set_var("USE_MOCK_SSH", "true");

    let mock_ssh = MockSSHManager::new();
    mock_ssh.expect_execute_command()
        .returning(|_| Ok(CommandResult {
            stdout: "Submitted batch job 12345678".to_string(),
            exit_code: 0,
        }));

    let result = submit_job_with_ssh(mock_ssh, "test_job").await;
    assert!(result.is_ok());
}
```

### Frontend State Testing
Test UI logic and state management with mocks:

```typescript
describe('Session Store', () => {
  beforeEach(() => {
    CoreClientFactory.reset();
    const mockClient = CoreClientFactory.getClient(true) as MockCoreClient;
    mockClient.enableErrorInjection(false);
  });

  it('should handle successful connection', async () => {
    const success = await sessionActions.connect('test.host', 'testuser', 'testpass');
    expect(success).toBe(true);
    expect(get(connectionState)).toBe('Connected');
  });
});
```

## Error Classification Testing

Test that errors are properly categorized for retry logic:

```rust
#[test]
fn test_error_categorization() {
    let network_error = SSHError::ConnectionFailed("timeout".to_string());
    let categorized = classify_error(&network_error);

    assert_eq!(categorized.category, ErrorCategory::Network);
    assert!(categorized.retryable);
}
```

## Path Safety Testing

Ensure directory paths are safe and don't allow traversal:

```rust
#[test]
fn test_path_generation_security() {
    let result = paths::project_directory("../admin", "job_001");
    assert!(result.is_err(), "Should reject path traversal attempts");

    let valid_result = paths::project_directory("testuser", "job_001");
    assert_eq!(valid_result.unwrap(), "/projects/testuser/namdrunner_jobs/job_001");
}
```

## Anti-Patterns to Avoid

### Don't Test Language Features
```rust
// ❌ Don't do this - tests String::len(), not our code
#[test]
fn test_string_length() {
    assert_eq!("hello".len(), 5);
}

// ✅ Do this - tests our business logic
#[test]
fn test_job_name_validation() {
    assert!(validate_job_name("hello").is_ok());
    assert!(validate_job_name("").is_err());
}
```

### Don't Test Mock Return Values
```rust
// ❌ Don't do this - tests mock setup, not our code
#[test]
fn test_mock_returns_configured_value() {
    let mock = MockSSH::new();
    mock.expect_command().returning(|| "success");
    assert_eq!(mock.command(), "success");
}

// ✅ Do this - tests how our code uses the mock
#[test]
fn test_command_output_parsing() {
    let mock = MockSSH::new();
    mock.expect_command().returning(|| "Submitted batch job 12345678");

    let job_id = parse_sbatch_output(mock.command()).unwrap();
    assert_eq!(job_id, "12345678");
}
```

### Don't Test External Libraries
```rust
// ❌ Don't test ssh2 crate functionality
#[test]
fn test_ssh2_connection() {
    let session = ssh2::Session::new().unwrap();
    // Testing ssh2 library behavior
}

// ✅ Test our SSH service logic
#[test]
fn test_connection_retry_logic() {
    let service = SSHService::new(MockSSH::failing());
    let result = service.connect_with_retry("host", "user", "pass");
    // Testing our retry implementation
}
```

## Test Environment Setup

All tests should use mocks and avoid real network operations:

```rust
fn setup_test_environment() {
    env::set_var("USE_MOCK_SSH", "true");
}
```

```typescript
beforeEach(() => {
    CoreClientFactory.reset();
    const mockClient = CoreClientFactory.getClient(true);
    mockClient.resetToCleanState();
});
```

## References

- **Testing Infrastructure**: See `docs/agent-capabilities.md` for detailed testing tools and mock data systems
- **CI/Testing Pipeline**: See `docs/agent-capabilities.md` for complete test execution workflows
- **Development Commands**: See `docs/technical-spec.md` for full command reference

## Success Criteria

1. **Security tests pass** - malicious input is rejected
2. **Business logic tests pass** - our parsing and validation works
3. **No real network calls** - all external operations use mocks
4. **Fast execution** - tests complete quickly for development workflow

Focus on reliability and security over performance benchmarks. Test what keeps scientists' credentials and data safe.