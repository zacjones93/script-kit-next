# Feature Bundle 30: Vercel AI Gateway Configuration & BYOK

## Goal

Configure Vercel AI Gateway as the default AI backend with support for:
- Vercel API key (primary)
- Bring Your Own Key (BYOK) for direct provider access
- Gateway URL customization
- Usage tracking and budgets

## Current API Key System

```rust
pub mod env_vars {
    pub const OPENAI_API_KEY: &str = "SCRIPT_KIT_OPENAI_API_KEY";
    pub const ANTHROPIC_API_KEY: &str = "SCRIPT_KIT_ANTHROPIC_API_KEY";
    pub const GOOGLE_API_KEY: &str = "SCRIPT_KIT_GOOGLE_API_KEY";
    // ...
}

pub struct DetectedKeys {
    pub openai: Option<String>,
    pub anthropic: Option<String>,
    // ...
}
```

## Proposed Configuration

### Environment Variables
```bash
# Primary: Vercel AI Gateway
SCRIPT_KIT_VERCEL_API_KEY=vercel_xxxxx

# Optional: Custom gateway URL (for enterprise/self-hosted)
SCRIPT_KIT_VERCEL_GATEWAY_URL=https://custom.gateway.example.com

# BYOK: Pass your own keys through Vercel (0% markup)
SCRIPT_KIT_VERCEL_OPENAI_KEY=sk-xxxxx
SCRIPT_KIT_VERCEL_ANTHROPIC_KEY=sk-ant-xxxxx

# Fallback: Direct provider access (if Vercel unavailable)
SCRIPT_KIT_OPENAI_API_KEY=sk-xxxxx
SCRIPT_KIT_ANTHROPIC_API_KEY=sk-ant-xxxxx
```

### Config.ts Integration
```typescript
// ~/.sk/kit/config.ts
export default {
  ai: {
    // Primary provider
    provider: "vercel", // "vercel" | "openai" | "anthropic" | "auto"

    // Vercel-specific
    vercel: {
      apiKey: process.env.SCRIPT_KIT_VERCEL_API_KEY,
      gatewayUrl: "https://api.vercel.com/v1/ai-gateway", // optional override

      // BYOK keys (passed through Vercel with 0% markup)
      byok: {
        openai: process.env.SCRIPT_KIT_VERCEL_OPENAI_KEY,
        anthropic: process.env.SCRIPT_KIT_VERCEL_ANTHROPIC_KEY,
      }
    },

    // Default model
    defaultModel: "claude-3-5-sonnet",

    // Budget controls (Vercel feature)
    budget: {
      monthly: 50.00, // USD
      alertAt: 40.00, // Warn at 80%
    }
  }
} satisfies Config;
```

### Rust Configuration
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub provider: AiProviderType,
    pub vercel: Option<VercelConfig>,
    pub default_model: Option<String>,
    pub budget: Option<BudgetConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VercelConfig {
    pub api_key: String,
    pub gateway_url: Option<String>,
    pub byok_openai: Option<String>,
    pub byok_anthropic: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum AiProviderType {
    Vercel,    // Route through Vercel Gateway
    OpenAi,    // Direct to OpenAI
    Anthropic, // Direct to Anthropic
    Auto,      // Use Vercel if available, else direct
}
```

## Provider Priority Logic

```rust
impl ProviderRegistry {
    pub fn from_config(config: &AiConfig) -> Self {
        let mut registry = Self::new();

        match config.provider {
            AiProviderType::Vercel => {
                // Vercel as primary (routes to all providers)
                if let Some(vercel) = &config.vercel {
                    registry.register(Arc::new(
                        VercelAiGatewayProvider::new(vercel.clone())
                    ));
                }
            }
            AiProviderType::Auto => {
                // Try Vercel first, fall back to direct
                if let Some(vercel) = &config.vercel {
                    registry.register(Arc::new(
                        VercelAiGatewayProvider::new(vercel.clone())
                    ));
                }
                // Also register direct providers as fallback
                if let Ok(key) = env::var("SCRIPT_KIT_OPENAI_API_KEY") {
                    registry.register(Arc::new(OpenAiProvider::new(key)));
                }
            }
            // ...
        }

        registry
    }
}
```

## Key Questions

1. **Default Behavior**: Should Vercel be default when ONLY `SCRIPT_KIT_VERCEL_API_KEY` is set?

2. **BYOK Headers**: How does Vercel accept BYOK keys in requests?
   - Custom header: `x-vercel-byok-openai: sk-xxxxx`?
   - In request body?
   - Configured in Vercel dashboard?

3. **Fallback Strategy**: If Vercel is unavailable:
   - Use direct provider keys if available?
   - Show error and let user retry?
   - Queue messages for later?

4. **Budget Enforcement**: Does Vercel enforce budgets server-side, or should we track client-side too?

5. **Sponsorship Branding**: How should we indicate "Powered by Vercel AI Gateway"?
   - In model picker?
   - In chat footer?
   - On first launch?

## Setup Flow for New Users

1. **First Launch**: Prompt to set up AI
2. **Option A**: "Use Vercel AI Gateway (Recommended)"
   - Link to get Vercel API key
   - $5 free credits included
3. **Option B**: "Bring Your Own Keys"
   - Enter OpenAI/Anthropic keys directly
4. **Store keys** in `~/.sk/kit/.env` or system keychain
5. **Show sponsorship**: "AI powered by Vercel"

## Implementation Checklist

- [ ] Add `SCRIPT_KIT_VERCEL_API_KEY` detection
- [ ] Add `VercelConfig` to config types
- [ ] Update `ProviderRegistry::from_config()`
- [ ] Add BYOK header support
- [ ] Implement fallback logic
- [ ] Add budget tracking (if Vercel provides API)
- [ ] Create setup wizard for new users
- [ ] Add "Powered by Vercel" attribution
- [ ] Document all configuration options

