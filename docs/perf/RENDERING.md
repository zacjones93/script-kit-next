# Rendering & Frame Performance Audit

**Date:** December 2024  
**Auditor:** RenderingAuditor (Swarm Agent)  
**Scope:** READ-ONLY analysis of rendering performance patterns

---

## Executive Summary

The Script Kit GPUI application has **well-implemented virtualization** for lists but suffers from **excessive cx.notify() calls** (78 total) and **render method complexity** that may cause unnecessary re-renders. The terminal prompt shows excellent optimization patterns (timer-based refresh) that should be applied more broadly.

### Quick Stats

| Metric | Count | Assessment |
|--------|-------|------------|
| `cx.notify()` calls | 78 | High - potential over-rendering |
| `uniform_list` uses | 19 | Good - proper virtualization |
| `render` methods | 100 | High - many per design variant |
| `.when()` patterns | 37 | Good - conditional rendering |
| `font_family()` calls | 91 | Medium - font switching overhead |
| Frame drop threshold | 32ms | Appropriate (30fps baseline) |

---

## 1. Current State Analysis

### 1.1 Virtualized List Rendering (GOOD)

**Location:** `src/main.rs:4472-4564`

The application correctly uses `uniform_list` for all scrollable content:

```rust
// src/main.rs:4472-4474
uniform_list(
    "script-list",
    item_count,
    cx.processor(move |this, visible_range, _window, cx| { ... })
)
```

**Virtualized components:**
- Script list (`src/main.rs:4472`)
- Arg prompt choices (`src/prompts.rs`)
- Clipboard history (`src/main.rs`)
- App launcher (`src/main.rs`)
- Window switcher (`src/main.rs`)
- Editor line rendering (`src/editor.rs`)

**Assessment:** This is correctly implemented with fixed 52px item height (`LIST_ITEM_HEIGHT`).

### 1.2 cx.notify() Call Distribution

| File | Count | Concern Level |
|------|-------|---------------|
| `src/main.rs` | 63 | **HIGH** - many in rapid event handlers |
| `src/actions.rs` | 4 | Low |
| `src/prompts.rs` | 4 | Low |
| `src/term_prompt.rs` | 3 | **OPTIMIZED** - uses timer-based refresh |
| `src/editor.rs` | 1 | Low |

### 1.3 Render Method Complexity

**Main render method:** `src/main.rs:3535-3584` (50 lines)
- Dispatches to 12 different view renderers
- Focus handling inline (should be extracted)

**render_script_list:** `src/main.rs:4362-4578` (~216 lines)
- Complex grouped items logic
- Inline color computation
- Scrollbar calculation on every render

**Design variant proliferation:**
- 10+ design variants, each with separate render implementations
- Each design has its own color/spacing/typography computation

---

## 2. Identified Bottlenecks

### P0 - Critical (Fix Immediately)

#### 2.1 Hover Handler cx.notify() in Hot Loop

**Location:** `src/main.rs:4507-4521`

```rust
let hover_handler = cx.listener(move |this, hovered: &bool, _window, cx| {
    if *hovered {
        if this.hovered_index != Some(ix) {
            this.hovered_index = Some(ix);
            cx.notify();  // TRIGGERS FULL RE-RENDER
        }
    } else {
        if this.hovered_index == Some(ix) {
            this.hovered_index = None;
            cx.notify();  // TRIGGERS FULL RE-RENDER
        }
    }
});
```

**Problem:** Every mouse hover enter/leave triggers a full window re-render. When moving mouse quickly over list items, this can cause dozens of re-renders per second.

**Impact:** High - directly affects perceived smoothness when browsing list with mouse.

#### 2.2 Click Handler cx.notify() Redundancy

**Location:** `src/main.rs:4526-4531`

```rust
let click_handler = cx.listener(move |this, _event, _window, cx| {
    if this.selected_index != ix {
        this.selected_index = ix;
        cx.notify();  // TRIGGERS FULL RE-RENDER
    }
});
```

**Problem:** Click handlers are created for every visible item on every render, each with closure capturing `ix`.

**Impact:** Memory allocation overhead per render cycle.

---

### P1 - High Priority

#### 2.3 ListItemColors Recomputation

**Location:** `src/main.rs:4376-4377, 4484`

```rust
// Computed once at render_script_list start
let _list_colors = ListItemColors::from_theme(theme);  // Line 4376

// Then RECOMPUTED inside uniform_list closure
let colors = ListItemColors::from_theme(&this.theme);  // Line 4484
```

**Problem:** `ListItemColors::from_theme()` is called once per render cycle AND once per visible range render. The closure should capture the pre-computed colors.

**Impact:** Medium - redundant computation on every scroll/render.

#### 2.4 Design Token Computation Per Render

**Location:** `src/main.rs:4364-4368`

