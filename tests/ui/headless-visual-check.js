// Headless UI Visual Verification for SSH environments
import { chromium } from '@playwright/test';
import path from 'path';
import fs from 'fs';

const SCREENSHOTS_DIR = './tests/ui/screenshots';

// Ensure screenshots directory exists
if (!fs.existsSync(SCREENSHOTS_DIR)) {
  fs.mkdirSync(SCREENSHOTS_DIR, { recursive: true });
}

class HeadlessUIChecker {
  constructor() {
    this.browser = null;
    this.page = null;
    this.screenshotCount = 0;
  }

  async initialize() {
    console.log('🤖 Starting Headless UI Check...\n');

    // Launch browser in headless mode for SSH environment
    this.browser = await chromium.launch({
      headless: true,  // Important for SSH
      args: ['--no-sandbox', '--disable-setuid-sandbox'] // Security flags for container environments
    });

    const context = await this.browser.newContext({
      viewport: { width: 1280, height: 720 }
    });

    this.page = await context.newPage();

    // Set up console logging
    this.page.on('console', msg => {
      console.log(`   📝 CONSOLE [${msg.type().toUpperCase()}]: ${msg.text()}`);
    });

    this.page.on('pageerror', error => {
      console.log(`   ❌ JS ERROR: ${error.message}`);
    });

    console.log('✅ Headless browser initialized');
  }

  async connectToApp(url = 'http://localhost:1420/') {
    console.log(`🌐 Connecting to ${url}...`);
    try {
      await this.page.goto(url, { waitUntil: 'networkidle', timeout: 10000 });
      await this.screenshot('01-app-loaded');
      console.log('✅ Connected to application');
      return true;
    } catch (error) {
      console.log(`❌ Failed to connect: ${error.message}`);
      return false;
    }
  }

  async screenshot(name) {
    const filename = `${name}.png`;
    const filepath = path.join(SCREENSHOTS_DIR, filename);
    await this.page.screenshot({ path: filepath, fullPage: true });
    console.log(`📸 Screenshot saved: ${filename}`);
    return filename;
  }

  async testNavigationFlow() {
    console.log('\n🧭 Testing Navigation Flow...');

    // Take initial screenshot
    await this.screenshot('02-initial-state');

    // Check if sidebar exists
    const sidebarExists = await this.page.locator('.sidebar, nav').first().isVisible();
    console.log(`   📋 Sidebar visible: ${sidebarExists}`);

    // Look for Jobs button/link
    const jobsButton = this.page.locator('button:has-text("Jobs"), a:has-text("Jobs"), .nav-item:has-text("Jobs")').first();
    const jobsButtonExists = await jobsButton.isVisible();
    console.log(`   📋 Jobs button visible: ${jobsButtonExists}`);

    if (jobsButtonExists) {
      await jobsButton.click();
      await this.page.waitForTimeout(1000);
      await this.screenshot('03-jobs-view');
      console.log('   ✅ Clicked Jobs navigation');
    }

    // Look for Create Job button
    const createButton = this.page.locator('button:has-text("Create"), button:has-text("New Job"), .nav-item:has-text("Create")').first();
    const createButtonExists = await createButton.isVisible();
    console.log(`   ➕ Create button visible: ${createButtonExists}`);

    if (createButtonExists) {
      await createButton.click();
      await this.page.waitForTimeout(1000);
      await this.screenshot('04-create-job-view');
      console.log('   ✅ Clicked Create Job navigation');
    }

    // Test connection dropdown
    const connectionDropdown = this.page.locator('button:has-text("Connect"), .connection-trigger, button:has-text("Disconnect")').first();
    const connectionExists = await connectionDropdown.isVisible();
    console.log(`   🔗 Connection control visible: ${connectionExists}`);

    if (connectionExists) {
      await connectionDropdown.click();
      await this.page.waitForTimeout(1000);
      await this.screenshot('05-connection-dropdown');
      console.log('   ✅ Opened connection dropdown');
    }
  }

  async testJobsTable() {
    console.log('\n📊 Testing Jobs Table...');

    // Navigate to jobs first
    await this.page.goto('http://localhost:1420/', { waitUntil: 'networkidle' });
    await this.page.waitForTimeout(2000);

    // Look for jobs table
    const table = this.page.locator('table, .jobs-table, .table');
    const tableExists = await table.first().isVisible();
    console.log(`   📋 Jobs table visible: ${tableExists}`);

    if (tableExists) {
      // Count rows
      const rows = await this.page.locator('tbody tr, .job-row').count();
      console.log(`   📊 Job rows found: ${rows}`);

      // Look for status badges
      const badges = await this.page.locator('.status-badge, .badge').count();
      console.log(`   🏷️  Status badges found: ${badges}`);

      await this.screenshot('06-jobs-table-detail');
    }
  }

  async testThemeToggle() {
    console.log('\n🌙 Testing Theme Toggle...');

    const themeToggle = this.page.locator('button:has-text("🌙"), button:has-text("☀️"), .theme-toggle');
    const toggleExists = await themeToggle.first().isVisible();
    console.log(`   🎨 Theme toggle visible: ${toggleExists}`);

    if (toggleExists) {
      await themeToggle.first().click();
      await this.page.waitForTimeout(1000);
      await this.screenshot('07-dark-theme');
      console.log('   ✅ Toggled theme');
    }
  }

  async analyzePageElements() {
    console.log('\n🔍 Analyzing Page Elements...');

    const title = await this.page.title();
    const buttons = await this.page.locator('button').count();
    const inputs = await this.page.locator('input').count();
    const links = await this.page.locator('a').count();
    const cards = await this.page.locator('.card, .namd-card').count();

    console.log(`   📝 Page Title: ${title}`);
    console.log(`   🔘 Buttons: ${buttons}`);
    console.log(`   📝 Inputs: ${inputs}`);
    console.log(`   🔗 Links: ${links}`);
    console.log(`   📄 Cards: ${cards}`);

    return { title, buttons, inputs, links, cards };
  }

  async cleanup() {
    console.log('\n📸 Screenshots saved to: tests/ui/screenshots/');
    console.log('   Use these files to verify UI matches mockup design\n');

    if (this.page) await this.page.close();
    if (this.browser) await this.browser.close();

    console.log('✅ Headless UI check complete');
  }
}

// Main execution
async function runHeadlessUICheck() {
  const checker = new HeadlessUIChecker();

  try {
    await checker.initialize();

    // Test basic connectivity
    const connected = await checker.connectToApp();
    if (!connected) {
      console.log('❌ Cannot connect to development server');
      return;
    }

    // Analyze page elements
    await checker.analyzePageElements();

    // Test navigation flow
    await checker.testNavigationFlow();

    // Test jobs table if visible
    await checker.testJobsTable();

    // Test theme toggle
    await checker.testThemeToggle();

    // Take final screenshot
    await checker.screenshot('08-final-state');

  } catch (error) {
    console.log(`❌ Test error: ${error.message}`);
    await checker.screenshot('99-error-state');
  } finally {
    await checker.cleanup();
  }
}

// Run the check
runHeadlessUICheck();