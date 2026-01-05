# Feature Bundle 35: Rust Libraries Audit

## Goal

Identify hand-written code that could be replaced with battle-tested community crates, and find missing libraries that would improve reliability and maintainability.

## Current Dependencies (Cargo.toml)

### Core Framework
| Crate | Version | Purpose | Assessment |
|-------|---------|---------|------------|
| `gpui` | git | UI framework | ✅ Correct choice |
| `anyhow` | 1.0 | Error handling | ✅ Standard |
| `thiserror` | 1.0 | Error types | ✅ Standard |
| `serde` | 1.0 | Serialization | ✅ Standard |

### Actively Used
| Crate | Version | Purpose | Assessment |
|-------|---------|---------|------------|
| `nucleo-matcher` | 0.3 | Fuzzy matching | ✅ Excellent (10-100x faster) |
| `syntect` | 5.2 | Syntax highlighting | ✅ Industry standard |
| `rusqlite` | 0.31 | SQLite | ✅ Best choice |
| `ureq` | 3.0 | HTTP client | ✅ Good for sync |
| `notify` | 6.1 | File watching | ✅ Standard |
| `global-hotkey` | 0.7 | Hotkeys | ✅ Works well |
| `tray-icon` | 0.21 | System tray | ✅ Works well |

### Underutilized
| Crate | Version | Purpose | Current Usage |
|-------|---------|---------|---------------|
| `sysinfo` | 0.33 | System info | ❌ In deps, NOT used for battery/power |
| `uuid` | 1.0 | UUIDs | ⚠️ Used minimally |
| `regex` | 1.0 | Patterns | ⚠️ Could replace some manual parsing |

## Hand-Written Code → Crate Alternatives

### 1. Template Variable System
**Current**: `src/template_variables.rs` (500+ lines)
- Custom `{{date}}`, `{{clipboard}}` substitution

**Better Crates**:
| Crate | Stars | Features |
|-------|-------|----------|
| `tera` | 3k+ | Jinja2-style, filters, inheritance |
| `handlebars` | 1.2k | Mustache-compatible, helpers |
| `minijinja` | 1k+ | Jinja2, excellent errors |

**Recommendation**: `minijinja` - lightweight, great error messages

### 2. Snippet Parser (VSCode Format)
**Current**: `src/snippet.rs` (400+ lines)
- Parses `${1:placeholder}`, `${1|choice1,choice2|}` syntax

**Better Crates**:
| Crate | Notes |
|-------|-------|
| `lsp-snippet` | LSP snippet format parser |
| `tree-sitter` | Could parse snippet grammar |

**Recommendation**: Keep custom (VSCode format is specific enough)

### 3. Menu Bar Reader
**Current**: `src/menu_bar.rs` (700+ lines)
- Raw AXUIElement API calls

**Better Crates**:
| Crate | Notes |
|-------|-------|
| `accessibility` | Rust AX bindings (limited) |
| `objc2` | Modern objc bindings |

**Recommendation**: Extract to separate crate, contribute to ecosystem

### 4. Clipboard Management
**Current**: `src/clipboard_history/` (2000+ lines)
- Custom polling, SQLite storage, image handling

**Better Crates**:
| Crate | Stars | Features |
|-------|-------|----------|
| `arboard` | 500+ | Cross-platform clipboard |
| `clipboard-rs` | 100+ | macOS native |

**Recommendation**: Keep custom (history + images is specialized)

### 5. JSON Config Parsing
**Current**: Manual serde_json for config.ts output

**Better Crates**:
| Crate | Features |
|-------|----------|
| `config` | Multi-source config merging |
| `figment` | Type-safe config extraction |

**Recommendation**: `figment` - would unify env vars + file config

### 6. Exponential Backoff
**Current**: Hand-rolled in `watcher.rs`

**Better Crate**: `backoff` (standard exponential backoff)

### 7. Debouncing
**Current**: Manual debounce in watcher.rs

**Better Crate**: `debounce` or `governor` for rate limiting

## Missing Libraries

### System Integration
| Need | Recommended Crate | Notes |
|------|-------------------|-------|
| Power management | `battery` | Cross-platform battery info |
| Sleep/wake events | (none good) | IOKit FFI needed |
| Network changes | `netlink-packet-route` | Linux only; macOS needs SCNetwork |

### Performance & Observability
| Need | Recommended Crate | Notes |
|------|-------------------|-------|
| Profiling | `pprof` | CPU profiling |
| Tracing spans | `tracing` | Already have, use more |
| Metrics | `metrics` | Counters, gauges, histograms |

### Testing
| Need | Recommended Crate | Notes |
|------|-------------------|-------|
| Property testing | `proptest` | Generate test cases |
| Snapshot testing | `insta` | UI/output snapshots |
| Mocking | `mockall` | Mock traits |

### Async (if migrating from ureq)
| Need | Recommended Crate | Notes |
|------|-------------------|-------|
| HTTP client | `reqwest` | Async, full-featured |
| Runtime | `tokio` | Standard async runtime |

## Specific Recommendations

### High Priority (Reduce Bugs)
1. **Add `backoff`** - Replace hand-rolled exponential backoff
2. **Add `battery`** - Use sysinfo alternative for battery monitoring
3. **Add `insta`** - Snapshot test the UI/output

### Medium Priority (Reduce Code)
4. **Add `minijinja`** - Replace template_variables.rs
5. **Add `figment`** - Unify config loading
6. **Use `sysinfo` fully** - Already in deps, add power monitoring

### Low Priority (Future)
7. **Consider `reqwest`** - If needing async HTTP
8. **Add `proptest`** - For parser testing
9. **Extract menu_bar.rs** - Contribute as separate crate

## Key Questions

1. **Template Engine**: Is `minijinja` worth the dependency for ~500 lines saved?

2. **Config Unification**: Should env vars + config.ts use `figment`?

3. **Testing Investment**: Is snapshot testing (`insta`) worth adding?

4. **Async Migration**: Should AI provider HTTP calls use `reqwest` async?

5. **Menu Bar Extraction**: Should `menu_bar.rs` become its own crate?

## Implementation Checklist

- [ ] Add `backoff` crate, replace manual backoff
- [ ] Enable `sysinfo` battery monitoring
- [ ] Add `insta` for snapshot tests
- [ ] Evaluate `minijinja` for templates
- [ ] Evaluate `figment` for config
- [ ] Add `proptest` for parser tests
- [ ] Document dependency decisions in ARCHITECTURE.md

