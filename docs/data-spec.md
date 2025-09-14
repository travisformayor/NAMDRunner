# Data Specification

This document defines all data structures, schemas, and formats used by NAMDRunner, including JSON metadata, SQLite schemas, file organization, and validation rules.

## JSON Metadata Schema

### job_info.json (Single Job)
This file is created in each job directory on the cluster and contains all job metadata.

```json
{
  "schema_version": "1.0",
  "job_id": "job_001",
  "job_name": "test_simulation",
  "created_at": "2025-01-15T10:30:00Z",
  "updated_at": "2025-01-15T11:45:00Z",
  "status": "COMPLETED",
  "slurm_job_id": "12345678",
  "submitted_at": "2025-01-15T10:35:00Z",
  "completed_at": "2025-01-15T11:00:00Z",
  "config": {
    "namd": {
      "steps": 50000,
      "temperature": 310.0,
      "timestep": 2.0,
      "outputname": "output",
      "dcd_freq": 1000,
      "restart_freq": 5000
    },
    "slurm": {
      "cores": 24,
      "memory": "16GB", 
      "walltime": "02:00:00",
      "partition": "amilan",
      "qos": "normal"
    }
  },
  "input_files": [
    {
      "name": "structure.pdb",
      "path": "input_files/structure.pdb",
      "type": "pdb"
    },
    {
      "name": "structure.psf", 
      "path": "input_files/structure.psf",
      "type": "psf"
    },
    {
      "name": "parameters.prm",
      "path": "input_files/parameters.prm", 
      "type": "prm"
    }
  ],
  "generated_files": {
    "namd_config": "config.namd",
    "slurm_script": "job.sbatch"
  },
  "output_files": {
    "slurm_stdout": "test_simulation_12345678.out",
    "slurm_stderr": "test_simulation_12345678.err",
    "namd_log": "namd_output.log"
  },
  "directories": {
    "project_dir": "/projects/username/namdrunner_jobs/job_001",
    "scratch_dir": "/scratch/alpine/username/namdrunner_jobs/job_001"
  },
  "error_info": null
}
```

### Error Information Format
When jobs fail, the `error_info` field contains:
```json
"error_info": {
  "error_type": "SLURM_ERROR",
  "error_message": "Job exceeded walltime limit", 
  "error_code": "TIMEOUT",
  "failed_at": "2025-01-15T13:00:00Z",
  "slurm_exit_code": 1
}
```

## SQLite Schema (Phase 1 Version)

### Jobs Table
```sql
-- Jobs table for local cache
CREATE TABLE jobs (
    job_id TEXT PRIMARY KEY,               -- Our internal job ID
    job_name TEXT NOT NULL,
    status TEXT NOT NULL,                  -- JobStatus enum values
    slurm_job_id TEXT,                     -- SLURM's job ID
    created_at TEXT NOT NULL,              -- ISO 8601 timestamp
    updated_at TEXT,
    submitted_at TEXT,
    completed_at TEXT,
    project_dir TEXT,
    scratch_dir TEXT,
    
    -- JSON columns for complex data
    namd_config_json TEXT,                 -- NAMD configuration as JSON
    slurm_config_json TEXT,                -- SLURM configuration as JSON
    input_files_json TEXT,                 -- Input file list as JSON
    output_files_json TEXT,                -- Output file list as JSON
    error_info TEXT                        -- Error information
);

-- Indexes for performance
CREATE INDEX idx_jobs_status ON jobs(status);
CREATE INDEX idx_jobs_slurm_id ON jobs(slurm_job_id);
CREATE INDEX idx_jobs_updated ON jobs(updated_at);
```

### Application Metadata Table
```sql
-- Application metadata table
CREATE TABLE app_metadata (
    key TEXT PRIMARY KEY,
    value TEXT,
    updated_at TEXT
);

-- Insert schema version
INSERT INTO app_metadata (key, value, updated_at) 
VALUES ('schema_version', '1.0', datetime('now'));
```

