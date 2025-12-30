use gpui::*;
use std::collections::HashMap;

/// A story renders a component in various states for preview
///
/// Stories are stateless previews of components. They render static elements
/// and don't require app state or window mutations.
pub trait Story: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn category(&self) -> &'static str;
    /// Render the story preview.
    /// Note: Window and App are provided for compatibility but stories should
    /// render stateless elements that don't depend on app state.
    fn render(&self) -> AnyElement;
    fn variants(&self) -> Vec<StoryVariant> {
        vec![StoryVariant::default()]
    }
}

#[derive(Default, Clone)]
pub struct StoryVariant {
    pub name: String,
    pub description: Option<String>,
    pub props: HashMap<String, String>,
}
