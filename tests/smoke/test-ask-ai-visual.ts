// @ts-nocheck
// Visual test for Ask AI [Tab] hint in main menu header
import '../../scripts/kit-sdk';

const fs = require('fs');
const path = require('path');

// Wait for the main menu to fully render
await new Promise(r => setTimeout(r, 1500));

// Capture screenshot
try {
  const screenshot = await captureScreenshot();
  const dir = path.join(process.cwd(), 'test-screenshots');
  fs.mkdirSync(dir, { recursive: true });
  const filePath = path.join(dir, `ask-ai-hint-${Date.now()}.png`);
  fs.writeFileSync(filePath, Buffer.from(screenshot.data, 'base64'));
  console.error(`[SCREENSHOT] Saved to: ${filePath}`);
  console.error(`[SCREENSHOT] Size: ${screenshot.data.length} bytes`);
} catch (e) {
  console.error(`[SCREENSHOT ERROR] ${e}`);
}

process.exit(0);
