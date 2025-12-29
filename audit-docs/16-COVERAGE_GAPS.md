# Coverage Gaps Analysis

> **Scope**: Prioritized analysis of testing gaps  
> **Risk Assessment**: High to Low priority  
> **Action Items**: Concrete remediation steps

## Summary

This document consolidates all coverage gaps identified across Rust, TypeScript, smoke tests, and visual testing. Gaps are prioritized by risk and impact.

### Gap Severity Matrix

```
                    IMPACT
                High         Medium        Low
              ┌───────────┬───────────┬───────────┐
    High      │  P0       │  P1       │  P2       │
LIKELIHOOD    │ CRITICAL  │ HIGH      │ MEDIUM    │
              ├───────────┼───────────┼───────────┤
    Medium    │  P1       │  P2       │  P3       │
              │ HIGH      │ MEDIUM    │ LOW       │
              ├───────────┼───────────┼───────────┤
    Low       │  P2       │  P3       │  P3       │
              │ MEDIUM    │ LOW       │ LOW       │
              └───────────┴───────────┴───────────┘
```

## P0 - Critical Gaps

### 1. No CI/CD Pipeline

| Aspect | Detail |
|--------|--------|
| **Gap** | No automated testing on PRs |
| **Risk** | Broken code can be merged undetected |
| **Impact** | Regressions, quality degradation |
| **Likelihood** | High (happens regularly without CI) |
| **Effort** | 2-4 hours |

**Remediation**:
```yaml
# Create .github/workflows/test.yml
# See audit-docs/15-TEST_INFRASTRUCTURE.md for full config
```

### 2. Protocol Coverage Only 17%

| Aspect | Detail |
|--------|--------|
| **Gap** | 49/59 protocol messages untested |
| **Risk** | Core functionality broken without detection |
| **Impact** | User-facing bugs |
| **Likelihood** | High (untested code breaks) |
| **Effort** | 8-16 hours |

**Missing Critical Messages**:
| Message | Risk | Priority |
|---------|------|----------|
| `setFilter` | Search broken | P0 |
| `keyDown/keyUp` | Keyboard nav broken | P0 |
| `escape` | Cancel broken | P0 |
| `submit` | Forms broken | P0 |

**Remediation**:
```typescript
// Create tests/smoke/test-protocol-*.ts for each message type
// Start with keyboard and filter messages
```

### 3. Critical SDK Methods Untested

| Aspect | Detail |
|--------|--------|
| **Gap** | Key SDK methods have no tests |
| **Risk** | Core features silently broken |
| **Impact** | Script authors hit bugs |
| **Likelihood** | Medium |
| **Effort** | 4-8 hours |

**Missing Methods**:
| Method | Category | Risk |
|--------|----------|------|
| `captureScreenshot()` | Visual | High - Used in visual testing |
| `exec()` | System | High - Shell command execution |
| `db()` | Storage | High - Data persistence |
| `store()` | Storage | High - Data persistence |
| `env()` | System | Medium - Environment access |

**Remediation**:
```typescript
// Create tests/sdk/test-capture-screenshot.ts
// Create tests/sdk/test-exec.ts
// Create tests/sdk/test-db.ts
// Create tests/sdk/test-store.ts
```

## P1 - High Priority Gaps

### 4. Ignored Rust Doctests

| Aspect | Detail |
|--------|--------|
| **Gap** | 46 doctests are ignored |
| **Risk** | Documentation examples may be wrong |
| **Impact** | Developer confusion |
| **Likelihood** | Medium |
| **Effort** | 4-6 hours |

**Remediation**:
```bash
# Review each ignored doctest:
grep -r "ignore" src/*.rs | grep "//!"

# For each:
# - Fix if example is valid
# - Convert to #[test] if complex
# - Remove if obsolete
```

### 5. Visual Testing Not CI-Ready

| Aspect | Detail |
|--------|--------|
| **Gap** | Visual tests can't run in CI |
| **Risk** | UI regressions undetected |
| **Impact** | Visual bugs in production |
| **Likelihood** | Medium |
| **Effort** | 8-16 hours |

**Blockers**:
- No headless mode
- Only 2 baseline images
- macOS only

**Remediation**:
See [Visual Testing](14-VISUAL_TESTING.md) for detailed plan.

### 6. Keyboard Navigation Untested

| Aspect | Detail |
|--------|--------|
| **Gap** | Arrow key navigation not tested |
| **Risk** | Navigation broken without detection |
| **Impact** | Core UX broken |
| **Likelihood** | Medium |
| **Effort** | 2-4 hours |

