# Icon System Audit

**Audit Date:** December 29, 2025  
**Agent:** IconAuditor  
**Task:** cell--9bnr5-mjqv3ey632g

## Executive Summary

Script Kit GPUI has a well-organized icon system with 22 SVG icons in `assets/icons/`, managed through the `src/designs/icon_variations.rs` module. The system provides type-safe icon access, categorization, and flexible rendering styles. However, there are **critical consistency issues** in the SVG files themselves and **notable gaps** in the icon coverage for a launcher application.

### Quick Stats
- **Total Icons:** 22
- **Categories:** 5 (Files, Actions, Status, Arrows, Media)
- **Rendering Styles:** 7 (Default, Small, Large, Muted, Accent, CircleBackground, SquareBackground)
- **Standard Size:** 16x16px (with 12px and 24px variants)

---

## 1. SVG File Analysis

### 1.1 Overview of All Icons

| Icon File | ViewBox | Width/Height | Stroke Width | Fill Method | Color Handling |
|-----------|---------|--------------|--------------|-------------|----------------|
| magnifying_glass.svg | 0 0 16 16 | 16x16 | 1.2 | stroke + fill shadow | `black` (hardcoded) |
| folder.svg | 0 0 16 16 | 16x16 | 1.2 | stroke-only | `black` (hardcoded) |
| folder_open.svg | 0 0 16 16 | 16x16 | 1.2 | fill + stroke | `black` (hardcoded) |
| star.svg | 0 0 16 16 | 16x16 | 1.2 | stroke-only | `black` (hardcoded) |
| trash.svg | 0 0 16 16 | 16x16 | 1.2 | stroke-only | `black` (hardcoded) |
| play_outlined.svg | 0 0 16 16 | 16x16 | 1.2 | stroke-only | `black` (hardcoded) |
| play_filled.svg | 0 0 16 16 | 16x16 | 1.2 | fill + stroke | `black` (hardcoded) |
| plus.svg | 0 0 16 16 | 16x16 | 1.2 | stroke-only | `black` (hardcoded) |
| star_filled.svg | 0 0 16 16 | 16x16 | 1.2 | fill + stroke | `black` (hardcoded) |
| settings.svg | 0 0 16 16 | 16x16 | 1.2 | stroke + fill | `black` (hardcoded) |
| **file_code.svg** | **none** | 16x16 | 1.2 | stroke-only | **`currentColor`** |
| terminal.svg | 0 0 16 16 | 16x16 | 1.0 | fill + stroke | `black` (hardcoded) |
| bolt_filled.svg | 0 0 16 16 | 16x16 | N/A | fill-only (fill-rule) | `black` (hardcoded) |
| bolt_outlined.svg | 0 0 16 16 | 16x16 | 1.2 | fill + stroke | `black` + `fill-opacity: 0.15` |
| chevron_down.svg | 0 0 16 16 | 16x16 | 1.2 | stroke-only | `black` (hardcoded) |
| arrow_down.svg | 0 0 16 16 | 16x16 | 1.2 | stroke-only | `black` (hardcoded) |
| copy.svg | 0 0 16 16 | 16x16 | 1.2 | stroke-only | `black` (hardcoded) |
| check.svg | 0 0 16 16 | 16x16 | 1.2 | stroke-only | `black` (hardcoded) |
| arrow_right.svg | 0 0 16 16 | 16x16 | 1.2 | stroke-only | `black` (hardcoded) |
| **code.svg** | **none** | 16x16 | 1.2 | stroke-only | **`currentColor`** |
| chevron_right.svg | 0 0 16 16 | 16x16 | 1.2 | stroke-only | `black` (hardcoded) |
| **file.svg** | **none** | 16x16 | 1.2 | stroke-only | **`currentColor`** |

### 1.2 Consistency Issues

#### CRITICAL: Inconsistent Color Handling

**3 out of 22 icons use `currentColor`** (theme-aware), while **19 use hardcoded `black`**.

| Using `currentColor` (GOOD) | Using Hardcoded `black` (BAD) |
|----------------------------|-------------------------------|
| file_code.svg | All 19 other icons |
| code.svg | |
| file.svg | |

**Impact:** Icons with hardcoded `black` won't respond to theme color changes. GPUI applies `text_color()` to SVGs, but this only works reliably with `currentColor` in the SVG source.

**Recommendation:** Convert all icons to use `currentColor`:
```svg
<!-- BEFORE (won't theme properly) -->
<path stroke="black" stroke-width="1.2" .../>

<!-- AFTER (themes correctly) -->
<path stroke="currentColor" stroke-width="1.2" .../>
```

#### MODERATE: Inconsistent ViewBox Declaration