### Python Implementation Compatibility Notes

The Python version used more complex schemas that should inform future expansion:

#### Job Groups (Future Consideration)
Python used UUID-based group IDs and multi-stage workflows:
```sql
-- Python implementation patterns (for reference)
-- job_groups table with UUID group IDs
-- job_stages table with stage_id, stage_number, stage_name
-- job_outputs table for cached SLURM stdout/stderr
```

#### Migration Strategy
- **Data Integrity**: Validate JSON before parsing, handle missing fields gracefully
- **Backward Compatibility**: Maintain compatibility for at least one version
- **Status Values**: Must match between Python and Rust implementations
- **Timestamp Format**: ISO 8601 strings for cross-language compatibility

## File Organization Requirements

### Directory Structure
> **See [`docs/cluster-guide.md`](cluster-guide.md)** for complete directory structure requirements and Alpine cluster file system details.

```
/projects/$USER/namdrunner_jobs/
└── {job_id}/
    ├── job_info.json           # This schema
    ├── input_files/
    │   ├── structure.pdb
    │   ├── structure.psf
    │   └── parameters.prm
    ├── config.namd             # Generated NAMD config
    ├── job.sbatch              # Generated SLURM script
    └── outputs/                # After job completion
        ├── {job_name}_{slurm_job_id}.out
        ├── {job_name}_{slurm_job_id}.err
        └── namd_output.log

/scratch/alpine/$USER/namdrunner_jobs/
└── {job_id}/                   # Working directory during execution
    ├── config.namd
    ├── job.sbatch
    ├── structure.pdb
    ├── structure.psf
    ├── parameters.prm
    ├── namd_output.log
    ├── output.dcd              # Trajectory
    ├── restart.coor            # Restart files
    ├── restart.vel
    └── restart.xsc
```

### File Naming Conventions
- **SLURM script**: `job.sbatch`
- **NAMD config**: `config.namd`
- **Job metadata**: `job_info.json`
- **SLURM Stdout**: `{job_name}_{slurm_job_id}.out`
- **SLURM Stderr**: `{job_name}_{slurm_job_id}.err`
- **NAMD Log**: `namd_output.log`
- **Trajectory**: `output.dcd`
- **Restart files**: `restart.coor`, `restart.vel`, `restart.xsc`

### Python Implementation Directory Pattern (Reference)
The Python version used this structure (informational for future multi-stage support):
```
/projects/$USER/namdrunner_jobs/
├── <job_group_name>/
│   ├── job_group_info.json      # Multi-stage metadata
│   ├── input_files/
│   └── stage_1/
       ├── config.namd
       ├── job.sbatch
       └── job_<id>.out
```

## Validation Rules

### Job ID Format
- **Pattern**: `^job_[0-9]{3,6}$`
- **Examples**: `job_001`, `job_12345`
- **Must be unique** within user's jobs

### File Path Validation
- **No absolute paths** in metadata
- **All paths relative** to job directory
- **No directory traversal** (`../`)
- **Allowed file extensions**: `.pdb`, `.psf`, `.prm`, `.namd`, `.sbatch`, `.out`, `.err`, `.log`, `.dcd`, `.coor`, `.vel`, `.xsc`

### Parameter Ranges
> **Note**: SLURM resource limits are defined in [`docs/cluster-guide.md`](cluster-guide.md) and may vary by partition.

```typescript
interface ValidationRules {
  namd: {
    steps: { min: 1, max: 100000000 };
    temperature: { min: 200, max: 400 };  // Kelvin
    timestep: { min: 0.1, max: 4.0 };     // femtoseconds
  };
  slurm: {
    cores: { min: 1, max: 128 };          // Max for amilan128c
    memory_gb: { min: 1, max: 256 };      // Varies by partition
    walltime_hours: { min: 0.1, max: 168 }; // Up to 7 days with long QoS
  };
}
```

### Resource Limits (Alpine Cluster)
> **For current resource limits, partition details, and QoS specifications, see [`docs/cluster-guide.md`](cluster-guide.md)**.

