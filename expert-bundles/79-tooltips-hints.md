# Expert Bundle #79: Tooltips & Hints

## Overview

Tooltips and hints provide contextual help without cluttering the interface. In Script Kit, they explain keyboard shortcuts, describe actions, show full text for truncated items, and guide new users. Good tooltips are discoverable, non-intrusive, and informative.

## Architecture

### Tooltip Types

```rust
// src/tooltips.rs
use gpui::*;
use std::time::Duration;

/// Different tooltip presentation styles
#[derive(Clone, Debug)]
pub enum TooltipStyle {
    /// Simple text tooltip
    Standard {
        text: SharedString,
    },
    /// Tooltip with keyboard shortcut
    WithShortcut {
        text: SharedString,
        shortcut: Vec<SharedString>, // ["⌘", "K"]
    },
    /// Rich tooltip with title and description
    Rich {
        title: SharedString,
        description: Option<SharedString>,
        shortcut: Option<Vec<SharedString>>,
    },
    /// Interactive hint with action
    Hint {
        text: SharedString,
        action_label: Option<SharedString>,
        dismissible: bool,
    },
}

/// Tooltip positioning relative to trigger
#[derive(Clone, Copy, Debug, Default)]
pub enum TooltipPosition {
    Top,
    TopStart,
    TopEnd,
    #[default]
    Bottom,
    BottomStart,
    BottomEnd,
    Left,
    Right,
}

/// Tooltip configuration
pub struct TooltipConfig {
    pub style: TooltipStyle,
    pub position: TooltipPosition,
    pub delay: Duration,
    pub offset: f32,
    pub max_width: Option<f32>,
}

impl Default for TooltipConfig {
    fn default() -> Self {
        Self {
            style: TooltipStyle::Standard { text: "".into() },
            position: TooltipPosition::Bottom,
            delay: Duration::from_millis(500), // Standard delay
            offset: 4.0, // Gap between trigger and tooltip
            max_width: Some(250.0),
        }
    }
}
```

### Tooltip Component

