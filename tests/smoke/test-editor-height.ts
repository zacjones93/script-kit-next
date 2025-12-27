// Name: Test Editor Height
// Description: Tests that editor fills the full 700px window height

import '../../scripts/kit-sdk';

console.error('[SMOKE] test-editor-height.ts starting...');

// Editor should trigger a window resize to MAX_HEIGHT (700px)
// and the editor content should fill the entire window
const code = await editor(`// This editor should fill the full 700px window height
// The window should resize from 500px to 700px when editor opens
// 
// Scroll down to verify the editor fills the space:
// Line 5
// Line 6
// Line 7
// Line 8
// Line 9
// Line 10
// Line 11
// Line 12
// Line 13
// Line 14
// Line 15
// Line 16
// Line 17
// Line 18
// Line 19
// Line 20
// Line 21
// Line 22
// Line 23
// Line 24
// Line 25
// Line 26
// Line 27
// Line 28
// Line 29
// Line 30
// Line 31
// Line 32
// Line 33
// Line 34
// Line 35
// 
// If you can see this without scrolling, the editor is too small!
// Press Cmd+Enter to submit, Escape to cancel
`, "typescript");

console.error(`[SMOKE] Editor result length: ${code?.length ?? 0}`);
console.error('[SMOKE] test-editor-height.ts completed!');
