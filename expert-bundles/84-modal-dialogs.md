# Expert Bundle #84: Modal Dialogs

## Overview

Modal dialogs interrupt the user flow to require a decision or input. In Script Kit, modals are used for confirmations, input collection, and important warnings. Good modal design is focused, accessible, and provides clear escape paths.

## Architecture

### Dialog Types

```rust
// src/dialogs.rs
use gpui::*;

/// Dialog configuration
#[derive(Clone)]
pub struct Dialog {
    pub id: String,
    pub variant: DialogVariant,
    pub title: SharedString,
    pub message: Option<SharedString>,
    pub icon: Option<DialogIcon>,
    pub actions: Vec<DialogAction>,
    pub dismissible: bool,
    pub width: DialogWidth,
}

#[derive(Clone)]
pub enum DialogVariant {
    /// Simple confirmation (OK/Cancel)
    Confirm,
    /// Destructive action confirmation
    Destructive,
    /// Information display
    Info,
    /// Warning message
    Warning,
    /// Input collection
    Input {
        placeholder: SharedString,
        default_value: Option<String>,
        validation: Option<InputValidation>,
    },
    /// Custom content
    Custom {
        content: AnyElement,
    },
}

#[derive(Clone, Copy)]
pub enum DialogIcon {
    Info,
    Warning,
    Error,
    Question,
    Success,
}

#[derive(Clone)]
pub struct DialogAction {
    pub id: String,
    pub label: SharedString,
    pub style: DialogActionStyle,
    pub is_default: bool,
    pub closes_dialog: bool,
}

#[derive(Clone, Copy)]
pub enum DialogActionStyle {
    Primary,
    Secondary,
    Destructive,
    Cancel,
}

#[derive(Clone, Copy)]
pub enum DialogWidth {
    Small,   // 320px
    Medium,  // 420px
    Large,   // 520px
}

#[derive(Clone)]
pub struct InputValidation {
    pub required: bool,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
    pub error_message: SharedString,
}

impl Dialog {
    pub fn confirm(title: impl Into<SharedString>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            variant: DialogVariant::Confirm,
            title: title.into(),
            message: None,
            icon: Some(DialogIcon::Question),
            actions: vec![
                DialogAction {
                    id: "cancel".into(),
                    label: "Cancel".into(),
                    style: DialogActionStyle::Cancel,
                    is_default: false,
                    closes_dialog: true,
                },
                DialogAction {
                    id: "confirm".into(),
                    label: "Confirm".into(),
                    style: DialogActionStyle::Primary,
                    is_default: true,
                    closes_dialog: true,
                },
            ],
            dismissible: true,
            width: DialogWidth::Small,
        }
    }
    
    pub fn destructive(title: impl Into<SharedString>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            variant: DialogVariant::Destructive,
            title: title.into(),
            message: None,
            icon: Some(DialogIcon::Warning),
            actions: vec![
                DialogAction {
                    id: "cancel".into(),
                    label: "Cancel".into(),
                    style: DialogActionStyle::Cancel,
                    is_default: false,
                    closes_dialog: true,
                },
                DialogAction {
                    id: "delete".into(),
                    label: "Delete".into(),
                    style: DialogActionStyle::Destructive,
                    is_default: false, // Don't make destructive actions default
                    closes_dialog: true,
                },
            ],
            dismissible: true,
            width: DialogWidth::Small,
        }
    }
    
    pub fn input(title: impl Into<SharedString>, placeholder: impl Into<SharedString>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            variant: DialogVariant::Input {
                placeholder: placeholder.into(),
                default_value: None,
                validation: None,
            },
            title: title.into(),
            message: None,
            icon: None,
            actions: vec![
                DialogAction {
                    id: "cancel".into(),
                    label: "Cancel".into(),
                    style: DialogActionStyle::Cancel,
                    is_default: false,
                    closes_dialog: true,
                },
                DialogAction {
                    id: "submit".into(),
                    label: "OK".into(),
                    style: DialogActionStyle::Primary,
                    is_default: true,
                    closes_dialog: true,
                },
            ],
            dismissible: true,
            width: DialogWidth::Medium,
        }
    }
    
    pub fn message(mut self, message: impl Into<SharedString>) -> Self {
        self.message = Some(message.into());
        self
    }
    
    pub fn width(mut self, width: DialogWidth) -> Self {
        self.width = width;
        self
    }
}
```

