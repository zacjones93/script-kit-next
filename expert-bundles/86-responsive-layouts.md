# Expert Bundle #86: Responsive Layouts

## Overview

Responsive layouts adapt to different window sizes and orientations. Script Kit windows can be resized by users or scripts, requiring layouts that gracefully handle various dimensions. Good responsive design maintains usability at all sizes.

## Architecture

### Layout Breakpoints

```rust
// src/responsive.rs
use gpui::*;

/// Window size categories
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowSize {
    /// < 400px width - minimal UI
    Compact,
    /// 400-600px width - standard prompt
    Small,
    /// 600-800px width - comfortable
    Medium,
    /// 800-1200px width - spacious
    Large,
    /// > 1200px width - extra room
    ExtraLarge,
}

impl WindowSize {
    pub fn from_width(width: f32) -> Self {
        match width {
            w if w < 400.0 => Self::Compact,
            w if w < 600.0 => Self::Small,
            w if w < 800.0 => Self::Medium,
            w if w < 1200.0 => Self::Large,
            _ => Self::ExtraLarge,
        }
    }
    
    pub fn min_width(&self) -> f32 {
        match self {
            Self::Compact => 0.0,
            Self::Small => 400.0,
            Self::Medium => 600.0,
            Self::Large => 800.0,
            Self::ExtraLarge => 1200.0,
        }
    }
}

/// Height categories for vertical layouts
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowHeight {
    /// < 300px - very constrained
    Short,
    /// 300-500px - typical prompt
    Medium,
    /// > 500px - tall window
    Tall,
}

impl WindowHeight {
    pub fn from_height(height: f32) -> Self {
        match height {
            h if h < 300.0 => Self::Short,
            h if h < 500.0 => Self::Medium,
            _ => Self::Tall,
        }
    }
}

/// Combined layout context
#[derive(Clone, Copy, Debug)]
pub struct LayoutContext {
    pub width: WindowSize,
    pub height: WindowHeight,
    pub aspect_ratio: f32,
    pub is_landscape: bool,
}

impl LayoutContext {
    pub fn from_size(size: Size<Pixels>) -> Self {
        let width_px = size.width.0;
        let height_px = size.height.0;
        
        Self {
            width: WindowSize::from_width(width_px),
            height: WindowHeight::from_height(height_px),
            aspect_ratio: width_px / height_px.max(1.0),
            is_landscape: width_px > height_px,
        }
    }
}
```

### Responsive Container

```rust
// src/components/responsive.rs
use gpui::*;

/// Container that adapts layout based on window size
pub struct ResponsiveContainer {
    children: Vec<AnyElement>,
    compact_layout: Box<dyn Fn(Vec<AnyElement>, &mut WindowContext) -> AnyElement>,
    default_layout: Box<dyn Fn(Vec<AnyElement>, &mut WindowContext) -> AnyElement>,
    large_layout: Option<Box<dyn Fn(Vec<AnyElement>, &mut WindowContext) -> AnyElement>>,
}

impl ResponsiveContainer {
    pub fn new() -> Self {
        Self {
            children: vec![],
            compact_layout: Box::new(|children, _| {
                div().flex().flex_col().children(children).into_any_element()
            }),
            default_layout: Box::new(|children, _| {
                div().flex().flex_col().children(children).into_any_element()
            }),
            large_layout: None,
        }
    }
    
    pub fn children(mut self, children: Vec<AnyElement>) -> Self {
        self.children = children;
        self
    }
    
    pub fn compact<F>(mut self, layout: F) -> Self 
    where 
        F: Fn(Vec<AnyElement>, &mut WindowContext) -> AnyElement + 'static 
    {
        self.compact_layout = Box::new(layout);
        self
    }
    
    pub fn default<F>(mut self, layout: F) -> Self 
    where 
        F: Fn(Vec<AnyElement>, &mut WindowContext) -> AnyElement + 'static 
    {
        self.default_layout = Box::new(layout);
        self
    }
    
    pub fn large<F>(mut self, layout: F) -> Self 
    where 
        F: Fn(Vec<AnyElement>, &mut WindowContext) -> AnyElement + 'static 
    {
        self.large_layout = Some(Box::new(layout));
        self
    }
}

impl Render for ResponsiveContainer {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let size = window.bounds().size;
        let layout_ctx = LayoutContext::from_size(size);
        let children = std::mem::take(&mut self.children);
        
        match layout_ctx.width {
            WindowSize::Compact => (self.compact_layout)(children, cx),
            WindowSize::Large | WindowSize::ExtraLarge => {
                if let Some(large) = &self.large_layout {
                    large(children, cx)
                } else {
                    (self.default_layout)(children, cx)
                }
            }
            _ => (self.default_layout)(children, cx),
        }
    }
}
```

### Responsive Helpers

