//! Scrollbar component stories for the storybook

use gpui::*;

use crate::storybook::{
    code_block, story_container, story_divider, story_section, Story, StoryVariant,
};

/// Story showcasing the Scrollbar component
pub struct ScrollbarStory;

impl Story for ScrollbarStory {
    fn id(&self) -> &'static str {
        "scrollbar"
    }

    fn name(&self) -> &'static str {
        "Scrollbar"
    }

    fn category(&self) -> &'static str {
        "Components"
    }

    fn render(&self) -> AnyElement {
        story_container()
            .child(
                story_section("Scrollbar Preview").child(
                    div()
                        .flex()
                        .flex_row()
                        .gap_4()
                        .child(render_scrollbar_demo("At top", 0.0, 0.3))
                        .child(render_scrollbar_demo("Middle", 0.35, 0.3))
                        .child(render_scrollbar_demo("At bottom", 0.7, 0.3)),
                ),
            )
            .child(story_divider())
            .child(
                story_section("Thumb Sizes").child(
                    div()
                        .flex()
                        .flex_row()
                        .gap_4()
                        .child(render_scrollbar_demo("Large thumb", 0.0, 0.6))
                        .child(render_scrollbar_demo("Medium thumb", 0.2, 0.3))
                        .child(render_scrollbar_demo("Small thumb", 0.3, 0.15)),
                ),
            )
            .child(story_divider())
            .child(
                story_section("Scrollable List Demo").child(
                    div()
                        .w(px(300.))
                        .h(px(200.))
                        .bg(rgb(0x252525))
                        .rounded_md()
                        .relative()
                        .overflow_hidden()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .p_2()
                                .children((0..20).map(|i| {
                                    div()
                                        .px_2()
                                        .py_1()
                                        .rounded_sm()
                                        .bg(rgb(0x2d2d2d))
                                        .text_sm()
                                        .text_color(rgb(0xcccccc))
                                        .child(format!("Item {}", i + 1))
                                })),
                        )
                        .child(
                            // Simulated scrollbar
                            div()
                                .absolute()
                                .top_1()
                                .bottom_1()
                                .right_1()
                                .w(px(6.))
                                .bg(rgba(0x00000000))
                                .rounded_full()
                                .child(
                                    div()
                                        .absolute()
                                        .top(px(20.))
                                        .w_full()
                                        .h(px(60.))
                                        .bg(rgba(0xffffff33))
                                        .rounded_full(),
                                ),
                        ),
                ),
            )
            .child(story_divider())
            .child(story_section("Usage").child(code_block(
                r#"
use crate::components::{Scrollbar, ScrollbarColors};

let colors = ScrollbarColors::from_theme(&theme);
Scrollbar::new(scroll_position, total_height, visible_height, colors)
    .render(window, cx)
"#,
            )))
            .into_any_element()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![
            StoryVariant {
                name: "default".into(),
                description: Some("Default scrollbar style".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "dragging".into(),
                description: Some("Scrollbar while dragging".into()),
                ..Default::default()
            },
        ]
    }
}

/// Render a scrollbar demo box
fn render_scrollbar_demo(label: &str, position: f32, size: f32) -> impl IntoElement {
    let track_height = 150.0_f32;
    let thumb_height = (track_height * size).max(24.0);
    let max_travel = track_height - thumb_height;
    let thumb_top = max_travel * position;

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap_2()
        .child(
            div()
                .text_xs()
                .text_color(rgb(0x888888))
                .child(label.to_string()),
        )
        .child(
            div()
                .w(px(40.))
                .h(px(track_height))
                .bg(rgb(0x252525))
                .rounded_md()
                .relative()
                .child(
                    // Track
                    div()
                        .absolute()
                        .top_1()
                        .bottom_1()
                        .right(px(2.))
                        .w(px(6.))
                        .bg(rgba(0x00000000))
                        .rounded_full()
                        .child(
                            // Thumb
                            div()
                                .absolute()
                                .top(px(thumb_top + 4.0))
                                .w_full()
                                .h(px(thumb_height))
                                .bg(rgba(0xffffff44))
                                .rounded_full()
                                .hover(|s| s.bg(rgba(0xffffff66))),
                        ),
                ),
        )
}

// Story is registered in stories/mod.rs via get_all_stories()
