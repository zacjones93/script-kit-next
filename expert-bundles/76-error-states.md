# Expert Bundle #76: Error States

## Overview

Error states communicate failures clearly and help users recover. Script Kit encounters errors from script execution failures, file system issues, API errors, and validation problems. Good error handling provides context, suggests solutions, and maintains user trust.

## Architecture

### Error Classification

```rust
// src/error.rs
use thiserror::Error;

/// Error severity determines UI treatment
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Informational - operation partially succeeded
    Info,
    /// Warning - something unexpected but recoverable
    Warning,
    /// Error - operation failed, user action needed
    Error,
    /// Critical - system-level failure
    Critical,
}

/// Structured error with UI metadata
#[derive(Clone, Debug, Error)]
pub struct ScriptKitError {
    /// Error category for grouping/filtering
    pub category: ErrorCategory,
    /// Severity level
    pub severity: ErrorSeverity,
    /// User-friendly title
    pub title: String,
    /// Detailed message
    pub message: String,
    /// Technical details (logs, stack trace)
    pub details: Option<String>,
    /// Suggested recovery actions
    pub recovery_actions: Vec<RecoveryAction>,
    /// Related documentation URL
    pub docs_url: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorCategory {
    ScriptExecution,
    ScriptParsing,
    FileSystem,
    Network,
    Configuration,
    Permission,
    Validation,
    Internal,
}

#[derive(Clone, Debug)]
pub struct RecoveryAction {
    pub label: String,
    pub action_type: RecoveryActionType,
}

#[derive(Clone, Debug)]
pub enum RecoveryActionType {
    Retry,
    OpenFile(PathBuf),
    OpenUrl(String),
    RunCommand(String),
    Dismiss,
    Custom(Box<dyn Fn(&mut WindowContext) + Send + Sync>),
}
```

### Error Display Components

```rust
// src/components/error_display.rs
use crate::theme::Theme;
use gpui::*;

/// Inline error for form fields
pub struct InlineError {
    message: SharedString,
    theme: Arc<Theme>,
}

impl InlineError {
    pub fn new(message: impl Into<SharedString>, theme: Arc<Theme>) -> Self {
        Self {
            message: message.into(),
            theme,
        }
    }
}

impl Render for InlineError {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_1()
            .py_1()
            .child(
                Icon::new("alert-circle")
                    .size(px(12.0))
                    .color(rgb(colors.semantic.error))
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(rgb(colors.semantic.error))
                    .child(self.message.clone())
            )
    }
}

/// Error banner for page-level errors
pub struct ErrorBanner {
    error: ScriptKitError,
    theme: Arc<Theme>,
    expanded: bool,
}

impl ErrorBanner {
    pub fn new(error: ScriptKitError, theme: Arc<Theme>) -> Self {
        Self {
            error,
            theme,
            expanded: false,
        }
    }
    
    fn severity_color(&self) -> u32 {
        let colors = &self.theme.colors;
        match self.error.severity {
            ErrorSeverity::Info => colors.semantic.info,
            ErrorSeverity::Warning => colors.semantic.warning,
            ErrorSeverity::Error => colors.semantic.error,
            ErrorSeverity::Critical => colors.semantic.error,
        }
    }
    
    fn severity_icon(&self) -> &'static str {
        match self.error.severity {
            ErrorSeverity::Info => "info",
            ErrorSeverity::Warning => "alert-triangle",
            ErrorSeverity::Error => "x-circle",
            ErrorSeverity::Critical => "alert-octagon",
        }
    }
}

impl Render for ErrorBanner {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        let severity_color = self.severity_color();
        
        div()
            .w_full()
            .p_3()
            .rounded_md()
            .bg(with_alpha(severity_color, 0.1))
            .border_l_4()
            .border_color(rgb(severity_color))
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
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        Icon::new(self.severity_icon())
                                            .size(px(16.0))
                                            .color(rgb(severity_color))
                                    )
                                    .child(
                                        div()
                                            .text_size(px(14.0))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(rgb(colors.text.primary))
                                            .child(self.error.title.clone())
                                    )
                            )
                            .child(
                                div()
                                    .cursor_pointer()
                                    .on_click(cx.listener(|this, _, _, _| {
                                        this.expanded = !this.expanded;
                                    }))
                                    .child(
                                        Icon::new(if self.expanded { "chevron-up" } else { "chevron-down" })
                                            .size(px(16.0))
                                            .color(rgb(colors.text.muted))
                                    )
                            )
                    )
                    // Message
                    .child(
                        div()
                            .text_size(px(13.0))
                            .text_color(rgb(colors.text.secondary))
                            .child(self.error.message.clone())
                    )
                    // Expanded details
                    .when(self.expanded && self.error.details.is_some(), |el| {
                        el.child(
                            div()
                                .mt_2()
                                .p_2()
                                .rounded_sm()
                                .bg(rgb(colors.background.secondary))
                                .font_family("monospace")
                                .text_size(px(11.0))
                                .text_color(rgb(colors.text.muted))
                                .overflow_x_auto()
                                .child(self.error.details.clone().unwrap())
                        )
                    })
                    // Recovery actions
                    .when(!self.error.recovery_actions.is_empty(), |el| {
                        el.child(
                            div()
                                .flex()
                                .flex_row()
                                .gap_2()
                                .mt_2()
                                .children(self.error.recovery_actions.iter().map(|action| {
                                    div()
                                        .px_3()
                                        .py_1()
                                        .rounded_sm()
                                        .text_size(px(12.0))
                                        .cursor_pointer()
                                        .bg(rgb(colors.ui.surface))
                                        .text_color(rgb(colors.text.primary))
                                        .hover(|s| s.bg(rgb(colors.ui.hover)))
                                        .child(action.label.clone())
                                }))
                        )
                    })
            )
    }
}

/// Full-page error state
pub struct ErrorPage {
    error: ScriptKitError,
    theme: Arc<Theme>,
}

impl ErrorPage {
    pub fn new(error: ScriptKitError, theme: Arc<Theme>) -> Self {
        Self { error, theme }
    }
}

impl Render for ErrorPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        div()
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .p_8()
            .gap_6()
            // Error icon
            .child(
                div()
                    .w(px(80.0))
                    .h(px(80.0))
                    .rounded_full()
                    .bg(with_alpha(colors.semantic.error, 0.1))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        Icon::new("alert-circle")
                            .size(px(40.0))
                            .color(rgb(colors.semantic.error))
                    )
            )
            // Error content
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_2()
                    .max_w(px(400.0))
                    .child(
                        div()
                            .text_size(px(18.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(colors.text.primary))
                            .child(self.error.title.clone())
                    )
                    .child(
                        div()
                            .text_size(px(14.0))
                            .text_color(rgb(colors.text.muted))
                            .text_align(TextAlign::Center)
                            .child(self.error.message.clone())
                    )
            )
            // Actions
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_3()
                    .children(self.error.recovery_actions.iter().enumerate().map(|(i, action)| {
                        let is_primary = i == 0;
                        div()
                            .px_4()
                            .py_2()
                            .rounded_md()
                            .cursor_pointer()
                            .when(is_primary, |el| {
                                el.bg(rgb(colors.accent.primary))
                                    .text_color(rgb(colors.background.main))
                                    .hover(|s| s.bg(rgb(colors.accent.hover)))
                            })
                            .when(!is_primary, |el| {
                                el.border_1()
                                    .border_color(rgb(colors.ui.border))
                                    .text_color(rgb(colors.text.primary))
                                    .hover(|s| s.bg(rgb(colors.ui.hover)))
                            })
                            .child(action.label.clone())
                    }))
            )
    }
}
```

