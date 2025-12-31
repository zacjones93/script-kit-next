// Name: Test Arg Actions Full Flow
// Description: Full test with screenshots before/after Cmd+K

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

console.error('[TEST] test-arg-actions-full starting...');

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

// The test sequence:
// 1. Script starts, arg prompt shown
// 2. Wait 500ms, capture "before-cmdk" screenshot
// 3. External test sends: {"type":"simulateKey","key":"k","modifiers":["cmd"]}
// 4. Wait 500ms, capture "after-cmdk" screenshot  
// 5. Continue test

// Capture BEFORE Cmd+K (initial state)
setTimeout(async () => {
  await saveScreenshot('test-before-cmdk');
  console.error('[TEST] BEFORE screenshot captured');
  console.error('[TEST] Waiting for simulateKey cmd+k...');
}, 500);

// Capture AFTER Cmd+K (this runs 2 seconds in, after external cmd+k should have been sent)
setTimeout(async () => {
  await saveScreenshot('test-after-cmdk');
  console.error('[TEST] AFTER screenshot captured');
  console.error('[TEST] Actions panel should be visible in after-cmdk screenshot');
}, 2000);

const result = await arg('Select a document:', choices, actions);

console.error('[TEST] Result:', result);
process.exit(0);
