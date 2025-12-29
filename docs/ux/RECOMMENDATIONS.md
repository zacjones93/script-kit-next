# UX Recommendations - Prioritized Action Items

**Generated:** December 29, 2025  
**Source:** Synthesis of 12 UX audit reports  
**Purpose:** Actionable improvement roadmap for Script Kit GPUI

---

## Priority Levels

| Level | Meaning | Timeframe |
|-------|---------|-----------|
| **P0** | Critical - Blocks adoption/accessibility | Immediate (days) |
| **P1** | High - Significant UX friction | This sprint (1-2 weeks) |
| **P2** | Medium - Notable improvements | Next sprint (2-4 weeks) |
| **P3** | Low - Polish and nice-to-have | Future (1+ months) |

---

## P0 - Critical (Must Fix Immediately)

### P0-1: Implement Fuzzy Search

**Impact:** High - Users cannot find scripts with typos  
**Effort:** Medium (2-3 days)  
**Source:** [COMPETITOR_ANALYSIS.md](./COMPETITOR_ANALYSIS.md), [PROMPT_TYPES.md](./PROMPT_TYPES.md)

**Current State:**
- Simple substring matching in filter logic
- No typo tolerance
- No result ranking

**Recommendation:**
```rust
// Integrate nucleo-matcher or similar
use nucleo_matcher::{Matcher, Utf32Str};

fn fuzzy_filter(query: &str, items: &[Script]) -> Vec<(usize, u32)> {
    let matcher = Matcher::new(nucleo_matcher::Config::DEFAULT);
    items.iter().enumerate()
        .filter_map(|(idx, item)| {
            matcher.fuzzy_match(Utf32Str::from(&item.name), Utf32Str::from(query))
                .map(|score| (idx, score))
        })
        .collect()
}
```

**Files Affected:**
- `src/main.rs` (filter logic ~line 1150)
- `src/scripts.rs` (Script struct)
- `Cargo.toml` (add nucleo-matcher dependency)

---

### P0-2: Add VoiceOver/Screen Reader Support

**Impact:** Critical - Application unusable for blind users  
**Effort:** High (1 week)  
**Source:** [ACCESSIBILITY.md](./ACCESSIBILITY.md)

**Current State:**
- No accessibility labels on any elements
- No ARIA-equivalent roles
- Interactive elements have no accessible names

**Recommendation:**
```rust
// Add accessibility extensions to GPUI elements
div()
    .accessibility_label("Script list")
    .accessibility_role(Role::List)
    .child(
        ListItem::new(name, colors)
            .accessibility_label(&format!("{}: {}", name, description))
            .accessibility_role(Role::ListItem)
    )
```

**Files Affected:**
- `src/list_item.rs` (ListItem component)
- `src/components/button.rs` (Button component)
- `src/components/toast.rs` (Toast announcements)
- `src/actions.rs` (ActionsDialog)
- `src/prompts.rs` (ArgPrompt, DivPrompt)
- All prompt files

**WCAG Criteria Addressed:**
- 1.1.1 Non-text Content
- 4.1.2 Name, Role, Value

---

### P0-3: Fix WCAG Color Contrast Failures

**Impact:** High - Low vision users cannot read text  
**Effort:** Low (2-4 hours)  
**Source:** [ACCESSIBILITY.md](./ACCESSIBILITY.md), [VISUAL_DESIGN.md](./VISUAL_DESIGN.md)

**Current State:**
| Text Type | Current | Ratio | Required |
|-----------|---------|-------|----------|
| Muted | `#808080` | 3.9:1 | 4.5:1 FAIL |
| Dimmed | `#666666` | 2.9:1 | 4.5:1 FAIL |

**Recommendation:**
```rust
// src/theme.rs - Update dark_default()
pub fn dark_default() -> Self {
    ColorScheme {
        text: TextColors {
            // Existing - passes
            primary: 0xffffff,    // 15.1:1 PASS
            secondary: 0xcccccc,  // 10.1:1 PASS
            tertiary: 0x999999,   // 5.3:1 PASS
            // FIXED values
            muted: 0x9e9e9e,      // 5.5:1 PASS (was 0x808080)
            dimmed: 0x888888,     // 4.5:1 PASS (was 0x666666)
        },
        // ...
    }
}
```

