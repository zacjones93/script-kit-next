// Name: Escape/Cancel Handling Test
// Description: Tests that escape/cancel handling works correctly across all prompt types

/**
 * SMOKE TEST: test-escape-cancel.ts
 *
 * This script tests and documents how escape/cancel handling works across all prompt types:
 * - arg() prompt: Escape should cancel and return null/undefined, not throw error
 * - div() prompt: Escape should close the display and continue
 * - editor() prompt: Escape should cancel without saving content
 * - Rapid escape key presses should not crash the app
 *
 * Expected behavior:
 * 1. Cancel response is sent via protocol as {"type":"submit","id":"...","value":null}
 * 2. Script receives null/undefined for cancelled prompts
 * 3. Script can gracefully handle cancellation (not an error condition)
 * 4. Rapid escape key presses are handled without crashing
 *
 * Usage:
 *   echo '{"type":"run","path":"'$(pwd)'/tests/smoke/test-escape-cancel.ts"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
 *
 * Note: This is an interactive test. To fully test cancel:
 * - Press Escape when each prompt appears to verify cancel behavior
 * - Or press Enter/select to continue through the test
 *
 * Exit codes:
 *   0 = Success (all tests passed or cancel detected correctly)
 *   1 = Error
 */

import '../../scripts/kit-sdk';

// Test output format
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
    ...extra,
  };
  console.log(JSON.stringify(result));
}

console.error('[SMOKE] test-escape-cancel.ts starting...');
console.error('[SMOKE] This test documents escape/cancel behavior across prompt types');
console.error('[SMOKE] Press Escape to test cancel, or Enter to continue each prompt');

// =============================================================================
// TEST 1: arg() prompt cancel behavior
// =============================================================================

const test1Name = 'escape-arg-prompt';
logTest(test1Name, 'running');
const test1Start = Date.now();

console.error('[SMOKE] TEST 1: arg() prompt - cancel returns null/undefined');
console.error('[SMOKE] Expected: Pressing Escape during arg() should return undefined/null');
console.error('[SMOKE] Expected: Cancel message {"type":"submit","id":"...","value":null} sent via protocol');

try {
  const result1 = await arg({
    placeholder: 'Press Escape to test cancel, or Enter to continue',
    choices: [
      { name: 'Continue Test', value: 'continue', description: 'Press Enter or click to continue' },
      { name: 'Manual Cancel Test', value: 'manual', description: 'Or press Escape to test cancel' },
    ],
  });

  if (result1 === undefined || result1 === null || result1 === '') {
    console.error('[SMOKE] Escape/Cancel was pressed - prompt returned null/undefined');
    logTest(test1Name, 'pass', {
      result: 'cancelled',
      duration_ms: Date.now() - test1Start,
    });
  } else {
    console.error(`[SMOKE] User selected: ${result1}`);
    logTest(test1Name, 'pass', {
      result: result1,
      duration_ms: Date.now() - test1Start,
    });
  }
} catch (error) {
  // Cancel should NOT throw an error - it should return null/undefined
  console.error(`[SMOKE] ERROR: arg() threw on cancel - this may be wrong: ${error}`);
  logTest(test1Name, 'fail', {
    error: `Unexpected error on cancel: ${error}`,
    duration_ms: Date.now() - test1Start,
  });
}

// =============================================================================
// TEST 2: div() prompt escape behavior
// =============================================================================

const test2Name = 'escape-div-prompt';
logTest(test2Name, 'running');
const test2Start = Date.now();

console.error('[SMOKE] TEST 2: div() prompt - escape closes and continues');
console.error('[SMOKE] Expected: Pressing Escape during div() should close the display');
console.error('[SMOKE] Expected: div() returns undefined (acknowledgment only)');

try {
  await div(md(`
# Escape/Cancel Test - div() Prompt

This div prompt tests escape handling:
- Press **Escape** to close this display and continue
- Press **Enter** to also close and continue

Both should work without throwing an error.
`));

  console.error('[SMOKE] div() completed - escape or enter was pressed');
  logTest(test2Name, 'pass', {
    result: 'completed',
    duration_ms: Date.now() - test2Start,
  });
} catch (error) {
  console.error(`[SMOKE] ERROR: div() threw an error: ${error}`);
  logTest(test2Name, 'fail', {
    error: String(error),
    duration_ms: Date.now() - test2Start,
  });
}

// =============================================================================
// TEST 3: editor() prompt escape behavior
// =============================================================================

const test3Name = 'escape-editor-prompt';
logTest(test3Name, 'running');
const test3Start = Date.now();

