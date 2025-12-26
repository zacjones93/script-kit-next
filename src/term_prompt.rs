//! Terminal prompt component for GPUI
//!
//! Renders terminal content and handles keyboard input.

use gpui::{
    div, prelude::*, rgb, Context, FocusHandle, Focusable, Render, SharedString, Window,
};
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info, warn};

use crate::terminal::{TerminalEvent, TerminalHandle};
use crate::theme::Theme;
use crate::prompts::SubmitCallback;

const SLOW_RENDER_THRESHOLD_MS: u128 = 16; // 60fps threshold

/// Terminal prompt GPUI component
pub struct TermPrompt {
    pub id: String,
    pub terminal: TerminalHandle,
    pub focus_handle: FocusHandle,
    pub on_submit: SubmitCallback,
    pub theme: Arc<Theme>,
    exited: bool,
    exit_code: Option<i32>,
}

impl TermPrompt {
    /// Create new terminal prompt
    pub fn new(
        id: String,
        command: Option<String>,
        focus_handle: FocusHandle,
        on_submit: SubmitCallback,
        theme: Arc<Theme>,
    ) -> anyhow::Result<Self> {
        let terminal = match command {
            Some(cmd) => TerminalHandle::with_command(&cmd, 80, 24)?,
            None => TerminalHandle::new(80, 24)?,
        };

        Ok(Self {
            id,
            terminal,
            focus_handle,
            on_submit,
            theme,
            exited: false,
            exit_code: None,
        })
    }

    /// Handle terminal exit
    fn handle_exit(&mut self, code: i32) {
        info!(code, "Terminal exited");
        self.exited = true;
        self.exit_code = Some(code);
        // Call submit callback with exit code
        (self.on_submit)(self.id.clone(), Some(code.to_string()));
    }

    /// Submit/cancel
    fn submit_cancel(&mut self) {
        (self.on_submit)(self.id.clone(), None);
    }
}

impl Focusable for TermPrompt {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TermPrompt {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let start = Instant::now();

        // Process terminal events
        if !self.exited {
            let events = self.terminal.process();
            for event in events {
                match event {
                    TerminalEvent::Exit(code) => self.handle_exit(code),
                    TerminalEvent::Bell => { /* could flash screen */ }
                    TerminalEvent::Title(_) => { /* could update title */ }
                    TerminalEvent::Output(_) => { /* handled by content() */ }
                }
            }
        }

        // Get terminal content
        let content = self.terminal.content();

        // Handle keyboard
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  _window: &mut Window,
                  cx: &mut Context<Self>| {
                let key_str = event.keystroke.key.to_lowercase();

                if key_str == "escape" {
                    this.submit_cancel();
                    return;
                }

                // Forward input to terminal
                if let Some(key_char) = &event.keystroke.key_char {
                    if let Err(e) = this.terminal.input(key_char.as_bytes()) {
                        warn!(error = %e, "Failed to send input to terminal");
                    }
                    cx.notify();
                } else {
                    // Handle special keys
                    let bytes: Option<&[u8]> = match key_str.as_str() {
                        "enter" => Some(b"\r"),
                        "backspace" => Some(b"\x7f"),
                        "tab" => Some(b"\t"),
                        "up" | "arrowup" => Some(b"\x1b[A"),
                        "down" | "arrowdown" => Some(b"\x1b[B"),
                        "right" | "arrowright" => Some(b"\x1b[C"),
                        "left" | "arrowleft" => Some(b"\x1b[D"),
                        _ => None,
                    };

                    if let Some(bytes) = bytes {
                        if let Err(e) = this.terminal.input(bytes) {
                            warn!(error = %e, "Failed to send special key to terminal");
                        }
                        cx.notify();
                    }
                }
            },
        );

        // Render terminal content
        let colors = &self.theme.colors;
        let mut lines_container = div()
            .flex()
            .flex_col()
            .flex_1()
            .w_full()
            .overflow_y_hidden()
            .font_family("monospace");

        for line in &content.lines {
            lines_container = lines_container.child(
                div()
                    .w_full()
                    .text_color(rgb(colors.text.primary))
                    .child(SharedString::from(line.clone())),
            );
        }

        // Log slow renders
        let elapsed = start.elapsed().as_millis();
        if elapsed > SLOW_RENDER_THRESHOLD_MS {
            warn!(elapsed_ms = elapsed, "Slow terminal render");
        } else {
            debug!(elapsed_ms = elapsed, "Terminal render");
        }

        // Main container
        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .bg(rgb(colors.background.main))
            .text_color(rgb(colors.text.primary))
            .key_context("term_prompt")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .child(lines_container)
    }
}