**3 icons lack an explicit viewBox attribute** (file_code.svg, code.svg, file.svg).

These rely on width/height for sizing, which can cause rendering issues when scaled.

**Recommendation:** Add `viewBox="0 0 16 16"` to all icons.

#### MINOR: Inconsistent Stroke Width

| Stroke Width | Icons | Count |
|--------------|-------|-------|
| 1.2 | Most icons | 20 |
| 1.0 | terminal.svg | 1 |
| N/A (fill-only) | bolt_filled.svg | 1 |

**Recommendation:** Standardize to `stroke-width="1.2"` for visual consistency.

#### MINOR: Fill Method Variations

| Method | Description | Icons |
|--------|-------------|-------|
| stroke-only | Outline icons | folder, star, play_outlined, plus, check, arrows, chevrons, copy |
| fill + stroke | Filled icons | folder_open, play_filled, star_filled, settings, terminal |
| fill-only | Solid icons | bolt_filled |
| fill-shadow | Shadow effect | magnifying_glass (has `fill-opacity="0.15"`) |

**Analysis:** The variation is intentional for outlined vs filled variants. The `_filled` and `_outlined` naming convention is good.

---

## 2. Icon Categories and Coverage

### 2.1 Current Categories

```rust
pub enum IconCategory {
    Files,    // file, file_code, folder, folder_open
    Actions,  // plus, trash, copy, settings, magnifying_glass, terminal, code
    Status,   // check, star, star_filled, bolt_filled, bolt_outlined
    Arrows,   // arrow_right, arrow_down, chevron_right, chevron_down
    Media,    // play_filled, play_outlined
}
```

### 2.2 Category Distribution

| Category | Icon Count | Icons |
|----------|------------|-------|
| Files | 4 | File, FileCode, Folder, FolderOpen |
| Actions | 7 | Plus, Trash, Copy, Settings, MagnifyingGlass, Terminal, Code |
| Status | 5 | Check, Star, StarFilled, BoltFilled, BoltOutlined |
| Arrows | 4 | ArrowRight, ArrowDown, ChevronRight, ChevronDown |
| Media | 2 | PlayFilled, PlayOutlined |
| **Total** | **22** | |

---

## 3. Missing Icons Analysis

### 3.1 Critical Missing Icons (High Priority)

For a launcher/automation application like Script Kit:

| Missing Icon | Use Case | Priority |
|--------------|----------|----------|
| **close / x** | Dismiss dialogs, clear input, close panels | P0 |
| **warning / alert** | Display warnings, caution states | P0 |
| **error / x-circle** | Error states, failed operations | P0 |
| **info / info-circle** | Information tooltips, help text | P0 |
| **success / check-circle** | Success states (check is plain, need circled) | P0 |

### 3.2 Important Missing Icons (Medium Priority)

| Missing Icon | Use Case | Priority |
|--------------|----------|----------|
| **edit / pencil** | Edit script, rename items | P1 |
| **duplicate** | Clone script, copy item | P1 |
| **external_link** | Open in browser, external URLs | P1 |
| **refresh / sync** | Reload scripts, sync state | P1 |
| **more / ellipsis** | Context menus, additional actions | P1 |
| **keyboard** | Keyboard shortcuts, hotkey hints | P1 |
| **arrow_up** | Navigate up (only have arrow_down) | P1 |
| **arrow_left** | Navigate back (only have arrow_right) | P1 |
| **chevron_up** | Collapse up | P1 |
| **chevron_left** | Navigate/collapse left | P1 |

### 3.3 Nice-to-Have Icons (Low Priority)

| Missing Icon | Use Case | Priority |
|--------------|----------|----------|
| **clock / time** | History, scheduling, recent items | P2 |
| **calendar** | Date-based scripts | P2 |
| **user** | User preferences, account | P2 |
| **lock / unlock** | Permissions, secure scripts | P2 |
| **command** | macOS command key symbol | P2 |
| **option** | macOS option key symbol | P2 |
| **shift** | Shift key symbol | P2 |
| **control** | Control key symbol | P2 |
| **download** | Download files/scripts | P2 |
| **upload** | Upload/share scripts | P2 |
| **link** | URLs, connections | P2 |
| **image** | Image/media files | P2 |
| **music** | Audio files | P2 |
| **video** | Video files | P2 |
| **globe** | Web, international | P2 |
| **moon / sun** | Theme toggle (dark/light) | P2 |

### 3.4 Comparison with Raycast/Alfred

