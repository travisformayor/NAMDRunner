# Agent Development Capabilities - NAMDRunner

Quick reference for autonomous development tools and infrastructure available in this project.

## Testing Infrastructure

### UI Testing - `npm run test:ui`
**Purpose**: Fast UI development with visual feedback
**Target**: Web browser via Vite dev server (`localhost:1420`)
**Speed**: Very fast (no build required)
**⚠️ Server Startup**: Vite dev server takes 1-3 minutes on first start

**Available Tools**:
- **Playwright**: Browser automation for component testing
- **Agent Debug Toolkit**: Visual debugging with screenshots 
- **Mock Backend**: Full mock data without Rust dependency
- **Live Reload**: Hot module replacement during development

**Mock Data Available**:
- Job fixtures with all lifecycle states (pending, running, completed, failed)
- Session/connection scenarios (success, auth failures, network issues)  
- File operation mocks (upload/download success/failure)
- SLURM command responses for testing

### E2E Testing - `npm run test:e2e`
**Purpose**: Full desktop app integration testing  
**Target**: Built Tauri binary with complete Rust backend  
**Speed**: Slower (requires ~5min build, run as background process)

**Available Tools**:
- **WebdriverIO**: Desktop app automation
- **tauri-driver**: Native Tauri WebDriver integration
- **Full IPC Testing**: TypeScript ↔ Rust boundary validation
- **Debug Logging**: Comprehensive test execution feedback

## Development Commands

### Most Frequently Used
```bash
npm run dev             # Vite dev server for UI work
npm run test            # Unit tests (Vitest + Rust)
npm run tauri dev       # Complete app with hot reload
npm run test:ui         # Fast UI testing/debugging
```

### Complete Command Set
```bash
# Frontend Development
npm run dev             # Svelte dev server (Vite) - localhost:1420
npm run build           # Build frontend static files
npm run preview         # Preview built frontend
npm run lint            # ESLint + Prettier
npm run check           # Svelte/TypeScript checking

# Backend Development  
cargo test              # Rust unit tests
cargo clippy            # Rust linting
cargo fmt               # Rust formatting

# Full Application
npm run tauri dev       # Desktop app with hot reload
npm run tauri build     # Production binary build

# Testing
npm run test            # Vitest unit tests
npm run test:ui         # UI testing (Playwright + Vite)
npm run test:e2e        # E2E testing (WebdriverIO + Tauri)
```

## Project Structure & Integration

### TypeScript ↔ Rust IPC
- **Commands**: Defined in `src-tauri/src/commands/`
- **Types**: Shared types in `src-tauri/src/types/`
- **Frontend Invocation**: `invoke('command_name', { params })`
- **Testing**: Use E2E tests for IPC boundary validation

### Mock Data System
- **Location**: `src/lib/test/fixtures/`
- **Scenarios**: `testDataManager.ts` - coordinated test scenarios
- **Job States**: `jobFixtures.ts` - complete job lifecycle
- **Sessions**: `sessionFixtures.ts` - connection scenarios  
- **Files**: `fileFixtures.ts` - upload/download operations
- **SLURM**: `slurmFixtures.ts` - command responses

### Development Workflow
1. **UI First**: Use `npm run dev` + `npm run test:ui` for component work
2. **Integration**: Use `npm run tauri dev` when testing IPC
3. **Full Testing**: Use `npm run test:e2e` for release validation

## Agent-Specific Capabilities

### Visual Debugging
- Screenshots automatically captured during UI tests
- Agent debug toolkit provides interactive exploration
- Test results saved to `tests/ui/screenshots/`

### Mock Backend Development  
- No Rust compilation required for UI development
- Complete mock API responses available
- Realistic test data scenarios pre-configured

### Autonomous Testing
- Self-contained test suites for both UI and E2E
- Debug output shows exact test execution steps
- Mock data covers all major application scenarios

## Environment Setup
```bash
# Required for development
source ~/.cargo/env     # Rust environment (always run first)
export DISPLAY=:99      # For SSH/headless testing

# Optional for advanced E2E testing
cargo install tauri-driver --locked    # WebDriver for desktop testing
```

## When to Use What

