# Extensions System Plan

> Rename "Scriptlets" → "Extensions" with Raycast-compatible manifest fields

## Overview

This plan covers:
1. **Terminology refactor**: Proper separation of Extension (bundle) vs Command (H2 section)
2. **Manifest alignment**: Add Raycast-compatible fields at BOTH extension and command levels
3. **Example extensions**: CleanShot and Chrome extensions as proof-of-concept

## Key Terminology (Corrected)

| Concept | Raycast Term | Script Kit Term | Definition |
|---------|--------------|-----------------|------------|
| Bundle/Package | Extension | Extension | The `.md` file with frontmatter |
| Runnable entry | Command | Command | Each H2 section within an extension |
| Bundle metadata | Manifest | ExtensionManifest | YAML frontmatter at file top |
| Command metadata | Command properties | CommandMetadata | Per-H2 metadata block |

**CRITICAL**: `Scriptlet` → `Command` (NOT `Extension`). An extension *contains* commands.

---

## Current State

### Existing Implementation

| Component | Location | Status |
|-----------|----------|--------|
| Scriptlet parsing | `src/scriptlets.rs` | ✅ Complete |
| Codefence metadata | `src/scriptlet_metadata.rs` | ✅ Complete |
| Bundle frontmatter | `src/scriptlets.rs:66-78` | ✅ Basic |
| Typed metadata | `src/metadata_parser.rs` | ✅ Complete |
| Schema parsing | `src/schema_parser.rs` | ✅ Complete |
| Cache layer | `src/scriptlet_cache.rs` | ✅ Complete |

---

## Phase 1: Raycast Manifest Alignment

### 1.1 Extension-Level Fields (Bundle Frontmatter)

