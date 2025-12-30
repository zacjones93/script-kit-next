//! Story registry - manual registration for compile-time story collection
//!
//! Instead of using inventory (which has const-fn requirements in newer Rust),
//! we use a manual registration approach where stories are collected in the
//! stories module and returned via get_all_stories().

use super::Story;

/// Entry for a registered story
pub struct StoryEntry {
    pub story: Box<dyn Story>,
}

impl StoryEntry {
    pub fn new(story: Box<dyn Story>) -> Self {
        Self { story }
    }
}

/// Get all registered stories
/// This function is implemented in the stories module
pub fn all_stories() -> impl Iterator<Item = &'static StoryEntry> {
    crate::stories::get_all_stories().iter()
}

/// Find stories by category
pub fn stories_by_category(category: &str) -> Vec<&'static StoryEntry> {
    all_stories()
        .filter(|e| e.story.category() == category)
        .collect()
}

/// Get unique categories
pub fn all_categories() -> Vec<&'static str> {
    let mut categories: Vec<_> = all_stories().map(|e| e.story.category()).collect();
    categories.sort();
    categories.dedup();
    categories
}
