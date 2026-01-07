# Expert Bundle #83: Context Menus

## Overview

Context menus provide relevant actions for items through right-click or keyboard shortcuts. In Script Kit, context menus appear for scripts, list items, text selections, and UI elements. Good context menus are contextual, organized, and keyboard-navigable.

## Architecture

### Context Menu System

```rust
// src/context_menu.rs
use gpui::*;

/// A context menu with items
#[derive(Clone)]
pub struct ContextMenu {
    pub items: Vec<ContextMenuItem>,
    pub position: Point<Pixels>,
    pub trigger: ContextMenuTrigger,
}

#[derive(Clone)]
pub enum ContextMenuItem {
    Action {
        id: String,
        label: SharedString,
        icon: Option<SharedString>,
        shortcut: Option<Shortcut>,
        disabled: bool,
        destructive: bool,
    },
    Submenu {
        label: SharedString,
        icon: Option<SharedString>,
        items: Vec<ContextMenuItem>,
    },
    Separator,
    Header {
        label: SharedString,
    },
    Toggle {
        id: String,
        label: SharedString,
        checked: bool,
    },
}

#[derive(Clone)]
pub enum ContextMenuTrigger {
    RightClick,
    Keyboard, // Usually Shift+F10 or Menu key
    LongPress,
}

impl ContextMenu {
    pub fn new(position: Point<Pixels>) -> Self {
        Self {
            items: vec![],
            position,
            trigger: ContextMenuTrigger::RightClick,
        }
    }
    
    pub fn action(
        mut self,
        id: impl Into<String>,
        label: impl Into<SharedString>,
    ) -> Self {
        self.items.push(ContextMenuItem::Action {
            id: id.into(),
            label: label.into(),
            icon: None,
            shortcut: None,
            disabled: false,
            destructive: false,
        });
        self
    }
    
    pub fn action_with_icon(
        mut self,
        id: impl Into<String>,
        label: impl Into<SharedString>,
        icon: impl Into<SharedString>,
    ) -> Self {
        self.items.push(ContextMenuItem::Action {
            id: id.into(),
            label: label.into(),
            icon: Some(icon.into()),
            shortcut: None,
            disabled: false,
            destructive: false,
        });
        self
    }
    
    pub fn separator(mut self) -> Self {
        self.items.push(ContextMenuItem::Separator);
        self
    }
    
    pub fn header(mut self, label: impl Into<SharedString>) -> Self {
        self.items.push(ContextMenuItem::Header { label: label.into() });
        self
    }
    
    pub fn submenu(
        mut self,
        label: impl Into<SharedString>,
        items: Vec<ContextMenuItem>,
    ) -> Self {
        self.items.push(ContextMenuItem::Submenu {
            label: label.into(),
            icon: None,
            items,
        });
        self
    }
    
    pub fn destructive_action(
        mut self,
        id: impl Into<String>,
        label: impl Into<SharedString>,
    ) -> Self {
        self.items.push(ContextMenuItem::Action {
            id: id.into(),
            label: label.into(),
            icon: Some("trash".into()),
            shortcut: None,
            disabled: false,
            destructive: true,
        });
        self
    }
}
```

### Context Menu Component

