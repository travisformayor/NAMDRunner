# NAMDRunner Project Specification

## What the Application Is (and Is Not)

### Purpose

A **local desktop application** that runs on a **Windows** researcher's PC to prepare, submit, and track **NAMD** simulations on a remote **Slurm** cluster. There are **no hosted services** and no server component we maintain.

### Primary User

A **fellow researcher** (not a paying customer). We can help them set it up initially, but the app must be **stable and low-maintenance** over time. This does not need to be an enterprise level application, but it should still use clean code and use good basic best practices.

### Hard Constraints (Must Honor)

* **Local-only desktop app** (avoid CORS entirely; no hosted backend; no local HTTP server).
* **SSH auth = username/password only** (no SSH keys).
* **Never persist credentials** to disk; re-auth is manual when sessions expire.
* **No cluster-resident application** and **no DB** on the cluster. Only files and Slurm CLI.
* **Local cache DB** is **SQLite**.
* UI should be pleasant and easy to evolve.
* Codebase must support **strict typing, unit tests, automated UI tests, linting**.

## End-to-End Workflow

1. **Connect via SSH** (password or keyboard-interactive). Session lives in memory only.
2. **Wizard**: build NAMD `.conf` from templates; user attaches inputs.
3. **Stage & upload** via SFTP to `/projects/$USER/namdrunner_jobs/...`; write **JSON** metadata files (top-level job/project info).
4. **Submit**: copy staged inputs to a **scratch** run dir; `sbatch` the generated job script; capture JobID.
5. **Track**: poll **Slurm** (`squeue` while pending/running, `sacct` for terminal states) and update local cache + remote JSON.
6. **Results**: browse remote folders, download outputs as needed.

## Data Placement Strategy

* **Local**: SQLite cache with projects, job rows, timestamps, hashes/sizes, last-seen statuses.
* **Remote** (cluster filesystem): JSON "meta" files under project/job folders; job scratch dir contains runtime outputs.
* **Single-writer rule**: the app writes the JSON; jobs write only inside their scratch/results directories.

## Target Platforms & Build Matrix

* **Users:** Windows (primary distribution target).
* **Development:** **Linux** (primary dev environment).
* **macOS:** We must be able to **build and run a macOS build locally** (manual testing only) on a Mac that has a copy of the repo.
* **CI:** Windows builds (portable `.exe`) via GitHub Actions. Optional Linux CI for tests.

## Security Model

### Credentials
* **In-memory only**: never log or persist. Mask inputs; clear memory on disconnect; handle crashes defensively.

### Application Security
* **Tauri capabilities**: enable only the commands and APIs we require; disable shell/navigation we don't use; set a strict CSP.
* **Logs**: local rotating logs with redaction; no telemetry; network calls restricted to SSH/SFTP.

## Packaging & Delivery

* **Windows artifact:** **portable `.exe`** built via **GitHub Actions** (Windows runner).
* **Linux:** developer builds for day-to-day work; optional Linux packaging if needed for internal testers.
* **macOS:** local builds for manual smoke on a Mac (developer machine), to validate the mac build path. (Distribution to end users is not required.)

## Success Criteria

### User Experience Goals
* Scientists can submit NAMD jobs without command line
* Jobs don't mysteriously fail due to our tool
* Tool works reliably for months without maintenance
* Scientists can reopen the tool and see past submitted jobs
* New developers can understand and modify code

### Technical Goals
* Portable Windows executable that "just works"
* No credential persistence (security requirement)
* Offline mode with local cache
* Type-safe boundaries between frontend and backend
* Comprehensive test coverage

## Things to Avoid (Enforce in Code Review)

* Storing credentials in any form; logging secrets; printing raw command lines with secrets.
* Adding hosted services, local HTTP servers, or anything that re-introduces **CORS**.
* Switching to SSH keys (cluster disallows).
* Installing a server or DB on the cluster.
* Attempting desktop E2E on macOS (no WKWebView WebDriver). Use **Linux desktop E2E**; mac builds are for manual smoke.
* Over-broad Tauri permissions.

## Open Items (Affect Implementation Details)

* **Slurm version**: confirm if `squeue` / `sacct` support JSON; otherwise plan formatted fallbacks.
* **Scratch purge cadence**: decide when to copy back results.