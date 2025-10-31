// @ts-nocheck
import { chromium } from '@playwright/test';
import fs from 'fs';
import path from 'path';

/**
 * Comprehensive Demo Mode Test for NAMDRunner
 *
 * This test script specifically focuses on demo mode functionality:
 * - Demo mode connection/disconnection workflow
 * - Demo mode toggle UI functionality
 * - Jobs interface with mock data
 * - Job creation interface navigation
 * - Job details view functionality
 * - UI responsiveness and navigation
 * - JavaScript error detection
 *
 * IMPORTANT: This script tests ONLY demo mode - no real server connections
 */

const SCREENSHOTS_DIR = './tests/ui/screenshots';
const RESULTS_DIR = './tests/ui/results';

class DemoModeTestSuite {
  constructor() {
    this.browser = null;
    this.page = null;
    this.results = {
      success: true,
      tests: [],
      screenshots: [],
      errors: [],
      consoleLogs: [],
      timestamp: new Date().toISOString(),
      testDuration: 0
    };
    this.startTime = Date.now();
    this.screenshotCount = 0;
  }

  async initialize() {
    console.log('🤖 Starting Demo Mode Comprehensive Test Suite...\n');

    // Ensure directories exist
    [SCREENSHOTS_DIR, RESULTS_DIR].forEach(dir => {
      if (!fs.existsSync(dir)) {
        fs.mkdirSync(dir, { recursive: true });
      }
    });

    // Auto-detect headless mode
    const isHeadless = process.env.SSH_CLIENT || process.env.DISPLAY === ':99' || process.env.CI;

    this.browser = await chromium.launch({
      headless: true, // Force headless for agent testing
      slowMo: 100, // Slow down for better observation
      args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-dev-shm-usage']
    });

    const context = await this.browser.newContext({
      viewport: { width: 1280, height: 720 }
    });

    this.page = await context.newPage();

    // Set up event listeners
    this.page.on('console', msg => {
      const logEntry = {
        type: msg.type(),
        text: msg.text(),
        timestamp: new Date().toISOString()
      };
      this.results.consoleLogs.push(logEntry);

      if (msg.type() === 'error') {
        this.results.errors.push(`Console Error: ${msg.text()}`);
        console.log(`   ❌ CONSOLE ERROR: ${msg.text()}`);
      } else {
        console.log(`   📝 CONSOLE [${msg.type().toUpperCase()}]: ${msg.text()}`);
      }
    });

    this.page.on('pageerror', error => {
      this.results.errors.push(`JavaScript Error: ${error.message}`);
      console.log(`   ❌ JS ERROR: ${error.message}`);
    });

    this.page.on('requestfailed', request => {
      const errorMsg = `Network Error: ${request.url()} - ${request.failure()?.errorText}`;
      this.results.errors.push(errorMsg);
      console.log(`   🌐 NETWORK ERROR: ${request.url()}`);
    });

    console.log('✅ Test suite initialized in headless mode');
  }

  async screenshot(name) {
    const filename = `demo-mode-${String(++this.screenshotCount).padStart(2, '0')}-${name}.png`;
    const filepath = path.join(SCREENSHOTS_DIR, filename);
    await this.page.screenshot({ path: filepath, fullPage: true });
    this.results.screenshots.push(filename);
    console.log(`   📸 Screenshot: ${filename}`);
    return filename;
  }

  async addTest(name, success, details = null) {
    const test = { name, status: success ? 'PASS' : 'FAIL', details };
    this.results.tests.push(test);
    if (!success) this.results.success = false;
    console.log(`   ${success ? '✅' : '❌'} ${name}`);
    return test;
  }

  async loadApplication() {
    console.log('\n1️⃣ Loading NAMDRunner Application...');
    try {
      await this.page.goto('http://localhost:1420', {
        waitUntil: 'networkidle',
        timeout: 30000
      });
      await this.screenshot('app-loaded');
      await this.addTest('Application loads successfully', true);
      return true;
    } catch (error) {
      await this.addTest('Application loads successfully', false, error.message);
      return false;
    }
  }

  async testInitialUIState() {
    console.log('\n2️⃣ Testing Initial UI State...');

    // Check for key UI elements
    const sidebar = await this.page.locator('.sidebar').first().isVisible();
    await this.addTest('Sidebar/Navigation visible', sidebar);

    const connectionStatus = await this.page.locator('.connection-trigger, .connection-status, [data-testid="connection"]').first().isVisible();
    await this.addTest('Connection status component visible', connectionStatus);

    const jobsButton = await this.page.locator('button:has-text("Jobs"), a:has-text("Jobs"), [href*="jobs"]').first().isVisible();
    await this.addTest('Jobs navigation button visible', jobsButton);

    const createButton = await this.page.locator('button:has-text("Create"), button:has-text("New"), [href*="create"]').first().isVisible();
    await this.addTest('Create job button visible', createButton);

    await this.screenshot('initial-ui-state');
  }

