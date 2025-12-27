use serde::{Deserialize, Serialize};
use std::process::Command;
use std::path::PathBuf;
use tracing::{info, warn, instrument};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub hotkey: HotkeyConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bun_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub modifiers: Vec<String>,
    pub key: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            hotkey: HotkeyConfig {
                modifiers: vec!["meta".to_string()],
                key: "Semicolon".to_string(),  // Cmd+; matches main.rs default
            },
            bun_path: None,  // Will use system PATH if not specified
            editor: None,    // Will use $EDITOR or fallback to "code"
        }
    }
}

impl Config {
    /// Returns the configured editor, falling back to $EDITOR env var or "code" (VS Code)
    /// Used by ActionsDialog "Open in Editor" action
    #[allow(dead_code)]  // Will be used by ActionsDialog worker
    pub fn get_editor(&self) -> String {
        self.editor
            .clone()
            .or_else(|| std::env::var("EDITOR").ok())
            .unwrap_or_else(|| "code".to_string())
    }
}

#[instrument(name = "load_config")]
pub fn load_config() -> Config {
    let config_path = PathBuf::from(shellexpand::tilde("~/.kit/config.ts").as_ref());

    // Check if config file exists
    if !config_path.exists() {
        info!(path = %config_path.display(), "Config file not found, using defaults");
        return Config::default();
    }

    // Step 1: Transpile TypeScript to JavaScript using bun build
    let tmp_js_path = "/tmp/kit-config.js";
    let build_output = Command::new("bun")
        .arg("build")
        .arg("--target=bun")
        .arg(config_path.to_string_lossy().to_string())
        .arg(format!("--outfile={}", tmp_js_path))
        .output();

    match build_output {
        Err(e) => {
            warn!(error = %e, "Failed to transpile config with bun, using defaults");
            return Config::default();
        }
        Ok(output) => {
            if !output.status.success() {
                warn!(
                    stderr = %String::from_utf8_lossy(&output.stderr),
                    "bun build failed, using defaults"
                );
                return Config::default();
            }
        }
    }

    // Step 2: Execute the transpiled JS and extract the default export as JSON
    let json_output = Command::new("bun")
        .arg("-e")
        .arg(format!(
            "console.log(JSON.stringify(require('{}').default))",
            tmp_js_path
        ))
        .output();

    match json_output {
        Err(e) => {
            warn!(error = %e, "Failed to execute bun to extract JSON, using defaults");
            Config::default()
        }
        Ok(output) => {
            if !output.status.success() {
                warn!(
                    stderr = %String::from_utf8_lossy(&output.stderr),
                    "bun execution failed, using defaults"
                );
                Config::default()
            } else {
                // Step 3: Parse the JSON output into Config struct
                let json_str = String::from_utf8_lossy(&output.stdout);
                match serde_json::from_str::<Config>(json_str.trim()) {
                    Ok(config) => {
                        info!(path = %config_path.display(), "Successfully loaded config");
                        config
                    }
                    Err(e) => {
                        warn!(
                            error = %e,
                            json_output = %json_str,
                            "Failed to parse config JSON, using defaults"
                        );
                        Config::default()
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.hotkey.modifiers, vec!["meta"]);
        assert_eq!(config.hotkey.key, "Semicolon");
        assert_eq!(config.bun_path, None);
        assert_eq!(config.editor, None);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config {
            hotkey: HotkeyConfig {
                modifiers: vec!["ctrl".to_string(), "alt".to_string()],
                key: "KeyA".to_string(),
            },
            bun_path: Some("/usr/local/bin/bun".to_string()),
            editor: Some("vim".to_string()),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.hotkey.modifiers, config.hotkey.modifiers);
        assert_eq!(deserialized.hotkey.key, config.hotkey.key);
        assert_eq!(deserialized.bun_path, config.bun_path);
        assert_eq!(deserialized.editor, config.editor);
    }

    #[test]
    fn test_hotkey_config_default_values() {
        let hotkey = HotkeyConfig {
            modifiers: vec!["meta".to_string(), "shift".to_string()],
            key: "KeyK".to_string(),
        };
        assert_eq!(hotkey.modifiers.len(), 2);
        assert!(hotkey.modifiers.contains(&"meta".to_string()));
        assert!(hotkey.modifiers.contains(&"shift".to_string()));
    }

    #[test]
    fn test_config_with_bun_path() {
        let config = Config {
            hotkey: HotkeyConfig {
                modifiers: vec!["meta".to_string()],
                key: "Semicolon".to_string(),
            },
            bun_path: Some("/custom/path/bun".to_string()),
            editor: None,
        };
        assert_eq!(config.bun_path, Some("/custom/path/bun".to_string()));
    }

    #[test]
    fn test_config_without_bun_path() {
        let config = Config {
            hotkey: HotkeyConfig {
                modifiers: vec!["meta".to_string()],
                key: "Semicolon".to_string(),
            },
            bun_path: None,
            editor: None,
        };
        assert_eq!(config.bun_path, None);
    }

    #[test]
    fn test_config_serialization_skip_none_bun_path() {
        let config = Config {
            hotkey: HotkeyConfig {
                modifiers: vec!["meta".to_string()],
                key: "Semicolon".to_string(),
            },
            bun_path: None,
            editor: None,
        };

        let json = serde_json::to_string(&config).unwrap();
        // Verify that bun_path is not included when None
        assert!(!json.contains("null"));
        // Should contain hotkey config
        assert!(json.contains("meta"));
    }

    #[test]
    fn test_config_serialization_preserves_multiple_modifiers() {
        let config = Config {
            hotkey: HotkeyConfig {
                modifiers: vec!["ctrl".to_string(), "shift".to_string(), "alt".to_string()],
                key: "KeyP".to_string(),
            },
            bun_path: None,
            editor: None,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.hotkey.modifiers.len(), 3);
        assert_eq!(deserialized.hotkey.key, "KeyP");
    }

    #[test]
    fn test_config_deserialization_with_custom_values() {
        let json = r#"{
            "hotkey": {
                "modifiers": ["shift", "alt"],
                "key": "KeyX"
            },
            "bun_path": "/usr/bin/bun"
        }"#;

        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.hotkey.modifiers, vec!["shift", "alt"]);
        assert_eq!(config.hotkey.key, "KeyX");
        assert_eq!(config.bun_path, Some("/usr/bin/bun".to_string()));
    }

    #[test]
    fn test_config_deserialization_minimal() {
        let json = r#"{
            "hotkey": {
                "modifiers": ["meta"],
                "key": "Semicolon"
            }
        }"#;

        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.hotkey.modifiers, vec!["meta"]);
        assert_eq!(config.hotkey.key, "Semicolon");
        assert_eq!(config.bun_path, None);
    }

    #[test]
    fn test_load_config_returns_config_struct() {
        // Test that load_config returns a valid Config struct
        // It may load from actual config or return defaults
        let config = load_config();
        // Verify it has required fields
        assert!(!config.hotkey.modifiers.is_empty());
        assert!(!config.hotkey.key.is_empty());
        // Either has bun_path set or it's None - both valid
        let _ = config.bun_path;
    }

    #[test]
    fn test_config_clone_independence() {
        let config1 = Config::default();
        let config2 = config1.clone();

        // Verify they are equal but independent
        assert_eq!(config1.hotkey.modifiers, config2.hotkey.modifiers);
        assert_eq!(config1.hotkey.key, config2.hotkey.key);
        assert_eq!(config1.bun_path, config2.bun_path);
        assert_eq!(config1.editor, config2.editor);
    }

    #[test]
    fn test_hotkey_config_clone() {
        let hotkey = HotkeyConfig {
            modifiers: vec!["meta".to_string(), "alt".to_string()],
            key: "KeyK".to_string(),
        };
        let cloned = hotkey.clone();

        assert_eq!(hotkey.modifiers, cloned.modifiers);
        assert_eq!(hotkey.key, cloned.key);
    }

    #[test]
    fn test_config_with_empty_modifiers_list() {
        let config = Config {
            hotkey: HotkeyConfig {
                modifiers: vec![],
                key: "KeyA".to_string(),
            },
            bun_path: None,
            editor: None,
        };

        assert_eq!(config.hotkey.modifiers.len(), 0);
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.hotkey.modifiers.len(), 0);
    }

    #[test]
    fn test_config_key_preservation() {
        let keys = vec!["Semicolon", "KeyK", "KeyP", "Space", "Enter"];
        for key in keys {
            let config = Config {
                hotkey: HotkeyConfig {
                    modifiers: vec!["meta".to_string()],
                    key: key.to_string(),
                },
                bun_path: None,
                editor: None,
            };

            let json = serde_json::to_string(&config).unwrap();
            let deserialized: Config = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized.hotkey.key, key);
        }
    }

    // Editor config tests
    #[test]
    fn test_config_with_editor() {
        let config = Config {
            hotkey: HotkeyConfig {
                modifiers: vec!["meta".to_string()],
                key: "Semicolon".to_string(),
            },
            bun_path: None,
            editor: Some("vim".to_string()),
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("vim"));
        
        let deserialized: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.editor, Some("vim".to_string()));
    }

    #[test]
    fn test_config_without_editor() {
        let config = Config {
            hotkey: HotkeyConfig {
                modifiers: vec!["meta".to_string()],
                key: "Semicolon".to_string(),
            },
            bun_path: None,
            editor: None,
        };

        let json = serde_json::to_string(&config).unwrap();
        // Editor should not appear in JSON when None (skip_serializing_if)
        assert!(!json.contains("editor"));
        
        let deserialized: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.editor, None);
    }

    #[test]
    fn test_get_editor_from_config() {
        let config = Config {
            hotkey: HotkeyConfig {
                modifiers: vec!["meta".to_string()],
                key: "Semicolon".to_string(),
            },
            bun_path: None,
            editor: Some("nvim".to_string()),
        };

        // Config editor takes precedence
        assert_eq!(config.get_editor(), "nvim");
    }

    #[test]
    fn test_get_editor_from_env() {
        // Save current EDITOR value
        let original_editor = std::env::var("EDITOR").ok();
        
        // Set EDITOR env var
        std::env::set_var("EDITOR", "emacs");
        
        let config = Config {
            hotkey: HotkeyConfig {
                modifiers: vec!["meta".to_string()],
                key: "Semicolon".to_string(),
            },
            bun_path: None,
            editor: None,
        };

        // Should fall back to EDITOR env var
        assert_eq!(config.get_editor(), "emacs");
        
        // Restore original EDITOR value
        match original_editor {
            Some(val) => std::env::set_var("EDITOR", val),
            None => std::env::remove_var("EDITOR"),
        }
    }

    #[test]
    fn test_get_editor_default() {
        // Save current EDITOR value
        let original_editor = std::env::var("EDITOR").ok();
        
        // Remove EDITOR env var
        std::env::remove_var("EDITOR");
        
        let config = Config {
            hotkey: HotkeyConfig {
                modifiers: vec!["meta".to_string()],
                key: "Semicolon".to_string(),
            },
            bun_path: None,
            editor: None,
        };

        // Should fall back to "code" default
        assert_eq!(config.get_editor(), "code");
        
        // Restore original EDITOR value
        if let Some(val) = original_editor {
            std::env::set_var("EDITOR", val);
        }
    }

    #[test]
    fn test_config_editor_priority() {
        // Save current EDITOR value
        let original_editor = std::env::var("EDITOR").ok();
        
        // Set EDITOR env var
        std::env::set_var("EDITOR", "emacs");
        
        // Config with editor set should take precedence over env var
        let config = Config {
            hotkey: HotkeyConfig {
                modifiers: vec!["meta".to_string()],
                key: "Semicolon".to_string(),
            },
            bun_path: None,
            editor: Some("vim".to_string()),
        };

        // Config editor should win
        assert_eq!(config.get_editor(), "vim");
        
        // Restore original EDITOR value
        match original_editor {
            Some(val) => std::env::set_var("EDITOR", val),
            None => std::env::remove_var("EDITOR"),
        }
    }

    #[test]
    fn test_config_deserialization_with_editor() {
        let json = r#"{
            "hotkey": {
                "modifiers": ["meta"],
                "key": "Semicolon"
            },
            "editor": "subl"
        }"#;

        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.editor, Some("subl".to_string()));
        assert_eq!(config.get_editor(), "subl");
    }
}
