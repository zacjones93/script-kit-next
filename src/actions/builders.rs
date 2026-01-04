//! Action builders
//!
//! Factory functions for creating context-specific action lists.

use super::types::{Action, ActionCategory, ScriptInfo};
use crate::prompts::PathInfo;

/// Get actions specific to a file/folder path
pub fn get_path_context_actions(path_info: &PathInfo) -> Vec<Action> {
    let mut actions = vec![
        Action::new(
            "copy_path",
            "Copy Path",
            Some("Copy the full path to clipboard".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⇧C"),
        Action::new(
            "open_in_finder",
            "Open in Finder",
            Some("Reveal in Finder".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⇧F"),
        Action::new(
            "open_in_editor",
            "Open in Editor",
            Some("Open in $EDITOR".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘E"),
        Action::new(
            "open_in_terminal",
            "Open in Terminal",
            Some("Open terminal at this location".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘T"),
        Action::new(
            "copy_filename",
            "Copy Filename",
            Some("Copy just the filename".to_string()),
            ActionCategory::ScriptContext,
        ),
        Action::new(
            "move_to_trash",
            "Move to Trash",
            Some(format!(
                "Delete {}",
                if path_info.is_dir { "folder" } else { "file" }
            )),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⌫"),
    ];

    // Add directory-specific action for navigating into
    if path_info.is_dir {
        actions.insert(
            0,
            Action::new(
                "open_directory",
                format!("Open \"{}\"", path_info.name),
                Some("Navigate into this directory".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("↵"),
        );
    } else {
        actions.insert(
            0,
            Action::new(
                "select_file",
                format!("Select \"{}\"", path_info.name),
                Some("Submit this file".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("↵"),
        );
    }

    actions
}

/// Get actions specific to the focused script
pub fn get_script_context_actions(script: &ScriptInfo) -> Vec<Action> {
    vec![
        Action::new(
            "run_script",
            format!("Run \"{}\"", script.name),
            Some("Execute this script".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("↵"),
        Action::new(
            "edit_script",
            "Edit Script",
            Some("Open in $EDITOR".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘E"),
        Action::new(
            "view_logs",
            "View Logs",
            Some("Show script execution logs".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘L"),
        Action::new(
            "reveal_in_finder",
            "Reveal in Finder",
            Some("Show script file in Finder".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⇧F"),
        Action::new(
            "copy_path",
            "Copy Path",
            Some("Copy script path to clipboard".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⇧C"),
    ]
}

/// Predefined global actions
pub fn get_global_actions() -> Vec<Action> {
    vec![
        Action::new(
            "create_script",
            "Create New Script",
            Some("Create a new TypeScript script".to_string()),
            ActionCategory::ScriptOps,
        )
        .with_shortcut("⌘N"),
        Action::new(
            "reload_scripts",
            "Reload Scripts",
            Some("Refresh the scripts list".to_string()),
            ActionCategory::GlobalOps,
        )
        .with_shortcut("⌘R"),
        Action::new(
            "settings",
            "Settings",
            Some("Configure preferences".to_string()),
            ActionCategory::GlobalOps,
        )
        .with_shortcut("⌘,"),
        Action::new(
            "quit",
            "Quit Script Kit",
            Some("Exit the application".to_string()),
            ActionCategory::GlobalOps,
        )
        .with_shortcut("⌘Q"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_script_context_actions() {
        let script = ScriptInfo::new("my-script", "/path/to/my-script.ts");
        let actions = get_script_context_actions(&script);

        assert!(!actions.is_empty());
        assert!(actions.iter().any(|a| a.id == "edit_script"));
        assert!(actions.iter().any(|a| a.id == "view_logs"));
        assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
        assert!(actions.iter().any(|a| a.id == "copy_path"));
        assert!(actions.iter().any(|a| a.id == "run_script"));
    }

    #[test]
    fn test_get_global_actions() {
        let actions = get_global_actions();

        assert!(!actions.is_empty());
        assert!(actions.iter().any(|a| a.id == "create_script"));
        assert!(actions.iter().any(|a| a.id == "reload_scripts"));
        assert!(actions.iter().any(|a| a.id == "settings"));
        assert!(actions.iter().any(|a| a.id == "quit"));
    }

    #[test]
    fn test_built_in_actions_have_no_has_action() {
        // All built-in actions should have has_action=false
        let script = ScriptInfo::new("test-script", "/path/to/test.ts");
        let script_actions = get_script_context_actions(&script);
        let global_actions = get_global_actions();

        for action in script_actions.iter() {
            assert!(
                !action.has_action,
                "Built-in action '{}' should have has_action=false",
                action.id
            );
        }

        for action in global_actions.iter() {
            assert!(
                !action.has_action,
                "Built-in action '{}' should have has_action=false",
                action.id
            );
        }
    }
}
