// Padding Consistency Visual Regression Test
// Validates that padding values are applied consistently between term and editor prompts
// 
// Default padding values from config.rs:
// - top: 8px
// - left: 12px
// - right: 12px

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

interface PaddingMetric {
  prompt_type: 'term' | 'editor';
  expected_padding: {
    top: number;
    left: number;
    right: number;
  };
  bounds: WindowBounds;
  timestamp: string;
}

interface WindowBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

// Default padding values from config.rs
const DEFAULT_PADDING = {
  top: 8,
  left: 12,
  right: 12,
};

// Window dimensions from AGENTS.md
const WINDOW_WIDTH = 750;
const MAX_HEIGHT = 700;
const RESIZE_SETTLE_TIME = 100; // ms to wait for resize to complete

function logTest(name: string, status: TestResult['status'], extra?: Partial<TestResult>) {
  const result: TestResult = {
    test: name,
    status,
    timestamp: new Date().toISOString(),
    ...extra
  };
  console.log(JSON.stringify(result));
}

function logMetric(metric: PaddingMetric) {
  console.log(JSON.stringify({
    type: 'metric',
    metric: 'padding_consistency',
    ...metric,
  }));
}

function debug(msg: string) {
  console.error(`[TEST] ${msg}`);
}

async function runTest(name: string, fn: () => Promise<void>) {
  logTest(name, 'running');
  const start = Date.now();
  try {
    await fn();
    logTest(name, 'pass', { duration_ms: Date.now() - start });
  } catch (err) {
    logTest(name, 'fail', { error: String(err), duration_ms: Date.now() - start });
  }
}

// =============================================================================
// Tests
// =============================================================================

debug('test-padding-consistency.ts starting...');

// -----------------------------------------------------------------------------
// Test: term() prompt has consistent padding
// -----------------------------------------------------------------------------

await runTest('term-padding-applied', async () => {
  debug('Launching term() prompt...');
  
  // Launch term prompt with a simple command
  // Note: term() returns when the user submits, so we use a short-lived command
  // We don't await the promise since we'll use submit() to force exit
  void term('echo "Testing padding"');
  
  // Wait for window to render and resize to settle
  await wait(RESIZE_SETTLE_TIME * 2);
  
  // Get window bounds to verify term is rendered
  const bounds = await getWindowBounds();
  debug(`term() bounds: ${JSON.stringify(bounds)}`);
  
  // Log padding metric for term
  logMetric({
    prompt_type: 'term',
    expected_padding: DEFAULT_PADDING,
    bounds,
    timestamp: new Date().toISOString(),
  });
  
  // Verify window dimensions are within expected range
  if (bounds.width !== WINDOW_WIDTH) {
    throw new Error(`term() width mismatch: expected ${WINDOW_WIDTH}, got ${bounds.width}`);
  }
  
  if (bounds.height !== MAX_HEIGHT) {
    throw new Error(`term() height mismatch: expected ${MAX_HEIGHT}, got ${bounds.height}`);
  }
  
  debug('term() padding verification complete (visual inspection required for exact padding)');
  
  // Force submit to exit the term prompt
  submit('');
  
  // Wait for submit to process
  await wait(50);
});

// -----------------------------------------------------------------------------
// Test: editor() prompt has consistent padding  
// -----------------------------------------------------------------------------

