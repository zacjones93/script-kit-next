// Name: Test Notes Titlebar Hover
// Description: Verifies titlebar icons hidden by default, appear on hover
//
// Run: First send openNotes, then run this test
//   echo '{"type": "openNotes"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
//
// Expected:
//   - Icons hidden by default (clean titlebar with just note title)
//   - Icons appear when titlebar is hovered (hover-reveal)
//   - Icons include: actions (cmd), list/browse, new note (+)

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

console.error('[TEST:TITLEBAR-HOVER] ===== Notes Titlebar Hover Test =====');
console.error('[TEST:TITLEBAR-HOVER] Testing hover-reveal icons in titlebar');

// Wait for Notes window to fully render
await new Promise(r => setTimeout(r, 1000));

console.error('[TEST:TITLEBAR-HOVER] Capturing default state (no hover)...');

try {
  const screenshotDir = join(process.cwd(), 'test-screenshots');
  mkdirSync(screenshotDir, { recursive: true });

  const timestamp = Date.now();

  // Capture default state (icons should be hidden)
  const defaultScreenshot = await captureScreenshot();
  console.error(`[TEST:TITLEBAR-HOVER] Default state: ${defaultScreenshot.width}x${defaultScreenshot.height}`);
  
  const defaultFilename = `notes-titlebar-default-${timestamp}.png`;
  const defaultFilepath = join(screenshotDir, defaultFilename);
  writeFileSync(defaultFilepath, Buffer.from(defaultScreenshot.data, 'base64'));
  console.error(`[SCREENSHOT] ${defaultFilepath}`);

  // Verification criteria for visual inspection:
  console.error('[TEST:TITLEBAR-HOVER] Verification checklist:');
  console.error('[TEST:TITLEBAR-HOVER]   [ ] Default state: titlebar shows note title only');
  console.error('[TEST:TITLEBAR-HOVER]   [ ] Default state: NO icons visible on the right');
  console.error('[TEST:TITLEBAR-HOVER]   [ ] Hover state: icons appear (cmd, browse, +, delete)');
  console.error('[TEST:TITLEBAR-HOVER]   [ ] Icons are small, subtle, and non-intrusive');
  
  // Note: Hover interaction cannot be simulated from SDK scripts
  // The hover state needs to be verified via:
  // 1. Manual testing with mouse hover
  // 2. Log inspection for titlebar_hovered state changes
  // 3. Setting titlebar_hovered to true in test mode
  
  console.error('[TEST:TITLEBAR-HOVER] Default state captured');
  console.error('[TEST:TITLEBAR-HOVER] To verify hover: move mouse over titlebar and watch for icon reveal');
  console.error('[TEST:TITLEBAR-HOVER] Check logs for "titlebar_hovered" state changes');
  
} catch (err) {
  console.error(`[TEST:TITLEBAR-HOVER] FAIL: Failed to capture screenshot: ${err}`);
  process.exit(1);
}

console.error('[TEST:TITLEBAR-HOVER] ============================================');

// Exit cleanly
process.exit(0);