  async testDemoModeToggle() {
    console.log('\n3️⃣ Testing Demo Mode Toggle Functionality...');

    try {
      // Look for connection settings or demo mode toggle
      const connectionTrigger = this.page.locator('.connection-trigger, .connection-status, [data-testid="connection"]').first();
      const isVisible = await connectionTrigger.isVisible();

      if (isVisible) {
        await connectionTrigger.click();
        await this.page.waitForTimeout(500);
        await this.screenshot('connection-dropdown-opened');

        // Look for demo mode option
        const demoOption = await this.page.locator('[data-testid="demo-mode"], button:has-text("Demo"), button:has-text("Mock")').isVisible();
        await this.addTest('Demo mode option available in dropdown', demoOption);

        if (demoOption) {
          await this.page.locator('[data-testid="demo-mode"], button:has-text("Demo"), button:has-text("Mock")').first().click();
          await this.page.waitForTimeout(500);
          await this.screenshot('demo-mode-selected');
          await this.addTest('Demo mode can be selected', true);
        }
      } else {
        // Check if already in demo mode or if toggle is elsewhere
        const demoIndicator = await this.page.locator(':has-text("Demo"), :has-text("Mock"), [data-mode="demo"]').isVisible();
        await this.addTest('Demo mode indicator visible', demoIndicator);
      }
    } catch (error) {
      await this.addTest('Demo mode toggle functionality', false, error.message);
    }
  }

  async testDemoModeConnection() {
    console.log('\n4️⃣ Testing Demo Mode Connection Workflow...');

    try {
      // Test connection in demo mode
      const connectButton = this.page.locator('button:has-text("Connect")').first();
      const connectVisible = await connectButton.isVisible();

      if (connectVisible) {
        await connectButton.click();
        await this.page.waitForTimeout(1000);
        await this.screenshot('demo-connect-clicked');

        // In demo mode, connection should be instant or show demo data
        const connectedState = await this.page.locator('[data-testid="connection-status-button"] .status-label:has-text("Connected")').isVisible();
        await this.addTest('Demo mode connection successful', connectedState);

        // Test disconnect
        const disconnectButton = this.page.locator('button:has-text("Disconnect")').first();
        const disconnectVisible = await disconnectButton.isVisible();

        if (disconnectVisible) {
          await disconnectButton.click();
          await this.page.waitForTimeout(500);
          await this.screenshot('demo-disconnected');
          await this.addTest('Demo mode disconnection successful', true);
        }
      } else {
        await this.addTest('Connect button available', false);
      }
    } catch (error) {
      await this.addTest('Demo mode connection workflow', false, error.message);
    }
  }

  async testJobsInterface() {
    console.log('\n5️⃣ Testing Jobs Interface and Mock Data...');

    try {
      // Navigate to jobs page
      const jobsNav = this.page.locator('button:has-text("Jobs"), a:has-text("Jobs"), [href*="jobs"]').first();
      const jobsNavVisible = await jobsNav.isVisible();

      if (jobsNavVisible) {
        await jobsNav.click();
        await this.page.waitForTimeout(1000);
        await this.screenshot('jobs-page-loaded');

        // Check for jobs table or list
        const jobsTable = await this.page.locator('table, .jobs-list, .job-card, [data-testid="jobs"]').isVisible();
        await this.addTest('Jobs table/list visible', jobsTable);

        // Check for mock job data
        const jobEntries = await this.page.locator('tr:has-text("job"), .job-item, .job-card').count();
        await this.addTest('Mock job data present', jobEntries > 0, `Found ${jobEntries} job entries`);

        // Test job details if any jobs exist
        if (jobEntries > 0) {
          const firstJob = this.page.locator('tr:has-text("job"), .job-item, .job-card').first();
          await firstJob.click();
          await this.page.waitForTimeout(500);
          await this.screenshot('job-details-opened');

          const jobDetails = await this.page.locator('.job-details, .modal, [role="dialog"]').isVisible();
          await this.addTest('Job details view opens', jobDetails);
        }
      } else {
        await this.addTest('Jobs navigation available', false);
      }
    } catch (error) {
      await this.addTest('Jobs interface functionality', false, error.message);
    }
  }

