import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

export const metadata = {
  name: "Visual Audit - Terminal Prompt",
  description: "Captures term() prompt for visual audit",
};

console.error('[AUDIT] Starting terminal visual audit...');

// Create screenshot directory
const screenshotDir = join(process.cwd(), '.test-screenshots', 'grid-audit');
mkdirSync(screenshotDir, { recursive: true });

// Capture after terminal renders some output
setTimeout(async () => {
  try {
    const screenshot = await captureScreenshot();
    console.error(`[AUDIT] Captured terminal: ${screenshot.width}x${screenshot.height}`);
    
    const filename = '06-terminal.png';
    const filepath = join(screenshotDir, filename);
    writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
    console.error(`[SCREENSHOT] ${filepath}`);
    
    process.exit(0);
  } catch (err) {
    console.error('[AUDIT] Screenshot failed:', err);
    process.exit(1);
  }
}, 2500); // More time for terminal to render

// Display terminal with a simple command
await term("echo 'Terminal Visual Audit Test' && ls -la");
