# Actions System - Expert Bundle

## Overview

Actions are contextual commands that can be performed on the currently selected item. They appear in a bottom-pinned dialog triggered by Tab or right-click.

## Action Architecture

### Action Definition

```rust
#[derive(Debug, Clone)]
pub struct Action {
    pub id: String,
    pub name: String,
    pub shortcut: Option<String>,
    pub icon: Option<String>,
    pub is_destructive: bool,
    pub is_default: bool,
}

impl Action {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            shortcut: None,
            icon: None,
            is_destructive: false,
            is_default: false,
        }
    }

    pub fn shortcut(mut self, shortcut: &str) -> Self {
        self.shortcut = Some(shortcut.to_string());
        self
    }

    pub fn icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }

    pub fn destructive(mut self) -> Self {
        self.is_destructive = true;
        self
    }

    pub fn default(mut self) -> Self {
        self.is_default = true;
        self
    }
}
```

### Standard Actions

```rust
pub fn standard_script_actions() -> Vec<Action> {
    vec![
        Action::new("run", "Run Script")
            .shortcut("Enter")
            .icon("play")
            .default(),
        Action::new("edit", "Edit in Editor")
            .shortcut("Cmd+E")
            .icon("edit"),
        Action::new("reveal", "Reveal in Finder")
            .shortcut("Cmd+Shift+R")
            .icon("folder"),
        Action::new("copy-path", "Copy Path")
            .shortcut("Cmd+Shift+C")
            .icon("copy"),
        Action::new("duplicate", "Duplicate Script")
            .shortcut("Cmd+D")
            .icon("copy-plus"),
        Action::new("rename", "Rename Script")
            .shortcut("Cmd+R")
            .icon("pencil"),
        Action::new("delete", "Delete Script")
            .shortcut("Cmd+Backspace")
            .icon("trash")
            .destructive(),
    ]
}
```

## Actions Dialog (src/actions.rs)

### Dialog Component

```rust
pub struct ActionsDialog {
    actions: Vec<Action>,
    selected_index: usize,
    focus_handle: FocusHandle,
    on_select: Box<dyn Fn(&Action, &mut Context<Self>)>,
}

impl ActionsDialog {
    pub fn new(
        actions: Vec<Action>,
        on_select: impl Fn(&Action, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            actions,
            selected_index: 0,
            focus_handle: cx.focus_handle(),
            on_select: Box::new(on_select),
        }
    }
}
```

### Rendering

```rust
impl Render for ActionsDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let item_height = 36.0;
        let max_visible = 8;
        let visible_count = self.actions.len().min(max_visible);
        let list_height = visible_count as f32 * item_height;
        
        div()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_key_down))
            // Backdrop
            .absolute()
            .inset_0()
            .bg(rgba(0x000000, 0.5))
            .flex()
            .flex_col()
            .justify_end()
            // Dialog
            .child(
                div()
                    .w_full()
                    .max_h(px(list_height + 8.0))
                    .bg(rgb(0x27272A))
                    .border_t_1()
                    .border_color(rgb(0x3F3F46))
                    .py_1()
                    .child(self.render_action_list(cx))
            )
    }

    fn render_action_list(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .children(
                self.actions.iter().enumerate().map(|(index, action)| {
                    self.render_action_item(action, index, cx)
                })
            )
    }

    fn render_action_item(
        &self,
        action: &Action,
        index: usize,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_selected = index == self.selected_index;
        let is_destructive = action.is_destructive;
        
        div()
            .id(ElementId::from(index))
            .h(px(36.0))
            .px_3()
            .flex()
            .items_center()
            .gap_3()
            .cursor_pointer()
            .when(is_selected, |d| d.bg(rgb(0x3F3F46)))
            .hover(|d| d.bg(rgb(0x3F3F46)))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.select_action(index, cx);
            }))
            // Icon
            .when_some(action.icon.as_ref(), |d, icon| {
                d.child(
                    Icon::new(icon_name(icon))
                        .size_4()
                        .text_color(if is_destructive {
                            rgb(0xEF4444)
                        } else {
                            rgb(0xA1A1AA)
                        })
                )
            })
            // Name
            .child(
                div()
                    .flex_1()
                    .text_sm()
                    .text_color(if is_destructive {
                        rgb(0xEF4444)
                    } else {
                        rgb(0xE4E4E7)
                    })
                    .child(&action.name)
            )
            // Shortcut
            .when_some(action.shortcut.as_ref(), |d, shortcut| {
                d.child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x71717A))
                        .child(shortcut.clone())
                )
            })
    }
}
```

### Keyboard Handling

```rust
impl ActionsDialog {
    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.key.as_ref().map(|k| k.as_str()).unwrap_or("");
        
        match key {
            "up" | "arrowup" => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                    cx.notify();
                }
            }
            "down" | "arrowdown" => {
                if self.selected_index < self.actions.len() - 1 {
                    self.selected_index += 1;
                    cx.notify();
                }
            }
            "enter" | "Enter" => {
                self.select_current(cx);
            }
            "escape" | "Escape" | "tab" | "Tab" => {
                cx.emit(ActionsEvent::Dismiss);
            }
            // Quick select by number
            "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                let index = key.parse::<usize>().unwrap() - 1;
                if index < self.actions.len() {
                    self.select_action(index, cx);
                }
            }
            _ => {}
        }
    }

    fn select_action(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(action) = self.actions.get(index) {
            (self.on_select)(action, cx);
        }
    }

    fn select_current(&mut self, cx: &mut Context<Self>) {
        self.select_action(self.selected_index, cx);
    }
}
```

