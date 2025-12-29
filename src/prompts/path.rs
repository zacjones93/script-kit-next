//! PathPrompt - File/folder picker prompt
//!
//! Features:
//! - Browse file system starting from optional path
//! - Filter files/folders by name
//! - Navigate with keyboard
//! - Submit selected path

use gpui::{
    div, prelude::*, px, rgb, uniform_list, Context, FocusHandle, Focusable, Render, 
    UniformListScrollHandle, Window,
};
use std::sync::Arc;
use std::path::Path;

use crate::logging;
use crate::theme;
use crate::designs::{DesignVariant, get_tokens};

/// Callback for prompt submission
/// Signature: (id: String, value: Option<String>)
pub type SubmitCallback = Arc<dyn Fn(String, Option<String>) + Send + Sync>;

/// PathPrompt - File/folder picker
///
/// Provides a file browser interface for selecting files or directories.
/// Supports starting from a specified path and filtering by name.
pub struct PathPrompt {
    /// Unique ID for this prompt instance
    pub id: String,
    /// Starting directory path (defaults to home if None)
    pub start_path: Option<String>,
    /// Hint text to display
    pub hint: Option<String>,
    /// Current directory being browsed
    pub current_path: String,
    /// Filter text for narrowing down results
    pub filter_text: String,
    /// Currently selected index in the list
    pub selected_index: usize,
    /// List of entries in current directory
    pub entries: Vec<PathEntry>,
    /// Filtered entries based on filter_text
    pub filtered_entries: Vec<PathEntry>,
    /// Focus handle for keyboard input
    pub focus_handle: FocusHandle,
    /// Callback when user submits a selection
    pub on_submit: SubmitCallback,
    /// Theme for styling
    pub theme: Arc<theme::Theme>,
    /// Design variant for styling
    pub design_variant: DesignVariant,
    /// Scroll handle for the list
    pub list_scroll_handle: UniformListScrollHandle,
}

/// A file system entry (file or directory)
#[derive(Clone, Debug)]
pub struct PathEntry {
    /// Display name
    pub name: String,
    /// Full path
    pub path: String,
    /// Whether this is a directory
    pub is_dir: bool,
}

impl PathPrompt {
    pub fn new(
        id: String,
        start_path: Option<String>,
        hint: Option<String>,
        focus_handle: FocusHandle,
        on_submit: SubmitCallback,
        theme: Arc<theme::Theme>,
    ) -> Self {
        let current_path = start_path.clone()
            .unwrap_or_else(|| dirs::home_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "/".to_string()));
        
        logging::log("PROMPTS", &format!("PathPrompt::new starting at: {}", current_path));
        
        // Load entries from current path
        let entries = Self::load_entries(&current_path);
        let filtered_entries = entries.clone();
        
        PathPrompt {
            id,
            start_path,
            hint,
            current_path,
            filter_text: String::new(),
            selected_index: 0,
            entries,
            filtered_entries,
            focus_handle,
            on_submit,
            theme,
            design_variant: DesignVariant::Default,
            list_scroll_handle: UniformListScrollHandle::new(),
        }
    }
    
    /// Load directory entries from a path
    fn load_entries(dir_path: &str) -> Vec<PathEntry> {
        let path = Path::new(dir_path);
        let mut entries = Vec::new();
        
        // Add parent directory entry if not at root
        if let Some(parent) = path.parent() {
            entries.push(PathEntry {
                name: "..".to_string(),
                path: parent.to_string_lossy().to_string(),
                is_dir: true,
            });
        }
        
        // Read directory entries
        if let Ok(read_dir) = std::fs::read_dir(path) {
            let mut dirs: Vec<PathEntry> = Vec::new();
            let mut files: Vec<PathEntry> = Vec::new();
            
            for entry in read_dir.flatten() {
                let entry_path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                
                // Skip hidden files (starting with .)
                if name.starts_with('.') {
                    continue;
                }
                
                let is_dir = entry_path.is_dir();
                let path_entry = PathEntry {
                    name,
                    path: entry_path.to_string_lossy().to_string(),
                    is_dir,
                };
                
                if is_dir {
                    dirs.push(path_entry);
                } else {
                    files.push(path_entry);
                }
            }
            
            // Sort alphabetically (case insensitive)
            dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            
            // Add dirs first, then files
            entries.extend(dirs);
            entries.extend(files);
        }
        
        logging::log("PROMPTS", &format!("PathPrompt loaded {} entries from {}", entries.len(), dir_path));
        entries
    }
    
    /// Update filtered entries based on filter text
    fn update_filtered(&mut self) {
        if self.filter_text.is_empty() {
            self.filtered_entries = self.entries.clone();
        } else {
            let filter_lower = self.filter_text.to_lowercase();
            self.filtered_entries = self.entries
                .iter()
                .filter(|e| e.name.to_lowercase().contains(&filter_lower))
                .cloned()
                .collect();
        }
        
        // Reset selection to 0 if out of bounds
        if self.selected_index >= self.filtered_entries.len() {
            self.selected_index = 0;
        }
    }
    
    /// Navigate into a directory
    fn navigate_to(&mut self, path: &str, cx: &mut Context<Self>) {
        self.current_path = path.to_string();
        self.entries = Self::load_entries(path);
        self.filter_text.clear();
        self.filtered_entries = self.entries.clone();
        self.selected_index = 0;
        cx.notify();
    }

    /// Submit the selected path or navigate into directory
    fn submit_selected(&mut self, cx: &mut Context<Self>) {
        if let Some(entry) = self.filtered_entries.get(self.selected_index) {
            if entry.is_dir {
                // Navigate into directory
                let path = entry.path.clone();
                self.navigate_to(&path, cx);
            } else {
                // Submit file path
                (self.on_submit)(self.id.clone(), Some(entry.path.clone()));
            }
        } else if !self.filter_text.is_empty() {
            // If no entry selected but filter has text, submit the filter as a path
            (self.on_submit)(self.id.clone(), Some(self.filter_text.clone()));
        }
    }

    /// Cancel - submit None
    fn submit_cancel(&mut self) {
        (self.on_submit)(self.id.clone(), None);
    }

    /// Move selection up
    fn move_up(&mut self, cx: &mut Context<Self>) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.list_scroll_handle.scroll_to_item(self.selected_index, gpui::ScrollStrategy::Top);
            cx.notify();
        }
    }

    /// Move selection down
    fn move_down(&mut self, cx: &mut Context<Self>) {
        if self.selected_index < self.filtered_entries.len().saturating_sub(1) {
            self.selected_index += 1;
            self.list_scroll_handle.scroll_to_item(self.selected_index, gpui::ScrollStrategy::Top);
            cx.notify();
        }
    }

    /// Handle character input
    fn handle_char(&mut self, ch: char, cx: &mut Context<Self>) {
        self.filter_text.push(ch);
        self.update_filtered();
        cx.notify();
    }

    /// Handle backspace - if filter empty, go up one directory
    fn handle_backspace(&mut self, cx: &mut Context<Self>) {
        if !self.filter_text.is_empty() {
            self.filter_text.pop();
            self.update_filtered();
            cx.notify();
        } else {
            // If filter is empty, navigate up one directory
            let path = Path::new(&self.current_path);
            if let Some(parent) = path.parent() {
                let parent_path = parent.to_string_lossy().to_string();
                self.navigate_to(&parent_path, cx);
            }
        }
    }
}

