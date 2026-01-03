import '../../scripts/kit-sdk';

// Helper to save screenshots
const screenshotDir = `${process.cwd()}/.test-screenshots/grid-audit`;
const fs = require('fs');
fs.mkdirSync(screenshotDir, { recursive: true });

async function capturePrompt(name: string, delay = 500) {
  await new Promise(r => setTimeout(r, delay));
  const screenshot = await captureScreenshot();
  const layout = await getLayoutInfo();
  
  const filepath = `${screenshotDir}/${name}.png`;
  fs.writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
  
  console.error(`\n=== ${name.toUpperCase()} ===`);
  console.error(`Screenshot: ${filepath}`);
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
  
  return layout;
}

// Start testing
console.error('[AUDIT] Starting comprehensive grid audit of all prompts...');

// Test div() which we can control
console.error('\n[TEST] div() prompt - basic HTML');
await div(`
  <div class="p-4">
    <h1 class="text-xl font-bold mb-2">Header Text</h1>
    <p class="text-secondary">Body paragraph with some content</p>
    <button class="mt-4 px-4 py-2 bg-blue-500 text-white rounded">Action Button</button>
  </div>
`);
const divLayout = await capturePrompt('div-basic');

console.error('\n[AUDIT] Basic div audit complete. Check .test-screenshots/grid-audit/ for visual results.');

process.exit(0);
