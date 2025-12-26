// Name: SDK Test - Utility Functions
// Description: Tests wait, uuid, compile, and HTTP methods

/**
 * SDK TEST: test-utils.ts
 * 
 * Tests utility functions that don't require user interaction.
 * 
 * Test cases:
 * 1. utils-wait: wait() delay function
 * 2. utils-uuid: uuid() generation
 * 3. utils-compile: compile() template function
 * 4. http-get: get() request (if network available)
 * 5. http-post: post() request (if network available)
 * 
 * Expected behavior:
 * - wait() delays execution by specified ms
 * - uuid() generates valid v4 UUIDs
 * - compile() creates template functions
 * - HTTP methods return response data
 */

import '../../scripts/kit-sdk';

// =============================================================================
// Test Infrastructure
// =============================================================================

interface TestResult {
  test: string;
  status: 'running' | 'pass' | 'fail' | 'skip';
  timestamp: string;
  result?: unknown;
  error?: string;
  duration_ms?: number;
  expected?: string;
  actual?: string;
}

function logTest(name: string, status: TestResult['status'], extra?: Partial<TestResult>) {
  const result: TestResult = {
    test: name,
    status,
    timestamp: new Date().toISOString(),
    ...extra
  };
  console.log(JSON.stringify(result));
}

function debug(msg: string) {
  console.error(`[TEST] ${msg}`);
}

// =============================================================================
// Tests
// =============================================================================

debug('test-utils.ts starting...');
debug(`SDK globals: wait=${typeof wait}, uuid=${typeof uuid}, compile=${typeof compile}`);

// -----------------------------------------------------------------------------
// Test 1: wait() delay function
// -----------------------------------------------------------------------------
const test1 = 'utils-wait';
logTest(test1, 'running');
const start1 = Date.now();

try {
  debug('Test 1: wait() delay function');
  
  const waitStart = Date.now();
  await wait(100);
  const elapsed = Date.now() - waitStart;
  
  debug(`wait(100) took ${elapsed}ms (expected ~100ms)`);
  
  // Allow 50ms tolerance for timing variations
  if (elapsed >= 90 && elapsed <= 200) {
    logTest(test1, 'pass', { result: `${elapsed}ms`, duration_ms: Date.now() - start1 });
  } else {
    logTest(test1, 'fail', { 
      error: `wait(100) took ${elapsed}ms, expected ~100ms`,
      duration_ms: Date.now() - start1 
    });
  }
} catch (err) {
  logTest(test1, 'fail', { error: String(err), duration_ms: Date.now() - start1 });
}

// -----------------------------------------------------------------------------
// Test 2: uuid() generation
// -----------------------------------------------------------------------------
const test2 = 'utils-uuid';
logTest(test2, 'running');
const start2 = Date.now();

try {
  debug('Test 2: uuid() generation');
  
  const id1 = uuid();
  const id2 = uuid();
  
  debug(`Generated UUID 1: ${id1}`);
  debug(`Generated UUID 2: ${id2}`);
  
  const uuidRegex = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
  const checks = [
    uuidRegex.test(id1),
    uuidRegex.test(id2),
    id1 !== id2,
  ];
  
  debug(`UUIDs are unique: ${id1 !== id2}`);
  debug(`UUID format valid: ${uuidRegex.test(id1)}`);
  
  if (checks.every(Boolean)) {
    logTest(test2, 'pass', { result: id1, duration_ms: Date.now() - start2 });
  } else {
    logTest(test2, 'fail', { 
      error: 'uuid() did not generate valid unique UUIDs',
      actual: id1,
      duration_ms: Date.now() - start2 
    });
  }
} catch (err) {
  logTest(test2, 'fail', { error: String(err), duration_ms: Date.now() - start2 });
}

// -----------------------------------------------------------------------------
// Test 3: compile() template function
// -----------------------------------------------------------------------------
const test3 = 'utils-compile';
logTest(test3, 'running');
const start3 = Date.now();

try {
  debug('Test 3: compile() template function');
  
  const greet = compile('Hello, {{name}}! You are {{age}} years old.');
  const result1 = greet({ name: 'Alice', age: 30 });
  const expected = 'Hello, Alice! You are 30 years old.';
  
  debug(`Template result: ${result1}`);
  debug(`Expected: ${expected}`);
  
  // Test with missing key
  const result2 = greet({ name: 'Bob' });
  debug(`Template with missing key: ${result2}`);
  
  if (result1 === expected) {
    logTest(test3, 'pass', { result: result1, duration_ms: Date.now() - start3 });
  } else {
    logTest(test3, 'fail', { 
      error: 'compile() did not produce expected output',
      expected,
      actual: result1,
      duration_ms: Date.now() - start3 
    });
  }
} catch (err) {
  logTest(test3, 'fail', { error: String(err), duration_ms: Date.now() - start3 });
}

