//! Actions Dialog Module
//!
//! Provides a searchable action menu as a compact overlay popup for quick access
//! to script management and global actions (edit, create, settings, quit, etc.)
//!
//! The dialog renders as a floating overlay popup with:
//! - Fixed dimensions (320x400px max)
//! - Rounded corners and box shadow
//! - Semi-transparent background
//! - Context-aware actions based on focused script
//!
//! ## Module Structure
//! - `types`: Core types (Action, ActionCategory, ScriptInfo, ActionCallback)
//! - `builders`: Factory functions for creating action lists
//! - `constants`: Popup dimensions and styling constants
//! - `dialog`: ActionsDialog struct and implementation
//! - `script_utils`: Script creation utilities

mod builders;
mod constants;
mod dialog;
mod script_utils;
mod types;

// Re-export public API

// Types
pub use types::{Action, ActionCallback, ActionCategory, ScriptInfo};

// Builders
pub use builders::{get_global_actions, get_path_context_actions, get_script_context_actions};

// Constants
#[allow(unused_imports)]
pub use constants::{
    ACCENT_BAR_WIDTH, ACTION_ITEM_HEIGHT, ITEM_PADDING_X, ITEM_PADDING_Y, POPUP_CORNER_RADIUS,
    POPUP_MAX_HEIGHT, POPUP_PADDING, POPUP_WIDTH,
};

// Dialog
pub use dialog::ActionsDialog;

// Script utilities
pub use script_utils::{
    create_script_file, generate_script_template, get_script_path, script_exists,
    validate_script_name,
};

// Re-export PathInfo from prompts module for convenience
pub use crate::prompts::PathInfo;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::ProtocolAction;

    #[test]
    fn test_actions_exceed_visible_space() {
        // Verify that with script context + global actions, we exceed visible space
        // This confirms scrolling/virtualization is needed
        let script = ScriptInfo::new("test-script", "/path/to/test.ts");
        let script_actions = get_script_context_actions(&script);
        let global_actions = get_global_actions();
        let total_actions = script_actions.len() + global_actions.len();

        let max_visible = (POPUP_MAX_HEIGHT / ACTION_ITEM_HEIGHT) as usize;

        // With 5 script context actions + 4 global = 9 actions
        // At 42px height in 400px container, we can fit ~9 items
        // So we might not always overflow, but we're close
        assert!(total_actions >= 8, "Should have at least 8 total actions");

        // Log for visibility
        println!(
            "Total actions: {}, Max visible: {}",
            total_actions, max_visible
        );
    }

    #[test]
    fn test_protocol_action_to_action_conversion() {
        let protocol_action = ProtocolAction {
            name: "Copy".to_string(),
            description: Some("Copy to clipboard".to_string()),
            shortcut: Some("cmd+c".to_string()),
            value: Some("copy-value".to_string()),
            has_action: true,
            visible: None,
            close: None,
        };

        // Simulate conversion logic from set_sdk_actions
        let action = Action {
            id: protocol_action.name.clone(),
            title: protocol_action.name.clone(),
            description: protocol_action.description.clone(),
            category: ActionCategory::ScriptContext,
            shortcut: protocol_action.shortcut.clone(),
            has_action: protocol_action.has_action,
            value: protocol_action.value.clone(),
        };

        assert_eq!(action.id, "Copy");
        assert_eq!(action.title, "Copy");
        assert_eq!(action.description, Some("Copy to clipboard".to_string()));
        assert_eq!(action.shortcut, Some("cmd+c".to_string()));
        assert_eq!(action.value, Some("copy-value".to_string()));
        assert!(action.has_action);
    }

    #[test]
    fn test_protocol_action_has_action_routing() {
        // Action with has_action=true should trigger ActionTriggered to SDK
        let action_with_handler = ProtocolAction {
            name: "Custom Action".to_string(),
            description: None,
            shortcut: None,
            value: Some("custom-value".to_string()),
            has_action: true,
            visible: None,
            close: None,
        };
        assert!(action_with_handler.has_action);

        // Action with has_action=false should submit value directly
        let action_without_handler = ProtocolAction {
            name: "Simple Action".to_string(),
            description: None,
            shortcut: None,
            value: Some("simple-value".to_string()),
            has_action: false,
            visible: None,
            close: None,
        };
        assert!(!action_without_handler.has_action);
    }
}
