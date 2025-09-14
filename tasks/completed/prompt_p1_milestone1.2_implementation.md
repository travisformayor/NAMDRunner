# Phase 1 Milestone 1.2 Implementation Prompt for Claude Code Agent

## Mission: Complete Mock Infrastructure for NAMDRunner

You are implementing **Phase 1 Milestone 1.2** of NAMDRunner, building on the solid foundation from Milestone 1.1. Your mission is to create comprehensive mock infrastructure that enables complete offline development and robust testing capabilities.

## 🎯 Core Objectives

### Primary Goals
1. **Enhanced Test Data Management** - Create utilities for consistent mock data handling
2. **Comprehensive E2E Testing** - Set up WebDriver testing for the Tauri app
3. **CI/CD Pipeline** - Configure automated builds for Linux and Windows
4. **Developer Experience** - Make offline development seamless and reliable

### Success Definition
By completion, developers should be able to:
- Run complete test suites offline with realistic mock data
- Test all major user flows without connecting to real clusters
- Get fast feedback from automated CI builds
- Easily switch between different test scenarios

## 📋 Required Reading (Critical - Read First!)

### Essential Context
- `tasks/active/phase1-milestone1.2-mock-infrastructure.md` - **YOUR DETAILED TASK PLAN**
- `tasks/completed/phase1-milestone1.1-foundation.md` - What's already built
- `docs/architecture.md` - Current implementation status
- `docs/technical-spec.md` - Development standards and patterns

### Reference Implementation  
- `docs/reference/python-implementation-reference.md` - Testing insights and patterns from Python version
- `docs/data-spec.md` - Mock data should match these schemas
- `docs/testing-spec.md` - Testing strategy and approaches

### Current Codebase Understanding
**Frontend Mock System** (already working):
- `src/lib/ports/coreClient-mock.ts` - Comprehensive mock client
- `src/lib/ports/clientFactory.ts` - Smart dev/prod switching
- `src/lib/stores/session.test.ts` - Example unit test

**Backend Mock System** (already working):
- `src-tauri/src/commands/connection.rs` - Mock connection handlers
- `src-tauri/src/commands/jobs.rs` - Mock job management  
- `src-tauri/src/commands/files.rs` - Mock file operations

**IPC**
- `tasks/phase1-interface-definitions.md` - Complete IPC interface contracts

## 🗺️ Implementation Roadmap

### Phase A: Test Data Management (Priority 1)
**Goal**: Create consistent, realistic test data across all mocks

#### A1. Test Fixture System
```typescript
// Create: src/lib/test/fixtures/
- jobFixtures.ts          // Job lifecycle scenarios
- sessionFixtures.ts      // Connection scenarios  
- fileFixtures.ts         // File system responses
- slurmFixtures.ts        // SLURM response patterns
- testDataManager.ts      // Central fixture coordinator
```

**Key Requirements**:
- Use realistic data (proper timestamps, job IDs, file sizes)
- Cover edge cases (long job names, special characters, large files)
- Include failure scenarios (network timeouts, auth errors)
- Make data deterministic for consistent tests

#### A2. Mock State Enhancement
```rust
// Enhance: src-tauri/src/commands/
- Update connection.rs with better state management
- Enhance jobs.rs with realistic job progressions  
- Improve files.rs with proper file simulation
```

**Key Requirements**:
- State persists across app restarts (for testing)
- Jobs progress realistically over time
- File operations feel authentic
- Error conditions are properly simulated

### Phase B: E2E Testing Infrastructure (Priority 2)  
**Goal**: Complete user flow testing with WebDriver

#### B1. WebDriver Setup
```bash
# Update: tests/e2e/
- Configure Playwright for Tauri WebDriver
- Create page object models for main components
- Set up test environment isolation
- Add screenshot capabilities
```

#### B2. Core Test Scenarios
```javascript
// Create comprehensive E2E tests:
tests/e2e/
├── connection/
│   ├── connect-success.spec.js
│   ├── connect-failure.spec.js  
│   └── disconnect.spec.js
├── jobs/
│   ├── job-lifecycle.spec.js
│   ├── job-management.spec.js
│   └── job-sync.spec.js  
└── ui/
    ├── responsive-layout.spec.js
    ├── error-handling.spec.js
    └── accessibility.spec.js
```

**Critical Success Factors**:
- Tests must be reliable (no flakiness)
- Cover happy path AND error scenarios
- Use realistic mock data from Phase A
- Include visual regression testing where appropriate

### Phase C: CI/CD Pipeline (Priority 3)
**Goal**: Automated builds and quality gates

#### C1. GitHub Actions Configuration
```yaml
# Create: .github/workflows/ci.yml
- Frontend: TypeScript compilation, linting, unit tests
- Backend: Rust compilation, clippy, unit tests  
- E2E: Full application testing with mocks
- Builds: Linux and Windows Tauri builds
```

#### C2. Quality Gates
- All TypeScript must compile without errors
- All Rust code must pass clippy without warnings  
- Test coverage >80% where applicable
- E2E tests must pass on both platforms
- Build artifacts must be functional

## 🔧 Technical Implementation Guidelines

### Mock Data Realism
```typescript
// Use realistic patterns:
const mockJobId = `job_${timestamp}_${randomString(4)}`; // job_20250905_af3x
const mockSlurmId = `${baseNumber + Math.floor(Math.random() * 1000000)}`; // 12345678
const mockTimestamp = new Date().toISOString(); // Proper ISO format
const mockFileSize = Math.floor(Math.random() * 1000000) + 1024; // Realistic sizes
```

