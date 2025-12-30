# Storybook Implementation Plan for script-kit-gpui

## Executive Summary

This document outlines a tiered implementation plan for a storybook-like component preview system tailored to script-kit-gpui. The plan synthesizes research from Zed's systems, GPUI patterns, Alacritty's hot-reload approach, and other Rust GUI frameworks.

**Key Insight**: script-kit-gpui already has strong foundations:
- 11 design variants in `src/designs/` with `DesignTokens` trait system
- 7 reusable components in `src/components/` with builder patterns
- Theme hot-reload via file watcher
- `captureScreenshot()` SDK function for visual testing
- stdin JSON protocol for automated testing

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        STORYBOOK ARCHITECTURE                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐        │
│  │   src/stories/  │    │  src/storybook/ │    │   bin/storybook │        │
│  │                 │    │                 │    │                 │        │
│  │  Component      │───▶│  Story trait    │───▶│  Standalone     │        │
│  │  stories with   │    │  StoryRegistry  │    │  CLI binary     │        │
│  │  #[story] macro │    │  Preview views  │    │  with fuzzy     │        │
│  │                 │    │                 │    │  search         │        │
│  └─────────────────┘    └─────────────────┘    └─────────────────┘        │
│          │                      │                      │                  │
│          │                      │                      │                  │
│          ▼                      ▼                      ▼                  │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │                    SHARED INFRASTRUCTURE                             │  │
│  │                                                                      │  │
│  │  • DesignTokens system (existing)                                   │  │
│  │  • Theme hot-reload (existing)                                      │  │
│  │  • captureScreenshot() for visual testing                           │  │
│  │  • Component builder patterns (existing)                            │  │
│  └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Four Use Cases

| Use Case | Description | Solution |
|----------|-------------|----------|
| **Dev Workflow** | Quickly iterate on components | Hot-reload + standalone binary |
| **Documentation** | Show all component states | Story catalog with variants |
| **Visual Testing** | Detect UI regressions | Screenshot comparison |
| **Theme Validation** | Test all 11 design variants | Theme matrix view |

---

## Tier 1: Quick Wins (1-2 Days)

### 1.1 Design Gallery Script

Create a TypeScript test script that renders all design variants using existing SDK.

**File**: `tests/smoke/design-gallery.ts`

```typescript
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

const DESIGNS = [
  'Default', 'Minimal', 'RetroTerminal', 'Glassmorphism',
  'Brutalist', 'NeonCyberpunk', 'Paper', 'AppleHIG',
  'Material3', 'Compact', 'Playful'
];

async function captureAllDesigns() {
  const screenshotDir = join(process.cwd(), 'test-screenshots', 'designs');
  mkdirSync(screenshotDir, { recursive: true });

  for (const design of DESIGNS) {
    // Switch design via keyboard shortcut simulation
    await div(`
      <div class="p-4">
        <h1 class="text-2xl font-bold">Design: ${design}</h1>
        <p class="text-gray-400">Sample content for visual testing</p>
      </div>
    `);
    
    await new Promise(r => setTimeout(r, 500));
    
    const screenshot = await captureScreenshot();
    const path = join(screenshotDir, `${design.toLowerCase()}.png`);
    writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
    console.error(`[GALLERY] Captured: ${path}`);
  }
}

captureAllDesigns().then(() => process.exit(0));
```

**Run**: 
```bash
echo '{"type":"run","path":"'$(pwd)'/tests/smoke/design-gallery.ts"}' | ./target/debug/script-kit-gpui
```

### 1.2 Component Showcase Script

**File**: `tests/smoke/component-showcase.ts`

```typescript
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

const COMPONENTS = {
  'button-primary': `<button class="px-4 py-2 bg-yellow-500 rounded">Primary</button>`,
  'button-ghost': `<button class="px-4 py-2 border border-gray-500 rounded">Ghost</button>`,
  'toast-success': `<div class="p-3 bg-green-800 rounded flex items-center gap-2">✓ Success</div>`,
  'toast-error': `<div class="p-3 bg-red-800 rounded flex items-center gap-2">✗ Error</div>`,
  'input-text': `<input type="text" placeholder="Enter text..." class="px-3 py-2 bg-gray-800 rounded" />`,
  'list-item-selected': `<div class="px-4 py-2 bg-yellow-500/20 border-l-2 border-yellow-500">Selected Item</div>`,
};

async function captureComponents() {
  const dir = join(process.cwd(), 'test-screenshots', 'components');
  mkdirSync(dir, { recursive: true });

  for (const [name, html] of Object.entries(COMPONENTS)) {
    await div(`<div class="p-8 bg-gray-900">${html}</div>`);
    await new Promise(r => setTimeout(r, 300));
    
    const screenshot = await captureScreenshot();
    writeFileSync(join(dir, `${name}.png`), Buffer.from(screenshot.data, 'base64'));
    console.error(`[COMPONENT] ${name}`);
  }
}

captureComponents().then(() => process.exit(0));
```

