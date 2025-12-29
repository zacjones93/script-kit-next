// Name: SDK Test - Prompt Flow Integration
// Description: Tests the complete prompt flow including early exit scenarios

/**
 * SDK TEST: test-prompt-flow.ts
 * 
 * Tests prompt flow integration including:
 * 1. Sequential prompt chaining (arg -> arg -> div)
 * 2. Early exit handling (user presses Escape)
 * 3. Empty value handling
 * 4. State preservation between prompts
 * 
 * Run via stdin JSON protocol:
 * echo '{"type": "run", "path": "'$(pwd)'/tests/sdk/test-prompt-flow.ts"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
 * 
 * Expected behavior:
 * - Each prompt should display and wait for user input
 * - Values should be preserved and passed between prompts
 * - Early exit (Escape) should return empty string or handle gracefully
 * - Script should exit cleanly after all prompts complete
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
  console.error(`[FLOW_TEST] ${msg}`);
}

// =============================================================================
// Tests
// =============================================================================

debug('test-prompt-flow.ts starting...');
debug(`SDK globals: arg=${typeof arg}, div=${typeof div}, md=${typeof md}`);

// Track collected values for state preservation test
const collectedValues: string[] = [];

// -----------------------------------------------------------------------------
// Test 1: Basic sequential prompts (arg -> arg)
// Tests that prompts can be chained and values are returned correctly
// -----------------------------------------------------------------------------
const test1 = 'sequential-prompts';
logTest(test1, 'running');
const start1 = Date.now();

try {
  debug('Test 1: Sequential arg() prompts');
  
  // First prompt
  const category = await arg('Step 1: Select a category', [
    { name: 'Fruits', value: 'fruits', description: 'Fresh produce' },
    { name: 'Vegetables', value: 'vegetables', description: 'Healthy greens' },
    { name: 'Dairy', value: 'dairy', description: 'Milk products' },
  ]);
  
  collectedValues.push(category);
  debug(`First selection: "${category}"`);
  
  // Second prompt - choices depend on first selection
  let items: { name: string; value: string; description?: string }[];
  switch (category) {
    case 'fruits':
      items = [
        { name: 'Apple', value: 'apple', description: 'Red and crispy' },
        { name: 'Banana', value: 'banana', description: 'Yellow and sweet' },
        { name: 'Orange', value: 'orange', description: 'Citrus delight' },
      ];
      break;
    case 'vegetables':
      items = [
        { name: 'Carrot', value: 'carrot', description: 'Orange root vegetable' },
        { name: 'Broccoli', value: 'broccoli', description: 'Green tree vegetable' },
        { name: 'Spinach', value: 'spinach', description: 'Leafy green' },
      ];
      break;
    case 'dairy':
      items = [
        { name: 'Milk', value: 'milk', description: 'Fresh dairy' },
        { name: 'Cheese', value: 'cheese', description: 'Aged goodness' },
        { name: 'Yogurt', value: 'yogurt', description: 'Cultured cream' },
      ];
      break;
    default:
      items = [{ name: 'Unknown', value: 'unknown' }];
  }
  
  const item = await arg(`Step 2: Select a ${category} item`, items);
  collectedValues.push(item);
  debug(`Second selection: "${item}"`);
  
  // Verify both values were captured
  if (typeof category === 'string' && typeof item === 'string') {
    logTest(test1, 'pass', { 
      result: { category, item },
      duration_ms: Date.now() - start1 
    });
  } else {
    logTest(test1, 'fail', { 
      error: `Expected strings, got category=${typeof category}, item=${typeof item}`,
      duration_ms: Date.now() - start1 
    });
  }
} catch (err) {
  logTest(test1, 'fail', { 
    error: String(err), 
    duration_ms: Date.now() - start1 
  });
}

// -----------------------------------------------------------------------------
// Test 2: Empty choices - text input mode
// Tests that arg() with empty choices works as text input
// -----------------------------------------------------------------------------
const test2 = 'empty-choices-text-input';
logTest(test2, 'running');
const start2 = Date.now();

try {
  debug('Test 2: arg() with empty choices (text input mode)');
  
  // This should show a text input with no choices
  const userInput = await arg('Enter some text (type anything):', []);
  
  collectedValues.push(userInput);
  debug(`User entered: "${userInput}"`);
  
  // Any string is valid (even empty if user just pressed Enter)
  if (typeof userInput === 'string') {
    logTest(test2, 'pass', { 
      result: userInput,
      duration_ms: Date.now() - start2 
    });
  } else {
    logTest(test2, 'fail', { 
      error: `Expected string, got ${typeof userInput}`,
      duration_ms: Date.now() - start2 
    });
  }
} catch (err) {
  logTest(test2, 'fail', { 
    error: String(err), 
    duration_ms: Date.now() - start2 
  });
}

// -----------------------------------------------------------------------------
// Test 3: div() after multiple prompts - state preservation
// Tests that div() correctly displays accumulated state from previous prompts
// -----------------------------------------------------------------------------
const test3 = 'state-preservation-div';
logTest(test3, 'running');
const start3 = Date.now();

try {
  debug('Test 3: div() showing accumulated state');
  
  const stateHtml = md(`# Prompt Flow Complete

## Collected Values

| Step | Value |
|------|-------|
| Category | ${collectedValues[0] || 'N/A'} |
| Item | ${collectedValues[1] || 'N/A'} |
| Text Input | ${collectedValues[2] || '(empty)'} |

## Test Summary

Total prompts completed: **${collectedValues.length}**

All values have been preserved across the prompt chain.

---

*Press Escape or click to continue to exit test.*`);
  
  await div(stateHtml);
  
  debug('div() completed - user dismissed');
  
  logTest(test3, 'pass', { 
    result: { collectedValues },
    duration_ms: Date.now() - start3 
  });
} catch (err) {
  logTest(test3, 'fail', { 
    error: String(err), 
    duration_ms: Date.now() - start3 
  });
}

// -----------------------------------------------------------------------------
// Test 4: Early exit simulation (checking Escape key behavior)
// This test verifies that empty returns from arg() are handled gracefully
// -----------------------------------------------------------------------------
const test4 = 'early-exit-handling';
logTest(test4, 'running');
const start4 = Date.now();

try {
  debug('Test 4: Testing early exit handling');
  debug('Note: In interactive mode, pressing Escape returns empty string');
  
  const result = await arg('Try pressing Escape to test early exit:', [
    { name: 'Continue with test', value: 'continue' },
    { name: 'Press Escape to exit early', value: 'should-not-see' },
  ]);
  
  // Empty string indicates early exit (Escape pressed)
  // Non-empty string means user selected an option
  if (result === '') {
    debug('User pressed Escape - early exit detected');
    logTest(test4, 'pass', { 
      result: 'early-exit-detected',
      duration_ms: Date.now() - start4 
    });
  } else {
    debug(`User selected: "${result}"`);
    logTest(test4, 'pass', { 
      result: `selection: ${result}`,
      duration_ms: Date.now() - start4 
    });
  }
} catch (err) {
  // An error here might indicate issues with early exit handling
  logTest(test4, 'fail', { 
    error: String(err), 
    duration_ms: Date.now() - start4,
    expected: 'Should handle Escape key gracefully'
  });
}

// -----------------------------------------------------------------------------
// Final Summary
// -----------------------------------------------------------------------------
debug('test-prompt-flow.ts completing...');

await div(md(`# All Prompt Flow Tests Complete

## Test Results

| Test | Description |
|------|-------------|
| sequential-prompts | Chained arg() calls with dependent choices |
| empty-choices-text-input | arg() with empty array = text input |
| state-preservation-div | div() showing accumulated state |
| early-exit-handling | Escape key / empty return handling |

---

### Collected Values Summary
\`\`\`json
${JSON.stringify(collectedValues, null, 2)}
\`\`\`

*All tests executed. Press Escape to exit.*`));

debug('test-prompt-flow.ts completed successfully!');
