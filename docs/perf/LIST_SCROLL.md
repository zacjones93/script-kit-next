# List Virtualization & Scroll Performance Audit

## Executive Summary

This document audits the scroll and virtualization implementation in Script Kit GPUI. The codebase uses GPUI's `uniform_list` for virtualized rendering across multiple list components. While the core patterns are sound, there are opportunities for optimization around scroll handle overhead, event coalescing, and item height consistency.

---

## 1. Virtualization Analysis

### 1.1 uniform_list Usage

The application uses `uniform_list` for virtualized list rendering across 7 distinct list contexts:

| Component | Handle Field | Item Height | Location |
|-----------|--------------|-------------|----------|
| Script List | `list_scroll_handle` | 40px (LIST_ITEM_HEIGHT) | main.rs:4472 |
| Arg Prompt Choices | `arg_list_scroll_handle` | 40px | main.rs:5214 |
| Clipboard History | `clipboard_list_scroll_handle` | 40px | main.rs:5850 |
| Window Switcher | `window_list_scroll_handle` | 40px | main.rs:6412 |
| Design Gallery | `design_gallery_scroll_handle` | 40px | main.rs:6726 |
| Actions Dialog | `scroll_handle` (in ActionsDialog) | 42px (ACTION_ITEM_HEIGHT) | actions.rs:662 |
| Editor Lines | `scroll_handle` (in EditorPrompt) | dynamic | editor.rs:1113 |

**Pattern used (main.rs:4472-4564):**
```rust
uniform_list(
    "script-list",
    item_count,
    cx.processor(move |this, visible_range, _window, cx| {
        // Render only visible items
        for ix in visible_range.clone() { ... }
    }),
)
.h_full()
.track_scroll(&self.list_scroll_handle)
```

### 1.2 Item Height Constants

The codebase has **inconsistent item height definitions** across designs:

| Constant | Value | Used By |
|----------|-------|---------|
| `LIST_ITEM_HEIGHT` | 40px | Default design, most components |
| `SECTION_HEADER_HEIGHT` | 24px | Section headers in grouped lists |
| `ACTION_ITEM_HEIGHT` | 42px | Actions popup |
| `MINIMAL_ITEM_HEIGHT` | 64px | Minimal design |
| `TERMINAL_ITEM_HEIGHT` | 28px | Retro Terminal design |
| `COMPACT_ITEM_HEIGHT` | 24px | Compact design |
| `PLAYFUL_ITEM_HEIGHT` | 64px | Playful design |
| `APPLE_HIG_ITEM_HEIGHT` | 44px | Apple HIG design |

**Issue:** The comment in list_item.rs (line 617) references 52px but the actual constant is 40px:
```rust
// The LIST_ITEM_HEIGHT constant is 52.0 and the component is integration-tested
// ^ This comment is outdated - actual value is 40.0
pub const LIST_ITEM_HEIGHT: f32 = 40.0;
```

### 1.3 Design Token System

The `get_item_height()` function (designs/mod.rs:288-291) provides dynamic item height lookup:
```rust
pub fn get_item_height(variant: DesignVariant) -> f32 {
    get_tokens(variant).item_height()
}
```

**Recommendation:** Use this function consistently instead of hardcoded heights to ensure design-aware virtualization.

---

## 2. Scroll Handle Assessment

### 2.1 Handle Inventory

The application maintains **5 scroll handles** in the main `ScriptListApp` struct plus 2 in sub-components:

```rust
// main.rs:757-765
list_scroll_handle: UniformListScrollHandle,
arg_list_scroll_handle: UniformListScrollHandle,
clipboard_list_scroll_handle: UniformListScrollHandle,
window_list_scroll_handle: UniformListScrollHandle,
design_gallery_scroll_handle: UniformListScrollHandle,

// actions.rs:179
pub scroll_handle: UniformListScrollHandle,

// editor.rs:121
scroll_handle: UniformListScrollHandle,
```

### 2.2 Overhead Analysis

**Memory Overhead per Handle:**
- `UniformListScrollHandle` is a lightweight wrapper around scroll state
- Each handle maintains: current offset, pending scroll operations
- Estimated: ~24-48 bytes per handle (minimal)

**Runtime Overhead:**
- Handles only consume CPU when their associated list is visible
- No polling - event-driven via `.track_scroll()`
- **Verdict:** Handle count is not a performance concern

### 2.3 Handle Lifecycle

Handles are initialized once at app creation (main.rs:931-935) and reused:
```rust
list_scroll_handle: UniformListScrollHandle::new(),
arg_list_scroll_handle: UniformListScrollHandle::new(),
// etc.
```

**Issue:** No handle cleanup or reset when switching views. The `last_scrolled_index` field (line 780) attempts to prevent redundant scrolls but only covers the main list.

---

## 3. Scroll Stabilization

### 3.1 last_scrolled_index Pattern

The codebase implements scroll stabilization to prevent jitter from redundant `scroll_to_item` calls:

