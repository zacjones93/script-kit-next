// Name: Test Arg Actions Panel - Delayed Capture
// Description: Tests arg() with actions - captures screenshot after delay

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

console.error('[TEST] Starting arg actions delayed capture test');

// Create screenshot directory
const screenshotDir = join(process.cwd(), 'test-screenshots');
mkdirSync(screenshotDir, { recursive: true });

// Define actions for the arg prompt
const actions = [
  { name: 'Copy Item', shortcut: 'cmd+c', onAction: () => console.error('[ACTION] Copy triggered') },
  { name: 'Edit Item', shortcut: 'cmd+e', onAction: () => console.error('[ACTION] Edit triggered') },
  { name: 'Delete Item', shortcut: 'cmd+d', onAction: () => console.error('[ACTION] Delete triggered') },
];

const choices = ['Apple', 'Banana', 'Cherry', 'Date', 'Elderberry'];

console.error('[TEST] Showing arg prompt with actions...');

// Set up a delayed screenshot capture
// This will capture AFTER the arg prompt is shown and AFTER Cmd+K might be pressed
setTimeout(async () => {
  console.error('[TEST] Capturing delayed screenshot (2s after arg shown)...');
  try {
    const screenshot = await captureScreenshot();
    console.error(`[TEST] Screenshot captured: ${screenshot.width}x${screenshot.height}`);
    const filepath = join(screenshotDir, `arg-actions-delayed-${Date.now()}.png`);
    writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
    console.error(`[SCREENSHOT] Saved to: ${filepath}`);
  } catch (err) {
    console.error('[TEST] Screenshot error:', err);
  }
}, 2000);

// Start the arg prompt with actions
const result = await arg({
  placeholder: 'Select a fruit (press Cmd+K for actions)',
  actions,
}, choices);

console.error(`[TEST] Result: ${result}`);
console.error('[TEST] Test complete');

process.exit(0);
