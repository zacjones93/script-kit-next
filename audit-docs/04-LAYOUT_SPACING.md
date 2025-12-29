# Layout & Spacing Audit Report

**Generated:** 2024-12-29  
**Agent:** LayoutAuditor  
**Files Analyzed:** `src/main.rs`, `src/prompts.rs`, `src/list_item.rs`, `src/actions.rs`, `src/editor.rs`

---

## Executive Summary

The codebase shows **moderate consistency** in layout patterns, with a well-defined design token system (`designs/` module) that provides centralized spacing values. However, there are **legacy hardcoded values** scattered throughout `main.rs` that don't use the token system, and **inconsistent gap/padding values** across different components.

### Key Findings

| Metric | Status | Notes |
|--------|--------|-------|
| Flexbox Pattern Consistency | ✅ Good | Consistent `flex().flex_col()/flex_row()` usage |
| Design Token Adoption | ⚠️ Partial | New code uses tokens, legacy code has hardcoded values |
| Spacing Scale Consistency | ⚠️ Partial | 4px-based scale exists but not universally applied |
| Gap Value Consistency | ❌ Poor | Ad-hoc gap values (2, 3, 4, 8, 12) scattered |
| Padding Consistency | ⚠️ Partial | Mix of `px()` and design tokens |
| Alignment Patterns | ✅ Good | Consistent use of `items_center`, `justify_center` |

---

## 1. Flexbox Patterns Analysis

### Standard Pattern (Correct)

The codebase follows the recommended GPUI layout order:
```rust
div()
    // 1. Layout direction
    .flex()
    .flex_col()           // or .flex_row()
    
    // 2. Sizing
    .w_full()
    .h_full() / .h(px(N))
    
    // 3. Spacing
    .px(px(16.))
    .py(px(8.))
    .gap_N() / .gap(px(N))
    
    // 4. Visual styling
    .bg(rgb(...))
    .border_color(rgb(...))
    .rounded(px(N))
    
    // 5. Children
    .child(...)
```

### Pattern Variants Found

#### Container Patterns

| Pattern | Usage | Files |
|---------|-------|-------|
| `flex().flex_col().w_full().h_full()` | Full-size vertical container | main.rs, prompts.rs, editor.rs |
| `flex().flex_row().items_center()` | Horizontal centered row | All files |
| `flex().flex_col().flex_1().min_h(px(0.))` | Growing vertical container | main.rs, prompts.rs, actions.rs |
| `flex().flex_row().justify_between()` | Space-between row | actions.rs, editor.rs |

#### Common Anti-Patterns

```rust
// ❌ ANTI-PATTERN: Chaining without grouping
div().flex().flex_row().gap_2().items_center().px(px(12.)).py(px(6.)).text_color(...)

// ✅ PREFERRED: Grouped by concern
div()
    .flex()
    .flex_row()
    .items_center()      // Layout
    .gap_2()              // Spacing
    .px(px(12.))
    .py(px(6.))
    .text_color(...)     // Visual
```

---

## 2. Spacing Scale Analysis

### Design Token Spacing Scale (from `designs/` module)

The design system defines a spacing scale via `DesignSpacing`:

```rust
pub struct DesignSpacing {
    pub padding_xs: f32,      // 4.0 (base unit)
    pub padding_sm: f32,      // 8.0
    pub padding_md: f32,      // 12.0
    pub padding_lg: f32,      // 16.0
    pub padding_xl: f32,      // 24.0
    pub gap_sm: f32,          // 4.0
    pub gap_md: f32,          // 8.0
    pub gap_lg: f32,          // 12.0
    pub item_padding_x: f32,  // 12.0
    pub item_padding_y: f32,  // 8.0
    pub margin_lg: f32,       // 16.0
}
```

**Base Unit: 4px** - This is the correct foundation for a spacing scale.

### Expected 4-Point Grid Scale

| Token | Expected (4px grid) | Actual | Status |
|-------|---------------------|--------|--------|
| xs | 4 | 4.0 | ✅ |
| sm | 8 | 8.0 | ✅ |
| md | 12 | 12.0 | ✅ |
| lg | 16 | 16.0 | ✅ |
| xl | 24 | 24.0 | ✅ |
| 2xl | 32 | ❌ Missing | Add |
| 3xl | 40 | ❌ Missing | Add |

---

