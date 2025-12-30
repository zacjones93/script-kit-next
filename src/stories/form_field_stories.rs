//! Form field component stories for the storybook

use gpui::*;

use crate::storybook::{
    code_block, story_container, story_divider, story_section, Story, StoryVariant,
};

/// Story showcasing the Form Field components
pub struct FormFieldStory;

impl Story for FormFieldStory {
    fn id(&self) -> &'static str {
        "form-fields"
    }

    fn name(&self) -> &'static str {
        "Form Fields"
    }

    fn category(&self) -> &'static str {
        "Components"
    }

    fn render(&self) -> AnyElement {
        // Note: Form fields require Context<Self> to render properly
        // This story shows placeholder representations

        story_container()
            .child(
                story_section("Text Input Fields")
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0x666666))
                                    .child("Text Field"),
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_2()
                                    .bg(rgb(0x2d2d2d))
                                    .rounded_md()
                                    .border_1()
                                    .border_color(rgb(0x464647))
                                    .text_color(rgb(0x888888))
                                    .child("Enter text..."),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .mt_4()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0x666666))
                                    .child("Password Field"),
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_2()
                                    .bg(rgb(0x2d2d2d))
                                    .rounded_md()
                                    .border_1()
                                    .border_color(rgb(0x464647))
                                    .text_color(rgb(0x888888))
                                    .child("********"),
                            ),
                    ),
            )
            .child(story_divider())
            .child(
                story_section("Text Area").child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(0x666666))
                                .child("Multi-line Input"),
                        )
                        .child(
                            div()
                                .px_3()
                                .py_2()
                                .h(px(100.))
                                .bg(rgb(0x2d2d2d))
                                .rounded_md()
                                .border_1()
                                .border_color(rgb(0x464647))
                                .text_color(rgb(0x888888))
                                .child("Enter multi-line text..."),
                        ),
                ),
            )
            .child(story_divider())
            .child(
                story_section("Checkbox")
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .w(px(16.))
                                    .h(px(16.))
                                    .bg(rgb(0x2d2d2d))
                                    .rounded_sm()
                                    .border_1()
                                    .border_color(rgb(0x464647)),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0xcccccc))
                                    .child("Unchecked option"),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .mt_2()
                            .child(
                                div()
                                    .w(px(16.))
                                    .h(px(16.))
                                    .bg(rgb(0x4a90d9))
                                    .rounded_sm()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_color(rgb(0xffffff))
                                    .text_xs()
                                    .child("✓"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0xcccccc))
                                    .child("Checked option"),
                            ),
                    ),
            )
            .child(story_divider())
            .child(story_section("Usage").child(code_block(
                r#"
use crate::components::{FormTextField, FormTextArea, FormCheckbox, FormFieldColors};
use crate::protocol::Field;

let field = Field::new("username".to_string())
    .with_label("Username".to_string())
    .with_placeholder("Enter username".to_string());

let colors = FormFieldColors::from_theme(&theme);
let text_field = FormTextField::new(field, colors, cx);
"#,
            )))
            .into_any_element()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![
            StoryVariant {
                name: "text".into(),
                description: Some("Text input field".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "password".into(),
                description: Some("Password input field".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "textarea".into(),
                description: Some("Multi-line text area".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "checkbox".into(),
                description: Some("Checkbox with label".into()),
                ..Default::default()
            },
        ]
    }
}

// Story is registered in stories/mod.rs via get_all_stories()