```rust
// main.rs:1252-1265
fn scroll_to_selected_if_needed(&mut self, _reason: &str) {
    let target = self.selected_index;
    
    // Check if we've already scrolled to this index
    if self.last_scrolled_index == Some(target) {
        return;
    }
    
    self.list_scroll_handle.scroll_to_item(target, ScrollStrategy::Nearest);
    self.last_scrolled_index = Some(target);
}
```

**Coverage:**
- Main script list: Yes (via `scroll_to_selected_if_needed`)
- Arg prompt: No (direct `scroll_to_item` calls at lines 5126, 5136)
- Clipboard history: No (direct calls at lines 5749, 5757)
- Window switcher: No (direct calls at lines 6614, 6621)
- Design gallery: No (direct calls at lines 7183, 7190)
- Actions dialog: No (direct calls at lines 345, 355)

**Recommendation:** Extract `scroll_to_selected_if_needed` pattern into a reusable helper and apply to all list components.

### 3.2 Scroll Reset Points

Scroll position resets to top on:
- Filter text change (main.rs:1358, 1364, 1370)
- New arg prompt (main.rs:3375, 3381)
- Filter changes in actions (actions.rs:321)

---

## 4. Event Coalescing

### 4.1 Documented 20ms Window

AGENTS.md documents a 20ms coalescing window for keyboard events:
> Implement a 20ms coalescing window for rapid key events

**However, this is NOT implemented in the current codebase.**

The `perf.rs` module provides timing infrastructure but no actual coalescing logic:
```rust
// perf.rs:36-47 - Tracking only, no coalescing
pub struct KeyEventTracker {
    event_times: VecDeque<Instant>,
    processing_durations: VecDeque<Duration>,
    // No coalescing fields
}
```

### 4.2 Logging Infrastructure

The logging module has placeholders for coalescing metrics (logging.rs:1001-1025):
```rust
pub fn log_scroll_batch(batch_size: usize, coalesced_from: usize) {
    // Logs coalescing events but no actual coalescing logic
}

pub fn log_key_repeat_timing(key: &str, interval_ms: u64, repeat_count: u32) {
    let is_fast = interval_ms < 50; // 50ms threshold
}
```

### 4.3 Current Key Handling

Arrow key events are processed synchronously without debouncing:
```rust
// main.rs keyboard handler pattern
match key.as_str() {
    "up" | "arrowup" => this.move_selection_up(cx),
    "down" | "arrowdown" => this.move_selection_down(cx),
    // Immediate processing, no coalescing
}
```

**Impact:** Rapid key repeat could cause:
- Multiple `scroll_to_item` calls in quick succession
- Unnecessary re-renders via `cx.notify()`
- Frame drops if render exceeds 16.67ms budget

---

## 5. Performance Thresholds

### 5.1 Defined Thresholds

From `perf.rs`:

| Metric | Threshold | Purpose |
|--------|-----------|---------|
| SLOW_KEY_THRESHOLD_US | 16,666 us (16.67ms) | 60fps frame budget |
| SLOW_SCROLL_THRESHOLD_US | 8,000 us (8ms) | Scroll operation budget |
| MAX_SAMPLES | 100 | Rolling average window |

### 5.2 Scroll Timer Usage

The `ScrollTimer` (perf.rs:156-237) is available but **not actively used** in the scroll code paths:

```rust
// Available API
pub fn start_scroll() -> Instant { ... }
pub fn end_scroll() -> Duration { ... }

// NOT called in main.rs scroll handling
```

### 5.3 TimingGuard Pattern

A RAII timing guard exists (perf.rs:444-486):
```rust
pub struct TimingGuard {
    operation: &'static str,
    start: Instant,
    threshold_us: u128,
}

// Available but not used for scroll operations
let _guard = TimingGuard::scroll();
```

---

## 6. Scrollbar Implementation

### 6.1 Scrollbar Component

Located in `src/components/scrollbar.rs`:
- Semi-transparent overlay design
- Fade-out behavior (1000ms inactivity)
- Theme-aware colors

### 6.2 Fade-Out Mechanism

```rust
// main.rs:1273-1294
fn trigger_scroll_activity(&mut self, cx: &mut Context<Self>) {
    self.is_scrolling = true;
    self.last_scroll_time = Some(std::time::Instant::now());
    
    // Schedule fade-out after 1000ms of inactivity
    cx.spawn(async move |this, cx| {
        Timer::after(Duration::from_millis(1000)).await;
        // Check if still inactive, then hide
    }).detach();
}
```

**Issue:** Each scroll action spawns a new task. Rapid scrolling creates many concurrent timer tasks, though only the latest check matters.

---

## 7. Identified Issues

### 7.1 Critical

1. **No event coalescing implemented** despite AGENTS.md documentation
   - Risk: Frame drops during rapid scrolling
   - Impact: High on slow systems or long lists

2. **Scroll stabilization only on main list**
   - 6 other list components lack jitter prevention
   - Risk: Visual jitter on arg/clipboard/window lists

### 7.2 Moderate

3. **Outdated comment in list_item.rs** (line 617)
   - Claims 52px height, actual is 40px
   - Risk: Developer confusion