### 1.3 Theme Matrix Generator

**File**: `scripts/theme-matrix.ts`

Generate a visual matrix of components across all themes:

```typescript
// Generates an HTML report with all components × all themes
// Outputs to ./docs/generated/theme-matrix.html
```

### 1.4 Quick Verification Commands

Add to `package.json`:

```json
{
  "scripts": {
    "storybook:capture": "cargo build && echo '{\"type\":\"run\",\"path\":\"'$(pwd)'/tests/smoke/design-gallery.ts\"}' | ./target/debug/script-kit-gpui",
    "storybook:components": "cargo build && echo '{\"type\":\"run\",\"path\":\"'$(pwd)'/tests/smoke/component-showcase.ts\"}' | ./target/debug/script-kit-gpui"
  }
}
```

---

## Tier 2: Core Storybook System (1-2 Weeks)

### 2.1 Story Trait Definition

**File**: `src/storybook/story.rs`

```rust
use gpui::*;

/// A story renders a component in various states for preview
pub trait Story: Send + Sync {
    /// Unique identifier for this story
    fn id(&self) -> &'static str;
    
    /// Display name in the catalog
    fn name(&self) -> &'static str;
    
    /// Category for organization (e.g., "Buttons", "Forms", "Layout")
    fn category(&self) -> &'static str;
    
    /// Render the story preview
    fn render(&self, cx: &mut WindowContext) -> AnyElement;
    
    /// Optional: Render story with specific theme
    fn render_with_theme(&self, theme: &Theme, cx: &mut WindowContext) -> AnyElement {
        self.render(cx)
    }
    
    /// Get all variants of this story
    fn variants(&self) -> Vec<StoryVariant> {
        vec![StoryVariant::default()]
    }
}

#[derive(Default)]
pub struct StoryVariant {
    pub name: String,
    pub description: Option<String>,
    pub props: HashMap<String, String>,
}
```

### 2.2 Story Registry (inventory pattern from Zed)

**File**: `src/storybook/registry.rs`

```rust
use inventory;

/// Register a story at compile time
#[macro_export]
macro_rules! register_story {
    ($story:expr) => {
        inventory::submit! {
            StoryEntry::new($story)
        }
    };
}

pub struct StoryEntry {
    pub story: Box<dyn Story>,
}

inventory::collect!(StoryEntry);

/// Get all registered stories
pub fn all_stories() -> impl Iterator<Item = &'static StoryEntry> {
    inventory::iter::<StoryEntry>()
}

/// Find stories by category
pub fn stories_by_category(category: &str) -> Vec<&'static StoryEntry> {
    all_stories()
        .filter(|e| e.story.category() == category)
        .collect()
}
```

### 2.3 Story Layout Helpers (from Zed's story crate)

**File**: `src/storybook/layout.rs`

```rust
use gpui::*;

/// Container for story content with consistent padding
pub fn story_container() -> Div {
    div()
        .flex()
        .flex_col()
        .gap_4()
        .p_4()
        .bg(rgb(0x1e1e1e))
        .size_full()
}

/// Section with title for grouping related items
pub fn story_section(title: &str) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .text_sm()
                .text_color(rgb(0x888888))
                .child(title.to_string())
        )
}

/// Individual item row in a story
pub fn story_item(label: &str, element: impl IntoElement) -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_4()
        .child(
            div()
                .w(px(120.))
                .text_sm()
                .text_color(rgb(0x666666))
                .child(label.to_string())
        )
        .child(element)
}

/// Code block for showing usage
pub fn code_block(code: &str) -> Div {
    div()
        .font_family("Menlo")
        .text_sm()
        .p_2()
        .bg(rgb(0x2d2d2d))
        .rounded_md()
        .child(code.to_string())
}
```

### 2.4 Storybook Browser UI

**File**: `src/storybook/browser.rs`

