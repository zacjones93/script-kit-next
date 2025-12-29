# Animation & Feedback Patterns Audit

**Audit Date:** 2024-12-29  
**Auditor:** animation-audit-worker  
**Cell ID:** cell--9bnr5-mjqv8x90cw7  
**Epic:** cell--9bnr5-mjqv8x8f0my

## Executive Summary

Script Kit GPUI implements a **state-based animation model** rather than CSS transitions. GPUI does not support CSS-like transitions or keyframe animations natively—instead, visual feedback relies on:

1. **Opacity changes** for visibility states
2. **Hover pseudo-states** for interactive elements
3. **Timer-based updates** for temporal effects
4. **State-driven re-renders** via `cx.notify()`

### Key Finding: Animation Token Infrastructure Exists But Is Unused

The codebase defines animation duration tokens in `DesignVisual`:
- `animation_fast: 100ms`
- `animation_normal: 200ms`  
- `animation_slow: 300ms`

However, **these tokens are never consumed** because GPUI lacks native transition support.

---

## Table of Contents

1. [Animation Infrastructure](#1-animation-infrastructure)
2. [Loading States & Spinners](#2-loading-states--spinners)
3. [Toast Notification System](#3-toast-notification-system)
4. [Selection Change Feedback](#4-selection-change-feedback)
5. [Window Show/Hide Transitions](#5-window-showhide-transitions)
6. [Scroll Animations](#6-scroll-animations)
7. [Hover State Transitions](#7-hover-state-transitions)
8. [Response Time Feedback](#8-response-time-feedback)
9. [Recommendations](#9-recommendations)

---

## 1. Animation Infrastructure

### 1.1 Animation Token Definitions

**Location:** `src/designs/traits.rs:338-378`

```rust
pub struct DesignVisual {
    // Animation durations (ms)
    /// Fast animation (100ms)
    pub animation_fast: u32,
    /// Normal animation (200ms)
    pub animation_normal: u32,
    /// Slow animation (300ms)
    pub animation_slow: u32,
}
```

### 1.2 Design-Specific Animation Values

| Design Variant | Fast | Normal | Slow | Notes |
|----------------|------|--------|------|-------|
| **Default** | 100ms | 200ms | 300ms | Standard timing |
| **Minimal** | 150ms | 250ms | 350ms | Slightly slower |
| **Brutalist** | 0ms | 0ms | 100ms | Minimal animations for terminal feel |
| **Apple HIG** | 150ms | 300ms | 500ms | Apple-style springy |
| **Material3** | 0ms | 0ms | 0ms | No animations defined |
| **Compact** | 50ms | 100ms | 150ms | Snappy, quick |
| **Glassmorphism** | 100ms | 200ms | 300ms | Standard |
| **Neon Cyberpunk** | 150ms | 250ms | 400ms | Slower for dramatic effect |
| **Paper** | 150ms | 250ms | 350ms | Slightly organic |
| **Playful** | 150ms | 300ms | 450ms | Bouncy animations |

### 1.3 Gap Analysis

**Issue:** Animation tokens exist but GPUI provides no native transition/animation API.

**Workarounds in codebase:**
- Manual timer-based updates (see [Section 6](#6-scroll-animations))
- State-based re-renders via `cx.notify()`
- Opacity values applied directly (no interpolation)

---

## 2. Loading States & Spinners

### 2.1 Current Implementation

**Status:** ❌ **No loading indicators exist**

The codebase has no spinner, progress bar, or skeleton loading components.

### 2.2 Loading Text Occurrences

| Location | Usage | Context |
|----------|-------|---------|
| `src/main.rs:6234` | `"Loading image..."` | Static text during image preview |
| `src/main.rs:801-869` | Log messages only | Script loading timing |

### 2.3 Background Loading Patterns

**Location:** `src/main.rs:836-869`

```rust
logging::log("APP", "Applications loading in background...");
// ... async app loading ...
logging::log("APP", "Background app loading complete: {} apps in {:.2}ms");
```

**Issue:** Background app loading shows no visual feedback to users—only log messages.

### 2.4 Recommendations

1. **Add loading spinner component** for:
   - App icon loading
   - Script execution initialization
   - Preview image fetching

2. **Add skeleton loading** for list items during initial load

---

## 3. Toast Notification System

### 3.1 Architecture

**Location:** `src/components/toast.rs` + `src/toast_manager.rs`

The toast system is well-implemented with:

| Feature | Status | Details |
|---------|--------|---------|
| Variants | ✅ | Success, Warning, Error, Info |
| Auto-dismiss | ✅ | 5000ms default, configurable |
| Dismiss button | ✅ | "×" button with hover state |
| Action buttons | ✅ | E.g., "Copy Error", "View Details" |
| Stacking | ✅ | Max 5 visible, top-right position |
| Icons | ✅ | ✓, ⚠, ✕, ℹ per variant |

### 3.2 Toast Lifecycle

```
ToastManager::push(Toast)
      │
      ▼
ToastNotification created (id, created_at, duration_ms)
      │
      ▼
render_toasts() → visible_toasts() (max 5)
      │
      ▼
tick() checks should_auto_dismiss()
      │
      ▼
cleanup() removes dismissed toasts
```

### 3.3 Toast Animation Gap

**Issue:** Toasts appear/disappear **instantly** with no entrance/exit animation.

**Current behavior:**
```rust
// src/components/toast.rs:314
.bg(rgba((colors.background << 8) | 0xF0)) // 94% opacity - no transition
```

**Missing animations:**
- Slide-in from right
- Fade-out on dismiss
- Stack reflow when toast removed

### 3.4 Toast Color System

| Variant | Icon Color | Border Color |
|---------|-----------|--------------|
| Success | `ui.success` | `ui.success` |
| Warning | `ui.warning` | `ui.warning` |
| Error | `ui.error` | `ui.error` |
| Info | `ui.info` | `ui.info` |

---

## 4. Selection Change Feedback

### 4.1 List Item Selection

**Location:** `src/list_item.rs:262-270`

```rust
/// Hovered items show a subtle background tint (25% opacity).
/// Selected items show full focus styling (50% opacity background + accent bar).
pub fn hovered(mut self, hovered: bool) -> Self {
    self.hovered = hovered;
    self
}
```

### 4.2 Selection Visual Hierarchy

| State | Background | Accent Bar | Text Color |
|-------|------------|------------|------------|
| Normal | Transparent | None | `text.secondary` |
| Hovered | 25% opacity tint | None | `text.secondary` |
| Selected | 50% opacity tint | 3px left bar | `text.primary` |

### 4.3 Selection Colors

**Location:** `src/list_item.rs:280-282`

```rust
let selected_bg = rgba((colors.accent_selected_subtle << 8) | 0x80); // 50% opacity
let hover_bg = rgba((colors.accent_selected_subtle << 8) | 0x40);    // 25% opacity
```

### 4.4 Selection Animation Gap

**Issue:** Selection changes are **instant** with no transition.

When user presses arrow keys rapidly, selection jumps without visual continuity.

**Recommendation:** Consider a brief highlight flash on selection change.

---

## 5. Window Show/Hide Transitions

### 5.1 Window Show Sequence

**Location:** `src/main.rs:647-685`

```rust
// Step 1: Position window BEFORE activating
self.move_first_window_to_bounds(cx, window);

// Step 2: Apply floating panel configuration
configure_as_floating_panel();

// Step 3: NOW activate the app
cx.activate(true);
window.activate_window();
```

### 5.2 Window Resize Behavior

**Location:** `src/window_resize.rs:109-216`

```rust
// macOS native call - animate:false for instant resize
let _: () = msg_send![window, setFrame:new_frame display:true animate:false];
```

**Current behavior:** Window resizes are **instant** (`animate:false`).

### 5.3 Resize View Types

| View Type | Height | Animated |
|-----------|--------|----------|
| ScriptList | 500px (fixed) | No |
| ArgPromptWithChoices | 500px (fixed) | No |
| ArgPromptNoChoices | 120px (compact) | No |
| EditorPrompt | 700px (full) | No |
| TermPrompt | 700px (full) | No |
| DivPrompt | 500px | No |

### 5.4 Window Reset on Hide

**Location:** `src/main.rs:620-630`

```rust
// Reset UI state before hiding (clears selection, scroll position, filter)
view.reset_state(cx);
// ...
window.hide();
```

### 5.5 Animation Gap

**Issue:** Window show/hide is instant with no fade or slide effect.

**Platform limitation:** GPUI doesn't expose macOS window animation APIs directly. The `setFrame:display:animate:` is called with `animate:false` to avoid RefCell borrow conflicts during render cycle.

---

## 6. Scroll Animations

### 6.1 Scroll Implementation

**Location:** `src/main.rs:741-745, 1267-1293`

```rust
// Scroll activity tracking for scrollbar fade
is_scrolling: bool,
last_scroll_time: Option<std::time::Instant>,

fn trigger_scroll_activity(&mut self, cx: &mut Context<Self>) {
    self.is_scrolling = true;
    self.last_scroll_time = Some(std::time::Instant::now());
    
    // Schedule fade-out after 1 second of inactivity
    Timer::after(Duration::from_millis(1000)).await;
    if last_time.elapsed() >= Duration::from_millis(1000) {
        app.is_scrolling = false;
    }
}
```

### 6.2 Scrollbar Fade System

**Location:** `src/components/scrollbar.rs:229-238`

```rust
// Handle scroll-activity-aware visibility
let thumb_opacity = match self.is_visible {
    Some(false) => 0.0,      // Hidden after fade timeout
    _ => self.colors.thumb_opacity,  // 0.4 default
};
```

### 6.3 Scrollbar Opacity Values

| State | Track Opacity | Thumb Opacity | Thumb Hover Opacity |
|-------|---------------|---------------|---------------------|
| Inactive | 0.0 | 0.0 | 0.0 |
| Active | 0.1 | 0.4 | 0.6 |

### 6.4 Scroll Animation Gap

**Issue:** Scrollbar visibility changes are **instant** (binary on/off).

**Current:** `is_scrolling = false` → thumb_opacity immediately becomes 0.0

**Desired:** Gradual fade from 0.4 → 0.0 over ~200ms

### 6.5 Scroll Strategy

**Location:** `src/main.rs:1263`

```rust
self.list_scroll_handle.scroll_to_item(target, ScrollStrategy::Nearest);
```

Available strategies: `Top`, `Center`, `Nearest`

---

## 7. Hover State Transitions

### 7.1 Button Hover

**Location:** `src/components/button.rs:173-235`

```rust
// Hover uses white at ~15% alpha - universal "lift" effect
let hover_overlay = rgba(0xffffff26); // ~15% alpha

button = button.hover(move |s| s.bg(hover_bg));
```

### 7.2 List Item Hover

**Location:** `src/main.rs:4505-4518`

```rust
let hover_handler = cx.listener(move |this: &mut ScriptListApp, hovered: &bool, _window, cx| {
    if *hovered {
        this.hovered_index = Some(ix);
        cx.notify();
    } else {
        if this.hovered_index == Some(ix) {
            this.hovered_index = None;
        }
        cx.notify();
    }
});
```

### 7.3 Toast Action Hover

**Location:** `src/components/toast.rs:378`

```rust
.hover(|s| s.bg(rgba((colors.action_background << 8) | 0xC0)))
```

### 7.4 Scrollbar Hover

**Location:** `src/components/scrollbar.rs:276-280`

```rust
.hover(move |s| {
    s.bg(rgba(
        (colors.thumb_hover << 8) | ((thumb_hover_opacity * 255.0) as u32),
    ))
})
```

### 7.5 Hover Animation Gap

**Issue:** All hover effects are **instant** state changes.

GPUI's `.hover()` pseudo-state applies immediately with no transition.

---

## 8. Response Time Feedback

### 8.1 Cursor Blink Timer

**Location:** `src/main.rs:886-904`

```rust
// Cursor blink timer - 530ms interval
Timer::after(std::time::Duration::from_millis(530)).await;
app.cursor_visible = !app.cursor_visible;
cx.notify();
```

### 8.2 Debounce Patterns

| System | Debounce Time | Purpose |
|--------|---------------|---------|
| File watcher | 500ms | Prevent rapid reloads |
| Config reload | 200ms | Debounce config changes |
| Scripts reload | 200ms | Debounce script changes |
| Stdin polling | 100ms | Batch stdin messages |

### 8.3 Auto-Submit Delay

**Location:** `src/executor.rs:47-54`

```rust
pub fn get_auto_submit_delay() -> Duration {
    env::var("SCRIPT_KIT_AUTO_SUBMIT_DELAY")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map(Duration::from_millis)
        .unwrap_or(Duration::from_millis(100))
}
```

### 8.4 Terminal Refresh

**Location:** `src/term_prompt.rs:211-227`

```rust
const REFRESH_INTERVAL_MS: u64 = 33; // ~30fps

fn start_refresh_timer(&mut self, cx: &mut Context<Self>) {
    Timer::after(Duration::from_millis(REFRESH_INTERVAL_MS)).await;
    // ... refresh terminal output
}
```

### 8.5 Performance Timing

**Location:** `src/perf.rs:156-339`

The `ScrollTimer` and `FrameTimer` structs provide performance measurement but no user-facing feedback.

---

## 9. Recommendations

### 9.1 High Priority (UX Impact)

| Issue | Current State | Recommendation | Effort |
|-------|---------------|----------------|--------|
| **No loading indicators** | Silent background loading | Add spinner component | Medium |
| **Instant toast appear/disappear** | Binary visibility | Implement fade animation via timer | Medium |
| **Instant scrollbar fade** | Binary visibility | Timer-based opacity interpolation | Low |

### 9.2 Medium Priority (Polish)

| Issue | Current State | Recommendation | Effort |
|-------|---------------|----------------|--------|
| **No selection transition** | Instant state change | Flash/highlight on change | Low |
| **Instant window resize** | `animate:false` | Test `animate:true` on next frame | Low |
| **Unused animation tokens** | Values defined but unused | Document or remove | Low |

### 9.3 Low Priority (Nice-to-Have)

| Issue | Current State | Recommendation | Effort |
|-------|---------------|----------------|--------|
| **No hover transitions** | GPUI limitation | Accept limitation or JS layer | High |
| **No entrance animations** | Instant visibility | Consider for v2 | High |

### 9.4 Implementation Strategy for Animations

Since GPUI lacks native transition support, implement animations via:

```rust
// Timer-based animation pattern
struct AnimatedValue {
    current: f32,
    target: f32,
    duration_ms: u64,
    start_time: Instant,
}

impl AnimatedValue {
    fn tick(&mut self) -> f32 {
        let elapsed = self.start_time.elapsed().as_millis() as f32;
        let progress = (elapsed / self.duration_ms as f32).min(1.0);
        self.current = lerp(self.current, self.target, ease_out(progress));
        self.current
    }
}
```

### 9.5 Fade Animation Example for Scrollbar

```rust
// Instead of binary visibility:
let thumb_opacity = match self.is_visible {
    Some(false) => 0.0,
    _ => self.colors.thumb_opacity,
};

// Use interpolated value:
let thumb_opacity = self.scrollbar_opacity.tick(); // 0.4 → 0.0 over 200ms
```

---

## Appendix A: Animation-Related File Locations

| File | Animation-Related Content |
|------|---------------------------|
| `src/designs/traits.rs` | Animation token definitions |
| `src/components/toast.rs` | Toast rendering (no animation) |
| `src/toast_manager.rs` | Toast lifecycle, auto-dismiss timers |
| `src/components/scrollbar.rs` | Scrollbar opacity handling |
| `src/components/button.rs` | Button hover states |
| `src/list_item.rs` | List item hover/selection states |
| `src/window_resize.rs` | Window resize (instant) |
| `src/main.rs` | Scroll activity tracking, cursor blink |
| `src/perf.rs` | Performance timing infrastructure |

## Appendix B: Timer Intervals Summary

| System | Interval | Purpose |
|--------|----------|---------|
| Cursor blink | 530ms | Text input cursor visibility |
| Scroll fade | 1000ms | Scrollbar hide after inactivity |
| Terminal refresh | 33ms (~30fps) | PTY output polling |
| Toast auto-dismiss | 5000ms | Default toast lifetime |
| Debounce (file) | 500ms | Prevent rapid reloads |
| Debounce (config) | 200ms | Config change batching |
| Window resize defer | 16ms (~1 frame) | Avoid RefCell conflicts |

---

**End of Audit**