- **Default partition**: "amilan"
- **Default QOS**: "normal"

## Schema Version Management

### JSON Schema Version
- **Always include** `schema_version` field for future compatibility
- **Version 1.0**: Initial single-job implementation
- **Future versions**: Will support job groups, multi-stage workflows

### SQLite Schema Evolution
```sql
-- Example future migration pattern
ALTER TABLE jobs ADD COLUMN job_group_id TEXT;
UPDATE app_metadata SET value = '1.1' WHERE key = 'schema_version';
```

### Compatibility Strategy
- **Graceful degradation** for unknown fields in JSON
- **Schema version checks** before parsing
- **Migration scripts** for breaking changes
- **Maintain at least one version backward compatibility**

## Data Type Mappings

### TypeScript to Rust
```rust
// String types
pub type JobId = String;          // job_001, job_002, etc.
pub type SlurmJobId = String;     // SLURM's numeric job ID as string
pub type Timestamp = DateTime<Utc>; // ISO 8601 in JSON, DateTime in Rust

// Status enums (must match exactly)
#[derive(Serialize, Deserialize)]
pub enum JobStatus {
    #[serde(rename = "CREATED")]
    Created,
    #[serde(rename = "PENDING")]
    Pending,
    #[serde(rename = "RUNNING")]
    Running,
    #[serde(rename = "COMPLETED")]
    Completed,
    #[serde(rename = "FAILED")]
    Failed,
    #[serde(rename = "CANCELLED")]
    Cancelled,
}
```

### SQLite to Rust Type Mapping
```rust
// SQLite TEXT -> Rust String
// SQLite INTEGER -> Rust i64
// SQLite REAL -> Rust f64
// JSON columns -> serde_json::Value -> structured types
```

## Data Integrity Requirements

### Required Fields
All job records must have:
- `job_id` (unique identifier)
- `job_name` (user-provided name)
- `status` (valid JobStatus enum value)
- `created_at` (ISO 8601 timestamp)

### Optional Fields with Defaults
- `partition` → "amilan"
- `qos` → "normal"
- `dcd_freq` → null (no trajectory output)
- `restart_freq` → null (no restart files)

### JSON Field Validation
```rust
// Example validation in Rust
impl JobInfo {
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Validate job_id format
        if !JOB_ID_REGEX.is_match(&self.job_id) {
            return Err(ValidationError::InvalidJobId);
        }
        
        // Validate temperature range
        if self.namd_config.temperature < 200.0 || self.namd_config.temperature > 400.0 {
            return Err(ValidationError::InvalidTemperature);
        }
        
        // Additional validations...
        Ok(())
    }
}
```

## Performance Considerations

### Database Indexing Strategy
- **Primary lookups**: job_id (primary key)
- **Status queries**: idx_jobs_status for filtering by status
- **SLURM integration**: idx_jobs_slurm_id for mapping SLURM jobs to our jobs
- **Temporal queries**: idx_jobs_updated for sync operations

### JSON Column Usage
- **Store complex nested data** (configs, file lists) as JSON
- **Query by simple fields** (status, job_id) using indexed columns
- **Avoid JSON path queries** for performance
- **Denormalize frequently queried fields** into dedicated columns

### Memory Management
- **Load job lists lazily** for large datasets
- **Paginate job queries** in UI
- **Cache frequently accessed** job metadata
- **Clear completed job details** from memory after display

## Important Implementation Notes

1. **Schema versioning is critical** - always include version field
2. **Timestamps must be UTC** and ISO 8601 formatted
3. **Status values must match** between TypeScript and Rust exactly
4. **File paths are relative** to job directory in metadata
5. **Working directory pattern** identifies NAMDRunner jobs
6. **JSON validation** prevents corrupt data from breaking the app
7. **SQLite WAL mode** recommended for concurrent access
8. **Regular maintenance** of completed job records may be needed
9. **Backup strategy** should include both SQLite and project directories
10. **Migration testing** required for schema changes