  async testJobCreation() {
    console.log('\n6️⃣ Testing Job Creation Interface...');

    try {
      // Navigate to job creation
      const createNav = this.page.locator('button:has-text("Create"), button:has-text("New"), a:has-text("Create"), [href*="create"]').first();
      const createNavVisible = await createNav.isVisible();

      if (createNavVisible) {
        await createNav.click();
        await this.page.waitForTimeout(1000);
        await this.screenshot('create-job-page-loaded');

        // Check for form fields
        const jobNameField = await this.page.locator('input[name="name"], input[name="jobName"], #jobName, #name').isVisible();
        await this.addTest('Job name field present', jobNameField);

        const temperatureField = await this.page.locator('input[name="temperature"], #temperature').isVisible();
        await this.addTest('Temperature field present', temperatureField);

        const partitionField = await this.page.locator('select[name="partition"], #partition').isVisible();
        await this.addTest('Partition field present', partitionField);

        // Test form interaction
        if (jobNameField) {
          await this.page.locator('input[name="name"], input[name="jobName"], #jobName, #name').first().fill('test-demo-job');
          await this.page.waitForTimeout(200);
          await this.screenshot('job-name-filled');
          await this.addTest('Job name field can be filled', true);
        }

        if (temperatureField) {
          await this.page.locator('input[name="temperature"], #temperature').first().fill('310');
          await this.page.waitForTimeout(200);
          await this.addTest('Temperature field can be filled', true);
        }

        await this.screenshot('create-job-form-filled');
      } else {
        await this.addTest('Create job navigation available', false);
      }
    } catch (error) {
      await this.addTest('Job creation interface', false, error.message);
    }
  }

  async testUIResponsiveness() {
    console.log('\n7️⃣ Testing UI Responsiveness and Navigation...');

    try {
      // Test theme toggle if available
      const themeToggle = this.page.locator('.theme-toggle, button:has-text("Dark"), button:has-text("Light")').first();
      const themeToggleVisible = await themeToggle.isVisible();

      if (themeToggleVisible) {
        await themeToggle.click();
        await this.page.waitForTimeout(500);
        await this.screenshot('theme-toggled');
        await this.addTest('Theme toggle functionality', true);
      } else {
        await this.addTest('Theme toggle available', false);
      }

      // Test responsive behavior (simulate mobile viewport)
      await this.page.setViewportSize({ width: 375, height: 667 });
      await this.page.waitForTimeout(500);
      await this.screenshot('mobile-viewport');
      await this.addTest('Mobile viewport rendering', true);

      // Test navigation still works on mobile
      const mobileNavVisible = await this.page.locator('.sidebar').first().isVisible();
      await this.addTest('Mobile navigation accessible', mobileNavVisible);

      // Restore desktop viewport
      await this.page.setViewportSize({ width: 1280, height: 720 });
      await this.page.waitForTimeout(500);
      await this.screenshot('desktop-viewport-restored');

    } catch (error) {
      await this.addTest('UI responsiveness testing', false, error.message);
    }
  }

  async testErrorHandling() {
    console.log('\n8️⃣ Testing Error Handling and Edge Cases...');

    try {
      // Test invalid form submission if form is available
      const submitButton = this.page.locator('button[type="submit"], button:has-text("Submit"), button:has-text("Create")').first();
      const submitVisible = await submitButton.isVisible();

      if (submitVisible) {
        // Try submitting empty form
        await submitButton.click();
        await this.page.waitForTimeout(1000);
        await this.screenshot('empty-form-submission');

        // Check for validation errors
        const validationError = await this.page.locator('[role="alert"]').first().isVisible();
        await this.addTest('Form validation shows errors for empty submission', validationError);
      }

      // Test navigation to non-existent routes
      await this.page.goto('http://localhost:1420/nonexistent');
      await this.page.waitForTimeout(500);
      await this.screenshot('nonexistent-route');

      const errorPage = await this.page.locator(':has-text("404"), :has-text("Not Found"), :has-text("Error")').isVisible();
      await this.addTest('404 page handling', errorPage);

      // Return to main page
      await this.page.goto('http://localhost:1420');
      await this.page.waitForTimeout(500);

    } catch (error) {
      await this.addTest('Error handling testing', false, error.message);
    }
  }

  async generateReport() {
    console.log('\n9️⃣ Generating Comprehensive Test Report...');

    this.results.testDuration = Date.now() - this.startTime;

    // Save detailed results
    const resultsFile = path.join(RESULTS_DIR, 'demo-mode-test-results.json');
    fs.writeFileSync(resultsFile, JSON.stringify(this.results, null, 2));

    // Generate human-readable report
    const report = this.generateHumanReadableReport();
    const reportFile = path.join(RESULTS_DIR, 'demo-mode-test-report.md');
    fs.writeFileSync(reportFile, report);

    console.log(`\n📄 Detailed results: ${resultsFile}`);
    console.log(`📄 Human-readable report: ${reportFile}`);
    console.log(`📸 Screenshots directory: ${SCREENSHOTS_DIR}`);
  }