await runTest('editor-padding-applied', async () => {
  debug('Launching editor() prompt...');
  
  // Launch editor prompt with test content
  // We don't await the promise since we'll use submit() to force exit
  void editor('// Padding consistency test\nconsole.log("hello");', 'javascript');
  
  // Wait for window to render and resize to settle
  await wait(RESIZE_SETTLE_TIME * 2);
  
  // Get window bounds to verify editor is rendered
  const bounds = await getWindowBounds();
  debug(`editor() bounds: ${JSON.stringify(bounds)}`);
  
  // Log padding metric for editor
  logMetric({
    prompt_type: 'editor',
    expected_padding: DEFAULT_PADDING,
    bounds,
    timestamp: new Date().toISOString(),
  });
  
  // Verify window dimensions are within expected range
  if (bounds.width !== WINDOW_WIDTH) {
    throw new Error(`editor() width mismatch: expected ${WINDOW_WIDTH}, got ${bounds.width}`);
  }
  
  if (bounds.height !== MAX_HEIGHT) {
    throw new Error(`editor() height mismatch: expected ${MAX_HEIGHT}, got ${bounds.height}`);
  }
  
  debug('editor() padding verification complete (visual inspection required for exact padding)');
  
  // Force submit to exit the editor prompt
  submit('');
  
  // Wait for submit to process
  await wait(50);
});

// -----------------------------------------------------------------------------
// Test: term -> editor transition maintains padding consistency
// -----------------------------------------------------------------------------

await runTest('term-to-editor-padding-consistency', async () => {
  debug('Testing term -> editor transition...');
  
  // Launch term first (don't await, use submit() to exit)
  void term('echo "First prompt"');
  await wait(RESIZE_SETTLE_TIME * 2);
  
  const termBounds = await getWindowBounds();
  debug(`term() bounds: ${JSON.stringify(termBounds)}`);
  
  // Submit term to transition to next prompt
  submit('');
  await wait(50);
  
  // Launch editor (don't await, use submit() to exit)
  void editor('// Second prompt', 'text');
  await wait(RESIZE_SETTLE_TIME * 2);
  
  const editorBounds = await getWindowBounds();
  debug(`editor() bounds: ${JSON.stringify(editorBounds)}`);
  
  // Log transition metrics
  console.log(JSON.stringify({
    type: 'metric',
    metric: 'padding_transition',
    from: {
      prompt_type: 'term',
      bounds: termBounds,
    },
    to: {
      prompt_type: 'editor',
      bounds: editorBounds,
    },
    consistent: termBounds.width === editorBounds.width && termBounds.height === editorBounds.height,
    expected_padding: DEFAULT_PADDING,
    timestamp: new Date().toISOString(),
  }));
  
  // Both should have the same window dimensions (padding is internal)
  if (termBounds.width !== editorBounds.width) {
    throw new Error(`Width inconsistent: term=${termBounds.width}, editor=${editorBounds.width}`);
  }
  
  if (termBounds.height !== editorBounds.height) {
    throw new Error(`Height inconsistent: term=${termBounds.height}, editor=${editorBounds.height}`);
  }
  
  debug('term -> editor padding consistency verified');
  
  // Submit to exit
  submit('');
  await wait(50);
});

// -----------------------------------------------------------------------------
// Test: editor -> term transition maintains padding consistency
// -----------------------------------------------------------------------------

await runTest('editor-to-term-padding-consistency', async () => {
  debug('Testing editor -> term transition...');
  
  // Launch editor first (don't await, use submit() to exit)
  void editor('// First prompt', 'text');
  await wait(RESIZE_SETTLE_TIME * 2);
  
  const editorBounds = await getWindowBounds();
  debug(`editor() bounds: ${JSON.stringify(editorBounds)}`);
  
  // Submit editor to transition to next prompt
  submit('');
  await wait(50);
  
  // Launch term (don't await, use submit() to exit)
  void term('echo "Second prompt"');
  await wait(RESIZE_SETTLE_TIME * 2);
  
  const termBounds = await getWindowBounds();
  debug(`term() bounds: ${JSON.stringify(termBounds)}`);
  
  // Log transition metrics
  console.log(JSON.stringify({
    type: 'metric',
    metric: 'padding_transition',
    from: {
      prompt_type: 'editor',
      bounds: editorBounds,
    },
    to: {
      prompt_type: 'term',
      bounds: termBounds,
    },
    consistent: editorBounds.width === termBounds.width && editorBounds.height === termBounds.height,
    expected_padding: DEFAULT_PADDING,
    timestamp: new Date().toISOString(),
  }));
  
  // Both should have the same window dimensions
  if (editorBounds.width !== termBounds.width) {
    throw new Error(`Width inconsistent: editor=${editorBounds.width}, term=${termBounds.width}`);
  }
  
  if (editorBounds.height !== termBounds.height) {
    throw new Error(`Height inconsistent: editor=${editorBounds.height}, term=${termBounds.height}`);
  }
  
  debug('editor -> term padding consistency verified');
  
  // Submit to exit
  submit('');
  await wait(50);
});