**Files Affected:**
- `src/theme.rs` (lines 140-180)

---

## P1 - High Priority (Fix in Next Sprint)

### P1-1: Add Loading Indicators

**Impact:** High - No feedback during async operations  
**Effort:** Medium (2-3 days)  
**Source:** [ANIMATION_FEEDBACK.md](./ANIMATION_FEEDBACK.md), [COMPONENT_LIBRARY.md](./COMPONENT_LIBRARY.md)

**Current State:**
- Background app loading shows only log messages
- No visual spinners or progress
- Users think app is frozen

**Recommendation:**
1. Create `Spinner` component:
```rust
pub struct Spinner {
    size: f32,
    color: HexColor,
    visible: bool,
}

impl Render for Spinner {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Timer-based rotation animation
        div()
            .size(px(self.size))
            .child(svg().path("spinner.svg").rotation(self.rotation))
    }
}
```

2. Add loading states to app:
```rust
struct ScriptListApp {
    is_loading_apps: bool,
    is_loading_scripts: bool,
    // ...
}
```

**Files Affected:**
- `src/components/mod.rs` (add spinner module)
- `src/components/spinner.rs` (new file)
- `src/main.rs` (loading state tracking)
- `assets/icons/spinner.svg` (new asset)

---

### P1-2: Implement Frecency Ranking

**Impact:** High - Most-used scripts buried in results  
**Effort:** Medium (2-3 days)  
**Source:** [COMPETITOR_ANALYSIS.md](./COMPETITOR_ANALYSIS.md)

**Current State:**
- `src/frecency.rs` module exists but not integrated
- Results are statically ordered

**Recommendation:**
```rust
// Integrate existing frecency module
fn get_ranked_results(&self, filter: &str) -> Vec<SearchResult> {
    let filtered = self.fuzzy_filter(filter);
    
    // Apply frecency boost
    filtered.sort_by(|a, b| {
        let a_score = a.fuzzy_score + self.frecency.get_score(&a.script.id);
        let b_score = b.fuzzy_score + self.frecency.get_score(&b.script.id);
        b_score.cmp(&a_score)
    });
    
    filtered
}

// Record usage on script execution
fn execute_script(&mut self, script: &Script) {
    self.frecency.record_access(&script.id);
    // ... execute
}
```

**Files Affected:**
- `src/main.rs` (integrate frecency)
- `src/frecency.rs` (already exists)
- Storage for frecency data (SQLite or JSON)

---

### P1-3: Complete FieldsPrompt UI

**Impact:** Medium - SDK feature doesn't work  
**Effort:** Medium (2-3 days)  
**Source:** [PROMPT_TYPES.md](./PROMPT_TYPES.md)

**Current State:**
- Protocol message type exists
- Field struct defined
- NO UI rendering

**Recommendation:**
```rust
pub struct FieldsPrompt {
    id: String,
    fields: Vec<Field>,
    values: Vec<String>,
    focused_field: usize,
    focus_handle: FocusHandle,
}

impl Render for FieldsPrompt {
    fn render(&mut self, ...) -> impl IntoElement {
        div()
            .flex_col()
            .gap_4()
            .children(self.fields.iter().enumerate().map(|(i, field)| {
                self.render_field(i, field)
            }))
    }
}
```

**Files Affected:**
- `src/prompts.rs` (add FieldsPrompt)
- `src/main.rs` (handle Fields message, add AppView variant)

---

### P1-4: Add Per-Item Action Shortcuts

**Impact:** High - Missing Raycast's killer feature  
**Effort:** High (1 week)  
**Source:** [COMPETITOR_ANALYSIS.md](./COMPETITOR_ANALYSIS.md)

**Current State:**
- Cmd+K shows actions popup
- No per-item keyboard shortcuts (Cmd+1, Cmd+2, etc.)

