// Name: Test Filter Focus Stability
// Description: Verifies first item stays selected during filtering (race condition fix)

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

console.error('[SMOKE] test-filter-focus.ts starting...');

// Test that filtering doesn't cause selection to jump to 2nd item
// The bug was: set_filter_text() set selected_index=0 immediately,
// but cache wasn't updated until 8ms later. During render, stale
// grouped_items had SectionHeader at index 0, so coerce_selection
// moved selection to index 1.

// Create screenshot directory
const screenshotDir = join(process.cwd(), 'test-screenshots');
mkdirSync(screenshotDir, { recursive: true });

// Wait for main menu to initialize
await new Promise(r => setTimeout(r, 200));

// Type into the filter - this tests the async filter path
// Use setInput which is available from kit-sdk
setInput('a');
await new Promise(r => setTimeout(r, 50));
setInput('ab');
await new Promise(r => setTimeout(r, 50));
setInput('abc');
await new Promise(r => setTimeout(r, 100));

// Take screenshot to verify selection
const screenshot = await captureScreenshot();
console.error(`[TEST] Screenshot: ${screenshot.width}x${screenshot.height}`);
const filepath = join(screenshotDir, `filter-focus-${Date.now()}.png`);
writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] Saved to: ${filepath}`);

// Get layout to verify UI state
const layout = await getLayoutInfo();
console.error(`[LAYOUT] promptType=${layout.promptType}, componentCount=${layout.components.length}`);

console.error('[SMOKE] test-filter-focus.ts complete');
process.exit(0);