### State Management Patterns
```rust
// Rust mock state should be:
lazy_static! {
    static ref MOCK_STATE: Mutex<MockStateManager> = Mutex::new(MockStateManager::new());
}

// With proper cleanup and reset capabilities
impl MockStateManager {
    pub fn reset_to_clean_state(&mut self) { /* ... */ }
    pub fn load_test_scenario(&mut self, scenario: TestScenario) { /* ... */ }
    pub fn advance_job_states(&mut self) { /* ... */ }
}
```

### E2E Testing Patterns
```javascript
// Page Object Model approach:
class ConnectionPage {
  async connectWithCredentials(host, username, password) {
    await this.page.locator('#host').fill(host);
    await this.page.locator('#username').fill(username); 
    await this.page.locator('#password').fill(password);
    await this.page.locator('button[type="submit"]').click();
  }
  
  async waitForConnectionStatus(expectedStatus) {
    await expect(this.page.locator('.status-text')).toContainText(expectedStatus);
  }
}
```

## ⚠️ Critical Implementation Notes

### Tauri-Specific Considerations
- WebDriver integration requires specific Tauri configuration
- File dialogs need proper mocking for E2E tests
- Window management may behave differently in test vs development
- Platform differences must be handled in CI

### Mock Behavior Requirements
- **Deterministic**: Same inputs always produce same outputs (for tests)
- **Fast**: Tests should run quickly without unnecessary delays
- **Stateful**: Changes persist appropriately within test scenarios
- **Resettable**: Clean state for each test run

### Quality Standards
- All new code must include appropriate tests
- Mock scenarios must cover both success and failure paths
- Documentation must be updated for any new patterns
- Error messages must be helpful for debugging

## 🚨 Common Pitfalls to Avoid

### Testing Antipatterns
- **Don't**: Create flaky tests that sometimes fail
- **Don't**: Test implementation details instead of user behavior  
- **Don't**: Write tests that depend on external systems
- **Don't**: Ignore test setup/teardown - leads to test pollution

### Mock Implementation Issues
- **Don't**: Make mocks too simple (unrealistic)
- **Don't**: Make mocks too complex (hard to maintain)
- **Don't**: Forget to handle error scenarios
- **Don't**: Create inconsistent mock behavior across components

### CI/CD Problems
- **Don't**: Skip testing the CI pipeline locally first
- **Don't**: Create CI jobs that take too long (>10 minutes total)
- **Don't**: Ignore platform-specific issues
- **Don't**: Forget to cache dependencies for speed

## 📚 Implementation Resources

### Key Files to Reference
- **Existing Mock Implementation**: `src/lib/ports/coreClient-mock.ts`
- **Test Example**: `src/lib/stores/session.test.ts`  
- **Rust Mock Commands**: `src-tauri/src/commands/*.rs`
- **Current E2E Test**: `tests/e2e/basic.spec.js`

### External Documentation
- [Playwright Tauri Testing](https://tauri.app/v1/guides/testing/webdriver/introduction)
- [GitHub Actions for Rust](https://github.com/actions-rs)
- [Vitest Testing Framework](https://vitest.dev)
- [Tauri Build Configuration](https://tauri.app/v1/guides/building/)

## 🎯 Milestone Completion Checklist

### Implementation Complete
- [ ] Test fixture system created with realistic data
- [ ] Mock state management enhanced for consistency
- [ ] WebDriver E2E tests cover major user flows
- [ ] CI pipeline builds successfully on Linux and Windows
- [ ] All existing tests continue to pass
- [ ] New test coverage meets quality standards

### Documentation Updated
- [ ] Test scenarios documented with examples
- [ ] CI troubleshooting guide created
- [ ] Mock data patterns documented  
- [ ] E2E testing approach documented

### Quality Validation  
- [ ] All acceptance criteria from task plan validated
- [ ] Mock behavior matches realistic usage patterns
- [ ] CI pipeline demonstrably catches common errors
- [ ] Test suite runs reliably without flakiness
- [ ] Build artifacts verified functional

## 🚀 Getting Started

### Step 1: Environment Setup
```bash
cd /media/share/namdrunner
npm ci                    # Ensure dependencies current
npm run test             # Verify existing tests pass
npm run tauri dev        # Confirm app still works
```

### Step 2: Create Your Implementation Plan
1. Read the detailed task plan thoroughly
2. Create specific subtasks for each phase
3. Plan your approach for WebDriver integration
4. Get approval for your implementation approach

### Step 3: Begin Implementation
- Start with Phase A (Test Data Management) 
- Build incrementally and test each piece
- Update documentation as you implement
- Ask for help with any Tauri-specific challenges

## 💡 Success Tips

- **Start Simple**: Get basic WebDriver working before complex scenarios
- **Test Frequently**: Run tests after each major change
- **Use Existing Patterns**: Follow established mock patterns from Milestone 1.1  
- **Document Issues**: Keep notes on any Tauri/testing challenges encountered
- **Focus on Reliability**: Better to have fewer reliable tests than many flaky ones

Remember: This milestone creates the testing foundation for all future development. Quality and reliability here pays dividends throughout the entire project!

Good luck! 🍀