```rust
pub struct StoryBrowser {
    stories: Vec<&'static StoryEntry>,
    selected_index: usize,
    filter: String,
    current_theme: Theme,
    design_variant: DesignVariant,
    scroll_handle: UniformListScrollHandle,
}

impl StoryBrowser {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let stories: Vec<_> = all_stories().collect();
        Self {
            stories,
            selected_index: 0,
            filter: String::new(),
            current_theme: Theme::default(),
            design_variant: DesignVariant::Default,
            scroll_handle: UniformListScrollHandle::new(),
        }
    }
    
    fn filtered_stories(&self) -> Vec<&&'static StoryEntry> {
        if self.filter.is_empty() {
            self.stories.iter().collect()
        } else {
            self.stories
                .iter()
                .filter(|s| {
                    s.story.name().to_lowercase().contains(&self.filter.to_lowercase())
                        || s.story.category().to_lowercase().contains(&self.filter.to_lowercase())
                })
                .collect()
        }
    }
}

impl Render for StoryBrowser {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let filtered = self.filtered_stories();
        
        div()
            .flex()
            .flex_row()
            .size_full()
            // Left sidebar: story list
            .child(
                div()
                    .w(px(280.))
                    .border_r_1()
                    .border_color(rgb(0x3d3d3d))
                    .flex()
                    .flex_col()
                    .child(self.render_search_bar(cx))
                    .child(self.render_story_list(&filtered, cx))
            )
            // Right panel: story preview
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .child(self.render_toolbar(cx))
                    .child(self.render_preview(cx))
            )
    }
}
```

### 2.5 Standalone Binary

**File**: `src/bin/storybook.rs`

```rust
//! Storybook - Component Preview Tool for script-kit-gpui
//!
//! Usage:
//!   cargo run -p storybook
//!   cargo run -p storybook -- --theme "One Dark"
//!   cargo run -p storybook -- --story "button-primary"

use clap::Parser;
use gpui::*;
use script_kit_gpui::storybook::*;

#[derive(Parser)]
#[command(name = "storybook")]
#[command(about = "Component preview tool for script-kit-gpui")]
struct Args {
    /// Initial theme to load
    #[arg(long)]
    theme: Option<String>,
    
    /// Jump directly to a specific story
    #[arg(long)]
    story: Option<String>,
    
    /// Design variant (default, minimal, brutalist, etc.)
    #[arg(long, default_value = "default")]
    design: String,
}

fn main() {
    let args = Args::parse();
    
    App::new().run(|cx: &mut AppContext| {
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(1500.), px(780.)),
                cx,
            ))),
            titlebar: Some(TitlebarOptions {
                title: Some("Script Kit Storybook".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        
        cx.open_window(options, |window, cx| {
            let mut browser = StoryBrowser::new(cx);
            
            // Apply CLI args
            if let Some(theme_name) = &args.theme {
                browser.load_theme(theme_name);
            }
            if let Some(story_id) = &args.story {
                browser.select_story(story_id);
            }
            
            cx.new_view(|_| browser)
        });
    });
}
```

### 2.6 Example Stories

**File**: `src/stories/button_stories.rs`

```rust
use crate::components::{Button, ButtonColors, ButtonVariant};
use crate::storybook::*;

pub struct ButtonStory;

impl Story for ButtonStory {
    fn id(&self) -> &'static str { "button" }
    fn name(&self) -> &'static str { "Button" }
    fn category(&self) -> &'static str { "Components" }
    
    fn render(&self, cx: &mut WindowContext) -> AnyElement {
        let theme = Theme::default();
        let colors = ButtonColors::from_theme(&theme);
        
        story_container()
            .child(story_section("Button Variants")
                .child(story_item("Primary", 
                    Button::new("Primary", colors)
                        .variant(ButtonVariant::Primary)))
                .child(story_item("Ghost",
                    Button::new("Ghost", colors)
                        .variant(ButtonVariant::Ghost)))
                .child(story_item("Icon",
                    Button::new("", colors)
                        .variant(ButtonVariant::Icon)
                        .icon("play"))))
            .child(story_section("With Shortcuts")
                .child(story_item("Enter",
                    Button::new("Submit", colors)
                        .shortcut("↵")))
                .child(story_item("Escape",
                    Button::new("Cancel", colors)
                        .shortcut("⎋"))))
            .child(story_section("Code")
                .child(code_block(r#"
Button::new("Click me", colors)
    .variant(ButtonVariant::Primary)
    .shortcut("↵")
    .on_click(|_, _, _| { ... })
"#)))
            .into_any_element()
    }
    
    fn variants(&self) -> Vec<StoryVariant> {
        vec![
            StoryVariant { name: "primary".into(), ..Default::default() },
            StoryVariant { name: "ghost".into(), ..Default::default() },
            StoryVariant { name: "icon".into(), ..Default::default() },
            StoryVariant { name: "disabled".into(), ..Default::default() },
        ]
    }
}

register_story!(Box::new(ButtonStory));
```

