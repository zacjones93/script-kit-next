# Expert Review: Extensions System Plan

## Context

I'm building Script Kit, a launcher/automation tool for macOS (similar to Raycast/Alfred). We're renaming our "scriptlets" system to "extensions" and aligning with Raycast's manifest format so users can easily port Raycast extensions to Script Kit.

## What I Need Reviewed

### 1. Manifest Field Mapping
Is our mapping of Raycast manifest fields complete? Are we missing any critical fields for porting extensions?

### 2. Extension Bundle Format
We're using markdown files with YAML frontmatter for extension bundles. Each H2 section is a command with codefence code blocks. Does this format make sense for portability?

### 3. Example Extensions
- **CleanShot**: Simple URL scheme integration (`cleanshot://capture-area`)
- **Chrome**: More complex with SQLite queries, AppleScript, and JSON parsing

Are these good proof-of-concept examples? Any obvious missing commands?

### 4. Terminology Migration
`Scriptlet` → `Extension`, `snippets/` → `extensions/`. Is this naming clearer?

### 5. Open Questions
- Preference storage: JSON per extension or SQLite?
- Icon format: Names only or also file paths?
- How to handle `min_version` checking?

---

## Bundled Code Context

The following files are included for reference:
- `plan/EXTENSIONS_PLAN.md` - Full plan with implementation details
- `src/scriptlets.rs` - Current scriptlet parsing/execution
- `src/scriptlet_metadata.rs` - Codefence metadata parsing
- `src/metadata_parser.rs` - TypedMetadata struct
- `src/scriptlet_cache.rs` - Caching layer

---

## File: plan/EXTENSIONS_PLAN.md

```markdown
# Extensions System Plan

> Rename "Scriptlets" → "Extensions" with Raycast-compatible manifest fields

## Overview

This plan covers:
1. **Terminology refactor**: Rename all "scriptlet" references to "extension"
2. **Manifest alignment**: Add Raycast-compatible fields for easy porting
3. **Example extensions**: CleanShot and Chrome extensions as proof-of-concept

## Current State

### Existing Implementation

| Component | Location | Status |
|-----------|----------|--------|
| Scriptlet parsing | `src/scriptlets.rs` | ✅ Complete |
| Codefence metadata | `src/scriptlet_metadata.rs` | ✅ Complete |
| Bundle frontmatter | `src/scriptlets.rs:66-78` | ✅ Basic (name, description, author, icon) |
| Typed metadata | `src/metadata_parser.rs` | ✅ Complete |
| Schema parsing | `src/schema_parser.rs` | ✅ Complete |
| Cache layer | `src/scriptlet_cache.rs` | ✅ Complete |

### Current Frontmatter Fields (BundleFrontmatter)

```rust
pub struct BundleFrontmatter {
    pub name: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub icon: Option<String>,
    pub extra: HashMap<String, serde_yaml::Value>,  // Catch-all
}
```

## Phase 1: Raycast Manifest Alignment

### Required Fields for Raycast Parity

Based on Raycast manifest documentation:

#### Extension-Level (Bundle Frontmatter)

| Raycast Field | Type | Required | Script Kit Mapping | Status |
|---------------|------|----------|-------------------|--------|
| `name` | string | Yes | `name` | ✅ Have |
| `title` | string | Yes | `title` (display name) | ❌ Need |
| `description` | string | Yes | `description` | ✅ Have |
| `icon` | string | Yes | `icon` | ✅ Have |
| `author` | string | Yes | `author` | ✅ Have |
| `license` | string | Yes | `license` | ❌ Need |
| `categories` | string[] | Yes | `categories` | ❌ Need |
| `keywords` | string[] | No | `keywords` | ❌ Need |
| `contributors` | string[] | No | `contributors` | ❌ Need |

### New ExtensionManifest Structure

```rust
/// Extension bundle metadata (YAML frontmatter)
/// Compatible with Raycast manifest for easy porting
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionManifest {
    // === Required for publishing ===
    pub name: String,
    pub title: String,
    pub description: String,
    pub icon: String,
    pub author: String,
    #[serde(default = "default_license")]
    pub license: String,
    #[serde(default)]
    pub categories: Vec<String>,
    
    // === Optional ===
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub contributors: Vec<String>,
    pub version: Option<String>,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    #[serde(default)]
    pub preferences: Vec<Preference>,
    
    // === Script Kit specific ===
    #[serde(default)]
    pub permissions: Vec<String>,
    pub min_version: Option<String>,
    
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}
```

### Valid Categories

```rust
pub const VALID_CATEGORIES: &[&str] = &[
    "Applications", "Communication", "Data", "Design Tools",
    "Developer Tools", "Documentation", "Finance", "Fun",
    "Media", "News", "Productivity", "Security", "System", "Web", "Other",
];
```

## Phase 2: Terminology Refactor

### File Renames

| Current | New |
|---------|-----|
| `src/scriptlets.rs` | `src/extensions.rs` |
| `src/scriptlet_metadata.rs` | `src/extension_metadata.rs` |
| `src/scriptlet_cache.rs` | `src/extension_cache.rs` |
| `~/.scriptkit/snippets/` | `~/.scriptkit/extensions/` |

### Struct/Type Renames

| Current | New |
|---------|-----|
| `Scriptlet` | `Extension` |
| `ScriptletMetadata` | `ExtensionMetadata` |
| `BundleFrontmatter` | `ExtensionManifest` |

## Phase 3: Example Extensions

### 3.1 CleanShot Extension

```yaml
---
name: cleanshot
title: CleanShot X
description: Capture screenshots, recordings, and annotations with CleanShot X
icon: camera
author: scriptkit
license: MIT
categories:
  - Productivity
  - Media
