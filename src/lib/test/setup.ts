import { vi } from 'vitest';
import { testDataManager, type TestScenario } from './fixtures/testDataManager';

// Mock Tauri APIs for testing
const mockInvoke = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}));

// Make mock available globally for tests
(globalThis as any).__TAURI_INVOKE__ = mockInvoke;

// Global test configuration
export const testConfig = {
  // Default timeouts for different operations
  timeouts: {
    connection: 5000,
    fileOperation: 10000,
    jobOperation: 15000,
    ui: 3000,
  },

  // Default test scenario for unit tests
  defaultScenario: 'clean_slate' as TestScenario,

  // Mock delays (can be overridden in tests)
  mockDelays: {
    enabled: true,
    multiplier: 0.1, // Speed up delays for testing (10% of normal)
  },

  // Test data isolation
  isolation: {
    resetBetweenTests: true,
    clearMockState: true,
  },
};

// Test setup functions
export function setupTestEnvironment(): void {
  // Set up test data manager with default scenario
  testDataManager.setScenario(testConfig.defaultScenario);

  // Configure any global test settings
  if (typeof globalThis !== 'undefined') {
    // Global environment setup
    (globalThis as any).__NAMDRUNNER_TEST_MODE__ = true;
    (globalThis as any).__TEST_DATA_MANAGER__ = testDataManager;
  }

  // Set up console overrides for quieter tests
  if (process.env.NODE_ENV === 'test') {
    const originalWarn = console.warn;
    console.warn = (...args) => {
      // Suppress known development warnings during tests
      const message = args[0]?.toString() || '';
      if (message.includes('dev mode') || message.includes('mock mode')) {
        return;
      }
      originalWarn(...args);
    };
  }
}

export function teardownTestEnvironment(): void {
  // Reset test data manager
  testDataManager.reset();

  // Clean up any global test state
  if (typeof globalThis !== 'undefined') {
    delete (globalThis as any).__NAMDRUNNER_TEST_MODE__;
    delete (globalThis as any).__TEST_DATA_MANAGER__;
  }
}

// Test utilities
export function withTestScenario<T>(
  scenario: TestScenario,
  testFn: () => T | Promise<T>
): Promise<T> {
  const originalScenario = testDataManager.getCurrentScenario();
  
  testDataManager.setScenario(scenario);
  
  const result = testFn();
  
  if (result instanceof Promise) {
    return result.finally(() => {
      testDataManager.setScenario(originalScenario);
    });
  }
  
  testDataManager.setScenario(originalScenario);
  return Promise.resolve(result);
}

// Mock timing utilities
export function getMockDelay(baseDuration: number): number {
  if (!testConfig.mockDelays.enabled) {
    return 0;
  }
  
  return Math.max(1, baseDuration * testConfig.mockDelays.multiplier);
}

export function enableFastMocks(): void {
  testConfig.mockDelays.enabled = false;
}

export function enableRealisticMocks(): void {
  testConfig.mockDelays.enabled = true;
  testConfig.mockDelays.multiplier = 1;
}

// Reset mocks and test data before each test
beforeEach(() => {
  vi.clearAllMocks();
  
  if (testConfig.isolation.resetBetweenTests) {
    testDataManager.reset();
  }
  
  if (testConfig.isolation.clearMockState) {
    setupTestEnvironment();
  }
});

// Export test fixtures for easy importing
export * from './fixtures/jobFixtures';
export * from './fixtures/sessionFixtures';
export * from './fixtures/fileFixtures';
export * from './fixtures/slurmFixtures';
export * from './fixtures/testDataManager';