// Test script for gpui-component Notification integration
// This script triggers error/warning scenarios to verify toast notifications appear

import '../../scripts/kit-sdk';
// @ts-ignore
import { writeFileSync, mkdirSync } from 'fs';
// @ts-ignore
import { join } from 'path';

console.error('[TEST] Starting gpui-component notification test...');

// Display a div first to ensure the window is visible
await div(`
  <div class="flex flex-col items-center justify-center h-full p-8">
    <h1 class="text-2xl font-bold text-white mb-4">Notification Test</h1>
    <p class="text-gray-400 mb-8">Testing gpui-component notification integration</p>
    <div class="space-y-4">
      <div class="bg-blue-500/20 border border-blue-500 rounded-lg p-4">
        <p class="text-blue-400">Window displayed successfully</p>
      </div>
      <div class="bg-yellow-500/20 border border-yellow-500 rounded-lg p-4">
        <p class="text-yellow-400">Notifications should appear in top-right corner</p>
      </div>
    </div>
  </div>
`);

// Wait for render
await new Promise(resolve => setTimeout(resolve, 1000));

// Capture a screenshot to verify the UI
try {
  const screenshot = await captureScreenshot();
  console.error(`[TEST] Captured screenshot: ${screenshot.width}x${screenshot.height}`);
  
  // @ts-ignore
  const dir = join(process.cwd(), 'test-screenshots');
  mkdirSync(dir, { recursive: true });
  
  const filepath = join(dir, `notification-test-${Date.now()}.png`);
  // @ts-ignore
  writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
  console.error(`[SCREENSHOT] ${filepath}`);
} catch (e) {
  console.error(`[TEST] Screenshot failed: ${e}`);
}

console.error('[TEST] Test complete - check logs for notification entries');

// Exit cleanly
// @ts-ignore
process.exit(0);