### Script Execution Errors

```rust
// src/executor.rs - Error handling for script execution
impl ScriptExecutor {
    pub fn handle_script_error(
        &self,
        script: &Script,
        exit_code: i32,
        stderr: &str,
    ) -> ScriptKitError {
        // Parse stderr for known error patterns
        let (title, message, recovery) = if stderr.contains("Cannot find module") {
            let module = self.extract_module_name(stderr);
            (
                "Module not found".to_string(),
                format!("The script requires '{}' which is not installed", module),
                vec![RecoveryAction {
                    label: format!("Install {}", module),
                    action_type: RecoveryActionType::RunCommand(
                        format!("bun add {}", module)
                    ),
                }],
            )
        } else if stderr.contains("SyntaxError") {
            let location = self.extract_syntax_location(stderr);
            (
                "Syntax error".to_string(),
                format!("There's a syntax error in the script at {}", location),
                vec![RecoveryAction {
                    label: "Open in editor".to_string(),
                    action_type: RecoveryActionType::OpenFile(script.path.clone()),
                }],
            )
        } else if stderr.contains("Permission denied") {
            (
                "Permission denied".to_string(),
                "The script doesn't have permission to perform this action".to_string(),
                vec![RecoveryAction {
                    label: "Check permissions".to_string(),
                    action_type: RecoveryActionType::OpenUrl(
                        "https://docs.scriptkit.com/permissions".to_string()
                    ),
                }],
            )
        } else {
            (
                format!("Script failed (exit code {})", exit_code),
                "The script encountered an error".to_string(),
                vec![
                    RecoveryAction {
                        label: "View logs".to_string(),
                        action_type: RecoveryActionType::OpenFile(
                            dirs::home_dir()
                                .unwrap()
                                .join(".scriptkit/logs/script-kit-gpui.jsonl")
                        ),
                    },
                    RecoveryAction {
                        label: "Edit script".to_string(),
                        action_type: RecoveryActionType::OpenFile(script.path.clone()),
                    },
                ],
            )
        };
        
        ScriptKitError {
            category: ErrorCategory::ScriptExecution,
            severity: ErrorSeverity::Error,
            title,
            message,
            details: Some(stderr.to_string()),
            recovery_actions: recovery,
            docs_url: Some("https://docs.scriptkit.com/troubleshooting".to_string()),
        }
    }
}
```

