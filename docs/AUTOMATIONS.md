# NAMDRunner Automation Architecture

**How to build automation workflows for job management and cluster operations.**

> See project README for project overview.
>
> **Related Docs:**
>
> - [ARCHITECTURE.md](ARCHITECTURE.md) - System architecture
> - [API.md](API.md) - IPC interfaces

An automation is an async function that orchestrates multiple steps with progress tracking:

```rust
pub async fn execute_job_creation_with_progress(
    _app_handle: AppHandle,
    params: CreateJobParams,
    progress_callback: impl Fn(&str),
) -> Result<(String, JobInfo)> {
    progress_callback("Starting job creation...");

    // Discrete, verifiable steps
    progress_callback("Validating connection...");
    validate_connection()?;

    progress_callback("Creating directories...");
    create_directories()?;

    progress_callback("Job creation completed");
    Ok((job_id, job_info))
}
```

**Core Principles:**

- **Simple async functions** - No complex state machines or actors
- **Progress callbacks** - Real-time UI updates via closures
- **Discrete steps** - Each step is verifiable and atomic
- **Result-based errors** - Standard Rust error propagation

## Building an Automation Chain

### 1. Define the Function Signature

```rust
// Pattern: Execute + Operation + WithProgress
pub async fn execute_job_submission_with_progress(
    app_handle: AppHandle,
    params: SubmitJobParams,
    progress_callback: impl Fn(&str),
) -> Result<JobInfo> {
    // Implementation
}
```

**Naming Convention:**

- `execute_*_with_progress` for automation functions
- Params struct for typed inputs
- Progress callback for UI updates
- Returns `Result<T>` for error handling

### 2. Structure the Steps

Break complex operations into discrete, verifiable steps:

```rust
pub async fn execute_job_creation_with_progress(
    app_handle: AppHandle,
    params: CreateJobParams,
    progress_callback: impl Fn(&str),
) -> Result<(String, JobInfo)> {
    // Step 1: Validate inputs
    progress_callback("Validating inputs...");
    let validated = validate_params(params)?;

    // Step 2: Create resources
    progress_callback("Creating directories...");
    create_directories(&validated)?;

    // Step 3: Upload files (if any)
    if !validated.files.is_empty() {
        for (i, file) in validated.files.iter().enumerate() {
            progress_callback(&format!("Uploading file {} of {}", i+1, validated.files.len()));
            upload_file(file)?;
        }
    }

    // Step 4: Save metadata
    progress_callback("Saving to database...");
    let job_info = save_job(validated)?;

    progress_callback("Complete");
    Ok((job_id, job_info))
}
```

**Step Guidelines:**

- Each step emits progress before executing
- Steps should be atomic (all-or-nothing)
- Use `?` operator for early returns on errors
- Return meaningful data structures

### 3. Integrate with Commands

Wire automation to Tauri commands using event emission:

```rust
#[tauri::command(rename_all = "snake_case")]
pub async fn create_job(
    app_handle: tauri::AppHandle,
    params: CreateJobParams
) -> ApiResult<JobInfo> {
    let handle_clone = app_handle.clone();

    match automations::execute_job_creation_with_progress(
        app_handle,
        params,
        move |msg| {
            let _ = handle_clone.emit("job-creation-progress", msg);
        }
    ).await {
        Ok((_job_id, job_info)) => ApiResult::success(job_info),
        Err(e) => ApiResult::error(e.to_string()),
    }
}
```

**Pattern:**

- Clone AppHandle for closure
- Pass closure that emits events
- Convert Result to ApiResult
- Use specific event names per automation

## Progress Tracking Pattern

### Backend: Emit Events

Use the progress callback to emit messages:

```rust
progress_callback("Starting operation...");
progress_callback("Step 1 of 3: Validating...");
progress_callback("Step 2 of 3: Processing...");
progress_callback("Step 3 of 3: Saving...");
progress_callback("Operation complete");
```

**Message Guidelines:**

- Start with action verb ("Creating...", "Uploading...", "Validating...")
- Use present continuous tense for in-progress operations
- Include counts for loops ("File 3 of 10")
- Final message confirms completion

### Frontend: Listen for Events

```typescript
// In Svelte store
interface JobProgress {
    message: string;
    isActive: boolean;
}

// Subscribe to progress events
const unlisten = await listen('job-creation-progress', (event) => {
    const message = event.payload as string;
    update(state => ({
        ...state,
        creationProgress: { message, isActive: true }
    }));
});

// Clean up listener when done
onDestroy(() => { unlisten(); });
```

**Event Names:**

- `job-creation-progress`
- `job-submission-progress`
- `job-deletion-progress`
- Use kebab-case, be specific

## Verification and Error Handling

### Input Validation

Always validate inputs before operations:

```rust
// Sanitize user inputs
let clean_name = crate::security::input::sanitize_job_id(&params.job_name)?;

// Validate connection
let username = connection_manager.get_username()
    .map_err(|_| "SSH connection not active")?;

// Validate paths
validate_path_safety(&path)?;
```

### Error Propagation

Use `?` operator for clean error propagation:

