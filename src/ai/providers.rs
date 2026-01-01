//! AI provider abstraction layer.
//!
//! This module provides a trait-based abstraction for AI providers, allowing
//! Script Kit to work with multiple AI services (OpenAI, Anthropic, etc.) through
//! a unified interface.
//!
//! # Architecture
//!
//! - `AiProvider` trait defines the interface all providers must implement
//! - `ProviderRegistry` manages available providers based on detected API keys
//! - Individual provider implementations (OpenAI, Anthropic, etc.) implement the trait
//!
//! # Usage
//!
//! ```rust,ignore
//! let registry = ProviderRegistry::from_environment();
//! if registry.has_any_provider() {
//!     let models = registry.get_all_models();
//!     // Use models...
//! }
//! ```

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

use super::config::{default_models, DetectedKeys, ModelInfo, ProviderConfig};

/// Message for AI provider API calls.
#[derive(Debug, Clone)]
pub struct ProviderMessage {
    /// Role of the message sender: "user", "assistant", or "system"
    pub role: String,
    /// Content of the message
    pub content: String,
}

impl ProviderMessage {
    /// Create a new user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    /// Create a new assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }

    /// Create a new system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }
}

/// Callback type for streaming responses.
pub type StreamCallback = Box<dyn Fn(String) + Send + Sync>;

/// Trait defining the interface for AI providers.
///
/// All AI providers (OpenAI, Anthropic, etc.) implement this trait to provide
/// a consistent interface for the AI window.
///
/// # Note on Async
///
/// Currently methods are synchronous for simplicity. When real HTTP integration
/// is added, these will become async using the `async_trait` crate.
pub trait AiProvider: Send + Sync {
    /// Unique identifier for this provider (e.g., "openai", "anthropic").
    fn provider_id(&self) -> &str;

    /// Human-readable display name (e.g., "OpenAI", "Anthropic").
    fn display_name(&self) -> &str;

    /// Get the list of available models for this provider.
    fn available_models(&self) -> Vec<ModelInfo>;

    /// Send a message and get a response (non-streaming).
    ///
    /// # Arguments
    ///
    /// * `messages` - The conversation history
    /// * `model_id` - The model to use for generation
    ///
    /// # Returns
    ///
    /// The generated response text, or an error.
    fn send_message(&self, messages: &[ProviderMessage], model_id: &str) -> Result<String>;

    /// Send a message with streaming response.
    ///
    /// # Arguments
    ///
    /// * `messages` - The conversation history
    /// * `model_id` - The model to use for generation
    /// * `on_chunk` - Callback invoked for each chunk of the response
    ///
    /// # Returns
    ///
    /// Ok(()) on success, or an error.
    fn stream_message(
        &self,
        messages: &[ProviderMessage],
        model_id: &str,
        on_chunk: StreamCallback,
    ) -> Result<()>;
}

/// OpenAI provider implementation.
pub struct OpenAiProvider {
    config: ProviderConfig,
}

impl OpenAiProvider {
    /// Create a new OpenAI provider with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            config: ProviderConfig::new("openai", "OpenAI", api_key),
        }
    }

    /// Create with a custom base URL (for Azure OpenAI or proxies).
    pub fn with_base_url(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            config: ProviderConfig::new("openai", "OpenAI", api_key).with_base_url(base_url),
        }
    }
}

impl AiProvider for OpenAiProvider {
    fn provider_id(&self) -> &str {
        &self.config.provider_id
    }

    fn display_name(&self) -> &str {
        &self.config.display_name
    }

    fn available_models(&self) -> Vec<ModelInfo> {
        default_models::openai()
    }

    fn send_message(&self, messages: &[ProviderMessage], model_id: &str) -> Result<String> {
        // TODO: Implement real API call when HTTP client is added
        // For now, return a mock response
        let last_user_msg = messages
            .iter()
            .rev()
            .find(|m| m.role == "user")
            .map(|m| m.content.as_str())
            .unwrap_or("(no message)");

        Ok(format!(
            "[Mock OpenAI Response]\nModel: {}\nProvider: {}\n\nI received your message: \"{}\"",
            model_id,
            self.display_name(),
            last_user_msg
        ))
    }

    fn stream_message(
        &self,
        messages: &[ProviderMessage],
        model_id: &str,
        on_chunk: StreamCallback,
    ) -> Result<()> {
        // TODO: Implement real streaming when HTTP client is added
        // For now, simulate streaming by sending chunks
        let response = self.send_message(messages, model_id)?;

        // Simulate streaming by splitting response into words
        for word in response.split_whitespace() {
            on_chunk(format!("{} ", word));
        }

        Ok(())
    }
}

/// Anthropic provider implementation.
pub struct AnthropicProvider {
    config: ProviderConfig,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            config: ProviderConfig::new("anthropic", "Anthropic", api_key),
        }
    }
}

impl AiProvider for AnthropicProvider {
    fn provider_id(&self) -> &str {
        &self.config.provider_id
    }

