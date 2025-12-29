# Smoke/E2E Tests Audit

> **Scope**: All smoke tests in `tests/smoke/`  
> **Status**: Moderate coverage with significant protocol gaps  
> **Protocol Coverage**: 17% (10/59 message types tested)

## Summary

The smoke test suite covers basic end-to-end scenarios but has critical gaps in protocol message coverage. Only 10 of 59 protocol messages have dedicated tests.

### Coverage Overview

```
Protocol Categories:
├── Window Messages:    ███░░░░░░░░░  37% (3/8)
├── Prompt Messages:    ███░░░░░░░░░  27% (4/15)
├── Input Messages:     ██░░░░░░░░░░  20% (2/10)
├── Display Messages:   █░░░░░░░░░░░  12% (1/8)
└── System Messages:    ░░░░░░░░░░░░   0% (0/18)
```

## Test File Inventory

### tests/smoke/ (47 files)

| File | Category | Status | Notes |
|------|----------|--------|-------|
| `hello-world.ts` | Basic | Complete | Sanity check |
| `hello-world-args.ts` | Interactive | Complete | User input |
| `simple-exit-test.ts` | Process | Complete | Exit handling |
| `test-window-reset.ts` | Window | Complete | State reset |
| `test-process-cleanup.ts` | Process | Complete | Cleanup |
| `test-actions-*.ts` (3) | Actions | Partial | Menu system |
| `test-arg-*.ts` (5) | Prompts | Good | Core prompts |
| `test-editor-*.ts` (4) | Editor | Good | Code editing |
| `test-div-*.ts` (3) | Display | Good | HTML rendering |
| `test-resize-*.ts` (4) | Window | Good | Size changes |
| `test-scroll-*.ts` (2) | Performance | Good | Scroll perf |
| `test-visual-*.ts` (3) | Visual | Partial | Baselines |
| `test-app-icons.ts` | Assets | Complete | Icon loading |
| `test-design-*.ts` (2) | Design | Partial | Themes |
| `test-filter-*.ts` (2) | Search | Partial | Filtering |
| `test-keyboard-*.ts` (2) | Input | Weak | Key events |
| `test-field-*.ts` (2) | Forms | Good | Form fields |
| Others (15+) | Mixed | Varies | Various features |

## Protocol Coverage Details

### Tested Messages (10)

| Message Type | Test File | Coverage |
|--------------|-----------|----------|
| `run` | hello-world.ts | Complete |
| `show` | test-window-reset.ts | Complete |
| `hide` | test-window-reset.ts | Complete |
| `arg` | test-arg-*.ts | Good |
| `div` | test-div-*.ts | Good |
| `editor` | test-editor-*.ts | Good |
| `setSize` | test-resize-*.ts | Good |
| `fields` | test-field-*.ts | Good |
| `actions` | test-actions-*.ts | Partial |
| `term` | test-term-basic.ts | Partial |

### Untested Messages (49) - CRITICAL GAPS

**Window Messages (5 untested)**
- `setPosition` - Window positioning
- `getBounds` - Window dimensions
- `setAlwaysOnTop` - Z-order
- `minimize` / `maximize`

**Prompt Messages (11 untested)**
- `form` - Multi-field forms
- `drop` - Drag and drop
- `hotkey` - Key capture
- `path` - Path selection
- `chat` - AI chat
- `mic` / `webcam` - Media capture
- `select` - Dropdowns
- `textarea` - Multi-line input
- `template` - Templates

**Input Messages (8 untested)**
- `setFilter` - **HIGH PRIORITY** - Search filtering
- `keyDown` / `keyUp` - **HIGH PRIORITY** - Keyboard
- `mouseClick` - Mouse events
- `submit` - Form submission
- `escape` - Cancel handling
- `blur` / `focus` - Focus events

**System Messages (18 untested)**
- `clipboard` operations
- `exec` - Shell commands
- `env` - Environment
- `path` operations
- `db` / `store` - Storage
- `notify` - Notifications
- All accessibility APIs

## Test Pattern

### Stdin JSON Protocol

All smoke tests use the stdin JSON protocol:

```bash
echo '{"type":"run","path":"'$(pwd)'/tests/smoke/test-example.ts"}' | \
  SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
```

### Test Script Structure

```typescript
// tests/smoke/test-example.ts
import '../../scripts/kit-sdk';

console.error('[SMOKE] Test starting...');

try {
  // Test code
  const result = await arg('Test prompt');
  console.error(`[SMOKE] Result: ${result}`);
  
  // Verification
  if (result === expected) {
    console.error('[SMOKE] PASS');
    process.exit(0);
  } else {
    console.error('[SMOKE] FAIL: unexpected result');
    process.exit(1);
  }
} catch (err) {
  console.error(`[SMOKE] ERROR: ${err}`);
  process.exit(1);
}
```

