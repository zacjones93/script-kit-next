# Window Management & Positioning Audit

**Audit Date:** 2025-12-29  
**Auditor:** window-audit-worker  
**Scope:** Window lifecycle, controls, resize, multi-monitor positioning, panel mode, vibrancy

---

## Executive Summary

Script Kit GPUI implements a sophisticated window management system optimized for a macOS floating panel launcher (Raycast/Spotlight-like). The architecture uses a role-based window registry to reliably identify the main window among system windows, native macOS APIs for positioning and panel behavior, and a well-documented resize system with clear view type mappings.

### Key Strengths

1. **Robust Window Identification** - Registry-based approach avoids fragile index-based lookups
2. **Multi-Monitor Awareness** - Correctly positions window on display containing mouse cursor
3. **Native Panel Behavior** - Proper NSFloatingWindowLevel and MoveToActiveSpace implementation
4. **Clean View-to-Height Mapping** - Simple, testable resize logic per view type
5. **Vibrancy Support** - Theme-based transparency with WindowBackgroundAppearance::Blurred

### Areas for Improvement

1. **No Window Position Memory** - Always recalculates position on show (by design, but worth noting)
2. **Display Detection Workaround** - Uses native NSScreen API because GPUI's `cx.displays()` returns incorrect origins
3. **Hardcoded Magic Numbers** - Some positioning constants could be configurable
4. **Limited Window Control Integration** - Accessibility-based window control is separate from internal window management

---

## 1. Window Manager (`src/window_manager.rs`)

### Purpose
Thread-safe registry to track windows by role, solving the problem of unreliable window indexing when tray icons and system windows exist.

### Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Window Manager Architecture                       │
├─────────────────────────────────────────────────────────────────────┤
│  OnceLock<Mutex<WindowManager>>                                     │
│                                                                      │
│  ┌─────────────────────────────────┐                                │
│  │ windows: HashMap<WindowRole, id> │                               │
│  │   • MainWindow -> NSWindow ptr   │                               │
│  │   • (future roles...)            │                               │
│  └─────────────────────────────────┘                                │
│                                                                      │
│  Public API:                                                         │
│  • register_window(role, id)      - Store window reference          │
│  • get_window(role) -> Option<id> - Retrieve by role                │
│  • get_main_window() -> Option<id>- Convenience for MainWindow     │
│  • find_and_register_main_window()- Auto-detect by size (~750x500) │
└─────────────────────────────────────────────────────────────────────┘
```

### Key Implementation Details

| Aspect | Implementation |
|--------|----------------|
| **Thread Safety** | `OnceLock<Mutex<WindowManager>>` singleton |
| **Window Identification** | Size-based detection (750px width, 100-800px height) |
| **Pointer Storage** | Raw `usize` to avoid lifetime issues, wrapped in `WindowId` |
| **Platform Support** | macOS only; all functions are no-ops on other platforms |

### Code Quality: EXCELLENT

- Well-documented with ASCII architecture diagram
- Clear problem statement in module docs
- Comprehensive unit tests including thread safety

### Recommendations

1. **Add Window Validation** - Could periodically verify stored window pointer is still valid
2. **Consider Role Expansion** - `WindowRole` enum is extensible for settings/preview windows

---

## 2. Window Control (`src/window_control.rs`)

### Purpose
External window management using macOS Accessibility APIs - allows listing, moving, resizing, and tiling windows from other applications.

### Capabilities

| Function | Description |
|----------|-------------|
| `list_windows()` | List all visible windows with metadata |
| `move_window(id, x, y)` | Reposition window |
| `resize_window(id, w, h)` | Change window size |
| `set_window_bounds(id, bounds)` | Position and resize atomically |
| `tile_window(id, position)` | Tile to half/quadrant/fullscreen |
| `minimize_window(id)` | Minimize to dock |
| `maximize_window(id)` | Fill display (not fullscreen mode) |
| `focus_window(id)` | Raise and activate |
| `close_window(id)` | Close via close button |

### Tile Positions

```rust
pub enum TilePosition {
    LeftHalf,    RightHalf,
    TopHalf,     BottomHalf,
    TopLeft,     TopRight,
    BottomLeft,  BottomRight,
    Fullscreen,
}
```

### Permission Requirements

- Requires **Accessibility permission** in System Preferences
- `has_accessibility_permission()` - Check if granted
- `request_accessibility_permission()` - Open System Preferences prompt

### Window Cache

Uses `OnceLock<Mutex<HashMap<u32, usize>>>` to cache window references by ID for repeated operations.

### Code Quality: GOOD

- Clean FFI bindings for CoreFoundation/ApplicationServices
- Comprehensive error handling with anyhow
- Instrumented with tracing

### Recommendations

1. **Stale Cache Handling** - Cache may hold references to closed windows
2. **Display Detection** - `get_visible_display_bounds` uses hardcoded menu bar (25px) and dock (70px) heights instead of querying NSScreen.visibleFrame
3. **Memory Management** - Some CFRelease calls commented out to avoid crashes; needs review

---

## 3. Window Resize (`src/window_resize.rs`)

### Purpose
Dynamic window height management for different view types with clear, fixed mappings.

### Height Constants

```rust
pub mod layout {
    pub const MIN_HEIGHT: Pixels = px(120.0);      // Input-only prompts
    pub const STANDARD_HEIGHT: Pixels = px(500.0); // Views with preview (ScriptList, ArgWithChoices, Div)
    pub const MAX_HEIGHT: Pixels = px(700.0);      // Full content (Editor, Terminal)
}
```

### View Type Mapping

| ViewType | Height | Resizes Dynamically? |
|----------|--------|---------------------|
| `ScriptList` | 500px | No - FIXED |
| `ArgPromptWithChoices` | 500px | No - FIXED |
| `ArgPromptNoChoices` | 120px | No - compact |
| `DivPrompt` | 500px | No - matches main |
| `EditorPrompt` | 700px | No - full height |
| `TermPrompt` | 700px | No - full height |

### Resize Implementation

```rust
// macOS coordinate system: Y=0 at bottom, increases upward
// To keep the TOP of the window fixed, adjust origin.y
let height_delta = target_height - current_height;
let new_origin_y = current_frame.origin.y - height_delta;
```

Key points:
- Uses WindowManager to get main window reliably
- Skips resize if already at target height (1px tolerance)
- Keeps top edge fixed when resizing
- No animation (`animate:false`)

### Deferred Resize Pattern

```rust
pub fn defer_resize_to_view<T: Render>(view_type: ViewType, item_count: usize, cx: &mut Context<T>) {
    // 16ms delay (~1 frame at 60fps) ensures GPUI render cycle completes
    cx.spawn(async move |_this, _cx| {
        Timer::after(Duration::from_millis(16)).await;
        resize_first_window_to_height(target_height);
    }).detach();
}
```

**Why?** Direct resize during GPUI's render cycle causes "RefCell already borrowed" errors.

### Code Quality: EXCELLENT

- Clear documentation of design decisions
- Comprehensive unit tests for all view types
- Well-structured height constants

### Recommendations

1. **Content-Aware Height** - Consider option for DivPrompt to size to content
2. **Configurable Heights** - Heights are hardcoded; could be theme/config options
3. **Smooth Resize** - Currently no animation; could add optional transition

---

## 4. Multi-Monitor Positioning

### Implementation Location
`src/main.rs`: `calculate_eye_line_bounds_on_mouse_display()`

### Algorithm

```
1. Get mouse cursor position via CGEvent API
2. Query all displays via NSScreen (NOT cx.displays() - has bugs)
3. Find display containing mouse cursor
4. Calculate eye-line position (14% from top)
5. Center window horizontally on that display
```

### Display Detection

```rust
fn get_macos_displays() -> Vec<DisplayBounds> {
    // Uses NSScreen directly because GPUI's display.bounds()
    // returns incorrect origins for secondary displays
    
    // Coordinate conversion:
    // macOS: Y=0 at bottom of primary screen
    // We want: Y=0 at top, increasing downward
    let flipped_y = primary_height - frame.origin.y - frame.size.height;
}
```

### Eye-Line Positioning

```rust
// Eye-line: position window top at ~14% from screen top (input bar at eye level)
let eye_line_y = display.origin_y + display.height * 0.14;
let center_x = display.origin_x + (display.width - window_width) / 2.0;
```

### Logging

Extensive position logging with boxed headers:
```
╔════════════════════════════════════════════════════════════╗
║  CALCULATING WINDOW POSITION FOR MOUSE DISPLAY             ║
╚════════════════════════════════════════════════════════════╝
```

### Code Quality: GOOD

- Clear documentation of coordinate system conversion
- Robust fallback if mouse position unavailable
- Good diagnostic logging

### Recommendations

1. **GPUI Bug Report** - Document the `cx.displays()` origin issue upstream
2. **Configurable Eye-Line** - 14% is hardcoded; could be user preference
3. **Display Change Handling** - Consider repositioning if display configuration changes while hidden

---

## 5. Window Show/Hide Mechanics

### State Tracking

```rust
static WINDOW_VISIBLE: AtomicBool = AtomicBool::new(false);
static NEEDS_RESET: AtomicBool = AtomicBool::new(false);
static PANEL_CONFIGURED: AtomicBool = AtomicBool::new(false);
```

### Toggle Flow (Hotkey Triggered)

```
Hotkey Pressed
    │
    ├── Is Visible?
    │       │
    │       ├── YES: Hide Flow
    │       │       ├── Cancel any running script
    │       │       ├── Reset to script list
    │       │       ├── Set WINDOW_VISIBLE = false
    │       │       └── cx.hide()
    │       │
    │       └── NO: Show Flow
    │               ├── Set WINDOW_VISIBLE = true
    │               ├── ensure_move_to_active_space()  ← MUST be first!
    │               ├── Calculate bounds on mouse display
    │               ├── move_first_window_to_bounds()
    │               ├── cx.activate(true)
    │               ├── configure_as_floating_panel() (first time only)
    │               └── win.activate_window() + focus