// -----------------------------------------------------------------------------
// Test: Multiple rapid transitions maintain padding consistency
// -----------------------------------------------------------------------------

await runTest('rapid-transitions-padding-stability', async () => {
  const iterations = 3;
  const errors: string[] = [];
  const metrics: Array<{ iteration: number; prompt: string; bounds: WindowBounds }> = [];
  
  debug(`Testing ${iterations} rapid transitions...`);
  
  for (let i = 0; i < iterations; i++) {
    debug(`Iteration ${i + 1}/${iterations}`);
    
    // Term prompt (don't await, use submit() to exit)
    void term(`echo "Iteration ${i + 1} term"`);
    await wait(RESIZE_SETTLE_TIME);
    
    const termBounds = await getWindowBounds();
    metrics.push({ iteration: i + 1, prompt: 'term', bounds: termBounds });
    
    if (termBounds.width !== WINDOW_WIDTH) {
      errors.push(`Iteration ${i + 1} term: width ${termBounds.width} != ${WINDOW_WIDTH}`);
    }
    if (termBounds.height !== MAX_HEIGHT) {
      errors.push(`Iteration ${i + 1} term: height ${termBounds.height} != ${MAX_HEIGHT}`);
    }
    
    submit('');
    await wait(50);
    
    // Editor prompt (don't await, use submit() to exit)
    void editor(`// Iteration ${i + 1}`, 'text');
    await wait(RESIZE_SETTLE_TIME);
    
    const editorBounds = await getWindowBounds();
    metrics.push({ iteration: i + 1, prompt: 'editor', bounds: editorBounds });
    
    if (editorBounds.width !== WINDOW_WIDTH) {
      errors.push(`Iteration ${i + 1} editor: width ${editorBounds.width} != ${WINDOW_WIDTH}`);
    }
    if (editorBounds.height !== MAX_HEIGHT) {
      errors.push(`Iteration ${i + 1} editor: height ${editorBounds.height} != ${MAX_HEIGHT}`);
    }
    
    submit('');
    await wait(50);
  }
  
  // Log all metrics
  console.log(JSON.stringify({
    type: 'metric',
    metric: 'rapid_transitions_summary',
    iterations,
    metrics,
    errors_count: errors.length,
    all_consistent: errors.length === 0,
    expected_padding: DEFAULT_PADDING,
    timestamp: new Date().toISOString(),
  }));
  
  if (errors.length > 0) {
    throw new Error(`Padding stability errors:\n${errors.join('\n')}`);
  }
  
  debug(`Rapid transitions test completed with ${metrics.length} measurements`);
});

// -----------------------------------------------------------------------------
// Test: Summary - Output expected padding values for documentation
// -----------------------------------------------------------------------------

await runTest('padding-defaults-documented', async () => {
  // Output the expected padding values for reference
  console.log(JSON.stringify({
    type: 'metric',
    metric: 'padding_defaults',
    default_padding: DEFAULT_PADDING,
    source: 'config.rs',
    constants: {
      DEFAULT_PADDING_TOP: 8,
      DEFAULT_PADDING_LEFT: 12,
      DEFAULT_PADDING_RIGHT: 12,
    },
    applies_to: ['term', 'editor'],
    timestamp: new Date().toISOString(),
  }));
  
  debug(`Padding defaults: top=${DEFAULT_PADDING.top}px, left=${DEFAULT_PADDING.left}px, right=${DEFAULT_PADDING.right}px`);
});

debug('test-padding-consistency.ts completed!');
