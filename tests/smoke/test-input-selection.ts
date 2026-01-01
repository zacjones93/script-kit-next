// Test script for TextInputState integration
// Tests: cursor positioning, selection, clipboard operations
import '../../scripts/kit-sdk';

console.error('[TEST] Starting input selection test...');

// Test 1: Basic arg() with no choices - tests text input
const result = await arg("Type 'hello' then select all (Cmd+A) and copy (Cmd+C):");
console.error(`[TEST] arg() result: ${result}`);

// Test 2: env() prompt - same input handling
const envResult = await env("TEST_VAR", "Type something here and test selection:");
console.error(`[TEST] env() result: ${envResult}`);

console.error('[TEST] Test complete. Check that:');
console.error('  - Cursor appears at correct position');
console.error('  - Shift+arrows selects text');
console.error('  - Cmd+A selects all');
console.error('  - Cmd+C/V/X work for clipboard');
console.error('  - Alt+arrows move by word');

exit(0);