```rust
// src/components/context_menu.rs
use crate::theme::Theme;
use gpui::*;

pub struct ContextMenuView {
    menu: ContextMenu,
    theme: Arc<Theme>,
    selected_index: Option<usize>,
    open_submenu: Option<usize>,
    focus_handle: FocusHandle,
}

impl ContextMenuView {
    pub fn new(menu: ContextMenu, theme: Arc<Theme>, cx: &mut WindowContext) -> Self {
        Self {
            menu,
            theme,
            selected_index: None,
            open_submenu: None,
            focus_handle: cx.focus_handle(),
        }
    }
    
    fn selectable_indices(&self) -> Vec<usize> {
        self.menu.items.iter()
            .enumerate()
            .filter_map(|(i, item)| match item {
                ContextMenuItem::Action { disabled, .. } if !disabled => Some(i),
                ContextMenuItem::Submenu { .. } => Some(i),
                ContextMenuItem::Toggle { .. } => Some(i),
                _ => None,
            })
            .collect()
    }
    
    fn move_selection(&mut self, direction: i32, cx: &mut WindowContext) {
        let indices = self.selectable_indices();
        if indices.is_empty() {
            return;
        }
        
        let current = self.selected_index
            .and_then(|i| indices.iter().position(|&idx| idx == i))
            .unwrap_or(0);
        
        let new_pos = if direction > 0 {
            (current + 1) % indices.len()
        } else {
            (current + indices.len() - 1) % indices.len()
        };
        
        self.selected_index = Some(indices[new_pos]);
        cx.notify();
    }
}

impl Render for ContextMenuView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        div()
            .key_context("ContextMenu")
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, e: &KeyDownEvent, window, cx| {
                let key = e.key.as_ref().map(|k| k.as_str()).unwrap_or("");
                match key {
                    "up" | "arrowup" => this.move_selection(-1, cx),
                    "down" | "arrowdown" => this.move_selection(1, cx),
                    "enter" | "Enter" => {
                        if let Some(idx) = this.selected_index {
                            cx.emit(ContextMenuAction(idx));
                        }
                    }
                    "escape" | "Escape" => cx.emit(ContextMenuClose),
                    "right" | "arrowright" => {
                        // Open submenu if selected
                        if let Some(idx) = this.selected_index {
                            if matches!(this.menu.items.get(idx), Some(ContextMenuItem::Submenu { .. })) {
                                this.open_submenu = Some(idx);
                                cx.notify();
                            }
                        }
                    }
                    "left" | "arrowleft" => {
                        // Close submenu
                        this.open_submenu = None;
                        cx.notify();
                    }
                    _ => {}
                }
            }))
            // Menu container
            .absolute()
            .left(self.menu.position.x)
            .top(self.menu.position.y)
            .z_index(1000)
            .min_w(px(180.0))
            .max_w(px(280.0))
            .py_1()
            .rounded_lg()
            .bg(rgb(colors.ui.menu_bg))
            .border_1()
            .border_color(rgb(colors.ui.border))
            .shadow_xl()
            // Menu items
            .children(self.menu.items.iter().enumerate().map(|(i, item)| {
                self.render_item(i, item, cx)
            }))
    }
}

impl ContextMenuView {
    fn render_item(&self, index: usize, item: &ContextMenuItem, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        let is_selected = self.selected_index == Some(index);
        
        match item {
            ContextMenuItem::Action { id, label, icon, shortcut, disabled, destructive } => {
                let id = id.clone();
                
                div()
                    .h(px(32.0))
                    .px_3()
                    .mx_1()
                    .rounded_md()
                    .flex()
                    .items_center()
                    .justify_between()
                    .cursor(if *disabled { CursorStyle::Default } else { CursorStyle::PointingHand })
                    .bg(rgb(if is_selected { colors.ui.hover } else { 0x00000000 }))
                    .hover(|s| if !*disabled { s.bg(rgb(colors.ui.hover)) } else { s })
                    .on_mouse_enter(cx.listener(move |this, _, cx| {
                        if !*disabled {
                            this.selected_index = Some(index);
                            cx.notify();
                        }
                    }))
                    .on_click(cx.listener(move |this, _, cx| {
                        if !*disabled {
                            cx.emit(ContextMenuAction(index));
                        }
                    }))
                    // Left side (icon + label)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .when_some(icon.clone(), |el, icon| {
                                el.child(
                                    Icon::new(icon)
                                        .size(px(14.0))
                                        .color(rgb(if *destructive {
                                            colors.semantic.error
                                        } else if *disabled {
                                            colors.text.disabled
                                        } else {
                                            colors.text.secondary
                                        }))
                                )
                            })
                            .child(
                                div()
                                    .text_size(px(13.0))
                                    .text_color(rgb(if *destructive {
                                        colors.semantic.error
                                    } else if *disabled {
                                        colors.text.disabled
                                    } else {
                                        colors.text.primary
                                    }))
                                    .child(label.clone())
                            )
                    )
                    // Right side (shortcut)
                    .when_some(shortcut.clone(), |el, sc| {
                        el.child(
                            div()
                                .text_size(px(11.0))
                                .text_color(rgb(colors.text.muted))
                                .font_family("monospace")
                                .child(sc.display())
                        )
                    })
            }
            
            ContextMenuItem::Submenu { label, icon, items } => {
                let has_open_submenu = self.open_submenu == Some(index);
                
                div()
                    .relative()
                    .h(px(32.0))
                    .px_3()
                    .mx_1()
                    .rounded_md()
                    .flex()
                    .items_center()
                    .justify_between()
                    .cursor_pointer()
                    .bg(rgb(if is_selected { colors.ui.hover } else { 0x00000000 }))
                    .hover(|s| s.bg(rgb(colors.ui.hover)))
                    .on_mouse_enter(cx.listener(move |this, _, cx| {
                        this.selected_index = Some(index);
                        this.open_submenu = Some(index);
                        cx.notify();
                    }))
                    // Left side
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .when_some(icon.clone(), |el, icon| {
                                el.child(Icon::new(icon).size(px(14.0)))
                            })
                            .child(
                                div()
                                    .text_size(px(13.0))
                                    .text_color(rgb(colors.text.primary))
                                    .child(label.clone())
                            )
                    )
                    // Chevron
                    .child(
                        Icon::new("chevron-right")
                            .size(px(12.0))
                            .color(rgb(colors.text.muted))
                    )
                    // Submenu
                    .when(has_open_submenu, |el| {
                        el.child(self.render_submenu(items, cx))
                    })
            }
            
            ContextMenuItem::Separator => {
                div()
                    .h(px(1.0))
                    .mx_2()
                    .my_1()
                    .bg(rgb(colors.ui.border))
            }
            
            ContextMenuItem::Header { label } => {
                div()
                    .h(px(28.0))
                    .px_3()
                    .flex()
                    .items_center()
                    .child(
                        div()
                            .text_size(px(11.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(colors.text.muted))
                            .text_transform(TextTransform::Uppercase)
                            .child(label.clone())
                    )
            }
            
            ContextMenuItem::Toggle { id, label, checked } => {
                let id = id.clone();
                
                div()
                    .h(px(32.0))
                    .px_3()
                    .mx_1()
                    .rounded_md()
                    .flex()
                    .items_center()
                    .justify_between()
                    .cursor_pointer()
                    .bg(rgb(if is_selected { colors.ui.hover } else { 0x00000000 }))
                    .hover(|s| s.bg(rgb(colors.ui.hover)))
                    .on_mouse_enter(cx.listener(move |this, _, cx| {
                        this.selected_index = Some(index);
                        cx.notify();
                    }))
                    .on_click(cx.listener(move |this, _, cx| {
                        cx.emit(ContextMenuAction(index));
                    }))
                    .child(
                        div()
                            .text_size(px(13.0))
                            .text_color(rgb(colors.text.primary))
                            .child(label.clone())
                    )
                    .child(
                        div()
                            .w(px(16.0))
                            .h(px(16.0))
                            .rounded_sm()
                            .border_1()
                            .border_color(rgb(if *checked { colors.accent.primary } else { colors.ui.border }))
                            .bg(rgb(if *checked { colors.accent.primary } else { 0x00000000 }))
                            .flex()
                            .items_center()
                            .justify_center()
                            .when(*checked, |el| {
                                el.child(
                                    Icon::new("check")
                                        .size(px(12.0))
                                        .color(rgb(colors.background.main))
                                )
                            })
                    )
            }
        }
        .into_any_element()
    }
    
    fn render_submenu(&self, items: &[ContextMenuItem], cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        div()
            .absolute()
            .left_full()
            .top_0()
            .ml_1()
            .min_w(px(160.0))
            .py_1()
            .rounded_lg()
            .bg(rgb(colors.ui.menu_bg))
            .border_1()
            .border_color(rgb(colors.ui.border))
            .shadow_xl()
            .children(items.iter().map(|item| {
                // Simplified submenu item rendering
                match item {
                    ContextMenuItem::Action { label, .. } => {
                        div()
                            .h(px(32.0))
                            .px_3()
                            .mx_1()
                            .rounded_md()
                            .flex()
                            .items_center()
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(colors.ui.hover)))
                            .child(
                                div()
                                    .text_size(px(13.0))
                                    .text_color(rgb(colors.text.primary))
                                    .child(label.clone())
                            )
                            .into_any_element()
                    }
                    _ => div().into_any_element(),
                }
            }))
    }
}

#[derive(Clone)]
pub struct ContextMenuAction(pub usize);

#[derive(Clone)]
pub struct ContextMenuClose;
```

