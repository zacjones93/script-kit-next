//! Actions Dialog Module
//!
//! Provides a searchable action menu for quick access to script management
//! and global actions (edit, create, settings, quit, etc.)

use gpui::{
    div, prelude::*, px, rgb, Context, FocusHandle, Focusable, Render, SharedString, Window,
};
use std::sync::Arc;
use crate::logging;

/// Callback for action selection
/// Signature: (action_id: String)
pub type ActionCallback = Arc<dyn Fn(String) + Send + Sync>;

/// Available actions in the actions menu
#[derive(Debug, Clone)]
pub struct Action {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub category: ActionCategory,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActionCategory {
    ScriptOps,    // Edit, Create, Delete script operations
    GlobalOps,    // Settings, Quit, etc.
}

impl Action {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        description: Option<String>,
        category: ActionCategory,
    ) -> Self {
        Action {
            id: id.into(),
            title: title.into(),
            description,
            category,
        }
    }
}

/// Predefined global actions
pub fn get_global_actions() -> Vec<Action> {
    vec![
        Action::new(
            "create_script",
            "Create New Script",
            Some("Create a new TypeScript script in ~/.kenv/scripts".to_string()),
            ActionCategory::ScriptOps,
        ),
        Action::new(
            "edit_script",
            "Edit Selected Script",
            Some("Open selected script in $EDITOR".to_string()),
            ActionCategory::ScriptOps,
        ),
        Action::new(
            "reload_scripts",
            "Reload Scripts",
            Some("Refresh the scripts list".to_string()),
            ActionCategory::GlobalOps,
        ),
        Action::new(
            "settings",
            "Settings",
            Some("Configure hotkeys and preferences".to_string()),
            ActionCategory::GlobalOps,
        ),
        Action::new(
            "quit",
            "Quit Script Kit",
            Some("Exit the application (Cmd+Q)".to_string()),
            ActionCategory::GlobalOps,
        ),
    ]
}

/// ActionsDialog - Mini searchable popup for quick actions
pub struct ActionsDialog {
    pub actions: Vec<Action>,
    pub filtered_actions: Vec<usize>, // Indices into actions
    pub selected_index: usize,          // Index within filtered_actions
    pub search_text: String,
    pub focus_handle: FocusHandle,
    pub on_select: ActionCallback,
}

impl ActionsDialog {
    pub fn new(
        focus_handle: FocusHandle,
        on_select: ActionCallback,
    ) -> Self {
        let actions = get_global_actions();
        let filtered_actions: Vec<usize> = (0..actions.len()).collect();
        
        logging::log("ACTIONS", &format!("ActionsDialog created with {} actions", actions.len()));
        
        ActionsDialog {
            actions,
            filtered_actions,
            selected_index: 0,
            search_text: String::new(),
            focus_handle,
            on_select,
        }
    }

    /// Refilter actions based on current search_text using fuzzy matching
    fn refilter(&mut self) {
        if self.search_text.is_empty() {
            self.filtered_actions = (0..self.actions.len()).collect();
        } else {
            let search_lower = self.search_text.to_lowercase();
            self.filtered_actions = self
                .actions
                .iter()
                .enumerate()
                .filter(|(_, action)| {
                    let title_match = action.title.to_lowercase().contains(&search_lower);
                    let desc_match = action
                        .description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&search_lower))
                        .unwrap_or(false);
                    title_match || desc_match
                })
                .map(|(idx, _)| idx)
                .collect();
        }
        self.selected_index = 0; // Reset selection when filtering
    }

    /// Handle character input
    fn handle_char(&mut self, ch: char, cx: &mut Context<Self>) {
        self.search_text.push(ch);
        self.refilter();
        cx.notify();
    }

    /// Handle backspace
    fn handle_backspace(&mut self, cx: &mut Context<Self>) {
        if !self.search_text.is_empty() {
            self.search_text.pop();
            self.refilter();
            cx.notify();
        }
    }

    /// Move selection up
    fn move_up(&mut self, cx: &mut Context<Self>) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            cx.notify();
        }
    }

    /// Move selection down
    fn move_down(&mut self, cx: &mut Context<Self>) {
        if self.selected_index < self.filtered_actions.len().saturating_sub(1) {
            self.selected_index += 1;
            cx.notify();
        }
    }

    /// Submit the selected action
    fn submit_selected(&mut self) {
        if let Some(&action_idx) = self.filtered_actions.get(self.selected_index) {
            if let Some(action) = self.actions.get(action_idx) {
                logging::log("ACTIONS", &format!("Action selected: {}", action.id));
                (self.on_select)(action.id.clone());
            }
        }
    }

    /// Cancel - close the dialog
    fn submit_cancel(&mut self) {
        logging::log("ACTIONS", "Actions dialog cancelled");
        (self.on_select)("__cancel__".to_string());
    }
}

