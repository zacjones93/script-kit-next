# Feature Bundle 22: Vibrancy Support Across All Windows

## Goal

Ensure background colors never block the macOS vibrancy effect across main, notes, and AI windows. Currently vibrancy is 95% implemented but some background colors may be opaque.

## Current State

**What Works:**
- `VibrancySettings` struct with enabled flag and material type
- `BackgroundOpacity` system with per-component opacity values (0.55-0.70)
- Window creation checks vibrancy and applies `WindowBackgroundAppearance::Blurred`
- All three windows (main, notes, AI) have vibrancy window options

**What's Missing:**
- Material type mapping (`"popover"`, `"hud"`) to GPUI not implemented (GPUI only has Blurred|Opaque)
- Some components may use opaque backgrounds that block vibrancy
- HUD windows use `Transparent` not `Blurred`
- No audit of all `bg()` calls to ensure they use theme opacity

## Key Questions

1. **Background Color Audit**: Which components use `rgb()` or `bg()` with fully opaque colors instead of `rgba()` with theme opacity?

2. **Opacity System Usage**: Is `BackgroundOpacity` consistently used? Are there hardcoded `bg(rgb(...))` calls that bypass the opacity system?

3. **Component-Level Vibrancy**: Should different components (header, list, preview) have different opacity levels? Current system has:
   - main: 0.60
   - title_bar: 0.65
   - search_box: 0.70
   - log_panel: 0.55

4. **Material Type**: GPUI only supports Blurred/Opaque/Transparent. Is there value in exposing macOS material types (popover, HUD, menu, sidebar) via NSVisualEffectView directly?

5. **Dark/Light Mode**: Does vibrancy work correctly in both appearances? Does the theme system adjust opacity for light vs dark?

## Implementation Checklist

- [ ] Audit all `bg()` calls for opaque colors
- [ ] Ensure all background colors use `rgba()` with theme opacity
- [ ] Verify vibrancy in Notes window
- [ ] Verify vibrancy in AI window
- [ ] Test HUD windows with vibrancy
- [ ] Document opacity values and their purpose

## Files to Review

- `src/theme/types.rs` - VibrancySettings, BackgroundOpacity
- `src/main.rs` - Window creation, vibrancy check
- `src/notes/window.rs` - Notes window background
- `src/ai/window.rs` - AI window background
- `src/hud_manager.rs` - HUD window background
- All render_* files for bg() usage

