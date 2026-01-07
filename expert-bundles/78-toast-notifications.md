# Expert Bundle #78: Toast Notifications

## Overview

Toast notifications are ephemeral messages that appear temporarily to communicate status without interrupting workflow. Script Kit uses toasts for success confirmations, error alerts, progress updates, and informational messages. They stack, auto-dismiss, and can include actions.

## Architecture

### Notification Types

```rust
// src/notifications.rs
use gpui::*;
use std::time::Duration;

/// Notification urgency levels
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

/// A single notification instance
#[derive(Clone)]
pub struct Notification {
    pub id: NotificationId,
    pub level: NotificationLevel,
    pub title: SharedString,
    pub message: Option<SharedString>,
    pub icon: Option<SharedString>,
    pub actions: Vec<NotificationAction>,
    pub duration: Option<Duration>,
    pub dismissible: bool,
    pub created_at: Instant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NotificationId(u64);

impl NotificationId {
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self(COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

#[derive(Clone)]
pub struct NotificationAction {
    pub label: SharedString,
    pub handler: Arc<dyn Fn(&mut WindowContext) + Send + Sync>,
    pub style: ActionStyle,
}

#[derive(Clone, Copy, Debug)]
pub enum ActionStyle {
    Primary,
    Secondary,
    Destructive,
}

impl Notification {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            id: NotificationId::new(),
            level: NotificationLevel::Info,
            title: title.into(),
            message: None,
            icon: None,
            actions: vec![],
            duration: Some(Duration::from_secs(5)),
            dismissible: true,
            created_at: Instant::now(),
        }
    }
    
    pub fn info(title: impl Into<SharedString>) -> Self {
        Self::new(title).level(NotificationLevel::Info)
    }
    
    pub fn success(title: impl Into<SharedString>) -> Self {
        Self::new(title)
            .level(NotificationLevel::Success)
            .duration(Duration::from_millis(2000))
    }
    
    pub fn warning(title: impl Into<SharedString>) -> Self {
        Self::new(title).level(NotificationLevel::Warning)
    }
    
    pub fn error(title: impl Into<SharedString>) -> Self {
        Self::new(title)
            .level(NotificationLevel::Error)
            .duration(Duration::from_secs(8))
    }
    
    pub fn level(mut self, level: NotificationLevel) -> Self {
        self.level = level;
        self
    }
    
    pub fn message(mut self, message: impl Into<SharedString>) -> Self {
        self.message = Some(message.into());
        self
    }
    
    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }
    
    pub fn action(mut self, label: impl Into<SharedString>, handler: impl Fn(&mut WindowContext) + Send + Sync + 'static) -> Self {
        self.actions.push(NotificationAction {
            label: label.into(),
            handler: Arc::new(handler),
            style: ActionStyle::Primary,
        });
        self
    }
    
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }
    
    pub fn persistent(mut self) -> Self {
        self.duration = None;
        self
    }
    
    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.dismissible = dismissible;
        self
    }
}
```

### Notification Manager