    fn display_name(&self) -> &str {
        &self.config.display_name
    }

    fn available_models(&self) -> Vec<ModelInfo> {
        default_models::anthropic()
    }

    fn send_message(&self, messages: &[ProviderMessage], model_id: &str) -> Result<String> {
        // TODO: Implement real API call when HTTP client is added
        let last_user_msg = messages
            .iter()
            .rev()
            .find(|m| m.role == "user")
            .map(|m| m.content.as_str())
            .unwrap_or("(no message)");

        Ok(format!(
            "[Mock Anthropic Response]\nModel: {}\nProvider: {}\n\nI received your message: \"{}\"",
            model_id,
            self.display_name(),
            last_user_msg
        ))
    }

    fn stream_message(
        &self,
        messages: &[ProviderMessage],
        model_id: &str,
        on_chunk: StreamCallback,
    ) -> Result<()> {
        let response = self.send_message(messages, model_id)?;

        for word in response.split_whitespace() {
            on_chunk(format!("{} ", word));
        }

        Ok(())
    }
}

/// Google (Gemini) provider implementation.
pub struct GoogleProvider {
    config: ProviderConfig,
}

impl GoogleProvider {
    /// Create a new Google provider with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            config: ProviderConfig::new("google", "Google", api_key),
        }
    }
}

impl AiProvider for GoogleProvider {
    fn provider_id(&self) -> &str {
        &self.config.provider_id
    }

    fn display_name(&self) -> &str {
        &self.config.display_name
    }

    fn available_models(&self) -> Vec<ModelInfo> {
        default_models::google()
    }

    fn send_message(&self, messages: &[ProviderMessage], model_id: &str) -> Result<String> {
        let last_user_msg = messages
            .iter()
            .rev()
            .find(|m| m.role == "user")
            .map(|m| m.content.as_str())
            .unwrap_or("(no message)");

        Ok(format!(
            "[Mock Google Response]\nModel: {}\nProvider: {}\n\nI received your message: \"{}\"",
            model_id,
            self.display_name(),
            last_user_msg
        ))
    }

    fn stream_message(
        &self,
        messages: &[ProviderMessage],
        model_id: &str,
        on_chunk: StreamCallback,
    ) -> Result<()> {
        let response = self.send_message(messages, model_id)?;

        for word in response.split_whitespace() {
            on_chunk(format!("{} ", word));
        }

        Ok(())
    }
}

/// Groq provider implementation.
pub struct GroqProvider {
    config: ProviderConfig,
}

impl GroqProvider {
    /// Create a new Groq provider with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            config: ProviderConfig::new("groq", "Groq", api_key),
        }
    }
}

impl AiProvider for GroqProvider {
    fn provider_id(&self) -> &str {
        &self.config.provider_id
    }

    fn display_name(&self) -> &str {
        &self.config.display_name
    }

    fn available_models(&self) -> Vec<ModelInfo> {
        default_models::groq()
    }

    fn send_message(&self, messages: &[ProviderMessage], model_id: &str) -> Result<String> {
        let last_user_msg = messages
            .iter()
            .rev()
            .find(|m| m.role == "user")
            .map(|m| m.content.as_str())
            .unwrap_or("(no message)");

        Ok(format!(
            "[Mock Groq Response]\nModel: {}\nProvider: {}\n\nI received your message: \"{}\"",
            model_id,
            self.display_name(),
            last_user_msg
        ))
    }

    fn stream_message(
        &self,
        messages: &[ProviderMessage],
        model_id: &str,
        on_chunk: StreamCallback,
    ) -> Result<()> {
        let response = self.send_message(messages, model_id)?;

        for word in response.split_whitespace() {
            on_chunk(format!("{} ", word));
        }

        Ok(())
    }
}

/// Registry of available AI providers.
///
/// The registry automatically discovers available providers based on
/// environment variables and provides a unified interface to access them.
pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn AiProvider>>,
}