**Recommendation:**
```rust
// Action struct with shortcut
pub struct Action {
    label: String,
    shortcut: Option<String>,  // "Cmd+1", "Cmd+E", etc.
    callback: ActionCallback,
}

// In keyboard handler
match key.as_str() {
    "1" if has_cmd => self.execute_action(0, cx),
    "2" if has_cmd => self.execute_action(1, cx),
    // ...
}

// Display shortcuts in ActionPanel
div()
    .child(action.label)
    .child(if let Some(shortcut) = &action.shortcut {
        div().text_xs().child(shortcut)
    })
```

**Files Affected:**
- `src/actions.rs` (Action struct, shortcut display)
- `src/main.rs` (keyboard handling for shortcuts)

---

### P1-5: Improve Startup Performance

**Impact:** High - 100ms warm start vs competitors' 30-50ms  
**Effort:** High (1 week)  
**Source:** [COMPETITOR_ANALYSIS.md](./COMPETITOR_ANALYSIS.md)

**Current State:**
- Cold start ~500ms
- Warm start ~100ms
- Competitors are 2-5x faster

**Recommendation:**
1. Pre-index scripts at startup
2. Lazy-load app icons
3. Cache script metadata
4. Use incremental compilation for scripts

```rust
// Lazy icon loading
struct LazyIcon {
    path: String,
    loaded: OnceCell<Option<Arc<RenderImage>>>,
}

impl LazyIcon {
    fn get(&self, cx: &App) -> Option<Arc<RenderImage>> {
        self.loaded.get_or_init(|| load_icon(&self.path, cx)).clone()
    }
}
```

**Files Affected:**
- `src/main.rs` (startup sequence)
- `src/scripts.rs` (script loading)
- `src/app_launcher.rs` (icon loading)

---

## P2 - Medium Priority (Plan for Future)

### P2-1: Connect Design Variant Renderers

**Impact:** Medium - 9 variants are non-functional  
**Effort:** High (1-2 weeks)  
**Source:** [DESIGN_VARIANTS.md](./DESIGN_VARIANTS.md)

**Current State:**
- 11 design variants defined
- Only Minimal and RetroTerminal have active custom renderers
- Others fall through to default ListItem

**Recommendation:**
Either:
1. **Connect renderers** - Pass app state (results, selection, filter) to variant renderers
2. **Or remove variants** - Keep only working variants to avoid confusion

```rust
// Fix DesignRenderer trait to pass required state
pub trait DesignRenderer {
    fn render_script_list(
        &self,
        results: &[SearchResult],
        selected_index: usize,
        filter: &str,
        cx: &mut Context<App>
    ) -> AnyElement;
}
```

**Files Affected:**
- `src/designs/mod.rs`
- `src/designs/traits.rs`
- All variant files in `src/designs/`

---

### P2-2: Add Terminal Scrollback

**Impact:** Medium - Cannot review previous output  
**Effort:** Medium (3-5 days)  
**Source:** [PROMPT_TYPES.md](./PROMPT_TYPES.md)

**Current State:**
- Terminal shows only visible viewport
- No history buffer
- Cannot scroll up

**Recommendation:**
```rust
pub struct TermPrompt {
    // Add scrollback
    scrollback_buffer: Vec<Vec<Cell>>,
    scrollback_offset: usize,
    max_scrollback_lines: usize,  // e.g., 10000
}

impl TermPrompt {
    fn scroll_up(&mut self, lines: usize) {
        self.scrollback_offset = (self.scrollback_offset + lines)
            .min(self.scrollback_buffer.len());
    }
    
    fn scroll_down(&mut self, lines: usize) {
        self.scrollback_offset = self.scrollback_offset.saturating_sub(lines);
    }
}
```

**Files Affected:**
- `src/term_prompt.rs`
- `src/terminal/alacritty.rs`

---

### P2-3: Standardize Icon Colors

**Impact:** Low-Medium - Icons don't adapt to themes  
**Effort:** Low (1 hour)  
**Source:** [ICONS_ASSETS.md](./ICONS_ASSETS.md)

**Current State:**
- 4 icons use `currentColor` (correct)
- 18 icons use hardcoded `black` (incorrect)

**Recommendation:**
```bash
# Run from assets/icons/
for svg in *.svg; do
  sed -i '' 's/stroke="black"/stroke="currentColor"/g' "$svg"
  sed -i '' 's/fill="black"/fill="currentColor"/g' "$svg"
done
```

