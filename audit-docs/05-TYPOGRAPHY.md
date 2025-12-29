# Typography and Text Hierarchy Audit

## Executive Summary

Script Kit GPUI uses a **partially formalized typography system** with configurable font sizes for editor and terminal, but **lacks a unified type scale** across the UI. The codebase has:

- **Good**: Configurable font sizes via `config.rs`, design token system in `designs/traits.rs`
- **Needs Improvement**: Inconsistent font size usage across components, no unified type scale, hardcoded values scattered throughout

## 1. Configuration System

### 1.1 Config Settings (`src/config.rs`)

| Setting | Default | Description |
|---------|---------|-------------|
| `editorFontSize` | 14.0px | Font size for code editor |
| `terminalFontSize` | 14.0px | Font size for terminal |
| `uiScale` | 1.0 | Global UI scale multiplier (NOT widely applied) |

**Constants:**
```rust
pub const DEFAULT_EDITOR_FONT_SIZE: f32 = 14.0;
pub const DEFAULT_TERMINAL_FONT_SIZE: f32 = 14.0;
pub const DEFAULT_UI_SCALE: f32 = 1.0;
```

**Usage via getters:**
```rust
config.get_editor_font_size()    // Returns f32
config.get_terminal_font_size()  // Returns f32
config.get_ui_scale()            // Returns f32 (currently unused in most components)
```

### 1.2 Design Token Typography (`src/designs/traits.rs`)

The design system defines a formal `DesignTypography` struct:

```rust
pub struct DesignTypography {
    // Font families
    pub font_family: &'static str,      // ".AppleSystemUIFont"
    pub font_family_mono: &'static str, // "Menlo"

    // Font sizes (in pixels)
    pub font_size_xs: f32,    // 10.0
    pub font_size_sm: f32,    // 12.0
    pub font_size_md: f32,    // 14.0 (base)
    pub font_size_lg: f32,    // 16.0
    pub font_size_xl: f32,    // 20.0
    pub font_size_title: f32, // 24.0

    // Font weights
    pub font_weight_thin: FontWeight,     // THIN (100)
    pub font_weight_light: FontWeight,    // LIGHT (300)
    pub font_weight_normal: FontWeight,   // NORMAL (400)
    pub font_weight_medium: FontWeight,   // MEDIUM (500)
    pub font_weight_semibold: FontWeight, // SEMIBOLD (600)
    pub font_weight_bold: FontWeight,     // BOLD (700)

    // Line heights (as multipliers)
    pub line_height_tight: f32,   // 1.2
    pub line_height_normal: f32,  // 1.5
    pub line_height_relaxed: f32, // 1.75
}
```

## 2. Font Size Scale Analysis

### 2.1 Design Token Scale (Default)

| Token | Size | Ratio from Base | Usage Intent |
|-------|------|-----------------|--------------|
| `font_size_xs` | 10px | 0.71x | Shortcuts, metadata, badges |
| `font_size_sm` | 12px | 0.86x | Descriptions, secondary text |
| `font_size_md` | 14px | 1.0x (base) | Body text, list items |
| `font_size_lg` | 16px | 1.14x | Headlines, selected items |
| `font_size_xl` | 20px | 1.43x | Section titles |
| `font_size_title` | 24px | 1.71x | Page titles |

**Note:** This is NOT a consistent modular scale. Ratios vary: 1.14, 1.18, 1.25, 1.20

### 2.2 Recommended Type Scale (1.25 ratio - not currently implemented)

| Step | Size | Purpose |
|------|------|---------|
| xs | 12px | Badges, metadata |
| sm | 14px | Descriptions |
| base | 16px | Body text |
| lg | 20px | Subheadings |
| xl | 24px | Headings |
| 2xl | 30px | Page titles |

### 2.3 Design Variant Typography Comparison

| Variant | Base Size | Weight Strategy | Notes |
|---------|-----------|-----------------|-------|
| Default | 14px | Standard weights | Uses all weight levels |
| Minimal | 16px | Thin/Light preference | `font_weight_normal = THIN` |
| RetroTerminal | 13px | All NORMAL/BOLD | Monospace only |
| Compact | 11px | Smaller scale (9-16px) | Power user density |
| Brutalist | 14px | Bold emphasis | `font_weight_bold = BLACK` |
| AppleHIG | 15px | iOS Dynamic Type | Follows Apple guidelines |
| Material3 | 14px | Medium preference | Google M3 scale |
| Paper | 14px | Serif font (Georgia) | Print-inspired |
| Playful | 14px | Friendly weights | Warm, approachable |

## 3. Actual Usage in Components

### 3.1 List Item (`src/list_item.rs`)

