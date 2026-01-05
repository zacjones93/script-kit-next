# Feature Bundle 26: Vercel AI Gateway Provider Implementation

## Goal

Create a new `VercelAiGatewayProvider` that implements the `AiProvider` trait, making Vercel AI Gateway the official/default AI backend for Script Kit.

## Vercel AI Gateway Overview

From [Vercel AI Gateway docs](https://vercel.com/docs/ai-gateway):
- **Unified API**: Single endpoint for hundreds of models (OpenAI, Anthropic, Google, etc.)
- **BYOK Support**: Bring your own API keys with 0% markup
- **Failover**: Automatic redirect to available providers on downtime
- **Low Latency**: <20ms request routing overhead
- **OpenAI-Compatible**: Works with existing OpenAI client libraries

## Current Architecture

Script Kit uses a **trait-based provider abstraction**:

```rust
pub trait AiProvider: Send + Sync {
    fn name(&self) -> &str;
    fn display_name(&self) -> &str;
    fn available_models(&self) -> Vec<ModelInfo>;
    fn send_message(&self, messages: &[ProviderMessage], model_id: &str) -> Result<String>;
    fn stream_message(&self, messages: &[ProviderMessage], model_id: &str, on_chunk: Box<dyn Fn(String) + Send>) -> Result<()>;
    fn supports_model(&self, model_id: &str) -> bool;
}
```

Existing providers: `OpenAiProvider`, `AnthropicProvider` (both use `ureq` HTTP client).

## Proposed Implementation

### New File: `src/ai/providers_vercel.rs`

```rust
use super::{AiProvider, ModelInfo, ProviderConfig, ProviderMessage};
use anyhow::{Context, Result};
use std::io::{BufRead, BufReader};

const VERCEL_GATEWAY_URL: &str = "https://api.vercel.com/v1/ai-gateway";

pub struct VercelAiGatewayProvider {
    config: ProviderConfig,
    gateway_url: String,
}

impl VercelAiGatewayProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            config: ProviderConfig::new("vercel", "Vercel AI Gateway", api_key),
            gateway_url: VERCEL_GATEWAY_URL.to_string(),
        }
    }

    pub fn with_custom_url(api_key: String, url: String) -> Self {
        Self {
            config: ProviderConfig::new("vercel", "Vercel AI Gateway", api_key),
            gateway_url: url,
        }
    }
}

impl AiProvider for VercelAiGatewayProvider {
    // Use OpenAI-compatible format for requests
    // Vercel routes to appropriate provider based on model ID
}
```

## Key Questions

1. **Default Provider**: Should Vercel AI Gateway be the DEFAULT provider when its API key is set, or should users explicitly select it?

2. **Model Routing**: Vercel routes based on model ID (e.g., `gpt-4o` → OpenAI, `claude-3-5-sonnet` → Anthropic). Should we:
   - Use Vercel's model IDs directly?
   - Add a prefix like `vercel/gpt-4o`?
   - Maintain a mapping table?

3. **Fallback Behavior**: If Vercel Gateway is unavailable, should we:
   - Fail immediately?
   - Fall back to direct provider APIs (if keys available)?
   - Show user a choice?

4. **Streaming Format**: Vercel passes through provider-specific streaming formats. Should we:
   - Detect provider from model ID and use appropriate parser?
   - Use only OpenAI-compatible format?
   - Request Vercel to normalize responses?

5. **Error Handling**: Vercel has specific error codes (rate limiting, quota, failover). How should we surface these to users?

## Implementation Checklist

- [ ] Create `src/ai/providers_vercel.rs`
- [ ] Implement `AiProvider` trait for `VercelAiGatewayProvider`
- [ ] Add `SCRIPT_KIT_VERCEL_API_KEY` env var detection
- [ ] Register provider in `ProviderRegistry::from_environment()`
- [ ] Update model list with Vercel-available models
- [ ] Test streaming with multiple model types
- [ ] Add Vercel-specific error handling
- [ ] Document setup in README

