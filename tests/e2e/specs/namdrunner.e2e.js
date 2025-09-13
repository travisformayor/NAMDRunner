// @ts-nocheck
describe('NAMDRunner Desktop App E2E', () => {
  before(async () => {
    console.log('🚀 Starting E2E test session...');
    console.log('📊 Browser capabilities:', browser.capabilities);
    console.log('🌐 Session ID:', browser.sessionId);
    console.log('🔗 WebDriver URL:', browser.options.hostname + ':' + browser.options.port);
  });

  it('should launch and show the main window', async () => {
    console.log('🧪 Testing app launch...');
    
    // Verify the app title
    const title = await browser.getTitle();
    console.log(`   📝 App title: ${title}`);
    expect(title).toBe('NAMDRunner');
    
    // Take a screenshot for debugging
    await browser.saveScreenshot('test-results/01-app-launched.png');
  });

  it('should display the main NAMDRunner interface', async () => {
    console.log('🧪 Testing main interface...');
    
    // Check for the main heading
    const heading = await $('h1');
    const headingText = await heading.getText();
    console.log(`   📝 Main heading: ${headingText}`);
    expect(headingText).toBe('NAMDRunner');
    
    // Check for Connect button
    const connectButton = await $('button*=Connect');
    const isConnectVisible = await connectButton.isDisplayed();
    console.log(`   🔘 Connect button visible: ${isConnectVisible}`);
    expect(isConnectVisible).toBe(true);
    
    await browser.saveScreenshot('test-results/02-main-interface.png');
  });

  it('should open connection dialog when Connect is clicked', async () => {
    console.log('🧪 Testing connection dialog...');
    
    // Click the Connect button
    const connectButton = await $('button*=Connect');
    await connectButton.click();
    
    // Wait for dialog to appear (give it some time)
    await browser.pause(1000);
    
    // Check if connection dialog appeared
    const dialog = await $('.dialog, [role="dialog"]');
    const isDialogVisible = await dialog.isDisplayed();
    console.log(`   💬 Connection dialog visible: ${isDialogVisible}`);
    expect(isDialogVisible).toBe(true);
    
    await browser.saveScreenshot('test-results/03-connection-dialog.png');
  });

  it('should have form fields in connection dialog', async () => {
    console.log('🧪 Testing connection form fields...');
    
    // Check for host input
    const hostInput = await $('input[placeholder*="host"], input[name*="host"], #host');
    const isHostVisible = await hostInput.isDisplayed();
    console.log(`   🏠 Host input visible: ${isHostVisible}`);
    expect(isHostVisible).toBe(true);
    
    // Check for username input  
    const usernameInput = await $('input[placeholder*="username"], input[name*="username"], #username');
    const isUsernameVisible = await usernameInput.isDisplayed();
    console.log(`   👤 Username input visible: ${isUsernameVisible}`);
    expect(isUsernameVisible).toBe(true);
    
    // Check for password input
    const passwordInput = await $('input[type="password"], #password');
    const isPasswordVisible = await passwordInput.isDisplayed();
    console.log(`   🔒 Password input visible: ${isPasswordVisible}`);
    expect(isPasswordVisible).toBe(true);
    
    await browser.saveScreenshot('test-results/04-form-fields.png');
  });

  it('should be able to fill form fields', async () => {
    console.log('🧪 Testing form interaction...');
    
    // Fill the host field
    const hostInput = await $('input[placeholder*="host"], input[name*="host"], #host');
    await hostInput.setValue('test.cluster.edu');
    const hostValue = await hostInput.getValue();
    console.log(`   🏠 Host value: ${hostValue}`);
    expect(hostValue).toBe('test.cluster.edu');
    
    // Fill the username field
    const usernameInput = await $('input[placeholder*="username"], input[name*="username"], #username');
    await usernameInput.setValue('testuser');
    const usernameValue = await usernameInput.getValue();
    console.log(`   👤 Username value: ${usernameValue}`);
    expect(usernameValue).toBe('testuser');
    
    // Fill the password field
    const passwordInput = await $('input[type="password"], #password');
    await passwordInput.setValue('testpass');
    const passwordValue = await passwordInput.getValue();
    console.log(`   🔒 Password field filled: ${passwordValue.length > 0}`);
    expect(passwordValue).toBe('testpass');
    
    await browser.saveScreenshot('test-results/05-form-filled.png');
  });
});