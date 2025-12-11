# Database & Data Schemas

This document defines all data persistence patterns for NAMDRunner, including SQLite storage, JSON metadata formats, validation rules, and data management strategies.

## Table of Contents
- [Database Architecture](#database-architecture)
- [Database Location](#database-location)
- [SQLite Schema](#sqlite-schema)
- [Database Management](#database-management)
- [JSON Metadata Schema](#json-metadata-schema)
  - [job_info.json (Server-Side)](#job_infojson-server-side)
  - [Template Schema](#template-schema)
  - [Template Values Schema](#template-values-schema)
  - [OutputFile Schema](#outputfile-schema)
- [File Organization](#file-organization)
- [Validation Rules](#validation-rules)
- [Data Type Mappings](#data-type-mappings)
- [Best Practices](#best-practices)

## Database Architecture

**Design Philosophy**: Simple storage for job caching and template management. No complex schema, no migrations, minimal manual serialization.

### Key Principles
1. **Job Document Storage**: Entire `JobInfo` struct serialized as JSON in `jobs` table
2. **Template Storage**: Templates stored with structured columns for easy querying, but variables serialized as JSON
3. **No Schema Coupling**: Adding fields to Rust struct = automatic DB support via serde
4. **Zero Migrations**: No backward compatibility needed - users can delete old DB
5. **Performance**: SQLite operations are fast enough for desktop use (< 100 jobs typical)

### When Data is Stored
- **Local SQLite Jobs Table**: Job caching only - enables offline viewing of job list
- **Local SQLite Templates Table**: Template definitions - embedded defaults loaded on first use, custom templates added by users
- **Cluster (job_info.json)**: Single source of truth for job metadata (includes template_id and template_values)
- **Sync Pattern**: Download from cluster → cache in SQLite → display in UI

## Database Location

Database initialization occurs during app startup in the `.setup()` hook, where the `AppHandle` is available for platform-specific path resolution.

**Development Builds:**
- Path: `./namdrunner_dev.db` (current working directory)
- Detected via `cfg!(debug_assertions)`
- Allows easy inspection during development

**Production Builds:**
- Path resolved via Tauri's `app_data_dir()` API
- Linux: `~/.local/share/namdrunner/namdrunner.db`
- Windows: `%APPDATA%\namdrunner\namdrunner.db`
- macOS: `~/Library/Application Support/namdrunner/namdrunner.db`
- Directory created automatically if missing

**Path Resolution:**
```rust
pub fn get_database_path(app_handle: &tauri::AppHandle) -> Result<PathBuf> {
    if cfg!(debug_assertions) {
        Ok(PathBuf::from("./namdrunner_dev.db"))
    } else {
        let app_data_dir = app_handle.path().app_data_dir()?;
        std::fs::create_dir_all(&app_data_dir)?;
        Ok(app_data_dir.join("namdrunner.db"))
    }
}
```

## SQLite Schema

**Two tables: jobs (document store) and templates (structured storage):**

```sql
-- Simple document store for job caching
CREATE TABLE IF NOT EXISTS jobs (
    job_id TEXT PRIMARY KEY,
    data TEXT NOT NULL
);

-- Index on status for filtering (uses JSON extraction)
CREATE INDEX IF NOT EXISTS idx_jobs_status
ON jobs(json_extract(data, '$.status'));

-- Template storage for NAMD simulation templates
CREATE TABLE IF NOT EXISTS templates (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    namd_config_template TEXT NOT NULL,  -- NAMD config with {{variable}} placeholders
    variables TEXT NOT NULL,               -- JSON: HashMap<String, VariableDefinition>
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

### Why This Works
- **Jobs table**: Document store pattern - serde handles serialization, no manual column mapping
- **Templates table**: Structured columns for common fields (id, name, description) enable efficient listing, while variables serialized as JSON for flexibility
- **Easy to extend**: Add fields to Rust types, serde handles the rest
- **JSON functions**: SQLite can query JSON directly (e.g., status index on jobs, template_id lookup)

### API Usage

```rust
// Job operations
db.save_job(&job_info)?;                    // Save entire JobInfo struct as JSON
let job = db.load_job("job_001")?;          // Load job by ID
let jobs = db.load_all_jobs()?;             // Load all jobs (sorted by created_at DESC)
db.delete_job("job_001")?;                  // Delete job

// Template operations
db.save_template(&template)?;                                   // Save/update template
let template = db.load_template("vacuum_optimization_v1")?;     // Load full template
let summaries = db.list_templates()?;                           // List all (id, name, description)
db.delete_template("custom_template_v1")?;                      // Delete template
let count = db.count_jobs_using_template("vacuum_optimization_v1")?;  // Count jobs using template

// Embedded template loading (automatic on first use)
ensure_default_templates_loaded()?;  // Idempotent - loads defaults if not already loaded
```

**That's it.** No manual serialization, no column lists, no migrations.

## Database Management

The application provides built-in database management operations accessible through the Settings page.

### Operations

**Get Database Info:**
- Returns current database file path and size
- Used for display in Settings UI
- Path determined during app initialization

**Backup Database:**
- Opens OS file save dialog for user to choose backup location
- Uses SQLite Backup API (`rusqlite::backup::Backup`) for safe online backups
- Creates consistent snapshot even while database is in use
- No application restart required

**Restore Database:**
- Opens OS file dialog for user to select backup file
- Validates source file is a valid SQLite database
- Closes current connection, replaces database file, reopens connection
- Atomic operation - holds `DATABASE` lock throughout
- Frontend automatically reloads all stores (jobs, templates, settings)
- No application restart required

**Reset Database:**
- Deletes current database file
- Reinitializes with fresh schema
- Atomic operation - holds `DATABASE` lock throughout
- Frontend automatically reloads all stores
- Safe even with running jobs - sync auto-discovers from cluster metadata
- No application restart required

### Connection Management

Database connections are managed through the global `DATABASE` instance:

```rust
lazy_static! {
    pub static ref DATABASE: Arc<Mutex<Option<JobDatabase>>> = Arc::new(Mutex::new(None));
    static ref DATABASE_PATH: Arc<Mutex<Option<PathBuf>>> = Arc::new(Mutex::new(None));
}

pub fn reinitialize_database(db_path: &str) -> Result<()> {
    let mut database_lock = DATABASE.lock().unwrap();

    // Drop old connection (closes SQLite)
    *database_lock = None;

    // Create new connection
    let db = JobDatabase::new(db_path)?;
    *database_lock = Some(db);

    // Update tracked path
    let mut path_lock = DATABASE_PATH.lock().unwrap();
    *path_lock = Some(PathBuf::from(db_path));

    Ok(())
}
```

**Key Points:**
- Connection reinitializiation is safe - proper locking prevents concurrent access
- All operations atomic (connection closed before file operation, reopened after)
- Backup uses SQLite Backup API for consistency (no file-level copy during active use)
- Frontend state reloads automatically after restore/reset

## JSON Metadata Schema

### job_info.json (Server-Side)
This file is created in each job directory on the cluster and contains all job metadata. The schema is generated by serializing the `JobInfo` struct from `src-tauri/src/types/core.rs`.

```json
{
  "job_id": "test-1_1760733363035078",
  "job_name": "test-1",
  "status": "COMPLETED",
  "slurm_job_id": "12345678",
  "created_at": "2025-01-15T10:30:00Z",
  "updated_at": "2025-01-15T11:45:00Z",
  "submitted_at": "2025-01-15T10:35:00Z",
  "completed_at": "2025-01-15T11:00:00Z",
  "project_dir": "/projects/username/namdrunner_jobs/test-1_1760733363035078",
  "scratch_dir": "/scratch/alpine/username/namdrunner_jobs/test-1_1760733363035078",
  "error_info": null,
  "slurm_stdout": null,
  "slurm_stderr": null,
  "template_id": "explicit_solvent_npt_v1",
  "template_values": {
    "structure_file": "hextube.psf",
    "coordinates_file": "hextube.pdb",
    "parameters_file": "par_all36_na.prm",
    "extrabonds_file": "hextube.exb",
    "output_name": "npt_equilibration",
    "temperature": 300.0,
    "timestep": 2.0,
    "cell_x": 124.0,
    "cell_y": 114.0,
    "cell_z": 323.0,
    "pme_grid_spacing": 1.5,
    "langevin_damping": 5.0,
    "langevin_piston_target": 1.01325,
    "xst_freq": 1200,
    "output_energies_freq": 1200,
    "dcd_freq": 1200,
    "restart_freq": 1200,
    "output_pressure_freq": 1200,
    "execution_command": "minimize",
    "steps": 4800
  },
  "slurm_config": {
    "cores": 24,
    "memory": "16GB",
    "walltime": "02:00:00",
    "partition": "amilan",
    "qos": "normal"
  },
  "input_files": [
    "hextube.psf",
    "hextube.pdb",
    "par_all36_na.prm",
    "hextube.exb"
  ],
  "output_files": [
    {
      "name": "output.dcd",
      "size": 145829376,
      "modified_at": "2025-01-15T11:00:00Z"
    },
    {
      "name": "restart.coor",
      "size": 8192000,
      "modified_at": "2025-01-15T11:00:00Z"
    },
    {
      "name": "restart.vel",
      "size": 8192000,
      "modified_at": "2025-01-15T11:00:00Z"
    },
    {
      "name": "restart.xsc",
      "size": 4096,
      "modified_at": "2025-01-15T11:00:00Z"
    }
  ],
  "remote_directory": "/projects/username/namdrunner_jobs/test-1_1760733363035078"
}
```

### Template Schema

Templates define NAMD configuration files with variable placeholders. They are stored in the `templates` table and define the structure for creating NAMD jobs.

```typescript
interface Template {
  id: string;                    // Unique template identifier (e.g., "explicit_solvent_npt_v1")
  name: string;                  // Display name (e.g., "Explicit Solvent NPT")
  description: string;           // User-facing description
  namd_config_template: string;  // NAMD config text with {{variable}} placeholders
  variables: {                   // Variable definitions (serialized HashMap)
    [key: string]: VariableDefinition;
  };
  created_at: string;            // RFC3339 timestamp
  updated_at: string;            // RFC3339 timestamp
}

interface VariableDefinition {
  key: string;                   // Variable key matching placeholder (e.g., "temperature")
  label: string;                 // UI display label (e.g., "Temperature (K)")
  var_type: VariableType;        // Type definition with constraints
  help_text?: string;            // Optional help text for UI
}

type VariableType =
  | { Number: { min: number; max: number; default: number } }
  | { Text: { default: string } }
  | { Boolean: { default: boolean } }
  | { FileUpload: { extensions: string[] } };  // e.g., [".psf", ".pdb"]

interface TemplateSummary {
  id: string;                    // Unique template identifier
  name: string;                  // Display name
  description: string;           // User-facing description
}
```

**Embedded Templates:**
- Default templates are embedded in the binary using `include_str!` macro
- Located in `src-tauri/templates/` directory (JSON files)
- Loaded automatically on first `list_templates()` call
- Current defaults: `vacuum_optimization_v1`, `explicit_solvent_npt_v1`

### Template Values Schema

Template values are stored as a JSON object mapping variable keys to their values. The values can be strings, numbers, or booleans depending on the variable type defined in the template.

```typescript
interface TemplateValues {
  [key: string]: string | number | boolean;

  // Example for explicit_solvent_npt_v1 template:
  structure_file: string;      // Filename only (e.g., "hextube.psf")
  coordinates_file: string;    // Filename only (e.g., "hextube.pdb")
  parameters_file: string;     // Filename only (e.g., "par_all36_na.prm")
  extrabonds_file: string;     // Filename only (e.g., "hextube.exb")
  output_name: string;         // Output prefix (e.g., "npt_equilibration")
  temperature: number;         // Simulation temperature in Kelvin
  timestep: number;            // Integration timestep in femtoseconds
  steps: number;               // Number of simulation steps
  execution_command: string;   // "minimize" or "run"
  cell_x: number;              // Periodic box X dimension
  cell_y: number;              // Periodic box Y dimension
  cell_z: number;              // Periodic box Z dimension
  // ... additional variables defined by template
}
```

**Template Rendering:**
- During job submission, template values are substituted into `{{variable}}` placeholders in the NAMD config template
- All variables are implicitly required - rendering fails if any placeholder is missing a value
- FileUpload variables: filenames get "input_files/" prepended automatically (e.g., "hextube.psf" → "input_files/hextube.psf")
- Number variables: rendered with appropriate precision (integers without decimals)
- Boolean variables: converted to "yes"/"no" for NAMD
- Text variables: used as-is

### OutputFile Schema

```typescript
interface OutputFile {
  name: string;              // Filename (e.g., "output.dcd")
  size: number;              // File size in bytes
  modified_at: string;       // RFC3339 timestamp from server
}
```

**When populated:**
- Created during automatic job completion (when job reaches terminal state)
- Backend does single batch SFTP readdir in project directory's outputs/
- All output files queried at once (no per-file round trips)

### InputFiles Field

The `input_files` field stores the list of uploaded input file names for the job.

**Type:** `Vec<String>`
**TypeScript:** `input_files: string[]`

**Always populated:**
- Set during job creation (empty array if no files uploaded)
- Contains just filenames (e.g., `["structure.pdb", "parameters.prm"]`)
- Used by InputFilesTab for display and download operations
- Files stored in `{job_directory}/input_files/` on cluster

## File Organization

### Directory Structure
```
/projects/$USER/namdrunner_jobs/
└── {job_id}/
    ├── job_info.json           # Complete job metadata (this schema)
    ├── config.namd             # Generated NAMD config (in job root)
    ├── job.sbatch              # Generated SLURM script (in job root)
    ├── input_files/
    │   ├── structure.pdb
    │   ├── structure.psf
    │   └── parameters.prm
    ├── outputs/                # Created after job completion (rsync from scratch)
    │   ├── output.dcd          # Trajectory
    │   ├── restart.coor        # Restart files
    │   ├── restart.vel
    │   └── restart.xsc
    └── logs/                   # SLURM logs
        ├── {job_name}_{slurm_job_id}.out
        └── {job_name}_{slurm_job_id}.err

/scratch/alpine/$USER/namdrunner_jobs/
└── {job_id}/                   # Working directory during execution
    ├── config.namd             # Copied from project
    ├── job.sbatch
    ├── input_files/
    │   ├── structure.pdb
    │   ├── structure.psf
    │   └── parameters.prm
    ├── outputs/
    │   ├── output.dcd          # Generated during run
    │   ├── restart.coor
    │   ├── restart.vel
    │   └── restart.xsc
    └── namd_output.log
```

**Key Points:**
- **Project directory**: Permanent storage, survives job completion
- **Scratch directory**: Fast local storage during job run, auto-purged after 90 days
- **Automatic rsync**: On job completion, scratch → project (data preservation)
- **Metadata fetching**: Input file sizes fetched after upload, output file sizes fetched after job completion

### File Naming Conventions
- **SLURM script**: `job.sbatch` (in job root)
- **NAMD config**: `config.namd` (in job root)
- **Job metadata**: `job_info.json` (in job root)
- **SLURM Stdout**: `logs/{job_name}_{slurm_job_id}.out`
- **SLURM Stderr**: `logs/{job_name}_{slurm_job_id}.err`
- **Trajectory**: `outputs/output.dcd`
- **Restart files**: `outputs/restart.{coor,vel,xsc}`

## Validation Rules

### Job ID Format
- **Pattern**: `{job_name}_{timestamp_millis}`
- **Examples**: `test-1_1760733363035078`, `sim_alpha_1760040523960286`
- **Globally unique** (timestamp component ensures uniqueness)

### File Path Validation
- **No directory traversal** (`../`)
- **Allowed file extensions**: `.pdb`, `.psf`, `.prm`, `.exb`, `.extra`, `.namd`, `.sbatch`, `.out`, `.err`, `.log`, `.dcd`, `.coor`, `.vel`, `.xsc`
- **Size limits**: 1GB per file (configurable)

### Parameter Ranges

**Template Variables:**
- Validation constraints are defined per-template in the `VariableDefinition.var_type` field
- Number type variables include min/max/default constraints
- FileUpload type variables include allowed file extensions
- See Template Schema for details

**SLURM Resources:**
```typescript
interface SlurmValidation {
  cores: { min: 1, max: 64 };           // Single-node limit
  memory_gb: { min: 1, max: 256 };      // Varies by partition
  walltime_hours: { min: 0.1, max: 168 }; // Up to 7 days with long QoS
}
```

## Data Type Mappings

### TypeScript ↔ Rust
```rust
// Core types
pub type JobId = String;          // {name}_{timestamp}
pub type SlurmJobId = String;     // SLURM's numeric job ID
pub type Timestamp = String;      // RFC3339 format

// Status enum (serde handles serialization)
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

// File type enum
#[derive(Serialize, Deserialize)]
pub enum NAMDFileType {
    #[serde(rename = "pdb")]
    Pdb,
    #[serde(rename = "psf")]
    Psf,
    #[serde(rename = "prm")]
    Prm,
    #[serde(rename = "exb")]
    Exb,
    #[serde(rename = "other")]
    Other,
}
```

### SQLite JSON ↔ Rust
```rust
// Save: serde_json::to_string(&job_info)
// Load: serde_json::from_str::<JobInfo>(&json_data)
```

**No manual mapping needed** - serde handles everything automatically.

## Best Practices

### Database Operations
1. **Always use `with_database()` wrapper** for access
2. **Synchronous is fine** - SQLite operations are microseconds, no async overhead needed
3. **No connection pooling** - single-threaded desktop app, one connection is enough
4. **Graceful degradation** - old DB? Delete it, recreate with new schema
5. **Initialize in `.setup()` hook** - ensures `AppHandle` available for platform-specific paths
6. **Use SQLite Backup API** - for consistent snapshots during active use

### Database Management
1. **Backup before major changes** - users can backup via Settings page
2. **Reset is safe** - sync auto-discovers jobs from cluster, templates can be recreated
3. **Restore is atomic** - holds lock throughout operation, prevents concurrent access
4. **No app restart needed** - connection reinitializes cleanly after restore/reset

### Schema Changes
1. **No migrations** - users delete old DB file or use Reset feature
2. **Add fields freely** - serde handles missing fields with `#[serde(default)]`
3. **Breaking changes OK** - development phase, no backward compatibility burden

### Performance
- **Typical dataset**: ~100 jobs cached locally
- **Query time**: < 1ms for load_all_jobs()
- **JSON parsing**: Negligible overhead for this data size
- **Status index**: Fast filtering by job status

### Data Integrity
1. **Server is source of truth** - SQLite is just a cache
2. **Sync resolves conflicts** - download from server overwrites local
3. **No complex sync logic** - simple replace-on-sync pattern
4. **User can always re-sync** - if local DB corrupt, just sync or reset

---

*For IPC interfaces and API contracts, see [`docs/API.md`](API.md).*
*For cluster-specific file system details, see [`docs/reference/alpine-cluster-reference.md`](reference/alpine-cluster-reference.md).*
