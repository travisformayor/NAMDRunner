// Detailed UI inspection for demo mode
import { chromium } from '@playwright/test';
import fs from 'fs';
import path from 'path';

const RESULTS_DIR = './tests/ui/results';

async function detailedInspection() {
  console.log('🔍 Starting Detailed UI Inspection...');

  let browser = null;
  let page = null;
  const findings = {
    demo_mode_status: null,
    navigation_elements: [],
    job_interface: {},
    connection_workflow: {},
    form_fields: {},
    errors_and_issues: []
  };

  try {
    browser = await chromium.launch({ headless: true });
    page = await browser.newPage();

    console.log('🌐 Loading application...');
    await page.goto('http://localhost:1420', { waitUntil: 'networkidle' });

    // Inspect demo mode status
    console.log('\n📱 Inspecting Demo Mode Status...');
    const consoleMessages = [];
    page.on('console', msg => {
      if (msg.text().includes('demo') || msg.text().includes('mock')) {
        consoleMessages.push(msg.text());
      }
    });

    // Check for demo mode indicators
    const connectionButton = await page.locator('[data-testid="connection-status-button"]');
    if (await connectionButton.isVisible()) {
      const buttonText = await connectionButton.textContent();
      findings.demo_mode_status = {
        connection_button_text: buttonText,
        is_connected: buttonText?.includes('Connected'),
        appears_demo: consoleMessages.some(msg => msg.includes('demo mode'))
      };
      console.log(`   Connection status: ${buttonText}`);
    }

    // Inspect navigation elements
    console.log('\n🧭 Inspecting Navigation Elements...');
    const navElements = await page.locator('.sidebar button, .sidebar a').all();
    for (const element of navElements) {
      const text = await element.textContent();
      const isVisible = await element.isVisible();
      findings.navigation_elements.push({ text: text?.trim(), visible: isVisible });
      console.log(`   Navigation: "${text?.trim()}" (visible: ${isVisible})`);
    }

    // Test Jobs interface
    console.log('\n📊 Inspecting Jobs Interface...');
    const jobsButton = page.locator('button', { hasText: 'Jobs' }).first();
    if (await jobsButton.isVisible()) {
      await jobsButton.click();
      await page.waitForTimeout(1000);

      // Count job entries
      const jobRows = await page.locator('tr:not(:first-child)').count(); // Exclude header
      const jobCards = await page.locator('.job-card, .job-item').count();

      findings.job_interface = {
        jobs_page_loaded: true,
        job_rows: jobRows,
        job_cards: jobCards,
        total_jobs: Math.max(jobRows - 1, jobCards), // -1 for potential header row
        has_mock_data: (jobRows > 1) || (jobCards > 0)
      };

      console.log(`   Job rows found: ${jobRows}`);
      console.log(`   Job cards found: ${jobCards}`);

      // Test job details
      if (jobRows > 1 || jobCards > 0) {
        const firstJobElement = jobRows > 1 ?
          page.locator('tr:not(:first-child)').first() :
          page.locator('.job-card, .job-item').first();

        if (await firstJobElement.isVisible()) {
          await firstJobElement.click();
          await page.waitForTimeout(500);

          const detailsVisible = await page.locator('.job-details, .modal, [role="dialog"]').isVisible();
          findings.job_interface.details_open = detailsVisible;
          console.log(`   Job details view opens: ${detailsVisible}`);
        }
      }
    }

    // Test Create Job interface
    console.log('\n🆕 Inspecting Create Job Interface...');
    const createButton = page.locator('button', { hasText: 'Create' }).first();
    if (await createButton.isVisible()) {
      await createButton.click();
      await page.waitForTimeout(1000);

      // Look for common form fields
      const nameField = await page.locator('input[name*="name"], #name, #jobName').isVisible();
      const tempField = await page.locator('input[name*="temp"], #temperature').isVisible();
      const partitionField = await page.locator('select[name*="partition"], #partition').isVisible();

      // Also check for any input fields
      const allInputs = await page.locator('input').count();
      const allSelects = await page.locator('select').count();

      findings.form_fields = {
        create_page_loaded: true,
        name_field: nameField,
        temperature_field: tempField,
        partition_field: partitionField,
        total_inputs: allInputs,
        total_selects: allSelects
      };

      console.log(`   Name field visible: ${nameField}`);
      console.log(`   Temperature field visible: ${tempField}`);
      console.log(`   Partition field visible: ${partitionField}`);
      console.log(`   Total inputs: ${allInputs}, selects: ${allSelects}`);
    }

    // Check connection workflow
    console.log('\n🔌 Inspecting Connection Workflow...');
    await page.goto('http://localhost:1420'); // Go back to main page
    await page.waitForTimeout(500);

    const connectionTrigger = page.locator('[data-testid="connection-status-button"]');
    if (await connectionTrigger.isVisible()) {
      await connectionTrigger.click();
      await page.waitForTimeout(500);

      // Look for dropdown options
      const dropdownVisible = await page.locator('.dropdown-content, .connection-dropdown').isVisible();
      const demoOption = await page.locator('button:has-text("Demo"), button:has-text("Mock")').isVisible();

      findings.connection_workflow = {
        dropdown_opens: dropdownVisible,
        demo_option_available: demoOption,
        appears_already_connected: await page.locator(':has-text("Connected")').count() > 0
      };

      console.log(`   Dropdown opens: ${dropdownVisible}`);
      console.log(`   Demo option available: ${demoOption}`);
    }

    console.log('\n✅ Inspection complete');

  } catch (error) {
    console.error('❌ Inspection failed:', error);
    findings.errors_and_issues.push(error.message);
  } finally {
    if (page) await page.close();
    if (browser) await browser.close();
  }

  // Save findings
  const resultsFile = path.join(RESULTS_DIR, 'detailed-inspection.json');
  fs.writeFileSync(resultsFile, JSON.stringify(findings, null, 2));
  console.log(`\n📄 Detailed findings saved to: ${resultsFile}`);

  return findings;
}

detailedInspection();