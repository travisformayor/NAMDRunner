// Basic E2E test to verify Tauri app launches
// This test can be run with WebDriver when E2E infrastructure is set up

import { test, expect } from '@playwright/test';

test.describe('NAMDRunner App', () => {
  test('should display app title and connection interface', async ({ page }) => {
    // Navigate to the app (this would be different for Tauri app)
    await page.goto('/');

    // Check that the app title is displayed
    await expect(page.locator('h1')).toContainText('NAMDRunner');
    await expect(page.locator('.subtitle')).toContainText('SLURM NAMD Simulation Manager');

    // Check that connection section is present
    await expect(page.locator('.connection-section')).toBeVisible();
    await expect(page.locator('h2')).toContainText('Cluster Connection');

    // Check that welcome section is visible when disconnected
    await expect(page.locator('.welcome-section')).toBeVisible();
    await expect(page.locator('h3')).toContainText('Welcome to NAMDRunner');

    // Check that connect button is present
    await expect(page.locator('button').filter({ hasText: 'Connect' })).toBeVisible();

    // Take a screenshot
    await page.screenshot({ path: 'tests/screenshots/basic-ui.png' });
  });

  test('should open connection dialog when connect button is clicked', async ({ page }) => {
    await page.goto('/');

    // Click the connect button
    await page.locator('button').filter({ hasText: 'Connect' }).click();

    // Check that connection dialog is visible
    await expect(page.locator('.dialog')).toBeVisible();
    await expect(page.locator('.dialog h2')).toContainText('Connect to Cluster');

    // Check form fields are present
    await expect(page.locator('#host')).toBeVisible();
    await expect(page.locator('#username')).toBeVisible();
    await expect(page.locator('#password')).toBeVisible();

    // Take a screenshot of the dialog
    await page.screenshot({ path: 'tests/screenshots/connection-dialog.png' });

    // Close dialog by clicking cancel
    await page.locator('button').filter({ hasText: 'Cancel' }).click();
    await expect(page.locator('.dialog')).not.toBeVisible();
  });

  // This test simulates successful connection using mock client
  test('should handle mock connection flow', async ({ page }) => {
    await page.goto('/');

    // Open connection dialog
    await page.locator('button').filter({ hasText: 'Connect' }).click();

    // Fill in connection details
    await page.locator('#host').fill('test.cluster.edu');
    await page.locator('#username').fill('testuser');
    await page.locator('#password').fill('testpass');

    // Submit form
    await page.locator('button[type="submit"]').click();

    // Wait for connection to complete (mock will succeed)
    await expect(page.locator('.status-text')).toContainText('Connected', { timeout: 5000 });

    // Check that jobs section appears when connected
    await expect(page.locator('.jobs-section')).toBeVisible();
    await expect(page.locator('.welcome-section')).not.toBeVisible();

    // Take screenshot of connected state
    await page.screenshot({ path: 'tests/screenshots/connected-state.png' });
  });
});