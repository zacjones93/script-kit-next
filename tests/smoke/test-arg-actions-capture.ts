// Name: Test Arg Actions Panel Screenshot Capture
// Description: Tests arg() with actions and captures screenshot when panel is open

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

console.error('[TEST] Starting arg actions capture test');

// Create screenshot directory
const screenshotDir = join(process.cwd(), 'test-screenshots');
mkdirSync(screenshotDir, { recursive: true });

// Define actions for the arg prompt
const actions = [
  { name: 'Copy', shortcut: 'cmd+c', onAction: () => console.error('[ACTION] Copy triggered') },
  { name: 'Edit', shortcut: 'cmd+e', onAction: () => console.error('[ACTION] Edit triggered') },
  { name: 'Delete', shortcut: 'cmd+d', onAction: () => console.error('[ACTION] Delete triggered') },
];

const choices = ['Apple', 'Banana', 'Cherry', 'Date', 'Elderberry'];

console.error('[TEST] Showing arg prompt with actions');

// Capture initial state
const initialScreenshot = await captureScreenshot();
console.error(`[TEST] Initial screenshot: ${initialScreenshot.width}x${initialScreenshot.height}`);
const initialPath = join(screenshotDir, `arg-actions-initial-${Date.now()}.png`);
writeFileSync(initialPath, Buffer.from(initialScreenshot.data, 'base64'));
console.error(`[SCREENSHOT] Initial saved to: ${initialPath}`);

// Start the arg prompt with actions (this will wait for user input)
// The test harness will send SimulateKey commands to interact with it
const result = await arg({
  placeholder: 'Select a fruit (press Cmd+K for actions)',
  actions,
}, choices);

console.error(`[TEST] Result: ${result}`);
console.error('[TEST] Test complete');

process.exit(0);
