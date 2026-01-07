//! PromptFooter - Reusable footer component for prompts
//!
//! This module provides a theme-aware footer component used across all prompt types.
//! It includes a Script Kit logo, primary action button, divider, and secondary action button.
//!
//! # Structure
//! ```text
//! | [Logo] |                           | [Primary ↵] | [Secondary ⌘K] |
//! ```
//!
//! # Example
//! ```rust,ignore
//! let footer = PromptFooter::new(
//!     PromptFooterConfig::new()
//!         .primary_label("Run Script")
//!         .primary_shortcut("↵")
//!         .secondary_label("Actions")
//!         .secondary_shortcut("⌘K"),
//!     PromptFooterColors::from_theme(&theme),
//! )
//! .on_primary_click(Box::new(|_, _, _| { /* handle run */ }))
//! .on_secondary_click(Box::new(|_, _, _| { /* handle actions */ }));
//! ```

#![allow(dead_code)]

use gpui::*;
use std::rc::Rc;

use crate::designs::DesignColors;
use crate::theme::Theme;
use crate::ui_foundation::{hstack, HexColorExt};
use crate::utils;

/// Pre-computed colors for PromptFooter rendering
///
/// This struct holds the primitive color values needed for footer rendering,
/// allowing efficient use in closures without cloning the full theme.
#[derive(Clone, Copy, Debug)]
pub struct PromptFooterColors {
    /// Accent color for logo background and primary text
    pub accent: u32,
    /// Muted text color for shortcut hints
    pub text_muted: u32,
    /// Border color for top border and divider
    pub border: u32,
}

impl PromptFooterColors {
    /// Create PromptFooterColors from theme reference
    pub fn from_theme(theme: &Theme) -> Self {
        Self {
            accent: theme.colors.accent.selected,
            text_muted: theme.colors.text.muted,
            border: theme.colors.ui.border,
        }
    }

    /// Create PromptFooterColors from design colors for design system support
    pub fn from_design(colors: &DesignColors) -> Self {
        Self {
            accent: colors.accent,
            text_muted: colors.text_muted,
            border: colors.border,
        }
    }
}

impl Default for PromptFooterColors {
    fn default() -> Self {
        Self {
            accent: 0xfbbf24, // Script Kit yellow/gold
            text_muted: 0x808080,
            border: 0x464647,
        }
    }
}

/// Configuration for PromptFooter display
#[derive(Clone, Debug)]
pub struct PromptFooterConfig {
    /// Label for the primary button (e.g., "Run Script", "Submit", "Paste")
    pub primary_label: String,
    /// Shortcut hint for primary button (e.g., "↵", "⌘+Enter")
    pub primary_shortcut: String,
    /// Label for the secondary button (e.g., "Actions")
    pub secondary_label: String,
    /// Shortcut hint for secondary button (e.g., "⌘K")
    pub secondary_shortcut: String,
    /// Whether to show the Script Kit logo
    pub show_logo: bool,
    /// Whether to show the secondary button
    pub show_secondary: bool,
}

impl Default for PromptFooterConfig {
    fn default() -> Self {
        Self {
            primary_label: "Run Script".to_string(),
            primary_shortcut: "↵".to_string(),
            secondary_label: "Actions".to_string(),
            secondary_shortcut: "⌘K".to_string(),
            show_logo: true,
            show_secondary: true,
        }
    }
}

impl PromptFooterConfig {
    /// Create a new default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the primary button label
    pub fn primary_label(mut self, label: impl Into<String>) -> Self {
        self.primary_label = label.into();
        self
    }

    /// Set the primary button shortcut hint
    pub fn primary_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.primary_shortcut = shortcut.into();
        self
    }

    /// Set the secondary button label
    pub fn secondary_label(mut self, label: impl Into<String>) -> Self {
        self.secondary_label = label.into();
        self
    }

    /// Set the secondary button shortcut hint
    pub fn secondary_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.secondary_shortcut = shortcut.into();
        self
    }

    /// Set whether to show the logo
    pub fn show_logo(mut self, show: bool) -> Self {
        self.show_logo = show;
        self
    }

    /// Set whether to show the secondary button
    pub fn show_secondary(mut self, show: bool) -> Self {
        self.show_secondary = show;
        self
    }
}

