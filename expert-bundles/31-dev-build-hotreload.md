# Feature Bundle 31: Dev Build & Hot Reload Improvements

## Goal

Improve the development experience with faster builds, smarter hot reloading, and better feedback loops.

## Current Implementation

### Build Script (`dev.sh`)
```bash
cargo watch -c -x run
```
- Simple cargo-watch setup
- Clears terminal on rebuild
- Rebuilds on any `.rs` file change

### Hot Reload Watchers (`src/watcher.rs` - 1579 lines)

| Watcher | Target | Debounce | Behavior |
|---------|--------|----------|----------|
| ConfigWatcher | `~/.sk/kit/config.ts` | 500ms | Reload config, re-register hotkeys |
| ThemeWatcher | `~/.sk/kit/theme.json` | 500ms | Reload theme across all windows |
| ScriptWatcher | `~/.sk/kit/scripts/` | 500ms | Reload script list |
| AppearanceWatcher | System dark/light | 2s poll | Update theme variant |

### Architecture Patterns

**Supervisor Loop with Backoff**:
```rust
// Exponential backoff: 100ms → 200ms → 400ms → ... → 30s max
loop {
    if let Err(e) = watch_loop() {
        backoff = std::cmp::min(backoff * 2, MAX_BACKOFF);
        sleep(Duration::from_millis(backoff));
    }
}
```

**Storm Coalescing**:
```rust
// If 200+ files change rapidly, emit FullReload instead of individual events
const STORM_THRESHOLD: usize = 200;
```

**Trailing-Edge Debounce**:
```rust
// Each new event resets the debounce timer
// Only fires after 500ms of quiet
```

## Gaps & Improvement Opportunities

### 1. No Asset Hot Reload
Currently NOT hot-reloaded:
- SVG icons
- Syntax highlighting themes
- Embedded resources

**Proposal**: Add AssetWatcher for `~/.sk/kit/assets/`

### 2. Polling-Based Appearance Detection
```rust
// Current: polls every 2 seconds
defaults read -g AppleInterfaceStyle
```

**Proposal**: Use NSDistributedNotificationCenter for instant detection:
```rust
// macOS native notification
"AppleInterfaceThemeChangedNotification"
```

### 3. No Incremental Compilation Hints
cargo-watch rebuilds everything on any change.

**Proposals**:
- Use `cargo-nextest` for faster test runs
- Add `--features dev` for debug-only code paths
- Profile-guided compilation settings

### 4. No Build Metrics Dashboard
No visibility into:
- Build times per module
- Hot reload latency
- Watcher event frequency

**Proposal**: Add build instrumentation logging

### 5. No Conditional Reloads
All script changes trigger full reload.

**Proposal**:
- Single file change → reload only that script
- Scriptlet change → reload only scriptlets
- Full reload only on schema/structural changes

## Key Questions

1. **Build Time Optimization**: Should we use split debuginfo, incremental compilation settings, or sccache?

2. **Hot Reload Granularity**: Should individual script changes reload just that script, or is full reload safer?

3. **Appearance Detection**: Is native NSDistributedNotificationCenter worth the complexity over polling?

4. **Asset Pipeline**: Should asset changes trigger Rust rebuild, or runtime reload?

5. **Dev UX**: Should we show a HUD notification on hot reload, or is silent reload preferred?

## Implementation Checklist

- [ ] Profile current build times by module
- [ ] Add asset watcher for SVG/themes
- [ ] Replace appearance polling with NSNotification
- [ ] Add build metrics logging
- [ ] Implement incremental script reload
- [ ] Consider sccache or mold linker
- [ ] Add `--features fast-dev` for dev-only opts

