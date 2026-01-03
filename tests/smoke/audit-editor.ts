import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

const dir = join(process.cwd(), '.test-screenshots', 'grid-audit');
mkdirSync(dir, { recursive: true });

console.error('[AUDIT] Testing EDITOR prompt with grid overlay');

// Use void to not await - editor takes (content, language)
void editor(`function hello() {
  console.log("Hello, World!");
}

// This is a code editor test
const x = 42;`, 'typescript');

await new Promise(r => setTimeout(r, 1500));

console.error('[AUDIT] Capturing editor screenshot...');
const ss = await captureScreenshot();
const filepath = join(dir, '02-editor.png');
writeFileSync(filepath, Buffer.from(ss.data, 'base64'));
console.error(`[AUDIT] Screenshot saved: ${filepath}`);

process.exit(0);
