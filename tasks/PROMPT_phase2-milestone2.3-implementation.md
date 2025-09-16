# Phase 2.3 Implementation Prompt for Next Engineer

## Overview
You are implementing **Milestone 2.3: Job Status Synchronization & Data Persistence**. This milestone builds job status tracking and SLURM integration on top of the solid directory management foundation from Phase 2.2.

**Key Context**: Phase 2.2 delivered complete job lifecycle directory management with security validation and retry logic. Now we need jobs to persist across sessions and stay synchronized with SLURM queue state.

## What You're Building

### The Core Functionality
The current implementation creates and manages job directories but has no persistence or status tracking:
- ✅ Can create jobs with project directories
- ✅ Can submit jobs with scratch directories
- ✅ Can delete jobs with safe cleanup
- ❌ **But jobs disappear when app closes**
- ❌ **But no status tracking (PENDING → RUNNING → COMPLETED)**
- ❌ **But no integration with SLURM queue state**

### Why This Matters
Without status synchronization and persistence:
- Scientists lose track of submitted jobs when closing the application
- No visibility into job progression through SLURM queue states
- No way to recover job information or check completion status
- Critical for actual scientific workflow where jobs run for hours/days

## Implementation Strategy

### 1. Start with Documentation Review (Critical!)
Before writing any code, read the comprehensive command references we have:

**MUST READ FIRST:**
- **`docs/reference/slurm-commands-reference.md`** - Your primary SLURM integration guide
  - Contains exact squeue, sacct, scancel command syntax for Alpine cluster
  - Provides output format examples and parsing patterns
  - Includes error handling strategies and mock data for testing
  - Shows proven module loading sequences and execution patterns

- **`docs/reference/namd-commands-reference.md`** - Essential for job lifecycle understanding
  - Resource allocation patterns by system size
  - File organization standards for job directories
  - Error detection patterns in NAMD logs

- **`docs/data-spec.md`** - Critical for database schema design
  - Complete SQLite schema for job persistence
  - JobInfo struct definition and validation rules
  - JSON metadata formats you'll need to persist

**Then investigate existing code patterns:**
```bash
# Check existing SLURM command infrastructure
cd /media/share/namdrunner-backend/src-tauri
rg "squeue\|sacct\|scancel" src/ -A 5 -B 5

# Look at existing job status parsing
rg "parse.*sbatch\|JobStatus" src/ -A 10

# Check how job lifecycle currently works
rg "create_job\|submit_job\|delete_job" src/commands/jobs.rs -A 15
```

**Expected Finding**: SLURM command helpers exist but aren't integrated with status tracking.

### 2. Database-First Implementation Order
Build persistence foundation before status sync complexity:

#### Step 1: SQLite Database Setup (Do This First!)
```rust
// src-tauri/Cargo.toml - Add dependency
[dependencies]
rusqlite = { version = "0.29", features = ["bundled"] }

// src-tauri/src/database/mod.rs
use rusqlite::{Connection, Result};

pub struct JobDatabase {
    conn: Connection,
}

impl JobDatabase {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        Self::initialize_schema(&conn)?;
        Ok(Self { conn })
    }

    fn initialize_schema(conn: &Connection) -> Result<()> {
        conn.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS jobs (
                job_id TEXT PRIMARY KEY,
                job_name TEXT NOT NULL,
                slurm_job_id TEXT,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT,
                submitted_at TEXT,
                completed_at TEXT,
                project_dir TEXT,
                scratch_dir TEXT,
                error_info TEXT
            );

            CREATE TABLE IF NOT EXISTS job_status_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                job_id TEXT NOT NULL,
                status TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                source TEXT NOT NULL,
                FOREIGN KEY (job_id) REFERENCES jobs (job_id)
            );
        "#)?;
        Ok(())
    }
}
```

#### Step 2: Extend JobInfo with Database Persistence
```rust
// src-tauri/src/types/core.rs - Enhance existing JobInfo
impl JobInfo {
    pub fn save_to_db(&self, db: &JobDatabase) -> Result<()> {
        // Insert or update job in database
    }

    pub fn load_from_db(job_id: &str, db: &JobDatabase) -> Result<Option<Self>> {
        // Load job from database by ID
    }

    pub fn update_status(&mut self, new_status: JobStatus, db: &JobDatabase) -> Result<()> {
        // Update status and save to database with history
    }
}
```

