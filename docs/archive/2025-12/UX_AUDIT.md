# Script Kit GPUI - Comprehensive UX Audit

**Audit Date:** December 29, 2025  
**Auditors:** AI Swarm (13 specialized workers)  
**Scope:** Complete UX evaluation of Script Kit GPUI launcher application

---

## Executive Summary

This comprehensive UX audit evaluated Script Kit GPUI, a 90k+ line Rust/GPUI launcher application targeting developers and automation enthusiasts. The audit covered 13 domains: visual design, components, typography, icons, keyboard navigation, animations, accessibility, window management, responsive behavior, prompt types, and competitor analysis.

### Key Strengths

- **Solid theme architecture** with 11 design variants and focus-aware colors
- **Excellent keyboard navigation** with proper cross-platform arrow key handling
- **Full terminal emulation** and **code editor prompts** - unique differentiators vs competitors
- **Robust window management** with multi-monitor awareness and eye-line positioning
- **TypeScript-first scripting** offers modern developer experience

### Critical Gaps

- **No fuzzy search** - substring matching only while competitors use fuzzy + frecency
- **WCAG accessibility gaps** - dimmed text fails contrast, no screen reader labels
- **Missing loading indicators** - no visual feedback for async operations
- **Incomplete prompt types** - many SDK-advertised prompts are protocol stubs only
- **No animations/transitions** - GPUI limitation means instant state changes

### Overall Score vs Competitors

| App | Score | Notes |
|-----|-------|-------|
| **Raycast** | 4.7/5 | Polish leader, 1000+ extensions |
| **Alfred** | 4.1/5 | Mature workflows, power users |
| **Script Kit** | 3.3/5 | Powerful but rough edges |
| **Spotlight** | 3.0/5 | Native integration, limited extensibility |

---

## Quick Reference

| Topic | Quick Finding | Priority |
|-------|---------------|----------|
| Search | Substring only, no fuzzy match | P0 |
| Contrast | Muted/dimmed text fails WCAG AA | P1 |
| Loading | No spinners or progress indicators | P1 |
| Prompts | 4/16 prompt types fully implemented | P1 |
| Animations | None (GPUI limitation) | P2 |
| Actions Panel | Partial Cmd+K, no per-item shortcuts | P1 |
| Terminal | Full PTY, missing scrollback/copy | P2 |
| Editor | Full featured, missing find/replace | P2 |
| Icons | 22 icons, 18 use hardcoded black | P2 |
| Design Variants | 11 exist, 9 fall through to default | P2 |

---

## Audit Reports Index

### Core Visual Design

| Report | Location | Key Finding |
|--------|----------|-------------|
| **Theme System & Colors** | [docs/ux/VISUAL_DESIGN.md](docs/ux/VISUAL_DESIGN.md) | Strong architecture, some hardcoded colors in terminal |
| **Design Variants** | [docs/ux/DESIGN_VARIANTS.md](docs/ux/DESIGN_VARIANTS.md) | 11 variants, only 2 fully implemented (Minimal, RetroTerminal) |
| **Component Library** | [docs/ux/COMPONENT_LIBRARY.md](docs/ux/COMPONENT_LIBRARY.md) | 4 core components, missing focus states on buttons |
| **Typography & Spacing** | [docs/ux/TYPOGRAPHY_SPACING.md](docs/ux/TYPOGRAPHY_SPACING.md) | Complete design token system, configurable font sizes |
| **Icons & Assets** | [docs/ux/ICONS_ASSETS.md](docs/ux/ICONS_ASSETS.md) | 22 icons, inconsistent color approach, no accessibility |

### Interaction & Behavior

| Report | Location | Key Finding |
|--------|----------|-------------|
| **Keyboard Navigation** | [docs/ux/KEYBOARD_NAVIGATION.md](docs/ux/KEYBOARD_NAVIGATION.md) | Excellent - proper dual arrow key matching |
| **Animation & Feedback** | [docs/ux/ANIMATION_FEEDBACK.md](docs/ux/ANIMATION_FEEDBACK.md) | Tokens defined but unused (GPUI limitation) |
| **Accessibility** | [docs/ux/ACCESSIBILITY.md](docs/ux/ACCESSIBILITY.md) | Critical gaps - no VoiceOver labels, contrast failures |
| **Window Management** | [docs/ux/WINDOW_MANAGEMENT.md](docs/ux/WINDOW_MANAGEMENT.md) | Robust system with eye-line positioning |
| **Responsive Behavior** | [docs/ux/RESPONSIVE_BEHAVIOR.md](docs/ux/RESPONSIVE_BEHAVIOR.md) | Fixed-width (750px), dynamic height per view type |

### Feature Analysis

| Report | Location | Key Finding |
|--------|----------|-------------|
| **Prompt Types** | [docs/ux/PROMPT_TYPES.md](docs/ux/PROMPT_TYPES.md) | 4 of 16 prompts complete, others are stubs |
| **Competitor Analysis** | [docs/ux/COMPETITOR_ANALYSIS.md](docs/ux/COMPETITOR_ANALYSIS.md) | Behind Raycast on polish, ahead on scripting power |

---

## Top 10 Critical Findings

### 1. No Fuzzy Search (P0)

**Impact:** Users cannot find scripts with typos or partial matches  
**Current:** Simple substring matching  
**Competitors:** All use fuzzy matching with typo tolerance  
**Files:** `src/main.rs` (filter logic)  
**Recommendation:** Integrate `nucleo` or similar fuzzy matching library

