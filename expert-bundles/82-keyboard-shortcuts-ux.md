# Expert Bundle #82: Keyboard Shortcuts UX

## Overview

Keyboard shortcuts are essential for power users in Script Kit. Good shortcut UX includes discoverability, consistency, conflict avoidance, and customization. Users should easily learn shortcuts through tooltips, menus, and dedicated help screens.

## Architecture

### Shortcut System

```rust
// src/shortcuts.rs
use gpui::*;
use std::collections::HashMap;

/// Represents a keyboard shortcut
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Shortcut {
    pub modifiers: Modifiers,
    pub key: String,
}

impl Shortcut {
    pub fn new(modifiers: Modifiers, key: impl Into<String>) -> Self {
        Self {
            modifiers,
            key: key.into(),
        }
    }
    
    /// Parse from string like "cmd+shift+k"
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('+').collect();
        if parts.is_empty() {
            return None;
        }
        
        let mut modifiers = Modifiers::default();
        let mut key = String::new();
        
        for part in parts {
            match part.to_lowercase().as_str() {
                "cmd" | "meta" | "command" => modifiers.cmd = true,
                "ctrl" | "control" => modifiers.ctrl = true,
                "alt" | "option" => modifiers.alt = true,
                "shift" => modifiers.shift = true,
                k => key = k.to_string(),
            }
        }
        
        if key.is_empty() {
            None
        } else {
            Some(Self { modifiers, key })
        }
    }
    
    /// Format for display
    pub fn display(&self) -> String {
        let mut parts = Vec::new();
        
        if self.modifiers.ctrl {
            parts.push("⌃");
        }
        if self.modifiers.alt {
            parts.push("⌥");
        }
        if self.modifiers.shift {
            parts.push("⇧");
        }
        if self.modifiers.cmd {
            parts.push("⌘");
        }
        
        parts.push(&self.format_key());
        parts.join("")
    }
    
    /// Format for display as separate badges
    pub fn display_parts(&self) -> Vec<String> {
        let mut parts = Vec::new();
        
        if self.modifiers.ctrl {
            parts.push("⌃".to_string());
        }
        if self.modifiers.alt {
            parts.push("⌥".to_string());
        }
        if self.modifiers.shift {
            parts.push("⇧".to_string());
        }
        if self.modifiers.cmd {
            parts.push("⌘".to_string());
        }
        
        parts.push(self.format_key());
        parts
    }
    
    fn format_key(&self) -> String {
        match self.key.to_lowercase().as_str() {
            "enter" | "return" => "↵".to_string(),
            "escape" | "esc" => "⎋".to_string(),
            "tab" => "⇥".to_string(),
            "backspace" => "⌫".to_string(),
            "delete" => "⌦".to_string(),
            "space" => "Space".to_string(),
            "up" | "arrowup" => "↑".to_string(),
            "down" | "arrowdown" => "↓".to_string(),
            "left" | "arrowleft" => "←".to_string(),
            "right" | "arrowright" => "→".to_string(),
            k => k.to_uppercase(),
        }
    }
}

#[derive(Clone, Default)]
pub struct Modifiers {
    pub cmd: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}
```

### Shortcut Registry

