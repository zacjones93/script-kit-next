// Name: Test Arg Actions with SimulateKey
// Description: Tests Cmd+K opens actions panel and arrow keys navigate it

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

console.error('[SMOKE] test-arg-actions-simulate starting...');

const actions = [
  {
    name: 'Action One',
    shortcut: 'cmd+1',
    description: 'First action',
    onAction: () => {
      console.error('[SMOKE] Action One triggered!');
    },
  },
  {
    name: 'Action Two',
    shortcut: 'cmd+2',
    description: 'Second action',
    onAction: () => {
      console.error('[SMOKE] Action Two triggered!');
    },
  },
  {
    name: 'Action Three',
    shortcut: 'cmd+3',
    description: 'Third action',
    onAction: () => {
      console.error('[SMOKE] Action Three triggered!');
    },
  },
];

const choices = ['Item A', 'Item B', 'Item C'];

// Helper to save screenshot
async function saveScreenshot(name: string) {
  try {
    const screenshot = await captureScreenshot();
    const screenshotDir = join(process.cwd(), 'test-screenshots');
    mkdirSync(screenshotDir, { recursive: true });
    const filepath = join(screenshotDir, `${name}-${Date.now()}.png`);
    writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
    console.error(`[SMOKE] Screenshot saved: ${filepath}`);
    return filepath;
  } catch (e) {
    console.error('[SMOKE] Screenshot error:', e);
    return null;
  }
}

// The test will be driven by stdin SimulateKey commands
// This script just sets up the arg prompt with actions
console.error('[SMOKE] Setting up arg prompt with 3 actions and 3 choices');
console.error('[SMOKE] TEST SEQUENCE: Send these stdin commands to test:');
console.error('[SMOKE]   1. Wait 500ms for UI to render');
console.error('[SMOKE]   2. {"type":"simulateKey","key":"k","modifiers":["cmd"]} - open actions panel');
console.error('[SMOKE]   3. Wait 300ms');
console.error('[SMOKE]   4. {"type":"simulateKey","key":"down","modifiers":[]} - select Action Two');
console.error('[SMOKE]   5. Wait 300ms');
console.error('[SMOKE]   6. {"type":"simulateKey","key":"enter","modifiers":[]} - trigger action');

// Capture initial state after a short delay
setTimeout(async () => {
  await saveScreenshot('arg-actions-initial');
  console.error('[SMOKE] Initial screenshot captured - arg prompt visible with Actions button');
}, 500);

const result = await arg('Pick an item (Cmd+K for actions):', choices, actions);

console.error('[SMOKE] Result:', result);
console.error('[SMOKE] test-arg-actions-simulate completed');

process.exit(0);
