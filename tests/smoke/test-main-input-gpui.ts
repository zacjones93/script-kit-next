import '../../scripts/kit-sdk';
import { mkdirSync, writeFileSync } from 'fs';
import { join } from 'path';

console.error('[TEST] Capturing gpui main input screenshot...');
// Wait for the window to render fully
await new Promise((resolve) => setTimeout(resolve, 600));

const screenshot = await captureScreenshot();
console.error(`[TEST] Captured: ${screenshot.width}x${screenshot.height}`);

const screenshotDir = join(process.cwd(), '.test-screenshots');
mkdirSync(screenshotDir, { recursive: true });

const filepath = join(screenshotDir, `main-input-gpui-${Date.now()}.png`);
writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] ${filepath}`);

await hide();

process.exit(0);
