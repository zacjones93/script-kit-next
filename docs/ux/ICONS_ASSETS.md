# Icon System & Visual Assets Audit

**Audit Date**: December 29, 2025  
**Auditor**: icon-audit-worker  
**Scope**: Complete analysis of SVG icon system, visual assets, theming, and accessibility

---

## Executive Summary

The Script Kit GPUI application has a **well-organized icon system** with 22 SVG icons across 5 categories. The system includes robust infrastructure for icon loading, theming, and rendering via GPUI's `svg()` element. However, there are **significant accessibility gaps** and some **consistency issues** that should be addressed.

### Key Findings

| Aspect | Status | Notes |
|--------|--------|-------|
| Icon Inventory | Good | 22 icons covering common use cases |
| Categorization | Excellent | Clear 5-category system (Files, Actions, Status, Arrows, Media) |
| Theming Support | Partial | Some icons use `currentColor`, others hardcode `black` |
| Size Consistency | Good | All 16x16 viewBox, but stroke widths vary |
| Accessibility | Poor | No alt text, aria labels, or titles in SVGs |
| Icon API | Excellent | Comprehensive `icon_variations.rs` with string parsing |

---

## 1. Icon Inventory

### Complete Icon List (22 Total)

| Filename | Category | ViewBox | Stroke Width | Color Approach | Filled/Outlined |
|----------|----------|---------|--------------|----------------|-----------------|
| `file.svg` | Files | 16x16 | 1.2 | `currentColor` | Outlined |
| `file_code.svg` | Files | 16x16 | 1.2 | `currentColor` | Outlined |
| `folder.svg` | Files | 16x16 | 1.2 | `black` | Outlined |
| `folder_open.svg` | Files | 16x16 | 1.2 | `black` | Mixed (fill+stroke) |
| `plus.svg` | Actions | 16x16 | 1.2 | `black` | Outlined |
| `trash.svg` | Actions | 16x16 | 1.2 | `black` | Outlined |
| `copy.svg` | Actions | 16x16 | 1.2 | `black` | Outlined |
| `settings.svg` | Actions | 16x16 | 1.2 | `black` (truncated) | Mixed |
| `magnifying_glass.svg` | Actions | 16x16 | 1.2 | `black` | Mixed (fill+stroke) |
| `terminal.svg` | Actions | 16x16 | 1.0 | `black` | Mixed (fill+stroke) |
| `code.svg` | Actions | 16x16 | 1.2 | `currentColor` | Outlined |
| `check.svg` | Status | 16x16 | 1.2 | `black` | Outlined |
| `star.svg` | Status | 16x16 | 1.2 | `black` | Outlined |
| `star_filled.svg` | Status | 16x16 | 1.2 | `black` | Filled |
| `bolt_filled.svg` | Status | 16x16 | N/A | `black` | Filled |
| `bolt_outlined.svg` | Status | 16x16 | 1.2 | `black` | Mixed (15% fill + stroke) |
| `arrow_right.svg` | Arrows | 16x16 | 1.2 | `black` | Outlined |
| `arrow_down.svg` | Arrows | 16x16 | 1.2 | `black` | Outlined |
| `chevron_right.svg` | Arrows | 16x16 | 1.2 | `black` | Outlined |
| `chevron_down.svg` | Arrows | 16x16 | 1.2 | `black` | Outlined |
| `play_filled.svg` | Media | 16x16 | 1.2 | `black` | Filled |
| `play_outlined.svg` | Media | 16x16 | 1.2 | `black` | Outlined |

### Category Distribution

```
Files (4):     file, file_code, folder, folder_open
Actions (7):   plus, trash, copy, settings, magnifying_glass, terminal, code
Status (5):    check, star, star_filled, bolt_filled, bolt_outlined
Arrows (4):    arrow_right, arrow_down, chevron_right, chevron_down
Media (2):     play_filled, play_outlined
```

---

## 2. Logo Asset

### `assets/logo.svg`