```rust
fn render_script_list(&mut self, cx: &mut Context<Self>) -> AnyElement {
    let tokens = get_tokens(self.current_design);
    let design_colors = tokens.colors();
    let design_spacing = tokens.spacing();
    let design_visual = tokens.visual();
    let design_typography = tokens.typography();
    // ...
}
```

**Problem:** Design tokens are computed on every render call, but `current_design` rarely changes.

**Recommendation:** Cache design tokens in `ScriptListApp` state, only recompute when `current_design` changes.

#### 2.5 Grouped Items Clone Per Render

**Location:** `src/main.rs:4439-4440`

```rust
let grouped_items_clone = grouped_items.clone();
let flat_results_clone = flat_results.clone();
```

**Problem:** Large vectors are cloned on every render to move into the uniform_list closure.

**Impact:** Memory allocation proportional to list size on every render.

---

### P2 - Medium Priority

#### 2.6 Font Family String Allocation

**Location:** Multiple files (91 occurrences)

```rust
.font_family(".AppleSystemUIFont")  // String allocation
.font_family("Menlo")               // String allocation
.font_family("SF Mono")             // String allocation
```

**Problem:** Font family strings are allocated on every render. Should use `const` or `static` references.

**Recommendation:**
```rust
const SYSTEM_FONT: &str = ".AppleSystemUIFont";
const MONO_FONT: &str = "Menlo";
// Use: .font_family(SYSTEM_FONT)
```

#### 2.7 Scrollbar Computation Every Render

**Location:** `src/main.rs:4443-4470`

```rust
// These calculations happen every render
let estimated_container_height = 400.0_f32;
let visible_items = (estimated_container_height / LIST_ITEM_HEIGHT) as usize;
let scroll_offset = if self.selected_index > visible_items.saturating_sub(1) { ... };
let scrollbar = Scrollbar::new(item_count, visible_items, scroll_offset, scrollbar_colors);
```

**Problem:** Scrollbar parameters are recalculated every render even when nothing changed.

**Recommendation:** Cache scrollbar state, only recalculate when `item_count`, `selected_index`, or `is_scrolling` changes.

#### 2.8 Toast Manager Tick in Render

**Location:** `src/main.rs:3621-3630`

```rust
fn render_toasts(&mut self, _cx: &mut Context<Self>) -> Option<impl IntoElement> {
    self.toast_manager.tick();       // Side effect in render
    self.toast_manager.cleanup();    // Side effect in render
    let _ = self.toast_manager.take_needs_notify();
    // ...
}
```

**Problem:** Toast tick/cleanup happens during render, not in a separate timer.

**Recommendation:** Use timer-based toast management like `term_prompt.rs` does.

---

### P3 - Low Priority (Nice to Have)

#### 2.9 Element Tree Depth

**Typical nesting depth in list items:** 5-7 levels

```rust
// src/list_item.rs:362-396
div()  // Container (1)
    .child(
        div()  // Content wrapper (2)
            .child(
                div()  // Name container (3)
                    .child(self.name)  // Text (4)
            )
            .child(
                div()  // Description container (3)
                    .child(desc)  // Text (4)
            )
    )
```

**Assessment:** Acceptable depth. GPUI handles this efficiently.

#### 2.10 Icon Rendering Fallback Chain

**Location:** `src/list_item.rs:288-358`

```rust
match &self.icon {
    Some(IconKind::Emoji(emoji)) => { ... }
    Some(IconKind::Image(render_image)) => { ... }
    Some(IconKind::Svg(name)) => {
        if let Some(icon_name) = icon_name_from_str(name) {
            // Primary SVG path
        } else {
            // Fallback to Code icon
        }
    }
    None => { ... }
}
```

**Assessment:** Well-structured with pre-decoded images. The comment at line 302 confirms optimization:
```rust
// Render pre-decoded image directly (no decoding on render - critical for perf)
```

---

## 3. Exemplary Patterns (Keep These)

### 3.1 Terminal Prompt Timer-Based Refresh

**Location:** `src/term_prompt.rs:504-554`

```rust
// No cx.notify() needed - timer handles refresh at 30fps
```

The terminal prompt uses a dedicated timer for refresh instead of triggering re-renders on every PTY output. This is the correct pattern.

### 3.2 Pre-Decoded Image Caching

**Location:** `src/list_item.rs:301-318`

```rust
Some(IconKind::Image(render_image)) => {
    // Render pre-decoded image directly (no decoding on render - critical for perf)
    let image = render_image.clone();
    div()
        .child(
            img(move |_window, _cx| Some(Ok(image.clone())))
        )
}
```

### 3.3 Frame Timing Infrastructure

**Location:** `src/perf.rs:277`

```rust
// Consider frame dropped if > 32ms (less than 30fps)
if d.as_millis() > 32 {
    self.dropped_frames += 1;
}
```

Good baseline metric tracking already exists.

### 3.4 ListItemColors Copy Pattern

**Location:** `src/list_item.rs` and `src/theme.rs`

The `ListItemColors` struct is designed to be `Copy` for efficient use in closures. This pattern should be extended.