## 3. Hardcoded Pixel Values Inventory

### main.rs (Highest Priority)

| Line Range | Value | Context | Should Be |
|------------|-------|---------|-----------|
| ~4779 | `12.0` | border_radius | `design_visual.radius_lg` |
| ~4798-4810 | `12.0` | border_radius fallback | Use tokens consistently |
| ~4815-4819 | `16.0, 8.0, 12.0` | header padding | Already uses conditional tokens |
| ~4848-4859 | `2.0` (border_normal) | cursor width | `design_visual.border_normal` ✅ |
| ~4884 | `28.` | header height | Should be token |
| ~4913-4917 | `4.` | button margin | `design_spacing.padding_xs` |
| ~4961-4969 | `120.0, 20.0` | actions search input | Should be constants or tokens |
| ~5043-5055 | `0.` | min_h for flex | Correct pattern |
| ~5077-5078 | `8.` | actions overlay padding | `design_spacing.padding_sm` |

### prompts.rs

| Line | Value | Context | Should Be |
|------|-------|---------|-----------|
| 280-302 | Uses `spacing.item_padding_x`, `spacing.padding_md` | Input container | ✅ Already using tokens |
| 306-313 | Uses `spacing.*` | Choices container | ✅ Already using tokens |
| 318-322 | Uses `spacing.padding_xl`, `spacing.item_padding_x` | Empty state | ✅ Already using tokens |
| 340-346 | Uses `spacing.item_padding_x`, `spacing.item_padding_y` | Choice items | ✅ Already using tokens |
| 514 | Uses `spacing.padding_lg` | DivPrompt padding | ✅ Already using tokens |

**prompts.rs is a model for token usage!**

### list_item.rs

| Line | Value | Context | Should Be |
|------|-------|---------|-----------|
| 27 | `40.0` | LIST_ITEM_HEIGHT | Keep as constant (used for virtualization) |
| 32 | `24.0` | SECTION_HEADER_HEIGHT | Keep as constant |
| 144 | `3.0` | ACCENT_BAR_WIDTH | Keep as constant |
| 291-295 | `20.` | Icon container | Should be constant `ICON_SIZE` |
| 305-306 | `20.` | Image container | Should use same constant |
| 315-316 | `16.` | SVG size | Should be constant |
| 374 | `14.` | Name font size | Should be typography token |
| 379 | `18.` | Name line height | Should be typography token |
| 389-390 | `12.`, `14.` | Description font/line | Should be typography tokens |
| 402-407 | `11.`, `6.`, `2.`, `3.` | Shortcut badge | Should be tokens |
| 429 | `12.` | Item horizontal padding | `design_spacing.item_padding_x` |
| 430 | `6.` | Item vertical padding | `design_spacing.padding_xs + 2` |
| 438 | `8.` | Gap between elements | `design_spacing.gap_md` |
| 477 | `4.` | Right padding | `design_spacing.padding_xs` |
| 599-604 | `16.`, `4.` | Section header padding | Should use tokens |

### actions.rs

| Line | Value | Context | Recommendation |
|------|-------|---------|----------------|
| 155-163 | Constants defined | `POPUP_WIDTH`, `ACTION_ITEM_HEIGHT`, etc. | ✅ Good - keep as module constants |
| 515-524 | `44.0`, `spacing.item_padding_x` | Input row | Mix of constant and token |
| 519-524 | Uses `spacing.item_padding_y + 2.0` | Padding adjustment | Should define in spacing |
| 534-538 | `24.0` | Icon container width | Should be constant |
| 548-551 | `240.0`, `28.0` | Search input size | Should be constants |
| 584-600 | `2.`, `16.`, `2.` | Cursor dimensions | Should be constants |
| 743 | `ACTION_ITEM_HEIGHT` | Item height | ✅ Uses constant |
| 762-767 | `ACCENT_BAR_WIDTH` | Accent bar | ✅ Uses constant |
| 797-800 | `6.`, `2.`, `4.` | Shortcut pill | Should be tokens |

### editor.rs

| Line | Value | Context | Recommendation |
|------|-------|---------|----------------|
| 34-42 | Constants defined | `BASE_CHAR_WIDTH`, `LINE_HEIGHT_MULTIPLIER`, `GUTTER_WIDTH` | ✅ Good - domain-specific |
| 1006-1009 | `2.0`, cursor height calculation, `2.0` | Cursor | OK - derived values |
| 1021 | `28.` | Status bar height | Should be constant |
| 1078 | `28.0` | STATUS_BAR_HEIGHT | ✅ Already a constant |