```rust
// src/components/tooltip.rs
use crate::theme::Theme;
use gpui::*;

pub struct Tooltip {
    config: TooltipConfig,
    theme: Arc<Theme>,
    visible: bool,
    trigger_bounds: Option<Bounds<Pixels>>,
}

impl Tooltip {
    pub fn new(config: TooltipConfig, theme: Arc<Theme>) -> Self {
        Self {
            config,
            theme,
            visible: false,
            trigger_bounds: None,
        }
    }
    
    fn calculate_position(&self, window_bounds: Bounds<Pixels>) -> Point<Pixels> {
        let trigger = self.trigger_bounds.unwrap_or_default();
        let offset = px(self.config.offset);
        
        match self.config.position {
            TooltipPosition::Top => Point::new(
                trigger.center().x,
                trigger.origin.y - offset,
            ),
            TooltipPosition::TopStart => Point::new(
                trigger.origin.x,
                trigger.origin.y - offset,
            ),
            TooltipPosition::Bottom => Point::new(
                trigger.center().x,
                trigger.origin.y + trigger.size.height + offset,
            ),
            TooltipPosition::BottomStart => Point::new(
                trigger.origin.x,
                trigger.origin.y + trigger.size.height + offset,
            ),
            TooltipPosition::Left => Point::new(
                trigger.origin.x - offset,
                trigger.center().y,
            ),
            TooltipPosition::Right => Point::new(
                trigger.origin.x + trigger.size.width + offset,
                trigger.center().y,
            ),
            _ => trigger.center(),
        }
    }
}

impl Render for Tooltip {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.visible {
            return div().into_any_element();
        }
        
        let colors = &self.theme.colors;
        
        let content = match &self.config.style {
            TooltipStyle::Standard { text } => {
                div()
                    .px_2()
                    .py_1()
                    .rounded_md()
                    .bg(rgb(colors.ui.tooltip_bg))
                    .shadow_sm()
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(rgb(colors.ui.tooltip_text))
                            .child(text.clone())
                    )
                    .into_any_element()
            }
            
            TooltipStyle::WithShortcut { text, shortcut } => {
                div()
                    .px_2()
                    .py_1()
                    .rounded_md()
                    .bg(rgb(colors.ui.tooltip_bg))
                    .shadow_sm()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .text_color(rgb(colors.ui.tooltip_text))
                                    .child(text.clone())
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap_1()
                                    .children(shortcut.iter().map(|key| {
                                        div()
                                            .px_1()
                                            .py_px()
                                            .rounded_sm()
                                            .bg(rgb(colors.ui.kbd_bg))
                                            .text_size(px(10.0))
                                            .font_family("monospace")
                                            .text_color(rgb(colors.ui.kbd_text))
                                            .child(key.clone())
                                    }))
                            )
                    )
                    .into_any_element()
            }
            
            TooltipStyle::Rich { title, description, shortcut } => {
                div()
                    .p_3()
                    .rounded_lg()
                    .bg(rgb(colors.ui.tooltip_bg))
                    .shadow_md()
                    .max_w(px(self.config.max_width.unwrap_or(250.0)))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            // Title row with optional shortcut
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div()
                                            .text_size(px(13.0))
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(rgb(colors.ui.tooltip_text))
                                            .child(title.clone())
                                    )
                                    .when_some(shortcut.clone(), |el, keys| {
                                        el.child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .gap_1()
                                                .children(keys.iter().map(|key| {
                                                    div()
                                                        .px_1()
                                                        .py_px()
                                                        .rounded_sm()
                                                        .bg(rgb(colors.ui.kbd_bg))
                                                        .text_size(px(10.0))
                                                        .child(key.clone())
                                                }))
                                        )
                                    })
                            )
                            // Optional description
                            .when_some(description.clone(), |el, desc| {
                                el.child(
                                    div()
                                        .text_size(px(11.0))
                                        .text_color(rgb(colors.text.muted))
                                        .child(desc)
                                )
                            })
                    )
                    .into_any_element()
            }
            
            TooltipStyle::Hint { text, action_label, dismissible } => {
                div()
                    .p_3()
                    .rounded_lg()
                    .bg(rgb(colors.accent.primary))
                    .shadow_lg()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_start()
                            .gap_2()
                            // Hint icon
                            .child(
                                Icon::new("lightbulb")
                                    .size(px(14.0))
                                    .color(rgb(colors.background.main))
                            )
                            // Content
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_size(px(12.0))
                                            .text_color(rgb(colors.background.main))
                                            .child(text.clone())
                                    )
                                    .when_some(action_label.clone(), |el, label| {
                                        el.child(
                                            div()
                                                .text_size(px(11.0))
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(rgb(colors.background.main))
                                                .opacity(0.9)
                                                .cursor_pointer()
                                                .child(label)
                                        )
                                    })
                            )
                            // Dismiss button
                            .when(*dismissible, |el| {
                                el.child(
                                    div()
                                        .cursor_pointer()
                                        .child(
                                            Icon::new("x")
                                                .size(px(12.0))
                                                .color(rgb(colors.background.main))
                                        )
                                )
                            })
                    )
                    .into_any_element()
            }
        };
        
        // Position the tooltip
        div()
            .absolute()
            .z_index(1000)
            .map(|el| {
                if let Some(bounds) = self.trigger_bounds {
                    let pos = self.calculate_position(bounds);
                    el.left(pos.x).top(pos.y)
                } else {
                    el
                }
            })
            .child(content)
            .into_any_element()
    }
}
```

### Tooltip Trigger System