### Dialog Component

```rust
// src/components/dialog.rs
use crate::theme::Theme;
use gpui::*;

pub struct DialogView {
    dialog: Dialog,
    theme: Arc<Theme>,
    focus_handle: FocusHandle,
    input_value: String,
    validation_error: Option<SharedString>,
}

impl DialogView {
    pub fn new(dialog: Dialog, theme: Arc<Theme>, cx: &mut WindowContext) -> Self {
        let default_value = match &dialog.variant {
            DialogVariant::Input { default_value, .. } => {
                default_value.clone().unwrap_or_default()
            }
            _ => String::new(),
        };
        
        Self {
            dialog,
            theme,
            focus_handle: cx.focus_handle(),
            input_value: default_value,
            validation_error: None,
        }
    }
}

impl Render for DialogView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        let width = match self.dialog.width {
            DialogWidth::Small => px(320.0),
            DialogWidth::Medium => px(420.0),
            DialogWidth::Large => px(520.0),
        };
        
        // Backdrop
        div()
            .absolute()
            .inset_0()
            .z_index(900)
            .bg(rgba(0x000000, 0.5))
            .flex()
            .items_center()
            .justify_center()
            .on_click(cx.listener(|this, _, cx| {
                if this.dialog.dismissible {
                    cx.emit(DialogClose);
                }
            }))
            // Dialog card
            .child(
                div()
                    .w(width)
                    .rounded_xl()
                    .bg(rgb(colors.ui.surface))
                    .border_1()
                    .border_color(rgb(colors.ui.border))
                    .shadow_2xl()
                    .overflow_hidden()
                    // Prevent click-through
                    .on_click(|_, _| {})
                    // Focus trap
                    .track_focus(&self.focus_handle)
                    .on_key_down(cx.listener(|this, e: &KeyDownEvent, _, cx| {
                        let key = e.key.as_ref().map(|k| k.as_str()).unwrap_or("");
                        match key {
                            "escape" | "Escape" if this.dialog.dismissible => {
                                cx.emit(DialogClose);
                            }
                            "enter" | "Enter" => {
                                // Find default action
                                if let Some(action) = this.dialog.actions.iter().find(|a| a.is_default) {
                                    cx.emit(DialogAction(action.id.clone()));
                                }
                            }
                            _ => {}
                        }
                    }))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            // Header
                            .child(self.render_header(cx))
                            // Content
                            .child(self.render_content(cx))
                            // Footer with actions
                            .child(self.render_footer(cx))
                    )
            )
    }
}

impl DialogView {
    fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        div()
            .px_5()
            .py_4()
            .flex()
            .flex_row()
            .items_center()
            .gap_3()
            // Icon
            .when_some(self.dialog.icon, |el, icon| {
                let (icon_name, icon_color) = match icon {
                    DialogIcon::Info => ("info", colors.semantic.info),
                    DialogIcon::Warning => ("alert-triangle", colors.semantic.warning),
                    DialogIcon::Error => ("x-circle", colors.semantic.error),
                    DialogIcon::Question => ("help-circle", colors.accent.primary),
                    DialogIcon::Success => ("check-circle", colors.semantic.success),
                };
                
                el.child(
                    div()
                        .w(px(40.0))
                        .h(px(40.0))
                        .rounded_full()
                        .bg(with_alpha(icon_color, 0.1))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            Icon::new(icon_name)
                                .size(px(20.0))
                                .color(rgb(icon_color))
                        )
                )
            })
            // Title
            .child(
                div()
                    .flex_1()
                    .child(
                        div()
                            .text_size(px(16.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(colors.text.primary))
                            .child(self.dialog.title.clone())
                    )
            )
            // Close button
            .when(self.dialog.dismissible, |el| {
                el.child(
                    div()
                        .cursor_pointer()
                        .p_1()
                        .rounded_md()
                        .hover(|s| s.bg(rgb(colors.ui.hover)))
                        .on_click(cx.listener(|_, _, cx| {
                            cx.emit(DialogClose);
                        }))
                        .child(
                            Icon::new("x")
                                .size(px(18.0))
                                .color(rgb(colors.text.muted))
                        )
                )
            })
    }
    
    fn render_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        div()
            .px_5()
            .pb_4()
            // Message
            .when_some(self.dialog.message.clone(), |el, msg| {
                el.child(
                    div()
                        .text_size(px(14.0))
                        .text_color(rgb(colors.text.secondary))
                        .child(msg)
                )
            })
            // Input field (if input variant)
            .when(matches!(self.dialog.variant, DialogVariant::Input { .. }), |el| {
                if let DialogVariant::Input { placeholder, .. } = &self.dialog.variant {
                    el.child(
                        div()
                            .mt_3()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .w_full()
                                    .px_3()
                                    .py_2()
                                    .rounded_md()
                                    .border_1()
                                    .border_color(rgb(if self.validation_error.is_some() {
                                        colors.semantic.error
                                    } else {
                                        colors.ui.border
                                    }))
                                    .bg(rgb(colors.ui.input))
                                    .child(
                                        input()
                                            .w_full()
                                            .bg_transparent()
                                            .text_size(px(14.0))
                                            .text_color(rgb(colors.text.primary))
                                            .placeholder(placeholder)
                                            .value(&self.input_value)
                                            .on_input(cx.listener(|this, value: &str, cx| {
                                                this.input_value = value.to_string();
                                                this.validation_error = None;
                                                cx.notify();
                                            }))
                                    )
                            )
                            .when_some(self.validation_error.clone(), |el, error| {
                                el.child(
                                    div()
                                        .text_size(px(12.0))
                                        .text_color(rgb(colors.semantic.error))
                                        .child(error)
                                )
                            })
                    )
                } else {
                    el
                }
            })
    }
    
    fn render_footer(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        div()
            .px_5()
            .py_4()
            .border_t_1()
            .border_color(rgb(colors.ui.border))
            .flex()
            .flex_row()
            .items_center()
            .justify_end()
            .gap_2()
            .children(self.dialog.actions.iter().map(|action| {
                let action_id = action.id.clone();
                let style = action.style;
                
                div()
                    .px_4()
                    .py_2()
                    .rounded_md()
                    .cursor_pointer()
                    .text_size(px(14.0))
                    .font_weight(FontWeight::MEDIUM)
                    .map(|el| match style {
                        DialogActionStyle::Primary => el
                            .bg(rgb(colors.accent.primary))
                            .text_color(rgb(colors.background.main))
                            .hover(|s| s.bg(rgb(colors.accent.hover))),
                        DialogActionStyle::Secondary => el
                            .bg(rgb(colors.ui.surface))
                            .border_1()
                            .border_color(rgb(colors.ui.border))
                            .text_color(rgb(colors.text.primary))
                            .hover(|s| s.bg(rgb(colors.ui.hover))),
                        DialogActionStyle::Destructive => el
                            .bg(rgb(colors.semantic.error))
                            .text_color(rgb(0xFFFFFF))
                            .hover(|s| s.opacity(0.9)),
                        DialogActionStyle::Cancel => el
                            .bg_transparent()
                            .text_color(rgb(colors.text.secondary))
                            .hover(|s| s.bg(rgb(colors.ui.hover))),
                    })
                    .on_click(cx.listener(move |this, _, cx| {
                        // Validate input if needed
                        if let DialogVariant::Input { validation, .. } = &this.dialog.variant {
                            if let Some(validation) = validation {
                                if validation.required && this.input_value.is_empty() {
                                    this.validation_error = Some(validation.error_message.clone());
                                    cx.notify();
                                    return;
                                }
                            }
                        }
                        
                        cx.emit(DialogAction(action_id.clone()));
                    }))
                    .child(action.label.clone())
            }))
    }
}

#[derive(Clone)]
pub struct DialogAction(pub String);

#[derive(Clone)]
pub struct DialogClose;
```

