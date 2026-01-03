// Test: Window actions target the previous app, not Script Kit
import '../../scripts/kit-sdk';

export const metadata = {
  name: "Test Window Action",
  description: "Verify window actions target the menu bar owner",
};

console.error('[TEST] Starting window action test...');

// This test verifies that when we execute a window action,
// we're targeting the previous app (menu bar owner), not Script Kit itself.

// First, let's display something so we can see Script Kit is active
await div(`
  <div class="p-8 text-center">
    <h1 class="text-2xl font-bold mb-4">Window Action Test</h1>
    <p class="text-gray-400 mb-4">This window should NOT be affected by window actions.</p>
    <p class="text-gray-400">The previously focused app's window should be tiled instead.</p>
    <p class="text-yellow-400 mt-4">Press Escape to close, then test manually.</p>
  </div>
`);

console.error('[TEST] Test complete - manual verification required');
console.error('[TEST] 1. Open another app (e.g., Finder)');
console.error('[TEST] 2. Invoke Script Kit with Cmd+;');
console.error('[TEST] 3. Type "tile left" and press Enter');
console.error('[TEST] 4. The Finder window should tile, not Script Kit');

process.exit(0);