// -----------------------------------------------------------------------------
// Test 4: HTTP GET request
// -----------------------------------------------------------------------------
const test4 = 'http-get';
logTest(test4, 'running');
const start4 = Date.now();

try {
  debug('Test 4: HTTP GET request');
  
  const response = await get('https://httpbin.org/get');
  
  debug(`GET response has data: ${!!response.data}`);
  
  if (response.data && typeof response.data === 'object') {
    logTest(test4, 'pass', { result: 'GET request successful', duration_ms: Date.now() - start4 });
  } else {
    logTest(test4, 'fail', { 
      error: 'GET request did not return expected data',
      duration_ms: Date.now() - start4 
    });
  }
} catch (err) {
  debug(`HTTP GET test skipped (network unavailable): ${err}`);
  logTest(test4, 'skip', { error: 'Network unavailable', duration_ms: Date.now() - start4 });
}

// -----------------------------------------------------------------------------
// Test 5: HTTP POST request
// -----------------------------------------------------------------------------
const test5 = 'http-post';
logTest(test5, 'running');
const start5 = Date.now();

try {
  debug('Test 5: HTTP POST request');
  
  const response = await post('https://httpbin.org/post', { message: 'hello' });
  
  debug(`POST response has data: ${!!response.data}`);
  
  if (response.data && typeof response.data === 'object') {
    logTest(test5, 'pass', { result: 'POST request successful', duration_ms: Date.now() - start5 });
  } else {
    logTest(test5, 'fail', { 
      error: 'POST request did not return expected data',
      duration_ms: Date.now() - start5 
    });
  }
} catch (err) {
  debug(`HTTP POST test skipped (network unavailable): ${err}`);
  logTest(test5, 'skip', { error: 'Network unavailable', duration_ms: Date.now() - start5 });
}

// -----------------------------------------------------------------------------
// Test 6: Window control functions (fire-and-forget)
// -----------------------------------------------------------------------------
const test6 = 'utils-window-control';
logTest(test6, 'running');
const start6 = Date.now();

try {
  debug('Test 6: Window control functions');
  
  // These are fire-and-forget, just verify they don't throw
  await show();
  debug('show() completed');
  
  // Note: hide() and blur() would affect the test runner, so just verify they exist
  const hasHide = typeof hide === 'function';
  const hasBlur = typeof blur === 'function';
  
  debug(`hide function exists: ${hasHide}`);
  debug(`blur function exists: ${hasBlur}`);
  
  if (hasHide && hasBlur) {
    logTest(test6, 'pass', { result: 'Window control functions available', duration_ms: Date.now() - start6 });
  } else {
    logTest(test6, 'fail', { 
      error: 'Window control functions not available',
      duration_ms: Date.now() - start6 
    });
  }
} catch (err) {
  logTest(test6, 'fail', { error: String(err), duration_ms: Date.now() - start6 });
}

// -----------------------------------------------------------------------------
// Test 7: Content setters (fire-and-forget)
// -----------------------------------------------------------------------------
const test7 = 'utils-content-setters';
logTest(test7, 'running');
const start7 = Date.now();

try {
  debug('Test 7: Content setter functions');
  
  // These are fire-and-forget, just verify they don't throw
  setPanel('<div>Panel content</div>');
  debug('setPanel() called');
  
  setPreview('<div>Preview content</div>');
  debug('setPreview() called');
  
  setPrompt('<div>Prompt content</div>');
  debug('setPrompt() called');
  
  logTest(test7, 'pass', { result: 'Content setters work', duration_ms: Date.now() - start7 });
} catch (err) {
  logTest(test7, 'fail', { error: String(err), duration_ms: Date.now() - start7 });
}

// -----------------------------------------------------------------------------
// Show Summary
// -----------------------------------------------------------------------------
debug('test-utils.ts completed!');

await div(md(`# Utility Functions Tests Complete

All utility function tests have been executed.

## Test Cases Run
1. **utils-wait**: wait() delay function
2. **utils-uuid**: uuid() generation
3. **utils-compile**: compile() template function
4. **http-get**: GET request (network dependent)
5. **http-post**: POST request (network dependent)
6. **utils-window-control**: Window control functions
7. **utils-content-setters**: Content setter functions

---

*Check the JSONL output for detailed results*

Press Escape or click to exit.`));

debug('test-utils.ts exiting...');