```rust
// src/shortcuts/registry.rs
use std::collections::HashMap;

/// Global shortcut registry
pub struct ShortcutRegistry {
    shortcuts: HashMap<String, ShortcutBinding>,
    categories: Vec<ShortcutCategory>,
    conflicts: Vec<ShortcutConflict>,
}

#[derive(Clone)]
pub struct ShortcutBinding {
    pub id: String,
    pub shortcut: Shortcut,
    pub action: SharedString,
    pub description: SharedString,
    pub category: String,
    pub customizable: bool,
    pub context: ShortcutContext,
}

#[derive(Clone, Copy, Debug)]
pub enum ShortcutContext {
    Global,          // Works anywhere
    MainMenu,        // Only in main menu
    Editor,          // Only in editor
    List,            // Any list context
    Dialog,          // In dialogs
}

#[derive(Clone)]
pub struct ShortcutCategory {
    pub id: String,
    pub label: SharedString,
    pub icon: Option<SharedString>,
}

#[derive(Clone)]
pub struct ShortcutConflict {
    pub shortcut: Shortcut,
    pub bindings: Vec<String>, // IDs of conflicting bindings
}

impl ShortcutRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            shortcuts: HashMap::new(),
            categories: vec![
                ShortcutCategory { id: "navigation".into(), label: "Navigation".into(), icon: Some("navigation".into()) },
                ShortcutCategory { id: "actions".into(), label: "Actions".into(), icon: Some("zap".into()) },
                ShortcutCategory { id: "editing".into(), label: "Editing".into(), icon: Some("edit".into()) },
                ShortcutCategory { id: "window".into(), label: "Window".into(), icon: Some("layout".into()) },
            ],
            conflicts: vec![],
        };
        
        registry.register_defaults();
        registry
    }
    
    fn register_defaults(&mut self) {
        // Navigation
        self.register(ShortcutBinding {
            id: "nav.up".into(),
            shortcut: Shortcut::parse("up").unwrap(),
            action: "Move up".into(),
            description: "Select previous item".into(),
            category: "navigation".into(),
            customizable: false,
            context: ShortcutContext::List,
        });
        
        self.register(ShortcutBinding {
            id: "nav.down".into(),
            shortcut: Shortcut::parse("down").unwrap(),
            action: "Move down".into(),
            description: "Select next item".into(),
            category: "navigation".into(),
            customizable: false,
            context: ShortcutContext::List,
        });
        
        // Actions
        self.register(ShortcutBinding {
            id: "action.run".into(),
            shortcut: Shortcut::parse("cmd+enter").unwrap(),
            action: "Run".into(),
            description: "Execute selected script".into(),
            category: "actions".into(),
            customizable: true,
            context: ShortcutContext::MainMenu,
        });
        
        self.register(ShortcutBinding {
            id: "action.actions".into(),
            shortcut: Shortcut::parse("tab").unwrap(),
            action: "Actions".into(),
            description: "Show available actions".into(),
            category: "actions".into(),
            customizable: true,
            context: ShortcutContext::MainMenu,
        });
        
        // Window
        self.register(ShortcutBinding {
            id: "window.close".into(),
            shortcut: Shortcut::parse("escape").unwrap(),
            action: "Close".into(),
            description: "Close window or cancel".into(),
            category: "window".into(),
            customizable: false,
            context: ShortcutContext::Global,
        });
    }
    
    pub fn register(&mut self, binding: ShortcutBinding) {
        // Check for conflicts
        for existing in self.shortcuts.values() {
            if existing.shortcut == binding.shortcut 
                && self.contexts_overlap(existing.context, binding.context) {
                self.conflicts.push(ShortcutConflict {
                    shortcut: binding.shortcut.clone(),
                    bindings: vec![existing.id.clone(), binding.id.clone()],
                });
            }
        }
        
        self.shortcuts.insert(binding.id.clone(), binding);
    }
    
    pub fn get_for_context(&self, context: ShortcutContext) -> Vec<&ShortcutBinding> {
        self.shortcuts.values()
            .filter(|b| self.context_matches(b.context, context))
            .collect()
    }
    
    fn contexts_overlap(&self, a: ShortcutContext, b: ShortcutContext) -> bool {
        matches!((a, b), 
            (ShortcutContext::Global, _) | 
            (_, ShortcutContext::Global) |
            (x, y) if std::mem::discriminant(&x) == std::mem::discriminant(&y)
        )
    }
    
    fn context_matches(&self, binding_context: ShortcutContext, current: ShortcutContext) -> bool {
        matches!(binding_context, ShortcutContext::Global) 
            || std::mem::discriminant(&binding_context) == std::mem::discriminant(&current)
    }
}
```

### Shortcut Display Components

