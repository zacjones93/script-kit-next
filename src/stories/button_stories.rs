//! Button component stories for the storybook

use gpui::*;

use crate::components::{Button, ButtonColors, ButtonVariant};
use crate::storybook::{
    code_block, story_container, story_divider, story_item, story_section, Story, StoryVariant,
};
use crate::theme::Theme;

/// Story showcasing the Button component
pub struct ButtonStory;

impl Story for ButtonStory {
    fn id(&self) -> &'static str {
        "button"
    }

    fn name(&self) -> &'static str {
        "Button"
    }

    fn category(&self) -> &'static str {
        "Components"
    }

    fn render(&self) -> AnyElement {
        let theme = Theme::default();
        let colors = ButtonColors::from_theme(&theme);

        story_container()
            .child(
                story_section("Button Variants")
                    .child(story_item(
                        "Primary",
                        Button::new("Primary Button", colors).variant(ButtonVariant::Primary),
                    ))
                    .child(story_item(
                        "Ghost",
                        Button::new("Ghost Button", colors).variant(ButtonVariant::Ghost),
                    ))
                    .child(story_item(
                        "Icon",
                        Button::new("", colors)
                            .variant(ButtonVariant::Icon)
                            .label("▶"),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("With Shortcuts")
                    .child(story_item(
                        "Enter",
                        Button::new("Submit", colors)
                            .variant(ButtonVariant::Primary)
                            .shortcut("↵"),
                    ))
                    .child(story_item(
                        "Escape",
                        Button::new("Cancel", colors)
                            .variant(ButtonVariant::Ghost)
                            .shortcut("⎋"),
                    ))
                    .child(story_item(
                        "Cmd+S",
                        Button::new("Save", colors)
                            .variant(ButtonVariant::Primary)
                            .shortcut("⌘S"),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("States")
                    .child(story_item(
                        "Normal",
                        Button::new("Normal", colors).variant(ButtonVariant::Primary),
                    ))
                    .child(story_item(
                        "Disabled",
                        Button::new("Disabled", colors)
                            .variant(ButtonVariant::Primary)
                            .disabled(true),
                    )),
            )
            .child(story_divider())
            .child(story_section("Usage").child(code_block(
                r#"
use crate::components::{Button, ButtonColors, ButtonVariant};

let colors = ButtonColors::from_theme(&theme);

Button::new("Click me", colors)
    .variant(ButtonVariant::Primary)
    .shortcut("↵")
    .on_click(Box::new(|_, _, _| {
        println!("Clicked!");
    }))
"#,
            )))
            .into_any_element()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![
            StoryVariant {
                name: "primary".into(),
                description: Some("Primary filled button".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "ghost".into(),
                description: Some("Ghost text-only button".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "icon".into(),
                description: Some("Compact icon button".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "disabled".into(),
                description: Some("Disabled state".into()),
                ..Default::default()
            },
        ]
    }
}