#### Step 3: SLURM Status Integration (Build on Existing SSH)
**Use the exact patterns from `docs/reference/slurm-commands-reference.md`:**

```rust
// src-tauri/src/slurm/status.rs
use crate::connection_utils::ConnectionUtils;

pub struct SlurmStatusSync {
    connection_utils: ConnectionUtils,
}

impl SlurmStatusSync {
    pub async fn sync_job_status(&self, slurm_job_id: &str) -> Result<JobStatus> {
        // Use exact command pattern from slurm-commands-reference.md
        let full_cmd = format!(
            "source /etc/profile && module load slurm/alpine && squeue -j {} --format='%T' --noheader",
            slurm_job_id
        );

        let result = ConnectionUtils::execute_command_with_retry(&full_cmd).await?;

        // If not in active queue, check completed jobs with sacct
        if result.stdout.trim().is_empty() {
            return self.check_completed_job(slurm_job_id).await;
        }

        Self::parse_slurm_status(&result.stdout)
    }

    async fn check_completed_job(&self, slurm_job_id: &str) -> Result<JobStatus> {
        // Use sacct pattern from reference docs
        let sacct_cmd = format!(
            "source /etc/profile && module load slurm/alpine && sacct -j {} --format=State --parsable2 --noheader",
            slurm_job_id
        );

        let result = ConnectionUtils::execute_command_with_retry(&sacct_cmd).await?;
        Self::parse_slurm_status(&result.stdout)
    }

    fn parse_slurm_status(output: &str) -> Result<JobStatus> {
        // Handle all SLURM states from reference documentation
        match output.trim() {
            "PD" | "PENDING" => Ok(JobStatus::Pending),
            "R" | "RUNNING" => Ok(JobStatus::Running),
            "CG" | "COMPLETING" => Ok(JobStatus::Running), // Still running
            "CD" | "COMPLETED" => Ok(JobStatus::Completed),
            "F" | "FAILED" => Ok(JobStatus::Failed),
            "CA" | "CANCELLED" => Ok(JobStatus::Failed),
            "TO" | "TIMEOUT" => Ok(JobStatus::Failed),
            "NF" | "NODE_FAIL" => Ok(JobStatus::Failed),
            "PR" | "PREEMPTED" => Ok(JobStatus::Failed),
            _ => Err(anyhow!("Unknown SLURM status: {}", output))
        }
    }
}
```

#### Step 4: Integrate with Existing Job Commands (Enhance, Don't Replace)
```rust
// src-tauri/src/commands/jobs.rs - Enhance existing functions
pub async fn create_job(params: CreateJobParams) -> CreateJobResult {
    // EXISTING: Input validation and directory creation (Phase 2.2)

    // NEW: Save to database
    let job_info = JobInfo { /* ... existing fields ... */ };
    if let Err(e) = job_info.save_to_db(&get_database()).await {
        return CreateJobResult {
            success: false,
            error: Some(format!("Failed to save job: {}", e)),
            ..Default::default()
        };
    }

    // EXISTING: Return success result
}

// NEW: Add status sync command
#[tauri::command]
pub async fn sync_job_status(job_id: String) -> SyncJobStatusResult {
    // Use existing validation from Phase 2.2
    let clean_job_id = match input::sanitize_job_id(&job_id) {
        Ok(id) => id,
        Err(e) => return SyncJobStatusResult::error(format!("Invalid job ID: {}", e)),
    };

    // Get current status from SLURM and update database
    let sync = SlurmStatusSync::new();
    match sync.sync_job_status(&clean_job_id).await {
        Ok(status) => {
            // Update database and return new status
            SyncJobStatusResult::success(status)
        }
        Err(e) => SyncJobStatusResult::error(format!("Sync failed: {}", e))
    }
}
```

### 3. Testing Strategy (Build on Phase 2.2 Patterns)