## Error Display Patterns

### Toast Notifications (Ephemeral Errors)

```rust
// src/notifications.rs
impl NotificationManager {
    pub fn show_error(&self, error: &ScriptKitError, cx: &mut WindowContext) {
        let duration = match error.severity {
            ErrorSeverity::Info => Duration::from_secs(3),
            ErrorSeverity::Warning => Duration::from_secs(5),
            ErrorSeverity::Error => Duration::from_secs(8),
            ErrorSeverity::Critical => Duration::from_secs(15), // Longer for critical
        };
        
        self.show_notification(
            Notification::new(error.title.clone())
                .message(error.message.clone())
                .severity(error.severity)
                .duration(duration)
                .actions(error.recovery_actions.clone()),
            cx,
        );
    }
}
```

### Inline Validation Errors

```rust
// src/prompts/arg.rs
impl ArgPrompt {
    fn render_input_with_validation(&self, cx: &mut WindowContext) -> impl IntoElement {
        let colors = &self.theme.colors;
        let has_error = self.validation_error.is_some();
        
        div()
            .flex()
            .flex_col()
            .gap_1()
            // Input field
            .child(
                div()
                    .w_full()
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .border_1()
                    .border_color(rgb(if has_error {
                        colors.semantic.error
                    } else {
                        colors.ui.border
                    }))
                    .bg(rgb(colors.ui.input))
                    .child(/* input element */)
            )
            // Error message
            .when_some(self.validation_error.clone(), |el, error| {
                el.child(InlineError::new(error, self.theme.clone()))
            })
    }
    
    fn validate_input(&mut self, value: &str) -> Option<String> {
        if let Some(validator) = &self.validator {
            match validator.validate(value) {
                Ok(()) => None,
                Err(msg) => Some(msg),
            }
        } else {
            None
        }
    }
}
```

### Error Recovery Flow

```rust
// src/error_recovery.rs
impl App {
    /// Handle error with optional automatic recovery
    pub fn handle_error_with_recovery(
        &mut self,
        error: ScriptKitError,
        cx: &mut WindowContext,
    ) {
        // Log the error
        tracing::error!(
            category = ?error.category,
            severity = ?error.severity,
            title = %error.title,
            message = %error.message,
            "Error occurred"
        );
        
        // Determine display method based on context
        match error.severity {
            ErrorSeverity::Info | ErrorSeverity::Warning => {
                // Show as toast
                self.notification_manager.show_error(&error, cx);
            }
            ErrorSeverity::Error => {
                // Show as banner if in active view, otherwise toast
                if self.has_active_prompt() {
                    self.show_error_banner(error, cx);
                } else {
                    self.notification_manager.show_error(&error, cx);
                }
            }
            ErrorSeverity::Critical => {
                // Show as full-page error
                self.show_error_page(error, cx);
            }
        }
    }
    
    fn show_error_banner(&mut self, error: ScriptKitError, cx: &mut WindowContext) {
        self.current_error = Some(error);
        cx.notify();
    }
    
    fn show_error_page(&mut self, error: ScriptKitError, cx: &mut WindowContext) {
        self.view_state = ViewState::Error(error);
        cx.notify();
    }
}
```

## Best Practices

### Error Message Guidelines

```rust
/// Error message writing guidelines
pub mod error_messages {
    // ✅ Good: Specific, actionable
    pub const MODULE_NOT_FOUND: &str = "Install the missing package with: bun add {module}";
    
    // ❌ Bad: Vague, unhelpful
    pub const GENERIC_ERROR: &str = "An error occurred";
    
    // ✅ Good: User-centric language
    pub const PERMISSION_ERROR: &str = "Script Kit needs permission to access your clipboard";
    
    // ❌ Bad: Technical jargon
    pub const PERMISSION_ERROR_BAD: &str = "EPERM: operation not permitted";
    
    // ✅ Good: Suggests next step
    pub const CONFIG_ERROR: &str = "Your config file has a syntax error. Open it in your editor to fix.";
    
    // ❌ Bad: Dead end
    pub const CONFIG_ERROR_BAD: &str = "Invalid configuration";
}

/// Recovery action principles
/// 1. Primary action: Most likely solution
/// 2. Secondary: Alternative approach
/// 3. Tertiary: Get more help
pub fn create_recovery_actions(error: &ScriptKitError) -> Vec<RecoveryAction> {
    match error.category {
        ErrorCategory::ScriptExecution => vec![
            RecoveryAction { label: "Retry".into(), action_type: RecoveryActionType::Retry },
            RecoveryAction { label: "Edit Script".into(), action_type: RecoveryActionType::OpenFile(/* path */) },
            RecoveryAction { label: "View Docs".into(), action_type: RecoveryActionType::OpenUrl(/* url */) },
        ],
        ErrorCategory::Network => vec![
            RecoveryAction { label: "Retry".into(), action_type: RecoveryActionType::Retry },
            RecoveryAction { label: "Check Connection".into(), action_type: RecoveryActionType::Custom(/* ... */) },
        ],
        // ...
    }
}
```

