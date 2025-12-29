// Name: SDK Test - env()
// Description: Tests env() prompt for environment variable input with keychain storage

// @ts-ignore - process is available at runtime via bun
declare const process: { env: Record<string, string | undefined>; exit: (code: number) => void };

/**
 * SDK TEST: test-env.ts
 * 
 * Tests the env() function for secure environment variable prompts:
 * 1. First call: Prompts for value (password-masked for secrets), stores in keychain
 * 2. Subsequent calls: Retrieves from keychain silently without showing UI
 * 
 * Expected behavior:
 * - env(key) sends JSONL message with type: 'env'
 * - Secret detection: keys containing 'secret', 'password', 'token', or 'key' are masked
 * - Values stored securely in system keychain (macOS Keychain)
 * - Subsequent calls return cached value without prompting
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
  console.error(`[TEST] ${msg}`);
}

// =============================================================================
// Tests
// =============================================================================

debug('test-env.ts starting...');
debug(`SDK globals: env=${typeof env}, arg=${typeof arg}`);

// -----------------------------------------------------------------------------
// Test 1: env() with a regular key (non-secret)
// Should show an unmasked input prompt
// -----------------------------------------------------------------------------
const test1 = 'env-regular-key';
logTest(test1, 'running');
const start1 = Date.now();

try {
  debug('Test 1: env() with regular key - should show input prompt');
  
  // This will prompt for a value. In testing, we'll check that the message is sent correctly.
  // For automated testing, you'd need to simulate user input or use forceSubmit.
  const result = await env('MY_CONFIG_VALUE');
  
  debug(`Test 1 result: "${result}"`);
  
  if (typeof result === 'string') {
    logTest(test1, 'pass', { result, duration_ms: Date.now() - start1 });
  } else {
    logTest(test1, 'fail', { 
      error: `Expected string, got ${typeof result}`,
      duration_ms: Date.now() - start1 
    });
  }
} catch (err) {
  logTest(test1, 'fail', { 
    error: String(err), 
    duration_ms: Date.now() - start1,
    expected: 'Should prompt for environment variable value'
  });
}

// -----------------------------------------------------------------------------
// Test 2: env() with a secret key (contains 'secret' in name)
// Should show a password-masked input prompt
// -----------------------------------------------------------------------------
const test2 = 'env-secret-key';
logTest(test2, 'running');
const start2 = Date.now();

try {
  debug('Test 2: env() with secret key - should show masked input');
  
  // Keys containing 'secret', 'password', 'token', or 'key' are auto-detected as secrets
  const result = await env('MY_SECRET_VALUE');
  
  debug(`Test 2 result: "${result ? '***' : '(empty)'}" (masked for security)`);
  
  if (typeof result === 'string') {
    logTest(test2, 'pass', { result: '(masked)', duration_ms: Date.now() - start2 });
  } else {
    logTest(test2, 'fail', { 
      error: `Expected string, got ${typeof result}`,
      duration_ms: Date.now() - start2 
    });
  }
} catch (err) {
  logTest(test2, 'fail', { 
    error: String(err), 
    duration_ms: Date.now() - start2,
    expected: 'Should prompt for secret value with masked input'
  });
}

// -----------------------------------------------------------------------------
// Test 3: env() with custom prompt function
// Should use the provided function instead of showing UI
// -----------------------------------------------------------------------------
const test3 = 'env-with-prompt-fn';
logTest(test3, 'running');
const start3 = Date.now();

try {
  debug('Test 3: env() with custom prompt function');
  
  // Clear the process.env value first to ensure prompt is called
  delete process.env['CUSTOM_PROMPTED_VALUE'];
  
  const customPrompt = async () => {
    debug('Custom prompt function called');
    return 'custom-value-from-function';
  };
  
  const result = await env('CUSTOM_PROMPTED_VALUE', customPrompt);
  
  debug(`Test 3 result: "${result}"`);
  
  if (result === 'custom-value-from-function') {
    logTest(test3, 'pass', { result, duration_ms: Date.now() - start3 });
  } else {
    logTest(test3, 'fail', { 
      error: `Expected "custom-value-from-function", got "${result}"`,
      duration_ms: Date.now() - start3 
    });
  }
} catch (err) {
  logTest(test3, 'fail', { 
    error: String(err), 
    duration_ms: Date.now() - start3,
    expected: 'Should use custom prompt function'
  });
}

// -----------------------------------------------------------------------------
// Test 4: env() with existing process.env value
// Should return immediately without prompting
// -----------------------------------------------------------------------------
const test4 = 'env-existing-value';
logTest(test4, 'running');
const start4 = Date.now();

try {
  debug('Test 4: env() with pre-set process.env value');
  
  // Pre-set the environment variable
  process.env['PRESET_VALUE'] = 'already-set-value';
  
  const result = await env('PRESET_VALUE');
  
  debug(`Test 4 result: "${result}"`);
  
  if (result === 'already-set-value') {
    logTest(test4, 'pass', { result, duration_ms: Date.now() - start4 });
  } else {
    logTest(test4, 'fail', { 
      error: `Expected "already-set-value", got "${result}"`,
      duration_ms: Date.now() - start4 
    });
  }
} catch (err) {
  logTest(test4, 'fail', { 
    error: String(err), 
    duration_ms: Date.now() - start4,
    expected: 'Should return existing env value immediately'
  });
}

// -----------------------------------------------------------------------------
// Cleanup
// -----------------------------------------------------------------------------
debug('test-env.ts complete');
debug('Note: Full keychain integration testing requires manual verification');

// Exit cleanly
process.exit(0);