```rust
pub async fn execute_operation(params: Params) -> Result<Output> {
    // Fails fast with ? operator
    let validated = validate_inputs(params)?;
    let resource = create_resource(&validated)?;
    let result = process_resource(resource)?;

    Ok(result)
}
```

**Error Handling:**

- Return `Result<T>` from all fallible operations
- Use `?` to propagate errors up the stack
- Provide context with `.map_err(|e| format!("Context: {}", e))`
- Log errors with `log_error!()` macro

### Atomic Operations

Ensure operations are all-or-nothing:

```rust
// BAD: Partial state on failure
create_directory()?;
upload_file()?;  // Fails - directory orphaned!

// GOOD: Rollback on failure
let dir = create_directory()?;
match upload_file() {
    Ok(_) => { /* success */ }
    Err(e) => {
        delete_directory(&dir)?;  // Rollback
        return Err(e);
    }
}
```

## Common Helpers

### Connection Management

```rust
use crate::state::connection_manager;

// Get connection manager
let conn_mgr = connection_manager();

// Verify connection active
let username = conn_mgr.get_username()
    .map_err(|_| "Connection not active")?;

// Upload files
conn_mgr.upload_file(&local_path, &remote_path).await?;

// Execute commands
let output = conn_mgr.execute_command("squeue").await?;
```

### Database Operations

```rust
use crate::db::with_database;

// Read from database
let job = with_database(|conn| {
    jobs::get_job_by_id(conn, &job_id)
})?;

// Write to database
with_database(|conn| {
    jobs::save_job(conn, &job_info)
})?;
```

### Path Construction

```rust
use crate::ssh::paths::construct_remote_job_path;

// Build safe remote paths
let project_dir = construct_remote_job_path(&username, &job_id, false)?;
let scratch_dir = construct_remote_job_path(&username, &job_id, true)?;

// Paths automatically validated for safety
```

### SLURM Operations

```rust
use crate::slurm::SlurmStatusSync;

// Query job status
let statuses = SlurmStatusSync::sync_all_jobs(&slurm_job_ids).await?;

// Cancel job
SlurmStatusSync::cancel_job(&slurm_job_id).await?;
```

### Logging

```rust
use crate::logging::{log_info, log_error, log_warn};

// Log with optional toast
log_info!("Job created: {}", job_id);
log_error!("Failed to upload file: {}", err, show_toast: true);
log_warn!("Connection slow, retrying...");
```

## Connection Failure Detection

Automations automatically detect connection failures through error pattern matching:

```rust
// Backend returns standard errors
Err(e) if e.to_string().contains("connection") => {
    return ApiResult::error("Connection failed".to_string());
}
```

**Frontend Pattern:**

```typescript
// Frontend detects connection errors
const result = await invoke('sync_jobs');
if (result.success === false) {
    if (isConnectionError(result.error)) {
        connectionStore.setState('expired');
        // Disable all actions until reconnect
    }
}

function isConnectionError(error: string): boolean {
    return error.toLowerCase().includes('connection') ||
           error.toLowerCase().includes('timeout') ||
           error.toLowerCase().includes('authentication');
}
```

**No Extra Network Calls:**

- Connection failures detected from operation errors
- No dedicated health check commands
- Frontend transitions state automatically
- User prompted to reconnect

## Module Structure

```
src-tauri/src/automations/
├── mod.rs                   # Module exports
├── job_creation.rs         # Template-based job creation
├── job_submission.rs       # SLURM submission
├── job_sync.rs             # Status synchronization
├── job_completion.rs       # Results retrieval
├── job_deletion.rs         # Cleanup operations
└── common.rs               # Shared helpers
```

## Key Automation Chains

1. **Job Creation** - `job_creation.rs`
   - Template loading and validation
   - Directory creation
   - File uploads
   - NAMD config generation
   - Metadata persistence

2. **Job Submission** - `job_submission.rs`
   - Directory mirroring (project → scratch)
   - SLURM sbatch execution
   - Status updates

3. **Status Sync** - `job_sync.rs`
   - Batch SLURM queries
   - Job discovery from cluster
   - Automatic completion triggering

4. **Job Completion** - `job_completion.rs`
   - Results mirroring (scratch → project)
   - Log caching
   - Metadata finalization

5. **Job Deletion** - `job_deletion.rs`
   - SLURM cancellation
   - Directory cleanup
   - Database removal

## Testing Automations

Follow NAMDRunner's 3-tier testing strategy:

**Unit Tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validation_step() {
        let params = create_test_params();
        let result = validate_inputs(params);
        assert!(result.is_ok());
    }
}
```

**Integration Tests:**

- Mock ConnectionManager
- Test complete automation flow
- Verify progress callbacks
- Check error handling

**E2E Tests:**

- Real automation execution
- Verify side effects (DB, filesystem)
- Test failure scenarios

## Security Considerations

All automations must follow security principles:

- **Input Sanitization** - Use `security::input` helpers
- **Path Safety** - Validate all paths with `validate_path_safety()`
- **No Credential Logging** - Never log passwords or keys
- **Command Injection Prevention** - Sanitize shell inputs
- **Audit Trail** - Log all automation actions