```rust
// src/tooltips/trigger.rs
use gpui::*;

/// Makes any element show a tooltip on hover
pub trait TooltipExt: Sized {
    fn tooltip(self, text: impl Into<SharedString>) -> TooltipWrapper<Self>;
    fn tooltip_with_shortcut(
        self, 
        text: impl Into<SharedString>, 
        shortcut: Vec<&str>
    ) -> TooltipWrapper<Self>;
    fn tooltip_rich(
        self,
        title: impl Into<SharedString>,
        description: Option<impl Into<SharedString>>,
    ) -> TooltipWrapper<Self>;
}

pub struct TooltipWrapper<E> {
    element: E,
    config: TooltipConfig,
}

impl<E: IntoElement> TooltipWrapper<E> {
    pub fn position(mut self, position: TooltipPosition) -> Self {
        self.config.position = position;
        self
    }
    
    pub fn delay(mut self, delay: Duration) -> Self {
        self.config.delay = delay;
        self
    }
}

impl<E: IntoElement> IntoElement for TooltipWrapper<E> {
    fn into_element(self) -> impl Element {
        TooltipTrigger {
            child: self.element.into_any_element(),
            config: self.config,
        }
    }
}

struct TooltipTrigger {
    child: AnyElement,
    config: TooltipConfig,
}

impl Element for TooltipTrigger {
    fn render(&mut self, window: &mut Window, cx: &mut WindowContext) -> impl IntoElement {
        let config = self.config.clone();
        
        div()
            .child(self.child.clone())
            .on_mouse_enter(move |_, window, cx| {
                // Start delay timer
                let config = config.clone();
                cx.spawn(|_, mut cx| async move {
                    Timer::after(config.delay).await;
                    // Show tooltip via global manager
                    cx.update(|cx| {
                        cx.emit(ShowTooltip { config });
                    }).ok();
                }).detach();
            })
            .on_mouse_leave(|_, window, cx| {
                cx.emit(HideTooltip);
            })
    }
}
```

## Usage Patterns

### Button Tooltips

```rust
// Simple tooltip on icon button
impl Toolbar {
    fn render_icon_button(&self, icon: &str, tooltip: &str, cx: &mut WindowContext) -> impl IntoElement {
        div()
            .p_2()
            .rounded_md()
            .cursor_pointer()
            .hover(|s| s.bg(rgb(colors.ui.hover)))
            .child(Icon::new(icon).size(px(16.0)))
            .tooltip(tooltip)
    }
    
    // Tooltip with keyboard shortcut
    fn render_action_button(&self, cx: &mut WindowContext) -> impl IntoElement {
        div()
            .px_3()
            .py_2()
            .rounded_md()
            .bg(rgb(colors.accent.primary))
            .child("Run Script")
            .tooltip_with_shortcut("Run the current script", vec!["⌘", "Enter"])
            .position(TooltipPosition::Bottom)
    }
}
```

### List Item Tooltips

```rust
// Tooltip for truncated text
impl ListItem {
    fn render(&self, cx: &mut WindowContext) -> impl IntoElement {
        let needs_tooltip = self.name.len() > 30;
        
        div()
            .h(px(52.0))
            .px_4()
            .flex()
            .items_center()
            .child(
                div()
                    .flex_1()
                    .truncate()
                    .child(&self.name)
            )
            .when(needs_tooltip, |el| {
                el.tooltip(&self.name)
                    .delay(Duration::from_millis(800)) // Longer delay for text
            })
    }
}
```

### Hint Overlays

```rust
// First-time user hint
impl MainMenu {
    fn render_with_hints(&self, cx: &mut WindowContext) -> impl IntoElement {
        let show_hint = self.is_first_run && !self.hint_dismissed;
        
        div()
            .relative()
            .child(self.render_main_content(cx))
            .when(show_hint, |el| {
                el.child(
                    div()
                        .absolute()
                        .top_4()
                        .right_4()
                        .child(
                            Tooltip::new(
                                TooltipConfig {
                                    style: TooltipStyle::Hint {
                                        text: "Type to search scripts, or press ⌘K for actions".into(),
                                        action_label: Some("Got it".into()),
                                        dismissible: true,
                                    },
                                    position: TooltipPosition::BottomEnd,
                                    ..Default::default()
                                },
                                self.theme.clone(),
                            )
                        )
                )
            })
    }
}
```

### Contextual Help

```rust
// Help icon with rich tooltip
impl SettingsPanel {
    fn render_setting_row(&self, setting: &Setting, cx: &mut WindowContext) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_size(px(13.0))
                            .child(&setting.label)
                    )
                    .when(setting.help_text.is_some(), |el| {
                        el.child(
                            div()
                                .cursor_help()
                                .child(
                                    Icon::new("help-circle")
                                        .size(px(14.0))
                                        .color(rgb(colors.text.muted))
                                )
                                .tooltip_rich(
                                    &setting.label,
                                    setting.help_text.as_deref(),
                                )
                                .position(TooltipPosition::Right)
                        )
                    })
            )
            .child(/* setting control */)
    }
}
```