```

### Critical Ordering

```rust
// Step 0: CRITICAL - Set MoveToActiveSpace BEFORE any activation
// This MUST happen before move_first_window_to_bounds, cx.activate(), 
// or win.activate_window() to prevent macOS from switching spaces
ensure_move_to_active_space();
```

### Hide Methods

1. **Hotkey** - Full reset + hide
2. **Escape Key** - Clears filter first, then hides if empty
3. **Script Completion** - `HideWindow` prompt message

### Code Quality: GOOD

- Clear state machine
- Proper ordering of macOS API calls
- Performance timing on hide operations

### Recommendations

1. **Escape Key Behavior** - Consider option to immediately hide vs. clear-first
2. **Animation** - No show/hide animation; could add subtle fade
3. **Focus Return** - Could return focus to previously active app on hide

---

## 6. Panel Mode (`NSFloatingWindowLevel`)

### Configuration Location
`src/main.rs`: `configure_as_floating_panel()`

### Settings Applied

| Property | Value | Effect |
|----------|-------|--------|
| `setLevel:` | 3 (NSFloatingWindowLevel) | Floats above normal windows |
| `setCollectionBehavior:` | 2 (MoveToActiveSpace) | Moves to current space when shown |
| `setRestorable:` | false | Disables macOS state restoration |
| `setFrameAutosaveName:` | empty string | Disables position caching |

### Space Behavior

```rust
fn ensure_move_to_active_space() {
    // NSWindowCollectionBehaviorMoveToActiveSpace = 2
    // Makes window MOVE to current space rather than forcing space switch
    let collection_behavior: u64 = 2;
    let _: () = msg_send![window, setCollectionBehavior:collection_behavior];
}
```

**Critical:** Must be called BEFORE any activation to prevent macOS from switching to the space where the window was last visible.

### One-Time Configuration

```rust
// Panel configured on first show only
if !PANEL_CONFIGURED.swap(true, Ordering::SeqCst) {
    configure_as_floating_panel();
}
```

### Code Quality: GOOD

- Correct NSWindow level usage
- Proper space behavior configuration
- State restoration disabled

### Recommendations

1. **Configurable Level** - Could offer option for status bar level (25) or screen saver level (1000)
2. **CanJoinAllSpaces Option** - Some users might prefer visible on all spaces
3. **Activation Policy** - Consider NSApplicationActivationPolicyAccessory for true background app behavior

---

## 7. Window Position Memory

### Current Behavior
**No persistence** - Window position is always recalculated on show:

1. Find display with mouse cursor
2. Center horizontally
3. Position at 14% eye-line from top

### Deliberate Design Choices

```rust
// CRITICAL: Disable macOS window state restoration
let _: () = msg_send![window, setRestorable:false];

