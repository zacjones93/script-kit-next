import '../../scripts/kit-sdk';

const fs = require('fs');
const dir = `${process.cwd()}/.test-screenshots/grid-audit`;
fs.mkdirSync(dir, { recursive: true });

console.error('[AUDIT] Testing DIV prompt - basic HTML');

await div(`
  <div class="p-4">
    <h1 class="text-2xl font-bold mb-4">Header Text</h1>
    <p class="text-secondary mb-2">Body paragraph with some content</p>
    <p class="text-muted">Muted text for secondary info</p>
    <button class="mt-4 px-4 py-2 bg-blue-500 text-white rounded">Action Button</button>
  </div>
`);

await new Promise(r => setTimeout(r, 800));
const ss = await captureScreenshot();
fs.writeFileSync(`${dir}/01-div-basic.png`, Buffer.from(ss.data, 'base64'));
console.error(`[AUDIT] Screenshot: ${dir}/01-div-basic.png`);

process.exit(0);
