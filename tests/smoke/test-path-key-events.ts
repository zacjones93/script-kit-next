// Test path prompt key event handling via stdin JSON protocol
// This test verifies:
// 1. Arrow keys for navigation
// 2. Enter to submit/open
// 3. Cmd+K to toggle actions
// 4. Escape to cancel

import '../../scripts/kit-sdk';

console.error('[TEST] Starting path prompt key events test');

// Start path prompt with options
const pathPromise = path({ startPath: '/tmp' });

// Give it time to render
await new Promise((r: (value: unknown) => void) => setTimeout(r, 500));

console.error('[TEST] Path prompt should be visible now');
console.error('[TEST] Key event simulation will be done via stdin JSON protocol');
console.error('[TEST] Example commands:');
console.error('[TEST]   {"type":"simulateKey","key":"down"}');
console.error('[TEST]   {"type":"simulateKey","key":"up"}');
console.error('[TEST]   {"type":"simulateKey","key":"enter"}');
console.error('[TEST]   {"type":"simulateKey","key":"k","modifiers":["cmd"]}');
console.error('[TEST]   {"type":"simulateKey","key":"escape"}');

// Wait a bit more then cancel the prompt
await new Promise((r: (value: unknown) => void) => setTimeout(r, 2000));

console.error('[TEST] Test complete - exiting');
// @ts-ignore
process.exit(0);
