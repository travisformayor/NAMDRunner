import { defineConfig } from '@playwright/test';

/**
 * Playwright configuration for NAMDRunner
 *
 * NOTE: This configuration is for the Agent Debug Toolkit only.
 * For full E2E testing of the built Tauri app, use WebdriverIO instead.
 * See docs/testing-spec.md for complete testing strategy.
 */

// Check if running in CI environment
const isCI = process.env.CI === 'true' || process.env.CI === '1';

export default defineConfig({
  testDir: './tests/ui', // UI testing with agent debug toolkit
  fullyParallel: false,
  forbidOnly: isCI,
  retries: isCI ? 2 : 0,
  workers: 1,
  timeout: 30000,
  reporter: [
    ['html'],
    ['json', { outputFile: 'test-results/playwright-results.json' }],
    isCI ? ['github'] : ['list']
  ],

  use: {
    // Configuration for web UI testing only
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
    headless: isCI,
    viewport: { width: 1280, height: 720 },
    storageState: undefined, // Fresh state for each test
  },

  projects: [
    {
      name: 'web-ui-tests',
      testDir: './tests/ui',
      use: {
        // For future web UI tests if needed
        // Agent debug toolkit uses its own setup
      },
    },
  ],

  // Output directories
  outputDir: 'tests/ui/test-results/',
  
  // Expect configuration for visual testing
  expect: {
    threshold: 0.2,
    toHaveScreenshot: { threshold: 0.2 },
    toMatchScreenshot: { threshold: 0.2 },
  },
});