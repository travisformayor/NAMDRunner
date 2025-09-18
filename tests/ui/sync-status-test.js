// Test script to verify sync status consistency
// Run this in the browser console to test sync status behavior

console.log('🧪 Testing Sync Status Consistency');

// Test 1: Check initial sync time
function testInitialSyncTime() {
  console.log('\n1️⃣ Testing initial sync time...');
  const syncStatusElement = document.querySelector('.status-text');
  if (syncStatusElement) {
    const statusText = syncStatusElement.textContent;
    console.log('Initial status:', statusText);

    // Should show "Last synced: X minutes ago" (not "Just now" since we set it to 15 minutes ago)
    if (statusText.includes('15 minutes ago') || statusText.includes('minute')) {
      console.log('✅ Initial sync time is realistic (15 minutes ago)');
    } else if (statusText.includes('Just now')) {
      console.log('❌ Initial sync time shows "Just now" - should be 15 minutes ago');
    } else {
      console.log('⚠️ Unexpected status text:', statusText);
    }
  } else {
    console.log('❌ Could not find sync status element');
  }
}

// Test 2: Test manual sync
function testManualSync() {
  console.log('\n2️⃣ Testing manual sync...');
  const syncButton = document.querySelector('.sync-button');
  if (syncButton && !syncButton.disabled) {
    console.log('Clicking sync button...');
    syncButton.click();

    // Check if sync icon is spinning
    setTimeout(() => {
      const syncIcon = document.querySelector('.sync-icon');
      if (syncIcon && syncIcon.classList.contains('spinning')) {
        console.log('✅ Sync animation is working');
      } else {
        console.log('❌ Sync animation not found');
      }

      // Check status after sync completes (1 second delay + buffer)
      setTimeout(() => {
        const syncStatusElement = document.querySelector('.status-text');
        if (syncStatusElement) {
          const statusText = syncStatusElement.textContent;
          console.log('Status after sync:', statusText);

          if (statusText.includes('Just now')) {
            console.log('✅ Sync time updated to "Just now" after manual sync');
          } else {
            console.log('❌ Sync time not updated after manual sync');
          }
        }
      }, 1500);
    }, 100);
  } else {
    console.log('❌ Sync button not found or disabled');
  }
}

// Test 3: Navigate to job details and back
function testNavigationConsistency() {
  console.log('\n3️⃣ Testing navigation consistency...');

  // Store current status
  const initialStatus = document.querySelector('.status-text')?.textContent;
  console.log('Status before navigation:', initialStatus);

  // Click on first job
  const firstJobRow = document.querySelector('.job-row');
  if (firstJobRow) {
    console.log('Navigating to job details...');
    firstJobRow.click();

    // Wait for navigation, then go back
    setTimeout(() => {
      const backButton = document.querySelector('.back-button');
      if (backButton) {
        console.log('Navigating back to jobs list...');
        backButton.click();

        // Check status after navigation
        setTimeout(() => {
          const finalStatus = document.querySelector('.status-text')?.textContent;
          console.log('Status after navigation back:', finalStatus);

          if (initialStatus === finalStatus) {
            console.log('✅ Sync status consistent after navigation');
          } else {
            console.log('❌ Sync status changed after navigation');
            console.log('  Before:', initialStatus);
            console.log('  After:', finalStatus);
          }
        }, 100);
      } else {
        console.log('❌ Back button not found');
      }
    }, 500);
  } else {
    console.log('❌ No job rows found for navigation test');
  }
}

// Test 4: Test connection state changes
function testConnectionStateSync() {
  console.log('\n4️⃣ Testing connection state sync consistency...');

  // Note: This would require manually disconnecting in the UI
  console.log('ℹ️ Test connection state manually:');
  console.log('  1. Disconnect from cluster (click connection dropdown -> disconnect)');
  console.log('  2. Check that status shows "Offline - showing cached data from [time]"');
  console.log('  3. Reconnect');
  console.log('  4. Check that status shows "Last synced: [time]"');
}

// Run all tests
function runSyncStatusTests() {
  console.log('Starting sync status consistency tests...');

  testInitialSyncTime();

  setTimeout(() => {
    testManualSync();

    setTimeout(() => {
      testNavigationConsistency();

      setTimeout(() => {
        testConnectionStateSync();
        console.log('\n🏁 Sync status tests completed!');
      }, 3000);
    }, 2000);
  }, 1000);
}

// Export for manual running
window.runSyncStatusTests = runSyncStatusTests;

console.log('📋 Run window.runSyncStatusTests() to start tests');
console.log('Or run individual tests:');
console.log('  - testInitialSyncTime()');
console.log('  - testManualSync()');
console.log('  - testNavigationConsistency()');
console.log('  - testConnectionStateSync()');

// Make functions available globally for manual testing
window.testInitialSyncTime = testInitialSyncTime;
window.testManualSync = testManualSync;
window.testNavigationConsistency = testNavigationConsistency;
window.testConnectionStateSync = testConnectionStateSync;