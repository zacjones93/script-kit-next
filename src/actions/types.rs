//! Action types and data structures
//!
//! Core types for the actions system including Action, ActionCategory, and ScriptInfo.

use std::sync::Arc;

/// Callback for action selection
/// Signature: (action_id: String)
pub type ActionCallback = Arc<dyn Fn(String) + Send + Sync>;

/// Information about the currently focused/selected script
/// Used for context-aware actions in the actions dialog
#[derive(Debug, Clone)]
pub struct ScriptInfo {
    /// Display name of the script
    pub name: String,
    /// Full path to the script file
    pub path: String,
}

impl ScriptInfo {
    pub fn new(name: impl Into<String>, path: impl Into<String>) -> Self {
        ScriptInfo {
            name: name.into(),
            path: path.into(),
        }
    }
}

/// Available actions in the actions menu
#[derive(Debug, Clone)]
pub struct Action {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub category: ActionCategory,
    /// Optional keyboard shortcut hint (e.g., "⌘E")
    pub shortcut: Option<String>,
    /// If true, send ActionTriggered to SDK; if false, submit value directly
    /// Built-in actions default to false; SDK actions may set this to true
    pub has_action: bool,
    /// Optional value to submit when action is triggered
    pub value: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActionCategory {
    ScriptContext, // Actions specific to the focused script
    ScriptOps,     // Edit, Create, Delete script operations
    GlobalOps,     // Settings, Quit, etc.
}

impl Action {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        description: Option<String>,
        category: ActionCategory,
    ) -> Self {
        Action {
            id: id.into(),
            title: title.into(),
            description,
            category,
            shortcut: None,
            has_action: false,
            value: None,
        }
    }

    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn with_has_action(mut self, has_action: bool) -> Self {
        self.has_action = has_action;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_info_creation() {
        let script = ScriptInfo::new("test-script", "/path/to/test-script.ts");
        assert_eq!(script.name, "test-script");
        assert_eq!(script.path, "/path/to/test-script.ts");
    }

    #[test]
    fn test_action_with_shortcut() {
        let action =
            Action::new("test", "Test Action", None, ActionCategory::GlobalOps).with_shortcut("⌘T");
        assert_eq!(action.shortcut, Some("⌘T".to_string()));
    }

    #[test]
    fn test_action_with_has_action() {
        let action = Action::new("test", "Test Action", None, ActionCategory::GlobalOps)
            .with_has_action(true);
        assert!(action.has_action);

        let action2 = Action::new("test2", "Test Action 2", None, ActionCategory::GlobalOps);
        assert!(!action2.has_action); // default is false
    }

    #[test]
    fn test_action_with_value() {
        let action = Action::new("test", "Test Action", None, ActionCategory::GlobalOps)
            .with_value("my-value");
        assert_eq!(action.value, Some("my-value".to_string()));

        let action2 = Action::new("test2", "Test Action 2", None, ActionCategory::GlobalOps);
        assert!(action2.value.is_none()); // default is None
    }

    #[test]
    fn test_action_new_defaults() {
        let action = Action::new(
            "id",
            "title",
            Some("desc".to_string()),
            ActionCategory::ScriptContext,
        );
        assert_eq!(action.id, "id");
        assert_eq!(action.title, "title");
        assert_eq!(action.description, Some("desc".to_string()));
        assert_eq!(action.category, ActionCategory::ScriptContext);
        assert!(action.shortcut.is_none());
        assert!(!action.has_action);
        assert!(action.value.is_none());
    }
}
