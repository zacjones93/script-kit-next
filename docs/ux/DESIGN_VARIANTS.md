# Design Variants Audit Report

**Audit Date:** December 29, 2025  
**Auditor:** UX Audit Agent  
**Scope:** `src/designs/` - 15 files, ~7,500 lines of code  

---

## Executive Summary

The Script Kit GPUI design system implements a comprehensive, pluggable design variant architecture with **11 visual themes** plus supporting infrastructure. The implementation demonstrates strong architectural patterns but has inconsistent completeness across variants.

### Key Findings

| Metric | Value |
|--------|-------|
| Total Design Variants | 11 |
| Fully Implemented | 2 (Minimal, RetroTerminal) |
| Partially Implemented | 9 |
| Design Token Implementations | 11/11 |
| Renderer Implementations | 11/11 |
| Custom Renderer Active | 2/11 |
| Icon Styles | 22 icons, 7 styles |
| Separator Styles | 41 styles |
| Header Styles | 28 styles |

### Quality Scores

| Variant | Quality | Completeness | Visual Differentiation | Score |
|---------|---------|--------------|------------------------|-------|
| **Minimal** | Excellent | Full | Strong | 5/5 |
| **RetroTerminal** | Excellent | Full | Strong | 5/5 |
| **AppleHIG** | Good | High | Strong | 4/5 |
| **Material3** | Good | High | Strong | 4/5 |
| **NeonCyberpunk** | Good | High | Strong | 4/5 |
| **Glassmorphism** | Good | Medium | Strong | 3.5/5 |
| **Brutalist** | Good | Medium | Strong | 3.5/5 |
| **Paper** | Good | Medium | Good | 3.5/5 |
| **Playful** | Good | Medium | Good | 3.5/5 |
| **Compact** | Fair | Low | Moderate | 3/5 |
| **Default** | Fair | Baseline | N/A | 3/5 |

---

## Architecture Analysis

### Design Token System (traits.rs)

The design token architecture is **well-designed** and provides comprehensive theming capability:

```
DesignTokens Trait
├── colors() -> DesignColors (21 color tokens)
├── spacing() -> DesignSpacing (13 spacing tokens)
├── typography() -> DesignTypography (12 type tokens)
├── visual() -> DesignVisual (16 visual effect tokens)
├── item_height() -> f32
└── variant() -> DesignVariant
```

**Strengths:**
- All token structs implement `Copy` for efficient closure use
- Default implementations provide sensible fallbacks
- Token structs are well-documented with clear semantic naming

**Issues:**
- `DesignTypography` has `font_family` as `&'static str`, limiting runtime customization
- No support for dark/light mode variants within a single design
- `shadow` color uses inconsistent format (0xRRGGBBAA vs 0xRRGGBB)

### Renderer Trait (traits.rs)

```rust
pub trait DesignRenderer<App>: Send + Sync {
    fn render_script_list(&self, app: &App, cx: &mut Context<App>) -> AnyElement;
    fn variant(&self) -> DesignVariant;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
}
```

**Issue:** The trait doesn't pass enough state (filter, selection, items) to actually render, making custom renderers placeholder-only for most variants.

### Module Organization (mod.rs)

**Strengths:**
- Clean enum-based variant selection
- Keyboard shortcuts mapped (Cmd+1 through Cmd+0)
- `get_tokens()` provides dynamic dispatch
- `render_design_item()` handles actual list item rendering

**Issues:**
- Only Minimal and RetroTerminal have custom renderers active
- Other 9 variants fall through to default `ListItem` renderer
- Lots of commented `#[allow(dead_code)]` suggests incomplete usage

---

## Per-Variant Analysis

### 1. Default (mod.rs)

**Quality Score: 3/5**

| Aspect | Status |
|--------|--------|
| Tokens | DefaultDesignTokens (complete) |
| Item Height | 40px |
| Custom Renderer | No (uses ListItem) |

**Colors:** Standard dark theme (bg: 0x1e1e1e, text: 0xffffff, accent: 0xfbbf24)

**Issues:**
- Serves as fallback, not a distinct design
- No unique visual identity

---

### 2. Minimal (minimal.rs)

**Quality Score: 5/5**

| Aspect | Status |
|--------|--------|
| Tokens | MinimalDesignTokens (complete) |
| Item Height | 64px |
| Custom Renderer | Yes (active) |
| Line Count | 468 |