#### Database Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_job_database_operations() {
        let temp_db = NamedTempFile::new().unwrap();
        let db = JobDatabase::new(temp_db.path().to_str().unwrap()).unwrap();

        let job = JobInfo {
            job_id: "test_job_001".to_string(),
            status: JobStatus::Created,
            // ... other fields
        };

        // Test save and load
        assert!(job.save_to_db(&db).is_ok());
        let loaded = JobInfo::load_from_db("test_job_001", &db).unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().job_id, "test_job_001");
    }
}
```

#### SLURM Status Parsing Testing (Use Reference Documentation)
**Test with all status codes from `docs/reference/slurm-commands-reference.md`:**
```rust
#[test]
fn test_slurm_status_parsing() {
    // Test all documented SLURM status codes
    assert_eq!(SlurmStatusSync::parse_slurm_status("PD").unwrap(), JobStatus::Pending);
    assert_eq!(SlurmStatusSync::parse_slurm_status("PENDING").unwrap(), JobStatus::Pending);
    assert_eq!(SlurmStatusSync::parse_slurm_status("R").unwrap(), JobStatus::Running);
    assert_eq!(SlurmStatusSync::parse_slurm_status("RUNNING").unwrap(), JobStatus::Running);
    assert_eq!(SlurmStatusSync::parse_slurm_status("CG").unwrap(), JobStatus::Running);
    assert_eq!(SlurmStatusSync::parse_slurm_status("CD").unwrap(), JobStatus::Completed);
    assert_eq!(SlurmStatusSync::parse_slurm_status("COMPLETED").unwrap(), JobStatus::Completed);
    assert_eq!(SlurmStatusSync::parse_slurm_status("F").unwrap(), JobStatus::Failed);
    assert_eq!(SlurmStatusSync::parse_slurm_status("FAILED").unwrap(), JobStatus::Failed);
    assert_eq!(SlurmStatusSync::parse_slurm_status("CA").unwrap(), JobStatus::Failed);
    assert_eq!(SlurmStatusSync::parse_slurm_status("TO").unwrap(), JobStatus::Failed);
    assert_eq!(SlurmStatusSync::parse_slurm_status("NF").unwrap(), JobStatus::Failed);
    assert_eq!(SlurmStatusSync::parse_slurm_status("PR").unwrap(), JobStatus::Failed);

    // Test error cases
    assert!(SlurmStatusSync::parse_slurm_status("UNKNOWN").is_err());
    assert!(SlurmStatusSync::parse_slurm_status("").is_err());
}

#[test]
fn test_slurm_command_construction() {
    // Test that commands match reference documentation exactly
    let sync = SlurmStatusSync::new();

    // Test squeue command format
    let expected_squeue = "source /etc/profile && module load slurm/alpine && squeue -j 12345678 --format='%T' --noheader";
    // Verify your command construction matches this pattern

    // Test sacct command format
    let expected_sacct = "source /etc/profile && module load slurm/alpine && sacct -j 12345678 --format=State --parsable2 --noheader";
    // Verify your command construction matches this pattern
}
```

## Key Files to Modify

### Primary Implementation Files
1. **`src-tauri/src/database/mod.rs`** (NEW) - SQLite database integration
2. **`src-tauri/src/slurm/status.rs`** (NEW) - SLURM status synchronization
3. **`src-tauri/src/commands/jobs.rs`** (ENHANCE) - Add database persistence to existing commands
4. **`src-tauri/src/types/core.rs`** (ENHANCE) - Extend JobInfo with database methods

### Integration Files
1. **`src-tauri/src/lib.rs`** - Add database initialization and SLURM status commands
2. **`src-tauri/Cargo.toml`** - Add rusqlite dependency
3. **`src-tauri/src/commands/mod.rs`** - Export new sync commands

## Common Pitfalls to Avoid

### 1. Don't Rebuild What Works
```rust
// ❌ Don't rewrite job creation - enhance it
pub async fn create_job_with_database(params: CreateJobParams) -> CreateJobResult {
    // Reimplementing directory creation, validation, etc.
}

// ✅ Enhance existing function
pub async fn create_job(params: CreateJobParams) -> CreateJobResult {
    // EXISTING: validation, directory creation (keep this!)

    // NEW: just add database save
    job_info.save_to_db(&database).await?;
}
```

### 2. Don't Ignore Existing Security Patterns
```rust
// ❌ Direct job ID usage
let status = sync_job_status(job_id).await;