/// Callback type for button click events
pub type FooterClickCallback = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// A reusable footer component for prompts
///
/// Displays:
/// - Script Kit logo (left, optional)
/// - Primary action button with shortcut
/// - Divider (optional, when secondary visible)
/// - Secondary action button with shortcut (optional)
///
/// Height: 40px fixed
#[derive(IntoElement)]
pub struct PromptFooter {
    config: PromptFooterConfig,
    colors: PromptFooterColors,
    on_primary_click: Option<Rc<FooterClickCallback>>,
    on_secondary_click: Option<Rc<FooterClickCallback>>,
}

impl PromptFooter {
    /// Create a new PromptFooter with the given configuration and colors
    pub fn new(config: PromptFooterConfig, colors: PromptFooterColors) -> Self {
        Self {
            config,
            colors,
            on_primary_click: None,
            on_secondary_click: None,
        }
    }

    /// Set the primary button click callback
    pub fn on_primary_click(mut self, callback: FooterClickCallback) -> Self {
        self.on_primary_click = Some(Rc::new(callback));
        self
    }

    /// Set the secondary button click callback
    pub fn on_secondary_click(mut self, callback: FooterClickCallback) -> Self {
        self.on_secondary_click = Some(Rc::new(callback));
        self
    }

    /// Render the Script Kit logo (20x20 yellow box with black icon)
    fn render_logo(&self) -> impl IntoElement {
        div()
            .w(px(20.))
            .h(px(20.))
            .flex()
            .items_center()
            .justify_center()
            .bg(self.colors.accent.rgba8(0xD9)) // 85% opacity
            .rounded(px(4.))
            .child(
                svg()
                    .external_path(utils::get_logo_path())
                    .size(px(13.))
                    .text_color(rgb(0x000000)), // Black logo inside yellow
            )
    }

    /// Render a footer button with label and shortcut
    fn render_button(
        &self,
        id: &'static str,
        label: String,
        shortcut: String,
        hover_bg: u32,
        on_click: Option<Rc<FooterClickCallback>>,
    ) -> impl IntoElement {
        let colors = self.colors;
        let mut btn = div()
            .id(id)
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.))
            .px(px(8.))
            .py(px(4.))
            .rounded(px(4.))
            .cursor_pointer()
            .hover(move |s| s.bg(rgba(hover_bg)));

        if let Some(callback) = on_click {
            btn = btn.on_click(move |event, window, cx| {
                callback(event, window, cx);
            });
        }

        btn.child(
            div()
                .text_sm()
                .text_color(colors.accent.to_rgb())
                .child(label),
        )
        .child(
            div()
                .text_sm()
                .text_color(colors.text_muted.to_rgb())
                .child(shortcut),
        )
    }

    /// Render the vertical divider between buttons
    fn render_divider(&self) -> impl IntoElement {
        div()
            .w(px(1.))
            .h(px(16.))
            .mx(px(4.))
            .bg(self.colors.border.rgba8(0x40)) // 25% opacity
    }
}

impl RenderOnce for PromptFooter {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let colors = self.colors;
        let hover_bg = (colors.accent << 8) | 0x26; // 15% opacity for hover

        // Build the right-side buttons container
        let mut buttons = hstack().gap(px(4.));

        // Primary button
        buttons = buttons.child(self.render_button(
            "footer-primary-button",
            self.config.primary_label.clone(),
            self.config.primary_shortcut.clone(),
            hover_bg,
            self.on_primary_click.clone(),
        ));

        // Divider + Secondary button (if enabled)
        if self.config.show_secondary {
            buttons = buttons.child(self.render_divider());
            buttons = buttons.child(self.render_button(
                "footer-secondary-button",
                self.config.secondary_label.clone(),
                self.config.secondary_shortcut.clone(),
                hover_bg,
                self.on_secondary_click.clone(),
            ));
        }

        // Main footer container (40px height)
        let mut footer = div()
            .w_full()
            .h(px(40.))
            .px(px(12.))
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .border_t_1()
            .border_color(colors.border.rgba8(0x30)); // Top border with 19% opacity

        // Left: Logo (if enabled)
        if self.config.show_logo {
            footer = footer.child(self.render_logo());
        } else {
            // Spacer for alignment when logo is hidden
            footer = footer.child(div());
        }

        // Right: Buttons
        footer.child(buttons)
    }
}

// Note: Tests omitted for this module due to GPUI macro recursion limit issues.
// The PromptFooter component is integration-tested via the main application's
// prompt rendering in main.rs.
//
// Verified traits:
// - PromptFooterColors: Copy, Clone, Debug, Default
// - PromptFooterConfig: Clone, Debug, Default + builder pattern
// - PromptFooter: builder pattern with .on_primary_click(), .on_secondary_click()