**Design Philosophy:**
- Maximum whitespace (80px horizontal, 24px vertical padding)
- NO borders, NO shadows
- Thin typography (FontWeight::THIN)
- Monochrome with single accent color
- Name-only display (no description)

**Implementation Quality:**
- Full renderer implementation with proper state handling
- Helper functions: `render_minimal_search_bar`, `render_minimal_list`, etc.
- WindowConfig struct for consistent styling
- Well-documented with clear design principles

**Strengths:**
- Complete implementation with all helper functions
- Clear separation of concerns
- Consistent application of minimal principles

---

### 3. RetroTerminal (retro_terminal.rs)

**Quality Score: 5/5**

| Aspect | Status |
|--------|--------|
| Tokens | RetroTerminalDesignTokens (complete) |
| Item Height | 28px (dense) |
| Custom Renderer | Yes (active) |
| Line Count | 686 |

**Design Philosophy:**
- Green-on-black CRT aesthetic (phosphor green: 0x00ff00)
- Monospace font (Menlo)
- Scanline effect via alternating row colors
- ASCII box characters for borders
- Blinking block cursor
- UPPERCASE text
- Inverted colors for selection

**Implementation Quality:**
- Complete renderer with ASCII art headers/footers
- Glow effects via BoxShadow
- Full standalone render functions for all components
- WindowConfig with glow settings
- Constants struct for external use

**Strengths:**
- Most distinctive visual identity
- Comprehensive helper function library
- Log panel with color-coded levels
- Preview panel with line numbers

---

### 4. Glassmorphism (glassmorphism.rs)

**Quality Score: 3.5/5**

| Aspect | Status |
|--------|--------|
| Tokens | GlassmorphismDesignTokens (complete) |
| Item Height | 56px |
| Custom Renderer | Partial |
| Line Count | 513 |

**Design Philosophy:**
- Frosted glass effect (requires WindowBackgroundAppearance::Blurred)
- Heavy transparency (0.3-0.6 alpha backgrounds)
- White/light borders with alpha
- Layered frosted panels
- Large rounded corners (16px+)

**Implementation Quality:**
- GlassColors struct with RGBA values
- `glass_card()` helper for consistent card styling
- Search bar and empty state renderers
- Window container with dual shadows

**Issues:**
- Renderer doesn't access app state (placeholder)
- List item renderer is `#[allow(dead_code)]`
- Relies on OS-level blur not always available

---

### 5. Brutalist (brutalist.rs)

**Quality Score: 3.5/5**

| Aspect | Status |
|--------|--------|
| Tokens | BrutalistDesignTokens (complete) |
| Item Height | 40px |
| Custom Renderer | Partial |
| Line Count | 405 |

**Design Philosophy:**
- Raw, anti-design aesthetic
- Georgia serif font
- 3px thick black borders
- Harsh colors: white/black/red/yellow
- NO rounded corners
- Asymmetric layouts (staggered offsets)
- ALL CAPS, underlined text

**Implementation Quality:**
- `render_brutalist_list()` helper with full styling
- Color inversion on hover (red bg)
- Asymmetric offset calculation by index
- BrutalistColors constants struct

**Issues:**
- Renderer is placeholder only
- Type label rendering incomplete
- Hard offset shadow could be more prominent

---

### 6. NeonCyberpunk (neon_cyberpunk.rs)

**Quality Score: 4/5**

| Aspect | Status |
|--------|--------|
| Tokens | NeonCyberpunkDesignTokens (complete) |
| Item Height | 40px (was incorrectly 34px in tokens) |
| Custom Renderer | Partial |
| Line Count | 617 |

**Design Philosophy:**
- Deep purple/black background (0x0a0015)
- Neon cyan (0x00ffff) and magenta (0xff00ff) accents
- Heavy glow effects via multiple box-shadows
- Monospace font (Menlo)
- Selected items: magenta glow
- Hover: cyan glow

**Implementation Quality:**
- Comprehensive color palette in module
- Static glow shadow generators (cyan_glow, magenta_glow)
- Status bar renderer
- Window components with neon styling
- NeonListItemColors struct for list rendering

**Issues:**
- Token item_height (34px) doesn't match constant (40px)
- Renderer is placeholder
- Animation effects not actually animated

---

### 7. Paper (paper.rs)

**Quality Score: 3.5/5**

| Aspect | Status |
|--------|--------|
| Tokens | PaperDesignTokens (complete) |
| Item Height | 34px |
| Custom Renderer | Partial |
| Line Count | 465 |