impl ProviderRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Create a registry populated from environment variables.
    ///
    /// Scans for `SCRIPT_KIT_*_API_KEY` environment variables and
    /// creates providers for each detected key.
    pub fn from_environment() -> Self {
        let keys = DetectedKeys::from_environment();
        let mut registry = Self::new();

        if let Some(key) = keys.openai {
            registry.register(Arc::new(OpenAiProvider::new(key)));
        }

        if let Some(key) = keys.anthropic {
            registry.register(Arc::new(AnthropicProvider::new(key)));
        }

        if let Some(key) = keys.google {
            registry.register(Arc::new(GoogleProvider::new(key)));
        }

        if let Some(key) = keys.groq {
            registry.register(Arc::new(GroqProvider::new(key)));
        }

        // Log which providers are available (without exposing keys)
        let available: Vec<_> = registry.providers.keys().collect();
        if !available.is_empty() {
            tracing::info!(
                providers = ?available,
                "AI providers initialized from environment"
            );
        } else {
            tracing::debug!("No AI provider API keys found in environment");
        }

        registry
    }

    /// Register a provider with the registry.
    pub fn register(&mut self, provider: Arc<dyn AiProvider>) {
        self.providers
            .insert(provider.provider_id().to_string(), provider);
    }

    /// Check if any providers are available.
    pub fn has_any_provider(&self) -> bool {
        !self.providers.is_empty()
    }

    /// Get a provider by ID.
    pub fn get_provider(&self, id: &str) -> Option<&Arc<dyn AiProvider>> {
        self.providers.get(id)
    }

    /// Get all registered provider IDs.
    pub fn provider_ids(&self) -> Vec<&str> {
        self.providers.keys().map(|s| s.as_str()).collect()
    }

    /// Get all available models from all providers.
    pub fn get_all_models(&self) -> Vec<ModelInfo> {
        let mut models = Vec::new();
        for provider in self.providers.values() {
            models.extend(provider.available_models());
        }
        models
    }

    /// Get models for a specific provider.
    pub fn get_models_for_provider(&self, provider_id: &str) -> Vec<ModelInfo> {
        self.providers
            .get(provider_id)
            .map(|p| p.available_models())
            .unwrap_or_default()
    }

    /// Find the provider that owns a specific model.
    pub fn find_provider_for_model(&self, model_id: &str) -> Option<&Arc<dyn AiProvider>> {
        self.providers
            .values()
            .find(|provider| provider.available_models().iter().any(|m| m.id == model_id))
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_message_constructors() {
        let user = ProviderMessage::user("Hello");
        assert_eq!(user.role, "user");
        assert_eq!(user.content, "Hello");

        let assistant = ProviderMessage::assistant("Hi there");
        assert_eq!(assistant.role, "assistant");
        assert_eq!(assistant.content, "Hi there");

        let system = ProviderMessage::system("You are helpful");
        assert_eq!(system.role, "system");
        assert_eq!(system.content, "You are helpful");
    }

    #[test]
    fn test_openai_provider() {
        let provider = OpenAiProvider::new("test-key");
        assert_eq!(provider.provider_id(), "openai");
        assert_eq!(provider.display_name(), "OpenAI");

        let models = provider.available_models();
        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.id == "gpt-4o"));
    }

    #[test]
    fn test_anthropic_provider() {
        let provider = AnthropicProvider::new("test-key");
        assert_eq!(provider.provider_id(), "anthropic");
        assert_eq!(provider.display_name(), "Anthropic");

        let models = provider.available_models();
        assert!(!models.is_empty());
    }

    #[test]
    fn test_send_message_mock() {
        let provider = OpenAiProvider::new("test-key");
        let messages = vec![
            ProviderMessage::system("You are helpful"),
            ProviderMessage::user("Hello, world!"),
        ];

        let response = provider.send_message(&messages, "gpt-4o").unwrap();
        assert!(response.contains("Hello, world!"));
        assert!(response.contains("OpenAI"));
    }

    #[test]
    fn test_stream_message_mock() {
        let provider = OpenAiProvider::new("test-key");
        let messages = vec![ProviderMessage::user("Test")];

        let chunks = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let chunks_clone = chunks.clone();

        provider
            .stream_message(
                &messages,
                "gpt-4o",
                Box::new(move |chunk| {
                    chunks_clone.lock().unwrap().push(chunk);
                }),
            )
            .unwrap();

        let collected = chunks.lock().unwrap();
        assert!(!collected.is_empty());
    }

    #[test]
    fn test_registry_empty() {
        let registry = ProviderRegistry::new();
        assert!(!registry.has_any_provider());
        assert!(registry.get_all_models().is_empty());
    }

    #[test]
    fn test_registry_register() {
        let mut registry = ProviderRegistry::new();
        registry.register(Arc::new(OpenAiProvider::new("test-key")));

        assert!(registry.has_any_provider());
        assert!(registry.get_provider("openai").is_some());
        assert!(registry.get_provider("anthropic").is_none());
    }

    #[test]
    fn test_registry_get_all_models() {
        let mut registry = ProviderRegistry::new();
        registry.register(Arc::new(OpenAiProvider::new("test")));
        registry.register(Arc::new(AnthropicProvider::new("test")));

        let models = registry.get_all_models();
        assert!(models.iter().any(|m| m.provider == "openai"));
        assert!(models.iter().any(|m| m.provider == "anthropic"));
    }

    #[test]
    fn test_registry_find_provider_for_model() {
        let mut registry = ProviderRegistry::new();
        registry.register(Arc::new(OpenAiProvider::new("test")));
        registry.register(Arc::new(AnthropicProvider::new("test")));

        let provider = registry.find_provider_for_model("gpt-4o");
        assert!(provider.is_some());
        assert_eq!(provider.unwrap().provider_id(), "openai");

        let provider = registry.find_provider_for_model("claude-3-5-sonnet-20241022");
        assert!(provider.is_some());
        assert_eq!(provider.unwrap().provider_id(), "anthropic");

        let provider = registry.find_provider_for_model("nonexistent");
        assert!(provider.is_none());
    }
}