// Also disable the window's autosave frame name
let empty_string: id = msg_send![class!(NSString), string];
let _: () = msg_send![window, setFrameAutosaveName:empty_string];
```

This is intentional - the app should appear on the active display at a consistent eye-line position, like Raycast/Spotlight.

### Trade-offs

| Approach | Pros | Cons |
|----------|------|------|
| **Always Recalculate** (current) | Consistent UX, always on active display | Can't remember preferred position |
| **Remember Last Position** | User control | May appear on wrong display |
| **Remember Per-Display** | Best of both | More complex, needs display identification |

### Recommendations

1. **Consider Per-Display Memory** - Store preferred position per display identifier
2. **Manual Override** - If user drags window, remember that override
3. **Config Option** - Let users choose behavior

---

## 8. Display Detection

### Primary API
`get_macos_displays()` using NSScreen:

```rust
fn get_macos_displays() -> Vec<DisplayBounds> {
    let screens: id = msg_send![class!(NSScreen), screens];
    // ... iterate and convert coordinates
}
```

### Coordinate System Conversion

```rust
// macOS: Y=0 at bottom of primary screen, increasing upward
// Converted to: Y=0 at top, increasing downward
let flipped_y = primary_height - frame.origin.y - frame.size.height;
```

### Mouse Position Detection

```rust
fn get_global_mouse_position() -> Option<(f64, f64)> {
    let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState).ok()?;
    let event = CGEvent::new(source).ok()?;
    let location = event.location();
    Some((location.x, location.y))
}
```

### GPUI Display API Issue

```rust
// GPUI's cx.displays() returns incorrect origins for secondary displays
let displays = get_macos_displays(); // Use native API instead
```

### Code Quality: GOOD

- Robust coordinate conversion
- Proper error handling
- Good fallback behavior

### Recommendations

1. **Report GPUI Bug** - Document and report the `cx.displays()` origin issue
2. **Display Change Events** - Subscribe to display configuration changes
3. **Retina Scaling** - Ensure correct handling of @2x displays

---

## 9. Vibrancy & Transparency

### Window Background Appearance

```rust
WindowOptions {
    window_background: WindowBackgroundAppearance::Blurred,
    // ...
}
```

### Theme Opacity Settings

```rust
impl Default for BackgroundOpacity {
    fn default() -> Self {
        BackgroundOpacity {
            main: 0.60,          // Lower for more vibrancy
            title_bar: 0.65,
            search_box: 0.70,
            log_panel: 0.55,
        }
    }
}
```

### Vibrancy Settings

```rust
pub struct VibrancySettings {
    pub enabled: bool,           // default: true
    pub material: String,        // default: "popover"
}
```

Available materials: "hud", "popover", "menu", "sidebar", "content"

### How Vibrancy Works

```
┌─────────────────────────────────────────┐
│ WindowBackgroundAppearance::Blurred     │  ← GPUI enables macOS blur
│                                         │
│ ┌─────────────────────────────────────┐ │
│ │ Background with opacity 0.60        │ │  ← Theme-controlled
│ │                                     │ │
│ │ Blur shows through transparent      │ │  ← Native macOS effect
│ │ portions of the background          │ │
│ └─────────────────────────────────────┘ │
└─────────────────────────────────────────┘
```

### Drop Shadow

```rust
pub struct DropShadow {
    pub color: HexColor,         // default: 0x000000
    pub blur_radius: f32,        // default: 30.0
    pub offset_x: f32,           // default: 0.0
    pub offset_y: f32,           // default: 4.0
}
```

### Code Quality: GOOD

- Theme-based customization
- Sensible defaults
- Clean API for opacity control

### Recommendations

1. **Material Selection** - Currently unused; could map to specific effects
2. **Focus-Based Opacity** - Could reduce vibrancy when unfocused
3. **Performance Option** - Allow disabling blur for lower-end machines

---

## Summary Table

| Component | File | Quality | Key Strength | Top Recommendation |
|-----------|------|---------|--------------|-------------------|
| Window Manager | `window_manager.rs` | Excellent | Role-based registry | Add window validation |
| Window Control | `window_control.rs` | Good | Accessibility API integration | Fix display bounds detection |
| Window Resize | `window_resize.rs` | Excellent | Clear view-height mapping | Make heights configurable |
| Multi-Monitor | `main.rs` | Good | Mouse-based display detection | Report GPUI display bug |
| Show/Hide | `main.rs` | Good | Proper API ordering | Add animation option |
| Panel Mode | `main.rs` | Good | Correct NSWindow settings | Consider activation policy |
| Position Memory | N/A | N/A | By design | Consider per-display memory |
| Display Detection | `main.rs` | Good | Coordinate conversion | Subscribe to display changes |
| Vibrancy | `theme.rs` + `panel.rs` | Good | Theme-based control | Add material selection |

---

## Appendix: Key Constants

```rust
// Window Size
const WINDOW_WIDTH: f32 = 750.0;
const MIN_HEIGHT: f32 = 120.0;
const STANDARD_HEIGHT: f32 = 500.0;
const MAX_HEIGHT: f32 = 700.0;

// Positioning
const EYE_LINE_PERCENT: f64 = 0.14;  // 14% from top

// Window Detection
const EXPECTED_WIDTH: f64 = 750.0;
const WIDTH_TOLERANCE: f64 = 50.0;
const MIN_DETECTION_HEIGHT: f64 = 100.0;
const MAX_DETECTION_HEIGHT: f64 = 800.0;

// Panel Configuration
const NSFloatingWindowLevel: i32 = 3;
const NSWindowCollectionBehaviorMoveToActiveSpace: u64 = 2;
```

---

## Appendix: Debug Logging Categories

| Category | Description |
|----------|-------------|
| `WINDOW_MGR` | Window registration and lookup |
| `POSITION` | Window positioning calculations |
| `RESIZE` | Height changes |
| `VISIBILITY` | Show/hide state changes |
| `HOTKEY` | Hotkey toggle handling |
| `PANEL` | Floating panel configuration |
| `PERF` | Performance timing |
