// Simple demo mode test to verify browser setup
import { chromium } from '@playwright/test';
import fs from 'fs';
import path from 'path';

const SCREENSHOTS_DIR = './tests/ui/screenshots';

async function simpleDemoTest() {
  console.log('🤖 Starting Simple Demo Mode Test...');

  // Ensure directory exists
  if (!fs.existsSync(SCREENSHOTS_DIR)) {
    fs.mkdirSync(SCREENSHOTS_DIR, { recursive: true });
  }

  let browser = null;
  let page = null;

  try {
    console.log('🌐 Launching browser...');
    browser = await chromium.launch({
      headless: true,
      args: ['--no-sandbox', '--disable-setuid-sandbox']
    });

    console.log('📱 Creating page...');
    page = await browser.newPage();

    console.log('🔗 Connecting to app...');
    await page.goto('http://localhost:1420', {
      waitUntil: 'networkidle',
      timeout: 30000
    });

    console.log('📸 Taking screenshot...');
    await page.screenshot({
      path: path.join(SCREENSHOTS_DIR, 'simple-demo-test.png'),
      fullPage: true
    });

    console.log('🔍 Checking basic elements...');
    const title = await page.title();
    console.log(`   Title: ${title}`);

    const buttons = await page.locator('button').count();
    console.log(`   Buttons found: ${buttons}`);

    console.log('✅ Simple test completed successfully!');
    return true;

  } catch (error) {
    console.error('❌ Test failed:', error.message);
    return false;
  } finally {
    if (page) await page.close();
    if (browser) await browser.close();
  }
}

simpleDemoTest();