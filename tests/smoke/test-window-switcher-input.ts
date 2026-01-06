// Test: Window Switcher Input
// This test needs to be run manually by:
// 1. Opening Script Kit main menu
// 2. Typing "window" to filter to Window Switcher
// 3. Pressing Enter to open Window Switcher
// Then the screenshot will show the input field

import '../../scripts/kit-sdk';

const fs = require('fs');
const dir = `${process.cwd()}/.test-screenshots`;
fs.mkdirSync(dir, { recursive: true });

console.error('[TEST] Window Switcher input test - verifying screenshot capture works');

try {
  // Wait for the app to be ready
  await new Promise(r => setTimeout(r, 500));
  
  // Capture screenshot of current state
  const screenshot = await captureScreenshot();
  
  // Save screenshot
  const filepath = `${dir}/window-switcher-input-${Date.now()}.png`;
  fs.writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
  
  console.error(`[TEST] Screenshot saved to ${filepath}`);
  console.error('[TEST] Test completed - check screenshot for input appearance');
} catch (e) {
  console.error(`[TEST] Failed: ${e}`);
}

process.exit(0);
