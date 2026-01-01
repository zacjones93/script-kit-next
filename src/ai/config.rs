//! AI provider configuration and environment variable detection.
//!
//! This module handles automatic discovery of AI provider API keys from environment
//! variables using the `SCRIPT_KIT_*_API_KEY` pattern for security.
//!
//! # Environment Variable Pattern
//!
//! API keys are detected with the `SCRIPT_KIT_` prefix:
//! - `SCRIPT_KIT_OPENAI_API_KEY` -> OpenAI provider
//! - `SCRIPT_KIT_ANTHROPIC_API_KEY` -> Anthropic provider
//!
//! This prefix ensures users explicitly configure keys for Script Kit,
//! rather than accidentally exposing keys from other applications.

use std::env;

/// Represents a detected AI provider configuration.
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    /// Unique identifier for the provider (e.g., "openai", "anthropic")
    pub provider_id: String,
    /// Human-readable name (e.g., "OpenAI", "Anthropic")
    pub display_name: String,
    /// The API key (should never be logged or displayed)
    api_key: String,
    /// Base URL for the API (for custom endpoints)
    pub base_url: Option<String>,
}

impl ProviderConfig {
    /// Create a new provider configuration.
    pub fn new(
        provider_id: impl Into<String>,
        display_name: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Self {
        Self {
            provider_id: provider_id.into(),
            display_name: display_name.into(),
            api_key: api_key.into(),
            base_url: None,
        }
    }

    /// Create a provider configuration with a custom base URL.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Get the API key for making requests.
    ///
    /// # Security Note
    /// This method intentionally returns a reference to prevent accidental
    /// copies of the API key. Never log or display the returned value.
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Check if this provider has a valid (non-empty) API key.
    pub fn has_valid_key(&self) -> bool {
        !self.api_key.is_empty()
    }
}

/// Information about an AI model.
#[derive(Debug, Clone)]
pub struct ModelInfo {
    /// Unique identifier for the model (e.g., "gpt-4o", "claude-3-5-sonnet")
    pub id: String,
    /// Human-readable display name
    pub display_name: String,
    /// Provider this model belongs to
    pub provider: String,
    /// Whether this model supports streaming responses
    pub supports_streaming: bool,
    /// Context window size in tokens
    pub context_window: u32,
}

impl ModelInfo {
    /// Create a new model info.
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        provider: impl Into<String>,
        supports_streaming: bool,
        context_window: u32,
    ) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            provider: provider.into(),
            supports_streaming,
            context_window,
        }
    }
}

/// Environment variable names for API keys.
pub mod env_vars {
    /// OpenAI API key environment variable
    pub const OPENAI_API_KEY: &str = "SCRIPT_KIT_OPENAI_API_KEY";
    /// Anthropic API key environment variable
    pub const ANTHROPIC_API_KEY: &str = "SCRIPT_KIT_ANTHROPIC_API_KEY";
    /// Google AI (Gemini) API key environment variable
    pub const GOOGLE_API_KEY: &str = "SCRIPT_KIT_GOOGLE_API_KEY";
    /// Groq API key environment variable
    pub const GROQ_API_KEY: &str = "SCRIPT_KIT_GROQ_API_KEY";
    /// OpenRouter API key environment variable
    pub const OPENROUTER_API_KEY: &str = "SCRIPT_KIT_OPENROUTER_API_KEY";
}

/// Detected API keys from environment.
#[derive(Debug, Default)]
pub struct DetectedKeys {
    pub openai: Option<String>,
    pub anthropic: Option<String>,
    pub google: Option<String>,
    pub groq: Option<String>,
    pub openrouter: Option<String>,
}

impl DetectedKeys {
    /// Scan environment variables for API keys.
    ///
    /// Looks for the `SCRIPT_KIT_*_API_KEY` pattern and collects all found keys.
    pub fn from_environment() -> Self {
        Self {
            openai: env::var(env_vars::OPENAI_API_KEY)
                .ok()
                .filter(|s| !s.is_empty()),
            anthropic: env::var(env_vars::ANTHROPIC_API_KEY)
                .ok()
                .filter(|s| !s.is_empty()),
            google: env::var(env_vars::GOOGLE_API_KEY)
                .ok()
                .filter(|s| !s.is_empty()),
            groq: env::var(env_vars::GROQ_API_KEY)
                .ok()
                .filter(|s| !s.is_empty()),
            openrouter: env::var(env_vars::OPENROUTER_API_KEY)
                .ok()
                .filter(|s| !s.is_empty()),
        }
    }

    /// Check if any API keys were detected.
    pub fn has_any(&self) -> bool {
        self.openai.is_some()
            || self.anthropic.is_some()
            || self.google.is_some()
            || self.groq.is_some()
            || self.openrouter.is_some()
    }

