// Name: SDK Test - md()
// Description: Tests md() markdown to HTML conversion

/**
 * SDK TEST: test-md.ts
 * 
 * Tests the md() function which converts Markdown to HTML.
 * This is a synchronous function that doesn't require user interaction.
 * 
 * Test cases:
 * 1. md-headings: H1, H2, H3 conversion
 * 2. md-formatting: Bold, italic, code
 * 3. md-lists: Unordered list conversion
 * 4. md-combined: All features together
 * 
 * Expected behavior:
 * - md() is synchronous (no Promise)
 * - Markdown syntax is converted to HTML tags
 * - Output can be passed to div()
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

function assertContains(actual: string, expected: string, message: string): boolean {
  if (!actual.includes(expected)) {
    debug(`FAIL: ${message}`);
    debug(`  Expected to contain: ${expected}`);
    debug(`  Actual: ${actual}`);
    return false;
  }
  return true;
}

// =============================================================================
// Tests
// =============================================================================

debug('test-md.ts starting...');
debug(`SDK globals: arg=${typeof arg}, div=${typeof div}, md=${typeof md}`);

// -----------------------------------------------------------------------------
// Test 1: Headings
// -----------------------------------------------------------------------------
const test1 = 'md-headings';
logTest(test1, 'running');
const start1 = Date.now();

try {
  debug('Test 1: md() heading conversion');
  
  const input = `# Heading 1
## Heading 2
### Heading 3`;
  
  const result = md(input);
  debug(`Test 1 input: ${input.replace(/\n/g, '\\n')}`);
  debug(`Test 1 output: ${result.replace(/\n/g, '\\n')}`);
  
  const checks = [
    assertContains(result, '<h1>Heading 1</h1>', 'H1 conversion'),
    assertContains(result, '<h2>Heading 2</h2>', 'H2 conversion'),
    assertContains(result, '<h3>Heading 3</h3>', 'H3 conversion'),
  ];
  
  if (checks.every(Boolean)) {
    logTest(test1, 'pass', { result: 'all headings converted', duration_ms: Date.now() - start1 });
  } else {
    logTest(test1, 'fail', { 
      error: 'Some headings not converted correctly',
      actual: result,
      duration_ms: Date.now() - start1 
    });
  }
} catch (err) {
  logTest(test1, 'fail', { error: String(err), duration_ms: Date.now() - start1 });
}

// -----------------------------------------------------------------------------
// Test 2: Text formatting
// -----------------------------------------------------------------------------
const test2 = 'md-formatting';
logTest(test2, 'running');
const start2 = Date.now();

try {
  debug('Test 2: md() text formatting');
  
  const input = `**bold text** and *italic text*`;
  const result = md(input);
  
  debug(`Test 2 input: ${input}`);
  debug(`Test 2 output: ${result}`);
  
  const checks = [
    assertContains(result, '<strong>bold text</strong>', 'Bold conversion'),
    assertContains(result, '<em>italic text</em>', 'Italic conversion'),
  ];
  
  if (checks.every(Boolean)) {
    logTest(test2, 'pass', { result: 'formatting converted', duration_ms: Date.now() - start2 });
  } else {
    logTest(test2, 'fail', { 
      error: 'Formatting not converted correctly',
      actual: result,
      duration_ms: Date.now() - start2 
    });
  }
} catch (err) {
  logTest(test2, 'fail', { error: String(err), duration_ms: Date.now() - start2 });
}

// -----------------------------------------------------------------------------
// Test 3: Lists
// -----------------------------------------------------------------------------
const test3 = 'md-lists';
logTest(test3, 'running');
const start3 = Date.now();

try {
  debug('Test 3: md() list conversion');
  
  const input = `- Item one
- Item two
- Item three`;
  
  const result = md(input);
  
  debug(`Test 3 input: ${input.replace(/\n/g, '\\n')}`);
  debug(`Test 3 output: ${result.replace(/\n/g, '\\n')}`);
  
  const checks = [
    assertContains(result, '<li>Item one</li>', 'List item 1'),
    assertContains(result, '<li>Item two</li>', 'List item 2'),
    assertContains(result, '<li>Item three</li>', 'List item 3'),
    assertContains(result, '<ul>', 'UL wrapper'),
  ];
  
  if (checks.every(Boolean)) {
    logTest(test3, 'pass', { result: 'list converted', duration_ms: Date.now() - start3 });
  } else {
    logTest(test3, 'fail', { 
      error: 'List not converted correctly',
      actual: result,
      duration_ms: Date.now() - start3 
    });
  }
} catch (err) {
  logTest(test3, 'fail', { error: String(err), duration_ms: Date.now() - start3 });
}

// -----------------------------------------------------------------------------
// Test 4: Combined features
// -----------------------------------------------------------------------------
const test4 = 'md-combined';
logTest(test4, 'running');
const start4 = Date.now();

try {
  debug('Test 4: md() combined features');
  
  const input = `# Welcome

This is **bold** and *italic* text.

## Features
- Feature one
- Feature two

### Details
More content here.`;
  
  const result = md(input);
  
  debug(`Test 4 output length: ${result.length} chars`);
  
  const checks = [
    assertContains(result, '<h1>Welcome</h1>', 'H1'),
    assertContains(result, '<h2>Features</h2>', 'H2'),
    assertContains(result, '<h3>Details</h3>', 'H3'),
    assertContains(result, '<strong>bold</strong>', 'Bold'),
    assertContains(result, '<em>italic</em>', 'Italic'),
    assertContains(result, '<li>Feature one</li>', 'List item'),
  ];
  
  if (checks.every(Boolean)) {
    logTest(test4, 'pass', { result: 'all features work together', duration_ms: Date.now() - start4 });
  } else {
    logTest(test4, 'fail', { 
      error: 'Combined features not working correctly',
      actual: result,
      duration_ms: Date.now() - start4 
    });
  }
} catch (err) {
  logTest(test4, 'fail', { error: String(err), duration_ms: Date.now() - start4 });
}

// -----------------------------------------------------------------------------
// Show Results
// -----------------------------------------------------------------------------
debug('test-md.ts completed!');

// Display summary using div() with md()
await div(md(`# md() Tests Complete

All \`md()\` markdown conversion tests have been executed.

## Test Cases Run
1. **md-headings**: H1, H2, H3 conversion
2. **md-formatting**: Bold and italic
3. **md-lists**: Unordered list items
4. **md-combined**: All features together

---

*Check the JSONL output for detailed results*

Press Escape or click to exit.`));

debug('test-md.ts exiting...');
