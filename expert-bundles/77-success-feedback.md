# Expert Bundle #77: Success Feedback

## Overview

Success feedback confirms that user actions completed successfully. In Script Kit, this includes script completion, file saves, clipboard operations, and settings changes. Good success feedback is quick, unobtrusive, and builds user confidence.

## Architecture

### Success Types

```rust
// src/feedback.rs
use gpui::*;

/// Categories of success feedback with different treatments
#[derive(Clone, Debug)]
pub enum SuccessType {
    /// Quick, ephemeral confirmation (clipboard copy, save)
    Ephemeral {
        message: SharedString,
        icon: Option<SharedString>,
    },
    /// Action completed with result to show
    WithResult {
        title: SharedString,
        result: SharedString,
        actions: Vec<SuccessAction>,
    },
    /// Multi-step process completed
    ProcessComplete {
        title: SharedString,
        summary: Vec<String>,
        duration: Option<Duration>,
    },
    /// Background operation finished
    BackgroundComplete {
        title: SharedString,
        details: Option<SharedString>,
    },
}

#[derive(Clone, Debug)]
pub struct SuccessAction {
    pub label: SharedString,
    pub icon: Option<SharedString>,
    pub handler: ActionHandler,
}
```

### Success Feedback Components

```rust
// src/components/success_feedback.rs
use crate::theme::Theme;
use gpui::*;

/// Quick checkmark animation for ephemeral success
pub struct SuccessCheck {
    theme: Arc<Theme>,
    show_animation: bool,
    message: Option<SharedString>,
}

impl SuccessCheck {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            show_animation: true,
            message: None,
        }
    }
    
    pub fn with_message(mut self, message: impl Into<SharedString>) -> Self {
        self.message = Some(message.into());
        self
    }
}

impl Render for SuccessCheck {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            // Animated checkmark
            .child(
                div()
                    .w(px(20.0))
                    .h(px(20.0))
                    .rounded_full()
                    .bg(rgb(colors.semantic.success))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        Icon::new("check")
                            .size(px(14.0))
                            .color(rgb(0xFFFFFF))
                    )
            )
            // Optional message
            .when_some(self.message.clone(), |el, msg| {
                el.child(
                    div()
                        .text_size(px(13.0))
                        .text_color(rgb(colors.text.primary))
                        .child(msg)
                )
            })
    }
}

/// Success toast notification
pub struct SuccessToast {
    success_type: SuccessType,
    theme: Arc<Theme>,
    visible: bool,
    auto_dismiss: Option<Duration>,
}

impl SuccessToast {
    pub fn new(success_type: SuccessType, theme: Arc<Theme>) -> Self {
        let auto_dismiss = match &success_type {
            SuccessType::Ephemeral { .. } => Some(Duration::from_millis(1500)),
            SuccessType::WithResult { .. } => Some(Duration::from_secs(5)),
            SuccessType::ProcessComplete { .. } => Some(Duration::from_secs(5)),
            SuccessType::BackgroundComplete { .. } => Some(Duration::from_secs(3)),
        };
        
        Self {
            success_type,
            theme,
            visible: true,
            auto_dismiss,
        }
    }
}

impl Render for SuccessToast {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        if !self.visible {
            return div().into_any_element();
        }
        
        match &self.success_type {
            SuccessType::Ephemeral { message, icon } => {
                div()
                    .px_4()
                    .py_2()
                    .rounded_lg()
                    .bg(rgb(colors.semantic.success))
                    .shadow_lg()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(
                        Icon::new(icon.clone().unwrap_or("check".into()))
                            .size(px(16.0))
                            .color(rgb(0xFFFFFF))
                    )
                    .child(
                        div()
                            .text_size(px(13.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgb(0xFFFFFF))
                            .child(message.clone())
                    )
                    .into_any_element()
            }
            
            SuccessType::WithResult { title, result, actions } => {
                div()
                    .p_4()
                    .rounded_lg()
                    .bg(rgb(colors.ui.surface))
                    .border_1()
                    .border_color(rgb(colors.semantic.success))
                    .shadow_lg()
                    .flex()
                    .flex_col()
                    .gap_3()
                    // Header
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                Icon::new("check-circle")
                                    .size(px(18.0))
                                    .color(rgb(colors.semantic.success))
                            )
                            .child(
                                div()
                                    .text_size(px(14.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(colors.text.primary))
                                    .child(title.clone())
                            )
                    )
                    // Result
                    .child(
                        div()
                            .p_2()
                            .rounded_md()
                            .bg(rgb(colors.background.secondary))
                            .text_size(px(12.0))
                            .font_family("monospace")
                            .text_color(rgb(colors.text.secondary))
                            .overflow_x_auto()
                            .child(result.clone())
                    )
                    // Actions
                    .when(!actions.is_empty(), |el| {
                        el.child(
                            div()
                                .flex()
                                .flex_row()
                                .gap_2()
                                .children(actions.iter().map(|action| {
                                    div()
                                        .px_3()
                                        .py_1()
                                        .rounded_sm()
                                        .text_size(px(12.0))
                                        .cursor_pointer()
                                        .bg(rgb(colors.ui.surface))
                                        .border_1()
                                        .border_color(rgb(colors.ui.border))
                                        .text_color(rgb(colors.text.primary))
                                        .hover(|s| s.bg(rgb(colors.ui.hover)))
                                        .child(action.label.clone())
                                }))
                        )
                    })
                    .into_any_element()
            }
            
            SuccessType::ProcessComplete { title, summary, duration } => {
                div()
                    .p_4()
                    .rounded_lg()
                    .bg(rgb(colors.ui.surface))
                    .shadow_lg()
                    .flex()
                    .flex_col()
                    .gap_3()
                    // Header with duration
                    .child(
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
                                        Icon::new("check-circle")
                                            .size(px(18.0))
                                            .color(rgb(colors.semantic.success))
                                    )
                                    .child(
                                        div()
                                            .text_size(px(14.0))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(rgb(colors.text.primary))
                                            .child(title.clone())
                                    )
                            )
                            .when_some(*duration, |el, dur| {
                                el.child(
                                    div()
                                        .text_size(px(11.0))
                                        .text_color(rgb(colors.text.muted))
                                        .child(format!("{:.1}s", dur.as_secs_f32()))
                                )
                            })
                    )
                    // Summary items
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .children(summary.iter().map(|item| {
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        Icon::new("check")
                                            .size(px(12.0))
                                            .color(rgb(colors.semantic.success))
                                    )
                                    .child(
                                        div()
                                            .text_size(px(12.0))
                                            .text_color(rgb(colors.text.secondary))
                                            .child(item.clone())
                                    )
                            }))
                    )
                    .into_any_element()
            }
            
            SuccessType::BackgroundComplete { title, details } => {
                div()
                    .px_4()
                    .py_3()
                    .rounded_lg()
                    .bg(rgb(colors.ui.surface))
                    .shadow_md()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(
                        Icon::new("check-circle")
                            .size(px(20.0))
                            .color(rgb(colors.semantic.success))
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .text_size(px(13.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(rgb(colors.text.primary))
                                    .child(title.clone())
                            )
                            .when_some(details.clone(), |el, det| {
                                el.child(
                                    div()
                                        .text_size(px(11.0))
                                        .text_color(rgb(colors.text.muted))
                                        .child(det)
                                )
                            })
                    )
                    .into_any_element()
            }
        }
    }
}
```