### 2. WCAG Contrast Failures (P1)

**Impact:** Low vision users cannot read muted/dimmed text  
**Current:** Muted text `#808080` on `#1e1e1e` = 3.9:1 (needs 4.5:1)  
**Files:** `src/theme.rs` (ColorScheme)  
**Recommendation:** Increase muted to `#9e9e9e`, dimmed to `#888888`

### 3. No Loading Indicators (P1)

**Impact:** No feedback during async operations (script loading, app icons)  
**Current:** Silent background loading, only log messages  
**Files:** `src/main.rs`, `src/components/`  
**Recommendation:** Add spinner component, skeleton loading for lists

### 4. Missing Screen Reader Support (P0)

**Impact:** VoiceOver users cannot use the application  
**Current:** No accessibility labels on any elements  
**Files:** All component files  
**Recommendation:** Add NSAccessibility integration, element labels

### 5. Incomplete Prompt Types (P1)

**Impact:** SDK advertises features that don't work  
**Status:**
- Complete: `arg()`, `div()`, `editor()`, `term()`  
- Partial: `form()`, `fields()`, `select()`  
- Stub only: `path()`, `hotkey()`, `drop()`, `chat()`, `webcam()`, `mic()`, `widget()`  
**Files:** `src/protocol.rs`, `src/prompts.rs`, `src/main.rs`

### 6. Design Variants Not Connected (P2)

**Impact:** 9 of 11 design variants fall through to default renderer  
**Current:** Only Minimal and RetroTerminal have active custom renderers  
**Files:** `src/designs/mod.rs`, `src/designs/*.rs`  
**Recommendation:** Connect app state to design renderers or remove variants

### 7. Hardcoded Icon Colors (P2)

**Impact:** 18 of 22 icons use `black` instead of `currentColor`, breaking theme support  
**Files:** `assets/icons/*.svg`  
**Recommendation:** Run sed replacement to use `currentColor`

### 8. No Animations (P2)

**Impact:** Jarring instant state changes (toasts, scrollbars, selection)  
**Current:** GPUI lacks native transition support  
**Files:** Animation tokens defined but unused in `src/designs/traits.rs`  
**Recommendation:** Implement timer-based interpolation for critical animations

### 9. Terminal Missing Features (P2)

**Impact:** Cannot scroll to previous output or copy text  
**Current:** No scrollback buffer, no selection/copy  
**Files:** `src/term_prompt.rs`  
**Recommendation:** Add scrollback buffer, mouse selection, Cmd+C/V

### 10. No Frecency Ranking (P1)

**Impact:** Frequently used scripts not prioritized  
**Current:** Static ordering  
**Competitors:** All use frecency (frequency + recency)  
**Files:** `src/frecency.rs` (module exists but not integrated)  
**Recommendation:** Integrate existing frecency module with search results

---

## Comparative Analysis Summary

### Script Kit vs Raycast

| Category | Script Kit | Raycast | Winner |
|----------|-----------|---------|--------|
| Extension language | TypeScript | TypeScript/React | Tie |
| Terminal prompt | Full PTY | Limited | Script Kit |
| Code editor | Full featured | None | Script Kit |
| Fuzzy search | No | Yes | Raycast |
| ActionPanel | Partial | Full | Raycast |
| Loading feedback | None | Full | Raycast |
| Extensions available | ~50 | 1000+ | Raycast |
| Open source | Yes | No | Script Kit |

### Unique Script Kit Advantages

1. **Terminal emulation** - Real PTY, no competitor has this
2. **Code editor prompts** - Full syntax highlighting, undo/redo
3. **Design variants** - 11 visual themes vs Raycast's 1
4. **Open source** - Full customization possible
5. **TypeScript SDK** - More accessible than Alfred's AppleScript

---

## Technical Debt Summary

### High Priority

| Issue | Location | Impact |
|-------|----------|--------|
| GPUI `cx.displays()` returns wrong origins | `src/main.rs` | Using native NSScreen workaround |
| Hardcoded terminal colors | `src/terminal/theme_adapter.rs` | 37 hardcoded values |
| RefCell borrow issues on resize | `src/window_resize.rs` | Using 16ms defer delay |

### Medium Priority

| Issue | Location | Impact |
|-------|----------|--------|
| Toast details toggle broken | `src/components/toast.rs` | Click handler not connected |
| Animation tokens unused | `src/designs/traits.rs` | GPUI limitation |
| `#[allow(dead_code)]` flags | Multiple files | Incomplete implementations |

---

## Next Steps

1. **Immediate (This Week)**
   - Fix WCAG contrast issues in theme.rs
   - Add loading spinner component

2. **Short-Term (This Month)**
   - Implement fuzzy search with frecency
   - Add accessibility labels to all components
   - Complete fields() and select() prompts

3. **Medium-Term (This Quarter)**
   - Connect remaining design variants
   - Add terminal scrollback and copy
   - Build ActionPanel with per-item shortcuts

4. **Long-Term (Strategic)**
   - Create extension store/gallery
   - Build migration guides from Alfred/Raycast
   - Position as "VS Code of launchers"

---

## Detailed Recommendations

See [docs/ux/RECOMMENDATIONS.md](docs/ux/RECOMMENDATIONS.md) for prioritized action items with:
- Priority level (P0-P3)
- Impact assessment
- Effort estimate
- Files affected
- Implementation guidance

---

*Audit synthesized from 12 individual reports by AI swarm workers.*  
*Next review recommended: After P0/P1 items addressed (~Q1 2025)*