```rust
// src/responsive/helpers.rs
use gpui::*;

/// Extension trait for responsive styling
pub trait ResponsiveExt {
    fn responsive_padding(self, layout: &LayoutContext) -> Self;
    fn responsive_gap(self, layout: &LayoutContext) -> Self;
    fn responsive_font_size(self, layout: &LayoutContext) -> Self;
    fn hide_below(self, min_width: WindowSize) -> Self;
    fn hide_above(self, max_width: WindowSize) -> Self;
}

impl<E: Styled> ResponsiveExt for E {
    fn responsive_padding(self, layout: &LayoutContext) -> Self {
        let padding = match layout.width {
            WindowSize::Compact => px(8.0),
            WindowSize::Small => px(12.0),
            WindowSize::Medium => px(16.0),
            WindowSize::Large => px(20.0),
            WindowSize::ExtraLarge => px(24.0),
        };
        self.p(padding)
    }
    
    fn responsive_gap(self, layout: &LayoutContext) -> Self {
        let gap = match layout.width {
            WindowSize::Compact => px(4.0),
            WindowSize::Small => px(8.0),
            WindowSize::Medium => px(12.0),
            WindowSize::Large | WindowSize::ExtraLarge => px(16.0),
        };
        self.gap(gap)
    }
    
    fn responsive_font_size(self, layout: &LayoutContext) -> Self {
        let size = match layout.width {
            WindowSize::Compact => px(12.0),
            WindowSize::Small | WindowSize::Medium => px(14.0),
            WindowSize::Large | WindowSize::ExtraLarge => px(16.0),
        };
        self.text_size(size)
    }
    
    fn hide_below(self, min_width: WindowSize) -> Self {
        // Would need layout context from render
        self
    }
    
    fn hide_above(self, max_width: WindowSize) -> Self {
        self
    }
}

/// Conditional rendering based on layout
pub fn when_width<E: IntoElement>(
    layout: &LayoutContext,
    min: WindowSize,
    element: impl FnOnce() -> E,
) -> Option<AnyElement> {
    if layout.width as u8 >= min as u8 {
        Some(element().into_any_element())
    } else {
        None
    }
}

pub fn unless_width<E: IntoElement>(
    layout: &LayoutContext,
    max: WindowSize,
    element: impl FnOnce() -> E,
) -> Option<AnyElement> {
    if (layout.width as u8) < (max as u8) {
        Some(element().into_any_element())
    } else {
        None
    }
}
```

## Layout Patterns

### Collapsible Sidebar

```rust
// Sidebar that collapses to icons on narrow windows
impl NotesApp {
    fn render_with_responsive_sidebar(&self, cx: &mut WindowContext) -> impl IntoElement {
        let size = cx.window_bounds().size;
        let layout = LayoutContext::from_size(size);
        let colors = &self.theme.colors;
        
        let sidebar_width = match layout.width {
            WindowSize::Compact => px(48.0),  // Icons only
            WindowSize::Small => px(180.0),   // Narrow labels
            _ => px(240.0),                   // Full sidebar
        };
        
        let show_labels = !matches!(layout.width, WindowSize::Compact);
        let show_search = matches!(layout.width, WindowSize::Medium | WindowSize::Large | WindowSize::ExtraLarge);
        
        div()
            .w_full()
            .h_full()
            .flex()
            .flex_row()
            // Sidebar
            .child(
                div()
                    .w(sidebar_width)
                    .h_full()
                    .border_r_1()
                    .border_color(rgb(colors.ui.border))
                    .flex()
                    .flex_col()
                    // Search (hidden on compact)
                    .when(show_search, |el| {
                        el.child(self.render_search(cx))
                    })
                    // Note list
                    .child(
                        div()
                            .flex_1()
                            .overflow_y_auto()
                            .children(self.notes.iter().map(|note| {
                                self.render_note_item(note, show_labels, cx)
                            }))
                    )
            )
            // Main content
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .child(self.render_editor(cx))
            )
    }
    
    fn render_note_item(&self, note: &Note, show_labels: bool, cx: &mut WindowContext) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        div()
            .h(px(if show_labels { 52.0 } else { 40.0 }))
            .px(px(if show_labels { 12.0 } else { 8.0 }))
            .flex()
            .items_center()
            .gap_2()
            .cursor_pointer()
            .hover(|s| s.bg(rgb(colors.ui.hover)))
            // Icon (always shown)
            .child(
                Icon::new("file-text")
                    .size(px(16.0))
                    .color(rgb(colors.text.secondary))
            )
            // Label (hidden on compact)
            .when(show_labels, |el| {
                el.child(
                    div()
                        .flex_1()
                        .truncate()
                        .text_size(px(13.0))
                        .text_color(rgb(colors.text.primary))
                        .child(&note.title)
                )
            })
    }
}
```

### Stacking on Narrow Windows

