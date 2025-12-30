//! MCP Kit Namespace Tools
//!
//! Implements the kit/* namespace MCP tools for Script Kit:
//! - kit/show: Show the Script Kit window
//! - kit/hide: Hide the Script Kit window
//! - kit/state: Get current app state

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Kit tool definitions for MCP tools/list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// Result of a kit tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub content: Vec<ToolContent>,
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

/// Content item in tool result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

/// App state returned by kit/state tool
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppState {
    pub visible: bool,
    pub focused: bool,
    #[serde(rename = "activePrompt")]
    pub active_prompt: Option<String>,
}

/// Returns the tool definitions for kit/* namespace tools
pub fn get_kit_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "kit/show".to_string(),
            description: "Show the Script Kit window".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "kit/hide".to_string(),
            description: "Hide the Script Kit window".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "kit/state".to_string(),
            description: "Get current Script Kit app state".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
    ]
}

/// Check if a tool name is in the kit/* namespace
pub fn is_kit_tool(name: &str) -> bool {
    name.starts_with("kit/")
}

/// Handle a kit/* namespace tool call
/// 
/// Note: This returns a result that the caller should use to actually perform
/// the window operations. The actual show/hide operations require GPUI context
/// which is not available in this module.
pub fn handle_kit_tool_call(name: &str, _arguments: &Value) -> ToolResult {
    match name {
        "kit/show" => ToolResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: "Window show requested".to_string(),
            }],
            is_error: None,
        },
        "kit/hide" => ToolResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: "Window hide requested".to_string(),
            }],
            is_error: None,
        },
        "kit/state" => {
            // Return default state - actual state will be injected by caller
            let state = AppState::default();
            ToolResult {
                content: vec![ToolContent {
                    content_type: "text".to_string(),
                    text: serde_json::to_string(&state).unwrap_or_default(),
                }],
                is_error: None,
            }
        }
        _ => ToolResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: format!("Unknown kit tool: {}", name),
            }],
            is_error: Some(true),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =======================================================
    // TDD Tests - Written FIRST per spec requirements
    // =======================================================

    #[test]
    fn test_kit_show_tool_definition() {
        let tools = get_kit_tool_definitions();
        let show_tool = tools.iter().find(|t| t.name == "kit/show");
        
        assert!(show_tool.is_some(), "kit/show tool should be defined");
        let tool = show_tool.unwrap();
        assert_eq!(tool.name, "kit/show");
        assert_eq!(tool.description, "Show the Script Kit window");
        assert!(tool.input_schema.get("type").is_some());
        assert_eq!(tool.input_schema.get("type").unwrap(), "object");
    }

    #[test]
    fn test_kit_hide_tool_definition() {
        let tools = get_kit_tool_definitions();
        let hide_tool = tools.iter().find(|t| t.name == "kit/hide");
        
        assert!(hide_tool.is_some(), "kit/hide tool should be defined");
        let tool = hide_tool.unwrap();
        assert_eq!(tool.name, "kit/hide");
        assert_eq!(tool.description, "Hide the Script Kit window");
        assert!(tool.input_schema.get("type").is_some());
        assert_eq!(tool.input_schema.get("type").unwrap(), "object");
    }

    #[test]
    fn test_kit_state_tool_definition() {
        let tools = get_kit_tool_definitions();
        let state_tool = tools.iter().find(|t| t.name == "kit/state");
        
        assert!(state_tool.is_some(), "kit/state tool should be defined");
        let tool = state_tool.unwrap();
        assert_eq!(tool.name, "kit/state");
        assert_eq!(tool.description, "Get current Script Kit app state");
        assert!(tool.input_schema.get("type").is_some());
        assert_eq!(tool.input_schema.get("type").unwrap(), "object");
    }

    #[test]
    fn test_tools_list_includes_kit_tools() {
        let tools = get_kit_tool_definitions();
        
        assert_eq!(tools.len(), 3, "Should have exactly 3 kit tools");
        
        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"kit/show"));
        assert!(tool_names.contains(&"kit/hide"));
        assert!(tool_names.contains(&"kit/state"));
    }

    #[test]
    fn test_kit_show_call_succeeds() {
        let result = handle_kit_tool_call("kit/show", &serde_json::json!({}));
        
        assert!(result.is_error.is_none() || result.is_error == Some(false));
        assert!(!result.content.is_empty());
        assert_eq!(result.content[0].content_type, "text");
        assert!(result.content[0].text.contains("show"));
    }

    #[test]
    fn test_kit_hide_call_succeeds() {
        let result = handle_kit_tool_call("kit/hide", &serde_json::json!({}));
        
        assert!(result.is_error.is_none() || result.is_error == Some(false));
        assert!(!result.content.is_empty());
        assert_eq!(result.content[0].content_type, "text");
        assert!(result.content[0].text.contains("hide"));
    }

    #[test]
    fn test_kit_state_returns_json() {
        let result = handle_kit_tool_call("kit/state", &serde_json::json!({}));
        
        assert!(result.is_error.is_none() || result.is_error == Some(false));
        assert!(!result.content.is_empty());
        assert_eq!(result.content[0].content_type, "text");
        
        // Verify the result is valid JSON with expected fields
        let state: Result<AppState, _> = serde_json::from_str(&result.content[0].text);
        assert!(state.is_ok(), "kit/state should return valid JSON");
        
        let state = state.unwrap();
        // Default state should have visible=false, focused=false
        assert!(!state.visible);
        assert!(!state.focused);
    }

    #[test]
    fn test_is_kit_tool() {
        assert!(is_kit_tool("kit/show"));
        assert!(is_kit_tool("kit/hide"));
        assert!(is_kit_tool("kit/state"));
        assert!(is_kit_tool("kit/custom"));
        
        assert!(!is_kit_tool("scripts/run"));
        assert!(!is_kit_tool("resources/list"));
        assert!(!is_kit_tool("kitshow")); // No slash
    }

    #[test]
    fn test_unknown_kit_tool_returns_error() {
        let result = handle_kit_tool_call("kit/unknown", &serde_json::json!({}));
        
        assert_eq!(result.is_error, Some(true));
        assert!(!result.content.is_empty());
        assert!(result.content[0].text.contains("Unknown kit tool"));
    }

    #[test]
    fn test_tool_definition_serialization() {
        let tools = get_kit_tool_definitions();
        let json = serde_json::to_value(&tools);
        
        assert!(json.is_ok(), "Tool definitions should serialize to JSON");
        
        let json = json.unwrap();
        assert!(json.is_array());
        
        // Check the first tool has expected structure
        let first_tool = &json[0];
        assert!(first_tool.get("name").is_some());
        assert!(first_tool.get("description").is_some());
        assert!(first_tool.get("inputSchema").is_some());
    }

    #[test]
    fn test_app_state_serialization() {
        let state = AppState {
            visible: true,
            focused: true,
            active_prompt: Some("arg".to_string()),
        };
        
        let json = serde_json::to_value(&state);
        assert!(json.is_ok());
        
        let json = json.unwrap();
        assert_eq!(json.get("visible").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(json.get("focused").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(json.get("activePrompt").and_then(|v| v.as_str()), Some("arg"));
    }

    #[test]
    fn test_tool_result_serialization() {
        let result = ToolResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: "test message".to_string(),
            }],
            is_error: None,
        };
        
        let json = serde_json::to_string(&result);
        assert!(json.is_ok());
        
        let json = json.unwrap();
        // is_error should be omitted when None
        assert!(!json.contains("isError"));
        assert!(json.contains("content"));
        assert!(json.contains("text"));
    }
}
