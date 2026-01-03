import '../../scripts/kit-sdk';

const fs = require('fs');
const screenshotDir = `${process.cwd()}/.test-screenshots/grid-audit`;
fs.mkdirSync(screenshotDir, { recursive: true });

console.error('[TEST] DIV PROMPT - Testing with grid overlay');

await div(`
  <div class="p-4">
    <h1 class="text-xl font-bold mb-2">Header Text</h1>
    <p class="text-secondary">Body paragraph with some content</p>
    <button class="mt-4 px-4 py-2 bg-blue-500 text-white rounded">Action Button</button>
  </div>
`);

await new Promise(r => setTimeout(r, 800));

const screenshot = await captureScreenshot();
const layout = await getLayoutInfo();

fs.writeFileSync(`${screenshotDir}/01-div-basic.png`, Buffer.from(screenshot.data, 'base64'));

console.error(`\n=== DIV PROMPT LAYOUT ===`);
console.error(`Screenshot: ${screenshotDir}/01-div-basic.png`);
console.error(`Window: ${layout.windowWidth}x${layout.windowHeight}`);
console.error(`Prompt type: ${layout.promptType}`);
console.error(`Components: ${layout.components.length}`);

for (const c of layout.components) {
  const bounds = `${Math.round(c.bounds.width)}x${Math.round(c.bounds.height)} at (${Math.round(c.bounds.x)}, ${Math.round(c.bounds.y)})`;
  console.error(`  [${c.type}] ${c.name}: ${bounds}`);
  if (c.boxModel?.padding) {
    const p = c.boxModel.padding;
    console.error(`    padding: T${p.top} R${p.right} B${p.bottom} L${p.left}`);
  }
  if (c.boxModel?.gap !== undefined) {
    console.error(`    gap: ${c.boxModel.gap}`);
  }
}

process.exit(0);
