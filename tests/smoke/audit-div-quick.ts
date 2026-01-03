import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

const dir = join(process.cwd(), '.test-screenshots', 'grid-audit');
mkdirSync(dir, { recursive: true });

console.error('[AUDIT] Testing DIV prompt with grid overlay');

// Use void to not await - div will display but script continues
void div(`
  <div class="p-4">
    <h1 class="text-2xl font-bold mb-4">Header Text</h1>
    <p class="text-secondary mb-2">Body paragraph with some content</p>
    <p class="text-muted">Muted text for secondary info</p>
    <button class="mt-4 px-4 py-2 bg-blue-500 text-white rounded">Action Button</button>
  </div>
`);

// Wait for render
await new Promise(r => setTimeout(r, 1000));

console.error('[AUDIT] Capturing screenshot...');
const ss = await captureScreenshot();
const filepath = join(dir, '01-div-basic.png');
writeFileSync(filepath, Buffer.from(ss.data, 'base64'));
console.error(`[AUDIT] Screenshot saved: ${filepath}`);

process.exit(0);