| Icon | Raycast | Alfred | Script Kit |
|------|---------|--------|------------|
| Search | ✓ | ✓ | ✓ (magnifying_glass) |
| Settings | ✓ | ✓ | ✓ |
| File | ✓ | ✓ | ✓ |
| Folder | ✓ | ✓ | ✓ |
| Terminal | ✓ | ✓ | ✓ |
| Clipboard | ✓ | ✓ | ✓ (copy) |
| Star/Favorite | ✓ | ✓ | ✓ |
| Play/Run | ✓ | ✓ | ✓ |
| Warning | ✓ | ✓ | **MISSING** |
| Error | ✓ | ✓ | **MISSING** |
| Close/X | ✓ | ✓ | **MISSING** |
| External Link | ✓ | ✓ | **MISSING** |
| Keyboard | ✓ | ✓ | **MISSING** |
| History/Clock | ✓ | ✓ | **MISSING** |

---

## 4. Icon Naming Conventions

### 4.1 Current Naming Pattern

```
{base_name}[_variant].svg
```

Examples:
- `star.svg` (outlined) → `star_filled.svg` (filled)
- `bolt_outlined.svg` → `bolt_filled.svg`
- `play_outlined.svg` → `play_filled.svg`

### 4.2 Naming Consistency Issues

| Pattern | Examples | Issue |
|---------|----------|-------|
| **Inconsistent variant naming** | star.svg (implied outlined), star_filled.svg | Should be `star_outlined.svg` for consistency |
| **Missing variant pairs** | folder.svg, folder_open.svg | "Open" isn't a fill variant, it's a state variant (correct) |

**Recommendation:** Adopt consistent naming:
- `{name}.svg` - Default state (usually outlined)
- `{name}_filled.svg` - Filled variant
- `{name}_outlined.svg` - Explicitly outlined (optional)
- `{name}_{state}.svg` - State variants (e.g., folder_open)

---

## 5. Icon Loading and Rendering

### 5.1 Loading Mechanism

Icons are loaded via GPUI's `svg()` element using `external_path()`:

```rust
// From icon_variations.rs
pub fn external_path(&self) -> &'static str {
    match self {
        Self::File => concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/file.svg"),
        // ...
    }
}
```

**Analysis:**
- ✅ Uses compile-time path resolution (`concat!` + `env!`)
- ✅ Returns `&'static str` for zero-allocation
- ⚠️ Paths are absolute from build directory (works for development, may need adjustment for distribution)

### 5.2 Rendering Pattern

From `list_item.rs`:

```rust
svg()
    .external_path(svg_path)
    .size(px(16.))
    .text_color(icon_text_color)
```

**Analysis:**
- ✅ Uses `text_color()` for theming
- ⚠️ Only works if SVGs use `currentColor` (see Section 1.2)
- ✅ Consistent 16px sizing
- ✅ Wrapped in flex container for alignment

### 5.3 Performance Considerations

| Aspect | Current State | Recommendation |
|--------|---------------|----------------|
| SVG Loading | External file load | Consider embedding for faster startup |
| Caching | Unknown (GPUI internal) | Verify GPUI caches parsed SVGs |
| File Size | Small (~0.5-2KB each) | Acceptable |
| Path Resolution | Compile-time | Good for performance |

---

## 6. Icon System Architecture

### 6.1 Type-Safe Icon Access

```rust
pub enum IconName {
    File,
    FileCode,
    // ...
}

impl IconName {
    pub fn external_path(&self) -> &'static str { ... }
    pub fn category(&self) -> IconCategory { ... }
    pub fn name(&self) -> &'static str { ... }
    pub fn description(&self) -> &'static str { ... }
}
```

**Strengths:**
- ✅ Compile-time icon name validation
- ✅ Category grouping for organization
- ✅ Human-readable names and descriptions
- ✅ String-to-enum conversion with aliases

### 6.2 Icon Styles

```rust
pub enum IconStyle {
    Default,      // 16px
    Small,        // 12px
    Large,        // 24px
    Muted,        // 50% opacity
    Accent,       // Highlight color
    CircleBackground,
    SquareBackground,
}
```

**Strengths:**
- ✅ Standard size variants (12/16/24px)
- ✅ Opacity handling
- ✅ Background decoration options

### 6.3 String Parsing with Aliases

```rust
pub fn icon_name_from_str(name: &str) -> Option<IconName> {
    match normalized.as_str() {
        "plus" | "add" => Some(IconName::Plus),
        "trash" | "delete" | "remove" => Some(IconName::Trash),
        "magnifyingglass" | "search" | "find" => Some(IconName::MagnifyingGlass),
        // ...
    }
}
```

**Strengths:**
- ✅ Flexible input handling (kebab-case, snake_case, spaces)
- ✅ Semantic aliases (search → MagnifyingGlass)
- ✅ Returns None for unknown icons (safe fallback)

---

## 7. Recommendations Summary

