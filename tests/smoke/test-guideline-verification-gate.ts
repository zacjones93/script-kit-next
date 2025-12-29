// Name: Guideline Test - Verification Gate
// Description: Tests that agents follow the verification gate protocol before committing

/**
 * GUIDELINE TEST: test-guideline-verification-gate.ts
 * 
 * This script simulates and tests the verification gate protocol that all
 * AI agents MUST follow before making git commits.
 * 
 * The verification gate requires:
 * 1. cargo check - must pass (no compilation errors)
 * 2. cargo clippy --all-targets -- -D warnings - must pass (no lints)
 * 3. cargo test - must pass (all tests green)
 * 
 * This test validates:
 * - Verification commands are tracked
 * - Verification happens BEFORE commit (timing assertion)
 * - Checklist completion status
 * - Evidence of verification in logs
 * 
 * Usage:
 *   echo '{"type":"run","path":"tests/smoke/test-guideline-verification-gate.ts"}' | \
 *     SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
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
  details?: Record<string, unknown>;
}

interface VerificationStep {
  command: string;
  executed: boolean;
  timestamp: number | null;
  exitCode: number | null;
  duration_ms: number | null;
}

interface AgentWorkSession {
  sessionStart: number;
  codeChangeTimestamp: number | null;
  verificationSteps: {
    cargoCheck: VerificationStep;
    cargoClippy: VerificationStep;
    cargoTest: VerificationStep;
  };
  commitTimestamp: number | null;
  commitAllowed: boolean;
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
  console.error(`[GUIDELINE-TEST] ${msg}`);
}

// =============================================================================
// Mock Agent Work Session
// =============================================================================

function createMockSession(): AgentWorkSession {
  return {
    sessionStart: Date.now(),
    codeChangeTimestamp: null,
    verificationSteps: {
      cargoCheck: {
        command: 'cargo check',
        executed: false,
        timestamp: null,
        exitCode: null,
        duration_ms: null
      },
      cargoClippy: {
        command: 'cargo clippy --all-targets -- -D warnings',
        executed: false,
        timestamp: null,
        exitCode: null,
        duration_ms: null
      },
      cargoTest: {
        command: 'cargo test',
        executed: false,
        timestamp: null,
        exitCode: null,
        duration_ms: null
      }
    },
    commitTimestamp: null,
    commitAllowed: false
  };
}

function simulateCodeChange(session: AgentWorkSession): void {
  session.codeChangeTimestamp = Date.now();
  debug(`Code change detected at ${session.codeChangeTimestamp}`);
}

function simulateVerificationStep(
  session: AgentWorkSession, 
  step: keyof AgentWorkSession['verificationSteps'],
  exitCode: number = 0,
  duration_ms: number = 100
): void {
  const verification = session.verificationSteps[step];
  verification.executed = true;
  verification.timestamp = Date.now();
  verification.exitCode = exitCode;
  verification.duration_ms = duration_ms;
  debug(`Executed: ${verification.command} (exit: ${exitCode}, duration: ${duration_ms}ms)`);
}

function simulateCommitAttempt(session: AgentWorkSession): { allowed: boolean; reason: string } {
  session.commitTimestamp = Date.now();
  
  // Check all verification steps were executed
  const { cargoCheck, cargoClippy, cargoTest } = session.verificationSteps;
  
  if (!cargoCheck.executed) {
    return { allowed: false, reason: 'cargo check was not run' };
  }
  if (!cargoClippy.executed) {
    return { allowed: false, reason: 'cargo clippy was not run' };
  }
  if (!cargoTest.executed) {
    return { allowed: false, reason: 'cargo test was not run' };
  }
  
  // Check all passed (exit code 0)
  if (cargoCheck.exitCode !== 0) {
    return { allowed: false, reason: `cargo check failed with exit code ${cargoCheck.exitCode}` };
  }
  if (cargoClippy.exitCode !== 0) {
    return { allowed: false, reason: `cargo clippy failed with exit code ${cargoClippy.exitCode}` };
  }
  if (cargoTest.exitCode !== 0) {
    return { allowed: false, reason: `cargo test failed with exit code ${cargoTest.exitCode}` };
  }
  
  // Check timing: verification MUST happen AFTER code change and BEFORE commit
  if (session.codeChangeTimestamp) {
    if (cargoCheck.timestamp! < session.codeChangeTimestamp) {
      return { allowed: false, reason: 'cargo check was run BEFORE code change (stale verification)' };
    }
    if (cargoClippy.timestamp! < session.codeChangeTimestamp) {
      return { allowed: false, reason: 'cargo clippy was run BEFORE code change (stale verification)' };
    }
    if (cargoTest.timestamp! < session.codeChangeTimestamp) {
      return { allowed: false, reason: 'cargo test was run BEFORE code change (stale verification)' };
    }
  }
  
  session.commitAllowed = true;
  return { allowed: true, reason: 'All verification gates passed' };
}

function generateChecklist(session: AgentWorkSession): string[] {
  const checklist: string[] = [];
  const { cargoCheck, cargoClippy, cargoTest } = session.verificationSteps;
  
  checklist.push(cargoCheck.executed && cargoCheck.exitCode === 0 
    ? '✅ cargo check - PASSED' 
    : '❌ cargo check - NOT RUN or FAILED');
    
  checklist.push(cargoClippy.executed && cargoClippy.exitCode === 0
    ? '✅ cargo clippy - PASSED'
    : '❌ cargo clippy - NOT RUN or FAILED');
    
  checklist.push(cargoTest.executed && cargoTest.exitCode === 0
    ? '✅ cargo test - PASSED'
    : '❌ cargo test - NOT RUN or FAILED');
    
  return checklist;
}

// =============================================================================
// Tests
// =============================================================================

debug('test-guideline-verification-gate.ts starting...');

// -----------------------------------------------------------------------------
// Test 1: Scenario where agent SKIPS verification (should FAIL commit)
// -----------------------------------------------------------------------------
const test1 = 'verification-skipped-commit-blocked';
logTest(test1, 'running');
const start1 = Date.now();

try {
  debug('Test 1: Agent makes code change but skips verification');
  
  const session = createMockSession();
  
  // Agent makes a code change
  simulateCodeChange(session);
  
  // Agent tries to commit WITHOUT running verification
  const result = simulateCommitAttempt(session);
  
  debug(`Commit attempt result: allowed=${result.allowed}, reason="${result.reason}"`);
  
  if (!result.allowed && result.reason.includes('was not run')) {
    logTest(test1, 'pass', { 
      result: result.reason,
      duration_ms: Date.now() - start1,
      details: {
        scenario: 'Agent skipped verification',
        expected: 'Commit should be blocked',
        actual: 'Commit was blocked',
        checklist: generateChecklist(session)
      }
    });
  } else {
    logTest(test1, 'fail', { 
      error: 'Commit should have been blocked when verification was skipped',
      duration_ms: Date.now() - start1 
    });
  }
} catch (err) {
  logTest(test1, 'fail', { error: String(err), duration_ms: Date.now() - start1 });
}

// -----------------------------------------------------------------------------
// Test 2: Scenario where agent runs verification CORRECTLY (should allow commit)
// -----------------------------------------------------------------------------
const test2 = 'verification-complete-commit-allowed';
logTest(test2, 'running');
const start2 = Date.now();

try {
  debug('Test 2: Agent runs full verification before commit');
  
  const session = createMockSession();
  
  // Agent makes a code change
  simulateCodeChange(session);
  
  // Small delay to ensure timestamp ordering
  await new Promise(r => setTimeout(r, 10));
  
  // Agent runs all verification steps
  simulateVerificationStep(session, 'cargoCheck', 0, 1500);
  await new Promise(r => setTimeout(r, 5));
  simulateVerificationStep(session, 'cargoClippy', 0, 2000);
  await new Promise(r => setTimeout(r, 5));
  simulateVerificationStep(session, 'cargoTest', 0, 5000);
  
  // Agent tries to commit AFTER verification
  const result = simulateCommitAttempt(session);
  
  debug(`Commit attempt result: allowed=${result.allowed}, reason="${result.reason}"`);
  
  if (result.allowed) {
    logTest(test2, 'pass', { 
      result: result.reason,
      duration_ms: Date.now() - start2,
      details: {
        scenario: 'Agent ran full verification',
        expected: 'Commit should be allowed',
        actual: 'Commit was allowed',
        checklist: generateChecklist(session)
      }
    });
  } else {
    logTest(test2, 'fail', { 
      error: `Commit should have been allowed: ${result.reason}`,
      duration_ms: Date.now() - start2 
    });
  }
} catch (err) {
  logTest(test2, 'fail', { error: String(err), duration_ms: Date.now() - start2 });
}

// -----------------------------------------------------------------------------
// Test 3: Scenario where verification was run BEFORE code change (stale)
// -----------------------------------------------------------------------------
const test3 = 'verification-stale-commit-blocked';
logTest(test3, 'running');
const start3 = Date.now();

try {
  debug('Test 3: Agent runs verification BEFORE code change (stale)');
  
  const session = createMockSession();
  
  // Agent runs verification first (WRONG ORDER)
  simulateVerificationStep(session, 'cargoCheck', 0, 1500);
  simulateVerificationStep(session, 'cargoClippy', 0, 2000);
  simulateVerificationStep(session, 'cargoTest', 0, 5000);
  
  // Small delay
  await new Promise(r => setTimeout(r, 10));
  
  // Agent makes code change AFTER verification
  simulateCodeChange(session);
  
  // Agent tries to commit with stale verification
  const result = simulateCommitAttempt(session);
  
  debug(`Commit attempt result: allowed=${result.allowed}, reason="${result.reason}"`);
  
  if (!result.allowed && result.reason.includes('BEFORE code change')) {
    logTest(test3, 'pass', { 
      result: result.reason,
      duration_ms: Date.now() - start3,
      details: {
        scenario: 'Verification ran before code change',
        expected: 'Commit should be blocked (stale verification)',
        actual: 'Commit was blocked',
        timingViolation: 'verification_timestamp < code_change_timestamp'
      }
    });
  } else {
    logTest(test3, 'fail', { 
      error: 'Commit should have been blocked with stale verification',
      duration_ms: Date.now() - start3 
    });
  }
} catch (err) {
  logTest(test3, 'fail', { error: String(err), duration_ms: Date.now() - start3 });
}

// -----------------------------------------------------------------------------
// Test 4: Scenario where cargo clippy fails (should block commit)
// -----------------------------------------------------------------------------
const test4 = 'verification-clippy-failed-commit-blocked';
logTest(test4, 'running');
const start4 = Date.now();

try {
  debug('Test 4: cargo clippy fails with warnings');
  
  const session = createMockSession();
  
  // Agent makes a code change
  simulateCodeChange(session);
  
  await new Promise(r => setTimeout(r, 10));
  
  // Agent runs verification but clippy fails
  simulateVerificationStep(session, 'cargoCheck', 0, 1500);
  simulateVerificationStep(session, 'cargoClippy', 1, 2000);  // Exit code 1 = failure
  simulateVerificationStep(session, 'cargoTest', 0, 5000);
  
  // Agent tries to commit despite clippy failure
  const result = simulateCommitAttempt(session);
  
  debug(`Commit attempt result: allowed=${result.allowed}, reason="${result.reason}"`);
  
  if (!result.allowed && result.reason.includes('clippy failed')) {
    logTest(test4, 'pass', { 
      result: result.reason,
      duration_ms: Date.now() - start4,
      details: {
        scenario: 'cargo clippy failed',
        expected: 'Commit should be blocked',
        actual: 'Commit was blocked',
        failedStep: 'cargo clippy --all-targets -- -D warnings'
      }
    });
  } else {
    logTest(test4, 'fail', { 
      error: 'Commit should have been blocked when clippy failed',
      duration_ms: Date.now() - start4 
    });
  }
} catch (err) {
  logTest(test4, 'fail', { error: String(err), duration_ms: Date.now() - start4 });
}

// -----------------------------------------------------------------------------
// Test 5: Scenario where cargo test fails (should block commit)
// -----------------------------------------------------------------------------
const test5 = 'verification-test-failed-commit-blocked';
logTest(test5, 'running');
const start5 = Date.now();

try {
  debug('Test 5: cargo test fails');
  
  const session = createMockSession();
  
  // Agent makes a code change
  simulateCodeChange(session);
  
  await new Promise(r => setTimeout(r, 10));
  
  // Agent runs verification but tests fail
  simulateVerificationStep(session, 'cargoCheck', 0, 1500);
  simulateVerificationStep(session, 'cargoClippy', 0, 2000);
  simulateVerificationStep(session, 'cargoTest', 1, 5000);  // Exit code 1 = failure
  
  // Agent tries to commit despite test failure
  const result = simulateCommitAttempt(session);
  
  debug(`Commit attempt result: allowed=${result.allowed}, reason="${result.reason}"`);
  
  if (!result.allowed && result.reason.includes('test failed')) {
    logTest(test5, 'pass', { 
      result: result.reason,
      duration_ms: Date.now() - start5,
      details: {
        scenario: 'cargo test failed',
        expected: 'Commit should be blocked',
        actual: 'Commit was blocked',
        failedStep: 'cargo test'
      }
    });
  } else {
    logTest(test5, 'fail', { 
      error: 'Commit should have been blocked when tests failed',
      duration_ms: Date.now() - start5 
    });
  }
} catch (err) {
  logTest(test5, 'fail', { error: String(err), duration_ms: Date.now() - start5 });
}

// -----------------------------------------------------------------------------
// Test 6: Partial verification (only cargo check run)
// -----------------------------------------------------------------------------
const test6 = 'verification-partial-commit-blocked';
logTest(test6, 'running');
const start6 = Date.now();

try {
  debug('Test 6: Agent only runs cargo check (partial verification)');
  
  const session = createMockSession();
  
  // Agent makes a code change
  simulateCodeChange(session);
  
  await new Promise(r => setTimeout(r, 10));
  
  // Agent only runs cargo check, skips clippy and test
  simulateVerificationStep(session, 'cargoCheck', 0, 1500);
  // cargoClippy and cargoTest NOT run
  
  // Agent tries to commit with partial verification
  const result = simulateCommitAttempt(session);
  
  debug(`Commit attempt result: allowed=${result.allowed}, reason="${result.reason}"`);
  
  if (!result.allowed && result.reason.includes('clippy was not run')) {
    logTest(test6, 'pass', { 
      result: result.reason,
      duration_ms: Date.now() - start6,
      details: {
        scenario: 'Only cargo check was run',
        expected: 'Commit should be blocked (missing clippy and test)',
        actual: 'Commit was blocked',
        checklist: generateChecklist(session)
      }
    });
  } else {
    logTest(test6, 'fail', { 
      error: 'Commit should have been blocked with partial verification',
      duration_ms: Date.now() - start6 
    });
  }
} catch (err) {
  logTest(test6, 'fail', { error: String(err), duration_ms: Date.now() - start6 });
}

// -----------------------------------------------------------------------------
// Summary and Guidelines Display
// -----------------------------------------------------------------------------
debug('test-guideline-verification-gate.ts tests completed!');

const summaryHtml = md(`# Verification Gate Tests Complete

## Test Results Summary

All verification gate scenarios have been tested.

| # | Test | Scenario |
|---|------|----------|
| 1 | verification-skipped | Agent skips verification entirely |
| 2 | verification-complete | Agent runs full verification correctly |
| 3 | verification-stale | Verification ran BEFORE code change |
| 4 | clippy-failed | cargo clippy fails with warnings |
| 5 | test-failed | cargo test fails |
| 6 | partial-verification | Only cargo check was run |

---

## Verification Gate Protocol (MANDATORY)

Before EVERY commit, agents MUST run:

\`\`\`bash
cargo check && cargo clippy --all-targets -- -D warnings && cargo test
\`\`\`

### Timing Requirements

1. **Code change** happens first
2. **Verification** runs AFTER code change
3. **Commit** only happens AFTER verification passes

### Checklist for Agents

- [ ] \`cargo check\` passes (no compilation errors)
- [ ] \`cargo clippy --all-targets -- -D warnings\` passes (no lints)
- [ ] \`cargo test\` passes (all tests green)
- [ ] All verification ran AFTER the code changes
- [ ] Evidence of passing output is in logs

---

*Check the JSONL output above for detailed test results*

Press Escape or click to exit.`);

await div(summaryHtml);

debug('test-guideline-verification-gate.ts exiting...');