## Usage Patterns

### Clipboard Copy Success

```rust
// src/clipboard.rs
impl ClipboardManager {
    pub fn copy_with_feedback(
        &self,
        content: &str,
        cx: &mut WindowContext,
    ) -> anyhow::Result<()> {
        // Perform copy
        cx.write_to_clipboard(ClipboardItem::new(content.to_string()))?;
        
        // Show ephemeral success
        self.notification_manager.show_success(
            SuccessType::Ephemeral {
                message: "Copied to clipboard".into(),
                icon: Some("clipboard-check".into()),
            },
            cx,
        );
        
        Ok(())
    }
}
```

### Script Completion Success

```rust
// src/executor.rs
impl ScriptExecutor {
    pub fn handle_script_success(
        &self,
        script: &Script,
        output: &str,
        duration: Duration,
        cx: &mut WindowContext,
    ) {
        // For scripts with output, show result
        if !output.is_empty() && output.len() < 500 {
            self.notification_manager.show_success(
                SuccessType::WithResult {
                    title: format!("{} completed", script.name).into(),
                    result: output.trim().into(),
                    actions: vec![
                        SuccessAction {
                            label: "Copy".into(),
                            icon: Some("clipboard".into()),
                            handler: ActionHandler::new(move |cx| {
                                cx.write_to_clipboard(ClipboardItem::new(output.to_string()));
                            }),
                        },
                    ],
                },
                cx,
            );
        } else {
            // For scripts without output or with long output
            self.notification_manager.show_success(
                SuccessType::BackgroundComplete {
                    title: format!("{} completed", script.name).into(),
                    details: Some(format!("Finished in {:.1}s", duration.as_secs_f32()).into()),
                },
                cx,
            );
        }
    }
}
```

