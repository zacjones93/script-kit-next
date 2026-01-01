// Name: Test Notes Single View
// Description: Verifies Notes window displays single note without sidebar
//
// Run: First send openNotes, then run this test:
//   echo '{"type": "openNotes"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1 &
//   sleep 1
//   # Then capture screenshot from within a script
//
// Expected:
//   - No sidebar visible (editor spans full width)
//   - Editor area takes up most of window
//   - Note title shown in titlebar
//   - Footer with character count visible

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

console.error('[TEST:SINGLE-VIEW] ===== Notes Single View Test =====');
console.error('[TEST:SINGLE-VIEW] Testing that Notes displays single note without sidebar');

// Wait for Notes window to fully render
await new Promise(r => setTimeout(r, 1000));

console.error('[TEST:SINGLE-VIEW] Capturing screenshot...');

try {
  const screenshot = await captureScreenshot();
  console.error(`[TEST:SINGLE-VIEW] Captured: ${screenshot.width}x${screenshot.height}`);

  // Save to test-screenshots directory
  const screenshotDir = join(process.cwd(), 'test-screenshots');
  mkdirSync(screenshotDir, { recursive: true });

  const timestamp = Date.now();
  const filename = `notes-single-view-${timestamp}.png`;
  const filepath = join(screenshotDir, filename);
  
  writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
  console.error(`[SCREENSHOT] ${filepath}`);
  
  // Verification criteria for visual inspection:
  console.error('[TEST:SINGLE-VIEW] Verification checklist:');
  console.error('[TEST:SINGLE-VIEW]   [ ] No sidebar visible on the left');
  console.error('[TEST:SINGLE-VIEW]   [ ] Editor fills the full window width');
  console.error('[TEST:SINGLE-VIEW]   [ ] Titlebar shows note title (or "Untitled Note")');
  console.error('[TEST:SINGLE-VIEW]   [ ] Footer shows character count');
  console.error('[TEST:SINGLE-VIEW]   [ ] Clean, minimal UI without list of notes');
  
  // Width check - single view should have wider content area
  // A sidebar layout typically has ~200-250px sidebar, leaving ~650-700px for content
  // Single view should use full ~900px width
  if (screenshot.width >= 850) {
    console.error(`[TEST:SINGLE-VIEW] PASS: Window width ${screenshot.width}px suggests full-width layout`);
  } else {
    console.error(`[TEST:SINGLE-VIEW] WARN: Window width ${screenshot.width}px - may have sidebar`);
  }

  console.error('[TEST:SINGLE-VIEW] Screenshot saved - READ the file to verify no sidebar');
  
} catch (err) {
  console.error(`[TEST:SINGLE-VIEW] FAIL: Failed to capture screenshot: ${err}`);
  process.exit(1);
}

console.error('[TEST:SINGLE-VIEW] ============================================');

// Exit cleanly
process.exit(0);
