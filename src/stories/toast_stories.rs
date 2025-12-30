//! Toast component stories for the storybook

use gpui::*;

use crate::components::{Toast, ToastAction, ToastColors, ToastVariant};
use crate::storybook::{
    code_block, story_container, story_divider, story_item, story_section, Story, StoryVariant,
};
use crate::theme::Theme;

/// Story showcasing the Toast component
pub struct ToastStory;

impl Story for ToastStory {
    fn id(&self) -> &'static str {
        "toast"
    }

    fn name(&self) -> &'static str {
        "Toast"
    }

    fn category(&self) -> &'static str {
        "Components"
    }

    fn render(&self) -> AnyElement {
        let theme = Theme::default();

        story_container()
            .child(
                story_section("Toast Variants")
                    .child(story_item(
                        "Success",
                        Toast::new(
                            "Operation completed successfully",
                            ToastColors::from_theme(&theme, ToastVariant::Success),
                        )
                        .variant(ToastVariant::Success),
                    ))
                    .child(story_item(
                        "Warning",
                        Toast::new(
                            "This action may have side effects",
                            ToastColors::from_theme(&theme, ToastVariant::Warning),
                        )
                        .variant(ToastVariant::Warning),
                    ))
                    .child(story_item(
                        "Error",
                        Toast::new(
                            "An error occurred while processing",
                            ToastColors::from_theme(&theme, ToastVariant::Error),
                        )
                        .variant(ToastVariant::Error),
                    ))
                    .child(story_item(
                        "Info",
                        Toast::new(
                            "New updates are available",
                            ToastColors::from_theme(&theme, ToastVariant::Info),
                        )
                        .variant(ToastVariant::Info),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("With Details")
                    .child(story_item(
                        "Expandable",
                        Toast::new(
                            "Script failed to execute",
                            ToastColors::from_theme(&theme, ToastVariant::Error),
                        )
                        .variant(ToastVariant::Error)
                        .details("Error: Cannot find module 'lodash'\n  at require (internal/modules/cjs/loader.js:999:32)"),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("With Actions")
                    .child(story_item(
                        "Retry",
                        Toast::new(
                            "Network request failed",
                            ToastColors::from_theme(&theme, ToastVariant::Error),
                        )
                        .variant(ToastVariant::Error)
                        .action(ToastAction::new("Retry", Box::new(|_, _, _| {
                            // No-op for story display
                        }))),
                    ))
                    .child(story_item(
                        "Undo",
                        Toast::new(
                            "Item deleted",
                            ToastColors::from_theme(&theme, ToastVariant::Info),
                        )
                        .variant(ToastVariant::Info)
                        .action(ToastAction::new("Undo", Box::new(|_, _, _| {
                            // No-op for story display
                        }))),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("Dismissible")
                    .child(story_item(
                        "With X button",
                        Toast::new(
                            "You can dismiss this notification",
                            ToastColors::from_theme(&theme, ToastVariant::Info),
                        )
                        .variant(ToastVariant::Info)
                        .dismissible(true),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("Usage")
                    .child(code_block(
                        r#"
use crate::components::{Toast, ToastColors, ToastVariant, ToastAction};

let colors = ToastColors::from_theme(&theme, ToastVariant::Error);

Toast::new("Error occurred", colors)
    .variant(ToastVariant::Error)
    .details("Stack trace here...")
    .action(ToastAction::new("Retry", Box::new(|_, _, _| { ... })))
    .dismissible(true)
"#,
                    )),
            )
            .into_any_element()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![
            StoryVariant {
                name: "success".into(),
                description: Some("Success notification".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "warning".into(),
                description: Some("Warning notification".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "error".into(),
                description: Some("Error notification".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "info".into(),
                description: Some("Info notification".into()),
                ..Default::default()
            },
        ]
    }
}
