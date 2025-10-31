// @ts-nocheck
import { chromium } from '@playwright/test';
import path from 'path';

/**
 * Agent Debug Toolkit for NAMDRunner Development
 * 
 * This is the main debugging tool for agent-first development.
 * Use this to:
 * - Take screenshots of current UI state
 * - Monitor console logs and errors
 * - Test UI interactions with full error reporting
 * - Debug layout and styling issues
 * 
 * Usage:
 *   DISPLAY=:99 node tests/ui/debug-toolkit.js
 */

const SCREENSHOTS_DIR = './tests/ui/screenshots';
const VIDEOS_DIR = './tests/ui/videos';

class AgentDebugger {
  constructor() {
    this.browser = null;
    this.page = null;
    this.consoleLogs = [];
    this.jsErrors = [];
    this.networkErrors = [];
    this.screenshotCount = 0;
  }

  async initialize() {
    console.log('🤖 Starting Agent Debug Session...\n');

    // Auto-detect if running in SSH/headless environment
    const isHeadless = process.env.SSH_CLIENT || process.env.DISPLAY === ':99' || process.env.CI;

    this.browser = await chromium.launch({
      headless: isHeadless,
      slowMo: isHeadless ? 50 : 200,
      args: isHeadless ? ['--no-sandbox', '--disable-setuid-sandbox'] : []
    });

    console.log(`   Running in ${isHeadless ? 'headless' : 'headed'} mode`);
    
    const context = await this.browser.newContext({
      viewport: { width: 1280, height: 720 },
      recordVideo: { dir: VIDEOS_DIR }
    });
    
    this.page = await context.newPage();
    
    // Set up event listeners
    this.page.on('console', msg => {
      const logEntry = {
        type: msg.type(),
        text: msg.text(),
        timestamp: new Date().toISOString()
      };
      this.consoleLogs.push(logEntry);
      console.log(`   📝 CONSOLE [${msg.type().toUpperCase()}]: ${msg.text()}`);
    });
    
    this.page.on('pageerror', error => {
      const errorEntry = {
        message: error.message,
        stack: error.stack,
        timestamp: new Date().toISOString()
      };
      this.jsErrors.push(errorEntry);
      console.log(`   ❌ JS ERROR: ${error.message}`);
    });
    
    this.page.on('requestfailed', request => {
      const networkError = {
        url: request.url(),
        failure: request.failure(),
        timestamp: new Date().toISOString()
      };
      this.networkErrors.push(networkError);
      console.log(`   🌐 NETWORK ERROR: ${request.url()}`);
    });
    
    console.log('✅ Debug session initialized');
  }

  async connectToApp(url = 'http://localhost:1420/') {
    console.log(`🌐 Connecting to ${url}...`);
    console.log('   ⏳ Note: Vite dev server may take 1-3 minutes to start on first run');

    try {
      await this.page.goto(url, { waitUntil: 'networkidle', timeout: 60000 });
      await this.screenshot('app-loaded');
      console.log('✅ Connected to application');
    } catch (error) {
      console.log('❌ Connection failed - server may still be starting');
      console.log('   💡 Try: curl -s http://localhost:1420 to check if server is ready');
      throw error;
    }
  }

  async screenshot(name = null) {
    const filename = name ? 
      `${String(++this.screenshotCount).padStart(2, '0')}-${name}.png` :
      `${String(++this.screenshotCount).padStart(2, '0')}-debug.png`;
    
    const filepath = path.join(SCREENSHOTS_DIR, filename);
    await this.page.screenshot({ path: filepath, fullPage: true });
    console.log(`📸 Screenshot saved: ${filename}`);
    return filename;
  }

  async click(selector, options = {}) {
    console.log(`🖱️  Attempting to click: ${selector}`);
    try {
      const element = this.page.locator(selector);
      const isVisible = await element.isVisible();
      const isEnabled = await element.isEnabled();
      
      console.log(`   - Visible: ${isVisible}, Enabled: ${isEnabled}`);
      
      if (isVisible && isEnabled) {
        await element.click({ timeout: 5000, ...options });
        console.log(`   ✅ Click successful`);
        await this.screenshot(`after-click-${selector.replace(/[^a-zA-Z0-9]/g, '-')}`);
        return { success: true };
      } else {
        console.log(`   ⚠️  Element not clickable`);
        return { success: false, reason: 'Element not clickable' };
      }
    } catch (error) {
      console.log(`   ❌ Click failed: ${error.message}`);
      await this.screenshot(`click-failed-${selector.replace(/[^a-zA-Z0-9]/g, '-')}`);
      return { success: false, error: error.message };
    }
  }

