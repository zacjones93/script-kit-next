//! User shortcut customization persistence.
//!
//! Handles loading and saving user shortcut overrides to/from config.
//! Format: HashMap<binding_id, Option<String>> where:
//! - Some(shortcut_string) = user override to new shortcut
//! - None = user disabled this shortcut

#![allow(dead_code)]

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::registry::ShortcutRegistry;
use super::types::{Shortcut, ShortcutParseError};

/// User shortcut overrides configuration.
///
/// Stored in ~/.scriptkit/shortcuts.json
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ShortcutOverrides {
    /// Map of binding_id -> override
    /// - Some(string) = new shortcut
    /// - null in JSON = disabled
    #[serde(default)]
    pub overrides: HashMap<String, Option<String>>,
}

/// Error that can occur when loading/saving shortcut overrides.
#[derive(Debug)]
pub enum PersistenceError {
    /// IO error reading/writing file
    Io(std::io::Error),
    /// JSON parse error
    Json(serde_json::Error),
    /// Invalid shortcut string in config
    InvalidShortcut {
        binding_id: String,
        shortcut: String,
        error: ShortcutParseError,
    },
}

impl std::fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {}", e),
            Self::Json(e) => write!(f, "JSON parse error: {}", e),
            Self::InvalidShortcut {
                binding_id,
                shortcut,
                error,
            } => {
                write!(
                    f,
                    "Invalid shortcut '{}' for binding '{}': {}",
                    shortcut, binding_id, error
                )
            }
        }
    }
}

impl std::error::Error for PersistenceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Json(e) => Some(e),
            Self::InvalidShortcut { error, .. } => Some(error),
        }
    }
}

impl From<std::io::Error> for PersistenceError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for PersistenceError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl ShortcutOverrides {
    /// Load overrides from a JSON file.
    ///
    /// Returns empty overrides if file doesn't exist.
    pub fn load(path: &Path) -> Result<Self, PersistenceError> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path)?;
        let overrides: Self = serde_json::from_str(&content)?;
        Ok(overrides)
    }

    /// Save overrides to a JSON file.
    pub fn save(&self, path: &Path) -> Result<(), PersistenceError> {
        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Apply overrides to a registry.
    ///
    /// Returns a list of parse errors for invalid shortcuts (but still applies valid ones).
    pub fn apply_to_registry(&self, registry: &mut ShortcutRegistry) -> Vec<PersistenceError> {
        let mut errors = Vec::new();

        for (binding_id, override_opt) in &self.overrides {
            match override_opt {
                None => {
                    // Disable this shortcut
                    registry.set_override(binding_id, None);
                }
                Some(shortcut_str) => {
                    // Parse and set override
                    match Shortcut::parse(shortcut_str) {
                        Ok(shortcut) => {
                            registry.set_override(binding_id, Some(shortcut));
                        }
                        Err(e) => {
                            errors.push(PersistenceError::InvalidShortcut {
                                binding_id: binding_id.clone(),
                                shortcut: shortcut_str.clone(),
                                error: e,
                            });
                        }
                    }
                }
            }
        }

        errors
    }

    /// Extract current overrides from a registry.
    pub fn from_registry(registry: &ShortcutRegistry) -> Self {
        let overrides = registry.export_overrides();
        Self { overrides }
    }

    /// Set an override.
    pub fn set(&mut self, binding_id: impl Into<String>, shortcut: Option<String>) {
        self.overrides.insert(binding_id.into(), shortcut);
    }

    /// Remove an override (revert to default).
    pub fn remove(&mut self, binding_id: &str) {
        self.overrides.remove(binding_id);
    }

    /// Check if a binding has an override.
    pub fn has_override(&self, binding_id: &str) -> bool {
        self.overrides.contains_key(binding_id)
    }

    /// Get the override for a binding.
    pub fn get(&self, binding_id: &str) -> Option<&Option<String>> {
        self.overrides.get(binding_id)
    }

    /// Get the number of overrides.
    pub fn len(&self) -> usize {
        self.overrides.len()
    }

    /// Check if there are no overrides.
    pub fn is_empty(&self) -> bool {
        self.overrides.is_empty()
    }

    /// Clear all overrides.
    pub fn clear(&mut self) {
        self.overrides.clear();
    }
}