```svg
<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" fill="currentColor" viewBox="0 0 32 32">
  <path fill="currentColor" d="M14 25a2 2 0 0 1 2-2h14a2 2 0 1 1 0 4H16a2 2 0 0 1-2-2ZM0 7.381c0-1.796 1.983-2.884 3.498-1.92l13.728 8.736c1.406.895 1.406 2.946 0 3.84L3.498 26.775C1.983 27.738 0 26.649 0 24.854V7.38Z"/>
</svg>
```

**Analysis**:
- Size: 32x32 (suitable for menu bar/tray icon)
- Color: Uses `currentColor` (theme-friendly)
- Design: Play button + command line underscore motif
- Usage: Rendered in main.rs at line 5014-5015 and embedded in tray.rs

**Strengths**:
- Clean, recognizable design
- Proper use of `currentColor` for theming
- Appropriate size for tray icon (32x32)

---

## 3. Icon Loading & Rendering

### GPUI SVG Rendering Flow

```rust
// From src/designs/icon_variations.rs
svg()
    .external_path(icon_name.external_path())  // File path to SVG
    .size(px(16.))                              // Size in pixels
    .text_color(rgb(0xffffff))                  // Color override
```

### Path Resolution System

The `IconName` enum provides compile-time paths via `concat!`:

```rust
pub fn external_path(&self) -> &'static str {
    match self {
        Self::File => concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/file.svg"),
        // ... etc
    }
}
```

**Strengths**:
- Compile-time path validation
- No runtime path construction overhead
- Static lifetime for GPUI compatibility

### Icon Kind System

The `list_item.rs` supports three icon types:

| Type | Use Case | Example |
|------|----------|---------|
| `IconKind::Emoji` | Text/emoji icons | "📜", "⚡" |
| `IconKind::Image` | Pre-decoded PNG (app icons) | App launcher icons |
| `IconKind::Svg` | SVG icons by name | "File", "Terminal" |

---

## 4. Theming Analysis

### Current State

**CRITICAL ISSUE**: Inconsistent color approaches across icons.

| Color Approach | Icons Using It | Count |
|---------------|----------------|-------|
| `currentColor` | file, file_code, code, logo | 4 |
| Hardcoded `black` | All others | 18 |

### Impact

When GPUI applies `text_color()` to an SVG:
- Icons using `currentColor`: Color is applied correctly
- Icons using `black`: Color override may not work as expected

### Theme Integration in list_item.rs

```rust
// Line 287: Icon text color matching selection state
let icon_text_color = if self.selected { 
    rgb(colors.text_primary) 
} else { 
    rgb(colors.text_secondary) 
};
```

**Note**: GPUI's `text_color()` on SVGs only affects `currentColor` fills/strokes.

---

## 5. Size Consistency Analysis

### ViewBox Consistency

All 22 icons use `viewBox="0 0 16 16"` (100% consistent).

### Actual Visual Size Audit

| Icon | Visual Bounds | Notes |
|------|---------------|-------|
| magnifying_glass | 3-13 x 3-13 | ~10px visual, 3px padding |
| folder | 2-14 x 3-13 | ~12px x 10px visual |
| check | 4.6-11.3 x 4.8-10.8 | ~7px visual, asymmetric |
| plus | 3.3-12.7 x 3.3-12.7 | ~9px visual, centered |
| terminal | 2.5-13.5 x 2.5-13.5 | ~11px visual |

**Finding**: Visual sizes are reasonably consistent (8-12px within 16px viewBox).

### Stroke Width Consistency

| Stroke Width | Icons |
|--------------|-------|
| 1.2px | 20 icons (91%) |
| 1.0px | terminal (4.5%) |
| N/A (filled) | bolt_filled (4.5%) |

**Recommendation**: Normalize `terminal.svg` to 1.2px stroke for consistency.

---

## 6. Icon Style Variations

### Defined Styles (from icon_variations.rs)

```rust
pub enum IconStyle {
    Default,        // 16px, 100% opacity
    Small,          // 12px
    Large,          // 24px
    Muted,          // 50% opacity
    Accent,         // Accent color
    CircleBackground,
    SquareBackground,
}
```

### Filled vs Outlined Pairs

| Concept | Outlined | Filled |
|---------|----------|--------|
| Star | `star.svg` | `star_filled.svg` |
| Bolt | `bolt_outlined.svg` | `bolt_filled.svg` |
| Play | `play_outlined.svg` | `play_filled.svg` |