---

## 4. Gap Usage Analysis

### Current Gap Values Found

| Value | Usage Count | Context |
|-------|-------------|---------|
| `gap_1()` | 1 | prompts.rs choice item |
| `gap_2()` | 8 | Various row layouts |
| `gap_3()` | 0 | Not used |
| `gap_4()` | 1 | editor.rs status bar |
| `gap(px(N))` | 12 | Custom gap values |

### Gap Value Mapping

| GPUI Method | Pixel Value | Design Token Equivalent |
|-------------|-------------|------------------------|
| `gap_1()` | 4px | `gap_sm` |
| `gap_2()` | 8px | `gap_md` |
| `gap_3()` | 12px | `gap_lg` |
| `gap_4()` | 16px | (missing) |
| `gap(px(8.))` | 8px | `gap_md` |
| `gap(px(12.))` | 12px | `gap_lg` |

### Recommendation

**Prefer design tokens over GPUI gap methods:**
```rust
// ❌ Avoid
.gap_2()

// ✅ Prefer  
.gap(px(spacing.gap_md))
```

This makes spacing relationships explicit and allows global adjustment.

---

## 5. Padding/Margin Analysis

### Horizontal Padding (px)

| Value | Count | Context |
|-------|-------|---------|
| `px(4.)` | 5 | Small spacing (buttons, margins) |
| `px(6.)` | 4 | Item padding (py), shortcut pills |
| `px(8.)` | 8 | Medium spacing |
| `px(12.)` | 15 | Common item padding |
| `px(16.)` | 12 | Container padding |
| `px(24.)` | 2 | Large spacing |

### Vertical Padding (py)

| Value | Count | Context |
|-------|-------|---------|
| `py(px(2.))` | 3 | Shortcut badges |
| `py(px(6.))` | 5 | List item padding |
| `py(px(8.))` | 8 | Common vertical padding |

### Observations

1. **Item Padding Consistency:** Most list items use `px(12.)` horizontal, `py(px(6.))` or `py(px(8.))` vertical
2. **Container Padding:** Consistently `px(16.)` 
3. **Small Spacing:** `px(4.)` used for gaps and margins

---

## 6. Alignment Pattern Analysis

### Horizontal Alignment

| Pattern | Usage | Purpose |
|---------|-------|---------|
| `items_center()` | 50+ | Vertical centering in rows |
| `items_start()` | 3 | Top alignment |
| `items_end()` | 2 | Bottom alignment |

### Vertical Alignment

| Pattern | Usage | Purpose |
|---------|-------|---------|
| `justify_center()` | 12 | Center content |
| `justify_between()` | 6 | Space between |
| `justify_end()` | 8 | Right/bottom align |

### Common Combined Patterns

```rust
// Centered content
.flex().items_center().justify_center()

// Space-between row
.flex().flex_row().items_center().justify_between()

// Top-aligned column
.flex().flex_col().items_start()
```

---

## 7. Inconsistencies Summary

### Critical Issues

1. **Mixed Token/Hardcoded Values in main.rs**
   - Header section uses conditional tokens (`is_default_design ? theme : design_colors`)
   - But then uses hardcoded values like `28.`, `4.`, `120.0`
   
2. **list_item.rs Doesn't Use Design Tokens**
   - All spacing is hardcoded (`12.`, `6.`, `8.`, `14.`, etc.)
   - Should accept spacing from design tokens

3. **actions.rs Mix of Constants and Hardcoded**
   - Has good module constants (`POPUP_WIDTH`, etc.)
   - But still has hardcoded values for padding and sizes

### Medium Priority

4. **Gap Method vs Token Inconsistency**
   - Some code uses `gap_2()`, others use `gap(px(8.))`
   - Should standardize on `gap(px(spacing.gap_X))`

5. **Missing Spacing Token Levels**
   - No `2xl` (32px) or `3xl` (40px) tokens
   - Some code uses `24.0` which suggests need for larger tokens

### Low Priority

6. **Shortcut Badge Styling Varies**
   - list_item.rs: `px(6.)`, `py(2.)`, `rounded(px(3.))`
   - actions.rs: `px(6.)`, `py(2.)`, `rounded(px(4.))`
   - Minor inconsistency in border radius