```rust
// src/notifications/manager.rs
use crate::theme::Theme;
use gpui::*;
use std::collections::VecDeque;

/// Maximum simultaneous visible notifications
const MAX_VISIBLE: usize = 5;

pub struct NotificationManager {
    notifications: VecDeque<Notification>,
    theme: Arc<Theme>,
    position: NotificationPosition,
    dismiss_timers: HashMap<NotificationId, Task<()>>,
}

#[derive(Clone, Copy, Debug)]
pub enum NotificationPosition {
    TopRight,
    TopLeft,
    TopCenter,
    BottomRight,
    BottomLeft,
    BottomCenter,
}

impl NotificationManager {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            notifications: VecDeque::new(),
            theme,
            position: NotificationPosition::BottomRight,
            dismiss_timers: HashMap::new(),
        }
    }
    
    pub fn show(&mut self, notification: Notification, cx: &mut WindowContext) {
        let id = notification.id;
        
        // Add to queue
        self.notifications.push_back(notification.clone());
        
        // Trim if over limit
        while self.notifications.len() > MAX_VISIBLE {
            if let Some(oldest) = self.notifications.pop_front() {
                self.dismiss_timers.remove(&oldest.id);
            }
        }
        
        // Set up auto-dismiss timer
        if let Some(duration) = notification.duration {
            let timer = cx.spawn(|this, mut cx| async move {
                Timer::after(duration).await;
                this.update(&mut cx, |this, cx| {
                    this.dismiss(id, cx);
                }).ok();
            });
            self.dismiss_timers.insert(id, timer);
        }
        
        cx.notify();
    }
    
    pub fn dismiss(&mut self, id: NotificationId, cx: &mut WindowContext) {
        self.notifications.retain(|n| n.id != id);
        self.dismiss_timers.remove(&id);
        cx.notify();
    }
    
    pub fn dismiss_all(&mut self, cx: &mut WindowContext) {
        self.notifications.clear();
        self.dismiss_timers.clear();
        cx.notify();
    }
    
    /// Convenience methods
    pub fn info(&mut self, title: impl Into<SharedString>, cx: &mut WindowContext) {
        self.show(Notification::info(title), cx);
    }
    
    pub fn success(&mut self, title: impl Into<SharedString>, cx: &mut WindowContext) {
        self.show(Notification::success(title), cx);
    }
    
    pub fn warning(&mut self, title: impl Into<SharedString>, cx: &mut WindowContext) {
        self.show(Notification::warning(title), cx);
    }
    
    pub fn error(&mut self, title: impl Into<SharedString>, cx: &mut WindowContext) {
        self.show(Notification::error(title), cx);
    }
}

impl Render for NotificationManager {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        // Position container
        let (pos_class, flex_dir) = match self.position {
            NotificationPosition::TopRight => ("top-4 right-4", FlexDirection::Column),
            NotificationPosition::TopLeft => ("top-4 left-4", FlexDirection::Column),
            NotificationPosition::TopCenter => ("top-4 left-1/2 -translate-x-1/2", FlexDirection::Column),
            NotificationPosition::BottomRight => ("bottom-4 right-4", FlexDirection::ColumnReverse),
            NotificationPosition::BottomLeft => ("bottom-4 left-4", FlexDirection::ColumnReverse),
            NotificationPosition::BottomCenter => ("bottom-4 left-1/2 -translate-x-1/2", FlexDirection::ColumnReverse),
        };
        
        div()
            .absolute()
            .z_index(1000)
            // Position based on config
            .map(|el| match self.position {
                NotificationPosition::TopRight => el.top_4().right_4(),
                NotificationPosition::TopLeft => el.top_4().left_4(),
                NotificationPosition::BottomRight => el.bottom_4().right_4(),
                NotificationPosition::BottomLeft => el.bottom_4().left_4(),
                _ => el.bottom_4().right_4(),
            })
            .flex()
            .flex_col()
            .map(|el| if matches!(self.position, NotificationPosition::BottomRight | NotificationPosition::BottomLeft | NotificationPosition::BottomCenter) {
                el.flex_col_reverse()
            } else {
                el
            })
            .gap_2()
            .children(self.notifications.iter().map(|notification| {
                NotificationToast::new(notification.clone(), self.theme.clone())
            }))
    }
}
```

### Toast Component