impl Focusable for ActionsDialog {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ActionsDialog {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let handle_key = cx.listener(move |this: &mut Self, event: &gpui::KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>| {
            let key_str = event.keystroke.key.to_lowercase();
            
            match key_str.as_str() {
                "up" | "arrowup" => this.move_up(cx),
                "down" | "arrowdown" => this.move_down(cx),
                "enter" => this.submit_selected(),
                "escape" => this.submit_cancel(),
                "backspace" => this.handle_backspace(cx),
                _ => {
                    // Try to capture printable characters
                    if let Some(ref key_char) = event.keystroke.key_char {
                        if let Some(ch) = key_char.chars().next() {
                            if !ch.is_control() {
                                this.handle_char(ch, cx);
                            }
                        }
                    }
                }
            }
        });

        // Render search input
        let search_display = if self.search_text.is_empty() {
            SharedString::from("Search actions...")
        } else {
            SharedString::from(self.search_text.clone())
        };

        let input_container = div()
            .w_full()
            .px(px(16.))
            .py(px(12.))
            .bg(rgb(0x2d2d2d))
            .border_b_1()
            .border_color(rgb(0x3d3d3d))
            .flex()
            .flex_row()
            .gap_2()
            .items_center()
            .child(div().text_color(rgb(0x888888)).child("⚙"))
            .child(
                div()
                    .flex_1()
                    .text_color(if self.search_text.is_empty() {
                        rgb(0x666666)
                    } else {
                        rgb(0xcccccc)
                    })
                    .child(search_display),
            );

        // Render action list
        let mut actions_container = div()
            .flex()
            .flex_col()
            .flex_1()
            .w_full()
            .max_h(px(400.));

        if self.filtered_actions.is_empty() {
            actions_container = actions_container.child(
                div()
                    .w_full()
                    .py(px(32.))
                    .px(px(16.))
                    .text_color(rgb(0x666666))
                    .child("No actions match your search"),
            );
        } else {
            for (idx, &action_idx) in self.filtered_actions.iter().enumerate() {
                if let Some(action) = self.actions.get(action_idx) {
                    let is_selected = idx == self.selected_index;
                    let bg = if is_selected {
                        rgb(0x0e47a1) // Blue highlight
                    } else {
                        rgb(0x1e1e1e)
                    };

                    let title_color = if is_selected {
                        rgb(0xffffff)
                    } else {
                        rgb(0xcccccc)
                    };

                    let desc_color = if is_selected {
                        rgb(0xaaaaaa)
                    } else {
                        rgb(0x888888)
                    };

                    let mut action_item = div()
                        .w_full()
                        .px(px(16.))
                        .py(px(10.))
                        .bg(bg)
                        .border_b_1()
                        .border_color(rgb(0x3d3d3d))
                        .flex()
                        .flex_col()
                        .gap_1();

                    // Action title
                    action_item = action_item.child(
                        div()
                            .text_color(title_color)
                            .text_base()
                            .child(action.title.clone()),
                    );

                    // Action description if present
                    if let Some(desc) = &action.description {
                        action_item = action_item.child(
                            div()
                                .text_color(desc_color)
                                .text_sm()
                                .child(desc.clone()),
                        );
                    }

                    actions_container = actions_container.child(action_item);
                }
            }
        }

        // Main dialog container - positioned as a modal/overlay
        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .bg(rgb(0x1e1e1e))
            .text_color(rgb(0xcccccc))
            .key_context("actions_dialog")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .child(input_container)
            .child(actions_container)
    }
}
