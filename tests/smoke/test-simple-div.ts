// Name: Test Simple Div
// Description: Debug HTML rendering with minimal example

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

console.error('[TEST] === SIMPLE DIV TEST ===');

// Super simple - just a colored box with text
const htmlContent = `
  <div class="p-6">
    <h2 class="text-white text-2xl font-bold">Hello World</h2>
    <p class="text-gray-300">This is a paragraph with gray text.</p>
    <div class="p-4 bg-green-700 rounded-lg mt-4">
      <span class="text-white">Text inside green box</span>
    </div>
    <div class="p-4 bg-blue-700 rounded-lg mt-4">
      <p class="text-white">Paragraph inside blue box</p>
    </div>
  </div>
`;

// Start div (non-blocking) - we intentionally don't await because we want to
// capture a screenshot while the div is displayed, then exit
void div(htmlContent);
await new Promise(r => setTimeout(r, 1000));
const filepath = await captureAndSave('simple-div');
console.error(`[TEST] Screenshot saved to: ${filepath}`);
process.exit(0);