**Files Affected:**
- All 18 SVG files in `assets/icons/`

---

### P2-4: Add Toast Entrance/Exit Animations

**Impact:** Low-Medium - Jarring instant appearance  
**Effort:** Medium (2-3 days)  
**Source:** [ANIMATION_FEEDBACK.md](./ANIMATION_FEEDBACK.md)

**Current State:**
- Toasts appear/disappear instantly
- No slide or fade

**Recommendation:**
```rust
pub struct AnimatedToast {
    toast: Toast,
    opacity: AnimatedValue,  // 0.0 -> 1.0 on enter
    offset_x: AnimatedValue, // slides from right
}

impl AnimatedToast {
    fn tick(&mut self) -> bool {
        self.opacity.tick();
        self.offset_x.tick();
        !self.opacity.is_complete()
    }
}
```

**Files Affected:**
- `src/components/toast.rs`
- `src/toast_manager.rs`

---

### P2-5: Add Editor Find/Replace

**Impact:** Medium - Essential for longer content  
**Effort:** Medium (3-5 days)  
**Source:** [PROMPT_TYPES.md](./PROMPT_TYPES.md)

**Current State:**
- Full code editor
- No Cmd+F find functionality

**Recommendation:**
```rust
struct EditorPrompt {
    // Add find state
    find_query: String,
    find_matches: Vec<(usize, usize)>,  // (line, col)
    current_match: usize,
    show_find_bar: bool,
}

// Keyboard handling
("f", true, false, false) => self.show_find_bar = true,
("g", true, false, false) => self.find_next(),
("g", true, true, false) => self.find_previous(),
```

**Files Affected:**
- `src/editor.rs`

---

### P2-6: Implement Multi-Select for SelectPrompt

**Impact:** Medium - Protocol supports but UI doesn't  
**Effort:** Medium (2-3 days)  
**Source:** [PROMPT_TYPES.md](./PROMPT_TYPES.md)

**Current State:**
- Select message has `multiple: Option<bool>`
- UI ignores it, always single-select

**Recommendation:**
```rust
// Add selection tracking
struct SelectPrompt {
    choices: Vec<Choice>,
    selected_indices: HashSet<usize>,  // for multi-select
    multiple: bool,
}

// Space to toggle selection
("space", false, false, false) if self.multiple => {
    if self.selected_indices.contains(&self.focused_index) {
        self.selected_indices.remove(&self.focused_index);
    } else {
        self.selected_indices.insert(self.focused_index);
    }
}

// Submit returns array in multi mode
fn submit(&self) -> Value {
    if self.multiple {
        serde_json::to_value(&self.get_selected_values())
    } else {
        serde_json::to_value(&self.get_focused_value())
    }
}
```

**Files Affected:**
- `src/prompts.rs` (SelectPrompt component)
- `src/main.rs` (handle multi-select response)

---

## P3 - Low Priority (Nice to Have)

### P3-1: Add Motion Reduction Preference

**Impact:** Low - Accessibility enhancement  
**Effort:** Low (1 day)  
**Source:** [ACCESSIBILITY.md](./ACCESSIBILITY.md)

**Recommendation:**
```rust
fn should_reduce_motion() -> bool {
    #[cfg(target_os = "macos")]
    unsafe {
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        msg_send![workspace, accessibilityDisplayShouldReduceMotion]
    }
    #[cfg(not(target_os = "macos"))]
    false
}
```

**Files Affected:**
- `src/main.rs` (query on startup)
- Animation-related code (disable if reduced motion)

---

### P3-2: Add High Contrast Mode

**Impact:** Low - Accessibility enhancement  
**Effort:** Medium (2-3 days)  
**Source:** [ACCESSIBILITY.md](./ACCESSIBILITY.md)

**Recommendation:**
```rust
impl ColorScheme {
    pub fn high_contrast() -> Self {
        ColorScheme {
            background: BackgroundColors { main: 0x000000, ... },
            text: TextColors {
                primary: 0xffffff,
                secondary: 0xffffff,
                muted: 0xe0e0e0,
                dimmed: 0xc0c0c0,
            },
            ui: UIColors { border: 0xffffff, ... },
            ...
        }
    }
}
```

