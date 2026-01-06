//! Header Tab & Spacing Variations
//!
//! 10 variations exploring:
//! - Tab badge opacity (20%, 30%, 40%, 50%)
//! - Logo left margin (8px, 10px, 12px, 16px)
//! - Combined variations
//!
//! Goal: Find the optimal Tab opacity and logo spacing

use gpui::*;

use crate::components::PromptHeaderColors;
use crate::storybook::{story_container, story_divider, story_section, Story, StoryVariant};
use crate::theme::Theme;
use crate::ui_foundation::HexColorExt;
use crate::utils;

/// Story showcasing Tab opacity and logo spacing variations
pub struct HeaderTabSpacingVariationsStory;

impl Story for HeaderTabSpacingVariationsStory {
    fn id(&self) -> &'static str {
        "header-tab-spacing-variations"
    }

    fn name(&self) -> &'static str {
        "Header Tab & Spacing"
    }

    fn category(&self) -> &'static str {
        "Layouts"
    }

    fn render(&self) -> AnyElement {
        let theme = Theme::default();
        let colors = PromptHeaderColors::from_theme(&theme);

        story_container()
            .child(
                story_section("Tab Badge Opacity (1-4)")
                    .child(variation_item(
                        "1. Tab 20% opacity",
                        render_tab_opacity(colors, 0.20, 16.0),
                    ))
                    .child(variation_item(
                        "2. Tab 30% opacity",
                        render_tab_opacity(colors, 0.30, 16.0),
                    ))
                    .child(variation_item(
                        "3. Tab 40% opacity (current)",
                        render_tab_opacity(colors, 0.40, 16.0),
                    ))
                    .child(variation_item(
                        "4. Tab 50% opacity",
                        render_tab_opacity(colors, 0.50, 16.0),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("Logo Left Margin (5-8)")
                    .child(variation_item(
                        "5. Logo 8px margin",
                        render_tab_opacity(colors, 0.30, 8.0),
                    ))
                    .child(variation_item(
                        "6. Logo 10px margin",
                        render_tab_opacity(colors, 0.30, 10.0),
                    ))
                    .child(variation_item(
                        "7. Logo 12px margin",
                        render_tab_opacity(colors, 0.30, 12.0),
                    ))
                    .child(variation_item(
                        "8. Logo 16px margin (current)",
                        render_tab_opacity(colors, 0.30, 16.0),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("Best Combinations (9-10)")
                    .child(variation_item(
                        "9. Tab 25% + Logo 10px",
                        render_tab_opacity(colors, 0.25, 10.0),
                    ))
                    .child(variation_item(
                        "10. Tab 30% + Logo 8px",
                        render_tab_opacity(colors, 0.30, 8.0),
                    )),
            )
            .into_any()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![
            StoryVariant {
                name: "tab-20".into(),
                description: Some("Tab 20% opacity".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "tab-30".into(),
                description: Some("Tab 30% opacity".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "tab-40".into(),
                description: Some("Tab 40% opacity".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "tab-50".into(),
                description: Some("Tab 50% opacity".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "logo-8px".into(),
                description: Some("Logo 8px margin".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "logo-10px".into(),
                description: Some("Logo 10px margin".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "logo-12px".into(),
                description: Some("Logo 12px margin".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "logo-16px".into(),
                description: Some("Logo 16px margin".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "best-1".into(),
                description: Some("Tab 25% + Logo 10px".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "best-2".into(),
                description: Some("Tab 30% + Logo 8px".into()),
                ..Default::default()
            },
        ]
    }
}

// =============================================================================
// HELPER COMPONENTS
// =============================================================================

/// Wrapper for each variation
fn variation_item(label: &str, content: impl IntoElement) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .text_xs()
                .text_color(rgb(0x888888))
                .child(label.to_string()),
        )
        .child(content)
}

/// Header container (dark background for visibility)
fn header_container(colors: PromptHeaderColors) -> Div {
    div()
        .w_full()
        .px(px(16.))
        .py(px(8.))
        .flex()
        .flex_row()
        .items_center()
        .bg(colors.background.to_rgb())
        .rounded(px(8.))
}

/// Script Kit label
fn script_kit_label(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .text_base()
        .font_weight(FontWeight::MEDIUM)
        .text_color(colors.text_primary.to_rgb())
        .child("Script Kit")
}

/// Logo box (21px container, 13px SVG, 4px radius, 85% opacity yellow)
fn logo_box() -> impl IntoElement {
    div()
        .w(px(21.))
        .h(px(21.))
        .flex()
        .items_center()
        .justify_center()
        .bg(rgba(0xFFD60AD9)) // 85% opacity yellow
        .rounded(px(4.))
        .child(
            svg()
                .external_path(utils::get_logo_path())
                .size(px(13.))
                .text_color(rgb(0x000000)),
        )
}

/// Ask AI hint with configurable Tab opacity
fn ask_ai_hint(colors: PromptHeaderColors, tab_opacity: f32) -> impl IntoElement {
    // Convert opacity to alpha hex value (0-255)
    let alpha = (tab_opacity * 255.0) as u32;
    let tab_bg = (colors.search_box_bg << 8) | alpha;

    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_1()
        .child(
            div()
                .text_sm()
                .text_color(colors.accent.to_rgb())
                .child("Ask AI"),
        )
        .child(
            div()
                .px(px(6.))
                .py(px(2.))
                .bg(rgba(tab_bg))
                .rounded(px(4.))
                .text_xs()
                .text_color(colors.text_muted.to_rgb())
                .child("Tab"),
        )
}

/// Run button
fn run_button(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_1()
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .text_color(colors.accent.to_rgb())
                .child("Run"),
        )
        .child(
            div()
                .text_sm()
                .text_color(colors.text_muted.to_rgb())
                .child("↵"),
        )
}

/// Actions button
fn actions_button(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_1()
        .child(
            div()
                .text_sm()
                .text_color(colors.accent.to_rgb())
                .child("Actions"),
        )
        .child(
            div()
                .text_xs()
                .text_color(colors.text_dimmed.to_rgb())
                .child("⌘K"),
        )
}

// =============================================================================
// VARIATION RENDERER
// =============================================================================

/// Render header with configurable Tab opacity and logo margin
fn render_tab_opacity(
    colors: PromptHeaderColors,
    tab_opacity: f32,
    logo_margin: f32,
) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors, tab_opacity))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
        .child(div().w(px(logo_margin)))
        .child(logo_box())
}