## Usage Patterns

### Delete Confirmation

```rust
impl ScriptList {
    fn confirm_delete(&mut self, script: &Script, cx: &mut WindowContext) {
        let script_name = script.name.clone();
        
        let dialog = Dialog::destructive("Delete Script?")
            .message(format!(
                "Are you sure you want to delete \"{}\"? This action cannot be undone.",
                script_name
            ));
        
        self.show_dialog(dialog, cx);
    }
    
    fn handle_dialog_action(&mut self, action: &str, cx: &mut WindowContext) {
        match action {
            "delete" => {
                if let Some(script) = self.script_to_delete.take() {
                    self.delete_script(&script, cx);
                    self.notifications.success("Script deleted", cx);
                }
            }
            "cancel" => {
                self.script_to_delete = None;
            }
            _ => {}
        }
        
        self.close_dialog(cx);
    }
}
```

### Rename Dialog

```rust
impl ScriptList {
    fn show_rename_dialog(&mut self, script: &Script, cx: &mut WindowContext) {
        let dialog = Dialog::input("Rename Script", "Enter new name")
            .variant(DialogVariant::Input {
                placeholder: "Script name".into(),
                default_value: Some(script.name.clone()),
                validation: Some(InputValidation {
                    required: true,
                    min_length: Some(1),
                    max_length: Some(100),
                    pattern: None,
                    error_message: "Name cannot be empty".into(),
                }),
            });
        
        self.show_dialog(dialog, cx);
    }
}
```