### Preserving User Data

```rust
// Never lose user input on error
impl EditorPrompt {
    fn handle_save_error(&mut self, error: std::io::Error, cx: &mut WindowContext) {
        // Keep the content in the editor
        // Don't clear or reset
        
        // Show error with retry option
        self.error = Some(ScriptKitError {
            category: ErrorCategory::FileSystem,
            severity: ErrorSeverity::Error,
            title: "Couldn't save".to_string(),
            message: format!("Failed to save: {}", error),
            details: None,
            recovery_actions: vec![
                RecoveryAction {
                    label: "Try Again".to_string(),
                    action_type: RecoveryActionType::Retry,
                },
                RecoveryAction {
                    label: "Save As...".to_string(),
                    action_type: RecoveryActionType::Custom(Box::new(|cx| {
                        // Open save dialog
                    })),
                },
                RecoveryAction {
                    label: "Copy to Clipboard".to_string(),
                    action_type: RecoveryActionType::Custom(Box::new(|cx| {
                        // Copy content to clipboard as backup
                    })),
                },
            ],
            docs_url: None,
        });
        
        cx.notify();
    }
}
```

## Testing

### Error State Test Script

```typescript
// tests/smoke/test-error-states.ts
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Test 1: Inline validation error
await div(`
  <div class="p-4 flex flex-col gap-1">
    <input 
      type="text" 
      class="w-full px-3 py-2 rounded-md border border-red-500 bg-zinc-800 text-white"
      placeholder="Enter email"
      value="invalid-email"
    />
    <div class="flex items-center gap-1 text-red-400 text-xs">
      <span>⚠️</span>
      <span>Please enter a valid email address</span>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot1 = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
writeFileSync(join(dir, 'error-inline.png'), Buffer.from(shot1.data, 'base64'));

// Test 2: Error banner
await div(`
  <div class="p-4">
    <div class="w-full p-3 rounded-md bg-red-900/20 border-l-4 border-red-500">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <span class="text-red-400">❌</span>
          <span class="font-semibold text-white">Script failed</span>
        </div>
        <span class="text-zinc-500 cursor-pointer">▼</span>
      </div>
      <div class="text-sm text-zinc-400 mt-1">
        The script encountered an unexpected error during execution
      </div>
      <div class="flex gap-2 mt-3">
        <button class="px-3 py-1 rounded-sm text-xs bg-zinc-700 text-white hover:bg-zinc-600">
          Retry
        </button>
        <button class="px-3 py-1 rounded-sm text-xs bg-zinc-700 text-white hover:bg-zinc-600">
          View Logs
        </button>
      </div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot2 = await captureScreenshot();
writeFileSync(join(dir, 'error-banner.png'), Buffer.from(shot2.data, 'base64'));

// Test 3: Full page error
await div(`
  <div class="w-full h-96 flex flex-col items-center justify-center p-8 gap-6">
    <div class="w-20 h-20 rounded-full bg-red-900/20 flex items-center justify-center">
      <span class="text-4xl">⚠️</span>
    </div>
    <div class="text-center">
      <div class="text-lg font-semibold text-white">Configuration Error</div>
      <div class="text-sm text-zinc-400 mt-2 max-w-sm">
        Your config.ts file contains a syntax error that prevents Script Kit from starting.
      </div>
    </div>
    <div class="flex gap-3">
      <button class="px-4 py-2 rounded-md bg-amber-500 text-black font-medium">
        Open Config
      </button>
      <button class="px-4 py-2 rounded-md border border-zinc-600 text-white">
        Reset to Default
      </button>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot3 = await captureScreenshot();
writeFileSync(join(dir, 'error-page.png'), Buffer.from(shot3.data, 'base64'));

console.error('[ERROR STATES] Test screenshots saved');
process.exit(0);
```

## Related Bundles

- Bundle #51: Error Handling Patterns - Core error architecture
- Bundle #75: Empty States - Related "nothing here" states
- Bundle #77: Success Feedback - Positive counterpart
- Bundle #78: Toast Notifications - Ephemeral error display