### Automated Input Simulation

For tests requiring user input, use the test harness:

```typescript
// tests/autonomous/test-with-input.ts
import { TestHarness } from './test-harness';

const harness = new TestHarness();
await harness.start('tests/smoke/test-arg.ts');

// Wait for prompt to appear
await harness.waitForPrompt('arg');

// Simulate input
await harness.sendKeys(['a', 'p', 'p', 'l', 'e']);
await harness.sendKey('Enter');

// Verify result
const result = await harness.getResult();
expect(result).toBe('apple');
```

## Gap Analysis

### High Priority Gaps

| Gap | Risk | Impact |
|-----|------|--------|
| `setFilter` not tested | High | Search broken undetected |
| `keyDown/keyUp` not tested | High | Keyboard nav broken |
| `escape` not tested | High | Cancel behavior broken |
| `submit` not tested | High | Form submission broken |
| Clipboard ops not tested | Medium | Copy/paste broken |

### Medium Priority Gaps

| Gap | Risk | Impact |
|-----|------|--------|
| `hotkey` not tested | Medium | Hotkey capture broken |
| `form` not tested | Medium | Multi-field forms broken |
| `drop` not tested | Medium | Drag-drop broken |
| Storage APIs not tested | Medium | Data persistence broken |

### Low Priority Gaps

| Gap | Risk | Notes |
|-----|------|-------|
| Media capture | Low | Hardware-dependent |
| Notifications | Low | OS-specific |
| Accessibility | Low | Manual testing may suffice |

## Recommendations

### P0: Add Critical Protocol Tests

```typescript
// tests/smoke/test-filter-protocol.ts
import '../../scripts/kit-sdk';

// Test setFilter message
const choices = ['Apple', 'Banana', 'Cherry'];
const argPromise = arg('Choose fruit', choices);

// Simulate filter - this tests the setFilter protocol message
await setFilter('ban');  // Should filter to 'Banana'

// Verify filter applied
const visible = await getVisibleChoices();
console.error(`[SMOKE] Visible after filter: ${visible.join(', ')}`);

if (visible.length === 1 && visible[0] === 'Banana') {
  console.error('[SMOKE] PASS');
}
```

### P1: Add Keyboard Navigation Tests

```typescript
// tests/smoke/test-keyboard-nav.ts
import '../../scripts/kit-sdk';

const choices = ['First', 'Second', 'Third'];
const argPromise = arg('Navigate', choices);

// Test arrow key navigation
await simulateKey('ArrowDown');
await simulateKey('ArrowDown');
await simulateKey('Enter');

const result = await argPromise;
if (result === 'Third') {
  console.error('[SMOKE] PASS: keyboard nav works');
}
```

### P2: Expand Protocol Coverage Matrix

Create a test matrix tracking all 59 protocol messages:

```typescript
// tests/protocol-coverage-matrix.ts
const PROTOCOL_MESSAGES = {
  // Window
  run: 'tested',
  show: 'tested',
  hide: 'tested',
  setPosition: 'TODO',
  getBounds: 'TODO',
  // ... etc
};

// Generate coverage report
function reportCoverage() {
  const tested = Object.values(PROTOCOL_MESSAGES).filter(s => s === 'tested').length;
  const total = Object.keys(PROTOCOL_MESSAGES).length;
  console.log(`Protocol Coverage: ${tested}/${total} (${(tested/total*100).toFixed(1)}%)`);
}
```

## Test Execution

### Manual Execution

```bash
# Build first
cargo build

# Run single smoke test
echo '{"type":"run","path":"'$(pwd)'/tests/smoke/hello-world.ts"}' | \
  SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1

# Run with timeout
timeout 30 bash -c 'echo ... | ./target/debug/script-kit-gpui 2>&1'
```

### Batch Execution

```bash
# Run all smoke tests
for test in tests/smoke/test-*.ts; do
  echo "Running: $test"
  timeout 30 bash -c 'echo '"'"'{"type":"run","path":"'"$(pwd)/$test"'"}'"'"' | ./target/debug/script-kit-gpui 2>&1'
done
```

## Test Quality Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Protocol Coverage | 17% | 50% |
| Test Files | 47 | 60 |
| Critical Paths | 40% | 90% |
| Avg Test Time | 3-5s | <5s |

---

*Part of [Testing Audit](../TESTING_AUDIT.md)*
