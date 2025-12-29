# TypeScript SDK Tests Audit

> **Scope**: All TypeScript tests in `tests/sdk/`  
> **Status**: Good coverage with critical gaps  
> **Coverage**: 78.2% (61/78 SDK methods tested)

## Summary

The TypeScript SDK test suite covers most core functionality but has critical gaps in utility methods and storage APIs. Tests use a consistent JSONL output pattern for machine parsing.

### Coverage by Tier

```
Tier 1 - Core Prompts:     ████████████ 100% (arg, div, editor, form)
Tier 2 - Input Types:      ████████████ 100% (hotkey, fields, drop)
Tier 3 - Display:          ███████████░  92% (term, toast, setPanel)
Tier 4 - Advanced:         █████████░░░  85% (actions, chat, widget)
Tier 5 - Utilities:        ██████░░░░░░  55% (clipboard, path, env)
Tier 6 - Storage:          █████░░░░░░░  50% (db, store, cache)
```

## Test File Inventory

### tests/sdk/ (21 files)

| File | Methods Tested | Status |
|------|----------------|--------|
| `test-arg.ts` | arg(), arg options | Complete |
| `test-div.ts` | div(), html rendering | Complete |
| `test-editor.ts` | editor(), syntax | Complete |
| `test-fields.ts` | fields(), form validation | Complete |
| `test-form.ts` | form(), multi-field | Complete |
| `test-hotkey.ts` | hotkey(), key capture | Complete |
| `test-term.ts` | term(), terminal output | Complete |
| `test-chat.ts` | chat(), AI interactions | Complete |
| `test-actions.ts` | actions(), menu system | Complete |
| `test-path.ts` | path(), file paths | Complete |
| `test-clipboard-history.ts` | clipboard history | Partial |
| `test-file-search.ts` | file search | Partial |
| `test-scroll-perf.ts` | scroll performance | Complete |
| `test-textarea.ts` | textarea input | Complete |
| `test-toast.ts` | toast notifications | Complete |
| `test-template.ts` | template system | Complete |
| `test-mic.ts` | microphone input | Partial |
| `test-webcam.ts` | webcam capture | Partial |
| `test-select.ts` | select dropdowns | Complete |
| `test-widget.ts` | widget system | Complete |
| `test-import-redirect.ts` | import handling | Complete |

## Test Pattern

### Standard Test Structure

```typescript
// tests/sdk/test-example.ts
import '../../scripts/kit-sdk';

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

// Test implementation
const testName = 'example-test';
logTest(testName, 'running');
const start = Date.now();

try {
  const result = await someFunction('input');
  
  if (result === expected) {
    logTest(testName, 'pass', { result, duration_ms: Date.now() - start });
  } else {
    logTest(testName, 'fail', { 
      error: `Expected "${expected}", got "${result}"`,
      duration_ms: Date.now() - start 
    });
  }
} catch (err) {
  logTest(testName, 'fail', { error: String(err), duration_ms: Date.now() - start });
}

process.exit(0);
```

### JSONL Output Format

```json
{"test": "arg-string-choices", "status": "running", "timestamp": "2024-12-27T10:30:45.123Z"}
{"test": "arg-string-choices", "status": "pass", "result": "Apple", "duration_ms": 45}
```

## Coverage Gaps

### Critical - Untested Methods (HIGH PRIORITY)

| Method | Category | Risk | Notes |
|--------|----------|------|-------|
| `captureScreenshot()` | Visual | High | Used in visual testing |
| `getWindowBounds()` | Window | High | Layout testing |
| `exec()` | System | High | Shell execution |
| `db()` | Storage | High | Database operations |
| `store()` | Storage | High | Persistent storage |
| `env()` | System | Medium | Environment access |
| `drop()` | Input | Medium | Drag & drop |

### Tier 5 Utilities (55% coverage)

| Method | Tested | Notes |
|--------|--------|-------|
| `clipboard()` | Yes | Basic operations |
| `clipboardHistory()` | Partial | Limited scenarios |
| `path()` | Yes | Path operations |
| `home()` | Yes | Home directory |
| `env()` | No | **Needs tests** |
| `exec()` | No | **Needs tests** |
| `getSelectedText()` | No | Needs accessibility |
| `getActiveApp()` | No | Needs accessibility |

### Tier 6 Storage (50% coverage)

| Method | Tested | Notes |
|--------|--------|-------|
| `db()` | No | **Critical gap** |
| `store()` | No | **Critical gap** |
| `cache()` | Partial | Basic only |
| `trash()` | No | File operations |

## Test Runner

### scripts/test-runner.ts

The test runner executes SDK tests with features:
- 30-second timeout per test
- JSONL output parsing
- Summary reporting
- Exit code handling

```bash
# Run all SDK tests
bun run scripts/test-runner.ts

# Run specific test
bun run scripts/test-runner.ts tests/sdk/test-arg.ts

# Run with filter
bun run scripts/test-runner.ts tests/sdk/test-*.ts
```

### scripts/test-harness.ts

Extended harness for integration scenarios:
- Multiple test file coordination
- Aggregate reporting
- CI-compatible output

## Recommendations

### P0: Add Critical Method Tests

```typescript
// tests/sdk/test-capture-screenshot.ts
import '../../scripts/kit-sdk';

const testName = 'capture-screenshot';
logTest(testName, 'running');

try {
  await div('<div class="p-4 bg-blue-500">Test</div>');
  await new Promise(r => setTimeout(r, 500));
  
  const screenshot = await captureScreenshot();
  
  if (screenshot.width > 0 && screenshot.height > 0 && screenshot.data) {
    logTest(testName, 'pass', { 
      result: `${screenshot.width}x${screenshot.height}`,
      duration_ms: Date.now() - start 
    });
  } else {
    logTest(testName, 'fail', { error: 'Invalid screenshot data' });
  }
} catch (err) {
  logTest(testName, 'fail', { error: String(err) });
}
```

### P1: Add Storage Tests

```typescript
// tests/sdk/test-db.ts
import '../../scripts/kit-sdk';

const testName = 'db-operations';
logTest(testName, 'running');

try {
  const testKey = 'test-key-' + Date.now();
  
  // Write
  await db(testKey, { value: 'test' });
  
  // Read
  const result = await db(testKey);
  
  // Cleanup
  await db(testKey, null);
  
  if (result?.value === 'test') {
    logTest(testName, 'pass', { result });
  } else {
    logTest(testName, 'fail', { error: 'Value mismatch' });
  }
} catch (err) {
  logTest(testName, 'fail', { error: String(err) });
}
```

### P2: Parallelize Test Execution

Current limitation: Tests run sequentially.

**Option 1**: Bun's built-in test runner
```bash
bun test tests/sdk/*.ts
```

**Option 2**: Custom parallel runner
```typescript
// scripts/parallel-test-runner.ts
const testFiles = await glob('tests/sdk/test-*.ts');
const results = await Promise.all(
  testFiles.map(file => runTest(file))
);
```

## Test Quality Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Method Coverage | 78% | 90% |
| Test Files | 21 | 30 |
| Avg Test Time | 2-5s | <3s |
| JSONL Compliance | 100% | 100% |

## Running Tests

```bash
# All SDK tests
bun run scripts/test-runner.ts

# Single test file
bun run tests/sdk/test-arg.ts

# With full GPUI integration
echo '{"type":"run","path":"'$(pwd)'/tests/sdk/test-arg.ts"}' | \
  SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1

# With timeout
timeout 30 bun run tests/sdk/test-arg.ts
```

---

*Part of [Testing Audit](../TESTING_AUDIT.md)*