---

## Tier 3: Advanced Features (2-4 Weeks)

### 3.1 Visual Regression Testing

**File**: `src/storybook/visual_testing.rs`

```rust
use image::{DynamicImage, ImageBuffer};
use std::path::Path;

pub struct VisualTest {
    pub story_id: String,
    pub variant: String,
    pub baseline_path: PathBuf,
    pub threshold: f64,  // 0.0 - 1.0, percentage difference allowed
}

impl VisualTest {
    /// Capture current screenshot and compare to baseline
    pub fn run(&self, cx: &mut WindowContext) -> VisualTestResult {
        // 1. Render story
        let screenshot = capture_window_screenshot(cx);
        
        // 2. Load baseline
        let baseline = if self.baseline_path.exists() {
            Some(image::open(&self.baseline_path).unwrap())
        } else {
            None
        };
        
        // 3. Compare using pixel diff (like dify crate)
        match baseline {
            Some(baseline) => {
                let diff = compute_diff(&screenshot, &baseline);
                if diff.percentage > self.threshold {
                    VisualTestResult::Failed { 
                        diff_percentage: diff.percentage,
                        diff_image: Some(diff.image),
                    }
                } else {
                    VisualTestResult::Passed
                }
            }
            None => {
                // Save as new baseline
                screenshot.save(&self.baseline_path).unwrap();
                VisualTestResult::NewBaseline
            }
        }
    }
}

pub enum VisualTestResult {
    Passed,
    Failed { diff_percentage: f64, diff_image: Option<DynamicImage> },
    NewBaseline,
}
```

### 3.2 CI Integration

**File**: `.github/workflows/visual-tests.yml`

```yaml
name: Visual Regression Tests

on:
  pull_request:
    paths:
      - 'src/components/**'
      - 'src/designs/**'
      - 'src/stories/**'

jobs:
  visual-test:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        
      - name: Build Storybook
        run: cargo build --release -p storybook
        
      - name: Run Visual Tests
        run: |
          ./target/release/storybook --mode ci --output ./visual-results
          
      - name: Upload Results
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: visual-diffs
          path: ./visual-results/diffs/
          
      - name: Comment PR with Diffs
        if: failure()
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const diffs = fs.readdirSync('./visual-results/diffs/');
            // Post comment with image links
```

### 3.3 Theme Matrix View

**File**: `src/storybook/theme_matrix.rs`

```rust
/// Renders a single component across all design variants in a grid
pub struct ThemeMatrixView {
    story: &'static dyn Story,
    designs: Vec<DesignVariant>,
}

impl Render for ThemeMatrixView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let designs = DesignVariant::all();
        
        div()
            .flex()
            .flex_wrap()
            .gap_4()
            .p_4()
            .children(designs.iter().map(|design| {
                let tokens = get_tokens(*design);
                div()
                    .w(px(300.))
                    .h(px(200.))
                    .border_1()
                    .border_color(rgb(0x3d3d3d))
                    .rounded_md()
                    .overflow_hidden()
                    .child(
                        div()
                            .px_2()
                            .py_1()
                            .bg(rgb(0x2d2d2d))
                            .text_xs()
                            .text_color(rgb(0x888888))
                            .child(design.name())
                    )
                    .child(
                        div()
                            .bg(rgb(tokens.colors().background))
                            .flex_1()
                            .p_2()
                            .child(self.story.render_with_design(*design, cx))
                    )
            }))
    }
}
```

### 3.4 Hot Module Replacement (Development)

Since GPUI doesn't support true HMR, use cargo-watch pattern:

**File**: `scripts/storybook-dev.sh`

```bash
#!/bin/bash
# Development mode with auto-reload

# Kill existing processes
pkill -f "storybook" || true

# Watch for changes and rebuild
cargo watch -x "run -p storybook" \
  -w src/components \
  -w src/designs \
  -w src/stories \
  -w src/storybook
```

### 3.5 Accessibility Testing Integration

**File**: `src/storybook/a11y.rs`

```rust
use accesskit::*;

/// Run accessibility checks on a story
pub fn check_accessibility(element: &AnyElement) -> Vec<A11yIssue> {
    let mut issues = vec![];
    
    // Check color contrast ratios
    // Check focus indicators
    // Check keyboard navigation
    // Check screen reader labels
    
    issues
}

pub struct A11yIssue {
    pub severity: IssueSeverity,
    pub rule: &'static str,
    pub message: String,
    pub element_id: Option<String>,
}
```

---

## File/Module Structure

