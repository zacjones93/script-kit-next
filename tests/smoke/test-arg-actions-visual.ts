// Name: Test Arg Actions Visual
// Description: Captures screenshots at each step to verify actions panel rendering

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

console.error('[SMOKE] test-arg-actions-visual starting...');

const actions = [
  {
    name: 'Copy Item',
    shortcut: 'cmd+c',
    description: 'Copy to clipboard',
    onAction: () => console.error('[ACTION] Copy triggered'),
  },
  {
    name: 'Edit Item',
    shortcut: 'cmd+e',
    description: 'Edit in editor',
    onAction: () => console.error('[ACTION] Edit triggered'),
  },
  {
    name: 'Delete Item',
    shortcut: 'cmd+backspace',
    description: 'Delete permanently',
    onAction: () => console.error('[ACTION] Delete triggered'),
  },
];

const choices = ['Document A', 'Document B', 'Document C'];

// Helper to save screenshot
async function saveScreenshot(name: string): Promise<string | null> {
  try {
    const screenshot = await captureScreenshot();
    const screenshotDir = join(process.cwd(), 'test-screenshots');
    mkdirSync(screenshotDir, { recursive: true });
    const filepath = join(screenshotDir, `${name}.png`);
    writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
    console.error(`[SCREENSHOT] ${name}: ${filepath}`);
    return filepath;
  } catch (e) {
    console.error(`[SCREENSHOT] Error capturing ${name}:`, e);
    return null;
  }
}

// Signal that we're ready for screenshots
// The test runner will send SimulateKey commands and request screenshots
console.error('[VISUAL] Ready for test sequence');
console.error('[VISUAL] Will capture screenshots at key moments');

// This script waits - the test sequence is:
// 1. Initial screenshot (arg prompt with Actions button)
// 2. SimulateKey Cmd+K
// 3. Screenshot (actions panel open)
// 4. SimulateKey Down
// 5. Screenshot (second action selected)
// 6. SimulateKey Enter or Escape

// Capture initial state after render
setTimeout(async () => {
  await saveScreenshot('arg-visual-01-initial');
}, 300);

const result = await arg('Select a document:', choices, actions);

console.error('[RESULT]', result);
process.exit(0);
