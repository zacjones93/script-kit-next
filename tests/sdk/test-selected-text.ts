// Name: SDK Test - Selected Text
// Description: Tests selected text APIs (getSelectedText, setSelectedText, hasAccessibilityPermission)

/**
 * SDK TEST: test-selected-text.ts
 * 
 * Tests the selected text APIs that interact with macOS Accessibility APIs.
 * 
 * Test cases:
 * 1. hasAccessibilityPermission - Returns boolean (always passes)
 * 2. getSelectedText - Get selected text (may skip without permission)
 * 3. setSelectedText - Set selected text (may skip without permission)
 * 4. requestAccessibilityPermission - Opens System Preferences (API exists check)
 * 
 * Note: These APIs require macOS Accessibility permission. Tests gracefully
 * skip if permission is not granted rather than failing.
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
  reason?: string;
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

debug('test-selected-text.ts starting...');
debug(`SDK globals: hasAccessibilityPermission=${typeof hasAccessibilityPermission}`);
debug(`SDK globals: getSelectedText=${typeof getSelectedText}`);
debug(`SDK globals: setSelectedText=${typeof setSelectedText}`);
debug(`SDK globals: requestAccessibilityPermission=${typeof requestAccessibilityPermission}`);

// -----------------------------------------------------------------------------
// Test 1: hasAccessibilityPermission - Returns boolean
// -----------------------------------------------------------------------------
const test1 = 'hasAccessibilityPermission-returns-boolean';
logTest(test1, 'running');
const start1 = Date.now();

let hasPermission = false;

try {
  debug('Test 1: hasAccessibilityPermission() returns boolean');
  
  // Verify function exists
  if (typeof hasAccessibilityPermission !== 'function') {
    throw new Error('hasAccessibilityPermission is not a function');
  }
  
  const result = await hasAccessibilityPermission();
  hasPermission = result;
  
  debug(`Test 1 result: ${result} (type: ${typeof result})`);
  
  // Verify it returns a boolean
  if (typeof result !== 'boolean') {
    throw new Error(`Expected boolean, got ${typeof result}`);
  }
  
  logTest(test1, 'pass', { 
    result: { hasPermission: result },
    duration_ms: Date.now() - start1 
  });
} catch (err) {
  logTest(test1, 'fail', { error: String(err), duration_ms: Date.now() - start1 });
}

// -----------------------------------------------------------------------------
// Test 2: getSelectedText - Get selected text
// -----------------------------------------------------------------------------
const test2 = 'getSelectedText-basic';
logTest(test2, 'running');
const start2 = Date.now();

try {
  debug('Test 2: getSelectedText()');
  
  // Verify function exists
  if (typeof getSelectedText !== 'function') {
    throw new Error('getSelectedText is not a function');
  }
  
  if (!hasPermission) {
    // Skip if no permission - this is expected behavior
    debug('Test 2 skipped: Accessibility permission not granted');
    logTest(test2, 'skip', { 
      reason: 'Accessibility permission not granted',
      duration_ms: Date.now() - start2 
    });
  } else {
    // Try to get selected text
    const result = await getSelectedText();
    
    debug(`Test 2 result: "${result}" (type: ${typeof result})`);
    
    // Verify it returns a string (empty string if nothing selected)
    if (typeof result !== 'string') {
      throw new Error(`Expected string, got ${typeof result}`);
    }
    
    logTest(test2, 'pass', { 
      result: { selectedText: result, length: result.length },
      duration_ms: Date.now() - start2 
    });
  }
} catch (err) {
  const errorStr = String(err);
  // If error mentions permission, skip instead of fail
  if (errorStr.includes('permission') || errorStr.includes('accessibility')) {
    logTest(test2, 'skip', { 
      reason: 'Accessibility permission required',
      error: errorStr,
      duration_ms: Date.now() - start2 
    });
  } else {
    logTest(test2, 'fail', { error: errorStr, duration_ms: Date.now() - start2 });
  }
}

// -----------------------------------------------------------------------------
// Test 3: setSelectedText - Set selected text
// -----------------------------------------------------------------------------
const test3 = 'setSelectedText-basic';
logTest(test3, 'running');
const start3 = Date.now();

try {
  debug('Test 3: setSelectedText()');
  
  // Verify function exists
  if (typeof setSelectedText !== 'function') {
    throw new Error('setSelectedText is not a function');
  }
  
  if (!hasPermission) {
    // Skip if no permission - this is expected behavior
    debug('Test 3 skipped: Accessibility permission not granted');
    logTest(test3, 'skip', { 
      reason: 'Accessibility permission not granted',
      duration_ms: Date.now() - start3 
    });
  } else {
    // Try to set selected text with a test string
    const testText = `Test from Script Kit: ${Date.now()}`;
    await setSelectedText(testText);
    
    debug(`Test 3 completed: setSelectedText("${testText}")`);
    
    logTest(test3, 'pass', { 
      result: { textSent: testText },
      duration_ms: Date.now() - start3 
    });
  }
} catch (err) {
  const errorStr = String(err);
  // If error mentions permission, skip instead of fail
  if (errorStr.includes('permission') || errorStr.includes('accessibility')) {
    logTest(test3, 'skip', { 
      reason: 'Accessibility permission required',
      error: errorStr,
      duration_ms: Date.now() - start3 
    });
  } else {
    logTest(test3, 'fail', { error: errorStr, duration_ms: Date.now() - start3 });
  }
}

// -----------------------------------------------------------------------------
// Test 4: requestAccessibilityPermission - API exists
// -----------------------------------------------------------------------------
const test4 = 'requestAccessibilityPermission-exists';
logTest(test4, 'running');
const start4 = Date.now();

try {
  debug('Test 4: requestAccessibilityPermission() API check');
  
  // Verify function exists
  if (typeof requestAccessibilityPermission !== 'function') {
    throw new Error('requestAccessibilityPermission is not a function');
  }
  
  debug('Test 4 completed: requestAccessibilityPermission function exists');
  
  // Note: We don't actually call this function as it opens System Preferences
  // which would be disruptive during automated testing
  
  logTest(test4, 'pass', { 
    result: { functionExists: true },
    duration_ms: Date.now() - start4 
  });
} catch (err) {
  logTest(test4, 'fail', { error: String(err), duration_ms: Date.now() - start4 });
}

// -----------------------------------------------------------------------------
// Test 5: getSelectedText returns empty string when nothing selected
// -----------------------------------------------------------------------------
const test5 = 'getSelectedText-empty-string';
logTest(test5, 'running');
const start5 = Date.now();

try {
  debug('Test 5: getSelectedText() returns empty string when nothing selected');
  
  if (!hasPermission) {
    debug('Test 5 skipped: Accessibility permission not granted');
    logTest(test5, 'skip', { 
      reason: 'Accessibility permission not granted',
      duration_ms: Date.now() - start5 
    });
  } else {
    // Note: This test verifies the return type is correct even when nothing is selected
    // The actual value depends on system state, so we just verify it's a string
    const result = await getSelectedText();
    
    debug(`Test 5 result: "${result}"`);
    
    // The result should be a string (empty or with content)
    if (typeof result !== 'string') {
      throw new Error(`Expected string, got ${typeof result}`);
    }
    
    logTest(test5, 'pass', { 
      result: { 
        returnedString: true, 
        isEmpty: result.length === 0,
        length: result.length 
      },
      duration_ms: Date.now() - start5 
    });
  }
} catch (err) {
  const errorStr = String(err);
  if (errorStr.includes('permission') || errorStr.includes('accessibility')) {
    logTest(test5, 'skip', { 
      reason: 'Accessibility permission required',
      error: errorStr,
      duration_ms: Date.now() - start5 
    });
  } else {
    logTest(test5, 'fail', { error: errorStr, duration_ms: Date.now() - start5 });
  }
}

// -----------------------------------------------------------------------------
// Test 6: setSelectedText with empty string
// -----------------------------------------------------------------------------
const test6 = 'setSelectedText-empty-string';
logTest(test6, 'running');
const start6 = Date.now();

try {
  debug('Test 6: setSelectedText("") with empty string');
  
  if (!hasPermission) {
    debug('Test 6 skipped: Accessibility permission not granted');
    logTest(test6, 'skip', { 
      reason: 'Accessibility permission not granted',
      duration_ms: Date.now() - start6 
    });
  } else {
    // Setting empty string should work (effectively clearing selection)
    await setSelectedText('');
    
    debug('Test 6 completed: setSelectedText("") succeeded');
    
    logTest(test6, 'pass', { 
      result: { emptyStringAccepted: true },
      duration_ms: Date.now() - start6 
    });
  }
} catch (err) {
  const errorStr = String(err);
  if (errorStr.includes('permission') || errorStr.includes('accessibility')) {
    logTest(test6, 'skip', { 
      reason: 'Accessibility permission required',
      error: errorStr,
      duration_ms: Date.now() - start6 
    });
  } else {
    logTest(test6, 'fail', { error: errorStr, duration_ms: Date.now() - start6 });
  }
}

// -----------------------------------------------------------------------------
// Show Summary
// -----------------------------------------------------------------------------
debug('test-selected-text.ts completed!');

await div(md(`# Selected Text API Tests Complete

All selected text API tests have been executed.

## Test Cases

### Permission Check
1. **hasAccessibilityPermission-returns-boolean**: Verifies function returns boolean

### Text Operations
2. **getSelectedText-basic**: Get currently selected text
3. **setSelectedText-basic**: Set/replace selected text
4. **requestAccessibilityPermission-exists**: API function exists

### Edge Cases
5. **getSelectedText-empty-string**: Returns empty string when nothing selected
6. **setSelectedText-empty-string**: Accepts empty string input

## Permission Status
- **Has Accessibility Permission**: ${hasPermission ? 'Yes' : 'No'}

${!hasPermission ? `
> **Note**: Some tests were skipped because accessibility permission is not granted.
> To enable full testing, grant accessibility permission in:
> System Preferences > Privacy & Security > Accessibility
` : ''}

---

*Check the JSONL output for detailed results*

Press Escape or click to exit.`));

debug('test-selected-text.ts exiting...');
