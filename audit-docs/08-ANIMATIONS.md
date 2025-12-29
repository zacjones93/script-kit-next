# Animation and Interaction Patterns Audit

**Audit Date:** December 29, 2025  
**Auditor:** AnimationAuditor  
**Scope:** Script Kit GPUI animations, transitions, and interaction patterns

---

## Executive Summary

Script Kit GPUI uses a **minimal animation approach** focused on immediate feedback and performance. The codebase relies primarily on:
- **Timer-based delays** for async operations and auto-dismiss behaviors
- **Opacity changes** for visual state transitions
- **Immediate state updates** (no CSS-style transition curves)
- **Scroll position management** via GPUI's `UniformListScrollHandle`

**Key Finding:** GPUI does not expose CSS-like transition/animation APIs. All "animations" are achieved through:
1. Discrete state changes with `cx.notify()`
2. Timer-based scheduled updates
3. Opacity property modifications

---

## 1. Transitions

### 1.1 Current Implementation

**GPUI does not have transition APIs.** All visual changes are immediate upon `cx.notify()`.

#### Window Animations
```rust
// src/window_resize.rs - No animation, immediate resize
let _: () = msg_send![window, setFrame:new_frame display:true animate:false];
```

The `animate:false` parameter explicitly disables macOS window animation. Resizing is instantaneous.

#### State Transitions
State changes trigger immediate re-renders:
```rust
// src/prompts.rs
fn move_up(&mut self, cx: &mut Context<Self>) {
    if self.selected_index > 0 {
        self.selected_index -= 1;
        cx.notify();  // Immediate re-render
    }
}
```

### 1.2 Timer-Based Pseudo-Animations

Several behaviors use `Timer::after()` for delayed state changes:

| Feature | Duration | Purpose |
|---------|----------|---------|
| Cursor Blink | 530ms | Toggle visibility |
| Scrollbar Fade | 1000ms | Hide after inactivity |
| Toast Auto-Dismiss | 2000-10000ms | Remove notification |
| Error Notification | 5000ms | Auto-clear error |
| Config/Scripts Reload | 200ms | Debounce file watcher |

#### Example: Cursor Blink Animation
```rust
// src/main.rs - Cursor blink timer
cx.spawn(async move |this, cx| {
    loop {
        Timer::after(std::time::Duration::from_millis(530)).await;
        let _ = cx.update(|cx| {
            this.update(cx, |app, cx| {
                app.cursor_visible = !app.cursor_visible;
                cx.notify();
            })
        });
    }
}).detach();
```

### 1.3 Assessment

| Aspect | Status | Notes |
|--------|--------|-------|
| Transition consistency | N/A | No transition system |
| Duration standards | Varies | 200ms-10000ms depending on use case |
| Easing curves | None | GPUI doesn't support easing |
| Property animations | Opacity only | No transform, scale, or position animations |

**Recommendation:** Consider adding a consistent timer constant module for standardized durations.

---

## 2. Hover States

### 2.1 Implementation Pattern

Hover states are implemented via GPUI's `.hover()` modifier and explicit `on_hover` callbacks:

#### CSS-like Hover (Buttons, Toast Actions)
```rust
// src/components/button.rs
.hover(move |s| s.bg(hover_bg))

// src/components/toast.rs
.hover(|s| s.bg(rgba((colors.action_background << 8) | 0xC0)))
.hover(|s| s.underline())
```

#### Explicit Hover Tracking (List Items)
```rust
// src/list_item.rs
if let (Some(idx), Some(callback)) = (index, on_hover_callback) {
    container = container.on_hover(move |hovered: &bool, _window, _cx| {
        if *hovered {
            logging::log_mouse_enter(idx, None);
        } else {
            logging::log_mouse_leave(idx, None);
        }
        callback(idx, *hovered);
    });
}
```

### 2.2 Hover Visual Patterns

| Component | Hover Effect | Implementation |
|-----------|--------------|----------------|
| **ListItem** | 25% opacity background tint | `hover_bg = rgba((accent << 8) \| 0x40)` |
| **Button** | 15% white overlay | `.hover(\|s\| s.bg(hover_overlay))` |
| **Toast Action** | Background opacity increase | `.hover(\|s\| s.bg(rgba(...\|0xC0)))` |
| **Toast Dismiss** | Background + text color change | `.hover(\|s\| s.bg(rgba(0xffffff10)))` |
| **Scrollbar Thumb** | Increased opacity (0.4 -> 0.6) | Theme-aware opacity change |

