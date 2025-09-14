# Agent Development Capabilities - NAMDRunner

Quick reference for autonomous development tools and infrastructure available in this project.

## Testing Infrastructure

### UI Testing - `npm run test:ui`
**Purpose**: Fast UI development with visual feedback  
**Target**: Web browser via Vite dev server (`localhost:1420`)  
**Speed**: Very fast (no build required)

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

## Quick Troubleshooting

**UI tests failing**: Check Vite dev server is running (`npm run dev`)  
**E2E tests failing**: Verify Tauri build succeeded and WebKit driver installed  
**Mock data issues**: Check `testDataManager.ts` scenario selection  
**IPC errors**: Use E2E tests to debug TypeScript ↔ Rust communication

**For complete testing documentation**: See `docs/testing-spec.md` for comprehensive testing strategies, setup instructions, debugging workflows, and CI pipeline configuration.