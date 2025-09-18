import { chromium } from '@playwright/test';
import fs from 'fs';
import path from 'path';

const SCREENSHOTS_DIR = './tests/ui/screenshots';

async function simpleUICheck() {
  console.log('🤖 Starting Simple UI Check...');

  // Ensure screenshots directory exists
  if (!fs.existsSync(SCREENSHOTS_DIR)) {
    fs.mkdirSync(SCREENSHOTS_DIR, { recursive: true });
  }

  let browser = null;
  let page = null;

  try {
    // Launch browser
    browser = await chromium.launch({
      headless: true,
      args: ['--no-sandbox', '--disable-dev-shm-usage']
    });

    // Create page
    page = await browser.newPage();

    console.log('📱 Navigating to UI...');

    // Navigate to the development server
    await page.goto('http://localhost:1420', {
      waitUntil: 'networkidle',
      timeout: 30000
    });

    console.log('📸 Taking screenshot...');

    // Take screenshot
    await page.screenshot({
      path: path.join(SCREENSHOTS_DIR, 'ui-check.png'),
      fullPage: true
    });

    // Check if page loaded
    const title = await page.title();
    console.log(`📄 Page title: ${title}`);

    // Check for any console errors
    const errors = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });

    // Wait a bit to catch any immediate errors
    await page.waitForTimeout(2000);

    console.log('✅ UI check complete!');
    console.log(`📸 Screenshot saved to: ${path.join(SCREENSHOTS_DIR, 'ui-check.png')}`);

    if (errors.length > 0) {
      console.log('❌ Console errors detected:');
      errors.forEach(error => console.log(`   - ${error}`));
    } else {
      console.log('✅ No console errors detected');
    }

  } catch (error) {
    console.error('❌ UI check failed:', error);
  } finally {
    if (page) await page.close();
    if (browser) await browser.close();
  }
}

simpleUICheck();