impl Focusable for PathPrompt {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

/// Height of each list item in pixels
const ITEM_HEIGHT: f32 = 36.0;

impl Render for PathPrompt {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tokens = get_tokens(self.design_variant);
        let colors = tokens.colors();
        let spacing = tokens.spacing();

        let handle_key = cx.listener(|this: &mut Self, event: &gpui::KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>| {
            let key_str = event.keystroke.key.to_lowercase();
            
            match key_str.as_str() {
                "up" | "arrowup" => this.move_up(cx),
                "down" | "arrowdown" => this.move_down(cx),
                "enter" => this.submit_selected(cx),
                "escape" => this.submit_cancel(),
                "backspace" => this.handle_backspace(cx),
                _ => {
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

        let (main_bg, text_color, text_muted, selected_bg, border_color) = if self.design_variant == DesignVariant::Default {
            (
                rgb(self.theme.colors.background.main), 
                rgb(self.theme.colors.text.primary),
                rgb(self.theme.colors.text.secondary),
                rgb(self.theme.colors.accent.selected),
                rgb(self.theme.colors.ui.border),
            )
        } else {
            (
                rgb(colors.background), 
                rgb(colors.text_primary),
                rgb(colors.text_secondary),
                rgb(colors.accent),
                rgb(colors.border),
            )
        };

        // Clone values needed for the closure
        let filtered_count = self.filtered_entries.len();
        let selected_index = self.selected_index;
        
        // Clone entries for the closure (uniform_list callback doesn't have access to self)
        let entries_for_list: Vec<(String, bool)> = self.filtered_entries
            .iter()
            .map(|e| (e.name.clone(), e.is_dir))
            .collect();
        
        // Build list items
        let list = uniform_list(
            "path-list",
            filtered_count,
            move |visible_range: std::ops::Range<usize>, _window, _cx| {
                visible_range.map(|ix| {
                    let (name, is_dir) = &entries_for_list[ix];
                    let is_selected = ix == selected_index;
                    
                    let icon = if name == ".." {
                        "⬆️"
                    } else if *is_dir {
                        "📁"
                    } else {
                        "📄"
                    };
                    
                    let item_bg = if is_selected {
                        selected_bg
                    } else {
                        main_bg
                    };
                    
                    div()
                        .id(ix)
                        .h(px(ITEM_HEIGHT))
                        .w_full()
                        .flex()
                        .items_center()
                        .gap_2()
                        .px_3()
                        .bg(item_bg)
                        .rounded_md()
                        .child(
                            div().child(icon)
                        )
                        .child(
                            div()
                                .flex_1()
                                .overflow_hidden()
                                .child(name.clone())
                        )
                        .when(*is_dir && name != "..", |d| {
                            d.child(
                                div()
                                    .text_xs()
                                    .text_color(text_muted)
                                    .child("→")
                            )
                        })
                })
                .collect()
            },
        )
        .track_scroll(&self.list_scroll_handle)
        .flex_1()
        .w_full();

        // Header showing current path
        let header = div()
            .w_full()
            .pb_2()
            .border_b_1()
            .border_color(border_color)
            .child(
                div()
                    .text_sm()
                    .text_color(text_muted)
                    .child(self.current_path.clone())
            )
            .when(!self.filter_text.is_empty(), |d| {
                d.child(
                    div()
                        .pt_1()
                        .text_sm()
                        .child(format!("Filter: {}", self.filter_text))
                )
            });

        // Hint at bottom
        let hint_text = self.hint.clone().unwrap_or_else(|| {
            format!("{} items • ↑↓ navigate • Enter select • Esc cancel", filtered_count)
        });
        let footer = div()
            .w_full()
            .pt_2()
            .text_xs()
            .text_color(text_muted)
            .child(hint_text);

        div()
            .id(gpui::ElementId::Name("window:path".into()))
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .bg(main_bg)
            .text_color(text_color)
            .p(px(spacing.padding_lg))
            .gap_2()
            .key_context("path_prompt")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .child(header)
            .child(list)
            .child(footer)
    }
}