  generateHumanReadableReport() {
    const passed = this.results.tests.filter(t => t.status === 'PASS').length;
    const failed = this.results.tests.filter(t => t.status === 'FAIL').length;
    const duration = Math.round(this.results.testDuration / 1000);

    let report = `# NAMDRunner Demo Mode Test Report\n\n`;
    report += `**Generated:** ${this.results.timestamp}\n`;
    report += `**Duration:** ${duration} seconds\n`;
    report += `**Overall Status:** ${this.results.success ? '✅ PASS' : '❌ FAIL'}\n\n`;

    report += `## Summary\n\n`;
    report += `- **Total Tests:** ${this.results.tests.length}\n`;
    report += `- **Passed:** ${passed}\n`;
    report += `- **Failed:** ${failed}\n`;
    report += `- **Screenshots:** ${this.results.screenshots.length}\n`;
    report += `- **Errors:** ${this.results.errors.length}\n\n`;

    report += `## Test Results\n\n`;
    this.results.tests.forEach((test, index) => {
      const status = test.status === 'PASS' ? '✅' : '❌';
      report += `${index + 1}. ${status} **${test.name}**\n`;
      if (test.details) {
        report += `   - Details: ${test.details}\n`;
      }
      report += `\n`;
    });

    if (this.results.errors.length > 0) {
      report += `## Errors Detected\n\n`;
      this.results.errors.forEach((error, index) => {
        report += `${index + 1}. ${error}\n`;
      });
      report += `\n`;
    }

    report += `## Screenshots\n\n`;
    this.results.screenshots.forEach((screenshot, index) => {
      report += `${index + 1}. ${screenshot}\n`;
    });

    report += `\n## Recommendations\n\n`;

    if (failed === 0) {
      report += `✅ All tests passed! The demo mode functionality is working well.\n\n`;
    } else {
      report += `❌ ${failed} test(s) failed. Review the following:\n\n`;
      this.results.tests.filter(t => t.status === 'FAIL').forEach(test => {
        report += `- **${test.name}**: ${test.details || 'Needs investigation'}\n`;
      });
      report += `\n`;
    }

    if (this.results.errors.length > 0) {
      report += `⚠️ JavaScript/Console errors detected. These should be addressed for better user experience.\n\n`;
    }

    report += `## Next Steps\n\n`;
    report += `1. Address any failed tests identified above\n`;
    report += `2. Fix JavaScript errors if any were detected\n`;
    report += `3. Consider additional edge case testing\n`;
    report += `4. Verify demo mode data accuracy and completeness\n`;

    return report;
  }

  async cleanup() {
    if (this.page) await this.page.close();
    if (this.browser) await this.browser.close();

    // Print summary
    console.log('\n📊 Demo Mode Test Summary:');
    console.log(`   Overall Status: ${this.results.success ? '✅ PASS' : '❌ FAIL'}`);
    console.log(`   Tests Run: ${this.results.tests.length}`);
    console.log(`   Passed: ${this.results.tests.filter(t => t.status === 'PASS').length}`);
    console.log(`   Failed: ${this.results.tests.filter(t => t.status === 'FAIL').length}`);
    console.log(`   Screenshots: ${this.results.screenshots.length}`);
    console.log(`   Errors: ${this.results.errors.length}`);
    console.log(`   Duration: ${Math.round(this.results.testDuration / 1000)}s`);

    if (this.results.errors.length > 0) {
      console.log('\n❌ Errors detected:');
      this.results.errors.forEach(error => console.log(`   - ${error}`));
    }

    return this.results.success;
  }
}

// Main test execution
async function runDemoModeTests() {
  const testSuite = new DemoModeTestSuite();

  try {
    await testSuite.initialize();

    // Run all test phases
    const appLoaded = await testSuite.loadApplication();
    if (!appLoaded) {
      console.log('❌ Cannot proceed - application failed to load');
      return false;
    }

    await testSuite.testInitialUIState();
    await testSuite.testDemoModeToggle();
    await testSuite.testDemoModeConnection();
    await testSuite.testJobsInterface();
    await testSuite.testJobCreation();
    await testSuite.testUIResponsiveness();
    await testSuite.testErrorHandling();
    await testSuite.generateReport();

    return await testSuite.cleanup();

  } catch (error) {
    console.error('❌ Test suite failed:', error);
    await testSuite.cleanup();
    return false;
  }
}

// Export for use in other scripts
export { DemoModeTestSuite };

// If run directly, execute tests
if (import.meta.url === `file://${process.argv[1]}`) {
  runDemoModeTests();
}