// Name: Test Toast Visual
// Description: Captures toast notifications using supported HTML structure

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

const screenshotDir = join(process.cwd(), 'test-screenshots');
mkdirSync(screenshotDir, { recursive: true });

async function captureAndSave(name: string): Promise<string> {
  const screenshot = await captureScreenshot({ hiDpi: true });
  const filepath = join(screenshotDir, `${name}-${Date.now()}.png`);
  writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
  console.error(`[SCREENSHOT] ${filepath}`);
  return filepath;
}

console.error('[TEST] === TOAST VISUAL TEST ===');

// Use simpler HTML structure - spans and paragraphs work better than nested divs
const htmlContent = `
  <div class="p-4">
    <h2 class="text-xl font-bold text-white mb-4">Toast Notification Styles</h2>
    
    <div class="p-3 bg-green-800 border border-green-500 rounded-lg mb-3">
      <p class="text-green-100 font-bold">✓ Success Toast</p>
      <p class="text-green-300 text-sm">Operation completed successfully</p>
    </div>
    
    <div class="p-3 bg-yellow-800 border border-yellow-500 rounded-lg mb-3">
      <p class="text-yellow-100 font-bold">⚠ Warning Toast</p>
      <p class="text-yellow-300 text-sm">Please review before continuing</p>
    </div>
    
    <div class="p-3 bg-red-800 border border-red-500 rounded-lg mb-3">
      <p class="text-red-100 font-bold">✕ Error Toast</p>
      <p class="text-red-300 text-sm">Something went wrong</p>
    </div>
    
    <div class="p-3 bg-blue-800 border border-blue-500 rounded-lg">
      <p class="text-blue-100 font-bold">ℹ Info Toast</p>
      <p class="text-blue-300 text-sm">Here is some helpful information</p>
    </div>
  </div>
`;

// Start div (non-blocking) - we intentionally don't await because we want to
// capture a screenshot while the div is displayed, then exit
void div(htmlContent);
await new Promise(r => setTimeout(r, 1000));
const filepath = await captureAndSave('toast-styles');
console.error(`[TEST] Screenshot saved to: ${filepath}`);
process.exit(0);