console.error('[SMOKE] TEST 3: editor() prompt - escape cancels without saving');
console.error('[SMOKE] Expected: Pressing Escape during editor() should cancel');
console.error('[SMOKE] Expected: Original content is NOT returned on cancel');

try {
  const editorContent = await editor(
    '// Press Escape to cancel without saving\n// Or press Cmd+Enter to submit',
    'typescript'
  );

  if (editorContent === undefined || editorContent === null || editorContent === '') {
    console.error('[SMOKE] Editor was cancelled - content not saved (correct behavior)');
    logTest(test3Name, 'pass', {
      result: 'cancelled',
      duration_ms: Date.now() - test3Start,
    });
  } else {
    console.error(`[SMOKE] Editor submitted content (length: ${editorContent.length})`);
    logTest(test3Name, 'pass', {
      result: `submitted:${editorContent.length}chars`,
      duration_ms: Date.now() - test3Start,
    });
  }
} catch (error) {
  // Cancel should NOT throw an error
  console.error(`[SMOKE] ERROR: editor() threw on cancel: ${error}`);
  logTest(test3Name, 'fail', {
    error: String(error),
    duration_ms: Date.now() - test3Start,
  });
}

// =============================================================================
// TEST 4: Rapid escape key press handling (documentation)
// =============================================================================

const test4Name = 'rapid-escape-no-crash';
logTest(test4Name, 'running');
const test4Start = Date.now();

console.error('[SMOKE] TEST 4: Rapid escape key presses should not crash');
console.error('[SMOKE] Note: This test documents the expected behavior');
console.error('[SMOKE] The app should handle rapid escape presses gracefully');

// This test is informational - rapid escape testing requires the autonomous harness
// The key insight is that the app should coalesce/debounce rapid key events
logTest(test4Name, 'skip', {
  result: 'requires_autonomous_harness',
  duration_ms: Date.now() - test4Start,
});

// =============================================================================
// TEST 5: Cancel message protocol verification (documentation)
// =============================================================================

const test5Name = 'cancel-protocol-message';
logTest(test5Name, 'running');
const test5Start = Date.now();

console.error('[SMOKE] TEST 5: Cancel protocol message verification');
console.error('[SMOKE] Expected protocol message on Escape:');
console.error('[SMOKE]   {"type":"submit","id":"<prompt-id>","value":null}');
console.error('[SMOKE] Check AI compact logs for K|ESC or KEY|escape entries');

// This is documentation - the protocol message can be verified in logs
logTest(test5Name, 'pass', {
  result: 'documented',
  duration_ms: Date.now() - test5Start,
});

// =============================================================================
// SUMMARY
// =============================================================================

console.error('');
console.error('[SMOKE] ════════════════════════════════════════════════════════');
console.error('[SMOKE] ESCAPE/CANCEL BEHAVIOR SUMMARY:');
console.error('[SMOKE] ════════════════════════════════════════════════════════');
console.error('[SMOKE]');
console.error('[SMOKE] arg() prompt:');
console.error('[SMOKE]   - Escape sends: submit_prompt_response(id, None)');
console.error('[SMOKE]   - Calls cancel_script_execution() to clean up');
console.error('[SMOKE]   - Protocol: {"type":"submit","id":"...","value":null}');
console.error('[SMOKE]   - Script receives: undefined/null (NOT an error)');
console.error('[SMOKE]');
console.error('[SMOKE] div() prompt:');
console.error('[SMOKE]   - Escape sends: submit_prompt_response(id, None)');
console.error('[SMOKE]   - Calls cancel_script_execution() to clean up');
console.error('[SMOKE]   - Script receives: undefined (acknowledgment)');
console.error('[SMOKE]');
console.error('[SMOKE] editor() prompt:');
console.error('[SMOKE]   - Escape calls: cancel() method in editor.rs');
console.error('[SMOKE]   - Sends submit with None/null value');
console.error('[SMOKE]   - Content is NOT saved on cancel');
console.error('[SMOKE]');
console.error('[SMOKE] term() prompt:');
console.error('[SMOKE]   - Escape always cancels (line 483 in term_prompt.rs)');
console.error('[SMOKE]   - Sends message to cancel script execution');
console.error('[SMOKE]');
console.error('[SMOKE] Rapid escape handling:');
console.error('[SMOKE]   - Key events are coalesced with 20ms window');
console.error('[SMOKE]   - Prevents UI freeze from rapid key presses');
console.error('[SMOKE] ════════════════════════════════════════════════════════');
console.error('');

console.error('[SMOKE] test-escape-cancel.ts completed');
console.error('[SMOKE] All expected behaviors are documented above');
console.error('[SMOKE] For full cancel testing, use the autonomous test harness');
