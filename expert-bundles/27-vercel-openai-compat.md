# Feature Bundle 27: Vercel AI Gateway OpenAI-Compatible API

## Goal

Leverage Vercel's [OpenAI-Compatible API](https://vercel.com/docs/ai-gateway/openai-compat) to access all models through a single, familiar interface.

## Vercel OpenAI-Compatible Endpoint

From Vercel docs:
- **Endpoint**: `https://api.vercel.com/v1/ai-gateway/openai/chat/completions`
- **Auth**: `Authorization: Bearer {VERCEL_API_KEY}`
- **Format**: Standard OpenAI chat completions format
- **Streaming**: SSE with `data: {json}` format, ends with `data: [DONE]`
- **Function Calling**: Supported (same as OpenAI spec)

## Current OpenAI Implementation

Script Kit's `OpenAiProvider` already handles:
```rust
// Request format
{
    "model": "gpt-4o",
    "messages": [{"role": "user", "content": "Hello"}],
    "stream": true
}

// Streaming parse
fn parse_sse_line(line: &str) -> Option<String> {
    // Extract: choices[0].delta.content
    // End: "data: [DONE]"
}
```

## Integration Strategy

### Option A: Modify Existing OpenAI Provider
```rust
impl OpenAiProvider {
    pub fn with_vercel_gateway(vercel_api_key: String) -> Self {
        Self {
            config: ProviderConfig::new("openai", "OpenAI via Vercel", vercel_api_key),
            base_url: "https://api.vercel.com/v1/ai-gateway/openai".to_string(),
        }
    }
}
```

### Option B: Create Wrapper Provider
```rust
pub struct VercelOpenAiProvider {
    inner: OpenAiProvider,  // Reuse existing logic
}

impl VercelOpenAiProvider {
    pub fn new(api_key: String) -> Self {
        let mut inner = OpenAiProvider::new(api_key);
        inner.set_base_url("https://api.vercel.com/v1/ai-gateway/openai");
        Self { inner }
    }
}
```

## Request Flow

```
User Message → Script Kit → Vercel Gateway → OpenAI API → Response
                   ↓
           Authorization: Bearer {VERCEL_KEY}
           POST /v1/ai-gateway/openai/chat/completions
```

## Key Questions

1. **URL Structure**: Is the Vercel endpoint exactly OpenAI-compatible (`/v1/chat/completions`) or does it have a different path?

2. **Header Requirements**: Does Vercel need additional headers beyond `Authorization`?
   - `x-vercel-api-version`?
   - `x-gateway-model-override`?

3. **Model ID Mapping**: Does `gpt-4o` work directly, or do we need `openai/gpt-4o`?

4. **Rate Limiting**: How does Vercel communicate rate limits? Standard `429` + `Retry-After` header?

5. **Function Calling**: Is tool/function calling fully supported through the gateway?

## Models Available via OpenAI-Compatible API

From Vercel docs, these work with OpenAI format:
- `gpt-4o`, `gpt-4o-mini`, `gpt-4-turbo`
- `claude-3-5-sonnet`, `claude-3-5-haiku`, `claude-3-opus` (Anthropic)
- `gemini-1.5-pro`, `gemini-1.5-flash` (Google)
- Many more via model routing

## Implementation Checklist

- [ ] Verify exact Vercel OpenAI endpoint URL
- [ ] Test request/response format compatibility
- [ ] Confirm streaming SSE format matches
- [ ] Test function calling support
- [ ] Handle Vercel-specific errors (quota, failover)
- [ ] Update model list for Vercel-available models
- [ ] Add integration tests

