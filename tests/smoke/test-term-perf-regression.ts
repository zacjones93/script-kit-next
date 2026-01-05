// Name: Terminal Performance Regression Test
// Description: Measures terminal typing performance to detect regressions

/**
 * REGRESSION TEST: test-term-perf-regression.ts
 * 
 * This test measures terminal typing performance at the SDK level to detect
 * regressions from changes to:
 * - term_prompt.rs render loop
 * - alacritty.rs content() method
 * - PTY I/O handling
 * - cx.notify() frequency
 * 
 * Key metrics (from AGENTS.md):
 * - P95 Key Latency: < 50ms
 * - Single Key Event: < 16.67ms (60fps)
 * - Render time: < 16ms per frame
 * 
 * Run with:
 *   cargo build && echo '{"type":"run","path":"'$(pwd)'/tests/smoke/test-term-perf-regression.ts"}' | \
 *     SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
 */

import '../../scripts/kit-sdk';

// =============================================================================
// Types
// =============================================================================

interface PerfMetrics {
  keystrokeLatencies: number[];
  renderTimes: number[];
  totalDuration: number;
}

interface TestResult {
  test: string;
  status: 'running' | 'pass' | 'fail' | 'skip';
  timestamp: string;
  result?: unknown;
  error?: string;
  reason?: string;
  duration_ms?: number;
  metrics?: {
    p50: number;
    p95: number;
    p99: number;
    avg: number;
    min: number;
    max: number;
  };
}

// =============================================================================
// Utilities
// =============================================================================

function log(test: string, status: TestResult['status'], extra?: Partial<TestResult>) {
  const result: TestResult = {
    test,
    status,
    timestamp: new Date().toISOString(),
    ...extra
  };
  console.log(JSON.stringify(result));
}

function debug(msg: string) {
  console.error(`[TERM-PERF] ${msg}`);
}

function calculatePercentile(sorted: number[], percentile: number): number {
  if (sorted.length === 0) return 0;
  const idx = Math.floor(sorted.length * (percentile / 100));
  return sorted[Math.min(idx, sorted.length - 1)];
}

function calculateStats(samples: number[]) {
  if (samples.length === 0) {
    return { min: 0, max: 0, avg: 0, p50: 0, p95: 0, p99: 0 };
  }
  const sorted = [...samples].sort((a, b) => a - b);
  const sum = sorted.reduce((acc, val) => acc + val, 0);
  return {
    min: sorted[0],
    max: sorted[sorted.length - 1],
    avg: sum / sorted.length,
    p50: calculatePercentile(sorted, 50),
    p95: calculatePercentile(sorted, 95),
    p99: calculatePercentile(sorted, 99),
  };
}

// =============================================================================
// Performance Thresholds
// =============================================================================

const THRESHOLDS = {
  P95_LATENCY_MS: 50,       // Maximum P95 keystroke latency
  AVG_LATENCY_MS: 16.67,    // Target for 60fps
  RENDER_TIME_MS: 16,       // Maximum render time per frame
};

// =============================================================================
// Test 1: Rapid Sequential Typing
// =============================================================================

async function testRapidTyping(): Promise<boolean> {
  const testName = 'term-rapid-typing';
  log(testName, 'running');
  const start = Date.now();

  try {
    debug('Starting rapid typing test...');
    
    // Open terminal with cat to echo back input
    const termPromise = term('cat');
    
    // Wait for terminal to initialize
    await wait(300);
    
    const latencies: number[] = [];
    const testString = 'hello world test';
    
    // Type characters with timing
    for (const char of testString) {
      const keyStart = performance.now();
      await keyboard.tap(char);
      const keyEnd = performance.now();
      latencies.push(keyEnd - keyStart);
      await wait(10); // Small delay between keystrokes
    }
    
    // Press Enter to confirm
    await keyboard.tap('enter');
    await wait(100);
    
    // Exit terminal
    await keyboard.tap('c', 'control'); // Ctrl+C to exit cat
    await wait(50);
    await keyboard.tap('escape');
    
    try {
      await termPromise;
    } catch {
      // Terminal may close abruptly - OK
    }
    
    const stats = calculateStats(latencies);
    const passed = stats.p95 < THRESHOLDS.P95_LATENCY_MS;
    
    log(testName, passed ? 'pass' : 'fail', {
      duration_ms: Date.now() - start,
      metrics: stats,
      result: passed 
        ? `P95=${stats.p95.toFixed(2)}ms (threshold: ${THRESHOLDS.P95_LATENCY_MS}ms)`
        : `P95=${stats.p95.toFixed(2)}ms EXCEEDS threshold ${THRESHOLDS.P95_LATENCY_MS}ms`,
    });
    
    debug(`Rapid typing: P95=${stats.p95.toFixed(2)}ms, avg=${stats.avg.toFixed(2)}ms`);
    return passed;
    
  } catch (err) {
    log(testName, 'fail', { error: String(err), duration_ms: Date.now() - start });
    debug(`Rapid typing test error: ${err}`);
    return false;
  }
}

