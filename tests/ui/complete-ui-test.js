// Complete UI Test - Test all states including connected mode
import { chromium } from '@playwright/test';
import path from 'path';
import fs from 'fs';

const SCREENSHOTS_DIR = './tests/ui/screenshots';

if (!fs.existsSync(SCREENSHOTS_DIR)) {
  fs.mkdirSync(SCREENSHOTS_DIR, { recursive: true });
}

async function completeUITest() {
  console.log('🎯 Complete UI Test - All States and Navigation\n');

  const browser = await chromium.launch({ headless: true, args: ['--no-sandbox'] });
  const context = await browser.newContext({ viewport: { width: 1280, height: 720 } });
  const page = await context.newPage();

  try {
    // Load the app
    console.log('🌐 Loading application...');
    await page.goto('http://localhost:1420/', { waitUntil: 'networkidle' });
    await page.screenshot({ path: path.join(SCREENSHOTS_DIR, 'test-01-app-loaded.png'), fullPage: true });
    console.log('📸 App loaded screenshot taken');

    // Test connection dropdown
    console.log('\n🔗 Testing connection dropdown...');
    const connectionButton = page.locator('button:has-text("Disconnected"), .connection-trigger');
    if (await connectionButton.isVisible()) {
      await connectionButton.click();
      await page.waitForTimeout(1000);
      await page.screenshot({ path: path.join(SCREENSHOTS_DIR, 'test-02-connection-dropdown.png'), fullPage: true });
      console.log('📸 Connection dropdown screenshot taken');

      // Close dropdown by clicking outside
      await page.click('body');
      await page.waitForTimeout(500);
    }

    // Simulate connection by executing JavaScript to change store state
    console.log('\n⚡ Simulating connection to show jobs table...');
    await page.evaluate(() => {
      // Access the stores and simulate connection
      if (window.sessionStore) {
        window.sessionStore.update(state => ({
          ...state,
          status: 'connected',
          host: 'test.cluster.edu',
          username: 'testuser',
          connectedAt: new Date().toISOString()
        }));
      }
    });

    await page.waitForTimeout(2000); // Wait for reactive updates
    await page.screenshot({ path: path.join(SCREENSHOTS_DIR, 'test-03-connected-state.png'), fullPage: true });
    console.log('📸 Connected state screenshot taken');

    // Test navigation to create job (should now be enabled)
    console.log('\n➕ Testing Create Job navigation...');
    const createButton = page.locator('button:has-text("Create"), .nav-item:has-text("Create")');
    if (await createButton.isEnabled()) {
      await createButton.click();
      await page.waitForTimeout(2000);
      await page.screenshot({ path: path.join(SCREENSHOTS_DIR, 'test-04-create-job-page.png'), fullPage: true });
      console.log('📸 Create Job page screenshot taken');
    }

    // Go back to jobs
    console.log('\n📋 Back to Jobs view...');
    const jobsButton = page.locator('button:has-text("Jobs"), .nav-item:has-text("Jobs")');
    await jobsButton.click();
    await page.waitForTimeout(2000);
    await page.screenshot({ path: path.join(SCREENSHOTS_DIR, 'test-05-jobs-with-table.png'), fullPage: true });
    console.log('📸 Jobs table screenshot taken');

    // Test theme toggle
    console.log('\n🌙 Testing theme toggle...');
    const themeToggle = page.locator('button:has-text("🌙"), button:has-text("☀️")');
    if (await themeToggle.isVisible()) {
      await themeToggle.click();
      await page.waitForTimeout(1000);
      await page.screenshot({ path: path.join(SCREENSHOTS_DIR, 'test-06-dark-theme.png'), fullPage: true });
      console.log('📸 Dark theme screenshot taken');
    }

    // Test SSH console
    console.log('\n💻 Testing SSH console...');
    const consoleToggle = page.locator('button:has-text("SSH Console"), .toggle-button');
    if (await consoleToggle.isVisible()) {
      await consoleToggle.click();
      await page.waitForTimeout(1000);
      await page.screenshot({ path: path.join(SCREENSHOTS_DIR, 'test-07-ssh-console.png'), fullPage: true });
      console.log('📸 SSH console screenshot taken');
    }

    // Test clicking on a job row to see job detail
    console.log('\n📄 Testing job detail view...');
    const jobRow = page.locator('tbody tr, .job-row').first();
    if (await jobRow.isVisible()) {
      await jobRow.click();
      await page.waitForTimeout(2000);
      await page.screenshot({ path: path.join(SCREENSHOTS_DIR, 'test-08-job-detail.png'), fullPage: true });
      console.log('📸 Job detail screenshot taken');
    }

    // Test responsive design
    console.log('\n📱 Testing responsive design...');
    await page.setViewportSize({ width: 768, height: 1024 });
    await page.waitForTimeout(1000);
    await page.screenshot({ path: path.join(SCREENSHOTS_DIR, 'test-09-tablet-view.png'), fullPage: true });

    await page.setViewportSize({ width: 375, height: 667 });
    await page.waitForTimeout(1000);
    await page.screenshot({ path: path.join(SCREENSHOTS_DIR, 'test-10-mobile-view.png'), fullPage: true });
    console.log('📸 Responsive design screenshots taken');

    console.log('\n✅ Complete UI test finished successfully!');
    console.log('📁 All screenshots saved to: tests/ui/screenshots/');

  } catch (error) {
    console.log(`❌ Test error: ${error.message}`);
    await page.screenshot({ path: path.join(SCREENSHOTS_DIR, 'test-99-error.png'), fullPage: true });
  } finally {
    await browser.close();
  }
}

completeUITest();