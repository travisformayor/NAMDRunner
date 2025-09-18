# NAMDRunner Technical Specification

## Technology Stack

### Shell/Runtime
* **Tauri v2** (desktop shell using system WebView). The frontend is built to **static assets** (HTML/CSS/JS) and **embedded** into the binary; no Node runtime ships, and there's no local server. This avoids CORS, keeps the footprint small, and minimizes maintenance.

### Frontend
* **Svelte + TypeScript**
  * Compile-time reactivity → small output and fewer "mystery re-renders."
  * Single-file components and explicit `$:` derivations → easier to read and reason about.
  * Solid unit/component testing story (Vitest + Svelte Testing Library).

### Backend (App Core)
* **Rust** with **Tauri commands** as the IPC boundary from UI.
* **SSH/SFTP** via Rust `ssh2` (libssh2). Password and keyboard-interactive auth match cluster policies.
* **SQLite** via `rusqlite` (or the Tauri SQL plugin).
* **Templating** (NAMD `.conf` + Slurm scripts) via `tera` or `handlebars`.

### Slurm Integration
* Submit with **`sbatch`** (parse returned JobID).
* Poll with **`squeue`** for queued/running and **`sacct`** for terminal states.
* Prefer JSON outputs if available on the site; otherwise use `--format` / `--parsable2` and parse.

### Why This Stack
* **Security/stability**: Rust core, minimal attack surface, no secrets on disk.
* **Maintainability**: typed boundaries (TS ↔ Rust), clear module seams, small binary.
* **UI velocity**: Svelte's component model is simple, predictable, and testable.

## System Architecture

### UI Layer (Svelte/TS)
* Components + Svelte stores for minimal app state; no direct system calls.
* All native operations go through a **single IPC client** module that calls Tauri commands.

### Rust Core
* Modules: `auth/session`, `sftp`, `slurm`, `sqlite`, `templating`, `logging`.
* Each module exposes typed functions; Tauri commands are thin wrappers.
* Slurm adapters parse `sbatch`, `squeue`, `sacct` outputs into typed models.

### Data Contracts
* Local SQLite schemas (projects, jobs, files, statuses) — see `docs/data-spec.md`
* Remote JSON shapes (`meta.json`, `job.json`) versioned with a `schema_version` field — see `docs/data-spec.md`
* IPC command interfaces between frontend and backend — see `docs/api-spec.md`
* Reconciliation treats **cluster** as source-of-truth on conflict

### Application States
* Session: `Disconnected | Connecting | Connected | Expired`.
* Job: `Created | Submitted | Pending | Running | Completed | Failed | Cancelled`.

## Development Setup

### Prerequisites
- **Node.js LTS** (via nvm recommended)
- **Rust toolchain** (via rustup.rs)
- **Git**

### First-Time Setup

> Follow the official Tauri v2 documentation for platform prerequisites: https://v2.tauri.app/start/

#### Linux/Fedora
```bash
# Tauri system dependencies
sudo dnf check-update
sudo dnf install -y webkit2gtk4.1-devel openssl-devel curl wget file libappindicator-gtk3-devel librsvg2-devel libxdo-devel
sudo dnf group install -y "C Development Tools and Libraries"

# Install Rust
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
source ~/.cargo/env

# Install Node.js via nvm
nvm install --lts && nvm use --lts

# Clone and setup project
git clone https://github.com/yourusername/namdrunner.git
cd namdrunner
npm install

# Smoke test
npm run tauri dev
```

#### macOS

```bash
# Install Dev tools
xcode-select --install
# Install Node/Rust with Homebrew
brew install node rust

# Clone repo
git clone https://github.com/yourusername/namdrunner.git
cd namdrunner
npm install
```

#### Windows

```powershell
# Install Rust from https://rustup.rs (MSVC)
# Install Node.js LTS from https://nodejs.org
# Ensure Visual Studio Build Tools / Desktop C++ are present for native deps

git clone https://github.com/yourusername/namdrunner.git
cd namdrunner
npm install
```

## Host vs. VM Builds (Rust)

**Goals**

* Keep **sources** on the shared mount (e.g., `/media/share/<REPO>`).
* On **Fedora VM**: send heavy I/O to VM disk.
* On **macOS host**: use normal project-local folders.

### Rust (Cargo `target/`)

**Fedora VM (zsh) — add to `~/.zshrc`:**

```zsh
# Use VM-local disk for Cargo artifacts
export CARGO_TARGET_DIR="$HOME/.cargo-target/namdrunner"
export CARGO_INCREMENTAL=0
```

Setup folder once:

```bash
mkdir -p "$HOME/.cargo-target/namdrunner"
```

**macOS host:**
No configuration needed. Ensure `CARGO_TARGET_DIR` is **not** set on macOS so Cargo writes to `./target` in the repo.

**Verify**