**Remediation**:
```typescript
// tests/smoke/test-keyboard-navigation.ts
import '../../scripts/kit-sdk';

const choices = ['First', 'Second', 'Third'];
await arg('Navigate with arrows', choices);

// Simulate: ArrowDown, ArrowDown, Enter
// Verify: result === 'Third'
```

## P2 - Medium Priority Gaps

### 7. TypeScript Test Parallelization

| Aspect | Detail |
|--------|--------|
| **Gap** | Tests run sequentially |
| **Risk** | Slow feedback cycle |
| **Impact** | Developer productivity |
| **Likelihood** | Low (just slower) |
| **Effort** | 4-8 hours |

**Remediation**:
```typescript
// Modify scripts/test-runner.ts
const results = await Promise.all(
  testFiles.map(file => runTest(file))
);
```

### 8. Storage API Coverage (50%)

| Aspect | Detail |
|--------|--------|
| **Gap** | `db()`, `store()`, `cache()` undertested |
| **Risk** | Data loss bugs |
| **Impact** | User data issues |
| **Likelihood** | Low |
| **Effort** | 4 hours |

**Remediation**:
```typescript
// tests/sdk/test-storage-apis.ts
// Test write, read, update, delete for each storage method
```

### 9. Form/Input Edge Cases

| Aspect | Detail |
|--------|--------|
| **Gap** | Complex form validation untested |
| **Risk** | Validation bugs |
| **Impact** | User frustration |
| **Likelihood** | Low |
| **Effort** | 4 hours |

**Remediation**:
```typescript
// tests/sdk/test-form-validation.ts
// Test required fields, format validation, error messages
```

## P3 - Low Priority Gaps

### 10. Media Capture APIs

| Aspect | Detail |
|--------|--------|
| **Gap** | `mic()`, `webcam()` minimally tested |
| **Risk** | Media capture broken |
| **Impact** | Media features broken |
| **Likelihood** | Low |
| **Effort** | 4 hours |

**Note**: Hardware-dependent, may need manual testing.

### 11. System Test Coverage

| Aspect | Detail |
|--------|--------|
| **Gap** | Feature-gated tests rarely run |
| **Risk** | System integration issues |
| **Impact** | Platform-specific bugs |
| **Likelihood** | Low |
| **Effort** | 2 hours |

**Remediation**:
```bash
# Add to CI (macOS only):
cargo test --features system-tests
```

### 12. Cross-Platform Testing

| Aspect | Detail |
|--------|--------|
| **Gap** | Only macOS tested |
| **Risk** | Linux/Windows issues |
| **Impact** | Platform bugs |
| **Likelihood** | Low (macOS primary target) |
| **Effort** | 16+ hours |

**Note**: Defer until cross-platform support is prioritized.

## Remediation Roadmap

### Week 1 (P0 Items)

| Day | Task | Hours |
|-----|------|-------|
| Mon | Create GitHub Actions workflow | 2 |
| Mon | Enable branch protection | 1 |
| Tue | Add `setFilter` smoke test | 2 |
| Tue | Add `keyDown/keyUp` smoke tests | 2 |
| Wed | Add `escape` and `submit` tests | 2 |
| Wed | Test `captureScreenshot()` | 2 |
| Thu | Test `exec()` | 2 |
| Thu | Test `db()` and `store()` | 2 |
| Fri | Review and document | 2 |

**Week 1 Total**: ~17 hours

### Week 2 (P1 Items)

| Day | Task | Hours |
|-----|------|-------|
| Mon | Review 15 ignored doctests | 3 |
| Tue | Review 15 ignored doctests | 3 |
| Wed | Review 16 ignored doctests | 3 |
| Thu | Add keyboard navigation tests | 3 |
| Fri | Create 5 visual baselines | 4 |

**Week 2 Total**: ~16 hours

### Week 3+ (P2/P3 Items)

- Parallelize TypeScript tests
- Expand storage API tests
- Add form validation tests
- Media capture testing
- Cross-platform investigation

## Coverage Targets

### Current vs Target

| Category | Current | Target | Gap |
|----------|---------|--------|-----|
| Rust Unit Tests | 843 | 900 | +57 |
| Ignored Doctests | 46 | <10 | -36 |
| SDK Method Coverage | 78% | 90% | +12% |
| Protocol Coverage | 17% | 50% | +33% |
| Visual Baselines | 2 | 15 | +13 |
| CI Pipeline | None | Full | +1 |

### Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| All PRs run CI | 100% | GitHub Actions logs |
| Test failures block merge | Yes | Branch protection |
| Protocol coverage | 50% | Test matrix tracking |
| SDK coverage | 90% | Method coverage report |
| Visual baselines | 15+ | Baseline file count |

---

*Part of [Testing Audit](../TESTING_AUDIT.md)*
