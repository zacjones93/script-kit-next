# Feature Bundle 33: Deep Links & URL Scheme Improvements

## Goal

Implement full deep link support with intuitive, inferrable URL patterns that enable external tools, browser extensions, and other apps to trigger Script Kit actions.

## Current State

### What's Declared
```toml
# Cargo.toml - URL scheme registered
osx_url_schemes = ["scriptkit"]
```

```rust
// src/config/types.rs - Format defined
pub fn command_id_to_deeplink(command_id: &str) -> String {
    format!("kit://commands/{}", command_id)
}
```

### What's NOT Implemented
- No URL handler callback in app startup
- No URL parsing/routing logic
- No parameter support
- No validation or error handling
- The scheme is registered but NEVER listened for

## Proposed URL Scheme Design

### Base URL Patterns
```
scriptkit://run/{script-name}
scriptkit://run/{script-name}?arg1=value&arg2=value
scriptkit://command/{built-in-command}
scriptkit://action/{action-name}
scriptkit://search?q={query}
scriptkit://snippet/{snippet-id}
scriptkit://open                    # Just open Script Kit
scriptkit://hide
scriptkit://settings
scriptkit://settings/{section}
```

### Examples
```bash
# Run a script
open "scriptkit://run/my-script"
open "scriptkit://run/my-script?input=hello&silent=true"

# Built-in commands
open "scriptkit://command/clipboard-history"
open "scriptkit://command/app-launcher"
open "scriptkit://command/window-switcher"

# Search with prefilled query
open "scriptkit://search?q=git"

# Open settings
open "scriptkit://settings/hotkeys"
open "scriptkit://settings/ai"

# Quick actions
open "scriptkit://snippet/expand-date"
open "scriptkit://action/paste-clipboard"
```

### URL Parameters
```
?input={value}      - Pre-fill input field
?silent=true        - Run without showing UI
?focus=true         - Bring window to front
?timeout={ms}       - Auto-close after timeout
?callback={url}     - URL to open with result
?x-success={url}    - x-callback-url success
?x-error={url}      - x-callback-url error
?x-cancel={url}     - x-callback-url cancel
```

## Inferrable Patterns

### Script Name Resolution
```
scriptkit://run/my-script     → ~/.sk/kit/scripts/my-script.ts
scriptkit://run/My Script     → ~/.sk/kit/scripts/my-script.ts (slug)
scriptkit://run/my-script.ts  → ~/.sk/kit/scripts/my-script.ts
scriptkit://run/utils/helper  → ~/.sk/kit/scripts/utils/helper.ts
```

### Alias Support
```typescript
// ~/.sk/kit/config.ts
export default {
  deepLinkAliases: {
    "todo": "add-todo",
    "note": "quick-note",
    "timer": "pomodoro-timer",
  }
}
```

### Built-in Shorthand
```
scriptkit://clip    → scriptkit://command/clipboard-history
scriptkit://apps    → scriptkit://command/app-launcher
scriptkit://win     → scriptkit://command/window-switcher
```

## Implementation Architecture

### 1. URL Handler Registration (macOS)
```rust
// In app startup
fn register_url_handler(cx: &mut AppContext) {
    cx.on_open_urls(move |urls, cx| {
        for url in urls {
            handle_deep_link(&url, cx);
        }
    });
}
```

### 2. URL Router
```rust
pub fn handle_deep_link(url: &str, cx: &mut AppContext) -> Result<()> {
    let parsed = Url::parse(url)?;

    match (parsed.host_str(), parsed.path_segments()) {
        (Some("run"), Some(segments)) => {
            let script_name = segments.collect::<Vec<_>>().join("/");
            let params = parsed.query_pairs().collect();
            run_script(&script_name, params, cx)
        }
        (Some("command"), Some(segments)) => {
            let command = segments.next().unwrap_or_default();
            run_builtin_command(command, cx)
        }
        // ...
    }
}
```

### 3. x-callback-url Support
```rust
// Allow external apps to get results
// scriptkit://run/my-script?x-success=myapp://callback
pub struct CallbackContext {
    success_url: Option<String>,
    error_url: Option<String>,
    cancel_url: Option<String>,
}
```

## Key Questions

1. **URL Authority**: Should it be `scriptkit://run/` or `scriptkit:run/`? (with/without slashes)

2. **Script Resolution**: How aggressive should fuzzy matching be?
   - Exact match only?
   - Case-insensitive?
   - Fuzzy match if no exact?

3. **Security**: Should there be a whitelist of scripts allowed via deep link?
   - All scripts allowed by default?
   - Explicit opt-in per script?
   - Prompt user on first call?

4. **Result Callback**: How should results be returned to x-callback-url callers?
   - JSON in URL parameter?
   - Write to pasteboard?
   - Temporary file?

5. **Browser Extension**: Should we provide a companion browser extension for triggering deep links?

## Implementation Checklist

- [ ] Add URL handler in app startup
- [ ] Implement URL parser/router
- [ ] Add script name resolution (fuzzy, aliases)
- [ ] Implement parameter passing to scripts
- [ ] Add x-callback-url support
- [ ] Create built-in command shortcuts
- [ ] Add security/whitelist options
- [ ] Document URL scheme for users
- [ ] Create browser extension (optional)