```rust
// Switch from side-by-side to stacked layout
impl FormPrompt {
    fn render_responsive_fields(&self, cx: &mut WindowContext) -> impl IntoElement {
        let size = cx.window_bounds().size;
        let layout = LayoutContext::from_size(size);
        let colors = &self.theme.colors;
        
        // Use horizontal layout for wide windows
        let use_horizontal = matches!(layout.width, WindowSize::Large | WindowSize::ExtraLarge);
        
        div()
            .w_full()
            .flex()
            .map(|el| if use_horizontal { el.flex_row() } else { el.flex_col() })
            .responsive_gap(&layout)
            .children(self.fields.iter().map(|field| {
                let field_width = if use_horizontal {
                    match self.fields.len() {
                        1 => "100%".into(),
                        2 => "50%".into(),
                        3 => "33.33%".into(),
                        _ => "25%".into(),
                    }
                } else {
                    "100%".into()
                };
                
                div()
                    .w_full()
                    .max_w(px(if use_horizontal { 300.0 } else { f32::MAX }))
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(rgb(colors.text.muted))
                            .child(&field.label)
                    )
                    .child(
                        self.render_field_input(field, cx)
                    )
            }))
    }
}
```

### Adaptive List Items

```rust
// List items that show more/less based on width
impl ScriptListItem {
    fn render_adaptive(&self, layout: &LayoutContext, cx: &mut WindowContext) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        let show_description = !matches!(layout.width, WindowSize::Compact);
        let show_shortcut = matches!(layout.width, WindowSize::Medium | WindowSize::Large | WindowSize::ExtraLarge);
        let show_category = matches!(layout.width, WindowSize::Large | WindowSize::ExtraLarge);
        
        div()
            .h(px(52.0))
            .px(px(match layout.width {
                WindowSize::Compact => 8.0,
                WindowSize::Small => 12.0,
                _ => 16.0,
            }))
            .flex()
            .items_center()
            .gap_3()
            // Icon
            .child(
                div()
                    .w(px(24.0))
                    .h(px(24.0))
                    .rounded_md()
                    .bg(rgb(colors.ui.surface))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(Icon::new(&self.script.icon).size(px(14.0)))
            )
            // Name and description
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .flex()
                    .flex_col()
                    .justify_center()
                    .child(
                        div()
                            .text_size(px(14.0))
                            .text_color(rgb(colors.text.primary))
                            .truncate()
                            .child(&self.script.name)
                    )
                    .when(show_description, |el| {
                        el.when_some(self.script.description.clone(), |el, desc| {
                            el.child(
                                div()
                                    .text_size(px(12.0))
                                    .text_color(rgb(colors.text.muted))
                                    .truncate()
                                    .child(desc)
                            )
                        })
                    })
            )
            // Category badge
            .when(show_category, |el| {
                el.when_some(self.script.category.clone(), |el, cat| {
                    el.child(
                        div()
                            .px_2()
                            .py_1()
                            .rounded_sm()
                            .bg(rgb(colors.ui.surface))
                            .text_size(px(10.0))
                            .text_color(rgb(colors.text.muted))
                            .child(cat)
                    )
                })
            })
            // Shortcut
            .when(show_shortcut, |el| {
                el.when_some(self.script.shortcut.clone(), |el, sc| {
                    el.child(
                        div()
                            .text_size(px(11.0))
                            .font_family("monospace")
                            .text_color(rgb(colors.text.muted))
                            .child(sc.display())
                    )
                })
            })
    }
}
```

## Testing

### Responsive Layout Test Script

```typescript
// tests/smoke/test-responsive-layouts.ts
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Test different widths by simulating layouts

// Test 1: Compact width (< 400px)
await div(`
  <div class="w-80 bg-zinc-900 rounded-lg overflow-hidden">
    <div class="flex">
      <!-- Collapsed sidebar (icons only) -->
      <div class="w-12 border-r border-zinc-700 py-2">
        <div class="w-10 h-10 mx-auto flex items-center justify-center hover:bg-zinc-800 rounded-md cursor-pointer">
          <span class="text-sm">📄</span>
        </div>
        <div class="w-10 h-10 mx-auto flex items-center justify-center bg-zinc-800 rounded-md cursor-pointer">
          <span class="text-sm">📝</span>
        </div>
        <div class="w-10 h-10 mx-auto flex items-center justify-center hover:bg-zinc-800 rounded-md cursor-pointer">
          <span class="text-sm">⚙️</span>
        </div>
      </div>
      <!-- Main content -->
      <div class="flex-1 p-2">
        <div class="text-xs text-zinc-400 mb-2">Compact Layout</div>
        <div class="h-8 bg-zinc-800 rounded-md"></div>
      </div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot1 = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
writeFileSync(join(dir, 'responsive-compact.png'), Buffer.from(shot1.data, 'base64'));