---

## 4. Recommended Optimizations

### Priority P0 (Do First)

| # | Optimization | Location | Estimated Impact |
|---|-------------|----------|------------------|
| 1 | Debounce hover state changes | `main.rs:4507-4521` | 50% reduction in hover re-renders |
| 2 | Batch hover/click handler creation | `main.rs:4507-4531` | Reduce closure allocations |

**Implementation for #1:**
```rust
// Add to ScriptListApp
last_hover_notify: Option<Instant>,

// In hover handler
let now = Instant::now();
if now.duration_since(this.last_hover_notify.unwrap_or(now)) > Duration::from_millis(16) {
    this.hovered_index = Some(ix);
    this.last_hover_notify = Some(now);
    cx.notify();
}
```

### Priority P1

| # | Optimization | Location | Estimated Impact |
|---|-------------|----------|------------------|
| 3 | Cache design tokens in state | `main.rs:4364-4368` | Avoid per-render computation |
| 4 | Use Arc for grouped_items | `main.rs:4439-4440` | Eliminate clone per render |
| 5 | Pre-capture ListItemColors in closure | `main.rs:4484` | Avoid redundant computation |

**Implementation for #4:**
```rust
// Instead of cloning vectors
let grouped_items = Arc::new(grouped_items);
let grouped_items_ref = Arc::clone(&grouped_items);
```

### Priority P2

| # | Optimization | Location | Estimated Impact |
|---|-------------|----------|------------------|
| 6 | Use const for font family strings | Multiple files | Minor memory reduction |
| 7 | Cache scrollbar state | `main.rs:4443-4470` | Avoid unnecessary computation |
| 8 | Move toast tick to timer | `main.rs:3621` | Consistent with term_prompt pattern |

### Priority P3

| # | Optimization | Location | Estimated Impact |
|---|-------------|----------|------------------|
| 9 | Profile and flatten deep element trees | Various designs | Minor - already acceptable |
| 10 | Consider memo-izing design variant renders | `src/designs/` | Reduce design-switch overhead |

---

## 5. Metrics to Track

### Current Baseline (from perf.rs)

| Metric | Threshold | Source |
|--------|-----------|--------|
| Frame drop | >32ms | `perf.rs:277` |
| Slow key event | >16.67ms | `perf.rs:26` |
| Slow scroll | >8ms | `perf.rs:29` |

### Recommended Additional Metrics

1. **Hover events per second** - Track rapid hover state changes
2. **cx.notify() calls per second** - Detect over-rendering
3. **Render method duration** - Per-view timing
4. **Memory allocation per render** - Vector/string allocations

### Measurement Points

```rust
// Add timing around key render methods
let start = Instant::now();
let result = self.render_script_list(cx);
let duration = start.elapsed();
if duration.as_millis() > 16 {
    warn!(duration_ms = duration.as_millis(), "Slow render_script_list");
}
```

---

## 6. Architecture Observations

### Strengths

1. **Proper virtualization** - `uniform_list` used consistently
2. **Theme system** - Colors centralized, no hardcoded values
3. **Design token pattern** - Consistent spacing/typography
4. **Pre-decoded images** - Correct caching for app icons
5. **Existing perf infrastructure** - `perf.rs` provides good foundation

### Areas for Improvement

1. **State management** - Too many cx.notify() trigger points
2. **Render method size** - `render_script_list` is 200+ lines
3. **Design variant proliferation** - 10+ designs with similar code
4. **Closure allocation** - Event handlers created per-item per-render

### Recommended Refactoring

1. **Extract focus handling** from main render method
2. **Create shared render helpers** for common patterns across designs
3. **Implement render caching** for stable views (e.g., empty states)
4. **Consider component-level memoization** for expensive subtrees

---

## Appendix A: File Locations Reference

| Concern | Primary Location |
|---------|------------------|
| Main render dispatch | `src/main.rs:3535-3584` |
| Script list rendering | `src/main.rs:4362-4578` |
| List item component | `src/list_item.rs:273-520` |
| Hover/click handlers | `src/main.rs:4507-4531` |
| Design tokens | `src/designs/*.rs` |
| Performance tracking | `src/perf.rs` |
| Terminal optimization | `src/term_prompt.rs:504-554` |
| Toast rendering | `src/main.rs:3620-3668` |

## Appendix B: cx.notify() Hotspots

**High-frequency paths (most concerning):**

1. `main.rs:4512` - Hover enter (in uniform_list loop)
2. `main.rs:4518` - Hover leave (in uniform_list loop)
3. `main.rs:4529` - Click handler (in uniform_list loop)
4. `main.rs:4703` - Keyboard navigation
5. `main.rs:2507-2796` - Stdin message handlers

**Well-optimized paths:**
1. `term_prompt.rs` - Uses timer, avoids cx.notify() spam
2. `editor.rs:755` - Single notify point

---

*End of Rendering Performance Audit*