### Inline Success State

```rust
// src/prompts/form.rs
impl FormPrompt {
    fn render_field_with_validation(&self, field: &FormField, cx: &mut WindowContext) -> impl IntoElement {
        let colors = &self.theme.colors;
        let validation_state = self.get_validation_state(&field.id);
        
        div()
            .flex()
            .flex_col()
            .gap_1()
            // Input with status indicator
            .child(
                div()
                    .relative()
                    .w_full()
                    .child(/* input field */)
                    // Success checkmark when valid
                    .when(validation_state == ValidationState::Valid, |el| {
                        el.child(
                            div()
                                .absolute()
                                .right_3()
                                .top_1_2()
                                .neg_translate_y_1_2()
                                .child(
                                    Icon::new("check")
                                        .size(px(14.0))
                                        .color(rgb(colors.semantic.success))
                                )
                        )
                    })
            )
            // Success message for special validations
            .when(validation_state == ValidationState::Valid && field.show_success, |el| {
                el.child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1()
                        .child(
                            Icon::new("check")
                                .size(px(12.0))
                                .color(rgb(colors.semantic.success))
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(rgb(colors.semantic.success))
                                .child(field.success_message.clone().unwrap_or_default())
                        )
                )
            })
    }
}
```

### Multi-Step Process Success

```rust
// src/batch.rs
impl BatchProcessor {
    pub fn show_batch_complete(
        &self,
        results: &BatchResults,
        cx: &mut WindowContext,
    ) {
        let summary: Vec<String> = results.steps.iter()
            .map(|step| format!("{}: {}", step.name, step.status))
            .collect();
        
        self.notification_manager.show_success(
            SuccessType::ProcessComplete {
                title: "Batch operation complete".into(),
                summary,
                duration: Some(results.total_duration),
            },
            cx,
        );
    }
}
```

## Animation Patterns

### Checkmark Animation

```rust
// src/animations/success.rs
use gpui::*;

pub struct AnimatedCheck {
    progress: f32, // 0.0 to 1.0
    animation_handle: Option<AnimationHandle>,
}

impl AnimatedCheck {
    pub fn start(&mut self, cx: &mut WindowContext) {
        self.progress = 0.0;
        
        // Animate over 300ms with ease-out
        self.animation_handle = Some(cx.animate(
            Duration::from_millis(300),
            Animation::new()
                .with_easing(ease_out_back) // Slight overshoot for "pop"
                .with_callback(move |progress| {
                    self.progress = progress;
                }),
        ));
    }
    
    fn render_check(&self) -> impl IntoElement {
        // SVG check path with stroke-dashoffset animation
        let path_length = 24.0;
        let visible_length = path_length * self.progress;
        
        svg()
            .size_5()
            .path("M5 12l5 5L20 7")
            .stroke_width(px(2.0))
            .stroke(rgb(0x22C55E)) // green-500
            .fill_none()
            .style("stroke-dasharray", format!("{}", path_length))
            .style("stroke-dashoffset", format!("{}", path_length - visible_length))
    }
}
```

### Success State Transition

```rust
// Button that shows success state after action
pub struct ActionButton {
    label: SharedString,
    state: ButtonState,
    theme: Arc<Theme>,
}

#[derive(Clone, Copy)]
pub enum ButtonState {
    Default,
    Loading,
    Success,
}

impl ActionButton {
    pub fn set_success(&mut self, cx: &mut WindowContext) {
        self.state = ButtonState::Success;
        cx.notify();
        
        // Return to default after delay
        cx.spawn(|this, mut cx| async move {
            Timer::after(Duration::from_millis(1500)).await;
            this.update(&mut cx, |this, cx| {
                this.state = ButtonState::Default;
                cx.notify();
            }).ok();
        }).detach();
    }
}

impl Render for ActionButton {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        let (bg, content) = match self.state {
            ButtonState::Default => (
                colors.accent.primary,
                self.label.clone().into_any_element(),
            ),
            ButtonState::Loading => (
                colors.accent.primary,
                Spinner::new(px(16.0)).into_any_element(),
            ),
            ButtonState::Success => (
                colors.semantic.success,
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    .child(Icon::new("check").size(px(14.0)))
                    .child("Done")
                    .into_any_element(),
            ),
        };
        
        div()
            .px_4()
            .py_2()
            .rounded_md()
            .bg(rgb(bg))
            .text_color(rgb(colors.background.main))
            .child(content)
    }
}
```

## Best Practices

### Timing Guidelines