## Integration with Main Palette

### Showing Actions

```rust
impl CommandPalette {
    fn show_actions(&mut self, cx: &mut Context<Self>) {
        if let Some(item) = self.get_selected_item() {
            self.actions = self.get_actions_for_item(item);
            self.show_actions_dialog = true;
            cx.notify();
        }
    }

    fn get_actions_for_item(&self, item: &CommandItem) -> Vec<Action> {
        match item.item_type {
            ItemType::Script => standard_script_actions(),
            ItemType::App => standard_app_actions(),
            ItemType::File => standard_file_actions(),
            ItemType::Snippet => standard_snippet_actions(),
        }
    }
}
```

### Handling Action Selection

```rust
impl CommandPalette {
    fn handle_action(&mut self, action: &Action, cx: &mut Context<Self>) {
        if let Some(item) = self.get_selected_item() {
            match action.id.as_str() {
                "run" => self.run_item(item, cx),
                "edit" => self.edit_item(item, cx),
                "reveal" => self.reveal_item(item, cx),
                "copy-path" => self.copy_path(item, cx),
                "duplicate" => self.duplicate_item(item, cx),
                "rename" => self.start_rename(item, cx),
                "delete" => self.confirm_delete(item, cx),
                _ => {}
            }
        }
        
        self.show_actions_dialog = false;
        cx.notify();
    }
}
```

## Context-Aware Actions

### Dynamic Action Generation

```rust
fn get_actions_for_script(script: &Script) -> Vec<Action> {
    let mut actions = vec![
        Action::new("run", "Run Script")
            .shortcut("Enter")
            .icon("play")
            .default(),
    ];
    
    // Add edit action if editor is configured
    if crate::config::has_editor() {
        actions.push(
            Action::new("edit", "Edit in Editor")
                .shortcut("Cmd+E")
                .icon("edit")
        );
    }
    
    // Add shortcut action if script has one
    if script.shortcut.is_some() {
        actions.push(
            Action::new("remove-shortcut", "Remove Shortcut")
                .icon("keyboard-off")
        );
    } else {
        actions.push(
            Action::new("add-shortcut", "Add Shortcut")
                .shortcut("Cmd+K")
                .icon("keyboard")
        );
    }
    
    // Add background toggle if applicable
    if script.background {
        actions.push(
            Action::new("stop-background", "Stop Background Script")
                .icon("stop-circle")
        );
    }
    
    // Always include destructive actions last
    actions.push(
        Action::new("delete", "Delete Script")
            .shortcut("Cmd+Backspace")
            .icon("trash")
            .destructive()
    );
    
    actions
}
```

## Confirmation Dialogs

### Destructive Action Confirmation

```rust
fn confirm_delete(&mut self, item: &CommandItem, cx: &mut Context<Self>) {
    self.confirmation = Some(ConfirmationDialog {
        title: format!("Delete \"{}\"?", item.name),
        message: "This action cannot be undone.".to_string(),
        confirm_label: "Delete".to_string(),
        is_destructive: true,
        on_confirm: Box::new(move |cx| {
            // Perform deletion
            delete_script(&item.path);
            cx.emit(PaletteEvent::Refresh);
        }),
    });
    cx.notify();
}
```

## Action Shortcuts in Footer

```rust
fn render_footer(&self) -> impl IntoElement {
    div()
        .h(px(40.0))
        .px_3()
        .flex()
        .items_center()
        .justify_between()
        .border_t_1()
        .border_color(rgb(0x3F3F46))
        .bg(rgb(0x18181B))
        // Primary action
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .px_1p5()
                        .py_0p5()
                        .rounded()
                        .bg(rgb(0x3B82F6))
                        .text_xs()
                        .child("Enter")
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0xA1A1AA))
                        .child("Run")
                )
        )
        // Actions trigger
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .px_1p5()
                        .py_0p5()
                        .rounded()
                        .bg(rgb(0x3F3F46))
                        .text_xs()
                        .child("Tab")
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0xA1A1AA))
                        .child("Actions")
                )
        )
}
```

## Best Practices

1. **Default action first** - Most common action at top with Enter shortcut
2. **Destructive actions last** - With red styling and confirmation
3. **Number shortcuts** - 1-9 for quick selection
4. **Context-aware** - Show relevant actions only
5. **Keyboard accessible** - All actions via keyboard
6. **Clear shortcuts** - Display shortcut hints consistently

## Summary

| Trigger | Behavior |
|---------|----------|
| Tab | Show actions dialog |
| Enter | Execute default/selected action |
| 1-9 | Quick select action by number |
| Escape | Close dialog |
| Cmd+Key | Direct action shortcuts |