4. **Timer task accumulation** in scrollbar fade-out
   - Creates new task per scroll action
   - Risk: Minor memory churn during rapid scroll

5. **Performance instrumentation not connected**
   - `ScrollTimer`, `TimingGuard` exist but unused
   - Risk: Blind spots in performance monitoring

### 7.3 Low

6. **Item height inconsistency across designs**
   - Different heights (24px-64px) for different designs
   - Managed via `get_item_height()` but some direct usage remains

---

## 8. Optimization Recommendations

### 8.1 Implement Event Coalescing (Priority: High)

Add 20ms coalescing window for keyboard navigation:

```rust
// Suggested addition to ScriptListApp
struct ScrollCoalescer {
    pending_direction: Option<ScrollDirection>,
    pending_delta: i32,
    last_event: Instant,
}

impl ScrollCoalescer {
    const WINDOW_MS: u64 = 20;
    
    fn process(&mut self, direction: ScrollDirection) -> Option<i32> {
        let now = Instant::now();
        if now.duration_since(self.last_event) < Duration::from_millis(Self::WINDOW_MS)
           && self.pending_direction == Some(direction) {
            self.pending_delta += 1;
            None // Coalesce
        } else {
            let result = self.pending_delta.take();
            self.pending_direction = Some(direction);
            self.pending_delta = 1;
            self.last_event = now;
            result
        }
    }
}
```

### 8.2 Extract Scroll Stabilization Helper (Priority: High)

```rust
// New utility function
fn scroll_if_needed(
    handle: &UniformListScrollHandle,
    last_index: &mut Option<usize>,
    target: usize,
) {
    if *last_index != Some(target) {
        handle.scroll_to_item(target, ScrollStrategy::Nearest);
        *last_index = Some(target);
    }
}

// Per-list tracking
struct ListScrollState {
    handle: UniformListScrollHandle,
    last_index: Option<usize>,
}
```

### 8.3 Connect Performance Instrumentation (Priority: Medium)

Wrap scroll operations with timing:
```rust
fn scroll_to_selected_if_needed(&mut self, reason: &str) {
    let _guard = perf::TimingGuard::scroll();
    // existing logic
}
```

### 8.4 Optimize Scrollbar Timer (Priority: Low)

Replace per-scroll task with debounced timer:
```rust
// Instead of spawning new task each scroll:
if self.fade_timer.is_none() {
    self.fade_timer = Some(cx.spawn(async move |...| { ... }));
}
// Reset timer on each scroll activity
```

### 8.5 Fix Documentation (Priority: Low)

Update list_item.rs line 617 comment:
```rust
// The LIST_ITEM_HEIGHT constant is 40.0 (not 52.0)
```

---

## 9. Performance Metrics Summary

### Current State

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| P95 Key Latency | < 50ms | Unknown | Not measured |
| Single Key Event | < 16.67ms | Unknown | Not measured |
| Scroll Operation | < 8ms | Unknown | Not measured |
| Event Coalescing | 20ms window | Not implemented | Missing |
| Scroll Jitter Prevention | All lists | Main list only | Partial |

### Recommended Instrumentation

1. Enable `perf::start_scroll()` / `end_scroll()` in navigation code
2. Add `log_key_repeat_timing()` calls to detect fast repeats
3. Log coalescing effectiveness with `log_scroll_batch()`

---

## 10. Files Analyzed

| File | Lines | Key Patterns |
|------|-------|--------------|
| src/main.rs | 7500+ | uniform_list usage, scroll handles, keyboard handling |
| src/list_item.rs | 619 | LIST_ITEM_HEIGHT constant, ListItem component |
| src/perf.rs | 548 | ScrollTimer, KeyEventTracker, TimingGuard |
| src/components/scrollbar.rs | 353 | Scrollbar component, fade-out logic |
| src/actions.rs | 1074 | ActionsDialog scroll handling |
| src/logging.rs | 1070+ | Scroll logging infrastructure |
| src/designs/mod.rs | 540+ | get_item_height(), design tokens |

---

## Appendix: scroll_to_item Call Sites

| Location | Component | Has Stabilization |
|----------|-----------|-------------------|
| main.rs:1050 | Script list init | No |
| main.rs:1263 | Script list navigation | Yes |
| main.rs:1358 | Script list filter reset | No |
| main.rs:3375 | Arg prompt init | No |
| main.rs:5126 | Arg prompt up | No |
| main.rs:5136 | Arg prompt down | No |
| main.rs:5749 | Clipboard up | No |
| main.rs:5757 | Clipboard down | No |
| main.rs:6614 | Window switcher up | No |
| main.rs:6621 | Window switcher down | No |
| main.rs:7183 | Design gallery up | No |
| main.rs:7190 | Design gallery down | No |
| actions.rs:321 | Actions filter reset | No |
| actions.rs:345 | Actions up | No |
| actions.rs:355 | Actions down | No |

**Total: 15 call sites, 1 with stabilization (6.7%)**