```rust
// Line 372-380: Name rendering
.text_size(px(14.))          // HARDCODED - not using tokens
.font_weight(FontWeight::MEDIUM)
.line_height(px(18.))        // 1.29 multiplier

// Line 385-395: Description rendering  
.text_size(px(12.))          // HARDCODED
.line_height(px(14.))        // 1.17 multiplier

// Line 401-407: Shortcut badge
.text_size(px(11.))          // HARDCODED

// Section header (line 607-613)
.text_xs()                   // Uses GPUI utility (approx 10-11px)
.font_weight(FontWeight::BOLD)
```

**Issues:**
- Uses hardcoded pixel values instead of design tokens
- Line height varies (1.17, 1.29) - no consistency with token system

### 3.2 Editor Prompt (`src/editor.rs`)

```rust
// Constants (lines 34-38)
const BASE_CHAR_WIDTH: f32 = 8.4;
const BASE_FONT_SIZE: f32 = 14.0;
const LINE_HEIGHT_MULTIPLIER: f32 = 1.43;  // 20/14 ratio

// Dynamic font size from config (lines 226-231)
fn font_size(&self) -> f32 {
    self.config.get_editor_font_size()  // Reads from config
}

fn line_height(&self) -> f32 {
    self.font_size() * LINE_HEIGHT_MULTIPLIER  // Dynamic calculation
}

// Status bar text (lines 1035-1052)
.text_xs()                   // Uses GPUI utility
.font_family("Menlo")
```

**Good:** Uses config-driven font size
**Issue:** Status bar uses GPUI utility while main editor uses custom calculation

### 3.3 Terminal Prompt (`src/term_prompt.rs`)

```rust
// Constants (lines 20-32)
const BASE_FONT_SIZE: f32 = 14.0;
const LINE_HEIGHT_MULTIPLIER: f32 = 1.3;  // Different from editor!
const BASE_CELL_WIDTH: f32 = 8.5;
const BASE_CELL_HEIGHT: f32 = BASE_FONT_SIZE * LINE_HEIGHT_MULTIPLIER;  // 18.2px

// Dynamic font size from config (lines 129-142)
fn font_size(&self) -> f32 {
    self.config.get_terminal_font_size()
}

fn cell_height(&self) -> f32 {
    self.font_size() * LINE_HEIGHT_MULTIPLIER
}
```

**Good:** Config-driven, consistent internal calculations
**Issue:** Different LINE_HEIGHT_MULTIPLIER (1.3) than editor (1.43)

### 3.4 Prompts (`src/prompts.rs`)

Uses design tokens via `get_tokens()`:

```rust
// Line 242: Gets design tokens
let tokens = get_tokens(self.design_variant);
let spacing = tokens.spacing();

// Line 350-354: Choice name
.text_base()                 // GPUI utility (16px)

// Line 362-364: Choice description
.text_sm()                   // GPUI utility (14px)
```

**Good:** Uses design token system for spacing
**Issue:** Uses GPUI text utilities instead of token font sizes

## 4. Font Weight Usage

### 4.1 Weight Distribution in Codebase

| Weight | Usage Locations |
|--------|-----------------|
| THIN | Minimal design default |
| LIGHT | Some designs |
| NORMAL | Terminal text, body text |
| MEDIUM | List item names, selected states |
| SEMIBOLD | Section headers (via BOLD) |
| BOLD | Section headers, shortcuts |
| BLACK | Brutalist design emphasis |

### 4.2 Actual Usage Patterns

```rust
// List items
.font_weight(FontWeight::MEDIUM)  // Names
.font_weight(FontWeight::BOLD)    // Section headers

// Editor
// (No explicit font weight - uses font default)

// Terminal  
.font_weight(gpui::FontWeight::BOLD)  // For BOLD attribute cells
```

## 5. Line Height Analysis

### 5.1 Defined Line Heights

| Component | Multiplier | Notes |
|-----------|------------|-------|
| Editor | 1.43 | Based on 20/14 ratio |
| Terminal | 1.3 | Room for descenders |
| Design Tokens | 1.2/1.5/1.75 | tight/normal/relaxed |
| List Items | Varies | 18px for 14px text (1.29) |

### 5.2 Inconsistency Issues

- **Editor vs Terminal**: 1.43 vs 1.3 - different multipliers
- **List items**: Hardcoded `line_height(px(18.))` for 14px text
- **Token system**: Defines 1.2/1.5/1.75 but rarely used

## 6. Text Color Hierarchy

### 6.1 Theme Color Definitions (`src/theme.rs`)

```rust
pub struct TextColors {
    pub primary: HexColor,   // 0xffffff (white) - headings, names
    pub secondary: HexColor, // 0xcccccc - body text, descriptions
    pub tertiary: HexColor,  // 0x999999 - hints
    pub muted: HexColor,     // 0x808080 - placeholders
    pub dimmed: HexColor,    // 0x666666 - disabled, metadata
}
```

### 6.2 Design Token Colors (`src/designs/traits.rs`)