## Usage Patterns

### Script Item Context Menu

```rust
impl ScriptList {
    fn show_context_menu(&mut self, script: &Script, position: Point<Pixels>, cx: &mut WindowContext) {
        let menu = ContextMenu::new(position)
            .action_with_icon("run", "Run Script", "play")
            .action_with_icon("edit", "Edit Script", "edit")
            .separator()
            .action_with_icon("copy-path", "Copy Path", "clipboard")
            .action_with_icon("reveal", "Reveal in Finder", "folder")
            .separator()
            .submenu("More Actions", vec![
                ContextMenuItem::Action {
                    id: "duplicate".into(),
                    label: "Duplicate".into(),
                    icon: Some("copy".into()),
                    shortcut: Some(Shortcut::parse("cmd+d").unwrap()),
                    disabled: false,
                    destructive: false,
                },
                ContextMenuItem::Action {
                    id: "rename".into(),
                    label: "Rename...".into(),
                    icon: Some("edit-2".into()),
                    shortcut: None,
                    disabled: false,
                    destructive: false,
                },
                ContextMenuItem::Action {
                    id: "move".into(),
                    label: "Move to...".into(),
                    icon: Some("folder".into()),
                    shortcut: None,
                    disabled: false,
                    destructive: false,
                },
            ])
            .separator()
            .destructive_action("delete", "Delete Script");
        
        self.context_menu = Some(ContextMenuView::new(menu, self.theme.clone(), cx));
        cx.notify();
    }
    
    fn handle_context_menu_action(&mut self, index: usize, cx: &mut WindowContext) {
        if let Some(menu) = &self.context_menu {
            if let Some(item) = menu.menu.items.get(index) {
                match item {
                    ContextMenuItem::Action { id, .. } => {
                        match id.as_str() {
                            "run" => self.run_selected_script(cx),
                            "edit" => self.edit_selected_script(cx),
                            "copy-path" => self.copy_script_path(cx),
                            "reveal" => self.reveal_in_finder(cx),
                            "delete" => self.confirm_delete(cx),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
        
        self.context_menu = None;
        cx.notify();
    }
}
```

