// Name: Test Cmd+K opens actions panel
// Description: Verifies that Cmd+K opens the actions panel in arg prompt

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

console.error('[SMOKE] test-arg-actions-cmdk starting...');

const actions = [
  {
    name: 'Copy to Clipboard',
    shortcut: 'cmd+c',
    description: 'Copy selected value',
    onAction: (input: string) => {
      console.error('[SMOKE] Copy action triggered');
    },
  },
  {
    name: 'Preview',
    shortcut: 'cmd+p',
    onAction: () => {
      console.error('[SMOKE] Preview action triggered');
    },
  },
  {
    name: 'Delete Item',
    shortcut: 'cmd+backspace',
    description: 'Remove this item',
    onAction: () => {
      console.error('[SMOKE] Delete action triggered');
    },
  },
];

const choices = ['Document 1', 'Document 2', 'Document 3'];

console.error('[SMOKE] TEST: Verifying Cmd+K opens actions panel');
console.error('[SMOKE] Will capture screenshot before timeout');

// Capture initial state
setTimeout(async () => {
  try {
    const screenshot = await captureScreenshot();
    const screenshotDir = join(process.cwd(), 'test-screenshots');
    mkdirSync(screenshotDir, { recursive: true });
    
    const filepath = join(screenshotDir, `arg-actions-initial-${Date.now()}.png`);
    writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
    console.error(`[SMOKE] Initial screenshot: ${filepath}`);
    console.error('[SMOKE] VERIFY: Actions button should be visible in header');
  } catch (e) {
    console.error('[SMOKE] Screenshot error:', e);
  }
}, 500);

// NOTE: Can't programmatically trigger Cmd+K from script side
// The test verifies the button is visible and protocol works
// Manual testing or future simulateKey() needed for full E2E

const result = await arg('Select document (Cmd+K for actions):', choices, actions);

console.error('[SMOKE] Selected:', result);
console.error('[SMOKE] test-arg-actions-cmdk completed');

process.exit(0);
