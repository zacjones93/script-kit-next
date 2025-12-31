// Name: Quick Test - div() Container Options
// Description: Quick visual test for container options - exits after first screenshot

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

console.error('[TEST] Starting div container options quick test...');

const SCREENSHOT_DIR = join(process.cwd(), '.test-screenshots');

// Test: Transparent background with gradient content
console.error('[TEST] Testing transparent container + gradient content');

// Set up the div with transparent background and no padding
// The content provides its own gradient background
const divPromise = div(`
  <div class="bg-gradient-to-br from-purple-600 via-pink-500 to-orange-400 p-8 rounded-xl text-white h-full w-full flex flex-col justify-center items-center">
    <h1 class="text-3xl font-bold mb-4">Transparent Container Test</h1>
    <p class="text-lg mb-2">containerBg: "transparent"</p>
    <p class="text-lg mb-2">containerPadding: "none"</p>
    <p class="text-sm opacity-75 mt-4">The gradient should extend edge-to-edge</p>
  </div>
`, { containerBg: 'transparent', containerPadding: 'none' });

// Wait, capture screenshot, then exit
setTimeout(async () => {
  try {
    console.error('[TEST] Capturing screenshot...');
    const screenshot = await captureScreenshot();
    console.error(`[TEST] Captured: ${screenshot.width}x${screenshot.height}`);
    
    mkdirSync(SCREENSHOT_DIR, { recursive: true });
    const filename = `div-options-transparent-gradient-${Date.now()}.png`;
    const filepath = join(SCREENSHOT_DIR, filename);
    writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
    console.error(`[SCREENSHOT] ${filepath}`);
    
    console.error('[TEST] Quick test complete - exiting');
    process.exit(0);
  } catch (err) {
    console.error('[TEST] Error:', err);
    process.exit(1);
  }
}, 1000);