### Text Selection Context Menu

```rust
impl EditorPrompt {
    fn show_selection_context_menu(&mut self, position: Point<Pixels>, cx: &mut WindowContext) {
        let has_selection = self.editor.has_selection();
        
        let menu = ContextMenu::new(position)
            .action_with_icon("cut", "Cut", "scissors")
            .action_with_icon("copy", "Copy", "copy")
            .action_with_icon("paste", "Paste", "clipboard")
            .separator()
            .action("select-all", "Select All");
        
        // Disable cut/copy if no selection
        if !has_selection {
            // Modify items to disable...
        }
        
        self.context_menu = Some(ContextMenuView::new(menu, self.theme.clone(), cx));
        cx.notify();
    }
}
```

### Triggering Context Menus

```rust
// Right-click trigger
impl ListItem {
    fn render(&self, cx: &mut WindowContext) -> impl IntoElement {
        div()
            .on_mouse_down(MouseButton::Right, cx.listener(|this, e: &MouseDownEvent, cx| {
                this.show_context_menu(e.position, cx);
            }))
            // ... rest of item
    }
}

// Keyboard trigger (Menu key or Shift+F10)
impl MainMenu {
    fn handle_key(&mut self, event: &KeyDownEvent, cx: &mut WindowContext) {
        let key = event.key.as_ref().map(|k| k.as_str()).unwrap_or("");
        
        match key {
            "ContextMenu" | "F10" if event.modifiers.shift => {
                if let Some(selected) = self.get_selected_item() {
                    let position = self.get_item_position(selected);
                    self.show_context_menu(selected, position, cx);
                }
            }
            _ => {}
        }
    }
}
```

## Position Calculation