**Design Philosophy:**
- Warm cream/beige backgrounds (0xfaf8f0)
- Realistic drop shadows (sepia-tinted)
- Georgia serif typography
- Bookmark-style left border for selection
- Cards resembling paper notes

**Implementation Quality:**
- `create_card_shadow()` with warm sepia tones
- `create_inset_shadow()` for search box
- Paper item renderer with bookmark indicator
- Full window component set

**Issues:**
- Item height inconsistency (34px in tokens, uses LIST_ITEM_HEIGHT)
- Renderer is placeholder
- Missing texture/grain effects

---

### 8. AppleHIG (apple_hig.rs)

**Quality Score: 4/5**

| Aspect | Status |
|--------|--------|
| Tokens | AppleHIGDesignTokens (complete) |
| Item Height | 44px (iOS standard) |
| Custom Renderer | Partial |
| Line Count | 495 |

**Design Philosophy:**
- iOS grouped table view style
- SF Pro/System font
- 44px touch target (iOS standard)
- Blue accent (0x007aff)
- Grouped rounded sections
- iOS-style chevron indicators

**Implementation Quality:**
- iOS color constants (SELECTION_BG, SEPARATOR)
- `render_list_item()` with first/last rounded corners
- `render_separator()` with inset
- `render_section_header()` for group labels
- Full window component set

**Issues:**
- Renderer uses demo content, not app state
- Missing SFSymbols integration
- No grouped list virtualization support

---

### 9. Material3 (material3.rs)

**Quality Score: 4/5**

| Aspect | Status |
|--------|--------|
| Tokens | Material3DesignTokens (complete) |
| Item Height | 56px (M3 list item) |
| Custom Renderer | Partial |
| Line Count | 505 |

**Design Philosophy:**
- M3 tonal surfaces (purple/lavender palette)
- Rounded corners (12px cards, 28px search pill)
- Elevation shadows
- Pill-shaped chips for metadata
- Roboto-style typography

**Implementation Quality:**
- Color tokens module with M3 palette
- Corner radius tokens (XS to Full)
- Elevation tokens (Level 0-3)
- Chip renderer for shortcuts
- Search bar with pill styling

**Issues:**
- Renderer uses hardcoded demo items
- Missing M3 state layers (hover opacity)
- No ripple effects
- Font should be Roboto, uses system font

---

### 10. Compact (compact.rs)

**Quality Score: 3/5**

| Aspect | Status |
|--------|--------|
| Tokens | CompactDesignTokens (complete) |
| Item Height | 24px |
| Custom Renderer | Partial |
| Line Count | 344 |

**Design Philosophy:**
- Maximum information density
- 10px font (text_xs)
- Minimal padding (4px horizontal, 2px vertical)
- Monospace font for consistent width
- Table-like borders between rows
- No preview panel

**Implementation Quality:**
- CompactListItem component with builder pattern
- Description truncation (40 chars)
- Border-bottom separators
- Minimal header (28px)

**Issues:**
- Requires theme Arc, unlike other variants
- Renderer is placeholder only
- No keyboard navigation hints
- Could be denser (currently 24px, could be 20px)

---

### 11. Playful (playful.rs)

**Quality Score: 3.5/5**

| Aspect | Status |
|--------|--------|
| Tokens | PlayfulDesignTokens (complete) |
| Item Height | 56px |
| Custom Renderer | Partial |
| Line Count | 603 |

**Design Philosophy:**
- Very rounded corners (24px+)
- Coral/mint/lavender palette
- Emoji integration (letter-based emoji mapping)
- Sparkle indicators for selection
- Pill-shaped badges
- Bouncy, friendly borders (2px)

**Implementation Quality:**
- `get_emoji_for_name()` function (A-Z mapping)
- PlayfulColors struct
- Search bar with fun placeholder
- Empty state with playful messaging
- Log panel with rotating colors

**Issues:**
- Renderer is placeholder
- Emoji mapping could be configurable
- Missing actual animation support

---

## Supporting Infrastructure

### Icon Variations (icon_variations.rs)

**Quality Score: 4/5**

| Metric | Value |
|--------|-------|
| Icon Count | 22 |
| Style Count | 7 |
| Categories | 5 |
| Line Count | 569 |

