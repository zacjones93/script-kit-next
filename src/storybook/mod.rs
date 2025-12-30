//! Storybook - Component preview system for script-kit-gpui
//!
//! This module provides a component preview system for GPUI components.
//!
//! # Components
//!
//! - [`Story`] - Trait for defining previewable stories
//! - [`StoryBrowser`] - Main UI for browsing stories
//! - [`story_container`], [`story_section`], etc. - Layout helpers
//!
//! # Usage
//!
//! ```ignore
//! // Define a story
//! use crate::storybook::{Story, StoryVariant, story_container, story_section, story_item};
//!
//! pub struct MyComponentStory;
//!
//! impl Story for MyComponentStory {
//!     fn id(&self) -> &'static str { "my-component" }
//!     fn name(&self) -> &'static str { "My Component" }
//!     fn category(&self) -> &'static str { "Components" }
//!     fn render(&self) -> AnyElement {
//!         story_container()
//!             .child(story_section("Variants")
//!                 .child(story_item("Default", MyComponent::new())))
//!             .into_any_element()
//!     }
//! }
//!
//! // Register it in stories/mod.rs get_all_stories()
//! ```

mod browser;
mod layout;
mod registry;
mod story;

pub use browser::StoryBrowser;
pub use layout::{code_block, story_container, story_divider, story_item, story_section};
pub use registry::{all_categories, all_stories, stories_by_category, StoryEntry};
pub use story::{Story, StoryVariant};