```rust
// src/notifications/toast.rs
pub struct NotificationToast {
    notification: Notification,
    theme: Arc<Theme>,
    hovered: bool,
    exiting: bool,
}

impl NotificationToast {
    pub fn new(notification: Notification, theme: Arc<Theme>) -> Self {
        Self {
            notification,
            theme,
            hovered: false,
            exiting: false,
        }
    }
    
    fn level_color(&self) -> u32 {
        let colors = &self.theme.colors;
        match self.notification.level {
            NotificationLevel::Info => colors.semantic.info,
            NotificationLevel::Success => colors.semantic.success,
            NotificationLevel::Warning => colors.semantic.warning,
            NotificationLevel::Error => colors.semantic.error,
        }
    }
    
    fn level_icon(&self) -> &'static str {
        match self.notification.level {
            NotificationLevel::Info => "info",
            NotificationLevel::Success => "check-circle",
            NotificationLevel::Warning => "alert-triangle",
            NotificationLevel::Error => "x-circle",
        }
    }
}

impl Render for NotificationToast {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        let level_color = self.level_color();
        let id = self.notification.id;
        
        div()
            .w(px(320.0))
            .p_3()
            .rounded_lg()
            .bg(rgb(colors.ui.surface))
            .border_l_4()
            .border_color(rgb(level_color))
            .shadow_lg()
            // Hover to pause auto-dismiss
            .on_mouse_enter(cx.listener(|this, _, _, cx| {
                this.hovered = true;
            }))
            .on_mouse_leave(cx.listener(|this, _, _, cx| {
                this.hovered = false;
            }))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    // Header row
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_start()
                            .gap_3()
                            // Icon
                            .child(
                                Icon::new(self.notification.icon.clone().unwrap_or_else(|| self.level_icon().into()))
                                    .size(px(18.0))
                                    .color(rgb(level_color))
                            )
                            // Content
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_size(px(13.0))
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(rgb(colors.text.primary))
                                            .child(self.notification.title.clone())
                                    )
                                    .when_some(self.notification.message.clone(), |el, msg| {
                                        el.child(
                                            div()
                                                .text_size(px(12.0))
                                                .text_color(rgb(colors.text.muted))
                                                .child(msg)
                                        )
                                    })
                            )
                            // Dismiss button
                            .when(self.notification.dismissible, |el| {
                                el.child(
                                    div()
                                        .cursor_pointer()
                                        .p_1()
                                        .rounded_sm()
                                        .hover(|s| s.bg(rgb(colors.ui.hover)))
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            // Trigger dismiss animation then remove
                                            cx.emit(DismissNotification(id));
                                        }))
                                        .child(
                                            Icon::new("x")
                                                .size(px(14.0))
                                                .color(rgb(colors.text.muted))
                                        )
                                )
                            })
                    )
                    // Actions
                    .when(!self.notification.actions.is_empty(), |el| {
                        el.child(
                            div()
                                .flex()
                                .flex_row()
                                .gap_2()
                                .mt_1()
                                .children(self.notification.actions.iter().map(|action| {
                                    let handler = action.handler.clone();
                                    let style = action.style;
                                    
                                    div()
                                        .px_3()
                                        .py_1()
                                        .rounded_sm()
                                        .text_size(px(12.0))
                                        .cursor_pointer()
                                        .map(|el| match style {
                                            ActionStyle::Primary => el
                                                .bg(rgb(colors.accent.primary))
                                                .text_color(rgb(colors.background.main))
                                                .hover(|s| s.bg(rgb(colors.accent.hover))),
                                            ActionStyle::Secondary => el
                                                .bg(rgb(colors.ui.surface))
                                                .border_1()
                                                .border_color(rgb(colors.ui.border))
                                                .text_color(rgb(colors.text.primary))
                                                .hover(|s| s.bg(rgb(colors.ui.hover))),
                                            ActionStyle::Destructive => el
                                                .bg(rgb(colors.semantic.error))
                                                .text_color(rgb(0xFFFFFF))
                                                .hover(|s| s.opacity(0.9)),
                                        })
                                        .on_click(cx.listener(move |_, _, _, cx| {
                                            handler(cx);
                                        }))
                                        .child(action.label.clone())
                                }))
                        )
                    })
            )
    }
}
```

## Usage Patterns

### Basic Notifications

```rust
// Show simple notifications
impl App {
    fn copy_to_clipboard(&mut self, content: &str, cx: &mut WindowContext) {
        cx.write_to_clipboard(ClipboardItem::new(content.to_string()));
        self.notifications.success("Copied to clipboard", cx);
    }
    
    fn save_file(&mut self, cx: &mut WindowContext) {
        match self.do_save() {
            Ok(_) => self.notifications.success("File saved", cx),
            Err(e) => self.notifications.error(format!("Save failed: {}", e), cx),
        }
    }
    
    fn network_request(&mut self, cx: &mut WindowContext) {
        self.notifications.show(
            Notification::info("Syncing...")
                .icon("refresh-cw")
                .persistent() // Don't auto-dismiss
                .dismissible(false), // Can't manually dismiss
            cx,
        );
    }
}
```

### Notifications with Actions

```rust
// Notification with undo action
impl App {
    fn delete_item(&mut self, item_id: ItemId, cx: &mut WindowContext) {
        let item = self.remove_item(item_id);
        
        self.notifications.show(
            Notification::info("Item deleted")
                .action("Undo", move |cx| {
                    // Restore the item
                    cx.emit(RestoreItem(item.clone()));
                })
                .duration(Duration::from_secs(8)), // Longer for undo
            cx,
        );
    }
}

// Notification with multiple actions
impl App {
    fn show_update_available(&mut self, version: &str, cx: &mut WindowContext) {
        self.notifications.show(
            Notification::info(format!("Update {} available", version))
                .message("Restart to apply the update")
                .action("Restart Now", |cx| {
                    cx.emit(RestartApp);
                })
                .action_secondary("Later", |cx| {
                    // Dismiss (no-op, just close)
                })
                .persistent(), // Don't auto-dismiss updates
            cx,
        );
    }
}
```

### Progress Notifications

