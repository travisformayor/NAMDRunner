import { Builder, By, until, Key } from 'selenium-webdriver';
import firefox from 'selenium-webdriver/firefox.js';

async function runE2ETests() {
    // Set up Firefox options for headless testing (can be disabled for debugging)
    const options = new firefox.Options();
    // options.addArguments('--headless'); // Uncomment for headless mode
    options.setPreference('dom.webnotifications.enabled', false);

    const driver = await new Builder()
        .forBrowser('firefox')
        .setFirefoxOptions(options)
        .build();

    try {
        console.log('🚀 Starting NAMDRunner E2E Tests...\n');

        // Navigate to the app
        console.log('📱 Navigating to http://localhost:1420/');
        await driver.get('http://localhost:1420/');

        // Wait for app to load
        console.log('⏳ Waiting for app to load...');
        await driver.wait(until.titleContains('NAMDRunner'), 10000);

        // Test 1: Initial app state - verify it starts disconnected with no jobs showing
        console.log('\n🧪 Test 1: Checking initial app state...');

        // Wait a few seconds to observe any auto-connection behavior
        console.log('⏳ Waiting 5 seconds to observe auto-connection behavior...');
        await driver.sleep(5000);

        // Check connection status - should be disconnected initially
        try {
            const disconnectedElement = await driver.wait(
                until.elementLocated(By.css('[data-testid="connection-status-disconnected"], .disconnected, .status-disconnected')),
                5000
            );
            console.log('✅ App starts in disconnected state');
        } catch (e) {
            // Try alternative selectors for connection status
            try {
                const connectionStatus = await driver.findElement(By.css('.connection-status, .status, [class*="status"], [class*="connection"]'));
                const statusText = await connectionStatus.getText();
                console.log(`⚠️  Connection status found: "${statusText}"`);

                if (statusText.toLowerCase().includes('disconnect') || statusText.toLowerCase().includes('not connected')) {
                    console.log('✅ App appears to start in disconnected state');
                } else {
                    console.log('❌ App may be auto-connecting - status: ' + statusText);
                }
            } catch (e2) {
                console.log('⚠️  Could not find connection status element - checking for connect button');
                try {
                    const connectButton = await driver.findElement(By.css('button[data-testid="connect"], button:contains("Connect"), button[class*="connect"]'));
                    console.log('✅ Connect button found - app appears to be disconnected');
                } catch (e3) {
                    console.log('❌ Could not determine connection state');
                }
            }
        }

        // Check that no jobs are showing initially
        try {
            const jobElements = await driver.findElements(By.css('[data-testid="job-item"], .job-item, .job, [class*="job"]'));
            if (jobElements.length === 0) {
                console.log('✅ No jobs showing initially');
            } else {
                console.log(`⚠️  Found ${jobElements.length} job elements initially`);
            }
        } catch (e) {
            console.log('✅ No job elements found initially');
        }

        // Test 2: Demo toggle functionality
        console.log('\n🧪 Test 2: Testing demo toggle functionality...');

        try {
            // Look for demo toggle/checkbox
            const demoToggle = await driver.wait(
                until.elementLocated(By.css('input[data-testid="demo-toggle"], input[type="checkbox"][data-testid*="demo"], input[type="checkbox"]:contains("demo"), label:contains("Demo") input, label:contains("demo") input')),
                5000
            );

            // Check if demo mode is currently enabled
            const isChecked = await demoToggle.isSelected();
            console.log(`Demo toggle current state: ${isChecked ? 'enabled' : 'disabled'}`);

            if (!isChecked) {
                console.log('🔄 Enabling demo mode...');
                await demoToggle.click();
                await driver.sleep(1000); // Wait for state change

                const newState = await demoToggle.isSelected();
                if (newState) {
                    console.log('✅ Demo mode enabled successfully');
                } else {
                    console.log('❌ Failed to enable demo mode');
                }
            } else {
                console.log('✅ Demo mode already enabled');
            }
        } catch (e) {
            console.log('⚠️  Could not find demo toggle - trying alternative selectors...');
            try {
                const demoElement = await driver.findElement(By.xpath("//label[contains(text(), 'Demo') or contains(text(), 'demo')]"));
                await demoElement.click();
                console.log('✅ Found and clicked demo element');
            } catch (e2) {
                console.log('❌ Could not find demo toggle element');
            }
        }

        // Test 3: Connection in demo mode
        console.log('\n🧪 Test 3: Testing connection in demo mode...');

        try {
            // Look for connect button
            const connectButton = await driver.wait(
                until.elementLocated(By.css('button[data-testid="connect"], button:contains("Connect"), button[class*="connect"]')),
                5000
            );

            console.log('🔄 Clicking connect button...');
            await connectButton.click();

            // Wait for connection to establish
            await driver.sleep(3000);

            // Check for connected state
            try {
                const connectedElement = await driver.wait(
                    until.elementLocated(By.css('[data-testid="connection-status-connected"], .connected, .status-connected')),
                    10000
                );
                console.log('✅ Successfully connected in demo mode');

                // Check for demo data (jobs)
                await driver.sleep(2000);
                const jobElements = await driver.findElements(By.css('[data-testid="job-item"], .job-item, .job, [class*="job"]'));
                if (jobElements.length > 0) {
                    console.log(`✅ Demo data loaded - found ${jobElements.length} jobs`);
                } else {
                    console.log('⚠️  No demo jobs found after connection');
                }

            } catch (e) {
                console.log('⚠️  Could not find connected status - checking for disconnect button...');
                try {
                    const disconnectButton = await driver.findElement(By.css('button[data-testid="disconnect"], button:contains("Disconnect"), button[class*="disconnect"]'));
                    console.log('✅ Found disconnect button - appears to be connected');
                } catch (e2) {
                    console.log('❌ Could not determine connection state after connect attempt');
                }
            }

        } catch (e) {
            console.log('❌ Could not find connect button');
        }

        // Test 4: Disconnect functionality
        console.log('\n🧪 Test 4: Testing disconnect functionality...');

        try {
            // Look for disconnect button
            const disconnectButton = await driver.wait(
                until.elementLocated(By.css('button[data-testid="disconnect"], button:contains("Disconnect"), button[class*="disconnect"]')),
                5000
            );

            console.log('🔄 Clicking disconnect button...');
            await disconnectButton.click();

            // Wait for disconnection
            await driver.sleep(2000);

            // Check for disconnected state
            try {
                const disconnectedElement = await driver.wait(
                    until.elementLocated(By.css('[data-testid="connection-status-disconnected"], .disconnected, .status-disconnected')),
                    5000
                );
                console.log('✅ Successfully disconnected');

                // Check that jobs are cleared (or should be)
                await driver.sleep(1000);
                const jobElements = await driver.findElements(By.css('[data-testid="job-item"], .job-item, .job, [class*="job"]'));
                console.log(`Jobs after disconnect: ${jobElements.length}`);

            } catch (e) {
                console.log('⚠️  Could not find disconnected status - checking for connect button...');
                try {
                    const connectButton = await driver.findElement(By.css('button[data-testid="connect"], button:contains("Connect"), button[class*="connect"]'));
                    console.log('✅ Found connect button - appears to be disconnected');
                } catch (e2) {
                    console.log('❌ Could not determine connection state after disconnect attempt');
                }
            }

        } catch (e) {
            console.log('❌ Could not find disconnect button - may already be disconnected');
        }

        // Test 5: Final verification - no auto-connection after disconnect
        console.log('\n🧪 Test 5: Verifying no auto-connection occurs...');
        console.log('⏳ Waiting 5 seconds to observe any auto-connection behavior...');
        await driver.sleep(5000);

        try {
            const connectButton = await driver.findElement(By.css('button[data-testid="connect"], button:contains("Connect"), button[class*="connect"]'));
            console.log('✅ Connect button still present - no auto-connection detected');
        } catch (e) {
            console.log('⚠️  Connect button not found - checking connection status...');
            try {
                const connectionElements = await driver.findElements(By.css('.connection-status, .status, [class*="status"], [class*="connection"]'));
                if (connectionElements.length > 0) {
                    const statusText = await connectionElements[0].getText();
                    console.log(`Connection status: "${statusText}"`);
                }
            } catch (e2) {
                console.log('Could not determine final connection state');
            }
        }

        console.log('\n🎉 E2E Tests completed!');

    } catch (error) {
        console.error('❌ Test failed with error:', error);
    } finally {
        // Close the browser
        await driver.quit();
    }
}

// Run the tests
runE2ETests();