```rust
// src/context_menu/positioning.rs

/// Calculate optimal menu position avoiding window edges
pub fn calculate_menu_position(
    click_position: Point<Pixels>,
    menu_size: Size<Pixels>,
    window_bounds: Bounds<Pixels>,
) -> Point<Pixels> {
    let mut pos = click_position;
    
    // Check right edge
    if pos.x + menu_size.width > window_bounds.origin.x + window_bounds.size.width {
        pos.x = click_position.x - menu_size.width;
    }
    
    // Check bottom edge
    if pos.y + menu_size.height > window_bounds.origin.y + window_bounds.size.height {
        pos.y = click_position.y - menu_size.height;
    }
    
    // Ensure not off-screen left/top
    pos.x = pos.x.max(window_bounds.origin.x);
    pos.y = pos.y.max(window_bounds.origin.y);
    
    pos
}

/// Calculate submenu position
pub fn calculate_submenu_position(
    parent_item_bounds: Bounds<Pixels>,
    submenu_size: Size<Pixels>,
    window_bounds: Bounds<Pixels>,
) -> Point<Pixels> {
    let mut pos = Point::new(
        parent_item_bounds.origin.x + parent_item_bounds.size.width,
        parent_item_bounds.origin.y,
    );
    
    // Check if submenu would overflow right
    if pos.x + submenu_size.width > window_bounds.origin.x + window_bounds.size.width {
        // Open to the left instead
        pos.x = parent_item_bounds.origin.x - submenu_size.width;
    }
    
    // Check bottom overflow
    if pos.y + submenu_size.height > window_bounds.origin.y + window_bounds.size.height {
        pos.y = window_bounds.origin.y + window_bounds.size.height - submenu_size.height;
    }
    
    pos
}
```

## Testing

### Context Menu Test Script

```typescript
// tests/smoke/test-context-menus.ts
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Test 1: Basic context menu
await div(`
  <div class="relative p-8">
    <div class="w-44 py-1 rounded-lg bg-zinc-800 border border-zinc-700 shadow-xl">
      <div class="h-8 px-3 mx-1 rounded-md flex items-center gap-2 hover:bg-zinc-700 cursor-pointer">
        <span class="text-sm">▶️</span>
        <span class="text-sm text-white">Run Script</span>
      </div>
      <div class="h-8 px-3 mx-1 rounded-md flex items-center gap-2 bg-zinc-700 cursor-pointer">
        <span class="text-sm">✏️</span>
        <span class="text-sm text-white">Edit Script</span>
      </div>
      <div class="h-px mx-2 my-1 bg-zinc-700"></div>
      <div class="h-8 px-3 mx-1 rounded-md flex items-center gap-2 hover:bg-zinc-700 cursor-pointer">
        <span class="text-sm">📋</span>
        <span class="text-sm text-white">Copy Path</span>
      </div>
      <div class="h-px mx-2 my-1 bg-zinc-700"></div>
      <div class="h-8 px-3 mx-1 rounded-md flex items-center gap-2 hover:bg-zinc-700 cursor-pointer">
        <span class="text-sm">🗑️</span>
        <span class="text-sm text-red-400">Delete</span>
      </div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot1 = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
writeFileSync(join(dir, 'context-menu-basic.png'), Buffer.from(shot1.data, 'base64'));

