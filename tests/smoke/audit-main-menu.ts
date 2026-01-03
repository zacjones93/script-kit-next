import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

const dir = join(process.cwd(), '.test-screenshots', 'grid-audit');
mkdirSync(dir, { recursive: true });

console.error('[AUDIT] Testing MAIN MENU (script list) with grid overlay');

// Wait for the main menu to render (it shows on app start)
await new Promise(r => setTimeout(r, 1000));

console.error('[AUDIT] Capturing main menu screenshot...');
const ss = await captureScreenshot();
const filepath = join(dir, '03-main-menu.png');
writeFileSync(filepath, Buffer.from(ss.data, 'base64'));
console.error(`[AUDIT] Screenshot saved: ${filepath}`);

process.exit(0);