### 7.1 Immediate Actions (P0)

1. **Fix Color Handling:** Convert all 19 icons with hardcoded `black` to use `currentColor`
2. **Add Missing ViewBox:** Add `viewBox="0 0 16 16"` to file.svg, file_code.svg, code.svg
3. **Add Critical Icons:** close, warning, error, info, success-circle

### 7.2 Short-Term Actions (P1)

1. **Standardize Stroke Width:** Convert terminal.svg from 1.0 to 1.2
2. **Complete Arrow Set:** Add arrow_up, arrow_left, chevron_up, chevron_left
3. **Add Common Actions:** edit, duplicate, external_link, refresh, more/ellipsis

### 7.3 Long-Term Actions (P2)

1. **Consider Icon Embedding:** Evaluate embedding SVGs in binary for faster startup
2. **Add Modifier Key Icons:** command, option, shift, control for hotkey display
3. **Add Utility Icons:** clock, calendar, user, lock, download, upload, globe, moon/sun

---

## 8. Technical Implementation Notes

### 8.1 Converting Icons to currentColor

```bash
# For each SVG file with hardcoded black:
sed -i '' 's/stroke="black"/stroke="currentColor"/g' assets/icons/*.svg
sed -i '' 's/fill="black"/fill="currentColor"/g' assets/icons/*.svg
```

**Note:** Preserve `fill="none"` and `fill-opacity` attributes.

### 8.2 Adding New Icons

1. Add SVG file to `assets/icons/`
2. Add enum variant to `IconName`
3. Update `all()`, `name()`, `description()`, `path()`, `external_path()`, `category()`
4. Add string aliases to `icon_name_from_str()`
5. Update tests

### 8.3 Icon Source Recommendations

- **Lucide Icons:** https://lucide.dev (MIT, consistent style, used by Zed)
- **Heroicons:** https://heroicons.com (MIT, clean design)
- **Phosphor Icons:** https://phosphoricons.com (MIT, comprehensive set)

---

## Appendix A: SVG File Contents

### A.1 Icons Using currentColor (Correctly)

**file_code.svg:**
```svg
<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="none">
  <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.2" d="M6.8 8.3 5.6 9.8l1.2 1.5M9.2 8.3l1.2 1.5-1.2 1.5M9.2 2v2.4a1.2 1.2 0 0 0 1.2 1.2h2.4"/>
  <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.2" d="M9.8 2H4.4a1.2 1.2 0 0 0-1.2 1.2v9.6A1.2 1.2 0 0 0 4.4 14h7.2a1.2 1.2 0 0 0 1.2-1.2V5l-3-3Z"/>
</svg>
```

**code.svg:**
```svg
<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="none">
  <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.2" d="m11.75 10.5 2.5-2.5-2.5-2.5M4.25 5.5 1.75 8l2.5 2.5M9.563 3 6.437 13"/>
</svg>
```

**file.svg:**
```svg
<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="none">
  <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.2" d="M9.875 2H4.25c-.332 0-.65.126-.884.351-.234.226-.366.53-.366.849v9.6c0 .318.132.623.366.849.235.225.552.351.884.351h7.5c.332 0 .65-.127.884-.351.234-.225.366-.53.366-.85V5L9.875 2Z"/>
  <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.2" d="M9 2v2.667A1.333 1.333 0 0 0 10.333 6H13"/>
</svg>
```

### A.2 Example Icons Needing currentColor Conversion

**magnifying_glass.svg (BEFORE):**
```svg
<svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
  <path d="M13 13L10.4138 10.4138ZM3 7.31034..." fill="black" fill-opacity="0.15"/>
  <path d="M13 13L10.4138 10.4138M3 7.31034..." stroke="black" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>
</svg>
```

**magnifying_glass.svg (AFTER - Recommended):**
```svg
<svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
  <path d="M13 13L10.4138 10.4138ZM3 7.31034..." fill="currentColor" fill-opacity="0.15"/>
  <path d="M13 13L10.4138 10.4138M3 7.31034..." stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>
</svg>
```

---

## Appendix B: Test Coverage

The icon system has comprehensive unit tests in `icon_variations.rs`:

```rust
#[test] fn test_icon_count() { assert_eq!(IconName::count(), 22); }
#[test] fn test_style_count() { assert_eq!(IconStyle::count(), 7); }
#[test] fn test_all_icons_have_paths() { /* validates .svg extension and icons/ prefix */ }
#[test] fn test_category_coverage() { /* ensures all icons belong to a category */ }
#[test] fn test_icon_name_from_str() { /* tests aliases and case handling */ }
#[test] fn test_style_sizes() { /* validates 12/16/24px sizes */ }
```

---

*End of Icon System Audit*
