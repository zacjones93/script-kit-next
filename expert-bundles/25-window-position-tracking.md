# Feature Bundle 25: Window Position Tracking & Persistence

## Goal

Track and persist window positions for main, notes, and AI windows. When reopened, windows appear in their last position. Include a "Reset Window Positions" built-in command.

## Current State

**Eye-Line Positioning (Calculated, Not Saved):**
```rust
// platform.rs - calculates position on every open
pub fn calculate_eye_line_bounds_on_mouse_display(
    window_size: gpui::Size<Pixels>,
) -> Bounds<Pixels> {
    // Centers horizontally on display containing mouse
    // Positions at ~14% from top (eye-line height)
}
```

**Window Creation:**
- Main: Uses `calculate_eye_line_bounds_on_mouse_display()`
- Notes: Uses `Bounds::centered()` in GPUI
- AI: Uses `Bounds::centered()` in GPUI

**What's NOT Implemented:**
- No position saved when window closed/moved
- No `window-state.json` or similar persistence
- Position always recalculated on startup
- User moves window, it resets next time

## Proposed Architecture

### 1. Position Storage

```typescript
// ~/.sk/kit/window-state.json
{
  "main": {
    "x": 100,
    "y": 200,
    "width": 750,
    "height": 475,
    "displayId": "main",
    "lastModified": "2024-01-15T..."
  },
  "notes": {
    "x": 850,
    "y": 100,
    "width": 400,
    "height": 600,
    "displayId": "main"
  },
  "ai": {
    "x": 200,
    "y": 150,
    "width": 500,
    "height": 700,
    "displayId": "secondary"
  }
}
```

### 2. Rust Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub main: Option<WindowBounds>,
    pub notes: Option<WindowBounds>,
    pub ai: Option<WindowBounds>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub display_id: Option<String>,
}
```

### 3. Restore Logic

```rust
fn get_window_bounds(role: WindowRole, size: Size<Pixels>, cx: &Context) -> Bounds<Pixels> {
    // 1. Try to load saved position
    if let Some(saved) = load_window_state(role) {
        // 2. Validate display still exists
        if is_display_available(&saved.display_id, cx) {
            // 3. Validate bounds are on-screen
            if is_bounds_visible(&saved, cx) {
                return saved.to_bounds();
            }
        }
    }
    // 4. Fallback to calculated position
    calculate_eye_line_bounds_on_mouse_display(size)
}
```

### 4. Save Logic

Need to detect window move/resize and save. Options:

**Option A: Save on close**
- Simple but loses data if app crashes
- Need window close event

**Option B: Save on move/resize**
- Requires GPUI or macOS events
- May need `NSWindowDelegate` for `windowDidMove:`
- Debounce rapid moves

**Option C: Periodic save**
- Poll window position every N seconds
- Less accurate but simpler

## Key Questions

1. **GPUI Events**: Does GPUI provide window move/resize callbacks? If not, we need macOS NSWindowDelegate.

2. **Multi-Monitor Edge Cases**:
   - Display unplugged → fallback to default
   - Resolution changed → bounds may be off-screen
   - How to detect "display no longer exists"?

3. **Display Identification**: Use display ID, name, or position? Display IDs can change across reboots.

4. **Reset Command UX**:
   - Should "Reset Window Positions" appear in main menu?
   - Only show if positions have been modified?
   - Reset all windows or per-window?

5. **Migration**: Existing users have no saved state. First open should save current calculated position.

## Implementation Steps

1. Add `WindowState` to `src/config/types.rs`
2. Create `src/window_state.rs` module:
   - `load_window_state(role) -> Option<WindowBounds>`
   - `save_window_state(role, bounds)`
   - `reset_all_positions()`
3. Update `main.rs` to restore position on window create
4. Update `notes/window.rs` and `ai/window.rs` similarly
5. Add position save triggers (close event or NSWindowDelegate)
6. Add "Reset Window Positions" built-in command
7. Handle edge cases (missing display, off-screen bounds)

## Built-in Command

```typescript
// Built-in: Reset Window Positions
{
  name: "Reset Window Positions",
  description: "Restore all windows to default positions",
  // Only show if any position has been customized
  visible: () => hasCustomWindowPositions(),
  run: async () => {
    await resetWindowPositions();
    notify("Window positions reset to defaults");
  }
}
```

