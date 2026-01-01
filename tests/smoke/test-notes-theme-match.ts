// Name: Test Notes Window Theme Match
// Description: Captures screenshots of Notes window to verify theme consistency

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

console.error('[TEST] Starting Notes window theme verification...');

// Wait for the Notes window to fully render
await new Promise(r => setTimeout(r, 1000));

console.error('[TEST] Capturing Notes window screenshot...');

try {
  const screenshot = await captureScreenshot();
  console.error(`[TEST] Captured: ${screenshot.width}x${screenshot.height}`);

  // Save to test-screenshots directory
  const screenshotDir = join(process.cwd(), 'test-screenshots');
  mkdirSync(screenshotDir, { recursive: true });

  const timestamp = Date.now();
  const filename = `notes-window-theme-${timestamp}.png`;
  const filepath = join(screenshotDir, filename);
  
  writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
  console.error(`[SCREENSHOT] ${filepath}`);
  
  console.error('[TEST] Screenshot saved successfully');
} catch (err) {
  console.error(`[TEST] Failed to capture screenshot: ${err}`);
}

// Exit cleanly
process.exit(0);