Based on [Raycast manifest documentation](https://developers.raycast.com/information/manifest):

| Raycast Field | Type | Required | Script Kit Mapping | Status |
|---------------|------|----------|-------------------|--------|
| `name` | string | Yes | `name` | ✅ Have |
| `title` | string | Yes | `title` | ❌ Need |
| `description` | string | Yes | `description` | ✅ Have |
| `icon` | string | Yes | `icon` | ✅ Have |
| `author` | string | Yes | `author` | ✅ Have |
| `license` | string | Yes | `license` | ❌ Need |
| `categories` | string[] | Yes | `categories` | ❌ Need |
| `platforms` | string[] | Yes | `platforms` | ❌ Need (accept, warn if not macOS) |
| `keywords` | string[] | No | `keywords` | ❌ Need |
| `contributors` | string[] | No | `contributors` | ❌ Need |
| `pastContributors` | string[] | No | Accept in `extra` | ✅ Via flatten |
| `owner` | string | No | Accept in `extra` | ✅ Via flatten |
| `access` | string | No | Accept in `extra` | ✅ Via flatten |
| `commands` | array | Yes | Generated from H2s | ✅ Implicit |
| `tools` | array | No | Accept in `extra` | ✅ Via flatten |
| `ai` | object | No | Accept in `extra` | ✅ Via flatten |
| `external` | string[] | No | Accept in `extra` | ✅ Via flatten |
| `preferences` | array | No | `preferences` | ❌ Need |

### 1.2 Command-Level Fields (Per-H2 Metadata)

**This is the big gap we need to fill.** Each H2 section needs these fields:

| Raycast Field | Type | Required | Script Kit Mapping | Status |
|---------------|------|----------|-------------------|--------|
| `name` | string | Yes | `command` (slug from H2) | ✅ Have |
| `title` | string | Yes | H2 header text | ✅ Have |
| `description` | string | Yes | `description` in metadata | ✅ Have |
| `mode` | enum | Yes | Inferred from tool type | ✅ Implicit |
| `subtitle` | string | No | `subtitle` | ❌ Need |
| `icon` | string | No | `icon` | ✅ Have |
| `keywords` | string[] | No | `keywords` | ❌ Need |
| `interval` | string | No | `cron`/`schedule` | ✅ Have |
| `arguments` | array | No | `inputs` from `{{var}}` | ✅ Have |
| `preferences` | array | No | Command-level prefs | ❌ Need |
| `disabledByDefault` | bool | No | `disabled` | ❌ Need |

### 1.3 New Struct Definitions

```rust
/// Extension bundle metadata (YAML frontmatter)
/// Compatible with Raycast manifest for easy porting
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionManifest {
    // === Required for publishing ===
    /// Unique URL-safe identifier (e.g., "cleanshot")
    pub name: String,
    /// Display name shown in UI (e.g., "CleanShot X")
    pub title: String,
    /// Full description
    pub description: String,
    /// Icon path or icon name (supports both)
    pub icon: String,
    /// Author's handle/username
    pub author: String,
    /// License identifier (e.g., "MIT")
    #[serde(default = "default_license")]
    pub license: String,
    /// Categories for discovery
    #[serde(default)]
    pub categories: Vec<String>,
    /// Supported platforms (accept but warn if not macOS)
    #[serde(default)]
    pub platforms: Vec<String>,
    
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
    /// Minimum Script Kit version (semver)
    #[serde(alias = "min_version")]
    pub min_version: Option<String>,
    /// Schema version for future format evolution
    pub manifest_version: Option<u32>,
    
    /// Catch-all for unknown/future Raycast fields
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}

/// Command metadata (per-H2 section)
/// Mirrors Raycast command properties
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata {
    // === From H2 header ===
    /// Command slug (auto-generated from title)
    pub name: String,
    /// Display title (the H2 header text)  
    pub title: String,
    
    // === From metadata block ===
    pub description: Option<String>,
    pub subtitle: Option<String>,
    pub icon: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    
    /// Command mode: "view" (default), "no-view", "menu-bar"
    #[serde(default = "default_mode")]
    pub mode: CommandMode,
    
    /// Background interval (e.g., "1m", "1h", "1d")
    pub interval: Option<String>,
    /// Cron expression (Script Kit extension)
    pub cron: Option<String>,
    /// Natural language schedule (Script Kit extension)
    pub schedule: Option<String>,
    
    /// Typed arguments (up to 3 in Raycast)
    #[serde(default)]
    pub arguments: Vec<Argument>,
    
    /// Command-level preferences (override/extend extension prefs)
    #[serde(default)]
    pub preferences: Vec<Preference>,
    
    /// If true, user must enable manually
    #[serde(default)]
    pub disabled_by_default: bool,
    
    // === Script Kit extensions ===
    pub shortcut: Option<String>,
    pub alias: Option<String>,
    pub expand: Option<String>,
    #[serde(default)]
    pub hidden: bool,
    
    /// Catch-all for unknown fields
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum CommandMode {
    #[default]
    View,
    NoView,
    MenuBar,
}

fn default_mode() -> CommandMode {
    CommandMode::View
}

/// Typed argument definition (Raycast compatible)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Argument {
    pub name: String,
    #[serde(rename = "type")]
    pub arg_type: ArgumentType,
    pub placeholder: String,
    #[serde(default)]
    pub required: bool,
    /// For dropdown type
    #[serde(default)]
    pub data: Vec<PreferenceOption>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ArgumentType {
    Text,
    Password,
    Dropdown,
}

/// Preference definition (Raycast compatible)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Preference {
    pub name: String,
    pub title: String,
    pub description: String,
    #[serde(rename = "type")]
    pub pref_type: PreferenceType,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default: Option<serde_json::Value>,
    pub placeholder: Option<String>,
    /// Label for checkbox type
    pub label: Option<String>,
    /// Options for dropdown type
    #[serde(default)]
    pub data: Vec<PreferenceOption>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PreferenceType {
    Textfield,
    Password,
    Checkbox,
    Dropdown,
    #[serde(rename = "appPicker")]
    AppPicker,
    File,
    Directory,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PreferenceOption {
    pub title: String,
    pub value: String,
}

fn default_license() -> String {
    "MIT".to_string()
}
```

### 1.4 Valid Categories

```rust
pub const VALID_CATEGORIES: &[&str] = &[
    "Applications",
    "Communication", 
    "Data",
    "Design Tools",
    "Developer Tools",
    "Documentation",
    "Finance",
    "Fun",
    "Media",
    "News",
    "Productivity",
    "Security",
    "System",
    "Web",
    "Other",
];
```

---

## Phase 2: Terminology Refactor

### File Renames

| Current | New |
|---------|-----|
| `src/scriptlets.rs` | `src/extensions.rs` |
| `src/scriptlet_metadata.rs` | `src/extension_metadata.rs` |
| `src/scriptlet_cache.rs` | `src/extension_cache.rs` |
| `src/scriptlet_tests.rs` | `src/extension_tests.rs` |

### Directory Renames

| Current | New |
|---------|-----|
| `~/.sk/kit/snippets/` | `~/.sk/kit/extensions/` |

### Struct/Type Renames (CORRECTED)

| Current | New | Raycast Equivalent |
|---------|-----|-------------------|
| `Scriptlet` | `Command` | Command |
| `ScriptletMetadata` | `CommandMetadata` | Command properties |
| `ScriptletMatch` | `CommandMatch` | - |
| `BundleFrontmatter` | `ExtensionManifest` | Manifest |
| `ScriptletValidationError` | `CommandValidationError` | - |
| `ScriptletParseResult` | `ExtensionParseResult` | - |

### Function Renames

| Current | New |
|---------|-----|
| `load_scriptlets()` | `load_extensions()` |
| `parse_scriptlet_section()` | `parse_command()` |
| `fuzzy_search_scriptlets()` | `fuzzy_search_commands()` |
| `parse_markdown_as_scriptlets()` | `parse_extension()` |

### Backward Compatibility

```rust
// Deprecated aliases
#[deprecated(since = "2.0.0", note = "Use Command instead")]
pub type Scriptlet = Command;

#[deprecated(since = "2.0.0", note = "Use CommandMetadata instead")]  
pub type ScriptletMetadata = CommandMetadata;
```

---

## Phase 3: Example Extensions

### Location

```
~/.sk/kit/examples/extensions/
├── cleanshot.md        # CleanShot X integration
├── chrome.md           # Chrome browser integration  
└── README.md           # Examples documentation
```

### 3.1 CleanShot Extension

```markdown
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
keywords:
  - screenshot
  - screen recording
  - annotation
  - capture
platforms:
  - macOS
---

# CleanShot X

## Capture Area

```metadata
{ "description": "Capture a selected area of the screen", "mode": "no-view" }
```

```open
cleanshot://capture-area
```

## Capture Fullscreen

```metadata
{ "description": "Capture the entire screen", "mode": "no-view" }
```

```open
cleanshot://capture-fullscreen
```

## Capture Window

```metadata
{ "description": "Capture a specific window", "mode": "no-view" }
```

```open
cleanshot://capture-window
```

## Record Screen

```metadata
{ "description": "Start a screen recording", "mode": "no-view" }
```

```open
cleanshot://record-screen
```

## Record GIF

```metadata
{ "description": "Record screen as animated GIF", "mode": "no-view" }
```

```open
cleanshot://record-gif
```

## Scrolling Capture

```metadata
{ "description": "Capture scrolling content", "mode": "no-view" }
```

```open
cleanshot://scrolling-capture
```

## Capture Text (OCR)

```metadata
{ "description": "Capture and extract text from screen", "mode": "no-view" }
```

```open
cleanshot://capture-text
```

## Toggle Desktop Icons

```metadata
{ "description": "Show or hide desktop icons", "mode": "no-view" }
```

```open
cleanshot://toggle-desktop-icons
```

## Open from Clipboard

```metadata
{ "description": "Open image from clipboard in editor", "mode": "no-view" }
```

```open
cleanshot://open-from-clipboard
```

## All-In-One

```metadata
{ "description": "Open all capture options overlay", "mode": "no-view" }
```

```open
cleanshot://all-in-one
```

## Pin Screenshot

```metadata
{ "description": "Pin a screenshot to screen", "mode": "no-view" }
```

```open
cleanshot://pin-screenshot
```

## Open History

```metadata
{ "description": "Open CleanShot capture history", "mode": "view" }
```

```open
cleanshot://open-history
```

## Restore Recently Closed

```metadata
{ "description": "Restore last closed capture", "mode": "no-view" }
```

```open
cleanshot://restore-recently-closed
```

## Self Timer

```metadata
{ "description": "Capture with countdown timer", "mode": "no-view" }
```

```open
cleanshot://self-timer
```

## Capture Previous Area

```metadata
{ "description": "Capture the same area as last time", "mode": "no-view" }
```

```open
cleanshot://capture-previous-area
```
```

### 3.2 Chrome Extension

```markdown
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
keywords:
  - browser
  - bookmarks
  - history
  - tabs
platforms:
  - macOS
preferences:
  - name: profile
    title: Chrome Profile
    description: Which Chrome profile to use
    type: dropdown
    required: false
    default: Default
    data:
      - title: Default
        value: Default
      - title: Profile 1
        value: "Profile 1"
      - title: Profile 2
        value: "Profile 2"
---

# Google Chrome

## Search Bookmarks

```metadata
{ 
  "description": "Search and open Chrome bookmarks",
  "mode": "view",
  "keywords": ["favorites", "saved"]
}
```

```ts
import { readFileSync, existsSync } from 'fs';
import { homedir } from 'os';
import { join } from 'path';

// Get profile from preferences (default to "Default")
const profile = await getPreference("profile") || "Default";

const bookmarksPath = join(
  homedir(),
  `Library/Application Support/Google/Chrome/${profile}/Bookmarks`
);

interface BookmarkNode {
  name: string;
  url?: string;
  children?: BookmarkNode[];
  type: 'url' | 'folder';
}

interface BookmarksFile {
  roots: {
    bookmark_bar: BookmarkNode;
    other: BookmarkNode;
    synced: BookmarkNode;
  };
}

function flattenBookmarks(node: BookmarkNode, path: string[] = []): { name: string; url: string; path: string }[] {
  const results: { name: string; url: string; path: string }[] = [];
  
  if (node.type === 'url' && node.url) {
    results.push({
      name: node.name,
      url: node.url,
      path: path.join(' > ')
    });
  }
  
  if (node.children) {
    for (const child of node.children) {
      results.push(...flattenBookmarks(child, [...path, node.name]));
    }
  }
  
  return results;
}

if (!existsSync(bookmarksPath)) {
  await div(`<div class="p-4 text-red-500">Chrome bookmarks not found. Is Chrome installed?</div>`);
  process.exit(1);
}

const bookmarksData: BookmarksFile = JSON.parse(readFileSync(bookmarksPath, 'utf-8'));
const allBookmarks = [
  ...flattenBookmarks(bookmarksData.roots.bookmark_bar),
  ...flattenBookmarks(bookmarksData.roots.other),
  ...flattenBookmarks(bookmarksData.roots.synced),
];

const selected = await arg('Search bookmarks', allBookmarks.map(b => ({
  name: b.name,
  description: b.path,
  value: b.url,
  preview: `<div class="p-4">
    <div class="font-bold">${b.name}</div>
    <div class="text-sm text-gray-500">${b.path}</div>
    <div class="text-xs text-blue-500 mt-2 break-all">${b.url}</div>
  </div>`
})));

await open(selected);
```

## Search History

```metadata
{ 
  "description": "Search Chrome browsing history",
  "mode": "view"
}
```

```ts
import { homedir } from 'os';
import { join } from 'path';
import { copyFileSync, unlinkSync, existsSync } from 'fs';
import Database from 'bun:sqlite';

const profile = await getPreference("profile") || "Default";

const historyPath = join(
  homedir(),
  `Library/Application Support/Google/Chrome/${profile}/History`
);

if (!existsSync(historyPath)) {
  await div(`<div class="p-4 text-red-500">Chrome history not found. Is Chrome installed?</div>`);
  process.exit(1);
}

// Chrome locks the database, so we need to copy it
const tempPath = '/tmp/chrome-history-copy.db';
copyFileSync(historyPath, tempPath);

const db = new Database(tempPath, { readonly: true });

const rows = db.query(`
  SELECT title, url, last_visit_time, visit_count
  FROM urls
  ORDER BY last_visit_time DESC
  LIMIT 1000
`).all() as { title: string; url: string; last_visit_time: number; visit_count: number }[];

db.close();
unlinkSync(tempPath);

// Chrome timestamps are microseconds since 1601-01-01
const chromeEpoch = 11644473600000000n;

const history = rows.map(row => {
  const timestamp = Number((BigInt(row.last_visit_time) - chromeEpoch) / 1000n);
  const date = new Date(timestamp);
  return {
    title: row.title || row.url,
    url: row.url,
    date: date.toLocaleDateString(),
    time: date.toLocaleTimeString(),
    visits: row.visit_count
  };
});

const selected = await arg('Search history', history.map(h => ({
  name: h.title,
  description: `${h.date} ${h.time} • ${h.visits} visits`,
  value: h.url
})));

await open(selected);
```

## Search Open Tabs

```metadata
{ 
  "description": "Search and switch to open Chrome tabs",
  "mode": "view",
  "keywords": ["switch", "window"]
}
```

```ts
const script = `
tell application "Google Chrome"
  set tabList to {}
  set windowIndex to 1
  repeat with w in windows
    set tabIndex to 1
    repeat with t in tabs of w
      set end of tabList to {title of t, URL of t, windowIndex, tabIndex}
      set tabIndex to tabIndex + 1
    end repeat
    set windowIndex to windowIndex + 1
  end repeat
  return tabList
end tell
`;

const result = await applescript(script);

// Parse AppleScript list result
const tabs = (result as string[][]).map(([title, url, windowIdx, tabIdx]) => ({
  title,
  url,
  windowIndex: parseInt(windowIdx),
  tabIndex: parseInt(tabIdx)
}));

const selected = await arg('Search tabs', tabs.map(t => ({
  name: t.title,
  description: t.url,
  value: t
})));

// Switch to selected tab
await applescript(`
tell application "Google Chrome"
  set active tab index of window ${selected.windowIndex} to ${selected.tabIndex}
  set index of window ${selected.windowIndex} to 1
  activate
end tell
`);
```

## New Tab

```metadata
{ "description": "Open a new Chrome tab", "mode": "no-view" }
```

```applescript
tell application "Google Chrome"
  activate
  tell front window
    make new tab
  end tell
end tell
```

## New Incognito Window

```metadata
{ "description": "Open a new Chrome incognito window", "mode": "no-view" }
```

```applescript
tell application "Google Chrome"
  activate
  make new window with properties {mode:"incognito"}
end tell
```

## Close Current Tab

```metadata
{ "description": "Close the current Chrome tab", "mode": "no-view" }
```

```applescript
tell application "Google Chrome"
  tell front window
    close active tab
  end tell
end tell
```
```

---

## Phase 4: Decisions (Based on Expert Feedback)

### 4.1 Preference Storage: JSON per Extension

**Decision**: Use JSON file per extension.

```
~/.sk/kit/preferences/
├── cleanshot.json
├── chrome.json
└── ...
```

**Rationale**:
- Preferences are small, keyed, rarely need relational queries
- JSON is portable, debuggable, easy to version/migrate
- SQLite is overkill for key-value prefs

**Secret handling**: Store passwords/API keys in macOS Keychain, not JSON.

### 4.2 Icon Format: Support Both Names and Paths

**Decision**: Accept both icon names and file paths.

**Resolution logic**:
```rust
fn resolve_icon(value: &str) -> IconSource {
    if value.starts_with("./") 
       || value.starts_with("/") 
       || value.contains("/")
       || value.ends_with(".png") 
       || value.ends_with(".svg") {
        IconSource::Path(value.to_string())
    } else {
        IconSource::Named(value.to_string())
    }
}
```

Command-level `icon` overrides extension-level `icon`.

### 4.3 minVersion Checking: Semver with Clear UX

**Decision**: Use semver parsing, disable extension if version too old.

```rust
use semver::{Version, VersionReq};

fn check_min_version(required: &str, current: &str) -> Result<(), String> {
    let req = VersionReq::parse(&format!(">={}", required))
        .map_err(|e| format!("Invalid minVersion: {}", e))?;
    let current = Version::parse(current)
        .map_err(|e| format!("Invalid current version: {}", e))?;
    
    if req.matches(&current) {
        Ok(())
    } else {
        Err(format!(
            "Extension requires Script Kit {} or newer (current: {})",
            required, current
        ))
    }
}
```

**UX**: Show error message pointing to upgrade, mark commands as unavailable.

### 4.4 Future: Folder Extension Bundles

For now: single-file `.md` extensions only.

Future (when we need assets/multi-file):
```
~/.sk/kit/extensions/
├── cleanshot.md              # Single-file extension
└── github/                   # Folder extension
    ├── manifest.yaml
    ├── commands/
    │   ├── search-issues.ts
    │   └── create-pr.ts
    └── assets/
        └── icon.png
```

---

## Phase 5: Implementation Tasks

### 5.1 Struct Updates (Priority: Critical)

- [ ] Add `CommandMetadata` struct with all Raycast command fields
- [ ] Update `ExtensionManifest` with missing extension-level fields  
- [ ] Add `Argument` struct for typed command arguments
- [ ] Add `Preference` struct matching Raycast schema
- [ ] Add `CommandMode` enum
- [ ] Update frontmatter parser
- [ ] Update codefence metadata parser to populate `CommandMetadata`

### 5.2 Terminology Refactor (Priority: High)

- [ ] Rename `Scriptlet` → `Command`
- [ ] Rename `ScriptletMetadata` → `CommandMetadata`
- [ ] Rename `BundleFrontmatter` → `ExtensionManifest`
- [ ] Rename files: `scriptlets.rs` → `extensions.rs`, etc.
- [ ] Update all imports
- [ ] Add deprecated type aliases
- [ ] Update tests

### 5.3 Directory Migration (Priority: Medium)

- [ ] Add `~/.sk/kit/extensions/` path support
- [ ] Keep `snippets/` working during transition
- [ ] Update file watchers
- [ ] Update cache keys

### 5.4 Preference System (Priority: Medium)

- [ ] Create `~/.sk/kit/preferences/` directory structure
- [ ] Implement `getPreference(name)` SDK function
- [ ] Add preference validation on extension load
- [ ] Add "required preference" gating before command runs

### 5.5 Example Extensions (Priority: Medium)

- [ ] Create CleanShot extension with all commands
- [ ] Create Chrome extension with profile preference
- [ ] Test all commands work correctly
- [ ] Add README with usage instructions

---

## Testing Checklist

### Unit Tests

- [ ] `ExtensionManifest` parsing with all Raycast fields
- [ ] `CommandMetadata` parsing with mode, arguments, preferences
- [ ] Category validation
- [ ] Icon resolution (names vs paths)
- [ ] minVersion checking
- [ ] Preference parsing and validation

### Integration Tests

- [ ] Load extensions from `~/.sk/kit/extensions/`
- [ ] CleanShot commands trigger URL schemes
- [ ] Chrome commands read bookmarks/history
- [ ] Preferences persist and load correctly
- [ ] Required preference gating works

---

## References

- [Raycast Manifest Documentation](https://developers.raycast.com/information/manifest)
- [Raycast File Structure](https://developers.raycast.com/information/file-structure)
- [Raycast Preferences](https://developers.raycast.com/api-reference/preferences)
- [Raycast Store Requirements](https://developers.raycast.com/basics/prepare-an-extension-for-store)