```rust
// src/components/shortcut_badge.rs
use crate::theme::Theme;
use gpui::*;

/// Display a keyboard shortcut as styled badges
pub struct ShortcutBadge {
    shortcut: Shortcut,
    theme: Arc<Theme>,
    size: ShortcutSize,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum ShortcutSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl ShortcutBadge {
    pub fn new(shortcut: Shortcut, theme: Arc<Theme>) -> Self {
        Self {
            shortcut,
            theme,
            size: ShortcutSize::default(),
        }
    }
    
    pub fn size(mut self, size: ShortcutSize) -> Self {
        self.size = size;
        self
    }
}

impl Render for ShortcutBadge {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        let (font_size, padding_x, padding_y, gap) = match self.size {
            ShortcutSize::Small => (px(9.0), px(3.0), px(1.0), px(2.0)),
            ShortcutSize::Medium => (px(10.0), px(4.0), px(2.0), px(3.0)),
            ShortcutSize::Large => (px(12.0), px(6.0), px(3.0), px(4.0)),
        };
        
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(gap)
            .children(self.shortcut.display_parts().into_iter().map(|key| {
                div()
                    .px(padding_x)
                    .py(padding_y)
                    .rounded_sm()
                    .bg(rgb(colors.ui.kbd_bg))
                    .border_1()
                    .border_color(rgb(colors.ui.kbd_border))
                    .text_size(font_size)
                    .font_family("monospace")
                    .text_color(rgb(colors.ui.kbd_text))
                    .child(key)
            }))
    }
}

/// Inline shortcut hint (e.g., in menu items)
pub struct ShortcutHint {
    shortcut: Shortcut,
    theme: Arc<Theme>,
}

impl ShortcutHint {
    pub fn new(shortcut: Shortcut, theme: Arc<Theme>) -> Self {
        Self { shortcut, theme }
    }
}

impl Render for ShortcutHint {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        div()
            .text_size(px(11.0))
            .text_color(rgb(colors.text.muted))
            .font_family("monospace")
            .child(self.shortcut.display())
    }
}
```

### Shortcuts Cheat Sheet