// ✅ Use existing validation
let clean_job_id = input::sanitize_job_id(&job_id)?;
let status = sync_job_status(&clean_job_id).await;
```

### 3. Don't Skip Error Classification
```rust
// ❌ Generic error handling
if let Err(e) = sync_status() {
    return Err(e);
}

// ✅ Use existing retry patterns for appropriate errors
match sync_status().await {
    Err(e) if e.is_retryable() => {
        // Use existing retry logic
        ConnectionUtils::retry_operation(|| sync_status()).await
    }
    Err(e) => Err(e), // Non-retryable
    Ok(status) => Ok(status),
}
```

## What Success Looks Like

### Functional Success
- Create job → saved to database, directories created (Phase 2.2 + new persistence)
- Submit job → SLURM job ID captured and tracked
- Close app → reopen shows existing jobs with current SLURM status
- Job completes → status automatically updates to COMPLETED
- Manual sync → job status refreshes from SLURM immediately

### Code Quality Success
- Database operations are transaction-safe and handle conflicts
- SLURM parsing handles all queue states and error conditions
- Status sync uses existing retry logic for network failures
- All new functionality has comprehensive unit tests
- Integration with existing Phase 2.2 patterns is seamless

## Integration with Existing Code

### Leverage Phase 2.2 Patterns
- **Use ConnectionUtils**: All SLURM commands should use existing retry wrapper
- **Use existing validation**: Job IDs, usernames go through existing sanitization
- **Use existing error handling**: Classify database errors as retryable vs non-retryable
- **Follow existing command patterns**: Database commands should mirror job commands structure

### Where to Hook In
```rust
// src-tauri/src/commands/jobs.rs - Existing functions to enhance
create_job_real() // Add: database save after directory creation
submit_job_real() // Add: capture SLURM job ID, update status to PENDING
delete_job_real() // Add: remove from database after directory cleanup

// NEW commands to add
sync_job_status() // Manual status refresh
list_jobs() // Load all jobs from database with current status
get_job_details() // Get full job info including status history
```

## Final Checklist

Before marking the task complete:
- [ ] Jobs persist across application restarts
- [ ] SLURM status sync works for all job states (PENDING, RUNNING, COMPLETED, FAILED)
- [ ] Database operations handle concurrent access and failures gracefully
- [ ] Status parsing handles all SLURM output formats and edge cases
- [ ] Integration with existing Phase 2.2 directory management is seamless
- [ ] All new functionality has comprehensive unit tests
- [ ] Manual and automatic status sync both work reliably
- [ ] Error handling follows existing patterns with proper retry logic

## Mock Data and Testing Resources

**Use our comprehensive reference documentation for testing:**

From `docs/reference/slurm-commands-reference.md`:
- **Mock squeue responses**: `12345678|test_job|R|00:15:30|01:44:30|1|24|16GB|amilan|/scratch/alpine/testuser/namdrunner_jobs/test_job`
- **Mock sacct responses**: `12345678|test_job|COMPLETED|0:0|2025-01-15T10:00:00|2025-01-15T11:00:00|01:00:00|/scratch/alpine/testuser/namdrunner_jobs/test_job`
- **Mock error responses**: `sbatch: error: Batch job submission failed: Requested node configuration is not available`

From `docs/reference/namd-commands-reference.md`:
- **Mock template variables** for testing config generation
- **Mock NAMD execution output** for job completion detection
- **Resource allocation patterns** for database testing

## Getting Help

If you get stuck:
1. **Start with our documentation** - `docs/reference/slurm-commands-reference.md` and `docs/reference/namd-commands-reference.md` have everything you need
2. **Check Phase 2.2 patterns** - Look at how directory management and retry logic work in existing code
3. **Review Python reference** - SLURM integration patterns in `docs/reference/NAMDRun-python/` for complex scenarios
4. **Test incrementally** - Get database working first, then SLURM parsing, then integration
5. **Follow existing conventions** - Use established patterns for validation, error handling, testing

The goal is bulletproof job persistence and status tracking that builds seamlessly on the solid Phase 2.2 foundation while providing the job continuity scientists need for long-running NAMD simulations.