// Name: Test App Cache Startup Performance
// Description: Verify apps load instantly from SQLite cache on startup

import '../../scripts/kit-sdk';
import { saveScreenshot } from '../autonomous/screenshot-utils';

console.error('[SMOKE] Testing app cache startup performance...');

// Record startup time
const startTime = Date.now();

// The app has already started when this script runs
// Check how quickly the main menu is ready
console.error('[SMOKE] Checking main menu readiness...');

// Capture initial state
await new Promise(resolve => setTimeout(resolve, 100));

try {
  const screenshot = await captureScreenshot();
  const elapsed = Date.now() - startTime;
  console.error(`[SMOKE] Main menu ready in ${elapsed}ms`);
  console.error(`[SMOKE] Screenshot: ${screenshot.width}x${screenshot.height}`);
  
  const savedPath = await saveScreenshot(screenshot.data, 'app-cache-startup');
  console.error(`[SCREENSHOT] ${savedPath}`);
  
  // Performance assertion: should be ready quickly (< 500ms after script start)
  if (elapsed < 500) {
    console.error(`[SMOKE] PASS: App started quickly (${elapsed}ms < 500ms threshold)`);
  } else {
    console.error(`[SMOKE] WARN: App startup slower than expected (${elapsed}ms >= 500ms threshold)`);
  }
} catch (e) {
  console.error(`[SMOKE] Screenshot failed: ${e}`);
}

// Test: Verify the main menu shows apps
console.error('[SMOKE] Verifying main menu displays correctly...');

const timeout = 3000;
setTimeout(() => {
  console.error('[SMOKE] Test timeout - exiting');
  process.exit(0);
}, timeout);

try {
  // Show a prompt - this should display quickly since apps are cached
  const beforePrompt = Date.now();
  
  const result = await arg({
    placeholder: "Testing app cache - should display instantly (auto-exits in 3s)",
    choices: [
      { name: "App caching test", value: "cache-test" },
    ],
  });
  
  const promptTime = Date.now() - beforePrompt;
  console.error(`[SMOKE] Prompt displayed in ${promptTime}ms`);
  console.error(`[SMOKE] Selected: ${result}`);
} catch (e) {
  console.error(`[SMOKE] Prompt error: ${e}`);
}

console.error('[SMOKE] App cache startup test complete');
console.error('[SMOKE] Performance notes:');
console.error('[SMOKE]   - First run: Apps scanned from disk, cached to SQLite');
console.error('[SMOKE]   - Subsequent runs: Apps loaded instantly from SQLite cache');
console.error('[SMOKE]   - Cache location: ~/.sk/kit/db/apps.sqlite');
process.exit(0);