**Icon Categories:**
- Files (4): File, FileCode, Folder, FolderOpen
- Actions (7): Plus, Trash, Copy, Settings, MagnifyingGlass, Terminal, Code
- Status (5): Check, Star, StarFilled, BoltFilled, BoltOutlined
- Arrows (4): ArrowRight, ArrowDown, ChevronRight, ChevronDown
- Media (2): PlayFilled, PlayOutlined

**Icon Styles:**
- Default (16px)
- Small (12px)
- Large (24px)
- Muted (50% opacity)
- Accent (highlight color)
- CircleBackground
- SquareBackground

**Strengths:**
- `icon_name_from_str()` with alias support
- External path generation for GPUI svg()
- Comprehensive test coverage

**Issues:**
- Some icons may be missing SVG files
- No support for custom/user icons

---

### Group Header Variations (group_header_variations.rs)

**Quality Score: 4/5**

| Metric | Value |
|--------|-------|
| Style Count | 28 |
| Categories | 5 |
| Line Count | 334 |

**Categories:**
- TextOnly (6): UppercaseLeft/Center, SmallCaps, Bold, Light, Monospace
- WithLines (6): LineLeft/Right/BothSides/Below/Above, DoubleLine
- WithBackground (5): Pill, FullWidth, Subtle, Gradient, Bordered
- Minimal (6): Dot/Dash/Bullet/Arrow/Chevron prefix, Dimmed
- Decorative (5): Bracketed, Quoted, Tagged, Numbered, IconPrefix

**Strengths:**
- `sample()` function for text preview
- Category-to-style mapping
- Comprehensive descriptions

**Issues:**
- No actual rendering implementations
- Just data structures, no GPUI elements

---

### Separator Variations (separator_variations.rs)

**Quality Score: 4.5/5**

| Metric | Value |
|--------|-------|
| Style Count | 41 |
| Categories | 8 |
| Line Count | 1221 |

**Categories:**
- LineBased (7): Solid, Dotted, Dashed, Double, Hairline, Thick, FadeEdges
- Typographic (5): Uppercase, SmallCaps, Italic, Bold, Underlined
- Decorative (6): Chevron, Dots, Diamond, Bracket, Arrow, Star
- SpacingBased (4): LargeGap, TightGap, Indented, HangingIndent
- Background (4): SubtleFill, Gradient, Frosted, Pill
- Minimalist (5): Invisible, SingleDot, Pipe, Colon, Slash
- Retro (5): AsciiBox, BoxDrawing, TerminalPrompt, DOS, Typewriter
- Modern (5): AnimatedFade, BlurOverlay, NeonGlow, GlassCard, Floating

**Strengths:**
- `SeparatorConfig` struct with 20+ parameters
- `default_config()` per style
- `decorations()` returns prefix/suffix strings
- `is_compatible_with()` checks design variant compatibility
- `recommended_for()` returns best matches per variant

**Issues:**
- No actual rendering implementations
- Config system not connected to renderers

---

## Cross-Variant Consistency Issues

### 1. Item Height Inconsistencies

| Variant | Token Height | Constant | Actual |
|---------|-------------|----------|--------|
| Default | 40px | LIST_ITEM_HEIGHT | 40px |
| Minimal | 64px | MINIMAL_ITEM_HEIGHT | 64px |
| RetroTerminal | 28px | TERMINAL_ITEM_HEIGHT | 28px |
| Compact | 24px | COMPACT_ITEM_HEIGHT | 24px |
| NeonCyberpunk | 34px | LIST_ITEM_HEIGHT (40px) | Mismatch |
| Paper | 34px | LIST_ITEM_HEIGHT (40px) | Mismatch |

**Recommendation:** Align token `item_height()` with actual constants.

### 2. Renderer State Access Pattern

Most renderers are placeholder because they can't access:
- Filtered results list
- Selected index
- Filter text
- Hover state

**Recommendation:** Modify `DesignRenderer` trait:
```rust
fn render_script_list(
    &self, 
    results: &[SearchResult],
    selected_index: usize,
    filter: &str,
    cx: &mut Context<App>
) -> AnyElement;
```

### 3. Color Format Inconsistency

| Location | Format |
|----------|--------|
| Most colors | 0xRRGGBB |
| Glassmorphism backgrounds | 0xRRGGBBAA |
| Shadow colors | hsla() |
| Some borders | rgba((hex << 8) | alpha) |

**Recommendation:** Standardize on 0xRRGGBB + separate alpha, or provide consistent helpers.

### 4. Font Family Variations