```rust
// src/components/shortcuts_panel.rs
use crate::theme::Theme;
use gpui::*;

pub struct ShortcutsPanel {
    registry: Arc<ShortcutRegistry>,
    theme: Arc<Theme>,
    search_query: String,
    selected_category: Option<String>,
}

impl ShortcutsPanel {
    pub fn new(registry: Arc<ShortcutRegistry>, theme: Arc<Theme>) -> Self {
        Self {
            registry,
            theme,
            search_query: String::new(),
            selected_category: None,
        }
    }
}

impl Render for ShortcutsPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        div()
            .w(px(500.0))
            .max_h(px(600.0))
            .flex()
            .flex_col()
            .bg(rgb(colors.ui.surface))
            .rounded_lg()
            .shadow_xl()
            .overflow_hidden()
            // Header
            .child(
                div()
                    .px_4()
                    .py_3()
                    .border_b_1()
                    .border_color(rgb(colors.ui.border))
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(14.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(colors.text.primary))
                            .child("Keyboard Shortcuts")
                    )
                    .child(
                        div()
                            .cursor_pointer()
                            .p_1()
                            .rounded_sm()
                            .hover(|s| s.bg(rgb(colors.ui.hover)))
                            .child(Icon::new("x").size(px(16.0)))
                    )
            )
            // Search
            .child(
                div()
                    .px_4()
                    .py_2()
                    .border_b_1()
                    .border_color(rgb(colors.ui.border))
                    .child(
                        div()
                            .w_full()
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .bg(rgb(colors.ui.input))
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(Icon::new("search").size(px(14.0)).color(rgb(colors.text.muted)))
                            .child(
                                input()
                                    .w_full()
                                    .bg_transparent()
                                    .text_size(px(13.0))
                                    .placeholder("Search shortcuts...")
                            )
                    )
            )
            // Categories and shortcuts
            .child(
                div()
                    .flex_1()
                    .overflow_y_auto()
                    .p_4()
                    .child(self.render_shortcuts(cx))
            )
            // Footer
            .child(
                div()
                    .px_4()
                    .py_2()
                    .border_t_1()
                    .border_color(rgb(colors.ui.border))
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(rgb(colors.text.muted))
                            .child("Press ? anytime to show this panel")
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1()
                            .rounded_md()
                            .cursor_pointer()
                            .text_size(px(12.0))
                            .text_color(rgb(colors.accent.primary))
                            .hover(|s| s.bg(rgb(colors.ui.hover)))
                            .child("Customize")
                    )
            )
    }
}

impl ShortcutsPanel {
    fn render_shortcuts(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        div()
            .flex()
            .flex_col()
            .gap_6()
            .children(self.registry.categories.iter().map(|category| {
                let shortcuts: Vec<_> = self.registry.shortcuts.values()
                    .filter(|s| s.category == category.id)
                    .filter(|s| {
                        if self.search_query.is_empty() {
                            true
                        } else {
                            let query = self.search_query.to_lowercase();
                            s.action.to_lowercase().contains(&query)
                                || s.description.to_lowercase().contains(&query)
                        }
                    })
                    .collect();
                
                if shortcuts.is_empty() {
                    return div().into_any_element();
                }
                
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    // Category header
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .when_some(category.icon.clone(), |el, icon| {
                                el.child(Icon::new(icon).size(px(14.0)).color(rgb(colors.text.muted)))
                            })
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(colors.text.muted))
                                    .text_transform(TextTransform::Uppercase)
                                    .child(category.label.clone())
                            )
                    )
                    // Shortcuts
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .rounded_md()
                            .bg(rgb(colors.background.main))
                            .border_1()
                            .border_color(rgb(colors.ui.border))
                            .children(shortcuts.iter().enumerate().map(|(i, shortcut)| {
                                div()
                                    .px_3()
                                    .py_2()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .when(i > 0, |el| {
                                        el.border_t_1().border_color(rgb(colors.ui.border))
                                    })
                                    // Action and description
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .child(
                                                div()
                                                    .text_size(px(13.0))
                                                    .text_color(rgb(colors.text.primary))
                                                    .child(shortcut.action.clone())
                                            )
                                            .child(
                                                div()
                                                    .text_size(px(11.0))
                                                    .text_color(rgb(colors.text.muted))
                                                    .child(shortcut.description.clone())
                                            )
                                    )
                                    // Shortcut badge
                                    .child(
                                        ShortcutBadge::new(shortcut.shortcut.clone(), self.theme.clone())
                                    )
                            }))
                    )
                    .into_any_element()
            }))
    }
}
```

## Usage Patterns

### Shortcuts in Menu Items

```rust
// Menu item with shortcut hint
impl MenuItem {
    fn render(&self, cx: &mut WindowContext) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        div()
            .h(px(32.0))
            .px_3()
            .flex()
            .items_center()
            .justify_between()
            .cursor_pointer()
            .hover(|s| s.bg(rgb(colors.ui.hover)))
            // Label
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .when_some(self.icon.clone(), |el, icon| {
                        el.child(Icon::new(icon).size(px(14.0)))
                    })
                    .child(
                        div()
                            .text_size(px(13.0))
                            .text_color(rgb(colors.text.primary))
                            .child(&self.label)
                    )
            )
            // Shortcut
            .when_some(self.shortcut.clone(), |el, shortcut| {
                el.child(ShortcutHint::new(shortcut, self.theme.clone()))
            })
    }
}
```

### Tooltips with Shortcuts

```rust
// Button tooltip showing shortcut
impl Toolbar {
    fn render_button(&self, cx: &mut WindowContext) -> impl IntoElement {
        div()
            .p_2()
            .rounded_md()
            .cursor_pointer()
            .hover(|s| s.bg(rgb(colors.ui.hover)))
            .child(Icon::new("play").size(px(16.0)))
            .tooltip_with_shortcut("Run script", Shortcut::parse("cmd+enter").unwrap())
    }
}
```

