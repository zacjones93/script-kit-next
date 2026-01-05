# Feature Bundle 29: Vercel AI Gateway Model Discovery & Selection

## Goal

Dynamically fetch and display available models from Vercel AI Gateway instead of hardcoded lists. Enable unified model selection across all providers.

## Current Model System

Script Kit has hardcoded model lists:
```rust
pub mod default_models {
    pub fn openai() -> Vec<ModelInfo> {
        vec![
            ModelInfo::new("gpt-4o", "GPT-4o", "openai", true, 128_000),
            ModelInfo::new("gpt-4o-mini", "GPT-4o Mini", "openai", true, 128_000),
            // ...
        ]
    }

    pub fn anthropic() -> Vec<ModelInfo> {
        vec![
            ModelInfo::new("claude-3-5-sonnet-20241022", "Claude 3.5 Sonnet", "anthropic", true, 200_000),
            // ...
        ]
    }
}
```

## Vercel Model Capabilities

From [Models & Providers](https://vercel.com/docs/ai-gateway/models-and-providers):
- Access to hundreds of models from multiple providers
- Unified model IDs across providers
- Dynamic model availability
- Model routing and aliases

## Proposed Architecture

### 1. Model Info Endpoint
```rust
// Fetch available models from Vercel
pub async fn fetch_vercel_models(api_key: &str) -> Result<Vec<ModelInfo>> {
    let response = ureq::get("https://api.vercel.com/v1/ai-gateway/models")
        .header("Authorization", &format!("Bearer {}", api_key))
        .call()?;

    let models: VercelModelsResponse = response.into_body().read_json()?;
    Ok(models.into_model_info_list())
}
```

### 2. Model Categories
Group models for the picker UI:
```rust
pub enum ModelCategory {
    Featured,       // Top models (GPT-4o, Claude 3.5 Sonnet)
    OpenAI,         // All OpenAI models
    Anthropic,      // All Claude models
    Google,         // Gemini models
    OpenSource,     // Llama, Mistral, etc.
    Specialized,    // Code, vision, etc.
}
```

### 3. Model Picker UI Enhancement
```rust
// Current: flat list
// Proposed: categorized with search
pub struct ModelPicker {
    categories: Vec<ModelCategory>,
    search_query: String,
    selected_category: Option<ModelCategory>,
    models: Vec<ModelInfo>,
}
```

## Key Questions

1. **API Endpoint**: Does Vercel have a `/models` endpoint to list available models?

2. **Caching Strategy**: How often should we refresh the model list?
   - On app start?
   - Periodically (hourly)?
   - On user request?

3. **Offline Fallback**: If we can't reach Vercel, should we:
   - Use cached model list?
   - Fall back to hardcoded defaults?
   - Show error?

4. **Model Metadata**: What info does Vercel provide per model?
   - Context window size?
   - Streaming support?
   - Pricing tier?
   - Capabilities (vision, function calling)?

5. **Model Aliases**: Does Vercel support aliases like `latest` or `best`?
   - `gpt-4o-latest` → latest GPT-4o version
   - `claude-best` → best available Claude model

## Model Selection UX

### Current Flow
1. User opens AI chat
2. Hardcoded model list shown
3. User selects model
4. Chat uses that model

### Proposed Flow
1. User opens AI chat
2. Fetch models from Vercel (with loading state)
3. Show categorized, searchable list
4. Featured models at top
5. User selects model
6. Store selection for future chats

## Implementation Checklist

- [ ] Research Vercel's model listing API
- [ ] Create `fetch_vercel_models()` function
- [ ] Add model caching with TTL
- [ ] Update ModelPicker UI for categories
- [ ] Add search/filter to model picker
- [ ] Show model metadata (context window, capabilities)
- [ ] Handle offline/error states
- [ ] Remember user's preferred models

