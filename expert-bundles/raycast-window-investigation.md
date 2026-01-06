# Raycast Window Configuration Investigation

## Date: 2026-01-05

## Summary

Investigation of Raycast's actual window configuration on macOS to understand how they achieve their native vibrancy appearance.

---

## 1. Window Levels (CGWindowLayer)

Raycast uses DIFFERENT window levels for different window types:

| Window Type | Layer | macOS Constant |
|-------------|-------|----------------|
| Main search window | 8 | NSModalPanelWindowLevel |
| Notes window | 8 | NSModalPanelWindowLevel |
| AI Chat window | 0 | NSNormalWindowLevel |
| Settings window | 0 | NSNormalWindowLevel |
| Menu bar items | 25 | NSStatusWindowLevel |

**KEY INSIGHT**: The main Raycast search window uses **Layer 8 (NSModalPanelWindowLevel)**, NOT Layer 3 (NSFloatingWindowLevel) as commonly assumed.

Our app currently uses NSFloatingWindowLevel (3). Consider changing to NSModalPanelWindowLevel (8) for the main window if we want it to appear above more windows consistently.

```rust
// Reference values
NSNormalWindowLevel = 0
NSFloatingWindowLevel = 3
NSModalPanelWindowLevel = 8
NSMainMenuWindowLevel = 24
NSStatusWindowLevel = 25
NSPopUpMenuWindowLevel = 101
```

---

## 2. Window Alpha

All Raycast windows report **alpha = 1.0** via CGWindowListCopyWindowInfo.

This confirms that Raycast:
- Does NOT use window-level transparency
- Uses NSVisualEffectView's blur material for visual transparency
- Draws semi-transparent content colors on top of the blur

---

## 3. Raycast Theme System (from open-source theme explorer)

Theme structure:
```typescript
type Theme = {
  author: string;
  name: string;
  appearance: "light" | "dark";
  colors: {
    background: string;        // Main background - SOLID hex color
    backgroundSecondary: string;
    text: string;
    selection: string;
    loader: string;
    red/orange/yellow/green/blue/purple/magenta: string;
  }
}
```

**Notable**:
- Themes only define SOLID hex colors (no alpha channel)
- Vibrancy effect is applied via NSVisualEffectView, not theme colors
- Dark themes use very dark backgrounds (e.g., #1E1E2E for Catppuccin Mocha)
- The blur effect tints these colors, making them appear semi-transparent

---

## 4. Application Settings (from `defaults read com.raycast.macos`)

Key findings:

```
raycastShouldFollowSystemAppearance = 0
```
Raycast forces dark mode and does NOT follow system appearance.

```
raycastPreferredWindowMode = compact
```
Uses compact window mode by default.

Theme IDs stored separately for light/dark appearances.

---

## 5. What Makes Raycast Look "Native"

Based on analysis:

### a) Window Level
Using NSModalPanelWindowLevel (8) makes it appear above regular windows but below system panels like Spotlight.

### b) Vibrancy via NSVisualEffectView
Raycast likely uses:
- **Material**: `.popover` or `.hudWindow` for dark mode
- **BlendingMode**: `.behindWindow` (blurs content behind)
- **State**: `.active` (always shows effect regardless of focus)
- **Appearance**: Forced to `.vibrantDark`

### c) No Visible Border
The blur effect itself creates the edge definition - no explicit border stroke needed.

### d) Soft Shadow
macOS provides a subtle shadow automatically for borderless windows with vibrancy.

### e) Corner Radius
~12px corner radius on the blur view, consistent with modern macOS design language.

### f) Dark Tint
Even over white backgrounds, Raycast maintains a dark appearance because NSVisualEffectView in dark mode applies a dark tint to the blurred content.

---

## 6. Comparison with Our Implementation

| Aspect | Raycast | Our App |
|--------|---------|---------|
| Window Level | 8 (ModalPanel) | 3 (Floating) |
| Window Alpha | 1.0 | 1.0 |
| Appearance | VibrantDark | VibrantDark |
| Material | Unknown (likely HUD_WINDOW) | HUD_WINDOW |
| Background Color | clearColor | clearColor |
| Swizzle Hack | Not needed (native app) | Yes (GPUI hides CAChameleonLayer) |

---

## 7. Actionable Recommendations

### a) Window Level Change
Consider changing from NSFloatingWindowLevel (3) to NSModalPanelWindowLevel (8):

```rust
// In platform.rs
let _: () = msg_send![window, setLevel: 8i32];  // NSModalPanelWindowLevel
```

### b) Vibrancy Config (Already Correct)
Our current configuration matches Raycast:
- HUD_WINDOW material
- VibrantDark appearance
- BehindWindow blending
- Active state
- Emphasized = true

### c) Background Opacity (Already Correct)
- Keep window alpha = 1.0 (opaque to system)
- Use semi-transparent GPUI colors for vibrancy effect
- Current 70-85% opacity approach is correct

### d) Border Removal
Consider removing explicit border and letting the blur edge define the window boundary. Or use very subtle border (0.5px, 10% white opacity).

### e) GPUI Limitation
The key difference is that GPUI hides the CAChameleonLayer which provides the native dark tint. Our swizzle hack is the correct workaround since we can't modify GPUI itself.

---

## 8. Files Referenced

| File | Purpose |
|------|---------|
| `src/platform.rs` | Window level, vibrancy configuration, swizzle hack |
| `src/theme/gpui_integration.rs` | GPUI theme color mapping with opacity |
| `src/main.rs` | Window creation with WindowBackgroundAppearance |
| `src/notes/window.rs` | Notes window vibrancy setup |

---

## 9. Investigation Methods Used

1. **CGWindowListCopyWindowInfo** - Retrieved window layers/alpha from running Raycast
2. **Accessibility API** - Attempted window inspection (blocked when windows hidden)
3. **defaults read** - Retrieved Raycast app settings
4. **GitHub theme explorer** - Analyzed open-source theme format
5. **Swift scripts** - Created inspection tools for window properties

---

## 10. Raw Data

### CGWindowListCopyWindowInfo for Raycast main search window:
```
Layer: 8 (NSModalPanelWindowLevel)
Alpha: 1.0
Bounds: {Height=95, Width=750, X=705, Y=52}
Name: (unnamed - search panel)
```

### Example theme (Catppuccin Mocha):
```json
{
  "appearance": "dark",
  "colors": {
    "background": "#1E1E2E",
    "backgroundSecondary": "#1E1E2E",
    "text": "#CDD6F4",
    "selection": "#6C7086"
  }
}
```