### Info Dialog

```rust
impl App {
    fn show_about_dialog(&mut self, cx: &mut WindowContext) {
        let dialog = Dialog {
            id: "about".into(),
            variant: DialogVariant::Info,
            title: "About Script Kit".into(),
            message: Some(format!(
                "Version {}\n\nA productivity tool for developers.",
                env!("CARGO_PKG_VERSION")
            ).into()),
            icon: Some(DialogIcon::Info),
            actions: vec![
                DialogAction {
                    id: "ok".into(),
                    label: "OK".into(),
                    style: DialogActionStyle::Primary,
                    is_default: true,
                    closes_dialog: true,
                },
            ],
            dismissible: true,
            width: DialogWidth::Small,
        };
        
        self.show_dialog(dialog, cx);
    }
}
```

## Animation

```rust
// src/dialogs/animation.rs

impl DialogView {
    fn render_animated(&self, cx: &mut WindowContext) -> impl IntoElement {
        // Backdrop fade in
        let backdrop = div()
            .with_animation(
                "dialog-backdrop",
                Animation::new()
                    .duration(Duration::from_millis(150))
                    .easing(ease_out_cubic),
                |el, progress| el.opacity(progress * 0.5),
            );
        
        // Dialog scale + fade
        let dialog = div()
            .with_animation(
                "dialog-card",
                Animation::new()
                    .duration(Duration::from_millis(200))
                    .easing(ease_out_back), // Slight overshoot
                |el, progress| {
                    let scale = 0.95 + (progress * 0.05);
                    el.opacity(progress)
                        .transform(Transform::scale(scale))
                },
            );
        
        backdrop.child(dialog.child(/* dialog content */))
    }
}
```

## Testing

### Dialog Test Script

```typescript
// tests/smoke/test-modal-dialogs.ts
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Test 1: Confirmation dialog
await div(`
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center">
    <div class="w-80 rounded-xl bg-zinc-800 border border-zinc-700 shadow-2xl overflow-hidden">
      <div class="px-5 py-4 flex items-center gap-3">
        <div class="w-10 h-10 rounded-full bg-amber-500/10 flex items-center justify-center">
          <span class="text-amber-500">❓</span>
        </div>
        <div class="flex-1">
          <div class="text-base font-semibold text-white">Confirm Action</div>
        </div>
        <span class="text-zinc-500 cursor-pointer">×</span>
      </div>
      <div class="px-5 pb-4">
        <div class="text-sm text-zinc-400">Are you sure you want to proceed?</div>
      </div>
      <div class="px-5 py-4 border-t border-zinc-700 flex justify-end gap-2">
        <button class="px-4 py-2 rounded-md text-sm text-zinc-400 hover:bg-zinc-700">Cancel</button>
        <button class="px-4 py-2 rounded-md text-sm bg-amber-500 text-black font-medium">Confirm</button>
      </div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot1 = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
writeFileSync(join(dir, 'dialog-confirm.png'), Buffer.from(shot1.data, 'base64'));