---

## 8. Recommended Spacing Scale

Based on the audit, here's the recommended unified spacing scale:

```rust
// Add to DesignSpacing
pub struct DesignSpacing {
    // Base unit: 4px
    pub unit: f32,            // 4.0 (reference only)
    
    // Content spacing
    pub padding_xxs: f32,     // 2.0 (half unit, for tight spacing)
    pub padding_xs: f32,      // 4.0
    pub padding_sm: f32,      // 8.0
    pub padding_md: f32,      // 12.0
    pub padding_lg: f32,      // 16.0
    pub padding_xl: f32,      // 24.0
    pub padding_2xl: f32,     // 32.0 (NEW)
    pub padding_3xl: f32,     // 40.0 (NEW)
    
    // Gaps (between elements)
    pub gap_xs: f32,          // 2.0 (NEW)
    pub gap_sm: f32,          // 4.0
    pub gap_md: f32,          // 8.0
    pub gap_lg: f32,          // 12.0
    pub gap_xl: f32,          // 16.0 (NEW)
    
    // Semantic spacing
    pub item_padding_x: f32,  // 12.0
    pub item_padding_y: f32,  // 8.0 (or 6.0?)
    pub section_padding: f32, // 16.0
    pub container_margin: f32,// 16.0
    
    // Component-specific
    pub header_padding_x: f32,// 16.0
    pub header_padding_y: f32,// 8.0
    pub header_height: f32,   // 44.0 (NEW)
    pub status_bar_height: f32, // 28.0 (NEW)
}
```

---

## 9. Action Items

### High Priority

- [ ] **Migrate list_item.rs to design tokens**
  - Replace hardcoded `12.`, `6.`, `8.` with `spacing.*`
  - Pass `DesignSpacing` to component or use global accessor

- [ ] **Standardize main.rs header spacing**
  - Define `HEADER_HEIGHT` constant
  - Use tokens for all padding values

- [ ] **Add missing token levels**
  - Add `padding_2xl: 32.0`
  - Add `gap_xl: 16.0`
  - Add `header_height`, `status_bar_height`

### Medium Priority

- [ ] **Standardize gap usage**
  - Replace `gap_2()` with `gap(px(spacing.gap_md))`
  - Document when to use which gap size

- [ ] **Unify shortcut badge styling**
  - Create shared constant or token for border radius
  - Ensure consistent padding

### Low Priority

- [ ] **Document spacing conventions**
  - Add spacing guide to AGENTS.md
  - Create visual reference

---

## 10. Positive Patterns to Preserve

### prompts.rs - Token Usage Model
```rust
let tokens = get_tokens(self.design_variant);
let spacing = tokens.spacing();

div()
    .px(px(spacing.item_padding_x))
    .py(px(spacing.padding_md))
```

### actions.rs - Module Constants
```rust
pub const POPUP_WIDTH: f32 = 320.0;
pub const POPUP_MAX_HEIGHT: f32 = 400.0;
pub const ACTION_ITEM_HEIGHT: f32 = 42.0;
```

### list_item.rs - Height Constants for Virtualization
```rust
pub const LIST_ITEM_HEIGHT: f32 = 40.0;
pub const SECTION_HEADER_HEIGHT: f32 = 24.0;
```

### Flex Container Pattern
```rust
div()
    .flex()
    .flex_col()
    .flex_1()
    .min_h(px(0.))  // Critical for proper shrinking
    .w_full()
    .overflow_hidden()
```

---

## Appendix: All px() Values by File

### main.rs (Selected)
`381.0, 246.0, 750., 0., 8., 12., 16., 28., 4., 120.0, 20.0, 2., 6., 14., 380.0, 280., 1.`

### prompts.rs
Uses design tokens almost exclusively: `spacing.item_padding_x`, `spacing.padding_md`, etc.

### list_item.rs
`40.0, 24.0, 3.0, 20., 16., 0., 14., 18., 12., 14., 11., 6., 2., 3., 8., 4., 16.`

### actions.rs
`320.0, 400.0, 12.0, 8.0, 12.0, 8.0, 42.0, 3.0, 44.0, 24.0, 240.0, 28.0, 2., 16., 6., 4.`

### editor.rs
`8.4, 14.0, 1.43, 50.0, 2.0, 28., 4.`
