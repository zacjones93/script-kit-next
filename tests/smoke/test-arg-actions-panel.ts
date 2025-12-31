// Name: Test arg() actions panel opens
// Description: Verifies that Cmd+K opens the actions panel when actions are provided

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

console.error('[SMOKE] test-arg-actions-panel starting...');

// Track action triggers
let actionTriggered = false;
let triggeredActionName = '';

const actions = [
  {
    name: 'Test Action 1',
    shortcut: 'cmd+1',
    onAction: (input: string) => {
      actionTriggered = true;
      triggeredActionName = 'Test Action 1';
      console.error('[SMOKE] Test Action 1 triggered with input:', input);
    },
  },
  {
    name: 'Test Action 2', 
    description: 'A test action with description',
    shortcut: 'cmd+2',
    onAction: () => {
      actionTriggered = true;
      triggeredActionName = 'Test Action 2';
      console.error('[SMOKE] Test Action 2 triggered');
    },
  },
];

const choices = ['Choice A', 'Choice B', 'Choice C'];

console.error('[SMOKE] Calling arg() with 3 choices and 2 actions');
console.error('[SMOKE] EXPECTED BEHAVIOR:');
console.error('[SMOKE]   1. Actions button should be visible in header');
console.error('[SMOKE]   2. Cmd+K should open actions panel');
console.error('[SMOKE]   3. Cmd+1 should trigger "Test Action 1"');
console.error('[SMOKE]   4. Cmd+2 should trigger "Test Action 2"');

// Wait a bit then capture screenshot to verify UI
setTimeout(async () => {
  try {
    const screenshot = await captureScreenshot();
    console.error(`[SMOKE] Screenshot captured: ${screenshot.width}x${screenshot.height}`);
    
    // Save screenshot for visual verification
    const screenshotDir = join(process.cwd(), 'test-screenshots');
    mkdirSync(screenshotDir, { recursive: true });
    const filepath = join(screenshotDir, `arg-actions-panel-${Date.now()}.png`);
    writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
    console.error(`[SMOKE] Screenshot saved to: ${filepath}`);
  } catch (e) {
    console.error('[SMOKE] Screenshot failed:', e);
  }
}, 1000);

const result = await arg('Pick a choice (Cmd+K for actions, Cmd+1/2 for shortcuts):', choices, actions);

console.error('[SMOKE] Result:', result);
console.error('[SMOKE] Action triggered:', actionTriggered);
console.error('[SMOKE] Triggered action name:', triggeredActionName);
console.error('[SMOKE] test-arg-actions-panel completed');

process.exit(0);