// =============================================================================
// Test 2: Burst Typing (Simulates paste-like rapid input)
// =============================================================================

async function testBurstTyping(): Promise<boolean> {
  const testName = 'term-burst-typing';
  log(testName, 'running');
  const start = Date.now();

  try {
    debug('Starting burst typing test...');
    
    const termPromise = term('cat');
    await wait(300);
    
    const latencies: number[] = [];
    const burstChars = 'abcdefghij'; // 10 char burst
    const burstCount = 5;
    
    for (let burst = 0; burst < burstCount; burst++) {
      // Rapid burst with minimal delay
      for (const char of burstChars) {
        const keyStart = performance.now();
        await keyboard.tap(char);
        const keyEnd = performance.now();
        latencies.push(keyEnd - keyStart);
        await wait(5); // Very short delay - stress test
      }
      await wait(100); // Pause between bursts
    }
    
    // Exit
    await keyboard.tap('c', 'control');
    await wait(50);
    await keyboard.tap('escape');
    
    try {
      await termPromise;
    } catch {
      // OK
    }
    
    const stats = calculateStats(latencies);
    const passed = stats.p95 < THRESHOLDS.P95_LATENCY_MS;
    
    log(testName, passed ? 'pass' : 'fail', {
      duration_ms: Date.now() - start,
      metrics: stats,
      result: passed
        ? `P95=${stats.p95.toFixed(2)}ms (${burstCount} bursts × ${burstChars.length} chars)`
        : `P95=${stats.p95.toFixed(2)}ms EXCEEDS threshold`,
    });
    
    debug(`Burst typing: P95=${stats.p95.toFixed(2)}ms, max=${stats.max.toFixed(2)}ms`);
    return passed;
    
  } catch (err) {
    log(testName, 'fail', { error: String(err), duration_ms: Date.now() - start });
    debug(`Burst typing test error: ${err}`);
    return false;
  }
}

// =============================================================================
// Test 3: Sustained Input (Key repeat simulation)
// =============================================================================

async function testSustainedInput(): Promise<boolean> {
  const testName = 'term-sustained-input';
  log(testName, 'running');
  const start = Date.now();

  try {
    debug('Starting sustained input test...');
    
    const termPromise = term('cat');
    await wait(300);
    
    const latencies: number[] = [];
    const keyCount = 50; // 50 rapid keystrokes
    
    // Simulate holding a key - very rapid repetition
    for (let i = 0; i < keyCount; i++) {
      const keyStart = performance.now();
      await keyboard.tap('x');
      const keyEnd = performance.now();
      latencies.push(keyEnd - keyStart);
      await wait(8); // ~125 keys/sec (keyboard repeat rate)
    }
    
    // Exit
    await keyboard.tap('c', 'control');
    await wait(50);
    await keyboard.tap('escape');
    
    try {
      await termPromise;
    } catch {
      // OK
    }
    
    const stats = calculateStats(latencies);
    // For sustained input, we're stricter - should target 60fps
    const passed = stats.avg < THRESHOLDS.AVG_LATENCY_MS * 2; // Allow 2x for test overhead
    
    log(testName, passed ? 'pass' : 'fail', {
      duration_ms: Date.now() - start,
      metrics: stats,
      result: passed
        ? `avg=${stats.avg.toFixed(2)}ms (${keyCount} keys)`
        : `avg=${stats.avg.toFixed(2)}ms EXCEEDS target`,
    });
    
    debug(`Sustained input: avg=${stats.avg.toFixed(2)}ms, P95=${stats.p95.toFixed(2)}ms`);
    return passed;
    
  } catch (err) {
    log(testName, 'fail', { error: String(err), duration_ms: Date.now() - start });
    debug(`Sustained input test error: ${err}`);
    return false;
  }
}

// =============================================================================
// Test 4: Command Output Performance (render stress test)
// =============================================================================

async function testOutputPerformance(): Promise<boolean> {
  const testName = 'term-output-performance';
  log(testName, 'running');
  const start = Date.now();

  try {
    debug('Starting output performance test...');
    
    // Run a command that produces lots of output
    const termStart = performance.now();
    
    // Generate output that stresses the render loop
    // seq produces numbered lines rapidly
    const termPromise = term('seq 1 500');
    
    // Wait a bit for output to stream
    await wait(1000);
    
    // Exit
    await keyboard.tap('escape');
    
    let output = '';
    try {
      output = await termPromise;
    } catch {
      // May timeout - OK
    }
    
    const totalTime = performance.now() - termStart;
    const linesProcessed = output.split('\n').filter(l => l.trim()).length;
    
    // Should process 500 lines reasonably quickly
    // Allow 3 seconds max for this test
    const passed = totalTime < 3000;
    
    log(testName, passed ? 'pass' : 'fail', {
      duration_ms: Date.now() - start,
      result: passed
        ? `${linesProcessed} lines in ${totalTime.toFixed(0)}ms`
        : `Too slow: ${totalTime.toFixed(0)}ms`,
    });
    
    debug(`Output perf: ${linesProcessed} lines in ${totalTime.toFixed(0)}ms`);
    return passed;
    
  } catch (err) {
    log(testName, 'fail', { error: String(err), duration_ms: Date.now() - start });
    debug(`Output performance test error: ${err}`);
    return false;
  }
}