```rust
// src/notifications/progress.rs
pub struct ProgressNotification {
    id: NotificationId,
    title: SharedString,
    progress: f32, // 0.0 to 1.0
    details: Option<SharedString>,
    theme: Arc<Theme>,
}

impl ProgressNotification {
    pub fn new(title: impl Into<SharedString>, theme: Arc<Theme>) -> Self {
        Self {
            id: NotificationId::new(),
            title: title.into(),
            progress: 0.0,
            details: None,
            theme,
        }
    }
    
    pub fn set_progress(&mut self, progress: f32, details: Option<String>, cx: &mut WindowContext) {
        self.progress = progress.clamp(0.0, 1.0);
        self.details = details.map(|s| s.into());
        cx.notify();
    }
}

impl Render for ProgressNotification {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        div()
            .w(px(320.0))
            .p_3()
            .rounded_lg()
            .bg(rgb(colors.ui.surface))
            .shadow_lg()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    // Title
                    .child(
                        div()
                            .text_size(px(13.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgb(colors.text.primary))
                            .child(self.title.clone())
                    )
                    // Progress bar
                    .child(
                        div()
                            .w_full()
                            .h(px(4.0))
                            .rounded_full()
                            .bg(rgb(colors.ui.border))
                            .child(
                                div()
                                    .h_full()
                                    .rounded_full()
                                    .bg(rgb(colors.accent.primary))
                                    .w(Percentage(self.progress * 100.0))
                            )
                    )
                    // Details
                    .when_some(self.details.clone(), |el, details| {
                        el.child(
                            div()
                                .text_size(px(11.0))
                                .text_color(rgb(colors.text.muted))
                                .child(details)
                        )
                    })
            )
    }
}
```

## Animation Patterns

### Enter/Exit Animations

```rust
// src/notifications/animations.rs
impl NotificationToast {
    fn animate_enter(&self, cx: &mut WindowContext) -> impl IntoElement {
        // Slide in from right + fade
        div()
            .with_animation(
                "notification-enter",
                Animation::new()
                    .duration(Duration::from_millis(200))
                    .easing(ease_out_cubic),
                move |el, progress| {
                    let translate_x = (1.0 - progress) * 100.0;
                    el.translate_x(px(translate_x))
                        .opacity(progress)
                },
            )
    }
    
    fn animate_exit(&self, cx: &mut WindowContext) -> impl IntoElement {
        // Fade out + shrink
        div()
            .with_animation(
                "notification-exit",
                Animation::new()
                    .duration(Duration::from_millis(150))
                    .easing(ease_in_cubic),
                move |el, progress| {
                    let opacity = 1.0 - progress;
                    let scale = 1.0 - (progress * 0.1);
                    el.opacity(opacity)
                        .transform(Transform::scale(scale))
                },
            )
    }
}
```

### Stack Animation

```rust
// When new notification pushes others
impl NotificationManager {
    fn render_with_stack_animation(&self, cx: &mut WindowContext) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_2()
            .children(self.notifications.iter().enumerate().map(|(i, notification)| {
                // Each toast slides up when new one appears
                NotificationToast::new(notification.clone(), self.theme.clone())
                    .with_animation(
                        format!("stack-{}", notification.id.0),
                        Animation::new()
                            .duration(Duration::from_millis(150))
                            .easing(ease_out_cubic),
                        move |el, _progress| el,
                    )
            }))
    }
}
```

## Best Practices

### Duration Guidelines

```rust
pub mod notification_durations {
    use std::time::Duration;
    
    /// Success confirmations - quick acknowledgment
    pub const SUCCESS: Duration = Duration::from_millis(2000);
    
    /// Informational - time to read
    pub const INFO: Duration = Duration::from_secs(4);
    
    /// Warnings - important to notice
    pub const WARNING: Duration = Duration::from_secs(6);
    
    /// Errors - time to understand
    pub const ERROR: Duration = Duration::from_secs(8);
    
    /// With undo action - time to act
    pub const UNDOABLE: Duration = Duration::from_secs(8);
    
    /// Critical/blocking - manual dismiss
    pub const CRITICAL: Option<Duration> = None;
}
```

### Content Guidelines

```rust
/// Notification content best practices
pub mod guidelines {
    // Titles: 2-4 words, action verb past tense
    // ✅ "Copied to clipboard"
    // ✅ "Script saved"
    // ❌ "The file has been successfully saved to disk"
    
    // Messages: Optional detail, 1 sentence max
    // ✅ "Press ⌘Z to undo"
    // ❌ "Your changes have been saved. You can undo this action..."
    
    // Actions: Verb or short phrase
    // ✅ "Undo", "Retry", "View"
    // ❌ "Click here to undo", "OK"
}
```

### Stacking Rules

