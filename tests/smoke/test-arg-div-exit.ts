// Name: Integration Test - arg() -> div() -> exit flow
// Description: Tests the complete prompt flow from arg() selection through div() display to clean exit

/**
 * INTEGRATION TEST: test-arg-div-exit.ts
 * 
 * Tests the complete prompt flow:
 * 1. arg() prompts user for selection
 * 2. div() displays the result
 * 3. Script exits cleanly
 * 
 * This is a critical integration test that verifies:
 * - SDK preload works correctly
 * - JSONL protocol communication is functional
 * - Prompt chaining works (arg -> div)
 * - Script exits cleanly after user dismisses div
 * 
 * Run via stdin JSON protocol:
 * echo '{"type": "run", "path": "'$(pwd)'/tests/smoke/test-arg-div-exit.ts"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
 * 
 * Expected log sequence:
 * 1. Script starts, SDK globals available
 * 2. arg() sends JSONL message with choices
 * 3. User selects an option (or first option auto-selected in test mode)
 * 4. div() sends JSONL message with result HTML
 * 5. User dismisses div (Escape or click)
 * 6. Script exits with code 0
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
  step?: string;
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
  console.error(`[INTEGRATION] ${msg}`);
}

// =============================================================================
// Main Test
// =============================================================================

const testName = 'arg-div-exit-flow';
const testStart = Date.now();

debug('test-arg-div-exit.ts starting...');
debug(`SDK globals: arg=${typeof arg}, div=${typeof div}, md=${typeof md}, exit=${typeof exit}`);

logTest(testName, 'running', { step: 'initialization' });

try {
  // Step 1: arg() prompt with choices
  debug('Step 1: Calling arg() with choices...');
  logTest(testName, 'running', { step: 'arg-prompt' });
  
  const argStart = Date.now();
  const selection = await arg('Select an option to test the flow:', [
    { name: 'Option A - First Choice', value: 'option-a', description: 'Select this for successful test' },
    { name: 'Option B - Second Choice', value: 'option-b', description: 'Another valid option' },
    { name: 'Option C - Third Choice', value: 'option-c', description: 'Yet another option' },
  ]);
  const argDuration = Date.now() - argStart;
  
  debug(`Step 1 complete: Selected "${selection}" in ${argDuration}ms`);
  
  if (!selection) {
    logTest(testName, 'fail', { 
      error: 'arg() returned empty/null value',
      step: 'arg-prompt',
      duration_ms: argDuration
    });
    exit(1);
  }
  
  // Step 2: div() to display result
  debug('Step 2: Calling div() to display result...');
  logTest(testName, 'running', { step: 'div-display' });
  
  const divStart = Date.now();
  await div(md(`# Integration Test Result

## Selected Value
You selected: **${selection}**

## Test Steps Completed
| Step | Status |
|------|--------|
| SDK Preload | Passed |
| arg() Prompt | Passed |
| User Selection | Passed |
| div() Display | Passed |

---

### Debug Info
- Selection: \`${selection}\`
- arg() Duration: \`${argDuration}ms\`
- Timestamp: \`${new Date().toISOString()}\`

*Press Escape or click anywhere to exit and complete the test.*`));
  
  const divDuration = Date.now() - divStart;
  debug(`Step 2 complete: div() dismissed after ${divDuration}ms`);
  
  // Step 3: Clean exit
  debug('Step 3: Exiting cleanly...');
  const totalDuration = Date.now() - testStart;
  
  logTest(testName, 'pass', { 
    result: { selection, argDuration, divDuration },
    duration_ms: totalDuration,
    step: 'complete'
  });
  
  debug(`Test complete! Total duration: ${totalDuration}ms`);
  debug('test-arg-div-exit.ts exiting successfully.');
  
} catch (err) {
  const duration = Date.now() - testStart;
  debug(`Test FAILED: ${err}`);
  
  logTest(testName, 'fail', {
    error: String(err),
    duration_ms: duration
  });
  
  exit(1);
}