    /// Get a summary of which providers are available (for logging).
    ///
    /// Returns a list of provider names that have API keys configured.
    /// Does NOT include the actual keys.
    pub fn available_providers(&self) -> Vec<&'static str> {
        let mut providers = Vec::new();
        if self.openai.is_some() {
            providers.push("OpenAI");
        }
        if self.anthropic.is_some() {
            providers.push("Anthropic");
        }
        if self.google.is_some() {
            providers.push("Google");
        }
        if self.groq.is_some() {
            providers.push("Groq");
        }
        if self.openrouter.is_some() {
            providers.push("OpenRouter");
        }
        providers
    }
}

/// Default models for each provider.
pub mod default_models {
    use super::ModelInfo;

    /// Get default OpenAI models.
    pub fn openai() -> Vec<ModelInfo> {
        vec![
            ModelInfo::new("gpt-4o", "GPT-4o", "openai", true, 128_000),
            ModelInfo::new("gpt-4o-mini", "GPT-4o Mini", "openai", true, 128_000),
            ModelInfo::new("gpt-4-turbo", "GPT-4 Turbo", "openai", true, 128_000),
            ModelInfo::new("gpt-3.5-turbo", "GPT-3.5 Turbo", "openai", true, 16_385),
        ]
    }

    /// Get default Anthropic models.
    pub fn anthropic() -> Vec<ModelInfo> {
        vec![
            ModelInfo::new(
                "claude-3-5-sonnet-20241022",
                "Claude 3.5 Sonnet",
                "anthropic",
                true,
                200_000,
            ),
            ModelInfo::new(
                "claude-3-5-haiku-20241022",
                "Claude 3.5 Haiku",
                "anthropic",
                true,
                200_000,
            ),
            ModelInfo::new(
                "claude-3-opus-20240229",
                "Claude 3 Opus",
                "anthropic",
                true,
                200_000,
            ),
        ]
    }

    /// Get default Google (Gemini) models.
    pub fn google() -> Vec<ModelInfo> {
        vec![
            ModelInfo::new(
                "gemini-2.0-flash-exp",
                "Gemini 2.0 Flash",
                "google",
                true,
                1_000_000,
            ),
            ModelInfo::new(
                "gemini-1.5-pro",
                "Gemini 1.5 Pro",
                "google",
                true,
                2_000_000,
            ),
            ModelInfo::new(
                "gemini-1.5-flash",
                "Gemini 1.5 Flash",
                "google",
                true,
                1_000_000,
            ),
        ]
    }

    /// Get default Groq models.
    pub fn groq() -> Vec<ModelInfo> {
        vec![
            ModelInfo::new(
                "llama-3.3-70b-versatile",
                "Llama 3.3 70B",
                "groq",
                true,
                128_000,
            ),
            ModelInfo::new(
                "llama-3.1-8b-instant",
                "Llama 3.1 8B Instant",
                "groq",
                true,
                128_000,
            ),
            ModelInfo::new("mixtral-8x7b-32768", "Mixtral 8x7B", "groq", true, 32_768),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_config_creation() {
        let config = ProviderConfig::new("openai", "OpenAI", "sk-test-key");
        assert_eq!(config.provider_id, "openai");
        assert_eq!(config.display_name, "OpenAI");
        assert_eq!(config.api_key(), "sk-test-key");
        assert!(config.has_valid_key());
    }

    #[test]
    fn test_provider_config_empty_key() {
        let config = ProviderConfig::new("openai", "OpenAI", "");
        assert!(!config.has_valid_key());
    }

    #[test]
    fn test_provider_config_with_base_url() {
        let config = ProviderConfig::new("openai", "OpenAI", "sk-test")
            .with_base_url("https://api.custom.com");
        assert_eq!(config.base_url, Some("https://api.custom.com".to_string()));
    }

    #[test]
    fn test_model_info_creation() {
        let model = ModelInfo::new("gpt-4o", "GPT-4o", "openai", true, 128_000);
        assert_eq!(model.id, "gpt-4o");
        assert_eq!(model.display_name, "GPT-4o");
        assert_eq!(model.provider, "openai");
        assert!(model.supports_streaming);
        assert_eq!(model.context_window, 128_000);
    }

    #[test]
    fn test_detected_keys_empty() {
        // Clear any existing env vars for this test
        let keys = DetectedKeys::default();
        assert!(!keys.has_any());
        assert!(keys.available_providers().is_empty());
    }

    #[test]
    fn test_detected_keys_with_provider() {
        // Manually construct to avoid env dependency in test
        let keys = DetectedKeys {
            openai: Some("sk-test".to_string()),
            anthropic: None,
            google: None,
            groq: None,
            openrouter: None,
        };
        assert!(keys.has_any());
        assert_eq!(keys.available_providers(), vec!["OpenAI"]);
    }

    #[test]
    fn test_default_models() {
        let openai_models = default_models::openai();
        assert!(!openai_models.is_empty());
        assert!(openai_models.iter().any(|m| m.id == "gpt-4o"));

        let anthropic_models = default_models::anthropic();
        assert!(!anthropic_models.is_empty());
        assert!(anthropic_models.iter().any(|m| m.id.contains("claude")));
    }
}