```rust
/// Success feedback timing recommendations
pub mod timing {
    use std::time::Duration;
    
    /// Ephemeral success (clipboard, toggle) - brief flash
    pub const EPHEMERAL: Duration = Duration::from_millis(1500);
    
    /// Standard success (save, create) - readable
    pub const STANDARD: Duration = Duration::from_secs(3);
    
    /// Success with details (script output) - time to review
    pub const WITH_DETAILS: Duration = Duration::from_secs(5);
    
    /// Never auto-dismiss (destructive action confirmations)
    pub const PERSISTENT: Option<Duration> = None;
}
```

### Success Message Guidelines

```rust
/// Success messages should be:
/// - Concise (2-4 words ideal)
/// - Action-oriented (past tense verb)
/// - Specific (what was done)
pub mod success_messages {
    // ✅ Good: Clear, specific
    pub const COPIED: &str = "Copied to clipboard";
    pub const SAVED: &str = "Changes saved";
    pub const CREATED: &str = "Script created";
    pub const DELETED: &str = "Item deleted";
    pub const SENT: &str = "Message sent";
    
    // ❌ Bad: Vague, wordy
    pub const SUCCESS_BAD: &str = "Operation completed successfully";
    pub const DONE_BAD: &str = "Done!";
}
```

## Testing

### Success Feedback Test Script

```typescript
// tests/smoke/test-success-feedback.ts
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Test 1: Ephemeral success toast
await div(`
  <div class="fixed bottom-4 right-4">
    <div class="px-4 py-2 rounded-lg bg-green-500 shadow-lg flex items-center gap-2">
      <span>✓</span>
      <span class="text-sm font-medium text-white">Copied to clipboard</span>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot1 = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
writeFileSync(join(dir, 'success-ephemeral.png'), Buffer.from(shot1.data, 'base64'));

// Test 2: Success with result
await div(`
  <div class="p-4">
    <div class="p-4 rounded-lg bg-zinc-800 border border-green-500 shadow-lg flex flex-col gap-3">
      <div class="flex items-center gap-2">
        <span class="text-green-400">✓</span>
        <span class="text-sm font-semibold text-white">Script completed</span>
      </div>
      <div class="p-2 rounded-md bg-zinc-900 text-xs font-mono text-zinc-300">
        Hello, World!
      </div>
      <div class="flex gap-2">
        <button class="px-3 py-1 rounded-sm text-xs bg-zinc-700 text-white">
          Copy
        </button>
        <button class="px-3 py-1 rounded-sm text-xs bg-zinc-700 text-white">
          Run Again
        </button>
      </div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot2 = await captureScreenshot();
writeFileSync(join(dir, 'success-with-result.png'), Buffer.from(shot2.data, 'base64'));

// Test 3: Process complete with summary
await div(`
  <div class="p-4">
    <div class="p-4 rounded-lg bg-zinc-800 shadow-lg flex flex-col gap-3">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <span class="text-green-400">✓</span>
          <span class="text-sm font-semibold text-white">Build complete</span>
        </div>
        <span class="text-xs text-zinc-500">2.3s</span>
      </div>
      <div class="flex flex-col gap-1">
        <div class="flex items-center gap-2 text-xs text-zinc-400">
          <span class="text-green-400">✓</span> Compiled 42 files
        </div>
        <div class="flex items-center gap-2 text-xs text-zinc-400">
          <span class="text-green-400">✓</span> Generated types
        </div>
        <div class="flex items-center gap-2 text-xs text-zinc-400">
          <span class="text-green-400">✓</span> No errors
        </div>
      </div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot3 = await captureScreenshot();
writeFileSync(join(dir, 'success-process.png'), Buffer.from(shot3.data, 'base64'));

// Test 4: Inline success check
await div(`
  <div class="p-4">
    <div class="relative w-full">
      <input 
        type="email" 
        value="user@example.com"
        class="w-full px-3 py-2 pr-10 rounded-md border border-green-500 bg-zinc-800 text-white"
      />
      <span class="absolute right-3 top-1/2 -translate-y-1/2 text-green-400">✓</span>
    </div>
    <div class="flex items-center gap-1 mt-1 text-xs text-green-400">
      <span>✓</span>
      <span>Email verified</span>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot4 = await captureScreenshot();
writeFileSync(join(dir, 'success-inline.png'), Buffer.from(shot4.data, 'base64'));

console.error('[SUCCESS FEEDBACK] Test screenshots saved');
process.exit(0);
```

## Related Bundles

- Bundle #74: Loading States - Before success
- Bundle #76: Error States - Negative counterpart
- Bundle #78: Toast Notifications - Delivery mechanism
- Bundle #55: Animation & Transitions - Success animations