## Timing & Animation

### Delay Strategies

```rust
pub mod tooltip_delays {
    use std::time::Duration;
    
    /// Instant tooltips for important info
    pub const INSTANT: Duration = Duration::from_millis(0);
    
    /// Quick tooltips for keyboard shortcuts
    pub const QUICK: Duration = Duration::from_millis(300);
    
    /// Standard delay for most tooltips
    pub const STANDARD: Duration = Duration::from_millis(500);
    
    /// Longer delay for truncated text
    pub const EXTENDED: Duration = Duration::from_millis(800);
    
    /// Hide delay when moving between tooltips
    pub const HIDE_DELAY: Duration = Duration::from_millis(100);
}
```

### Fade Animation

```rust
impl Tooltip {
    fn render_animated(&self, cx: &mut WindowContext) -> impl IntoElement {
        div()
            .with_animation(
                "tooltip-fade",
                Animation::new()
                    .duration(Duration::from_millis(150))
                    .easing(ease_out_cubic),
                move |el, progress| {
                    el.opacity(progress)
                        .translate_y(px((1.0 - progress) * 4.0))
                },
            )
            .child(/* tooltip content */)
    }
}
```

## Best Practices

### Content Guidelines

```rust
/// Tooltip content best practices
pub mod guidelines {
    // Keep tooltips short (1-5 words ideal)
    // ✅ "Copy to clipboard"
    // ❌ "Click this button to copy the selected text to your clipboard"
    
    // Use sentence case
    // ✅ "Open settings"
    // ❌ "Open Settings"
    
    // Don't repeat visible labels
    // Button says "Save" → tooltip: "⌘S" not "Save file"
    
    // Show shortcuts when available
    // ✅ "Undo ⌘Z"
    // ❌ "Undo"
}

/// Keyboard shortcut formatting
pub fn format_shortcut(keys: &[&str]) -> Vec<SharedString> {
    keys.iter()
        .map(|k| match *k {
            "cmd" | "meta" => "⌘".into(),
            "ctrl" | "control" => "⌃".into(),
            "alt" | "option" => "⌥".into(),
            "shift" => "⇧".into(),
            "enter" | "return" => "↵".into(),
            "escape" | "esc" => "⎋".into(),
            "tab" => "⇥".into(),
            "backspace" => "⌫".into(),
            "delete" => "⌦".into(),
            "up" => "↑".into(),
            "down" => "↓".into(),
            "left" => "←".into(),
            "right" => "→".into(),
            "space" => "Space".into(),
            other => other.to_uppercase().into(),
        })
        .collect()
}
```

### Positioning Logic

```rust
impl Tooltip {
    /// Calculate best position avoiding viewport edges
    fn best_position(
        &self,
        trigger: Bounds<Pixels>,
        tooltip_size: Size<Pixels>,
        viewport: Bounds<Pixels>,
    ) -> TooltipPosition {
        let preferred = self.config.position;
        
        // Check if preferred position fits
        let fits = match preferred {
            TooltipPosition::Top | TooltipPosition::TopStart | TooltipPosition::TopEnd => {
                trigger.origin.y - tooltip_size.height - px(8.0) > viewport.origin.y
            }
            TooltipPosition::Bottom | TooltipPosition::BottomStart | TooltipPosition::BottomEnd => {
                trigger.origin.y + trigger.size.height + tooltip_size.height + px(8.0) 
                    < viewport.origin.y + viewport.size.height
            }
            TooltipPosition::Left => {
                trigger.origin.x - tooltip_size.width - px(8.0) > viewport.origin.x
            }
            TooltipPosition::Right => {
                trigger.origin.x + trigger.size.width + tooltip_size.width + px(8.0)
                    < viewport.origin.x + viewport.size.width
            }
        };
        
        if fits {
            preferred
        } else {
            // Flip to opposite side
            match preferred {
                TooltipPosition::Top => TooltipPosition::Bottom,
                TooltipPosition::TopStart => TooltipPosition::BottomStart,
                TooltipPosition::TopEnd => TooltipPosition::BottomEnd,
                TooltipPosition::Bottom => TooltipPosition::Top,
                TooltipPosition::BottomStart => TooltipPosition::TopStart,
                TooltipPosition::BottomEnd => TooltipPosition::TopEnd,
                TooltipPosition::Left => TooltipPosition::Right,
                TooltipPosition::Right => TooltipPosition::Left,
            }
        }
    }
}
```

