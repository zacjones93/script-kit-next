# Feature Bundle 28: Vercel AI Gateway Anthropic-Compatible API

## Goal

Use Vercel's [Anthropic-Compatible API](https://vercel.com/docs/ai-gateway/anthropic-compat) to access Claude models through Script Kit's existing Anthropic provider with minimal changes.

## Vercel Anthropic-Compatible Endpoint

From Vercel docs:
- **Endpoint**: `https://api.vercel.com/v1/ai-gateway/anthropic/messages`
- **Auth**: `x-api-key: {VERCEL_API_KEY}` (Anthropic style)
- **Format**: Standard Anthropic messages API format
- **Streaming**: SSE with `content_block_delta` events

## Current Anthropic Implementation

Script Kit's `AnthropicProvider`:
```rust
const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

// Request format
{
    "model": "claude-3-5-sonnet-20241022",
    "max_tokens": 4096,
    "system": "System prompt here",
    "messages": [{"role": "user", "content": "Hello"}],
    "stream": true
}

// Headers
"anthropic-version": "2023-06-01"
"x-api-key": {api_key}

// Streaming parse
fn parse_sse_line(line: &str) -> Option<String> {
    // type == "content_block_delta" → delta.text
}
```

## Integration Strategy

### Minimal Change Approach
```rust
impl AnthropicProvider {
    pub fn with_vercel_gateway(vercel_api_key: String) -> Self {
        Self {
            config: ProviderConfig::new("anthropic", "Anthropic via Vercel", vercel_api_key),
            base_url: "https://api.vercel.com/v1/ai-gateway/anthropic".to_string(),
        }
    }
}
```

### Header Considerations
Vercel may need:
- Same `anthropic-version` header
- Different auth header format?
- Additional Vercel-specific headers?

## Request Flow

```
User Message → Script Kit → Vercel Gateway → Anthropic API → Response
                   ↓
           x-api-key: {VERCEL_KEY}
           anthropic-version: 2023-06-01
           POST /v1/ai-gateway/anthropic/messages
```

## Key Questions

1. **Auth Header**: Does Vercel use `x-api-key` or `Authorization: Bearer`?

2. **Version Header**: Is `anthropic-version: 2023-06-01` still required through Vercel?

3. **System Message Handling**: Does Vercel pass through the separate `system` field correctly?

4. **Streaming Format**: Does Vercel normalize to `content_block_delta` or pass through exactly?

5. **Model IDs**: Do we use full IDs (`claude-3-5-sonnet-20241022`) or short names?

## Claude Models via Vercel

Available through Anthropic-compatible API:
- `claude-3-5-sonnet-20241022` (Claude 3.5 Sonnet)
- `claude-3-5-haiku-20241022` (Claude 3.5 Haiku)
- `claude-3-opus-20240229` (Claude 3 Opus)
- `claude-sonnet-4` (Claude Sonnet 4 - if available)
- `claude-opus-4` (Claude Opus 4 - if available)

## Implementation Checklist

- [ ] Verify exact Vercel Anthropic endpoint URL
- [ ] Test auth header requirements
- [ ] Confirm streaming format compatibility
- [ ] Test system message handling
- [ ] Verify max_tokens passthrough
- [ ] Handle Vercel error responses
- [ ] Update model list with current Claude models