```bash
# On VM: artifacts at ~/.cargo-target/namdrunner
ls -1 "$HOME/.cargo-target/namdrunner" | head

# On Mac: artifacts in ./target
test -d target && echo "macOS using ./target ✅"
```

## Development Commands

```bash
# Frontend development
npm run dev               # Svelte dev server (Vite)
npm run build             # Build static frontend
npm run preview           # Preview built assets
npm run check             # svelte-kit sync + svelte-check
npm run check:watch       # svelte-check --watch
npm run lint              # ESLint + Prettier (check)
npm run lint:fix          # ESLint --fix + Prettier --write

# Tests
npm run test              # Vitest unit tests
npm run test:vitest-ui    # Vitest UI
npm run test:run          # Vitest run (CI-friendly)
npm run test:ui           # UI testing toolkit (under Xvfb)
npm run test:e2e          # WebdriverIO E2E (under Xvfb)

# Rust (executed in `src-tauri/`)
cargo test                # Rust unit tests
cargo clippy              # Rust lint

# Full Tauri application
npm run tauri dev         # Run app with hot reload
npm run tauri build       # Build release binary
```

## VM Development Environment (Optional)

For developers using a Fedora VM environment (e.g., UTM on macOS).

<details>
<summary>Fedora VM Setup (Click to expand)</summary>

### Platform
* **Fedora 38 ARM64** (UTM VM) for development
* **Workspace**: `/media/share/<repo-worktree>` mounted from the host (synced with host machine)

### Host Setup (Outside the VM)
1. **Port forwarding with socat**:
   ```bash
   socat TCP-LISTEN:2222,fork,reuseaddr TCP:<vm ip address>:22
   ```

2. **SSH config**:
   ```ssh
   Host fedora-vm
     HostName 127.0.0.1
     Port 2222
     User fedora
     IdentityFile ~/.ssh/utm_ed25519
   ```


</details>

## Testing Strategy

NAMDRunner uses a multi-tier testing approach with unit tests, UI testing, and desktop E2E testing. For complete testing documentation including setup, workflows, and debugging, see [`docs/testing-spec.md`](testing-spec.md).

## Coding Standards

For comprehensive coding standards, architectural patterns, and build configuration, see [`docs/developer-guidelines.md`](developer-guidelines.md).

Key areas covered:
- Clean architecture principles and anti-patterns
- TypeScript/Rust configuration and tooling
- Error handling patterns (`Result<T>`)
- Service development patterns
- Security guidelines for credential handling
- Testing and performance best practices

## Repository Structure

```
/src/                     # Svelte + TS components (no Tauri in here)
/src/lib/ports/coreClient.ts
/src/lib/ports/coreClient-tauri.ts  
/src/lib/ports/coreClient-mock.ts
/src/lib/domain/          # pure logic (parsers, mapping, validation)
/src/lib/fixtures/        # deterministic UI fixtures for tests
/src-tauri/               # Rust: ssh/sftp/slurm/sqlite/templating + commands
/tests/                   # Vitest + Svelte Testing Library
/tests/e2e/               # Linux WebDriver specs (real desktop)
/tests/ui/                # UI testing with agent debug toolkit
/ci/                      # workflows, scripts (xvfb, deps)
```

## First Milestone Implementation Plan

1. **Skeleton app**: Tauri project, Svelte starter, one window, visible "Connect" stub.
2. **IPC port**: define `coreClient` interface; implement **mock** version; wire UI to mock.
3. **Unit/component**: add Vitest + Svelte Testing Library; one component test (wizard field validation).
4. **Desktop E2E scaffold (Linux)**: add `/tests/e2e/` with **WebdriverIO + tauri-driver**; single spec — launch app → click **Connect** → assert banner → **take a screenshot**.
5. **Rust parsers**: stand-alone functions + unit tests for `squeue` / `sacct` sample outputs.
6. **CI**: **Linux** job runs unit/component + desktop E2E under **Xvfb**, uploads screenshots/logs. **Windows** job builds and publishes the **portable `.exe`** artifact.
7. **macOS build path**: verify local build on a Mac (manual smoke only).

## UX Requirements

* Explicit **Connect/Disconnect/Reconnect** controls and visible session state.
* Clear job status with last-polled timestamp.
* Non-blocking status refresh; errors as dismissible banners with retry.

## References

- [WebDriver | Tauri](https://v2.tauri.app/develop/tests/webdriver/)
- [Prerequisites | Tauri](https://v2.tauri.app/start/prerequisites/)
- [Visual comparisons | Playwright](https://playwright.dev/docs/test-snapshots)
- [tauri-apps/webdriver-example](https://github.com/tauri-apps/webdriver-example)
- [WebdriverIO | Tauri](https://v2.tauri.app/develop/tests/webdriver/example/webdriverio/)
- [Playwright](https://playwright.dev/)
- [Continuous Integration | Tauri](https://v2.tauri.app/develop/tests/webdriver/ci/)
- [How to Run Your Tests Headless with Xvfb](https://elementalselenium.com/tips/38-headless)