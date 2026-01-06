import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Show a simple UI element
await div(`
  <div class="flex flex-col gap-4 p-8">
    <div class="text-white text-2xl font-bold">Vibrancy Test</div>
    <div class="text-gray-300">This window should have a dark blur effect</div>
    <div class="text-gray-400 text-sm">If you can read this clearly, vibrancy is working</div>
    <div class="mt-4 p-4 bg-white/10 rounded-lg">
      <div class="text-white">Semi-transparent box (10% white)</div>
    </div>
    <div class="mt-2 p-4 bg-black/30 rounded-lg">
      <div class="text-white">Semi-transparent box (30% black)</div>
    </div>
  </div>
`);

// Wait for render
await new Promise(r => setTimeout(r, 1000));

// Capture screenshot
const screenshot = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });

const filename = `vibrancy-test-${Date.now()}.png`;
const path = join(dir, filename);
writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] ${path}`);

// Exit
process.exit(0);