  async fill(selector, value) {
    console.log(`⌨️  Filling ${selector} with: ${value}`);
    try {
      await this.page.locator(selector).fill(value);
      console.log(`   ✅ Fill successful`);
      await this.screenshot(`after-fill-${selector.replace(/[^a-zA-Z0-9]/g, '-')}`);
      return { success: true };
    } catch (error) {
      console.log(`   ❌ Fill failed: ${error.message}`);
      return { success: false, error: error.message };
    }
  }

  async waitForElement(selector, timeout = 5000) {
    console.log(`⏳ Waiting for element: ${selector}`);
    try {
      await this.page.waitForSelector(selector, { timeout });
      console.log(`   ✅ Element appeared`);
      return { success: true };
    } catch (error) {
      console.log(`   ❌ Element did not appear: ${error.message}`);
      return { success: false, error: error.message };
    }
  }

  async analyzeCurrentState() {
    console.log('\n🔍 Analyzing current page state...');
    
    const title = await this.page.title();
    const url = this.page.url();
    const buttons = await this.page.locator('button').count();
    const inputs = await this.page.locator('input').count();
    const links = await this.page.locator('a').count();
    
    console.log(`   📝 Title: ${title}`);
    console.log(`   🌐 URL: ${url}`);
    console.log(`   🔘 Buttons: ${buttons}`);
    console.log(`   📝 Inputs: ${inputs}`);
    console.log(`   🔗 Links: ${links}`);
    
    await this.screenshot('state-analysis');
    
    return { title, url, buttons, inputs, links };
  }

  async getDebugSummary() {
    return {
      consoleLogs: this.consoleLogs.length,
      jsErrors: this.jsErrors.length,
      networkErrors: this.networkErrors.length,
      screenshots: this.screenshotCount,
      errors: this.jsErrors,
      recentLogs: this.consoleLogs.slice(-10)
    };
  }

  async cleanup() {
    console.log('\n📊 Debug Session Summary:');
    const summary = await this.getDebugSummary();
    console.log(`   📝 Console logs: ${summary.consoleLogs}`);
    console.log(`   ❌ JavaScript errors: ${summary.jsErrors}`);
    console.log(`   🌐 Network errors: ${summary.networkErrors}`);
    console.log(`   📸 Screenshots taken: ${summary.screenshots}`);
    
    if (this.page) await this.page.close();
    if (this.browser) await this.browser.close();
    
    console.log('\n✅ Debug session complete');
  }
}

// Example usage function
async function exampleDebuggingSession() {
  const agentDebugger = new AgentDebugger();
  
  try {
    await agentDebugger.initialize();
    await agentDebugger.connectToApp();
    
    // Analyze initial state
    await agentDebugger.analyzeCurrentState();
    
    // Test connecting
    await agentDebugger.click('button:has-text("Connect")');
    
    // Wait a bit and take another screenshot
    await new Promise(resolve => setTimeout(resolve, 1000));
    await agentDebugger.screenshot('after-connect-click');
    
    // Check if dialog appeared
    const dialogVisible = await agentDebugger.page.locator('.dialog, [role="dialog"]').isVisible();
    console.log(`🔍 Dialog visible: ${dialogVisible}`);
    
    if (dialogVisible) {
      // Try filling form
      await agentDebugger.fill('input[placeholder*="host"], #host', 'test.cluster.edu');
      await agentDebugger.fill('input[placeholder*="username"], #username', 'testuser');
      await agentDebugger.fill('input[type="password"], #password', 'testpass');
      
      await agentDebugger.screenshot('form-filled');
    }
    
  } catch (error) {
    console.log(`❌ Debug session error: ${error.message}`);
  } finally {
    await agentDebugger.cleanup();
  }
}

// Export for use in other scripts
export { AgentDebugger };

// If run directly, execute example
if (import.meta.url === `file://${process.argv[1]}`) {
  exampleDebuggingSession();
}