```rust
impl NotificationManager {
    /// Rules for notification stacking
    fn should_replace(&self, new: &Notification) -> Option<NotificationId> {
        // Replace duplicate success notifications
        if new.level == NotificationLevel::Success {
            for existing in &self.notifications {
                if existing.level == NotificationLevel::Success 
                    && existing.title == new.title {
                    return Some(existing.id);
                }
            }
        }
        
        // Replace progress notifications with same title
        if new.message.as_deref() == Some("progress") {
            for existing in &self.notifications {
                if existing.title == new.title {
                    return Some(existing.id);
                }
            }
        }
        
        None
    }
}
```

## Testing

### Toast Test Script

```typescript
// tests/smoke/test-toast-notifications.ts
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Test 1: Success toast
await div(`
  <div class="relative h-64">
    <div class="absolute bottom-4 right-4">
      <div class="w-80 p-3 rounded-lg bg-zinc-800 border-l-4 border-green-500 shadow-lg">
        <div class="flex items-start gap-3">
          <span class="text-green-400">✓</span>
          <div class="flex-1">
            <div class="text-sm font-medium text-white">Copied to clipboard</div>
          </div>
          <span class="text-zinc-500 cursor-pointer text-sm">×</span>
        </div>
      </div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot1 = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
writeFileSync(join(dir, 'toast-success.png'), Buffer.from(shot1.data, 'base64'));

// Test 2: Error toast with action
await div(`
  <div class="relative h-64">
    <div class="absolute bottom-4 right-4">
      <div class="w-80 p-3 rounded-lg bg-zinc-800 border-l-4 border-red-500 shadow-lg">
        <div class="flex items-start gap-3">
          <span class="text-red-400">✕</span>
          <div class="flex-1 flex flex-col gap-2">
            <div class="text-sm font-medium text-white">Save failed</div>
            <div class="text-xs text-zinc-400">Permission denied</div>
            <div class="flex gap-2 mt-1">
              <button class="px-3 py-1 rounded-sm text-xs bg-amber-500 text-black">Retry</button>
              <button class="px-3 py-1 rounded-sm text-xs bg-zinc-700 text-white border border-zinc-600">View Log</button>
            </div>
          </div>
          <span class="text-zinc-500 cursor-pointer text-sm">×</span>
        </div>
      </div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot2 = await captureScreenshot();
writeFileSync(join(dir, 'toast-error.png'), Buffer.from(shot2.data, 'base64'));

// Test 3: Stacked toasts
await div(`
  <div class="relative h-80">
    <div class="absolute bottom-4 right-4 flex flex-col gap-2">
      <div class="w-80 p-3 rounded-lg bg-zinc-800 border-l-4 border-blue-500 shadow-lg">
        <div class="flex items-start gap-3">
          <span class="text-blue-400">ℹ</span>
          <div class="text-sm font-medium text-white">Update available</div>
        </div>
      </div>
      <div class="w-80 p-3 rounded-lg bg-zinc-800 border-l-4 border-yellow-500 shadow-lg">
        <div class="flex items-start gap-3">
          <span class="text-yellow-400">⚠</span>
          <div class="text-sm font-medium text-white">Script deprecated</div>
        </div>
      </div>
      <div class="w-80 p-3 rounded-lg bg-zinc-800 border-l-4 border-green-500 shadow-lg">
        <div class="flex items-start gap-3">
          <span class="text-green-400">✓</span>
          <div class="text-sm font-medium text-white">Settings saved</div>
        </div>
      </div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot3 = await captureScreenshot();
writeFileSync(join(dir, 'toast-stacked.png'), Buffer.from(shot3.data, 'base64'));

// Test 4: Progress toast
await div(`
  <div class="relative h-64">
    <div class="absolute bottom-4 right-4">
      <div class="w-80 p-3 rounded-lg bg-zinc-800 shadow-lg">
        <div class="flex flex-col gap-2">
          <div class="text-sm font-medium text-white">Downloading update...</div>
          <div class="w-full h-1 rounded-full bg-zinc-700">
            <div class="h-full rounded-full bg-amber-500" style="width: 65%"></div>
          </div>
          <div class="text-xs text-zinc-500">65% • 2.3 MB / 3.5 MB</div>
        </div>
      </div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot4 = await captureScreenshot();
writeFileSync(join(dir, 'toast-progress.png'), Buffer.from(shot4.data, 'base64'));

console.error('[TOAST NOTIFICATIONS] Test screenshots saved');
process.exit(0);
```

## Related Bundles

- Bundle #76: Error States - Error notification content
- Bundle #77: Success Feedback - Success notification patterns
- Bundle #55: Animation & Transitions - Toast animations
- Bundle #62: Logging & Observability - Logging notification events