| Variant | Primary Font |
|---------|-------------|
| Default | .AppleSystemUIFont |
| Minimal | .AppleSystemUIFont |
| RetroTerminal | Menlo |
| Brutalist | Georgia |
| Compact | Menlo |
| Paper | Georgia |
| AppleHIG | .AppleSystemUIFont |
| Material3 | .AppleSystemUIFont (should be Roboto) |
| NeonCyberpunk | Menlo |
| Glassmorphism | .AppleSystemUIFont |
| Playful | .AppleSystemUIFont |

**Issue:** No font fallback chains, platform-specific fonts may not exist.

### 5. Missing Standalone Functions

| Variant | Has Standalone Renders |
|---------|----------------------|
| Minimal | Yes (6 functions) |
| RetroTerminal | Yes (7 functions) |
| Glassmorphism | Yes (4 functions) |
| Brutalist | Yes (4 functions) |
| NeonCyberpunk | Yes (4 functions) |
| Paper | Yes (4 functions) |
| AppleHIG | Yes (4 functions) |
| Material3 | Yes (4 functions) |
| Compact | Yes (4 functions) |
| Playful | Yes (4 functions) |
| Default | No (uses ListItem) |

---

## Variant Switching UX

### Current Implementation

- Keyboard shortcuts: Cmd+1 through Cmd+0 (10 variants)
- Playful has no shortcut (only via cycling)
- `next()` / `prev()` methods for cycling
- No visual feedback during switch
- Immediate switch, no animation

### Issues

1. **No transition animation** - Jarring switch between vastly different designs
2. **Playful unreachable** - No keyboard shortcut assigned
3. **No preview** - Users must switch to see what a design looks like
4. **No persistence** - Design resets on restart

### Recommendations

1. Add design picker dialog with previews
2. Persist selection to config.ts
3. Consider transition animations for Modern category
4. Add Cmd+Shift+D for design picker

---

## Recommendations

### Priority 1: Critical Fixes

1. **Complete renderer integration** - Connect app state to renderers
2. **Fix item height mismatches** - Align tokens with constants
3. **Persist design selection** - Save to config.ts

### Priority 2: Quality Improvements

1. **Add design preview dialog** - Show thumbnails before switching
2. **Implement separator rendering** - Use separator_variations configs
3. **Add header style rendering** - Use group_header_variations
4. **Font fallback chains** - Handle missing fonts gracefully

### Priority 3: Enhancements

1. **Light/dark mode per variant** - Focus-aware already exists, extend to mode
2. **Custom color overrides** - Per-variant theme.json support
3. **Transition animations** - Smooth design switching
4. **Icon theme matching** - Icons should respect variant colors

### Priority 4: Polish

1. **Brutalist asymmetry** - Make more pronounced
2. **Glassmorphism blur** - Graceful fallback when OS blur unavailable
3. **Playful animations** - Add actual bouncy animations
4. **NeonCyberpunk glow** - Add pulsing animation

---

## Appendix: Line Count Summary

| File | Lines | Purpose |
|------|-------|---------|
| mod.rs | 737 | Module coordination, dispatch |
| traits.rs | 1663 | Token system, trait definitions |
| separator_variations.rs | 1221 | 41 separator styles |
| retro_terminal.rs | 686 | Terminal design |
| neon_cyberpunk.rs | 617 | Cyberpunk design |
| playful.rs | 603 | Playful design |
| icon_variations.rs | 569 | 22 icons, 7 styles |
| glassmorphism.rs | 513 | Glass design |
| material3.rs | 505 | M3 design |
| apple_hig.rs | 495 | iOS design |
| minimal.rs | 468 | Minimal design |
| paper.rs | 465 | Paper design |
| brutalist.rs | 405 | Brutalist design |
| compact.rs | 344 | Compact design |
| group_header_variations.rs | 334 | 28 header styles |
| **TOTAL** | **~9,625** | |

---

## Test Coverage

| File | Has Tests | Coverage |
|------|-----------|----------|
| mod.rs | Yes | 12 tests (variants, cycling, tokens) |
| traits.rs | No | Implicit via mod.rs tests |
| separator_variations.rs | Yes | 12 tests |
| icon_variations.rs | Yes | 8 tests |
| group_header_variations.rs | Yes | 5 tests |
| Individual variants | No | None |

**Recommendation:** Add snapshot tests for each variant's rendered output.

---

*Report generated as part of comprehensive UX audit. See also: THEME_SYSTEM.md, COMPONENT_LIBRARY.md*
