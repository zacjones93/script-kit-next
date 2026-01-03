import '../../scripts/kit-sdk';

console.error('[TEST] Testing getLayoutInfo()...');

// Display a div
await div(`<div class="p-4"><h1>Hello World</h1></div>`);

// Wait for render
await new Promise(r => setTimeout(r, 1000));

console.error('[TEST] Calling getLayoutInfo()...');
try {
  const layout = await getLayoutInfo();
  console.error('[TEST] Got layout:', JSON.stringify(layout, null, 2));
} catch (e) {
  console.error('[TEST] Error:', e);
}

process.exit(0);
