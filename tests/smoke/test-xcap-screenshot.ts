import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Display something
await div(`<div class="p-8 bg-blue-500 text-white text-2xl">Screenshot Test</div>`);
await new Promise(r => setTimeout(r, 1000));

// Capture screenshot
const screenshot = await captureScreenshot();
console.error(`[TEST] Screenshot captured: ${screenshot.width}x${screenshot.height}`);

// Save it
const dir = join(process.cwd(), '.mocks/test');
mkdirSync(dir, { recursive: true });
const path = join(dir, 'xcap-test.png');
writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
console.error(`[TEST] Saved to: ${path}`);

process.exit(0);
