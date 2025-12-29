// Name: Malformed JSONL Test
// Description: Tests that the app gracefully handles malformed JSONL from scripts
// This script intentionally sends bad data mixed with good data to verify recovery

import '../../scripts/kit-sdk';

console.error('[MALFORMED-TEST] Starting malformed JSONL test...');

// Helper to send raw data to stdout (bypassing SDK validation)
function sendRaw(data: string) {
  console.log(data);
}

// Test 1: Send some malformed lines followed by a valid message
console.error('[MALFORMED-TEST] Test 1: Invalid JSON mixed with valid messages');

// These should be logged/skipped by the app
sendRaw('This is not JSON at all');
sendRaw('{ broken json }');
sendRaw('{"incomplete": json');
sendRaw('{"type": "nonexistent", "id": "test"}'); // Unknown type
sendRaw('{}'); // Missing type field

// Small delay to ensure processing
await new Promise(r => setTimeout(r, 100));

console.error('[MALFORMED-TEST] Test 2: Various edge cases');

// More edge cases
sendRaw('null'); // Valid JSON but not an object
sendRaw('[]'); // Array instead of object
sendRaw('123'); // Number instead of object
sendRaw('"string"'); // String instead of object
sendRaw('{"type": null}'); // Null type field
sendRaw('{"type": 123}'); // Wrong type for type field
sendRaw('{"type": "arg"}'); // Missing required fields

// Wait a bit
await new Promise(r => setTimeout(r, 100));

console.error('[MALFORMED-TEST] Test 3: Binary/control characters (edge case)');

// Send some binary-ish data that might trip up parsers
sendRaw('\x00\x01\x02'); // Null bytes
sendRaw('{"type": "arg", "id": "test\x00embedded"}'); // Embedded null

// Wait for processing
await new Promise(r => setTimeout(r, 100));

console.error('[MALFORMED-TEST] Test 4: Very long invalid line');

// Very long line that's not valid JSON (should be truncated in error logs)
sendRaw('x'.repeat(1000));

// Wait for processing
await new Promise(r => setTimeout(r, 100));

console.error('[MALFORMED-TEST] Test 5: Unicode edge cases');

// Unicode in unexpected places
sendRaw('{"type": ""}'); // Empty type
sendRaw('{"type": "beep", "extra": "\ud83d\ude00"}'); // Emoji (should parse fine but unknown field)

// Now send a valid message to verify recovery
console.error('[MALFORMED-TEST] Test 6: Verify app still works after bad data');

// Use the SDK to send a proper div - this proves the app recovered
await div(`
  <div class="p-4 text-center">
    <h1 class="text-2xl font-bold text-green-400">Recovery Test Passed!</h1>
    <p class="text-gray-300 mt-2">
      The app successfully recovered from malformed JSONL.
    </p>
    <p class="text-gray-400 text-sm mt-4">
      Check the logs for parse error warnings.
    </p>
  </div>
`, "flex items-center justify-center min-h-64");

console.error('[MALFORMED-TEST] Test complete. Check logs for:');
console.error('  - Parse error warnings with position context');
console.error('  - Malformed line counts');
console.error('  - Graceful recovery messages');

// Exit with success code (the test passes if the app didn't crash)
process.exit(0);
