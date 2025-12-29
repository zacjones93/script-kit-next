/**
 * Test Script: Intentional Crash Scenarios
 * 
 * This script tests the crash handling behavior of the GPUI app.
 * It provides different crash modes to verify the app properly:
 * 1. Detects when a script exits unexpectedly
 * 2. Cleans up prompt state and returns to script list
 * 3. Shows user-friendly error toast with crash info
 * 4. Logs crash details for debugging
 * 
 * Usage:
 *   CRASH_MODE=<mode> bun run tests/smoke/test-script-crash.ts
 * 
 * Modes:
 *   exit1     - Exit with code 1 (general error)
 *   throw     - Throw an uncaught exception
 *   timeout   - Exit without responding to prompt (simulates hang/crash mid-prompt)
 *   sigabrt   - Abort with SIGABRT (call process.abort())
 *   sigkill   - Request SIGKILL on self (may not work in all environments)
 *   sigsegv   - Attempt to trigger SIGSEGV (platform-dependent)
 *   mid-arg   - Crash during arg() prompt
 *   mid-div   - Crash during div() display
 *   exit42    - Exit with code 42 (custom error code)
 */

import '../../scripts/kit-sdk';

const CRASH_MODE = process.env.CRASH_MODE || 'throw';

console.error(`[CRASH-TEST] Starting with CRASH_MODE=${CRASH_MODE}`);

// Helper to delay execution
const sleep = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

// Signal that we're about to crash (for log verification)
const signalCrash = (mode: string) => {
  console.error(`[CRASH-TEST] Initiating crash: ${mode}`);
};

async function runCrashTest() {
  switch (CRASH_MODE) {
    case 'exit1':
      // Simple exit with error code
      signalCrash('exit with code 1');
      process.exit(1);
      break;

    case 'exit42':
      // Exit with custom code
      signalCrash('exit with code 42');
      process.exit(42);
      break;

    case 'throw':
      // Throw uncaught exception
      signalCrash('uncaught exception');
      throw new Error('Intentional crash for testing - uncaught exception');

    case 'timeout':
      // Start a prompt but never respond (simulates script hanging)
      signalCrash('timeout - starting prompt then exiting without response');
      // Don't await - just exit abruptly
      arg('This prompt will never get a response...');
      await sleep(100);
      process.exit(1);
      break;

    case 'mid-arg':
      // Start arg prompt, wait briefly, then crash
      signalCrash('crash mid-arg prompt');
      const argPromise = arg('Select an option (will crash)', [
        { name: 'Option 1', value: '1' },
        { name: 'Option 2', value: '2' },
        { name: 'Option 3', value: '3' },
      ]);
      // Give time for UI to render
      await sleep(500);
      throw new Error('Intentional crash during arg() prompt');

    case 'mid-div':
      // Start div prompt, wait briefly, then crash
      signalCrash('crash mid-div display');
      const divPromise = div(`
        <div class="p-4 bg-red-500 text-white">
          <h1>This div will crash</h1>
          <p>The script will throw an error after this displays...</p>
        </div>
      `);
      await sleep(500);
      throw new Error('Intentional crash during div() display');

    case 'sigabrt':
      // Abort the process
      signalCrash('SIGABRT via process.abort()');
      process.abort();
      break;

    case 'sigkill':
      // Kill self with SIGKILL
      signalCrash('SIGKILL via process.kill()');
      process.kill(process.pid, 'SIGKILL');
      break;

    case 'sigsegv':
      // This is tricky - we can't easily cause a segfault in JS
      // Instead, we'll just exit with 139 (128 + 11) to simulate
      signalCrash('simulated SIGSEGV (exit 139)');
      process.exit(139);
      break;

    case 'reference-error':
      // Reference a non-existent variable
      signalCrash('ReferenceError');
      // @ts-expect-error - intentional error
      console.log(nonExistentVariable);
      break;

    case 'type-error':
      // Call undefined as function
      signalCrash('TypeError');
      // @ts-expect-error - intentional error
      const x: any = null;
      x.someMethod();
      break;

    case 'syntax-error':
      // Can't really do syntax error at runtime, but we can eval bad code
      signalCrash('syntax error via eval');
      eval('function { broken syntax');
      break;

    case 'oom':
      // Attempt to exhaust memory (may take time or be caught by runtime)
      signalCrash('out of memory attempt');
      const arrays: number[][] = [];
      while (true) {
        arrays.push(new Array(1000000).fill(0));
      }
      break;

    case 'infinite-loop':
      // Infinite synchronous loop (will hang, needs external kill)
      signalCrash('infinite loop - script will hang');
      console.error('[CRASH-TEST] This will require external termination');
      while (true) {
        // Intentionally empty - infinite loop
      }
      break;

    case 'stderr-then-exit':
      // Write error to stderr then exit with error code
      signalCrash('stderr message then exit');
      console.error('Error: Something went wrong in the script');
      console.error('at testFunction (test.ts:42:10)');
      console.error('at main (test.ts:100:5)');
      process.exit(1);
      break;

    default:
      console.error(`[CRASH-TEST] Unknown CRASH_MODE: ${CRASH_MODE}`);
      console.error('[CRASH-TEST] Valid modes: exit1, exit42, throw, timeout, mid-arg, mid-div, sigabrt, sigkill, sigsegv, reference-error, type-error, syntax-error, oom, infinite-loop, stderr-then-exit');
      process.exit(1);
  }
}

// Run the crash test
runCrashTest().catch(err => {
  console.error(`[CRASH-TEST] Caught error in main: ${err.message}`);
  console.error(err.stack);
  process.exit(1);
});
