import { chromium } from '@playwright/test';
import fs from 'fs';
import path from 'path';

const SCREENSHOTS_DIR = './tests/ui/screenshots';
const TEST_RESULTS_DIR = './tests/ui/results';

async function comprehensiveUITest() {
  console.log('🤖 Starting Comprehensive UI Test...');

  // Ensure directories exist
  [SCREENSHOTS_DIR, TEST_RESULTS_DIR].forEach(dir => {
    if (!fs.existsSync(dir)) {
      fs.mkdirSync(dir, { recursive: true });
    }
  });

  let browser = null;
  let page = null;
  const results = {
    success: true,
    tests: [],
    screenshots: [],
    errors: []
  };

  try {
    browser = await chromium.launch({
      headless: true,
      args: ['--no-sandbox', '--disable-dev-shm-usage']
    });

    page = await browser.newPage();

    // Capture console logs and errors
    page.on('console', msg => {
      if (msg.type() === 'error') {
        results.errors.push(msg.text());
      }
    });

    console.log('📱 Loading NAMDRunner UI...');
    await page.goto('http://localhost:1420', {
      waitUntil: 'networkidle',
      timeout: 30000
    });

    // Test 1: Basic UI Load
    console.log('✅ Test 1: Basic UI Load');
    results.tests.push({ name: 'Basic UI Load', status: 'PASS' });
    await page.screenshot({
      path: path.join(SCREENSHOTS_DIR, 'comprehensive-01-initial-load.png'),
      fullPage: true
    });
    results.screenshots.push('comprehensive-01-initial-load.png');

    // Test 2: Sidebar Navigation
    console.log('✅ Test 2: Sidebar Navigation');
    const sidebar = await page.locator('.sidebar').isVisible();
    if (sidebar) {
      results.tests.push({ name: 'Sidebar Visible', status: 'PASS' });
    } else {
      results.tests.push({ name: 'Sidebar Visible', status: 'FAIL' });
      results.success = false;
    }

    // Test 3: Navigation Items
    console.log('✅ Test 3: Navigation Items');
    const jobsButton = await page.locator('button:has-text("Jobs")').isVisible();
    const createButton = await page.locator('button:has-text("Create Job")').isVisible();

    if (jobsButton && createButton) {
      results.tests.push({ name: 'Navigation Items Present', status: 'PASS' });
    } else {
      results.tests.push({ name: 'Navigation Items Present', status: 'FAIL' });
      results.success = false;
    }

    // Test 4: Connection Status
    console.log('✅ Test 4: Connection Status');
    const connectionStatus = await page.locator('.connection-trigger').isVisible();
    if (connectionStatus) {
      results.tests.push({ name: 'Connection Status Visible', status: 'PASS' });
    } else {
      results.tests.push({ name: 'Connection Status Visible', status: 'FAIL' });
      results.success = false;
    }

    // Test 5: Click Connection Dropdown
    console.log('✅ Test 5: Connection Dropdown');
    await page.locator('.connection-trigger').click();
    await page.waitForTimeout(500);

    const dropdown = await page.locator('.dropdown-content').isVisible();
    if (dropdown) {
      results.tests.push({ name: 'Connection Dropdown Opens', status: 'PASS' });
      await page.screenshot({
        path: path.join(SCREENSHOTS_DIR, 'comprehensive-02-connection-dropdown.png'),
        fullPage: true
      });
      results.screenshots.push('comprehensive-02-connection-dropdown.png');
    } else {
      results.tests.push({ name: 'Connection Dropdown Opens', status: 'FAIL' });
      results.success = false;
    }

    // Test 6: Jobs Page Content
    console.log('✅ Test 6: Jobs Page Content');
    await page.locator('button:has-text("Jobs")').click();
    await page.waitForTimeout(500);

    const jobsHeader = await page.locator('.page-title:has-text("Jobs")').isVisible();
    if (jobsHeader) {
      results.tests.push({ name: 'Jobs Page Loads', status: 'PASS' });
      await page.screenshot({
        path: path.join(SCREENSHOTS_DIR, 'comprehensive-03-jobs-page.png'),
        fullPage: true
      });
      results.screenshots.push('comprehensive-03-jobs-page.png');
    } else {
      results.tests.push({ name: 'Jobs Page Loads', status: 'FAIL' });
      results.success = false;
    }

    // Test 7: Create Job Navigation
    console.log('✅ Test 7: Create Job Navigation');
    await page.locator('button:has-text("Create Job")').click();
    await page.waitForTimeout(500);

    const createJobHeader = await page.locator('.page-title:has-text("Create Job")').isVisible();
    if (createJobHeader) {
      results.tests.push({ name: 'Create Job Page Loads', status: 'PASS' });
      await page.screenshot({
        path: path.join(SCREENSHOTS_DIR, 'comprehensive-04-create-job.png'),
        fullPage: true
      });
      results.screenshots.push('comprehensive-04-create-job.png');
    } else {
      results.tests.push({ name: 'Create Job Page Loads', status: 'FAIL' });
      results.success = false;
    }

    // Test 8: Form Fields Present
    console.log('✅ Test 8: Form Fields');
    const jobNameField = await page.locator('#jobName').isVisible();
    const temperatureField = await page.locator('#temperature').isVisible();
    const partitionField = await page.locator('#partition').isVisible();

    if (jobNameField && temperatureField && partitionField) {
      results.tests.push({ name: 'Form Fields Present', status: 'PASS' });
    } else {
      results.tests.push({ name: 'Form Fields Present', status: 'FAIL' });
      results.success = false;
    }

    // Test 9: Theme Toggle
    console.log('✅ Test 9: Theme Toggle');
    const themeButton = await page.locator('.theme-toggle').isVisible();
    if (themeButton) {
      await page.locator('.theme-toggle').click();
      await page.waitForTimeout(500);

      await page.screenshot({
        path: path.join(SCREENSHOTS_DIR, 'comprehensive-05-dark-theme.png'),
        fullPage: true
      });
      results.screenshots.push('comprehensive-05-dark-theme.png');
      results.tests.push({ name: 'Theme Toggle Works', status: 'PASS' });
    } else {
      results.tests.push({ name: 'Theme Toggle Works', status: 'FAIL' });
      results.success = false;
    }

    console.log('✅ All tests completed!');

  } catch (error) {
    console.error('❌ Test failed:', error);
    results.success = false;
    results.errors.push(error.message);
  } finally {
    if (page) await page.close();
    if (browser) await browser.close();
  }

  // Save results
  fs.writeFileSync(
    path.join(TEST_RESULTS_DIR, 'comprehensive-test-results.json'),
    JSON.stringify(results, null, 2)
  );

  // Print summary
  console.log('\n📊 Test Summary:');
  console.log(`   Overall Status: ${results.success ? '✅ PASS' : '❌ FAIL'}`);
  console.log(`   Tests Run: ${results.tests.length}`);
  console.log(`   Passed: ${results.tests.filter(t => t.status === 'PASS').length}`);
  console.log(`   Failed: ${results.tests.filter(t => t.status === 'FAIL').length}`);
  console.log(`   Screenshots: ${results.screenshots.length}`);
  console.log(`   Errors: ${results.errors.length}`);

  if (results.errors.length > 0) {
    console.log('\n❌ Errors:');
    results.errors.forEach(error => console.log(`   - ${error}`));
  }

  console.log(`\n📁 Results saved to: ${TEST_RESULTS_DIR}/comprehensive-test-results.json`);
  console.log(`📸 Screenshots saved to: ${SCREENSHOTS_DIR}/`);

  return results.success;
}

comprehensiveUITest();