### Keyboard Help Trigger

```rust
// Show shortcuts panel with ? key
impl App {
    fn handle_key(&mut self, event: &KeyDownEvent, cx: &mut WindowContext) {
        let key = event.key.as_ref().map(|k| k.as_str()).unwrap_or("");
        
        match key {
            "?" if event.modifiers.shift => {
                self.show_shortcuts_panel(cx);
            }
            _ => {}
        }
    }
    
    fn show_shortcuts_panel(&mut self, cx: &mut WindowContext) {
        self.overlay = Some(Overlay::Shortcuts(
            ShortcutsPanel::new(self.shortcut_registry.clone(), self.theme.clone())
        ));
        cx.notify();
    }
}
```

## Testing

### Shortcuts Test Script

```typescript
// tests/smoke/test-keyboard-shortcuts.ts
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Test 1: Shortcut badges
await div(`
  <div class="p-4 flex flex-col gap-4">
    <div class="flex items-center gap-4">
      <span class="text-xs text-zinc-500">Small:</span>
      <div class="flex gap-0.5">
        <span class="px-1 py-0.5 rounded bg-zinc-700 border border-zinc-600 text-[9px] font-mono text-zinc-400">⌘</span>
        <span class="px-1 py-0.5 rounded bg-zinc-700 border border-zinc-600 text-[9px] font-mono text-zinc-400">K</span>
      </div>
    </div>
    <div class="flex items-center gap-4">
      <span class="text-xs text-zinc-500">Medium:</span>
      <div class="flex gap-1">
        <span class="px-1.5 py-0.5 rounded bg-zinc-700 border border-zinc-600 text-[10px] font-mono text-zinc-400">⌘</span>
        <span class="px-1.5 py-0.5 rounded bg-zinc-700 border border-zinc-600 text-[10px] font-mono text-zinc-400">⇧</span>
        <span class="px-1.5 py-0.5 rounded bg-zinc-700 border border-zinc-600 text-[10px] font-mono text-zinc-400">P</span>
      </div>
    </div>
    <div class="flex items-center gap-4">
      <span class="text-xs text-zinc-500">Large:</span>
      <div class="flex gap-1">
        <span class="px-2 py-1 rounded bg-zinc-700 border border-zinc-600 text-xs font-mono text-zinc-400">⌘</span>
        <span class="px-2 py-1 rounded bg-zinc-700 border border-zinc-600 text-xs font-mono text-zinc-400">↵</span>
      </div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot1 = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
writeFileSync(join(dir, 'shortcut-badges.png'), Buffer.from(shot1.data, 'base64'));