**Consistency**: Good paired variants for interactive state changes.

---

## 7. Accessibility Audit

### Current State: POOR

No accessibility attributes found in any SVG:

```bash
grep -E "aria|alt=|role=|title=|accessib" assets/icons/*.svg
# No results
```

### Missing Accessibility Features

| Feature | Current | Recommended |
|---------|---------|-------------|
| `<title>` element | Missing | Add descriptive title |
| `role="img"` | Missing | Add for semantic meaning |
| `aria-label` | Missing | Add at render time in GPUI |
| `aria-hidden` | Missing | Add when decorative |

### Recommended SVG Template

```xml
<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" 
     viewBox="0 0 16 16" fill="none" 
     role="img" aria-labelledby="title">
  <title id="title">File icon</title>
  <path .../>
</svg>
```

### GPUI-Level Accessibility

GPUI does not currently expose accessibility attributes for SVG elements. Accessibility should be handled at the container level:

```rust
// Recommended pattern (when GPUI supports it)
div()
    .role(Role::Img)
    .aria_label("Search")
    .child(svg().external_path(IconName::MagnifyingGlass.external_path()))
```

---

## 8. Missing Icons Analysis

### Common Actions Without Icons

| Action | Current Fallback | Recommended Icon |
|--------|------------------|------------------|
| Close/X | None | `x.svg` or `close.svg` |
| Edit | None | `pencil.svg` or `edit.svg` |
| Refresh | None | `refresh.svg` or `arrow-clockwise.svg` |
| Undo | None | `undo.svg` or `arrow-left.svg` |
| Redo | None | `redo.svg` or `arrow-right.svg` |
| Download | None | `download.svg` or `arrow-down-tray.svg` |
| Upload | None | `upload.svg` or `arrow-up-tray.svg` |
| Info | None | `info.svg` or `info-circle.svg` |
| Warning | None | `warning.svg` or `exclamation-triangle.svg` |
| Error | None | `error.svg` or `x-circle.svg` |
| Success | `check.svg` | Already covered |
| Menu/Hamburger | None | `menu.svg` or `bars.svg` |
| External Link | None | `external-link.svg` or `arrow-top-right-on-square.svg` |

### Priority Additions

**High Priority** (used in common UI patterns):
1. `x.svg` / `close.svg` - Close buttons, dismiss actions
2. `warning.svg` - Alert/warning states
3. `error.svg` - Error notifications
4. `info.svg` - Info tooltips, help

**Medium Priority** (nice to have):
5. `edit.svg` - Edit actions
6. `refresh.svg` - Reload/refresh
7. `external-link.svg` - External navigation

---

## 9. Icon API Assessment

### String-to-Icon Mapping

The `icon_name_from_str()` function provides flexible parsing:

```rust
// Supports multiple formats:
icon_name_from_str("File")           // Exact match
icon_name_from_str("file")           // Lowercase
icon_name_from_str("file code")      // With spaces
icon_name_from_str("file-code")      // Kebab case
icon_name_from_str("file_code")      // Snake case

// Supports aliases:
icon_name_from_str("search")         // → MagnifyingGlass
icon_name_from_str("add")            // → Plus
icon_name_from_str("delete")         // → Trash
icon_name_from_str("gear")           // → Settings
icon_name_from_str("run")            // → PlayFilled
```

**Strengths**:
- Flexible input handling
- Semantic aliases for discoverability
- Well-tested (see unit tests in icon_variations.rs)

### Test Coverage

```rust
// From icon_variations.rs tests
#[test]
fn test_icon_count() { assert_eq!(IconName::count(), 22); }
#[test]
fn test_all_icons_have_paths() { /* validates .svg extension */ }
#[test]
fn test_category_coverage() { /* ensures all icons categorized */ }
#[test]
fn test_icon_name_from_str() { /* comprehensive parsing tests */ }
```

---

## 10. Tray Icon Implementation

### System Tray Architecture

