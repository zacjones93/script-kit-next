# Window Management Audit - Script Kit GPUI

## Executive Summary

The Script Kit GPUI window management system is well-architected for a launcher-style application. It provides multi-monitor support, floating panel behavior, and responsive show/hide toggling. The implementation uses a combination of GPUI's cross-platform APIs and macOS-specific Cocoa/Objective-C integration for native behavior.

**Overall Assessment: GOOD** - The codebase demonstrates solid window management patterns with proper multi-monitor support and macOS floating panel integration. Minor improvements possible in position persistence and animation smoothness.

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Multi-Monitor Support](#multi-monitor-support)
3. [Window Positioning](#window-positioning)
4. [Floating Panel Behavior (macOS)](#floating-panel-behavior-macos)
5. [Show/Hide Behavior](#showhide-behavior)
6. [Size Management](#size-management)
7. [Window Control (Accessibility APIs)](#window-control-accessibility-apis)
8. [Code Quality Analysis](#code-quality-analysis)
9. [Recommendations](#recommendations)

---

## Architecture Overview

### Module Structure

```
Window Management Modules:
├── window_manager.rs     # Thread-safe window registry (role-based lookup)
├── window_control.rs     # External window control via Accessibility APIs
├── window_resize.rs      # Dynamic window height management
├── panel.rs              # Floating panel configuration & cursor styling
└── main.rs               # Positioning, show/hide, window creation
```

### Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| Role-based window registry | Avoids NSApp window array index issues with tray/system windows |
| Native macOS APIs for positioning | GPUI's `display.bounds()` returns incorrect origins for secondary displays |
| Deferred resize operations | Prevents RefCell borrow conflicts during GPUI render cycle |
| One-time panel configuration | Avoids redundant native API calls on each show |

### Module Responsibilities

#### window_manager.rs (547 lines)
- **Purpose**: Thread-safe registry to track windows by role
- **Pattern**: `OnceLock<Mutex<HashMap<WindowRole, WindowId>>>`
- **Problem Solved**: NSApp's windows array contains unpredictable mix of app windows, tray popups, and system overlays
- **Key Functions**:
  - `register_window(role, id)` - Register a window by role
  - `get_main_window()` - Retrieve main window ID
  - `find_and_register_main_window()` - Find window by characteristic size (750×400-600)

#### window_control.rs (1081 lines)
- **Purpose**: Control external windows via macOS Accessibility APIs
- **Capabilities**: List windows, move, resize, minimize, maximize, tile, close, focus
- **Dependencies**: `AXUIElement`, `core_graphics`, `macos_accessibility_client`
- **Key Functions**:
  - `list_windows()` - Enumerate all visible windows
  - `tile_window(id, position)` - Snap to screen positions
  - `set_window_bounds(id, bounds)` - Precise positioning

#### window_resize.rs (296 lines)
- **Purpose**: Dynamic window height based on view type
- **Pattern**: Fixed heights for view types, deferred async execution
- **Height Rules**:
  - ScriptList/ArgWithChoices/DivPrompt: 500px (fixed)
  - ArgNoChoices: 120px (compact)
  - Editor/Terminal: 700px (full)

#### panel.rs (319 lines)
- **Purpose**: macOS floating panel configuration, cursor styling constants
- **Features**:
  - `WindowVibrancy` enum (Opaque, Transparent, Blurred)
  - `PlaceholderConfig` for input field behavior
  - `CursorStyle` constants for text input rendering

---

## Multi-Monitor Support

### Display Detection

**Implementation**: Native NSScreen API (not GPUI)

```rust
// main.rs:107-139
fn get_macos_displays() -> Vec<DisplayBounds> {
    unsafe {
        let screens: id = msg_send![class!(NSScreen), screens];
        let count: usize = msg_send![screens, count];
        
        // Coordinate flip: macOS Y=0 at bottom → Y=0 at top
        let main_frame: NSRect = msg_send![main_screen, frame];
        let primary_height = main_frame.size.height;
        
        for i in 0..count {
            let frame: NSRect = msg_send![screen, frame];
            let flipped_y = primary_height - frame.origin.y - frame.size.height;
            displays.push(DisplayBounds { origin_x, origin_y: flipped_y, width, height });
        }
    }
}
```

**Why Not GPUI?**: The codebase explicitly notes:
> "GPUI's cx.displays() returns incorrect origins for secondary displays"

### Mouse-Based Display Selection

```rust
// main.rs:307-392
fn calculate_eye_line_bounds_on_mouse_display(window_size, cx) {
    // 1. Get all displays via native API
    let displays = get_macos_displays();
    
    // 2. Get global mouse position via CoreGraphics
    let (mouse_x, mouse_y) = get_global_mouse_position()?;
    
    // 3. Find display containing mouse
    let target_display = displays.iter().find(|d| {
        mouse_x >= d.origin_x && mouse_x < d.origin_x + d.width &&
        mouse_y >= d.origin_y && mouse_y < d.origin_y + d.height
    });
    
    // 4. Fall back to primary if not found
    target_display.or(displays.first())
}
```

### Assessment: EXCELLENT ✅

| Criteria | Status |
|----------|--------|
| Correct display detection | ✅ Uses native NSScreen API |
| Proper coordinate flipping | ✅ Handles macOS bottom-left origin |
| Mouse-based selection | ✅ Finds display containing cursor |
| Fallback handling | ✅ Falls back to primary display |
| Logging for debugging | ✅ Comprehensive position logging |

---

## Window Positioning

### Eye-Line Positioning

The window is positioned at "eye-line" height (upper 14% of screen), matching Raycast/Alfred behavior:

```rust
// main.rs:366-371
// Eye-line: position window top at ~14% from screen top
let eye_line_y = display.origin_y + display.height * 0.14;

// Center horizontally on the display
let window_width: f64 = window_size.width.into();
let center_x = display.origin_x + (display.width - window_width) / 2.0;
```

### Coordinate System Handling

```rust
// main.rs:144-204 - move_first_window_to()
// macOS coordinates: Y=0 at bottom, increases upward
// GPUI/app coordinates: Y=0 at top, increases downward

// Convert from top-left origin (y down) to bottom-left origin (y up)
let flipped_y = primary_screen_height - y - height;

let new_frame = NSRect::new(
    NSPoint::new(x, flipped_y),
    NSSize::new(width, height),
);
let _: () = msg_send![window, setFrame:new_frame display:true animate:false];
```

### Position Persistence

**Current Behavior**: No position persistence - window always repositions to mouse display on show.

```rust
// From configure_as_floating_panel():
// Disable macOS window state restoration
let _: () = msg_send![window, setRestorable:false];
let _: () = msg_send![window, setFrameAutosaveName:empty_string];
```

**Rationale**: This is intentional for launcher behavior - always appear where the user is working.

### Assessment: GOOD ✅

| Criteria | Status |
|----------|--------|
| Eye-line positioning | ✅ Upper 14% of screen |
| Horizontal centering | ✅ Centered on target display |
| Coordinate conversion | ✅ Proper macOS flip handling |
| Position on show | ✅ Recalculates each time |
| State restoration | ✅ Explicitly disabled |

### Potential Enhancement
Consider caching display info to avoid NSScreen queries on rapid show/hide cycles.

---

## Floating Panel Behavior (macOS)

### NSWindow Configuration

```rust
// main.rs:8236-8271
fn configure_as_floating_panel() {
    unsafe {
        let window: id = msg_send![app, keyWindow];
        
        // 1. NSFloatingWindowLevel = 3 (above normal windows)
        let _: () = msg_send![window, setLevel:floating_level];
        
        // 2. MoveToActiveSpace = 2 (moves to current space, not all spaces)
        let _: () = msg_send![window, setCollectionBehavior:collection_behavior];
        
        // 3. Disable state restoration
        let _: () = msg_send![window, setRestorable:false];
        let _: () = msg_send![window, setFrameAutosaveName:empty_string];
    }
}
```

### Window Level Hierarchy

| Level | Value | Usage |
|-------|-------|-------|
| Normal | 0 | Standard app windows |
| Floating | 3 | Script Kit main window |
| Modal | 8+ | System dialogs |
| Screen Saver | 1000+ | Full-screen overlays |

### Collection Behavior Options

| Behavior | Value | Effect |
|----------|-------|--------|
| Default | 0 | Window stays on original space |
| CanJoinAllSpaces | 1 | Appears on all spaces simultaneously |
| **MoveToActiveSpace** | 2 | Moves to user's current space (USED) |
| Stationary | 16 | Fixed position, no space switching |

### Pre-Activation Setup

**Critical Pattern**: Collection behavior MUST be set before window activation:

```rust
// main.rs:648-671
// Step 0: CRITICAL - Set MoveToActiveSpace BEFORE any activation
ensure_move_to_active_space();

// Step 1: Calculate new bounds
let new_bounds = calculate_eye_line_bounds_on_mouse_display(window_size, cx);

// Step 2: Move window (position only, no activation)
move_first_window_to_bounds(&new_bounds);

// Step 3: NOW activate the app
cx.activate(true);
```

### Assessment: EXCELLENT ✅

| Criteria | Status |
|----------|--------|
| Floating level | ✅ Level 3 (NSFloatingWindowLevel) |
| Space behavior | ✅ MoveToActiveSpace (not all spaces) |
| Activation order | ✅ Collection behavior set first |
| State restoration | ✅ Disabled to prevent position caching |
| One-time config | ✅ PANEL_CONFIGURED flag prevents redundancy |

---

## Show/Hide Behavior

### Toggle Mechanism

```rust
// main.rs:621-707 - Hotkey handler
if WINDOW_VISIBLE.load(Ordering::SeqCst) {
    // HIDE: Cancel script, reset UI, hide window
    view.cancel_script_execution(ctx);
    view.reset_to_script_list(ctx);
    cx.hide();
} else {
    // SHOW sequence:
    // 1. Set MoveToActiveSpace collection behavior
    ensure_move_to_active_space();
    // 2. Calculate eye-line bounds on mouse display
    let new_bounds = calculate_eye_line_bounds_on_mouse_display(...);
    // 3. Move window to new position
    move_first_window_to_bounds(&new_bounds);
    // 4. Activate app (makes visible)
    cx.activate(true);
    // 5. Configure as floating panel (first show only)
    configure_as_floating_panel();
    // 6. Activate window and focus
    win.activate_window();
    win.focus(&focus_handle, cx);
}
```

### Visibility State Tracking

```rust
// Global atomics for state coordination
static WINDOW_VISIBLE: AtomicBool = AtomicBool::new(false);  // Starts hidden
static NEEDS_RESET: AtomicBool = AtomicBool::new(false);     // Reset flag
static PANEL_CONFIGURED: AtomicBool = AtomicBool::new(false); // One-time setup
```

### Performance Logging

```rust
// Hide latency measurement
let hide_start = std::time::Instant::now();
cx.hide();
let hide_elapsed = hide_start.elapsed();
logging::log("PERF", &format!("Window hide took {:.2}ms", ...));
```

### Focus Management

```rust
// On show:
win.activate_window();
let focus_handle = view.focus_handle(cx);
win.focus(&focus_handle, cx);

// Also sets NEEDS_RESET flag for post-script cleanup
```

### Assessment: EXCELLENT ✅

| Criteria | Status |
|----------|--------|
| Toggle works | ✅ Reliable show/hide cycle |
| State tracking | ✅ AtomicBool for thread safety |
| Focus handling | ✅ Proper focus on show |
| Script cleanup | ✅ Cancels running scripts on hide |
| Performance tracking | ✅ Latency logging |

---

## Size Management

### View-Based Height System

```rust
// window_resize.rs:23-36
pub mod layout {
    pub const MIN_HEIGHT: Pixels = px(120.0);      // Input-only prompts
    pub const STANDARD_HEIGHT: Pixels = px(500.0); // Script list, arg with choices
    pub const MAX_HEIGHT: Pixels = px(700.0);      // Editor, terminal
}
```

### Height Calculation

```rust
// window_resize.rs:63-98
pub fn height_for_view(view_type: ViewType, _item_count: usize) -> Pixels {
    match view_type {
        ViewType::ScriptList | ViewType::ArgPromptWithChoices | ViewType::DivPrompt 
            => STANDARD_HEIGHT,
        ViewType::ArgPromptNoChoices 
            => MIN_HEIGHT,
        ViewType::EditorPrompt | ViewType::TermPrompt 
            => MAX_HEIGHT,
    }
}
```

### Deferred Resize Pattern

```rust
// window_resize.rs:123-139
pub fn defer_resize_to_view<T: Render>(view_type, item_count, cx) {
    let target_height = height_for_view(view_type, item_count);
    
    cx.spawn(async move |_this, _cx| {
        // 16ms delay (~1 frame at 60fps) ensures GPUI render cycle completes
        Timer::after(Duration::from_millis(16)).await;
        
        if window_manager::get_main_window().is_some() {
            resize_first_window_to_height(target_height);
        }
    }).detach();
}
```

**Why Deferred?**: Direct calls during GPUI's render cycle cause "RefCell already borrowed" errors because `setFrame:display:animate:` happens synchronously.

### Resize Implementation

```rust
// window_resize.rs:153-217
pub fn resize_first_window_to_height(target_height: Pixels) {
    // Get window from WindowManager (not NSApp array)
    let window = window_manager::get_main_window()?;
    
    // Skip if already at target (within 1px tolerance)
    if (current_height - height_f64).abs() < 1.0 {
        return;
    }
    
    // Keep TOP edge fixed (anchor from top)
    // macOS Y=0 at bottom, so adjust origin.y when height changes
    let height_delta = height_f64 - current_height;
    let new_origin_y = current_frame.origin.y - height_delta;
    
    let _: () = msg_send![window, setFrame:new_frame display:true animate:false];
}
```

### Assessment: GOOD ✅

| Criteria | Status |
|----------|--------|
| Fixed heights per view | ✅ Predictable sizing |
| Top-edge anchoring | ✅ Proper coordinate math |
| Deferred execution | ✅ Avoids RefCell conflicts |
| Skip redundant resizes | ✅ 1px tolerance check |
| Animation | ⚠️ Disabled (animate:false) |

### Enhancement Opportunity
Consider enabling `animate:true` for smoother height transitions.

---

## Window Control (Accessibility APIs)

### Overview

The `window_control.rs` module provides control over external windows (not Script Kit's own window) via macOS Accessibility APIs.

### Permission Requirement

```rust
pub fn has_accessibility_permission() -> bool {
    accessibility::application_is_trusted()
}

pub fn request_accessibility_permission() -> bool {
    accessibility::application_is_trusted_with_prompt()
}
```

### Window Operations

| Operation | Function | Description |
|-----------|----------|-------------|
| List | `list_windows()` | Enumerate all visible windows |
| Move | `move_window(id, x, y)` | Change position |
| Resize | `resize_window(id, w, h)` | Change dimensions |
| Tile | `tile_window(id, position)` | Snap to halves/quadrants |
| Minimize | `minimize_window(id)` | Dock window |
| Maximize | `maximize_window(id)` | Fill visible area |
| Close | `close_window(id)` | Close window |
| Focus | `focus_window(id)` | Bring to front |

### Tile Positions

```rust
pub enum TilePosition {
    LeftHalf,      // Left 50%
    RightHalf,     // Right 50%
    TopHalf,       // Top 50%
    BottomHalf,    // Bottom 50%
    TopLeft,       // Top-left quadrant
    TopRight,      // Top-right quadrant
    BottomLeft,    // Bottom-left quadrant
    BottomRight,   // Bottom-right quadrant
    Fullscreen,    // Entire visible area
}
```

### Window Caching

```rust
// Cache AXUIElement references for repeated operations
static WINDOW_CACHE: OnceLock<Mutex<HashMap<u32, usize>>> = OnceLock::new();

fn cache_window(id: u32, window_ref: AXUIElementRef) { ... }
fn get_cached_window(id: u32) -> Option<AXUIElementRef> { ... }
fn clear_window_cache() { ... }
```

### Display Bounds Calculation

```rust
fn get_visible_display_bounds(x: i32, y: i32) -> Bounds {
    // Account for menu bar (25px on main) and dock (~70px)
    let menu_bar_height = if is_main { 25 } else { 0 };
    let dock_height = if is_main { 70 } else { 0 };
    
    // Note: Should ideally query NSScreen.visibleFrame
}
```

### Assessment: GOOD ✅

| Criteria | Status |
|----------|--------|
| Permission handling | ✅ Proper checks |
| Window enumeration | ✅ Filters small/utility windows |
| Caching | ✅ Avoids repeated AX queries |
| Tiling positions | ✅ Full set of options |
| Display bounds | ⚠️ Hardcoded menu/dock heights |

### Enhancement Opportunity
Query `NSScreen.visibleFrame` for accurate visible bounds instead of hardcoded dock/menu bar heights.

---

## Code Quality Analysis

### Strengths

1. **Thread Safety**: Proper use of `OnceLock`, `Mutex`, and `AtomicBool`
2. **Comprehensive Logging**: Position, resize, and visibility changes logged
3. **Error Handling**: Graceful fallbacks (e.g., primary display when mouse not found)
4. **Documentation**: Extensive module-level documentation explaining design decisions
5. **Testing**: Unit tests for core logic (tile bounds calculation, layout constants)

### Code Patterns

#### Thread-Safe Singleton (WindowManager)
```rust
static WINDOW_MANAGER: OnceLock<Mutex<WindowManager>> = OnceLock::new();

fn get_manager() -> &'static Mutex<WindowManager> {
    WINDOW_MANAGER.get_or_init(|| Mutex::new(WindowManager::new()))
}
```

#### Platform Abstraction
```rust
#[cfg(target_os = "macos")]
pub fn configure_as_floating_panel() { /* macOS impl */ }

#[cfg(not(target_os = "macos"))]
pub fn configure_as_floating_panel() { /* no-op */ }
```

#### Deferred Async Operations
```rust
cx.spawn(async move |_this, _cx| {
    Timer::after(Duration::from_millis(16)).await;
    // Safe to call NSWindow APIs now
}).detach();
```

### Potential Issues

1. **Hardcoded Magic Numbers**:
   - Eye-line at 0.14 of screen height
   - Menu bar 25px, dock 70px
   - Resize delay 16ms

2. **No Animation**: All window operations use `animate:false`

3. **Window Size Detection**: `find_and_register_main_window()` relies on size heuristics (750×400-600)

---

## Recommendations

### Priority 1: Critical (None Identified)

No critical issues found. The window management is robust.

### Priority 2: Improvements

| Issue | Recommendation | Effort |
|-------|----------------|--------|
| Hardcoded dock/menu heights | Query `NSScreen.visibleFrame` | Medium |
| No resize animation | Consider `animate:true` for height changes | Low |
| Magic eye-line constant | Make configurable via config.ts | Low |

### Priority 3: Enhancements

| Enhancement | Benefit | Effort |
|-------------|---------|--------|
| Display info caching | Faster show on rapid toggle | Low |
| Custom window level | Allow config-based level selection | Low |
| Window shadow control | Match other launcher aesthetics | Medium |

### Code Example: Improved Display Bounds

```rust
// Instead of hardcoded values:
fn get_visible_display_bounds(screen: id) -> Bounds {
    unsafe {
        // visibleFrame excludes menu bar and dock automatically
        let visible: NSRect = msg_send![screen, visibleFrame];
        Bounds::from_cg_rect(visible)
    }
}
```

---

## Summary

The Script Kit GPUI window management system is well-implemented with:

- ✅ Proper multi-monitor support using native NSScreen APIs
- ✅ Correct floating panel behavior with space management
- ✅ Reliable show/hide toggling with state tracking
- ✅ Thread-safe window registry avoiding NSApp array issues
- ✅ Deferred resize pattern preventing GPUI conflicts

The main areas for future improvement are:
1. Using NSScreen.visibleFrame for accurate bounds
2. Adding smooth animations for resize transitions
3. Making positioning constants configurable

**Overall Rating: 8.5/10**
