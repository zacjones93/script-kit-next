// Name: Test Section Header Height
// Description: Verifies section headers are 24px height (half of regular 48px items)

/**
 * SMOKE TEST: test-section-header-height.ts
 * 
 * This script captures a screenshot of the main menu to verify
 * section headers (RECENT, MAIN) render at ~24px height (half of
 * regular 48px list items).
 * 
 * Usage:
 *   cargo build && echo '{"type":"run","path":"'$(pwd)'/tests/smoke/test-section-header-height.ts"}' | \
 *     SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
 */

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

console.error('[SMOKE] test-section-header-height starting...');

// Give the app time to fully render the script list with frecency sections
// The main menu should show RECENT and MAIN section headers
await new Promise(resolve => setTimeout(resolve, 2000));

// Capture screenshot to verify section header heights
console.error('[SMOKE] Capturing screenshot...');
const screenshot = await captureScreenshot();
console.error(`[SMOKE] Screenshot: ${screenshot.width}x${screenshot.height}`);

// Save to test-screenshots directory
const screenshotDir = join(process.cwd(), 'test-screenshots');
mkdirSync(screenshotDir, { recursive: true });

const filename = `section-header-${Date.now()}.png`;
const filepath = join(screenshotDir, filename);
writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));

console.error(`[SCREENSHOT] Saved to: ${filepath}`);
console.error('[SMOKE] Visual verification needed: Check that RECENT/MAIN headers are ~50% height of list items');
console.error('[SMOKE] Expected: Section headers at 24px, list items at 48px');

// Exit cleanly
process.exit(0);