**Files Affected:**
- `src/theme.rs`
- `src/main.rs` (detect preference)

---

### P3-3: Add Audio Feedback Option

**Impact:** Low - Polish  
**Effort:** Low (1 day)  
**Source:** [COMPETITOR_ANALYSIS.md](./COMPETITOR_ANALYSIS.md)

**Recommendation:**
```rust
fn play_feedback_sound(sound: FeedbackSound) {
    #[cfg(target_os = "macos")]
    match sound {
        FeedbackSound::Select => NSSound::play("Pop"),
        FeedbackSound::Error => NSSound::play("Basso"),
        FeedbackSound::Success => NSSound::play("Glass"),
    }
}
```

**Files Affected:**
- `src/main.rs` or new `src/audio.rs`
- `src/config.rs` (enable/disable setting)

---

### P3-4: Add Icon Accessibility Titles

**Impact:** Low - Accessibility polish  
**Effort:** Low (2-4 hours)  
**Source:** [ICONS_ASSETS.md](./ICONS_ASSETS.md)

**Recommendation:**
```xml
<!-- Add to each SVG -->
<svg ... role="img" aria-labelledby="title">
  <title id="title">File icon</title>
  <path .../>
</svg>
```

**Files Affected:**
- All 22 SVG files in `assets/icons/`

---

### P3-5: Add Configurable Item Density

**Impact:** Low - User preference  
**Effort:** Low (1-2 days)  
**Source:** [RESPONSIVE_BEHAVIOR.md](./RESPONSIVE_BEHAVIOR.md)

**Recommendation:**
```typescript
// config.ts
export default {
  itemDensity: "default" | "compact" | "relaxed",
}
```

```rust
// src/list_item.rs
fn item_height(density: ItemDensity) -> f32 {
    match density {
        ItemDensity::Compact => 32.0,
        ItemDensity::Default => 40.0,
        ItemDensity::Relaxed => 52.0,
    }
}
```

**Files Affected:**
- `src/config.rs`
- `src/list_item.rs`
- `src/window_resize.rs`

---

## Implementation Roadmap

### Week 1-2: Critical Fixes

| Item | Assignee | Days |
|------|----------|------|
| P0-3: WCAG Contrast | - | 0.5 |
| P1-1: Loading Spinner | - | 2 |
| P0-1: Fuzzy Search | - | 3 |

### Week 3-4: Core Improvements

| Item | Assignee | Days |
|------|----------|------|
| P1-2: Frecency Ranking | - | 3 |
| P1-3: FieldsPrompt | - | 3 |
| P0-2: VoiceOver Labels | - | 5 |

### Week 5-6: Polish

| Item | Assignee | Days |
|------|----------|------|
| P1-4: Action Shortcuts | - | 5 |
| P2-3: Icon Colors | - | 0.5 |
| P2-4: Toast Animations | - | 3 |

### Week 7+: Extended Features

| Item | Priority | Days |
|------|----------|------|
| P2-1: Design Variants | P2 | 10 |
| P2-2: Terminal Scrollback | P2 | 4 |
| P2-5: Editor Find | P2 | 4 |
| P1-5: Startup Performance | P1 | 5 |

---

## Success Metrics

### After P0 Completion

- [ ] All text meets WCAG AA contrast (4.5:1+)
- [ ] Fuzzy search finds scripts with 1-2 typos
- [ ] VoiceOver can navigate all interactive elements

### After P1 Completion

- [ ] Loading indicators visible during async ops
- [ ] Recently-used scripts appear first
- [ ] FieldsPrompt renders and submits values
- [ ] At least 3 action shortcuts work (Cmd+1/2/3)

### After P2 Completion

- [ ] All 11 design variants render correctly
- [ ] Terminal scrollback works (1000+ lines)
- [ ] All icons adapt to theme colors
- [ ] Toast animations feel smooth

---

*Recommendations synthesized from 12 UX audit reports.*  
*See [UX_AUDIT.md](../../UX_AUDIT.md) for full audit summary.*