### 2.3 Hover State Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     HOVER STATE TRACKING                         │
├─────────────────────────────────────────────────────────────────┤
│  hovered_index: Option<usize>  (subtle visual feedback - 25%)   │
│  selected_index: usize         (full focus styling - 50%)       │
│                                                                  │
│  Separation allows:                                              │
│  - Mouse hover for preview/highlight                            │
│  - Keyboard navigation maintains selection                       │
│  - Click commits hover to selection                              │
└─────────────────────────────────────────────────────────────────┘
```

### 2.4 Assessment

| Aspect | Status | Notes |
|--------|--------|-------|
| Hover consistency | Good | All interactive elements have hover states |
| Opacity levels | Standardized | 25% hover, 50% selected |
| Cursor styling | Present | `.cursor_pointer()` on interactive elements |
| Accessibility | Partial | No focus-visible indicators |

---

## 3. Selection States

### 3.1 Selection Visual Hierarchy

```
┌─────────────────────────────────────────────────────────────────┐
│  VISUAL STATE HIERARCHY (lightest to strongest)                  │
├─────────────────────────────────────────────────────────────────┤
│  1. Default:    Transparent background                           │
│  2. Hovered:    25% opacity accent_selected_subtle (0x40 alpha)  │
│  3. Selected:   50% opacity accent_selected_subtle (0x80 alpha)  │
│                 + Accent bar (3px left border, accent color)     │
│                 + Text color change (secondary -> primary)       │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 ListItem Selection Implementation

```rust
// src/list_item.rs
let selected_bg = rgba((colors.accent_selected_subtle << 8) | 0x80);  // 50% opacity
let hover_bg = rgba((colors.accent_selected_subtle << 8) | 0x40);     // 25% opacity

let bg_color = if self.selected {
    selected_bg  // Full focus styling
} else if self.hovered {
    hover_bg     // Subtle hover feedback
} else {
    rgba(0x00000000)  // Transparent
};

// Accent bar (3px left border)
let accent_bar = if self.show_accent_bar {
    div()
        .w(px(ACCENT_BAR_WIDTH))  // 3.0px
        .h_full()
        .bg(if self.selected { accent_color } else { rgba(0x00000000) })
} else { ... };
```

### 3.3 Multi-Select Support

**Current Status: NOT IMPLEMENTED**

The codebase uses single `selected_index: usize` throughout:
- `ScriptListApp.selected_index`
- `ArgPrompt.selected_index`
- `ClipboardHistoryView.selected_index`
- `AppLauncherView.selected_index`
- `WindowSwitcherView.selected_index`

No `Vec<usize>` or `HashSet<usize>` for multi-selection.

### 3.4 Assessment

| Aspect | Status | Notes |
|--------|--------|-------|
| Selection styling | Good | Clear visual hierarchy |
| Accent bar | Consistent | 3px left border on selected |
| Multi-select | Missing | Single selection only |
| Selection animation | None | Immediate state change |

---

## 4. Loading States

### 4.1 Current Implementation

**No explicit loading UI patterns exist.** The app uses:

#### Background Loading (Apps)
```rust
// src/main.rs - App loading happens in background
if app_launcher_enabled {
    std::thread::spawn(move || {
        let apps = app_launcher::scan_applications().clone();
        let _ = tx.send((apps, elapsed));
    });
    
    cx.spawn(async move |this, cx| {
        loop {
            Timer::after(std::time::Duration::from_millis(50)).await;
            match rx.try_recv() {
                Ok((apps, elapsed)) => { /* update state */ }
                Err(TryRecvError::Empty) => continue,
                Err(TryRecvError::Disconnected) => break,
            }
        }
    }).detach();
}
```

The UI shows scripts/scriptlets immediately while apps load in background.

### 4.2 Missing Loading Patterns

| Pattern | Status | Use Case |
|---------|--------|----------|
| Spinner/loader | Not implemented | Long operations |
| Skeleton screens | Not implemented | Initial content loading |
| Progress bars | Not implemented | File operations |
| Loading text | Not implemented | Background operations |

### 4.3 Toast-Based Feedback