// Test 2: Destructive dialog
await div(`
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center">
    <div class="w-80 rounded-xl bg-zinc-800 border border-zinc-700 shadow-2xl overflow-hidden">
      <div class="px-5 py-4 flex items-center gap-3">
        <div class="w-10 h-10 rounded-full bg-red-500/10 flex items-center justify-center">
          <span class="text-red-500">⚠️</span>
        </div>
        <div class="flex-1">
          <div class="text-base font-semibold text-white">Delete Script?</div>
        </div>
        <span class="text-zinc-500 cursor-pointer">×</span>
      </div>
      <div class="px-5 pb-4">
        <div class="text-sm text-zinc-400">Are you sure you want to delete "my-script"? This action cannot be undone.</div>
      </div>
      <div class="px-5 py-4 border-t border-zinc-700 flex justify-end gap-2">
        <button class="px-4 py-2 rounded-md text-sm text-zinc-400 hover:bg-zinc-700">Cancel</button>
        <button class="px-4 py-2 rounded-md text-sm bg-red-500 text-white font-medium">Delete</button>
      </div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot2 = await captureScreenshot();
writeFileSync(join(dir, 'dialog-destructive.png'), Buffer.from(shot2.data, 'base64'));

// Test 3: Input dialog
await div(`
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center">
    <div class="w-[420px] rounded-xl bg-zinc-800 border border-zinc-700 shadow-2xl overflow-hidden">
      <div class="px-5 py-4 flex items-center gap-3">
        <div class="flex-1">
          <div class="text-base font-semibold text-white">Rename Script</div>
        </div>
        <span class="text-zinc-500 cursor-pointer">×</span>
      </div>
      <div class="px-5 pb-4">
        <div class="text-sm text-zinc-400 mb-3">Enter a new name for your script.</div>
        <input 
          type="text" 
          value="my-awesome-script"
          class="w-full px-3 py-2 rounded-md border border-zinc-600 bg-zinc-900 text-white text-sm"
          placeholder="Script name"
        />
      </div>
      <div class="px-5 py-4 border-t border-zinc-700 flex justify-end gap-2">
        <button class="px-4 py-2 rounded-md text-sm text-zinc-400 hover:bg-zinc-700">Cancel</button>
        <button class="px-4 py-2 rounded-md text-sm bg-amber-500 text-black font-medium">Rename</button>
      </div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot3 = await captureScreenshot();
writeFileSync(join(dir, 'dialog-input.png'), Buffer.from(shot3.data, 'base64'));

// Test 4: Input dialog with validation error
await div(`
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center">
    <div class="w-[420px] rounded-xl bg-zinc-800 border border-zinc-700 shadow-2xl overflow-hidden">
      <div class="px-5 py-4 flex items-center gap-3">
        <div class="flex-1">
          <div class="text-base font-semibold text-white">Rename Script</div>
        </div>
        <span class="text-zinc-500 cursor-pointer">×</span>
      </div>
      <div class="px-5 pb-4">
        <div class="text-sm text-zinc-400 mb-3">Enter a new name for your script.</div>
        <input 
          type="text" 
          value=""
          class="w-full px-3 py-2 rounded-md border border-red-500 bg-zinc-900 text-white text-sm"
          placeholder="Script name"
        />
        <div class="text-xs text-red-400 mt-1">Name cannot be empty</div>
      </div>
      <div class="px-5 py-4 border-t border-zinc-700 flex justify-end gap-2">
        <button class="px-4 py-2 rounded-md text-sm text-zinc-400 hover:bg-zinc-700">Cancel</button>
        <button class="px-4 py-2 rounded-md text-sm bg-amber-500 text-black font-medium">Rename</button>
      </div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot4 = await captureScreenshot();
writeFileSync(join(dir, 'dialog-validation.png'), Buffer.from(shot4.data, 'base64'));

console.error('[MODAL DIALOGS] Test screenshots saved');
process.exit(0);
```

## Related Bundles

- Bundle #83: Context Menus - Triggering dialogs from menus
- Bundle #76: Error States - Error dialogs
- Bundle #65: Focus Management - Focus trapping in dialogs