## Testing

### Tooltip Test Script

```typescript
// tests/smoke/test-tooltips.ts
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Test 1: Simple tooltip
await div(`
  <div class="p-8 flex items-center justify-center">
    <div class="relative">
      <button class="p-2 rounded-md bg-zinc-700 hover:bg-zinc-600">
        <span class="text-xl">📋</span>
      </button>
      <div class="absolute top-full left-1/2 -translate-x-1/2 mt-1 px-2 py-1 rounded-md bg-zinc-900 shadow-sm">
        <span class="text-xs text-zinc-300">Copy</span>
      </div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot1 = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
writeFileSync(join(dir, 'tooltip-simple.png'), Buffer.from(shot1.data, 'base64'));

// Test 2: Tooltip with shortcut
await div(`
  <div class="p-8 flex items-center justify-center">
    <div class="relative">
      <button class="px-4 py-2 rounded-md bg-amber-500 text-black font-medium">
        Run Script
      </button>
      <div class="absolute top-full left-1/2 -translate-x-1/2 mt-1 px-2 py-1 rounded-md bg-zinc-900 shadow-sm">
        <div class="flex items-center gap-3">
          <span class="text-xs text-zinc-300">Run the current script</span>
          <div class="flex gap-1">
            <span class="px-1 py-0.5 rounded bg-zinc-700 text-[10px] font-mono text-zinc-400">⌘</span>
            <span class="px-1 py-0.5 rounded bg-zinc-700 text-[10px] font-mono text-zinc-400">↵</span>
          </div>
        </div>
      </div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot2 = await captureScreenshot();
writeFileSync(join(dir, 'tooltip-shortcut.png'), Buffer.from(shot2.data, 'base64'));

// Test 3: Rich tooltip
await div(`
  <div class="p-8 flex items-center justify-center">
    <div class="relative">
      <div class="flex items-center gap-2">
        <span class="text-sm text-white">API Key</span>
        <span class="text-zinc-500 cursor-help">ⓘ</span>
      </div>
      <div class="absolute left-full top-1/2 -translate-y-1/2 ml-2 p-3 rounded-lg bg-zinc-900 shadow-md max-w-[250px]">
        <div class="text-sm font-medium text-white">API Key</div>
        <div class="text-xs text-zinc-400 mt-1">
          Your API key is used to authenticate requests. Keep it secret and never share it publicly.
        </div>
      </div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot3 = await captureScreenshot();
writeFileSync(join(dir, 'tooltip-rich.png'), Buffer.from(shot3.data, 'base64'));

// Test 4: Hint tooltip
await div(`
  <div class="p-8">
    <div class="relative">
      <input 
        type="text" 
        placeholder="Search scripts..."
        class="w-full px-4 py-2 rounded-md bg-zinc-800 text-white border border-zinc-700"
      />
      <div class="absolute top-full right-0 mt-2 p-3 rounded-lg bg-amber-500 shadow-lg">
        <div class="flex items-start gap-2">
          <span class="text-black">💡</span>
          <div class="flex flex-col gap-2">
            <span class="text-xs text-black">
              Type to search scripts, or press ⌘K for actions
            </span>
            <span class="text-xs font-medium text-black/80 cursor-pointer">
              Got it
            </span>
          </div>
          <span class="text-black/60 cursor-pointer text-xs">×</span>
        </div>
      </div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot4 = await captureScreenshot();
writeFileSync(join(dir, 'tooltip-hint.png'), Buffer.from(shot4.data, 'base64'));

console.error('[TOOLTIPS] Test screenshots saved');
process.exit(0);
```

## Related Bundles

- Bundle #82: Keyboard Shortcuts UX - Shortcut display in tooltips
- Bundle #88: Onboarding UX - Hint tooltips for new users
- Bundle #79: Modal Dialogs - When to use tooltip vs. dialog