Instead of loading indicators, the app uses toasts for operation feedback:
```rust
// Success/error feedback via toast
).duration_ms(Some(5000))  // 5 second auto-dismiss
```

### 4.4 Assessment

| Aspect | Status | Notes |
|--------|--------|-------|
| Loading indicators | Missing | No spinners/skeletons |
| Background operations | Silent | No visual feedback during load |
| Progress feedback | Toast-based | Post-completion only |
| Perceived performance | Good | Fast enough to skip loading UI |

**Recommendation:** Consider adding loading states for script execution and file operations.

---

## 5. Scroll Behavior

### 5.1 Scroll Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    SCROLL IMPLEMENTATION                         │
├─────────────────────────────────────────────────────────────────┤
│  uniform_list()           - Virtualized list (fixed item height) │
│  UniformListScrollHandle  - Programmatic scroll control          │
│  ScrollStrategy           - Top | Nearest | Center               │
│  .track_scroll(&handle)   - Connects list to scroll handle       │
└─────────────────────────────────────────────────────────────────┘
```

### 5.2 Scroll Handles

Six separate scroll handles for different views:
```rust
list_scroll_handle: UniformListScrollHandle,           // Main script list
arg_list_scroll_handle: UniformListScrollHandle,       // Arg prompt choices
clipboard_list_scroll_handle: UniformListScrollHandle, // Clipboard history
window_list_scroll_handle: UniformListScrollHandle,    // Window switcher
design_gallery_scroll_handle: UniformListScrollHandle, // Design gallery
```

### 5.3 Scroll Behaviors

#### Scroll-to-Item on Selection
```rust
// src/main.rs
fn scroll_to_selected_if_needed(&mut self, _reason: &str) {
    let target = self.selected_index;
    
    if self.last_scrolled_index == Some(target) {
        return;  // Skip redundant scroll
    }
    
    self.list_scroll_handle.scroll_to_item(target, ScrollStrategy::Nearest);
    self.last_scrolled_index = Some(target);
}
```

#### Filter Change Scroll Reset
```rust
// Scroll to top when filter changes
self.list_scroll_handle.scroll_to_item(0, ScrollStrategy::Top);
self.last_scrolled_index = Some(0);
```

### 5.4 Scrollbar Fade Animation

```rust
// Scroll activity tracking
is_scrolling: bool,
last_scroll_time: Option<std::time::Instant>,

fn trigger_scroll_activity(&mut self, cx: &mut Context<Self>) {
    self.is_scrolling = true;
    self.last_scroll_time = Some(std::time::Instant::now());
    
    // Schedule fade-out after 1000ms of inactivity
    cx.spawn(async move |this, cx| {
        Timer::after(std::time::Duration::from_millis(1000)).await;
        let _ = cx.update(|cx| {
            this.update(cx, |app, cx| {
                if let Some(last_time) = app.last_scroll_time {
                    if last_time.elapsed() >= Duration::from_millis(1000) {
                        app.is_scrolling = false;  // Fade out
                        cx.notify();
                    }
                }
            })
        });
    }).detach();
}
```

### 5.5 Assessment

| Aspect | Status | Notes |
|--------|--------|-------|
| Smooth scrolling | Native | GPUI handles via uniform_list |
| Scroll-to-item | Working | ScrollStrategy::Nearest |
| Momentum scrolling | Native | macOS handles this |
| Position preservation | Good | `last_scrolled_index` tracking |
| Scrollbar visibility | Fade animation | 1000ms timeout |

---

## 6. Window Animations

### 6.1 Current Implementation

**All window animations are DISABLED:**

```rust
// src/window_resize.rs
let _: () = msg_send![window, setFrame:new_frame display:true animate:false];
```

### 6.2 Window Lifecycle

```
┌─────────────────────────────────────────────────────────────────┐
│                    WINDOW VISIBILITY                             │
├─────────────────────────────────────────────────────────────────┤
│  Show Sequence:                                                  │
│  1. ensure_move_to_active_space()                               │
│  2. Calculate bounds on mouse display                           │
│  3. move_first_window_to_bounds() (instant)                     │
│  4. cx.activate(true)                                           │
│  5. Configure as floating panel (first show only)               │
│  6. Window activated and focused                                 │
│                                                                  │
│  Hide Sequence:                                                  │
│  1. WINDOW_VISIBLE.store(false)                                 │
│  2. Cancel script if in prompt mode                             │
│  3. Reset to script list                                        │
│  4. cx.hide()                                                   │
└─────────────────────────────────────────────────────────────────┘
```

### 6.3 Window Resize Behavior

```rust
// Fixed heights (no dynamic resizing for main views)
pub const MIN_HEIGHT: Pixels = px(120.0);      // Input-only prompts
pub const STANDARD_HEIGHT: Pixels = px(500.0); // Script list, arg with choices
pub const MAX_HEIGHT: Pixels = px(700.0);      // Editor, terminal

