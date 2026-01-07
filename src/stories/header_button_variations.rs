//! Header Button Variations
//!
//! Explores different button orders, hover states, and whether to include Run button.
//!
//! Variations:
//! 1-4: Different button orders (with Run)
//! 5-8: Different button orders (without Run)
//! 9-12: Different hover intensities

use gpui::*;

use crate::components::PromptHeaderColors;
use crate::storybook::{story_container, story_divider, story_section, Story, StoryVariant};
use crate::theme::Theme;
use crate::ui_foundation::HexColorExt;
use crate::utils;

pub struct HeaderButtonVariationsStory;

impl Story for HeaderButtonVariationsStory {
    fn id(&self) -> &'static str {
        "header-button-variations"
    }

    fn name(&self) -> &'static str {
        "Header Button Variations"
    }

    fn category(&self) -> &'static str {
        "Layouts"
    }

    fn render(&self) -> AnyElement {
        let theme = Theme::default();
        let colors = PromptHeaderColors::from_theme(&theme);

        story_container()
            .child(
                story_section("Button Order - With Run (1-4)")
                    .child(variation_item(
                        "1. Ask AI | Run | Actions (current)",
                        render_order_1(colors),
                    ))
                    .child(variation_item(
                        "2. Run | Ask AI | Actions",
                        render_order_2(colors),
                    ))
                    .child(variation_item(
                        "3. Actions | Run | Ask AI",
                        render_order_3(colors),
                    ))
                    .child(variation_item(
                        "4. Ask AI | Actions | Run",
                        render_order_4(colors),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("Button Order - Without Run (5-8)")
                    .child(variation_item(
                        "5. Ask AI | Actions",
                        render_no_run_1(colors),
                    ))
                    .child(variation_item(
                        "6. Actions | Ask AI",
                        render_no_run_2(colors),
                    ))
                    .child(variation_item(
                        "7. Ask AI | Actions (larger gap)",
                        render_no_run_3(colors),
                    ))
                    .child(variation_item(
                        "8. Ask AI | Actions (with divider)",
                        render_no_run_4(colors),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("Hover Intensity (9-12) - hover to see")
                    .child(variation_item(
                        "9. Hover 10% opacity",
                        render_hover_intensity(colors, 0.10),
                    ))
                    .child(variation_item(
                        "10. Hover 15% opacity (current)",
                        render_hover_intensity(colors, 0.15),
                    ))
                    .child(variation_item(
                        "11. Hover 20% opacity",
                        render_hover_intensity(colors, 0.20),
                    ))
                    .child(variation_item(
                        "12. Hover 25% opacity",
                        render_hover_intensity(colors, 0.25),
                    )),
            )
            .into_any()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![
            StoryVariant {
                name: "order-1".into(),
                description: Some("Ask AI | Run | Actions".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "order-2".into(),
                description: Some("Run | Ask AI | Actions".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "order-3".into(),
                description: Some("Actions | Run | Ask AI".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "order-4".into(),
                description: Some("Ask AI | Actions | Run".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "no-run-1".into(),
                description: Some("Ask AI | Actions".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "no-run-2".into(),
                description: Some("Actions | Ask AI".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "no-run-3".into(),
                description: Some("Larger gap".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "no-run-4".into(),
                description: Some("With divider".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "hover-10".into(),
                description: Some("10% hover".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "hover-15".into(),
                description: Some("15% hover".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "hover-20".into(),
                description: Some("20% hover".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "hover-25".into(),
                description: Some("25% hover".into()),
                ..Default::default()
            },
        ]
    }
}

// =============================================================================
// HELPER COMPONENTS
// =============================================================================

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

fn script_kit_label(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .text_base()
        .font_weight(FontWeight::MEDIUM)
        .text_color(colors.text_primary.to_rgb())
        .child("Script Kit")
}

fn logo_box() -> impl IntoElement {
    div()
        .w(px(19.))
        .h(px(19.))
        .flex()
        .items_center()
        .justify_center()
        .bg(rgba(0xFFD60AD9)) // 85% opacity yellow
        .rounded(px(4.))
        .child(
            svg()
                .external_path(utils::get_logo_path())
                .size(px(12.))
                .text_color(rgb(0x000000)),
        )
}

/// Ask AI button with hover
fn ask_ai_button(colors: PromptHeaderColors, hover_opacity: f32) -> Stateful<Div> {
    let hover_alpha = (hover_opacity * 255.0) as u32;
    let hover_bg = (colors.accent << 8) | hover_alpha;
    let tab_bg = (colors.search_box_bg << 8) | 0x4D; // 30% opacity

    div()
        .id("ask-ai-btn")
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.))
        .px(px(6.))
        .py(px(4.))
        .rounded(px(4.))
        .cursor_pointer()
        .hover(move |s| s.bg(rgba(hover_bg)))
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

/// Run button with hover
fn run_button(colors: PromptHeaderColors, hover_opacity: f32) -> Stateful<Div> {
    let hover_alpha = (hover_opacity * 255.0) as u32;
    let hover_bg = (colors.accent << 8) | hover_alpha;

    div()
        .id("run-btn")
        .flex()
        .flex_row()
        .items_center()
        .gap(px(4.))
        .px(px(6.))
        .py(px(4.))
        .rounded(px(4.))
        .cursor_pointer()
        .hover(move |s| s.bg(rgba(hover_bg)))
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

/// Actions button with hover
fn actions_button(colors: PromptHeaderColors, hover_opacity: f32) -> Stateful<Div> {
    let hover_alpha = (hover_opacity * 255.0) as u32;
    let hover_bg = (colors.accent << 8) | hover_alpha;

    div()
        .id("actions-btn")
        .flex()
        .flex_row()
        .items_center()
        .gap(px(4.))
        .px(px(6.))
        .py(px(4.))
        .rounded(px(4.))
        .cursor_pointer()
        .hover(move |s| s.bg(rgba(hover_bg)))
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

/// Vertical divider
fn divider(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .w(px(1.))
        .h(px(16.))
        .bg(rgba((colors.border << 8) | 0x40))
}

// =============================================================================
// ORDER VARIATIONS - WITH RUN
// =============================================================================

/// 1. Ask AI | Run | Actions (current)
fn render_order_1(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors, 0.15))
        .child(div().w(px(4.)))
        .child(run_button(colors, 0.15))
        .child(div().w(px(4.)))
        .child(actions_button(colors, 0.15))
        .child(div().w(px(8.)))
        .child(logo_box())
}

/// 2. Run | Ask AI | Actions
fn render_order_2(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(run_button(colors, 0.15))
        .child(div().w(px(4.)))
        .child(ask_ai_button(colors, 0.15))
        .child(div().w(px(4.)))
        .child(actions_button(colors, 0.15))
        .child(div().w(px(8.)))
        .child(logo_box())
}

/// 3. Actions | Run | Ask AI
fn render_order_3(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(actions_button(colors, 0.15))
        .child(div().w(px(4.)))
        .child(run_button(colors, 0.15))
        .child(div().w(px(4.)))
        .child(ask_ai_button(colors, 0.15))
        .child(div().w(px(8.)))
        .child(logo_box())
}

/// 4. Ask AI | Actions | Run
fn render_order_4(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors, 0.15))
        .child(div().w(px(4.)))
        .child(actions_button(colors, 0.15))
        .child(div().w(px(4.)))
        .child(run_button(colors, 0.15))
        .child(div().w(px(8.)))
        .child(logo_box())
}

// =============================================================================
// ORDER VARIATIONS - WITHOUT RUN
// =============================================================================

/// 5. Ask AI | Actions (no Run)
fn render_no_run_1(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors, 0.15))
        .child(div().w(px(4.)))
        .child(actions_button(colors, 0.15))
        .child(div().w(px(8.)))
        .child(logo_box())
}

/// 6. Actions | Ask AI (no Run)
fn render_no_run_2(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(actions_button(colors, 0.15))
        .child(div().w(px(4.)))
        .child(ask_ai_button(colors, 0.15))
        .child(div().w(px(8.)))
        .child(logo_box())
}

/// 7. Ask AI | Actions (larger gap)
fn render_no_run_3(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors, 0.15))
        .child(div().w(px(12.)))
        .child(actions_button(colors, 0.15))
        .child(div().w(px(8.)))
        .child(logo_box())
}

/// 8. Ask AI | Actions (with divider)
fn render_no_run_4(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors, 0.15))
        .child(div().w(px(8.)))
        .child(divider(colors))
        .child(div().w(px(8.)))
        .child(actions_button(colors, 0.15))
        .child(div().w(px(8.)))
        .child(logo_box())
}

// =============================================================================
// HOVER INTENSITY VARIATIONS
// =============================================================================

fn render_hover_intensity(colors: PromptHeaderColors, hover_opacity: f32) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors, hover_opacity))
        .child(div().w(px(4.)))
        .child(run_button(colors, hover_opacity))
        .child(div().w(px(4.)))
        .child(actions_button(colors, hover_opacity))
        .child(div().w(px(8.)))
        .child(logo_box())
}