// Test 2: Context menu with shortcuts
await div(`
  <div class="relative p-8">
    <div class="w-56 py-1 rounded-lg bg-zinc-800 border border-zinc-700 shadow-xl">
      <div class="h-8 px-3 mx-1 rounded-md flex items-center justify-between hover:bg-zinc-700 cursor-pointer">
        <div class="flex items-center gap-2">
          <span class="text-sm">✂️</span>
          <span class="text-sm text-white">Cut</span>
        </div>
        <span class="text-xs text-zinc-500 font-mono">⌘X</span>
      </div>
      <div class="h-8 px-3 mx-1 rounded-md flex items-center justify-between hover:bg-zinc-700 cursor-pointer">
        <div class="flex items-center gap-2">
          <span class="text-sm">📋</span>
          <span class="text-sm text-white">Copy</span>
        </div>
        <span class="text-xs text-zinc-500 font-mono">⌘C</span>
      </div>
      <div class="h-8 px-3 mx-1 rounded-md flex items-center justify-between hover:bg-zinc-700 cursor-pointer">
        <div class="flex items-center gap-2">
          <span class="text-sm">📥</span>
          <span class="text-sm text-white">Paste</span>
        </div>
        <span class="text-xs text-zinc-500 font-mono">⌘V</span>
      </div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot2 = await captureScreenshot();
writeFileSync(join(dir, 'context-menu-shortcuts.png'), Buffer.from(shot2.data, 'base64'));

// Test 3: Context menu with submenu
await div(`
  <div class="relative p-8">
    <div class="w-48 py-1 rounded-lg bg-zinc-800 border border-zinc-700 shadow-xl">
      <div class="h-8 px-3 mx-1 rounded-md flex items-center gap-2 hover:bg-zinc-700 cursor-pointer">
        <span class="text-sm text-white">New Script</span>
      </div>
      <div class="relative h-8 px-3 mx-1 rounded-md flex items-center justify-between bg-zinc-700 cursor-pointer">
        <span class="text-sm text-white">Move to</span>
        <span class="text-xs text-zinc-500">›</span>
        <!-- Submenu -->
        <div class="absolute left-full top-0 ml-1 w-40 py-1 rounded-lg bg-zinc-800 border border-zinc-700 shadow-xl">
          <div class="h-8 px-3 mx-1 rounded-md flex items-center hover:bg-zinc-700 cursor-pointer">
            <span class="text-sm text-white">Scripts</span>
          </div>
          <div class="h-8 px-3 mx-1 rounded-md flex items-center hover:bg-zinc-700 cursor-pointer">
            <span class="text-sm text-white">Tools</span>
          </div>
          <div class="h-8 px-3 mx-1 rounded-md flex items-center hover:bg-zinc-700 cursor-pointer">
            <span class="text-sm text-white">Snippets</span>
          </div>
        </div>
      </div>
      <div class="h-px mx-2 my-1 bg-zinc-700"></div>
      <div class="h-8 px-3 mx-1 rounded-md flex items-center gap-2 hover:bg-zinc-700 cursor-pointer">
        <span class="text-sm text-red-400">Delete</span>
      </div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot3 = await captureScreenshot();
writeFileSync(join(dir, 'context-menu-submenu.png'), Buffer.from(shot3.data, 'base64'));

// Test 4: Context menu with header and toggles
await div(`
  <div class="relative p-8">
    <div class="w-52 py-1 rounded-lg bg-zinc-800 border border-zinc-700 shadow-xl">
      <div class="h-7 px-3 flex items-center">
        <span class="text-xs font-semibold text-zinc-500 uppercase">View Options</span>
      </div>
      <div class="h-8 px-3 mx-1 rounded-md flex items-center justify-between hover:bg-zinc-700 cursor-pointer">
        <span class="text-sm text-white">Show Icons</span>
        <div class="w-4 h-4 rounded-sm border border-amber-500 bg-amber-500 flex items-center justify-center">
          <span class="text-black text-xs">✓</span>
        </div>
      </div>
      <div class="h-8 px-3 mx-1 rounded-md flex items-center justify-between hover:bg-zinc-700 cursor-pointer">
        <span class="text-sm text-white">Show Descriptions</span>
        <div class="w-4 h-4 rounded-sm border border-zinc-600"></div>
      </div>
      <div class="h-8 px-3 mx-1 rounded-md flex items-center justify-between hover:bg-zinc-700 cursor-pointer">
        <span class="text-sm text-white">Compact Mode</span>
        <div class="w-4 h-4 rounded-sm border border-zinc-600"></div>
      </div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot4 = await captureScreenshot();
writeFileSync(join(dir, 'context-menu-toggles.png'), Buffer.from(shot4.data, 'base64'));

console.error('[CONTEXT MENUS] Test screenshots saved');
process.exit(0);
```

## Related Bundles

- Bundle #72: Actions System - Actions in context menus
- Bundle #82: Keyboard Shortcuts UX - Shortcuts in menu items
- Bundle #84: Modal Dialogs - Confirmation dialogs from menu actions