| Scenario | Tool | Command | Speed |
|----------|------|---------|--------|
| UI component work | Playwright + Vite | `npm run test:ui` | Fast |  
| Form/dialog testing | Agent Debug Toolkit | `npm run test:ui` | Fast |
| IPC boundary testing | WebdriverIO + Tauri | `npm run test:e2e` | Slow |
| Release validation | Full E2E suite | `npm run test:e2e` | Slow |
| Unit testing | Vitest + Rust | `npm run test` | Fast |

## SSH/Headless Environment Notes

### For Agent Development Sessions
- **Always use headless mode**: `chromium.launch({ headless: true })`
- **Server startup**: Wait 60-180 seconds for Vite dev server before testing
- **Port checking**: Use `curl -s http://localhost:1420` to verify server readiness
- **Background processes**: Run `npm run dev &` in background, wait for "VITE ready" message

### Headless Browser Configuration
```javascript
// Correct configuration for SSH environments
const browser = await chromium.launch({
  headless: true,  // Required for SSH
  args: ['--no-sandbox', '--disable-setuid-sandbox']
});
```

## Quick Troubleshooting

**UI tests failing**: Check Vite dev server is running (`npm run dev`) and wait for full startup
**Server not responding**: Wait 1-3 minutes for initial Vite startup, check with `curl localhost:1420`
**Headless browser issues**: Ensure `headless: true` and proper security args in Playwright config
**E2E tests failing**: Verify Tauri build succeeded and WebKit driver installed
**Mock data issues**: Check `testDataManager.ts` scenario selection
**IPC errors**: Use E2E tests to debug TypeScript ↔ Rust communication

## Testing Infrastructure Details

### Mock Data Patterns

**SLURM Command Responses**:
```rust
const MOCK_SQUEUE_RUNNING: &str = "12345678|test_job|R|00:15:30|01:44:30|1|24|16GB|amilan|/scratch/alpine/testuser/namdrunner_jobs/test_job";
const MOCK_SACCT_COMPLETED: &str = "12345678|test_job|COMPLETED|0:0|2025-01-15T10:00:00|2025-01-15T11:00:00|01:00:00|/scratch/alpine/testuser/namdrunner_jobs/test_job";
const MOCK_SBATCH_SUCCESS: &str = "Submitted batch job 12345678";
```

**Job Lifecycle States**:
- **CREATED** - Job exists locally but not submitted
- **PENDING** - Submitted to SLURM, waiting for resources
- **RUNNING** - Executing on cluster
- **COMPLETED** - Finished successfully
- **FAILED** - Job failed with error
- **CANCELLED** - User or system cancelled job

### CI Testing Pipeline

**Linux CI Job**:
1. Run unit tests (Vitest + Cargo)
2. Run fast UI tests with agent debug toolkit
3. Build Tauri application (`--debug` for testing)
4. Install `tauri-driver` and WebKit prerequisites
5. Run WebdriverIO E2E tests under Xvfb
6. Upload screenshots, test results, and logs as artifacts

**Windows CI Job**:
1. Build portable `.exe` file
2. Publish as release artifact
3. No desktop E2E testing required

### Debug Environment Setup

**UI Testing Setup (Playwright + Vite)**:
```bash
Xvfb :99 -screen 0 1280x720x24 &  # Virtual display for SSH/CI environments
export DISPLAY=:99
npm run dev                        # Start Vite dev server (localhost:1420)
npm run test:ui                    # Agent debugging toolkit with screenshots
```

**E2E Testing Setup (WebdriverIO + tauri-driver)**:
```bash
# Prerequisites (one-time setup)
cargo install tauri-driver --locked     # WebDriver for Tauri apps
which WebKitWebDriver                   # Verify WebKit driver exists

# Run E2E tests (automatically builds Tauri binary)
export DISPLAY=:99                      # Virtual display if needed
npm run test:e2e                        # Complete desktop app testing
```

### Logging Configuration

**Development Logging Setup**:
```rust
// In main.rs
use log::LevelFilter;

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Debug)
        .init();

    // Rest of main function
}
```

**For focused testing principles**: See [`docs/CONTRIBUTING.md#testing-strategy`](../CONTRIBUTING.md#testing-strategy) for business logic testing philosophy and patterns.