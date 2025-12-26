// Name: SDK Test - term()
// Description: Tests term() terminal prompt functionality

/**
 * SDK TEST: test-term.ts
 * 
 * Tests the term() function which displays an interactive terminal.
 * 
 * Test cases:
 * 1. term-with-command: Terminal with a pre-filled command
 * 2. term-empty: Terminal without a command (interactive mode)
 * 
 * Expected behavior:
 * - term() sends JSONL message with type: 'term'
 * - Terminal window appears with optional command
 * - User can interact with terminal
 * - Terminal output/exit status returned on close
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

debug('test-term.ts starting...');
debug(`SDK globals: term=${typeof term}, div=${typeof div}, md=${typeof md}`);

// -----------------------------------------------------------------------------
// Test 1: term with command
// -----------------------------------------------------------------------------
const test1 = 'term-with-command';
logTest(test1, 'running');
const start1 = Date.now();

try {
  debug('Test 1: term() with a command');
  
  // Execute a simple echo command in the terminal
  const result = await term('echo "Hello from terminal test"');
  
  debug(`Test 1 result: "${result}"`);
  
  // Terminal should have executed and returned
  logTest(test1, 'pass', { 
    result, 
    duration_ms: Date.now() - start1 
  });
} catch (err) {
  logTest(test1, 'fail', { error: String(err), duration_ms: Date.now() - start1 });
}

// -----------------------------------------------------------------------------
// Test 2: term without command (interactive)
// -----------------------------------------------------------------------------
const test2 = 'term-interactive';
logTest(test2, 'running');
const start2 = Date.now();

try {
  debug('Test 2: term() without command (interactive mode)');
  
  // Open terminal without a command - user can type
  const result = await term();
  
  debug(`Test 2 result: "${result}"`);
  
  // Just verify it returns without error
  logTest(test2, 'pass', { 
    result, 
    duration_ms: Date.now() - start2 
  });
} catch (err) {
  logTest(test2, 'fail', { error: String(err), duration_ms: Date.now() - start2 });
}

// -----------------------------------------------------------------------------
// Show Summary
// -----------------------------------------------------------------------------
debug('test-term.ts completed!');

await div(md(`# term() Tests Complete

All \`term()\` tests have been executed.

## Test Cases Run
1. **term-with-command**: Terminal with echo command
2. **term-interactive**: Terminal without command

---

*Check the JSONL output for detailed results*

Press Escape or click to exit.`));

debug('test-term.ts exiting...');