/// Get the default path for shortcut overrides.
pub fn default_overrides_path() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".scriptkit")
        .join("shortcuts.json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn load_nonexistent_returns_empty() {
        let result = ShortcutOverrides::load(Path::new("/nonexistent/path/shortcuts.json"));
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("shortcuts.json");

        let mut overrides = ShortcutOverrides::default();
        overrides.set("test.action", Some("cmd+k".to_string()));
        overrides.set("test.disabled", None);

        overrides.save(&path).unwrap();

        let loaded = ShortcutOverrides::load(&path).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded.get("test.action"), Some(&Some("cmd+k".to_string())));
        assert_eq!(loaded.get("test.disabled"), Some(&None));
    }

    #[test]
    fn apply_valid_override_to_registry() {
        use super::super::context::ShortcutContext;
        use super::super::registry::{ShortcutBinding, ShortcutCategory, ShortcutRegistry};
        use super::super::types::Modifiers;

        let mut registry = ShortcutRegistry::new();
        registry.register(ShortcutBinding::builtin(
            "test.action",
            "Test",
            Shortcut {
                key: "k".to_string(),
                modifiers: Modifiers::cmd(),
            },
            ShortcutContext::Global,
            ShortcutCategory::Actions,
        ));

        let mut overrides = ShortcutOverrides::default();
        overrides.set("test.action", Some("cmd+j".to_string()));

        let errors = overrides.apply_to_registry(&mut registry);
        assert!(errors.is_empty());

        let shortcut = registry.get_shortcut("test.action").unwrap();
        assert_eq!(shortcut.key, "j");
    }

    #[test]
    fn apply_disable_override_to_registry() {
        use super::super::context::ShortcutContext;
        use super::super::registry::{ShortcutBinding, ShortcutCategory, ShortcutRegistry};
        use super::super::types::Modifiers;

        let mut registry = ShortcutRegistry::new();
        registry.register(ShortcutBinding::builtin(
            "test.action",
            "Test",
            Shortcut {
                key: "k".to_string(),
                modifiers: Modifiers::cmd(),
            },
            ShortcutContext::Global,
            ShortcutCategory::Actions,
        ));

        let mut overrides = ShortcutOverrides::default();
        overrides.set("test.action", None);

        let errors = overrides.apply_to_registry(&mut registry);
        assert!(errors.is_empty());

        assert!(registry.is_disabled("test.action"));
    }

    #[test]
    fn apply_invalid_shortcut_returns_error() {
        use super::super::registry::ShortcutRegistry;

        let mut registry = ShortcutRegistry::new();
        let mut overrides = ShortcutOverrides::default();
        overrides.set("test.action", Some("invalid+shortcut+xyz".to_string()));

        let errors = overrides.apply_to_registry(&mut registry);
        assert_eq!(errors.len(), 1);
        match &errors[0] {
            PersistenceError::InvalidShortcut { binding_id, .. } => {
                assert_eq!(binding_id, "test.action");
            }
            _ => panic!("Expected InvalidShortcut error"),
        }
    }

    #[test]
    fn set_and_remove_override() {
        let mut overrides = ShortcutOverrides::default();

        overrides.set("test.action", Some("cmd+k".to_string()));
        assert!(overrides.has_override("test.action"));

        overrides.remove("test.action");
        assert!(!overrides.has_override("test.action"));
    }

    #[test]
    fn json_format_is_readable() {
        let mut overrides = ShortcutOverrides::default();
        overrides.set("nav.up", Some("cmd+k".to_string()));
        overrides.set("nav.down", Some("cmd+j".to_string()));
        overrides.set("edit.copy", None);

        let json = serde_json::to_string_pretty(&overrides).unwrap();

        // Verify it's human-readable
        assert!(json.contains("nav.up"));
        assert!(json.contains("cmd+k"));
        assert!(json.contains("edit.copy"));
        assert!(json.contains("null")); // disabled shortcut
    }
}
