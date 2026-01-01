// Name: Test Notes Auto Resize
// Description: Verifies Notes window height grows with content
//
// Run: First send openNotes, then run this test
//   echo '{"type": "openNotes"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
//
// Expected:
//   - Window height increases as content is added
//   - Height changes logged for verification
//   - No scrollbar when content fits

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

console.error('[TEST:AUTO-RESIZE] ===== Notes Auto Resize Test =====');
console.error('[TEST:AUTO-RESIZE] Testing that window height grows with content');

// Wait for Notes window to fully render
await new Promise(r => setTimeout(r, 1000));

console.error('[TEST:AUTO-RESIZE] Capturing initial state...');

try {
  // Capture initial screenshot
  const initialScreenshot = await captureScreenshot();
  console.error(`[TEST:AUTO-RESIZE] Initial size: ${initialScreenshot.width}x${initialScreenshot.height}`);

  const screenshotDir = join(process.cwd(), 'test-screenshots');
  mkdirSync(screenshotDir, { recursive: true });

  const timestamp = Date.now();
  
  // Save initial screenshot
  const initialFilename = `notes-auto-resize-initial-${timestamp}.png`;
  const initialFilepath = join(screenshotDir, initialFilename);
  writeFileSync(initialFilepath, Buffer.from(initialScreenshot.data, 'base64'));
  console.error(`[SCREENSHOT] ${initialFilepath}`);

  // Verification criteria for visual inspection:
  console.error('[TEST:AUTO-RESIZE] Verification checklist:');
  console.error('[TEST:AUTO-RESIZE]   [ ] Window starts at reasonable default height');
  console.error('[TEST:AUTO-RESIZE]   [ ] Check logs for RESIZE events when content grows');
  console.error('[TEST:AUTO-RESIZE]   [ ] Window height should increase with multi-line content');
  console.error('[TEST:AUTO-RESIZE]   [ ] Content should not overflow or require scrolling');
  
  // Note: Actual content addition would require interaction with the Notes window
  // which isn't directly possible from SDK scripts. This test captures baseline state.
  // The auto-resize behavior should be verified by:
  // 1. Looking at RESIZE logs in stderr
  // 2. Manual testing or automated UI interaction
  
  console.error('[TEST:AUTO-RESIZE] Initial state captured');
  console.error('[TEST:AUTO-RESIZE] To verify auto-resize: add content to Notes window and watch logs for RESIZE events');
  
} catch (err) {
  console.error(`[TEST:AUTO-RESIZE] FAIL: Failed to capture screenshot: ${err}`);
  process.exit(1);
}

console.error('[TEST:AUTO-RESIZE] ============================================');

// Exit cleanly
process.exit(0);
