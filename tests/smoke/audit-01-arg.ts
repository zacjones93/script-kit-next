import '../../scripts/kit-sdk';

const fs = require('fs');
const dir = `${process.cwd()}/.test-screenshots/grid-audit`;
fs.mkdirSync(dir, { recursive: true });

console.error('[AUDIT] Testing ARG prompt with grid overlay');

// Show arg prompt with choices
const result = await arg({
  placeholder: 'Select a fruit',
  choices: [
    { name: 'Apple', description: 'A red fruit', value: 'apple' },
    { name: 'Banana', description: 'A yellow fruit', value: 'banana' },
    { name: 'Cherry', description: 'A small red fruit', value: 'cherry' },
    { name: 'Date', description: 'A sweet fruit', value: 'date' },
    { name: 'Elderberry', description: 'A purple berry', value: 'elderberry' },
  ]
});

// Wait for render then capture
await new Promise(r => setTimeout(r, 500));
const ss = await captureScreenshot();
fs.writeFileSync(`${dir}/01-arg-choices.png`, Buffer.from(ss.data, 'base64'));
console.error(`[AUDIT] Screenshot saved: ${dir}/01-arg-choices.png`);
console.error(`[AUDIT] Selected: ${result}`);

process.exit(0);