// Views with preview panel - FIXED height
ViewType::ScriptList | ViewType::ArgPromptWithChoices | ViewType::DivPrompt => {
    STANDARD_HEIGHT  // Never resizes dynamically
}
```

### 6.4 Assessment

| Aspect | Status | Notes |
|--------|--------|-------|
| Open/close animations | Disabled | Instant show/hide |
| Resize smoothing | Disabled | Immediate frame change |
| Position transitions | None | Instant repositioning |
| Performance | Optimal | No animation overhead |

---

## 7. Performance Considerations

### 7.1 Event Coalescing (Documented Pattern)

The AGENTS.md documents a 20ms coalescing window for rapid keyboard events:
```rust
// Pattern from AGENTS.md (not directly found in current code)
fn process_arrow_key_with_coalescing(&mut self, direction: ScrollDirection) {
    let coalesce_window = Duration::from_millis(20);
    
    if now.duration_since(self.last_scroll_time) < coalesce_window
       && self.pending_scroll_direction == Some(direction) {
        self.pending_scroll_delta += 1;
        return;
    }
    // ...
}
```

### 7.2 60fps Target

Performance targets from AGENTS.md:
| Metric | Threshold |
|--------|-----------|
| P95 Key Latency | < 50ms |
| Single Key Event | < 16.67ms (60fps) |
| Scroll Operation | < 8ms |

### 7.3 Optimization Patterns

1. **Filter Cache:** Avoids recomputing search results on every render
2. **Preview Cache:** Syntax highlighting cached by path
3. **Image Decoding:** Pre-decode PNG icons at load time, not render time
4. **Scroll Stabilization:** `last_scrolled_index` prevents redundant scroll_to_item calls

---

## 8. Recommendations

### 8.1 Short-term Improvements

1. **Standardize Timer Durations**
   - Create `src/animations.rs` with constants:
   ```rust
   pub const CURSOR_BLINK_MS: u64 = 530;
   pub const SCROLLBAR_FADE_MS: u64 = 1000;
   pub const TOAST_DEFAULT_MS: u64 = 5000;
   pub const TOAST_ERROR_MS: u64 = 10000;
   pub const DEBOUNCE_MS: u64 = 200;
   ```

2. **Add Loading States**
   - Simple "Loading..." text for script execution
   - Skeleton placeholders for app launcher on first load

3. **Focus-Visible Indicators**
   - Add keyboard focus ring for accessibility
   - Differentiate mouse hover from keyboard focus visually

### 8.2 Long-term Considerations

1. **Spring-based Animations**
   - If GPUI adds animation APIs, consider spring physics
   - Target 60fps with GPU-accelerated properties

2. **Multi-Select Support**
   - Change `selected_index: usize` to `selected_indices: Vec<usize>`
   - Add Cmd+Click and Shift+Click behaviors

3. **Transition System**
   - Abstract Timer-based state changes into reusable animation primitives
   - Consider opacity fade-in/out for view transitions

---

## 9. Summary Table

| Category | Current State | Consistency | Performance |
|----------|---------------|-------------|-------------|
| Transitions | None (immediate) | N/A | Excellent |
| Hover States | Opacity-based | Good (25%/50%) | Excellent |
| Selection | Single-select, visual hierarchy | Good | Excellent |
| Loading States | Missing | N/A | N/A |
| Scroll Behavior | GPUI native + fade | Good | Excellent |
| Window Animations | Disabled | N/A | Excellent |

**Overall Assessment:** The animation system is minimal by design, prioritizing immediate feedback and performance over visual polish. The opacity-based hover/selection states are consistent. The main gap is loading state UI for longer operations.

---

*Audit completed by AnimationAuditor agent*