```
src/
├── storybook/
│   ├── mod.rs              # Public exports
│   ├── story.rs            # Story trait
│   ├── registry.rs         # inventory-based registration
│   ├── layout.rs           # Layout helpers (story_container, etc.)
│   ├── browser.rs          # StoryBrowser UI
│   ├── theme_matrix.rs     # Multi-theme view
│   ├── visual_testing.rs   # Screenshot comparison
│   └── a11y.rs             # Accessibility checks
│
├── stories/
│   ├── mod.rs              # Story collection
│   ├── button_stories.rs
│   ├── toast_stories.rs
│   ├── form_field_stories.rs
│   ├── list_item_stories.rs
│   ├── scrollbar_stories.rs
│   └── design_token_stories.rs
│
├── bin/
│   └── storybook.rs        # Standalone binary

tests/
├── smoke/
│   ├── design-gallery.ts    # Quick visual capture
│   └── component-showcase.ts
│
├── visual/
│   ├── baselines/           # PNG baseline images
│   └── visual-test-runner.ts

scripts/
├── storybook-dev.sh         # Dev mode with watch
└── generate-visual-baselines.ts

docs/
└── generated/
    └── theme-matrix.html    # Auto-generated theme comparison
```

---

## Command Reference

### Development Commands

```bash
# Quick capture of all designs (Tier 1)
bun run storybook:capture

# Capture all component states (Tier 1)
bun run storybook:components

# Run storybook browser (Tier 2)
cargo run -p storybook

# Run with specific theme
cargo run -p storybook -- --theme "One Dark"

# Jump to specific story
cargo run -p storybook -- --story "button-primary"

# Development mode with auto-reload (Tier 2)
./scripts/storybook-dev.sh

# Run visual regression tests (Tier 3)
cargo test --features visual-tests

# Update visual baselines
cargo run -p storybook -- --update-baselines
```

### CI/Automation Commands

```bash
# Headless visual test (CI)
./target/release/storybook --mode ci --output ./results

# Generate theme matrix HTML
cargo run -p storybook -- --generate-docs

# Export all stories as PNGs
cargo run -p storybook -- --export ./screenshots
```

---

## Implementation Priorities

### Phase 1: Foundation (Week 1)
1. ✅ Tier 1 scripts (design-gallery.ts, component-showcase.ts)
2. Define Story trait and layout helpers
3. Create 3-4 example stories for existing components
4. Basic StoryBrowser with left sidebar

### Phase 2: Core System (Week 2-3)
1. inventory-based story registry
2. Standalone storybook binary with CLI args
3. Theme switching in browser
4. Design variant matrix view
5. Search/filter functionality

### Phase 3: Testing Integration (Week 4)
1. Visual regression with pixel diff
2. CI workflow for visual tests
3. Baseline management
4. PR comments with diffs

### Phase 4: Polish (Ongoing)
1. Hot reload improvements
2. Accessibility testing
3. Documentation generation
4. Performance profiling view

---

## Key Design Decisions

### Why inventory crate over manual registration?
- Compile-time story discovery
- No central registration file to maintain
- Same pattern Zed uses successfully

### Why standalone binary vs in-app panel?
- Isolation: doesn't affect main app
- Performance: dedicated window
- CI: can run headless
- Same pattern as Zed's storybook

### Why screenshot-based visual testing?
- Works with any component
- Catches pixel-level regressions
- No AccessKit dependency for testing
- Proven pattern (egui_kittest, insta)

### Why not web-based like Storybook.js?
- No WebView dependency
- Native performance
- Direct GPUI rendering
- Simpler architecture

---

## Dependencies to Add

```toml
# Cargo.toml

[dependencies]
inventory = "0.3"  # Story registration
clap = { version = "4", features = ["derive"] }  # CLI args

[dev-dependencies]
image = "0.25"     # Screenshot comparison
dify = "0.7"       # Image diffing (optional)
insta = "1.40"     # Snapshot testing
```

---

## Success Metrics

| Metric | Target |
|--------|--------|
| Story coverage | 100% of public components |
| Visual test runtime | < 30s for full suite |
| Design variant coverage | All 11 variants |
| CI integration | PR blocking on visual regression |
| Documentation | Auto-generated from stories |

---

## References

- [Zed storybook crate](https://github.com/zed-industries/zed/tree/main/crates/storybook)
- [Zed story layout helpers](https://github.com/zed-industries/zed/tree/main/crates/story)
- [inventory crate](https://docs.rs/inventory)
- [insta snapshot testing](https://insta.rs/)
- [egui_kittest](https://github.com/emilk/egui/tree/master/crates/egui_kittest)