// Test 2: Menu with shortcuts
await div(`
  <div class="w-64 bg-zinc-800 rounded-lg shadow-lg border border-zinc-700 py-1">
    <div class="px-3 py-2 flex items-center justify-between hover:bg-zinc-700 cursor-pointer">
      <div class="flex items-center gap-2">
        <span class="text-sm">▶️</span>
        <span class="text-sm text-white">Run Script</span>
      </div>
      <span class="text-xs text-zinc-500 font-mono">⌘↵</span>
    </div>
    <div class="px-3 py-2 flex items-center justify-between hover:bg-zinc-700 cursor-pointer">
      <div class="flex items-center gap-2">
        <span class="text-sm">✏️</span>
        <span class="text-sm text-white">Edit Script</span>
      </div>
      <span class="text-xs text-zinc-500 font-mono">⌘E</span>
    </div>
    <div class="px-3 py-2 flex items-center justify-between hover:bg-zinc-700 cursor-pointer">
      <div class="flex items-center gap-2">
        <span class="text-sm">📋</span>
        <span class="text-sm text-white">Copy Path</span>
      </div>
      <span class="text-xs text-zinc-500 font-mono">⌘⇧C</span>
    </div>
    <div class="border-t border-zinc-700 my-1"></div>
    <div class="px-3 py-2 flex items-center justify-between hover:bg-zinc-700 cursor-pointer">
      <div class="flex items-center gap-2">
        <span class="text-sm">🗑️</span>
        <span class="text-sm text-red-400">Delete</span>
      </div>
      <span class="text-xs text-zinc-500 font-mono">⌘⌫</span>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot2 = await captureScreenshot();
writeFileSync(join(dir, 'shortcut-menu.png'), Buffer.from(shot2.data, 'base64'));

// Test 3: Shortcuts panel (cheat sheet)
await div(`
  <div class="w-[400px] bg-zinc-800 rounded-lg shadow-xl border border-zinc-700 overflow-hidden">
    <div class="px-4 py-3 border-b border-zinc-700 flex items-center justify-between">
      <span class="text-sm font-semibold text-white">Keyboard Shortcuts</span>
      <span class="text-zinc-500 cursor-pointer">×</span>
    </div>
    <div class="p-4 flex flex-col gap-4 max-h-80 overflow-y-auto">
      <!-- Navigation -->
      <div class="flex flex-col gap-2">
        <div class="flex items-center gap-2">
          <span class="text-xs font-semibold text-zinc-500 uppercase">Navigation</span>
        </div>
        <div class="bg-zinc-900 rounded-md border border-zinc-700">
          <div class="px-3 py-2 flex items-center justify-between">
            <div>
              <div class="text-sm text-white">Move up</div>
              <div class="text-xs text-zinc-500">Select previous item</div>
            </div>
            <div class="flex gap-1">
              <span class="px-1.5 py-0.5 rounded bg-zinc-700 border border-zinc-600 text-[10px] font-mono text-zinc-400">↑</span>
            </div>
          </div>
          <div class="px-3 py-2 flex items-center justify-between border-t border-zinc-700">
            <div>
              <div class="text-sm text-white">Move down</div>
              <div class="text-xs text-zinc-500">Select next item</div>
            </div>
            <div class="flex gap-1">
              <span class="px-1.5 py-0.5 rounded bg-zinc-700 border border-zinc-600 text-[10px] font-mono text-zinc-400">↓</span>
            </div>
          </div>
        </div>
      </div>
      <!-- Actions -->
      <div class="flex flex-col gap-2">
        <div class="flex items-center gap-2">
          <span class="text-xs font-semibold text-zinc-500 uppercase">Actions</span>
        </div>
        <div class="bg-zinc-900 rounded-md border border-zinc-700">
          <div class="px-3 py-2 flex items-center justify-between">
            <div>
              <div class="text-sm text-white">Run</div>
              <div class="text-xs text-zinc-500">Execute selected script</div>
            </div>
            <div class="flex gap-1">
              <span class="px-1.5 py-0.5 rounded bg-zinc-700 border border-zinc-600 text-[10px] font-mono text-zinc-400">⌘</span>
              <span class="px-1.5 py-0.5 rounded bg-zinc-700 border border-zinc-600 text-[10px] font-mono text-zinc-400">↵</span>
            </div>
          </div>
        </div>
      </div>
    </div>
    <div class="px-4 py-2 border-t border-zinc-700 flex items-center justify-between">
      <span class="text-xs text-zinc-500">Press ? anytime to show this panel</span>
      <span class="text-xs text-amber-400 cursor-pointer">Customize</span>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot3 = await captureScreenshot();
writeFileSync(join(dir, 'shortcuts-panel.png'), Buffer.from(shot3.data, 'base64'));

console.error('[KEYBOARD SHORTCUTS] Test screenshots saved');
process.exit(0);
```

## Related Bundles

- Bundle #56: Hotkey System Architecture - Global hotkey implementation
- Bundle #79: Tooltips & Hints - Shortcut display in tooltips
- Bundle #72: Actions System - Action shortcuts