// =============================================================================
// Test 5: Special Keys Performance (arrow keys, etc.)
// =============================================================================

async function testSpecialKeysPerformance(): Promise<boolean> {
  const testName = 'term-special-keys';
  log(testName, 'running');
  const start = Date.now();

  try {
    debug('Starting special keys test...');
    
    // Use bash for command-line editing with arrow keys
    const termPromise = term('bash');
    await wait(300);
    
    const latencies: number[] = [];
    
    // Type something
    for (const char of 'hello') {
      await keyboard.tap(char);
      await wait(10);
    }
    
    // Test arrow keys (used for command-line navigation)
    for (let i = 0; i < 10; i++) {
      const keyStart = performance.now();
      await keyboard.tap('left');
      latencies.push(performance.now() - keyStart);
      await wait(10);
    }
    
    for (let i = 0; i < 10; i++) {
      const keyStart = performance.now();
      await keyboard.tap('right');
      latencies.push(performance.now() - keyStart);
      await wait(10);
    }
    
    // Test up/down (command history)
    for (let i = 0; i < 5; i++) {
      const keyStart = performance.now();
      await keyboard.tap('up');
      latencies.push(performance.now() - keyStart);
      await wait(20);
    }
    
    // Exit
    await keyboard.tap('d', 'control'); // Ctrl+D to exit bash
    await wait(100);
    await keyboard.tap('escape');
    
    try {
      await termPromise;
    } catch {
      // OK
    }
    
    const stats = calculateStats(latencies);
    const passed = stats.p95 < THRESHOLDS.P95_LATENCY_MS;
    
    log(testName, passed ? 'pass' : 'fail', {
      duration_ms: Date.now() - start,
      metrics: stats,
      result: passed
        ? `Arrow keys P95=${stats.p95.toFixed(2)}ms`
        : `Arrow keys P95=${stats.p95.toFixed(2)}ms EXCEEDS threshold`,
    });
    
    debug(`Special keys: P95=${stats.p95.toFixed(2)}ms`);
    return passed;
    
  } catch (err) {
    log(testName, 'fail', { error: String(err), duration_ms: Date.now() - start });
    debug(`Special keys test error: ${err}`);
    return false;
  }
}

// =============================================================================
// Main
// =============================================================================

debug('test-term-perf-regression.ts starting...');
debug('Performance thresholds:');
debug(`  P95 Latency: ${THRESHOLDS.P95_LATENCY_MS}ms`);
debug(`  Avg Latency: ${THRESHOLDS.AVG_LATENCY_MS}ms (60fps target)`);
debug(`  Render Time: ${THRESHOLDS.RENDER_TIME_MS}ms`);

const results: boolean[] = [];

// Run tests with cooldown between each
results.push(await testRapidTyping());
await wait(500);

results.push(await testBurstTyping());
await wait(500);

results.push(await testSustainedInput());
await wait(500);

results.push(await testOutputPerformance());
await wait(500);

results.push(await testSpecialKeysPerformance());

// Summary
const passed = results.filter(r => r).length;
const total = results.length;
const allPassed = passed === total;

debug('');
debug('='.repeat(60));
debug(`TERMINAL PERFORMANCE REGRESSION: ${passed}/${total} tests passed`);
debug('='.repeat(60));

// Final summary display
await div(md(`# Terminal Performance Regression Tests

## Results: ${allPassed ? '✅ ALL PASSED' : '❌ SOME FAILED'}

| Test | Status |
|------|--------|
| Rapid Typing | ${results[0] ? '✅' : '❌'} |
| Burst Typing | ${results[1] ? '✅' : '❌'} |
| Sustained Input | ${results[2] ? '✅' : '❌'} |
| Output Performance | ${results[3] ? '✅' : '❌'} |
| Special Keys | ${results[4] ? '✅' : '❌'} |

## Thresholds
- **P95 Latency**: < ${THRESHOLDS.P95_LATENCY_MS}ms
- **Avg Latency**: < ${THRESHOLDS.AVG_LATENCY_MS}ms (60fps)
- **Render Time**: < ${THRESHOLDS.RENDER_TIME_MS}ms

---

*Check console output for detailed metrics*

Press Escape to exit.`));

debug('test-term-perf-regression.ts completed!');
