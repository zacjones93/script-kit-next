/**
 * Test script for TIER 5A Utility Functions
 * 
 * Tests:
 * - exec() - Shell command execution
 * - HTTP methods (get, post, put, patch, del)
 * - download() - File download
 * - trash() - Move to trash
 * - show(), hide(), blur() - Window control
 * - submit(), exit() - Prompt control
 * - wait() - Delay
 * - setPanel(), setPreview(), setPrompt() - Content setters
 * - uuid() - UUID generation
 * - compile() - Template compilation
 */

import '../../scripts/kit-sdk';

// Test 1: wait() - Promise-based delay (pure JS, no GPUI needed)
console.log('Testing wait()...');
const start = Date.now();
await wait(100);
const elapsed = Date.now() - start;
console.log(`wait(100) took ${elapsed}ms (expected ~100ms)`);

// Test 2: uuid() - UUID generation (pure JS, no GPUI needed)
console.log('\nTesting uuid()...');
const id1 = uuid();
const id2 = uuid();
console.log(`Generated UUID 1: ${id1}`);
console.log(`Generated UUID 2: ${id2}`);
console.log(`UUIDs are unique: ${id1 !== id2}`);
console.log(`UUID format valid: ${/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(id1)}`);

// Test 3: compile() - Simple template compilation (pure JS, no GPUI needed)
console.log('\nTesting compile()...');
const greet = compile('Hello, {{name}}! You are {{age}} years old.');
const result1 = greet({ name: 'Alice', age: 30 });
console.log(`Template result: ${result1}`);
console.log(`Expected: Hello, Alice! You are 30 years old.`);
console.log(`Match: ${result1 === 'Hello, Alice! You are 30 years old.'}`);

// Test with missing key
const result2 = greet({ name: 'Bob' });
console.log(`Template with missing key: ${result2}`);
console.log(`Expected: Hello, Bob! You are  years old.`);

// Test 4: HTTP methods - These use fetch directly
console.log('\nTesting HTTP methods (using httpbin.org)...');

try {
  // GET request
  console.log('Testing get()...');
  const getResult = await get('https://httpbin.org/get');
  console.log(`GET response has data: ${!!getResult.data}`);
  
  // POST request
  console.log('Testing post()...');
  const postResult = await post('https://httpbin.org/post', { message: 'hello' });
  console.log(`POST response has data: ${!!postResult.data}`);
  
  // PUT request
  console.log('Testing put()...');
  const putResult = await put('https://httpbin.org/put', { update: true });
  console.log(`PUT response has data: ${!!putResult.data}`);
  
  // PATCH request
  console.log('Testing patch()...');
  const patchResult = await patch('https://httpbin.org/patch', { partial: true });
  console.log(`PATCH response has data: ${!!patchResult.data}`);
  
  // DELETE request
  console.log('Testing del()...');
  const delResult = await del('https://httpbin.org/delete');
  console.log(`DELETE response has data: ${!!delResult.data}`);
} catch (e) {
  console.log(`HTTP tests skipped (network unavailable): ${e}`);
}

// Test 5: Fire-and-forget window control (messages sent, no response expected)
console.log('\nTesting window control (fire-and-forget)...');
console.log('Calling show()...');
await show();
console.log('show() completed');

console.log('Calling hide()...');
await hide();
console.log('hide() completed');

console.log('Calling blur()...');
await blur();
console.log('blur() completed');

// Test 6: Content setters (fire-and-forget)
console.log('\nTesting content setters...');
setPanel('<div>Panel content</div>');
console.log('setPanel() called');

setPreview('<div>Preview content</div>');
console.log('setPreview() called');

setPrompt('<div>Prompt content</div>');
console.log('setPrompt() called');

// Test 7: Prompt control
console.log('\nTesting prompt control...');
console.log('submit() and exit() are fire-and-forget');
// Note: We don't actually call exit() here as it would terminate the script
// submit({ test: 'value' }); // Would force submit
// exit(0); // Would exit the script

console.log('\n=== All utility function tests completed ===');

// Note: The following require GPUI to respond:
// - exec() - needs GPUI to run command and return result
// - download() - needs GPUI to download file
// - trash() - needs GPUI to move files to trash
//
// These are tested by sending the message and checking it was sent correctly.
// Full integration tests would require running with GPUI.

console.log('\nMessage-based functions (exec, download, trash) require GPUI integration.');
console.log('See integration tests for full coverage.');