```rust
pub struct DesignColors {
    pub text_primary: u32,    // 0xffffff - headings, names
    pub text_secondary: u32,  // 0xcccccc - descriptions, labels
    pub text_muted: u32,      // 0x808080 - placeholders, hints
    pub text_dimmed: u32,     // 0x666666 - disabled, inactive
    pub text_on_accent: u32,  // 0x000000 - text on colored backgrounds
}
```

### 6.3 Actual Usage in Components

**List Item (`list_item.rs`):**
```rust
// Selected state
rgb(colors.text_primary)     // Name when selected

// Unselected state  
rgb(colors.text_secondary)   // Name when not selected
rgb(colors.text_muted)       // Description (always)
rgb(colors.text_dimmed)      // Shortcuts/badges
```

**Editor (`editor.rs`):**
```rust
rgb(colors.text.primary)     // Code text
rgb(colors.text.muted)       // Line numbers
rgb(colors.text.secondary)   // Status bar text
```

## 7. Font Family Usage

### 7.1 System Fonts

| Font | Purpose | Files |
|------|---------|-------|
| `.AppleSystemUIFont` | UI text | list_item.rs, prompts.rs |
| `Menlo` | Code, terminal | editor.rs, term_prompt.rs |

### 7.2 Design Variant Fonts

| Variant | UI Font | Mono Font |
|---------|---------|-----------|
| Default | .AppleSystemUIFont | Menlo |
| Brutalist | Helvetica Neue | Courier |
| Paper | Georgia | Courier New |
| Compact | .AppleSystemUIFont | SF Mono |
| AppleHIG | .AppleSystemUIFont | SF Mono |
| RetroTerminal | Menlo | Menlo |

## 8. Issues and Recommendations

### 8.1 Critical Issues

1. **Hardcoded Values**: List items use `px(14.)`, `px(12.)`, `px(11.)` instead of design tokens
2. **Inconsistent Line Heights**: Editor (1.43) vs Terminal (1.3) vs Tokens (1.2/1.5/1.75)
3. **Mixed Font Size Sources**: Some use config, some use tokens, some use GPUI utilities
4. **UI Scale Unused**: `config.get_ui_scale()` exists but is rarely applied

### 8.2 Recommendations

1. **Unify to Design Token System**
   - Replace hardcoded `px(14.)` with `px(tokens.typography().font_size_md)`
   - Use token line heights consistently

2. **Standardize Line Height**
   - Pick consistent multipliers: 1.4 for UI text, 1.5 for readable content
   - Apply to all components

3. **Implement Modular Scale**
   - Adopt 1.25 ratio: 12, 14, 16, 20, 24, 30
   - Update `DesignTypography` defaults

4. **Apply UI Scale Globally**
   - Multiply all font sizes by `config.get_ui_scale()`
   - Enable accessibility scaling

5. **Reduce Font Families**
   - System UI: `.AppleSystemUIFont` (already correct)
   - Monospace: `Menlo` as default, `SF Mono` for Apple HIG

### 8.3 Refactoring Priority

| Priority | Task | Files Affected |
|----------|------|----------------|
| High | Replace hardcoded sizes in list_item.rs | list_item.rs |
| High | Unify line height multipliers | editor.rs, term_prompt.rs |
| Medium | Apply design tokens in prompts.rs | prompts.rs |
| Medium | Implement UI scale multiplier | All render methods |
| Low | Adopt modular type scale | designs/traits.rs |

## 9. Typography Token Quick Reference

### 9.1 Current Token Values (Default)

```rust
DesignTypography {
    font_family: ".AppleSystemUIFont",
    font_family_mono: "Menlo",
    
    font_size_xs: 10.0,
    font_size_sm: 12.0,
    font_size_md: 14.0,  // BASE
    font_size_lg: 16.0,
    font_size_xl: 20.0,
    font_size_title: 24.0,
    
    font_weight_thin: THIN,      // 100
    font_weight_light: LIGHT,    // 300
    font_weight_normal: NORMAL,  // 400
    font_weight_medium: MEDIUM,  // 500
    font_weight_semibold: SEMIBOLD, // 600
    font_weight_bold: BOLD,      // 700
    
    line_height_tight: 1.2,
    line_height_normal: 1.5,
    line_height_relaxed: 1.75,
}
```

### 9.2 Suggested Usage Guidelines

| Element | Size | Weight | Line Height | Color |
|---------|------|--------|-------------|-------|
| Page Title | title (24px) | semibold | tight | primary |
| Section Header | xs (10px) | bold | tight | dimmed |
| List Item Name | md (14px) | medium | tight | secondary/primary |
| Description | sm (12px) | normal | tight | muted |
| Badge/Shortcut | xs (10px) | normal | - | dimmed |
| Editor Code | config | normal | 1.43 | primary |
| Terminal | config | normal | 1.3 | primary |
| Status Bar | xs | normal | - | secondary |

---

*Audit completed: 2024-12-29*
*Files analyzed: config.rs, list_item.rs, prompts.rs, editor.rs, term_prompt.rs, theme.rs, designs/traits.rs*