---
```

**Commands** (15 total):
- Capture Area, Fullscreen, Window, Previous Area
- Record Screen, Record GIF
- Scrolling Capture, Capture Text (OCR)
- Toggle Desktop Icons, Open from Clipboard
- All-In-One, Pin Screenshot, Open History
- Restore Recently Closed, Self Timer

All use simple `open` tool with URL schemes like `cleanshot://capture-area`

### 3.2 Chrome Extension

```yaml
---
name: chrome
title: Google Chrome
description: Search bookmarks, history, tabs, and control Chrome
icon: chrome
author: scriptkit
license: MIT
categories:
  - Applications
  - Web
---
```

**Commands** (6 total):
- Search Bookmarks (JSON parsing)
- Search History (SQLite via bun:sqlite)
- Search Open Tabs (AppleScript)
- New Tab, New Incognito Window, Close Current Tab (AppleScript)

## Implementation Tasks

### Priority: High
- [ ] Add new fields to `ExtensionManifest`
- [ ] Rename files and types
- [ ] Update all imports

### Priority: Medium
- [ ] Add `~/.scriptkit/extensions/` path support
- [ ] Create example extensions
- [ ] Migration from `snippets/` → `extensions/`

### Priority: Low (Future)
- [ ] Preference storage system
- [ ] `getPreference(name)` SDK function

## Open Questions

1. **Preference storage format**: JSON file per extension vs single SQLite DB?
2. **Icon format**: Support both icon names and file paths?
3. **Version checking**: How to handle `min_version` requirement?
4. **Extension registry**: Future plans for community extensions?
```

---

## File: src/scriptlets.rs (key structures)

```rust
/// Valid tool types that can be used in code fences
pub const VALID_TOOLS: &[&str] = &[
    "bash", "python", "kit", "ts", "js", "transform", "template",
    "open", "edit", "paste", "type", "submit", "applescript",
    "ruby", "perl", "php", "node", "deno", "bun",
    "zsh", "sh", "fish", "cmd", "powershell", "pwsh",
];

/// Frontmatter metadata for a scriptlet bundle (markdown file)
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct BundleFrontmatter {
    pub name: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub icon: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}

/// Metadata extracted from HTML comments in scriptlets
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ScriptletMetadata {
    pub trigger: Option<String>,
    pub shortcut: Option<String>,
    pub cron: Option<String>,
    pub schedule: Option<String>,
    pub background: Option<bool>,
    pub watch: Option<String>,
    pub system: Option<String>,
    pub description: Option<String>,
    pub expand: Option<String>,
    pub alias: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}

/// A scriptlet parsed from a markdown file
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Scriptlet {
    pub name: String,
    pub command: String,
    pub tool: String,
    pub scriptlet_content: String,
    pub inputs: Vec<String>,
    pub group: String,
    pub preview: Option<String>,
    pub metadata: ScriptletMetadata,
    pub typed_metadata: Option<TypedMetadata>,
    pub schema: Option<Schema>,
    pub kit: Option<String>,
    pub source_path: Option<String>,
}
```

---

## File: src/metadata_parser.rs (TypedMetadata)

```rust
/// Typed metadata extracted from a `metadata = { ... }` global declaration
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TypedMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub enter: Option<String>,
    pub alias: Option<String>,
    pub icon: Option<String>,
    pub shortcut: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub hidden: bool,
    pub placeholder: Option<String>,
    pub cron: Option<String>,
    pub schedule: Option<String>,
    #[serde(default)]
    pub watch: Vec<String>,
    #[serde(default)]
    pub background: bool,
    #[serde(default)]
    pub system: bool,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}
```

---

## File: src/scriptlet_metadata.rs (Codefence parsing)

```rust
/// Result of parsing codefence metadata from a scriptlet
#[derive(Debug, Clone, Default)]
pub struct CodefenceParseResult {
    /// Parsed metadata from ```metadata block
    pub metadata: Option<TypedMetadata>,
    /// Parsed schema from ```schema block
    pub schema: Option<Schema>,
    /// The code content from the main code block (e.g., ```ts)
    pub code: Option<CodeBlock>,
    /// Parse errors encountered
    pub errors: Vec<String>,
}

/// A code block with its language and content
#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub language: String,
    pub content: String,
}

/// Parse codefence blocks from markdown scriptlet content
/// Looks for:
/// - ```metadata\n{...}\n``` - JSON metadata block
/// - ```schema\n{...}\n``` - JSON schema block  
/// - ```<lang>\n...\n``` - Main code block
pub fn parse_codefence_metadata(content: &str) -> CodefenceParseResult { ... }
```

---

## File: src/scriptlet_cache.rs (summary)

The cache layer provides:
- In-memory caching of parsed scriptlets
- File watcher integration for hot reload
- Frecency-based sorting for recently used items
- Thread-safe access via `Arc<Mutex<...>>`