// Test 2: Medium width (600-800px)
await div(`
  <div class="w-[600px] bg-zinc-900 rounded-lg overflow-hidden">
    <div class="flex">
      <!-- Sidebar with labels -->
      <div class="w-44 border-r border-zinc-700 p-2">
        <div class="h-9 px-3 flex items-center gap-2 hover:bg-zinc-800 rounded-md cursor-pointer">
          <span class="text-sm">📄</span>
          <span class="text-sm text-white truncate">Documents</span>
        </div>
        <div class="h-9 px-3 flex items-center gap-2 bg-zinc-800 rounded-md cursor-pointer">
          <span class="text-sm">📝</span>
          <span class="text-sm text-white truncate">Notes</span>
        </div>
        <div class="h-9 px-3 flex items-center gap-2 hover:bg-zinc-800 rounded-md cursor-pointer">
          <span class="text-sm">⚙️</span>
          <span class="text-sm text-white truncate">Settings</span>
        </div>
      </div>
      <!-- Main content -->
      <div class="flex-1 p-4">
        <div class="text-sm text-zinc-400 mb-2">Medium Layout</div>
        <div class="h-24 bg-zinc-800 rounded-md"></div>
      </div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot2 = await captureScreenshot();
writeFileSync(join(dir, 'responsive-medium.png'), Buffer.from(shot2.data, 'base64'));

// Test 3: Form layout adaptation
await div(`
  <div class="p-4 flex flex-col gap-4">
    <!-- Narrow: stacked -->
    <div class="w-80 bg-zinc-800 rounded-lg p-4">
      <div class="text-xs text-zinc-500 mb-2">Narrow (stacked)</div>
      <div class="flex flex-col gap-3">
        <div class="flex flex-col gap-1">
          <span class="text-xs text-zinc-400">Name</span>
          <div class="h-9 bg-zinc-700 rounded-md"></div>
        </div>
        <div class="flex flex-col gap-1">
          <span class="text-xs text-zinc-400">Email</span>
          <div class="h-9 bg-zinc-700 rounded-md"></div>
        </div>
      </div>
    </div>
    
    <!-- Wide: side by side -->
    <div class="w-[500px] bg-zinc-800 rounded-lg p-4">
      <div class="text-xs text-zinc-500 mb-2">Wide (horizontal)</div>
      <div class="flex gap-3">
        <div class="flex-1 flex flex-col gap-1">
          <span class="text-xs text-zinc-400">Name</span>
          <div class="h-9 bg-zinc-700 rounded-md"></div>
        </div>
        <div class="flex-1 flex flex-col gap-1">
          <span class="text-xs text-zinc-400">Email</span>
          <div class="h-9 bg-zinc-700 rounded-md"></div>
        </div>
      </div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot3 = await captureScreenshot();
writeFileSync(join(dir, 'responsive-form.png'), Buffer.from(shot3.data, 'base64'));

// Test 4: List item adaptation
await div(`
  <div class="p-4 flex flex-col gap-4">
    <!-- Compact list item -->
    <div class="w-64 h-10 px-2 flex items-center gap-2 bg-zinc-800 rounded-md">
      <span class="text-sm">📄</span>
      <span class="text-sm text-white truncate flex-1">My Script</span>
    </div>
    
    <!-- Medium list item -->
    <div class="w-96 h-[52px] px-3 flex items-center gap-3 bg-zinc-800 rounded-md">
      <div class="w-6 h-6 rounded-md bg-zinc-700 flex items-center justify-center">
        <span class="text-xs">📄</span>
      </div>
      <div class="flex-1">
        <div class="text-sm text-white">My Script</div>
        <div class="text-xs text-zinc-500 truncate">A helpful automation</div>
      </div>
    </div>
    
    <!-- Large list item -->
    <div class="w-[600px] h-[52px] px-4 flex items-center gap-3 bg-zinc-800 rounded-md">
      <div class="w-6 h-6 rounded-md bg-zinc-700 flex items-center justify-center">
        <span class="text-xs">📄</span>
      </div>
      <div class="flex-1">
        <div class="text-sm text-white">My Script</div>
        <div class="text-xs text-zinc-500 truncate">A helpful automation script</div>
      </div>
      <span class="px-2 py-1 rounded-sm bg-zinc-700 text-[10px] text-zinc-400">tools</span>
      <span class="text-xs text-zinc-500 font-mono">⌘K</span>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot4 = await captureScreenshot();
writeFileSync(join(dir, 'responsive-list-items.png'), Buffer.from(shot4.data, 'base64'));

console.error('[RESPONSIVE LAYOUTS] Test screenshots saved');
process.exit(0);
```

## Related Bundles

- Bundle #12: Window Management - Window sizing
- Bundle #64: List Virtualization - List adaptation
- Bundle #63: Config System - User size preferences