```
tray.rs
├── LOGO_SVG constant (32x32 SVG string)
├── TrayManager::create_icon_from_svg()
│   ├── usvg::Tree::from_str() - Parse SVG
│   ├── tiny_skia::Pixmap::new() - Create pixel buffer
│   ├── resvg::render() - Render to pixels
│   └── Icon::from_rgba() - Create tray icon
└── TrayIconBuilder with .with_icon_as_template(true)
```

**Strengths**:
- Proper macOS template image handling
- Light/dark mode adaptation
- SVG-to-raster conversion for cross-platform compatibility

**Tray Icon Size**: 32x32 (appropriate for Retina displays)

---

## 11. Recommendations

### High Priority

1. **Standardize Color Approach**
   - Convert all icons to use `currentColor` instead of `black`
   - This enables proper theming across light/dark modes
   
   ```xml
   <!-- Before -->
   <path stroke="black" .../>
   
   <!-- After -->
   <path stroke="currentColor" .../>
   ```

2. **Add Accessibility Attributes**
   - Add `<title>` elements to all SVGs
   - Add `role="img"` to SVGs
   - Consider GPUI-level aria-label support

3. **Add Missing Critical Icons**
   - `x.svg` / `close.svg`
   - `warning.svg`
   - `error.svg`
   - `info.svg`

### Medium Priority

4. **Normalize Stroke Widths**
   - Update `terminal.svg` from 1.0 to 1.2 stroke width

5. **Add Icon Hover States**
   - Consider adding `*_hover.svg` variants for interactive icons
   - Or implement color transitions at GPUI level

6. **Document Icon Usage**
   - Add JSDoc-style comments to IconName variants
   - Create visual icon catalog in design gallery

### Low Priority

7. **Add Animation Support**
   - Consider animated SVGs for loading/progress states
   - GPUI may need extension for animated SVG support

8. **Icon Size Presets**
   - Formalize 12px, 16px, 24px, 32px size presets
   - Ensure all icons render well at each size

---

## 12. Technical Debt

### Issues Found

| Issue | File | Line | Severity |
|-------|------|------|----------|
| Hardcoded black color | 18 SVG files | N/A | Medium |
| Missing accessibility | All 22 SVGs | N/A | High |
| Inconsistent stroke width | terminal.svg | 1 | Low |
| `#[allow(dead_code)]` on IconStyle | icon_variations.rs | 280 | Low |

### Test Gaps

- No visual regression tests for icon rendering
- No accessibility tests
- No theme switching tests for icon colors

---

## Appendix A: Icon File Details

### File Sizes

```
4.0K  arrow_down.svg
4.0K  arrow_right.svg
4.0K  bolt_filled.svg
4.0K  bolt_outlined.svg
4.0K  check.svg
4.0K  chevron_down.svg
4.0K  chevron_right.svg
4.0K  code.svg
4.0K  copy.svg
4.0K  file.svg
4.0K  file_code.svg
4.0K  folder.svg
4.0K  folder_open.svg
4.0K  magnifying_glass.svg
4.0K  play_filled.svg
4.0K  play_outlined.svg
4.0K  plus.svg
4.0K  settings.svg (truncated in audit - large path data)
4.0K  star.svg
4.0K  star_filled.svg
4.0K  terminal.svg
4.0K  trash.svg
```

### Appendix B: IconName Enum Reference

```rust
pub enum IconName {
    // Files (4)
    File, FileCode, Folder, FolderOpen,
    
    // Actions (7)
    Plus, Trash, Copy, Settings, MagnifyingGlass, Terminal, Code,
    
    // Status (5)
    Check, Star, StarFilled, BoltFilled, BoltOutlined,
    
    // Arrows (4)
    ArrowRight, ArrowDown, ChevronRight, ChevronDown,
    
    // Media (2)
    PlayFilled, PlayOutlined,
}
```

---

## Appendix C: Color Migration Script

For converting icons to `currentColor`:

```bash
#!/bin/bash
# Run from assets/icons/
for svg in *.svg; do
  sed -i '' 's/stroke="black"/stroke="currentColor"/g' "$svg"
  sed -i '' 's/fill="black"/fill="currentColor"/g' "$svg"
done
```

**Note**: Manual review needed after running - some icons intentionally use black for fill effects.

---

*End of Icon System & Visual Assets Audit*
