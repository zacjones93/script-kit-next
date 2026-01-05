# Expert Review Request: mdflow Agents Integration Plan

## Context

We're planning to integrate [mdflow](https://github.com/johnlindquist/mdflow) as first-class runnable agents in Script Kit GPUI. mdflow allows markdown files to become executable AI prompts that can be run against Claude, Gemini, Codex, or Copilot.

## What We Need Reviewed

1. **Architecture approach**: Does the plan properly follow the existing patterns for scripts and scriptlets?

2. **Data model design**: Is the `Agent`, `AgentBackend`, `AgentFrontmatter` struct design appropriate?

3. **Execution strategy**: Should we spawn `mdflow` as a subprocess or consider embedding its functionality?

4. **Open questions** (please provide recommendations):
   - **Interactive mode**: Terminal window vs embedded PTY for interactive agents?
   - **mdflow installation**: Bundle as dependency vs require user installation?
   - **Template variable UI**: Use existing `fields()` prompt for `_inputs`?
   - **Streaming output**: Real-time streaming vs wait for completion?
   - **Backend availability**: How to handle missing AI CLIs (claude, gemini, etc.)?

5. **Missing considerations**: Are there any patterns, edge cases, or integration points we've overlooked?

## Key Design Decisions Made

- Location: `~/.scriptkit/*/agents/*.md` (parallel to scripts/ and scriptlets/)
- Backend detection from filename: `.claude.md`, `.gemini.md`, `.codex.md`, `.copilot.md`
- Interactive mode via `.i.` marker in filename (e.g., `task.i.claude.md`)
- Frontmatter parsed as YAML and converted to CLI flags
- Execution via spawning `mdflow` subprocess

## Bundled Context

The following files are included for reference:
- `plan/AGENTS_PLAN.md` - The full implementation plan
- `src/scripts.rs` - Script/Scriptlet types and fuzzy search (patterns to follow)
- `src/scriptlets.rs` - Scriptlet parsing (YAML frontmatter, markdown parsing)
- `src/watcher.rs` - File watcher implementation to extend

---

This file is a merged representation of the filtered codebase, combined into a single document by packx.

<file_summary>
This section contains a summary of this file.

<purpose>
This file contains a packed representation of filtered repository contents.
It is designed to be easily consumable by AI systems for analysis, code review,
or other automated processes.
</purpose>

<usage_guidelines>
- Treat this file as a snapshot of the repository's state
- Be aware that this file may contain sensitive information
</usage_guidelines>

<notes>
- Files were filtered by packx based on content and extension matching
- Total files included: 3
</notes>
</file_summary>

<directory_structure>
src/scriptlets.rs
src/watcher.rs
plan/AGENTS_PLAN.md
</directory_structure>

<files>
This section contains the contents of the repository's files.

<file path="src/scriptlets.rs">
//! Scriptlet parsing and variable substitution module
//!
//! This module provides comprehensive support for parsing markdown files
//! containing scriptlets (code snippets with metadata) and performing
//! variable substitution in scriptlet content.
//!
//! # Types
//! - `Scriptlet`: Full scriptlet with all metadata
//! - `ScriptletMetadata`: Parsed HTML comment metadata
//!
//! # Features
//! - Parse markdown files with H1 groups and H2 scriptlets
//! - Extract metadata from HTML comments
//! - Handle nested code fences (``` inside ~~~ and vice versa)
//! - Variable substitution with named inputs, positional args, and conditionals

use crate::metadata_parser::TypedMetadata;
use crate::schema_parser::Schema;
use crate::scriptlet_metadata::parse_codefence_metadata;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::debug;

/// Valid tool types that can be used in code fences
pub const VALID_TOOLS: &[&str] = &[
    "bash",
    "python",
    "kit",
    "ts",
    "js",
    "transform",
    "template",
    "open",
    "edit",
    "paste",
    "type",
    "submit",
    "applescript",
    "ruby",
    "perl",
    "php",
    "node",
    "deno",
    "bun",
    // Shell variants
    "zsh",
    "sh",
    "fish",
    "cmd",
    "powershell",
    "pwsh",
];

/// Shell tools (tools that execute in a shell environment)
pub const SHELL_TOOLS: &[&str] = &["bash", "zsh", "sh", "fish", "cmd", "powershell", "pwsh"];

// ============================================================================
// Bundle Frontmatter (YAML at top of markdown files)
// ============================================================================

/// Frontmatter metadata for a scriptlet bundle (markdown file)
/// This is parsed from YAML at the top of the file, delimited by `---`
#[allow(dead_code)] // Public API for future use
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct BundleFrontmatter {
    /// Bundle name
    pub name: Option<String>,
    /// Bundle description
    pub description: Option<String>,
    /// Author of the bundle
    pub author: Option<String>,
    /// Default icon for scriptlets in this bundle
    pub icon: Option<String>,
    /// Any additional fields
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}

/// Parse YAML frontmatter from the beginning of markdown content
///
/// Frontmatter is delimited by `---` at the start and end:
/// ```markdown
/// ---
/// name: My Bundle
/// icon: Star
/// ---
/// # Content starts here
/// ```
#[allow(dead_code)] // Public API for future use
pub fn parse_bundle_frontmatter(content: &str) -> Option<BundleFrontmatter> {
    let trimmed = content.trim_start();

    // Must start with ---
    if !trimmed.starts_with("---") {
        return None;
    }

    // Find the closing ---
    let after_first = &trimmed[3..];
    let end_pos = after_first.find("\n---")?;

    let yaml_content = &after_first[..end_pos].trim();

    match serde_yaml::from_str::<BundleFrontmatter>(yaml_content) {
        Ok(fm) => Some(fm),
        Err(e) => {
            debug!(error = %e, "Failed to parse bundle frontmatter");
            None
        }
    }
}

/// Get a default icon for a tool type
#[allow(dead_code)] // Public API for future use
pub fn tool_type_to_icon(tool: &str) -> &'static str {
    match tool {
        "bash" | "zsh" | "sh" | "fish" => "terminal",
        "python" => "snake",
        "ruby" => "gem",
        "node" | "js" | "ts" | "kit" => "file-code",
        "open" => "external-link",
        "edit" => "edit",
        "paste" => "clipboard",
        "type" => "keyboard",
        "template" => "file-text",
        "transform" => "refresh-cw",
        "applescript" => "apple",
        "powershell" | "pwsh" | "cmd" => "terminal",
        "perl" => "code",
        "php" => "code",
        "deno" | "bun" => "file-code",
        _ => "file",
    }
}

/// Resolve the icon for a scriptlet using priority order:
/// 1. Scriptlet-level metadata icon
/// 2. Bundle frontmatter default icon
/// 3. Tool-type default icon
#[allow(dead_code)] // Public API for future use
pub fn resolve_scriptlet_icon(
    metadata: &ScriptletMetadata,
    frontmatter: Option<&BundleFrontmatter>,
    tool: &str,
) -> String {
    // Check scriptlet metadata first (via extra field for now)
    if let Some(icon) = metadata.extra.get("icon") {
        return icon.clone();
    }

    // Check bundle frontmatter
    if let Some(fm) = frontmatter {
        if let Some(ref icon) = fm.icon {
            return icon.clone();
        }
    }

    // Fall back to tool default
    tool_type_to_icon(tool).to_string()
}

// ============================================================================
// Validation Error Types
// ============================================================================

/// Error encountered during scriptlet validation.
/// Allows per-scriptlet validation with graceful degradation -
/// valid scriptlets can still be loaded even when others fail.
#[allow(dead_code)] // Public API for future use
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ScriptletValidationError {
    /// Path to the source file
    pub file_path: PathBuf,
    /// Name of the scriptlet that failed (if identifiable)
    pub scriptlet_name: Option<String>,
    /// Line number where the error occurred (1-based)
    pub line_number: Option<usize>,
    /// Description of what went wrong
    pub error_message: String,
}

#[allow(dead_code)] // Public API for future use
impl ScriptletValidationError {
    /// Create a new validation error
    pub fn new(
        file_path: impl Into<PathBuf>,
        scriptlet_name: Option<String>,
        line_number: Option<usize>,
        error_message: impl Into<String>,
    ) -> Self {
        Self {
            file_path: file_path.into(),
            scriptlet_name,
            line_number,
            error_message: error_message.into(),
        }
    }
}

impl std::fmt::Display for ScriptletValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.file_path.display())?;
        if let Some(line) = self.line_number {
            write!(f, ":{}", line)?;
        }
        if let Some(ref name) = self.scriptlet_name {
            write!(f, " [{}]", name)?;
        }
        write!(f, ": {}", self.error_message)
    }
}

/// Result of parsing scriptlets from a markdown file with validation.
/// Contains both successfully parsed scriptlets and any validation errors encountered.
#[allow(dead_code)] // Public API for future use
#[derive(Clone, Debug, Default)]
pub struct ScriptletParseResult {
    /// Successfully parsed scriptlets
    pub scriptlets: Vec<Scriptlet>,
    /// Validation errors for scriptlets that failed to parse
    pub errors: Vec<ScriptletValidationError>,
    /// Bundle-level frontmatter (if present)
    pub frontmatter: Option<BundleFrontmatter>,
}

/// Metadata extracted from HTML comments in scriptlets
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ScriptletMetadata {
    /// Trigger text that activates this scriptlet
    pub trigger: Option<String>,
    /// Keyboard shortcut (e.g., "cmd shift k")
    pub shortcut: Option<String>,
    /// Raw cron expression (e.g., "*/5 * * * *")
    pub cron: Option<String>,
    /// Natural language schedule (e.g., "every tuesday at 2pm") - converted to cron internally
    pub schedule: Option<String>,
    /// Whether to run in background
    pub background: Option<bool>,
    /// File paths to watch for changes
    pub watch: Option<String>,
    /// System event to trigger on
    pub system: Option<String>,
    /// Description of the scriptlet
    pub description: Option<String>,
    /// Text expansion trigger (e.g., "type,,")
    pub expand: Option<String>,
    /// Alias trigger - when user types alias + space, immediately run script
    pub alias: Option<String>,
    /// Any additional metadata key-value pairs
    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}

/// A scriptlet parsed from a markdown file
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Scriptlet {
    /// Name of the scriptlet (from H2 header)
    pub name: String,
    /// Command identifier (slugified name)
    pub command: String,
    /// Tool type (bash, python, ts, etc.)
    pub tool: String,
    /// The actual code content
    pub scriptlet_content: String,
    /// Named input placeholders (e.g., ["variableName", "otherVar"])
    pub inputs: Vec<String>,
    /// Group name (from H1 header)
    pub group: String,
    /// HTML preview content (if any)
    pub preview: Option<String>,
    /// Parsed metadata from HTML comments (legacy format)
    pub metadata: ScriptletMetadata,
    /// Typed metadata from codefence ```metadata block (new format)
    pub typed_metadata: Option<TypedMetadata>,
    /// Schema definition from codefence ```schema block
    pub schema: Option<Schema>,
    /// The kit this scriptlet belongs to
    pub kit: Option<String>,
    /// Source file path
    pub source_path: Option<String>,
}

#[allow(dead_code)]
impl Scriptlet {
    /// Create a new scriptlet with minimal required fields
    pub fn new(name: String, tool: String, content: String) -> Self {
        let command = slugify(&name);
        let inputs = extract_named_inputs(&content);

        Scriptlet {
            name,
            command,
            tool,
            scriptlet_content: content,
            inputs,
            group: String::new(),
            preview: None,
            metadata: ScriptletMetadata::default(),
            typed_metadata: None,
            schema: None,
            kit: None,
            source_path: None,
        }
    }

    /// Check if this scriptlet uses a shell tool
    pub fn is_shell(&self) -> bool {
        SHELL_TOOLS.contains(&self.tool.as_str())
    }

    /// Check if the tool type is valid
    pub fn is_valid_tool(&self) -> bool {
        VALID_TOOLS.contains(&self.tool.as_str())
    }
}

/// Convert a name to a command slug (lowercase, spaces to hyphens)
fn slugify(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Extract named input placeholders from scriptlet content
/// Finds all {{variableName}} patterns
fn extract_named_inputs(content: &str) -> Vec<String> {
    let mut inputs = Vec::new();
    let mut chars = content.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '{' && chars.peek() == Some(&'{') {
            chars.next(); // consume second {
            let mut name = String::new();

            // Skip if it's a conditional ({{#if, {{else, {{/if)
            if chars.peek() == Some(&'#') || chars.peek() == Some(&'/') {
                continue;
            }

            // Collect the variable name
            while let Some(&ch) = chars.peek() {
                if ch == '}' {
                    break;
                }
                name.push(ch);
                chars.next();
            }

            // Skip closing }}
            if chars.peek() == Some(&'}') {
                chars.next();
                if chars.peek() == Some(&'}') {
                    chars.next();
                }
            }

            // Add if valid identifier and not already present
            let trimmed = name.trim();
            if !trimmed.is_empty()
                && !trimmed.starts_with('#')
                && !trimmed.starts_with('/')
                && trimmed != "else"
                && !inputs.contains(&trimmed.to_string())
            {
                inputs.push(trimmed.to_string());
            }
        }
    }

    inputs
}

/// Parse metadata from HTML comments
/// Supports format: <!-- key: value\nkey2: value2 -->
pub fn parse_html_comment_metadata(text: &str) -> ScriptletMetadata {
    let mut metadata = ScriptletMetadata::default();

    // Find all HTML comment blocks
    let mut remaining = text;
    while let Some(start) = remaining.find("<!--") {
        if let Some(end) = remaining[start..].find("-->") {
            let comment_content = &remaining[start + 4..start + end];

            // Parse key: value pairs
            for line in comment_content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                if let Some(colon_pos) = trimmed.find(':') {
                    let key = trimmed[..colon_pos].trim().to_lowercase();
                    let value = trimmed[colon_pos + 1..].trim().to_string();

                    if value.is_empty() {
                        continue;
                    }

                    match key.as_str() {
                        "trigger" => metadata.trigger = Some(value),
                        "shortcut" => metadata.shortcut = Some(value),
                        "cron" => metadata.cron = Some(value),
                        "schedule" => metadata.schedule = Some(value),
                        "background" => {
                            metadata.background =
                                Some(value.to_lowercase() == "true" || value == "1")
                        }
                        "watch" => metadata.watch = Some(value),
                        "system" => metadata.system = Some(value),
                        "description" => metadata.description = Some(value),
                        "expand" => metadata.expand = Some(value),
                        "alias" => metadata.alias = Some(value),
                        _ => {
                            metadata.extra.insert(key, value);
                        }
                    }
                }
            }

            remaining = &remaining[start + end + 3..];
        } else {
            break;
        }
    }

    metadata
}

/// State for parsing code fences
#[derive(Clone, Copy, PartialEq)]
enum FenceType {
    Backticks, // ```
    Tildes,    // ~~~
}

/// Extract code block from text, handling nested fences
/// Returns (tool, code) if found
pub fn extract_code_block_nested(text: &str) -> Option<(String, String)> {
    let lines: Vec<&str> = text.lines().collect();
    let mut in_fence = false;
    let mut fence_type: Option<FenceType> = None;
    let mut fence_count = 0;
    let mut tool = String::new();
    let mut code_lines = Vec::new();
    let mut found = false;

    for line in lines {
        let trimmed = line.trim_start();

        if !in_fence {
            // Check for opening fence
            if let Some(fence_info) = detect_fence_start(trimmed) {
                in_fence = true;
                fence_type = Some(fence_info.0);
                fence_count = fence_info.1;
                tool = fence_info.2;
                continue;
            }
        } else {
            // Check for closing fence (same type, same or more chars)
            if is_matching_fence_end(trimmed, fence_type.unwrap(), fence_count) {
                found = true;
                break;
            }
            code_lines.push(line);
        }
    }

    if found {
        let code = code_lines.join("\n");
        Some((tool, code.trim().to_string()))
    } else if in_fence && !code_lines.is_empty() {
        // Unclosed fence, but we have content
        let code = code_lines.join("\n");
        Some((tool, code.trim().to_string()))
    } else {
        None
    }
}

/// Detect if a line starts a code fence, returns (fence_type, count, language)
fn detect_fence_start(line: &str) -> Option<(FenceType, usize, String)> {
    let backtick_count = line.chars().take_while(|&c| c == '`').count();
    if backtick_count >= 3 {
        let rest = &line[backtick_count..];
        let lang = rest.split_whitespace().next().unwrap_or("").to_string();
        return Some((FenceType::Backticks, backtick_count, lang));
    }

    let tilde_count = line.chars().take_while(|&c| c == '~').count();
    if tilde_count >= 3 {
        let rest = &line[tilde_count..];
        let lang = rest.split_whitespace().next().unwrap_or("").to_string();
        return Some((FenceType::Tildes, tilde_count, lang));
    }

    None
}

/// Check if a line is a closing fence matching the opening
fn is_matching_fence_end(line: &str, fence_type: FenceType, min_count: usize) -> bool {
    let count = match fence_type {
        FenceType::Backticks => line.chars().take_while(|&c| c == '`').count(),
        FenceType::Tildes => line.chars().take_while(|&c| c == '~').count(),
    };

    if count < min_count {
        return false;
    }

    // Rest of line should be empty or whitespace
    let rest = &line[count..];
    rest.chars().all(|c| c.is_whitespace())
}

/// Parse a markdown file into scriptlets
///
/// # Format
/// - H1 headers (`# Group Name`) define groups
/// - H1 can have a code fence that prepends to all scriptlets in that group
/// - H2 headers (`## Scriptlet Name`) define individual scriptlets
/// - HTML comments contain metadata
/// - Code fences contain the scriptlet code
pub fn parse_markdown_as_scriptlets(content: &str, source_path: Option<&str>) -> Vec<Scriptlet> {
    let mut scriptlets = Vec::new();
    let mut current_group = String::new();
    let mut global_prepend = String::new();

    // Split by headers while preserving the header type
    let sections = split_by_headers(content);

    for section in sections {
        let section_text = section.text;
        let first_line = section_text.lines().next().unwrap_or("");

        if first_line.starts_with("## ") {
            // H2: Individual scriptlet
            let name = first_line
                .strip_prefix("## ")
                .unwrap_or("")
                .trim()
                .to_string();

            if name.is_empty() {
                continue;
            }

            // Try codefence metadata first (new format)
            let codefence_result = parse_codefence_metadata(section_text);
            let typed_metadata = codefence_result.metadata;
            let schema = codefence_result.schema;

            // Also parse HTML comment metadata (legacy format, for backward compatibility)
            let metadata = parse_html_comment_metadata(section_text);

            // Extract code block - prefer codefence result if available, else use legacy extraction
            let code_block = if let Some(ref code_block) = codefence_result.code {
                Some((code_block.language.clone(), code_block.content.clone()))
            } else {
                extract_code_block_nested(section_text)
            };

            if let Some((tool_str, mut code)) = code_block {
                // Prepend global code if exists and tool matches
                if !global_prepend.is_empty() {
                    code = format!("{}\n{}", global_prepend, code);
                }

                // Validate tool type
                let tool: String = if tool_str.is_empty() {
                    "ts".to_string()
                } else {
                    tool_str
                };

                // Check if tool is valid, warn if not
                if !VALID_TOOLS.contains(&tool.as_str()) {
                    debug!(tool = %tool, name = %name, "Unknown tool type in scriptlet");
                }

                let inputs = extract_named_inputs(&code);
                let command = slugify(&name);

                scriptlets.push(Scriptlet {
                    name,
                    command,
                    tool,
                    scriptlet_content: code,
                    inputs,
                    group: current_group.clone(),
                    preview: None,
                    metadata,
                    typed_metadata,
                    schema,
                    kit: None,
                    source_path: source_path.map(|s| s.to_string()),
                });
            }
        } else if first_line.starts_with("# ") {
            // H1: Group header
            let group_name = first_line
                .strip_prefix("# ")
                .unwrap_or("")
                .trim()
                .to_string();
            current_group = group_name;

            // Check for global prepend code block
            if let Some((_, code)) = extract_code_block_nested(section_text) {
                global_prepend = code;
            } else {
                global_prepend.clear();
            }
        }
    }

    scriptlets
}

/// Section of markdown content with its header level
struct MarkdownSection<'a> {
    text: &'a str,
}

/// Split markdown content by headers, preserving header lines
fn split_by_headers(content: &str) -> Vec<MarkdownSection<'_>> {
    let mut sections = Vec::new();
    let mut current_start = 0;
    let mut in_fence = false;
    let mut fence_type: Option<FenceType> = None;
    let mut fence_count = 0;

    let lines: Vec<&str> = content.lines().collect();
    let line_starts: Vec<usize> = std::iter::once(0)
        .chain(content.match_indices('\n').map(|(i, _)| i + 1))
        .collect();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();

        // Track fence state
        if !in_fence {
            if let Some(fence_info) = detect_fence_start(trimmed) {
                in_fence = true;
                fence_type = Some(fence_info.0);
                fence_count = fence_info.1;
                continue;
            }
        } else if is_matching_fence_end(trimmed, fence_type.unwrap(), fence_count) {
            in_fence = false;
            fence_type = None;
            fence_count = 0;
            continue;
        }

        // Only split on headers outside of fences
        if !in_fence && (trimmed.starts_with("# ") || trimmed.starts_with("## ")) {
            if i > 0 {
                let start = line_starts[current_start];
                let end = line_starts[i];
                if end > start {
                    sections.push(MarkdownSection {
                        text: &content[start..end],
                    });
                }
            }
            current_start = i;
        }
    }

    // Add remaining content
    if current_start < lines.len() {
        let start = line_starts[current_start];
        sections.push(MarkdownSection {
            text: &content[start..],
        });
    }

    sections
}

// ============================================================================
// Validation-Aware Parsing
// ============================================================================

/// Parse markdown file into scriptlets with validation and graceful degradation.
///
/// Unlike `parse_markdown_as_scriptlets`, this function:
/// - Returns both valid scriptlets AND validation errors
/// - Continues parsing after individual scriptlet validation failures
/// - Parses bundle-level frontmatter
/// - Resolves icons using the priority order (scriptlet > frontmatter > tool default)
#[allow(dead_code)] // Public API for future use
pub fn parse_scriptlets_with_validation(
    content: &str,
    source_path: Option<&str>,
) -> ScriptletParseResult {
    let mut result = ScriptletParseResult::default();
    let file_path = PathBuf::from(source_path.unwrap_or("<unknown>"));

    // Parse bundle-level frontmatter
    result.frontmatter = parse_bundle_frontmatter(content);

    let mut current_group = String::new();
    let mut global_prepend = String::new();

    // Split by headers while preserving the header type and line numbers
    let sections = split_by_headers_with_line_numbers(content);

    for section in sections {
        let section_text = section.text;
        let section_start_line = section.line_number;
        let first_line = section_text.lines().next().unwrap_or("");

        if first_line.starts_with("## ") {
            // H2: Individual scriptlet
            let name = first_line
                .strip_prefix("## ")
                .unwrap_or("")
                .trim()
                .to_string();

            if name.is_empty() {
                result.errors.push(ScriptletValidationError::new(
                    &file_path,
                    None,
                    Some(section_start_line),
                    "Empty scriptlet name (H2 header with no text)",
                ));
                continue;
            }

            // Try to parse this scriptlet, catching any validation errors
            match parse_single_scriptlet(
                section_text,
                &name,
                &current_group,
                &global_prepend,
                source_path,
                result.frontmatter.as_ref(),
                section_start_line,
                &file_path,
            ) {
                Ok(scriptlet) => result.scriptlets.push(scriptlet),
                Err(error) => result.errors.push(error),
            }
        } else if first_line.starts_with("# ") {
            // H1: Group header
            let group_name = first_line
                .strip_prefix("# ")
                .unwrap_or("")
                .trim()
                .to_string();
            current_group = group_name;

            // Check for global prepend code block
            if let Some((_, code)) = extract_code_block_nested(section_text) {
                global_prepend = code;
            } else {
                global_prepend.clear();
            }
        }
    }

    result
}

/// Parse a single scriptlet from a section, returning either a Scriptlet or a validation error
#[allow(dead_code)] // Used by parse_scriptlets_with_validation
#[allow(clippy::too_many_arguments)]
fn parse_single_scriptlet(
    section_text: &str,
    name: &str,
    current_group: &str,
    global_prepend: &str,
    source_path: Option<&str>,
    frontmatter: Option<&BundleFrontmatter>,
    section_start_line: usize,
    file_path: &PathBuf,
) -> Result<Scriptlet, ScriptletValidationError> {
    // Try codefence metadata first (new format)
    let codefence_result = parse_codefence_metadata(section_text);
    let typed_metadata = codefence_result.metadata;
    let schema = codefence_result.schema;

    // Check for codefence parse errors - log but don't fail
    for error in &codefence_result.errors {
        debug!(error = %error, scriptlet = %name, "Codefence parse warning");
    }

    // Also parse HTML comment metadata (legacy format, for backward compatibility)
    let metadata = parse_html_comment_metadata(section_text);

    // Extract code block - prefer codefence result if available
    let code_block = if let Some(ref code_block) = codefence_result.code {
        Some((code_block.language.clone(), code_block.content.clone()))
    } else {
        extract_code_block_nested(section_text)
    };

    let (tool_str, mut code) = code_block.ok_or_else(|| {
        ScriptletValidationError::new(
            file_path,
            Some(name.to_string()),
            Some(section_start_line),
            "No code block found in scriptlet",
        )
    })?;

    // Prepend global code if exists
    if !global_prepend.is_empty() {
        code = format!("{}\n{}", global_prepend, code);
    }

    // Default tool type to "ts" if empty
    let tool = if tool_str.is_empty() {
        "ts".to_string()
    } else {
        tool_str
    };

    // Check if tool is valid - emit warning but don't fail
    if !VALID_TOOLS.contains(&tool.as_str()) {
        debug!(tool = %tool, name = %name, "Unknown tool type in scriptlet");
    }

    // Resolve icon using priority order
    let _resolved_icon = resolve_scriptlet_icon(&metadata, frontmatter, &tool);

    let inputs = extract_named_inputs(&code);
    let command = slugify(name);

    Ok(Scriptlet {
        name: name.to_string(),
        command,
        tool,
        scriptlet_content: code,
        inputs,
        group: current_group.to_string(),
        preview: None,
        metadata,
        typed_metadata,
        schema,
        kit: None,
        source_path: source_path.map(|s| s.to_string()),
    })
}

/// Section of markdown content with its header level and line number
#[allow(dead_code)] // Used by split_by_headers_with_line_numbers
struct MarkdownSectionWithLine<'a> {
    text: &'a str,
    line_number: usize, // 1-based line number
}

/// Split markdown content by headers, preserving header lines and line numbers
#[allow(dead_code)] // Used by parse_scriptlets_with_validation
fn split_by_headers_with_line_numbers(content: &str) -> Vec<MarkdownSectionWithLine<'_>> {
    let mut sections = Vec::new();
    let mut current_start = 0;
    let mut current_start_line = 1; // 1-based
    let mut in_fence = false;
    let mut fence_type: Option<FenceType> = None;
    let mut fence_count = 0;

    let lines: Vec<&str> = content.lines().collect();
    let line_starts: Vec<usize> = std::iter::once(0)
        .chain(content.match_indices('\n').map(|(i, _)| i + 1))
        .collect();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();

        // Track fence state
        if !in_fence {
            if let Some(fence_info) = detect_fence_start(trimmed) {
                in_fence = true;
                fence_type = Some(fence_info.0);
                fence_count = fence_info.1;
                continue;
            }
        } else if is_matching_fence_end(trimmed, fence_type.unwrap(), fence_count) {
            in_fence = false;
            fence_type = None;
            fence_count = 0;
            continue;
        }

        // Only split on headers outside of fences
        if !in_fence && (trimmed.starts_with("# ") || trimmed.starts_with("## ")) {
            if i > 0 {
                let start = line_starts[current_start];
                let end = line_starts[i];
                if end > start {
                    sections.push(MarkdownSectionWithLine {
                        text: &content[start..end],
                        line_number: current_start_line,
                    });
                }
            }
            current_start = i;
            current_start_line = i + 1; // Convert to 1-based
        }
    }

    // Add remaining content
    if current_start < lines.len() {
        let start = line_starts[current_start];
        sections.push(MarkdownSectionWithLine {
            text: &content[start..],
            line_number: current_start_line,
        });
    }

    sections
}

// ============================================================================
// Variable Substitution
// ============================================================================

/// Format a scriptlet by substituting variables
///
/// # Variable Types
/// - `{{variableName}}` - Named input, replaced with value from inputs map
/// - `$1`, `$2`, etc. (Unix) or `%1`, `%2`, etc. (Windows) - Positional args
/// - `$@` (Unix) or `%*` (Windows) - All arguments
///
/// # Arguments
/// * `content` - The scriptlet content with placeholders
/// * `inputs` - Map of variable names to values
/// * `positional_args` - List of positional arguments
/// * `windows` - If true, use Windows-style placeholders (%1, %*)
pub fn format_scriptlet(
    content: &str,
    inputs: &HashMap<String, String>,
    positional_args: &[String],
    windows: bool,
) -> String {
    let mut result = content.to_string();

    // Replace named inputs {{variableName}}
    for (name, value) in inputs {
        let placeholder = format!("{{{{{}}}}}", name);
        result = result.replace(&placeholder, value);
    }

    // Replace positional arguments
    if windows {
        // Windows style: %1, %2, etc.
        for (i, arg) in positional_args.iter().enumerate() {
            let placeholder = format!("%{}", i + 1);
            result = result.replace(&placeholder, arg);
        }

        // Replace %* with all args quoted
        let all_args = positional_args
            .iter()
            .map(|a| format!("\"{}\"", a.replace('\"', "\\\"")))
            .collect::<Vec<_>>()
            .join(" ");
        result = result.replace("%*", &all_args);
    } else {
        // Unix style: $1, $2, etc.
        for (i, arg) in positional_args.iter().enumerate() {
            let placeholder = format!("${}", i + 1);
            result = result.replace(&placeholder, arg);
        }

        // Replace $@ with all args quoted
        let all_args = positional_args
            .iter()
            .map(|a| format!("\"{}\"", a.replace('\"', "\\\"")))
            .collect::<Vec<_>>()
            .join(" ");
        result = result.replace("$@", &all_args);
    }

    result
}

/// Process conditional blocks in scriptlet content
///
/// Supports:
/// - `{{#if flag}}...{{/if}}` - Include content if flag is truthy
/// - `{{#if flag}}...{{else}}...{{/if}}` - If-else
/// - `{{#if flag}}...{{else if other}}...{{else}}...{{/if}}` - If-else-if chains
///
/// # Arguments
/// * `content` - The scriptlet content with conditionals
/// * `flags` - Map of flag names to boolean values
pub fn process_conditionals(content: &str, flags: &HashMap<String, bool>) -> String {
    process_conditionals_impl(content, flags)
}

/// Internal implementation that handles the recursive conditional processing
fn process_conditionals_impl(content: &str, flags: &HashMap<String, bool>) -> String {
    let mut result = String::with_capacity(content.len());
    let mut i = 0;
    let bytes = content.as_bytes();

    while i < bytes.len() {
        // Check for {{#if
        if i + 5 < bytes.len() && &bytes[i..i + 3] == b"{{#" {
            // Find the closing }}
            if let Some(end_tag) = find_closing_braces(content, i + 3) {
                let directive = &content[i + 3..end_tag];

                if directive.starts_with("if ") {
                    let flag_name = directive.strip_prefix("if ").unwrap().trim();
                    let remaining = &content[end_tag + 2..];
                    let (processed, consumed) = process_if_block(remaining, flag_name, flags);
                    result.push_str(&processed);
                    i = end_tag + 2 + consumed;
                    continue;
                }
            }
        }

        // Not a conditional, just copy the character
        if i < content.len() {
            result.push(content[i..].chars().next().unwrap());
            i += content[i..].chars().next().unwrap().len_utf8();
        } else {
            break;
        }
    }

    result
}

/// Find the position of closing }} starting from a given position
fn find_closing_braces(content: &str, start: usize) -> Option<usize> {
    let bytes = content.as_bytes();
    let mut i = start;

    while i + 1 < bytes.len() {
        if bytes[i] == b'}' && bytes[i + 1] == b'}' {
            return Some(i);
        }
        i += 1;
    }

    None
}

/// Process a single if block, returning (result, bytes_consumed)
fn process_if_block(
    content: &str,
    flag_name: &str,
    flags: &HashMap<String, bool>,
) -> (String, usize) {
    let flag_value = flags.get(flag_name).copied().unwrap_or(false);

    let mut depth = 1;
    let mut if_content = String::new();
    let mut else_content = String::new();
    let mut else_if_chains: Vec<(String, String)> = Vec::new(); // (flag, content)
    let mut in_else = false;
    let mut current_else_if_flag: Option<String> = None;
    let mut consumed = 0;

    let mut chars = content.chars().peekable();
    let mut pos = 0;

    while let Some(c) = chars.next() {
        pos += c.len_utf8();

        if c == '{' && chars.peek() == Some(&'{') {
            chars.next();
            pos += 1;

            // Read what's inside
            let mut inner = String::new();
            while let Some(&ch) = chars.peek() {
                if ch == '}' {
                    break;
                }
                inner.push(ch);
                chars.next();
                pos += ch.len_utf8();
            }

            // Skip closing }}
            if chars.peek() == Some(&'}') {
                chars.next();
                pos += 1;
                if chars.peek() == Some(&'}') {
                    chars.next();
                    pos += 1;
                }
            }

            let inner_trimmed = inner.trim();

            if inner_trimmed.starts_with("#if ") {
                depth += 1;
                // Add to current content - inner already contains the #
                let tag = format!("{{{{{}}}}}", inner_trimmed);
                if in_else {
                    if current_else_if_flag.is_some() {
                        else_if_chains.last_mut().unwrap().1.push_str(&tag);
                    } else {
                        else_content.push_str(&tag);
                    }
                } else {
                    if_content.push_str(&tag);
                }
            } else if inner_trimmed == "/if" {
                depth -= 1;
                if depth == 0 {
                    consumed = pos;
                    break;
                } else {
                    let tag = "{{/if}}";
                    if in_else {
                        if current_else_if_flag.is_some() {
                            else_if_chains.last_mut().unwrap().1.push_str(tag);
                        } else {
                            else_content.push_str(tag);
                        }
                    } else {
                        if_content.push_str(tag);
                    }
                }
            } else if inner_trimmed == "else" && depth == 1 {
                in_else = true;
                current_else_if_flag = None;
            } else if inner_trimmed.starts_with("else if ") && depth == 1 {
                let else_if_flag = inner_trimmed
                    .strip_prefix("else if ")
                    .unwrap()
                    .trim()
                    .to_string();
                in_else = true;
                current_else_if_flag = Some(else_if_flag.clone());
                else_if_chains.push((else_if_flag, String::new()));
            } else {
                // Some other tag, add to current content
                let tag = format!("{{{{{}}}}}", inner);
                if in_else {
                    if current_else_if_flag.is_some() {
                        else_if_chains.last_mut().unwrap().1.push_str(&tag);
                    } else {
                        else_content.push_str(&tag);
                    }
                } else {
                    if_content.push_str(&tag);
                }
            }
        } else if in_else {
            if current_else_if_flag.is_some() {
                else_if_chains.last_mut().unwrap().1.push(c);
            } else {
                else_content.push(c);
            }
        } else {
            if_content.push(c);
        }
    }

    // Determine which content to use
    let result = if flag_value {
        // Process nested conditionals in if_content
        process_conditionals(&if_content, flags)
    } else {
        // Check else-if chains
        let mut found = false;
        let mut selected_content = String::new();

        for (chain_flag, chain_content) in &else_if_chains {
            if flags.get(chain_flag).copied().unwrap_or(false) {
                selected_content = process_conditionals(chain_content, flags);
                found = true;
                break;
            }
        }

        if !found {
            // Use else content
            process_conditionals(&else_content, flags)
        } else {
            selected_content
        }
    };

    (result, consumed)
}

// ============================================================================
// Interpreter Tool Constants and Error Helpers
// ============================================================================

/// Interpreter tools that require an external interpreter to execute
#[allow(dead_code)] // Infrastructure ready for use in executor.rs
pub const INTERPRETER_TOOLS: &[&str] = &["python", "ruby", "perl", "php", "node"];

/// Get the interpreter command for a given tool
///
/// # Arguments
/// * `tool` - The tool name (e.g., "python", "ruby")
///
/// # Returns
/// The interpreter command to use (e.g., "python3" for "python")
#[allow(dead_code)] // Infrastructure ready for use in executor.rs
pub fn get_interpreter_command(tool: &str) -> String {
    match tool {
        "python" => "python3".to_string(),
        "ruby" => "ruby".to_string(),
        "perl" => "perl".to_string(),
        "php" => "php".to_string(),
        "node" => "node".to_string(),
        _ => tool.to_string(),
    }
}

/// Get platform-specific installation instructions for an interpreter
///
/// # Arguments
/// * `interpreter` - The interpreter name (e.g., "python3", "ruby")
///
/// # Returns
/// A user-friendly error message with installation instructions
#[allow(dead_code)] // Infrastructure ready for use in executor.rs
pub fn interpreter_not_found_message(interpreter: &str) -> String {
    let tool_name = match interpreter {
        "python3" | "python" => "Python",
        "ruby" => "Ruby",
        "perl" => "Perl",
        "php" => "PHP",
        "node" | "nodejs" => "Node.js",
        _ => interpreter,
    };

    let install_instructions = get_platform_install_instructions(interpreter);

    format!(
        "{} interpreter not found.\n\n{}\n\nAfter installation, restart Script Kit.",
        tool_name, install_instructions
    )
}

/// Get platform-specific installation instructions
///
/// # Arguments
/// * `interpreter` - The interpreter name
///
/// # Returns
/// Platform-specific installation command suggestions
#[allow(dead_code)] // Used by interpreter_not_found_message
fn get_platform_install_instructions(interpreter: &str) -> String {
    #[cfg(target_os = "macos")]
    {
        get_macos_install_instructions(interpreter)
    }
    #[cfg(target_os = "linux")]
    {
        get_linux_install_instructions(interpreter)
    }
    #[cfg(target_os = "windows")]
    {
        get_windows_install_instructions(interpreter)
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        format!(
            "Please install {} using your system's package manager.",
            interpreter
        )
    }
}

/// Get macOS installation instructions (Homebrew)
#[cfg(target_os = "macos")]
#[allow(dead_code)] // Used by get_platform_install_instructions
fn get_macos_install_instructions(interpreter: &str) -> String {
    let brew_package = match interpreter {
        "python3" | "python" => "python",
        "ruby" => "ruby",
        "perl" => "perl",
        "php" => "php",
        "node" | "nodejs" => "node",
        _ => interpreter,
    };

    format!(
        "Install using Homebrew:\n  brew install {}\n\nOr download from the official website.",
        brew_package
    )
}

/// Get Linux installation instructions (apt/dnf)
#[cfg(target_os = "linux")]
fn get_linux_install_instructions(interpreter: &str) -> String {
    let (apt_package, dnf_package) = match interpreter {
        "python3" | "python" => ("python3", "python3"),
        "ruby" => ("ruby", "ruby"),
        "perl" => ("perl", "perl"),
        "php" => ("php", "php-cli"),
        "node" | "nodejs" => ("nodejs", "nodejs"),
        _ => (interpreter, interpreter),
    };

    format!(
        "Install using your package manager:\n\n  Debian/Ubuntu:\n    sudo apt install {}\n\n  Fedora/RHEL:\n    sudo dnf install {}",
        apt_package, dnf_package
    )
}

/// Get Windows installation instructions
#[cfg(target_os = "windows")]
fn get_windows_install_instructions(interpreter: &str) -> String {
    let (choco_package, download_url) = match interpreter {
        "python3" | "python" => ("python", "https://www.python.org/downloads/"),
        "ruby" => ("ruby", "https://rubyinstaller.org/"),
        "perl" => ("strawberryperl", "https://strawberryperl.com/"),
        "php" => ("php", "https://windows.php.net/download/"),
        "node" | "nodejs" => ("nodejs", "https://nodejs.org/"),
        _ => (interpreter, ""),
    };

    if download_url.is_empty() {
        format!(
            "Install using Chocolatey:\n  choco install {}",
            choco_package
        )
    } else {
        format!(
            "Install using Chocolatey:\n  choco install {}\n\nOr download from:\n  {}",
            choco_package, download_url
        )
    }
}

/// Check if a tool is an interpreter tool
///
/// # Arguments
/// * `tool` - The tool name to check
///
/// # Returns
/// `true` if the tool requires an external interpreter
#[allow(dead_code)] // Infrastructure ready for use in executor.rs
pub fn is_interpreter_tool(tool: &str) -> bool {
    INTERPRETER_TOOLS.contains(&tool)
}

/// Get the file extension for a given interpreter tool
///
/// # Arguments
/// * `tool` - The tool name
///
/// # Returns
/// The appropriate file extension for scripts of that type
#[allow(dead_code)] // Infrastructure ready for use in executor.rs
pub fn get_interpreter_extension(tool: &str) -> &'static str {
    match tool {
        "python" => "py",
        "ruby" => "rb",
        "perl" => "pl",
        "php" => "php",
        "node" => "js",
        _ => "txt",
    }
}

/// Validate that a tool name is a known interpreter
///
/// # Arguments
/// * `tool` - The tool name to validate
///
/// # Returns
/// `Ok(())` if valid, `Err` with descriptive message if not
#[allow(dead_code)] // Infrastructure ready for use in executor.rs
pub fn validate_interpreter_tool(tool: &str) -> Result<(), String> {
    if is_interpreter_tool(tool) {
        Ok(())
    } else if VALID_TOOLS.contains(&tool) {
        Err(format!(
            "'{}' is a valid tool but not an interpreter tool",
            tool
        ))
    } else {
        Err(format!("'{}' is not a recognized tool type", tool))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[path = "scriptlet_tests.rs"]
mod tests;

</file>

<file path="src/watcher.rs">
#![allow(dead_code)]
use notify::{recommended_watcher, RecursiveMode, Result as NotifyResult, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use std::process::Command;
use tracing::{info, warn};

/// Event emitted when config needs to be reloaded
#[derive(Debug, Clone)]
pub enum ConfigReloadEvent {
    Reload,
}

/// Event emitted when theme needs to be reloaded
#[derive(Debug, Clone)]
pub enum ThemeReloadEvent {
    Reload,
}

/// Event emitted when scripts need to be reloaded
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptReloadEvent {
    /// A specific file was modified
    FileChanged(PathBuf),
    /// A new file was created
    FileCreated(PathBuf),
    /// A file was deleted
    FileDeleted(PathBuf),
    /// Fallback for complex events (e.g., bulk changes, renames)
    FullReload,
}

/// Event emitted when system appearance changes (light/dark mode)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppearanceChangeEvent {
    /// Dark mode is now active
    Dark,
    /// Light mode is now active
    Light,
}

/// Watches ~/.scriptkit/config.ts for changes and emits reload events
pub struct ConfigWatcher {
    tx: Option<Sender<ConfigReloadEvent>>,
    watcher_thread: Option<thread::JoinHandle<()>>,
}

impl ConfigWatcher {
    /// Create a new ConfigWatcher
    ///
    /// Returns a tuple of (watcher, receiver) where receiver will emit ConfigReloadEvent
    /// when the config file changes.
    pub fn new() -> (Self, Receiver<ConfigReloadEvent>) {
        let (tx, rx) = channel();
        let watcher = ConfigWatcher {
            tx: Some(tx),
            watcher_thread: None,
        };
        (watcher, rx)
    }

    /// Start watching the config file for changes
    ///
    /// This spawns a background thread that watches ~/.scriptkit/config.ts and sends
    /// reload events through the receiver when changes are detected.
    pub fn start(&mut self) -> NotifyResult<()> {
        let tx = self
            .tx
            .take()
            .ok_or_else(|| std::io::Error::other("watcher already started"))?;

        let thread_handle = thread::spawn(move || {
            if let Err(e) = Self::watch_loop(tx) {
                warn!(error = %e, watcher = "config", "Config watcher error");
            }
        });

        self.watcher_thread = Some(thread_handle);
        Ok(())
    }

    /// Internal watch loop running in background thread
    fn watch_loop(tx: Sender<ConfigReloadEvent>) -> NotifyResult<()> {
        // Expand the config path
        let config_path = PathBuf::from(shellexpand::tilde("~/.scriptkit/config.ts").as_ref());

        // Get the parent directory to watch
        let watch_path = config_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));

        // Create a debounce timer using Arc<Mutex>
        let debounce_active = Arc::new(Mutex::new(false));
        let debounce_active_clone = debounce_active.clone();

        // Channel for the file watcher thread
        let (watch_tx, watch_rx) = channel();

        // Create the watcher with a callback
        let mut watcher: Box<dyn Watcher> = Box::new(recommended_watcher(
            move |res: notify::Result<notify::Event>| {
                let _ = watch_tx.send(res);
            },
        )?);

        // Watch the directory containing config.ts
        watcher.watch(watch_path, RecursiveMode::NonRecursive)?;

        info!(
            path = %watch_path.display(),
            target = "config.ts",
            "Config watcher started"
        );

        // Main watch loop
        loop {
            match watch_rx.recv() {
                Ok(Ok(event)) => {
                    // Check if this is an event for config.ts
                    let is_config_change = event.paths.iter().any(|path: &PathBuf| {
                        path.file_name()
                            .and_then(|name| name.to_str())
                            .map(|name| name == "config.ts")
                            .unwrap_or(false)
                    });

                    // Only care about Create and Modify events
                    let is_relevant_event = matches!(
                        event.kind,
                        notify::EventKind::Create(_) | notify::EventKind::Modify(_)
                    );

                    if is_config_change && is_relevant_event {
                        // Check if debounce is already active
                        let mut debounce = debounce_active_clone.lock().unwrap();
                        if !*debounce {
                            *debounce = true;
                            drop(debounce); // Release lock before spawning thread

                            let tx_clone = tx.clone();
                            let debounce_flag = debounce_active_clone.clone();

                            // Spawn debounce thread
                            thread::spawn(move || {
                                thread::sleep(Duration::from_millis(500));
                                let _ = tx_clone.send(ConfigReloadEvent::Reload);
                                let mut flag = debounce_flag.lock().unwrap();
                                *flag = false;
                                info!(
                                    file = "config.ts",
                                    "Config file changed, emitting reload event"
                                );
                            });
                        }
                    }
                }
                Ok(Err(e)) => {
                    warn!(error = %e, watcher = "config", "File watcher error");
                }
                Err(_) => {
                    // Channel closed, exit watch loop
                    info!(watcher = "config", "Config watcher shutting down");
                    break;
                }
            }
        }

        Ok(())
    }
}

impl Drop for ConfigWatcher {
    fn drop(&mut self) {
        // Wait for watcher thread to finish (with timeout)
        if let Some(handle) = self.watcher_thread.take() {
            let _ = handle.join();
        }
    }
}

/// Watches ~/.scriptkit/theme.json for changes and emits reload events
pub struct ThemeWatcher {
    tx: Option<Sender<ThemeReloadEvent>>,
    watcher_thread: Option<thread::JoinHandle<()>>,
}

impl ThemeWatcher {
    /// Create a new ThemeWatcher
    ///
    /// Returns a tuple of (watcher, receiver) where receiver will emit ThemeReloadEvent
    /// when the theme file changes.
    pub fn new() -> (Self, Receiver<ThemeReloadEvent>) {
        let (tx, rx) = channel();
        let watcher = ThemeWatcher {
            tx: Some(tx),
            watcher_thread: None,
        };
        (watcher, rx)
    }

    /// Start watching the theme file for changes
    ///
    /// This spawns a background thread that watches ~/.scriptkit/theme.json and sends
    /// reload events through the receiver when changes are detected.
    pub fn start(&mut self) -> NotifyResult<()> {
        let tx = self
            .tx
            .take()
            .ok_or_else(|| std::io::Error::other("watcher already started"))?;

        let thread_handle = thread::spawn(move || {
            if let Err(e) = Self::watch_loop(tx) {
                warn!(error = %e, watcher = "theme", "Theme watcher error");
            }
        });

        self.watcher_thread = Some(thread_handle);
        Ok(())
    }

    /// Internal watch loop running in background thread
    fn watch_loop(tx: Sender<ThemeReloadEvent>) -> NotifyResult<()> {
        // Expand the theme path
        let theme_path = PathBuf::from(shellexpand::tilde("~/.scriptkit/theme.json").as_ref());

        // Get the parent directory to watch
        let watch_path = theme_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));

        // Create a debounce timer using Arc<Mutex>
        let debounce_active = Arc::new(Mutex::new(false));
        let debounce_active_clone = debounce_active.clone();

        // Channel for the file watcher thread
        let (watch_tx, watch_rx) = channel();

        // Create the watcher with a callback
        let mut watcher: Box<dyn Watcher> = Box::new(recommended_watcher(
            move |res: notify::Result<notify::Event>| {
                let _ = watch_tx.send(res);
            },
        )?);

        // Watch the directory containing theme.json
        watcher.watch(watch_path, RecursiveMode::NonRecursive)?;

        info!(
            path = %watch_path.display(),
            target = "theme.json",
            "Theme watcher started"
        );

        // Main watch loop
        loop {
            match watch_rx.recv() {
                Ok(Ok(event)) => {
                    // Check if this is an event for theme.json
                    let is_theme_change = event.paths.iter().any(|path: &PathBuf| {
                        path.file_name()
                            .and_then(|name| name.to_str())
                            .map(|name| name == "theme.json")
                            .unwrap_or(false)
                    });

                    // Only care about Create and Modify events
                    let is_relevant_event = matches!(
                        event.kind,
                        notify::EventKind::Create(_) | notify::EventKind::Modify(_)
                    );

                    if is_theme_change && is_relevant_event {
                        // Check if debounce is already active
                        let mut debounce = debounce_active_clone.lock().unwrap();
                        if !*debounce {
                            *debounce = true;
                            drop(debounce); // Release lock before spawning thread

                            let tx_clone = tx.clone();
                            let debounce_flag = debounce_active_clone.clone();

                            // Spawn debounce thread
                            thread::spawn(move || {
                                thread::sleep(Duration::from_millis(500));
                                let _ = tx_clone.send(ThemeReloadEvent::Reload);
                                let mut flag = debounce_flag.lock().unwrap();
                                *flag = false;
                                info!(
                                    file = "theme.json",
                                    "Theme file changed, emitting reload event"
                                );
                            });
                        }
                    }
                }
                Ok(Err(e)) => {
                    warn!(error = %e, watcher = "theme", "File watcher error");
                }
                Err(_) => {
                    // Channel closed, exit watch loop
                    info!(watcher = "theme", "Theme watcher shutting down");
                    break;
                }
            }
        }

        Ok(())
    }
}

impl Drop for ThemeWatcher {
    fn drop(&mut self) {
        // Wait for watcher thread to finish (with timeout)
        if let Some(handle) = self.watcher_thread.take() {
            let _ = handle.join();
        }
    }
}

/// Check if a file path is a relevant script file (ts, js, or md)
fn is_relevant_script_file(path: &std::path::Path) -> bool {
    // Skip hidden files
    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
        if file_name.starts_with('.') {
            return false;
        }
    }

    // Check for relevant extensions
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("ts") | Some("js") | Some("md")
    )
}

/// Watches ~/.scriptkit/scripts and ~/.scriptkit/scriptlets directories for changes and emits reload events
pub struct ScriptWatcher {
    tx: Option<Sender<ScriptReloadEvent>>,
    watcher_thread: Option<thread::JoinHandle<()>>,
}

impl ScriptWatcher {
    /// Create a new ScriptWatcher
    ///
    /// Returns a tuple of (watcher, receiver) where receiver will emit ScriptReloadEvent
    /// when files in the scripts directory change.
    pub fn new() -> (Self, Receiver<ScriptReloadEvent>) {
        let (tx, rx) = channel();
        let watcher = ScriptWatcher {
            tx: Some(tx),
            watcher_thread: None,
        };
        (watcher, rx)
    }

    /// Start watching the scripts directory for changes
    ///
    /// This spawns a background thread that watches ~/.scriptkit/scripts recursively and sends
    /// reload events through the receiver when scripts are added, modified, or deleted.
    pub fn start(&mut self) -> NotifyResult<()> {
        let tx = self
            .tx
            .take()
            .ok_or_else(|| std::io::Error::other("watcher already started"))?;

        let thread_handle = thread::spawn(move || {
            if let Err(e) = Self::watch_loop(tx) {
                warn!(error = %e, watcher = "scripts", "Script watcher error");
            }
        });

        self.watcher_thread = Some(thread_handle);
        Ok(())
    }

    /// Internal watch loop running in background thread
    fn watch_loop(tx: Sender<ScriptReloadEvent>) -> NotifyResult<()> {
        // Expand the scripts and scriptlets paths
        let scripts_path = PathBuf::from(shellexpand::tilde("~/.scriptkit/scripts").as_ref());
        let scriptlets_path = PathBuf::from(shellexpand::tilde("~/.scriptkit/scriptlets").as_ref());

        // Track pending events for debouncing (path -> (event_type, timestamp))
        let pending_events: Arc<
            Mutex<std::collections::HashMap<PathBuf, (ScriptReloadEvent, std::time::Instant)>>,
        > = Arc::new(Mutex::new(std::collections::HashMap::new()));
        let pending_events_clone = pending_events.clone();

        // Debounce interval
        let debounce_ms = 500;

        // Channel for the file watcher thread
        let (watch_tx, watch_rx) = channel();

        // Create the watcher with a callback
        let mut watcher: Box<dyn Watcher> = Box::new(recommended_watcher(
            move |res: notify::Result<notify::Event>| {
                let _ = watch_tx.send(res);
            },
        )?);

        // Watch the scripts directory recursively
        watcher.watch(&scripts_path, RecursiveMode::Recursive)?;

        // Watch the scriptlets directory recursively (for *.md files)
        if scriptlets_path.exists() {
            watcher.watch(&scriptlets_path, RecursiveMode::Recursive)?;
            info!(
                path = %scriptlets_path.display(),
                recursive = true,
                "Scriptlets watcher started"
            );
        }

        info!(
            path = %scripts_path.display(),
            recursive = true,
            "Script watcher started"
        );

        // Spawn a background thread to flush pending events after debounce interval
        let tx_clone = tx.clone();
        let flush_pending = pending_events_clone.clone();
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(100)); // Check every 100ms

                let now = std::time::Instant::now();
                let mut events_to_send = Vec::new();

                {
                    let mut pending = flush_pending.lock().unwrap();
                    let debounce_threshold = Duration::from_millis(debounce_ms);

                    // Find events that have been pending long enough
                    let expired: Vec<PathBuf> = pending
                        .iter()
                        .filter(|(_, (_, timestamp))| {
                            now.duration_since(*timestamp) >= debounce_threshold
                        })
                        .map(|(path, _)| path.clone())
                        .collect();

                    // Remove expired events and collect them for sending
                    for path in expired {
                        if let Some((event, _)) = pending.remove(&path) {
                            events_to_send.push((path, event));
                        }
                    }
                }

                // Send events outside the lock
                for (path, event) in events_to_send {
                    info!(
                        path = %path.display(),
                        event_type = ?event,
                        "Emitting script reload event"
                    );
                    if tx_clone.send(event).is_err() {
                        // Channel closed, exit flush thread
                        return;
                    }
                }
            }
        });

        // Main watch loop
        loop {
            match watch_rx.recv() {
                Ok(Ok(event)) => {
                    // Process each path in the event
                    for path in event.paths.iter() {
                        // Skip non-relevant files
                        if !is_relevant_script_file(path) {
                            continue;
                        }

                        // Determine the event type based on notify::EventKind
                        let reload_event = match event.kind {
                            notify::EventKind::Create(_) => {
                                ScriptReloadEvent::FileCreated(path.clone())
                            }
                            notify::EventKind::Modify(_) => {
                                ScriptReloadEvent::FileChanged(path.clone())
                            }
                            notify::EventKind::Remove(_) => {
                                ScriptReloadEvent::FileDeleted(path.clone())
                            }
                            // For other events (Access, Other), use FullReload as fallback
                            _ => continue,
                        };

                        // Update pending events map (this implements per-file debouncing)
                        let mut pending = pending_events.lock().unwrap();
                        pending.insert(path.clone(), (reload_event, std::time::Instant::now()));
                    }
                }
                Ok(Err(e)) => {
                    warn!(error = %e, watcher = "scripts", "File watcher error");
                }
                Err(_) => {
                    // Channel closed, exit watch loop
                    info!(watcher = "scripts", "Script watcher shutting down");
                    break;
                }
            }
        }

        Ok(())
    }
}

impl Drop for ScriptWatcher {
    fn drop(&mut self) {
        // Wait for watcher thread to finish (with timeout)
        if let Some(handle) = self.watcher_thread.take() {
            let _ = handle.join();
        }
    }
}

/// Watches system appearance (light/dark mode) for changes and emits events
///
/// This watcher polls the system appearance setting every 2 seconds by running
/// the `defaults read -g AppleInterfaceStyle` command on macOS.
pub struct AppearanceWatcher {
    tx: Option<async_channel::Sender<AppearanceChangeEvent>>,
    watcher_thread: Option<thread::JoinHandle<()>>,
}

impl AppearanceWatcher {
    /// Create a new AppearanceWatcher
    ///
    /// Returns a tuple of (watcher, receiver) where receiver will emit AppearanceChangeEvent
    /// when the system appearance changes.
    pub fn new() -> (Self, async_channel::Receiver<AppearanceChangeEvent>) {
        let (tx, rx) = async_channel::bounded(100);
        let watcher = AppearanceWatcher {
            tx: Some(tx),
            watcher_thread: None,
        };
        (watcher, rx)
    }

    /// Start watching the system appearance for changes
    ///
    /// This spawns a background thread that polls the system appearance every 2 seconds
    /// and sends appearance change events through the receiver when changes are detected.
    pub fn start(&mut self) -> Result<(), String> {
        let tx = self
            .tx
            .take()
            .ok_or_else(|| "watcher already started".to_string())?;

        let thread_handle = thread::spawn(move || {
            if let Err(e) = Self::watch_loop(tx) {
                warn!(error = %e, watcher = "appearance", "Appearance watcher error");
            }
        });

        self.watcher_thread = Some(thread_handle);
        Ok(())
    }

    /// Internal watch loop running in background thread
    fn watch_loop(tx: async_channel::Sender<AppearanceChangeEvent>) -> Result<(), String> {
        let mut last_appearance: Option<AppearanceChangeEvent> = None;
        let poll_interval = Duration::from_secs(2);

        info!(poll_interval_secs = 2, "Appearance watcher started");

        loop {
            // Detect current system appearance
            let current_appearance = Self::detect_appearance();

            // Send event if appearance changed
            if last_appearance != Some(current_appearance.clone()) {
                let mode = match current_appearance {
                    AppearanceChangeEvent::Dark => "dark",
                    AppearanceChangeEvent::Light => "light",
                };
                info!(mode = mode, "System appearance changed");
                if tx.send_blocking(current_appearance.clone()).is_err() {
                    info!(
                        watcher = "appearance",
                        "Appearance watcher receiver dropped, shutting down"
                    );
                    break;
                }
                last_appearance = Some(current_appearance);
            }

            // Poll every 2 seconds
            thread::sleep(poll_interval);
        }

        Ok(())
    }

    /// Detect the current system appearance
    fn detect_appearance() -> AppearanceChangeEvent {
        match Command::new("defaults")
            .args(["read", "-g", "AppleInterfaceStyle"])
            .output()
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.to_lowercase().contains("dark") {
                    AppearanceChangeEvent::Dark
                } else {
                    AppearanceChangeEvent::Light
                }
            }
            Err(_) => {
                // Command failed, likely in light mode on macOS
                AppearanceChangeEvent::Light
            }
        }
    }
}

impl Drop for AppearanceWatcher {
    fn drop(&mut self) {
        // Wait for watcher thread to finish (with timeout)
        if let Some(handle) = self.watcher_thread.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_watcher_creation() {
        let (_watcher, _rx) = ConfigWatcher::new();
        // Watcher should be created without panicking
    }

    #[test]
    fn test_config_reload_event_clone() {
        let event = ConfigReloadEvent::Reload;
        let _cloned = event.clone();
        // Event should be cloneable
    }

    #[test]
    fn test_theme_watcher_creation() {
        let (_watcher, _rx) = ThemeWatcher::new();
        // Watcher should be created without panicking
    }

    #[test]
    fn test_theme_reload_event_clone() {
        let event = ThemeReloadEvent::Reload;
        let _cloned = event.clone();
        // Event should be cloneable
    }

    #[test]
    fn test_script_watcher_creation() {
        let (_watcher, _rx) = ScriptWatcher::new();
        // Watcher should be created without panicking
    }

    #[test]
    fn test_script_reload_event_clone() {
        let event = ScriptReloadEvent::FullReload;
        let _cloned = event.clone();
        // Event should be cloneable
    }

    #[test]
    fn test_script_reload_event_file_changed() {
        let path = PathBuf::from("/test/path/script.ts");
        let event = ScriptReloadEvent::FileChanged(path.clone());

        // Verify the event contains the correct path
        if let ScriptReloadEvent::FileChanged(event_path) = event {
            assert_eq!(event_path, path);
        } else {
            panic!("Expected FileChanged variant");
        }
    }

    #[test]
    fn test_script_reload_event_file_created() {
        let path = PathBuf::from("/test/path/new-script.ts");
        let event = ScriptReloadEvent::FileCreated(path.clone());

        // Verify the event contains the correct path
        if let ScriptReloadEvent::FileCreated(event_path) = event {
            assert_eq!(event_path, path);
        } else {
            panic!("Expected FileCreated variant");
        }
    }

    #[test]
    fn test_script_reload_event_file_deleted() {
        let path = PathBuf::from("/test/path/deleted-script.ts");
        let event = ScriptReloadEvent::FileDeleted(path.clone());

        // Verify the event contains the correct path
        if let ScriptReloadEvent::FileDeleted(event_path) = event {
            assert_eq!(event_path, path);
        } else {
            panic!("Expected FileDeleted variant");
        }
    }

    #[test]
    fn test_script_reload_event_equality() {
        let path1 = PathBuf::from("/test/path/script.ts");
        let path2 = PathBuf::from("/test/path/script.ts");
        let path3 = PathBuf::from("/test/path/other.ts");

        // Same path should be equal
        assert_eq!(
            ScriptReloadEvent::FileChanged(path1.clone()),
            ScriptReloadEvent::FileChanged(path2.clone())
        );

        // Different paths should not be equal
        assert_ne!(
            ScriptReloadEvent::FileChanged(path1.clone()),
            ScriptReloadEvent::FileChanged(path3.clone())
        );

        // Different event types should not be equal
        assert_ne!(
            ScriptReloadEvent::FileChanged(path1.clone()),
            ScriptReloadEvent::FileCreated(path1.clone())
        );

        // FullReload should equal itself
        assert_eq!(ScriptReloadEvent::FullReload, ScriptReloadEvent::FullReload);
    }

    #[test]
    fn test_extract_file_path_from_event() {
        // Test helper function for extracting paths from notify events
        use notify::event::{CreateKind, ModifyKind, RemoveKind};

        let test_path = PathBuf::from("/Users/test/.scriptkit/scripts/hello.ts");

        // Test Create event
        let create_event = notify::Event {
            kind: notify::EventKind::Create(CreateKind::File),
            paths: vec![test_path.clone()],
            attrs: Default::default(),
        };
        assert_eq!(create_event.paths.first(), Some(&test_path));

        // Test Modify event
        let modify_event = notify::Event {
            kind: notify::EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Content)),
            paths: vec![test_path.clone()],
            attrs: Default::default(),
        };
        assert_eq!(modify_event.paths.first(), Some(&test_path));

        // Test Remove event
        let remove_event = notify::Event {
            kind: notify::EventKind::Remove(RemoveKind::File),
            paths: vec![test_path.clone()],
            attrs: Default::default(),
        };
        assert_eq!(remove_event.paths.first(), Some(&test_path));
    }

    #[test]
    fn test_is_relevant_script_file() {
        use std::path::Path;

        // Test that we correctly identify relevant script files
        let ts_path = Path::new("/Users/test/.scriptkit/scripts/hello.ts");
        let js_path = Path::new("/Users/test/.scriptkit/scripts/hello.js");
        let md_path = Path::new("/Users/test/.scriptkit/scriptlets/hello.md");
        let txt_path = Path::new("/Users/test/.scriptkit/scripts/readme.txt");
        let hidden_path = Path::new("/Users/test/.scriptkit/scripts/.hidden.ts");

        // TypeScript files should be relevant
        assert!(is_relevant_script_file(ts_path));

        // JavaScript files should be relevant
        assert!(is_relevant_script_file(js_path));

        // Markdown files in scriptlets should be relevant
        assert!(is_relevant_script_file(md_path));

        // Other file types should not be relevant
        assert!(!is_relevant_script_file(txt_path));

        // Hidden files should not be relevant
        assert!(!is_relevant_script_file(hidden_path));
    }

    #[test]
    fn test_appearance_change_event_clone() {
        let event_dark = AppearanceChangeEvent::Dark;
        let _cloned = event_dark.clone();
        let event_light = AppearanceChangeEvent::Light;
        let _cloned = event_light.clone();
        // Events should be cloneable
    }

    #[test]
    fn test_appearance_change_event_equality() {
        let dark1 = AppearanceChangeEvent::Dark;
        let dark2 = AppearanceChangeEvent::Dark;
        let light = AppearanceChangeEvent::Light;

        assert_eq!(dark1, dark2);
        assert_ne!(dark1, light);
    }

    #[test]
    fn test_appearance_watcher_creation() {
        let (_watcher, _rx) = AppearanceWatcher::new();
        // Watcher should be created without panicking
    }
}

</file>

<file path="plan/AGENTS_PLAN.md">
# Agents System Plan

> Integrate mdflow as runnable markdown agents in `~/.scriptkit/main/agents/*.md`

## Overview

This plan covers integrating [mdflow](https://github.com/johnlindquist/mdflow) as first-class agents in Script Kit, following the same patterns established for scripts and scriptlets:

1. **File watching**: Monitor `~/.scriptkit/*/agents/` directories for changes
2. **Parsing**: Extract agent metadata from YAML frontmatter and filename patterns
3. **Main menu integration**: Display agents alongside scripts/scriptlets in unified search
4. **Execution**: Run agents via `mdflow` CLI with proper stdin/stdout handling
5. **Backend selection**: Support `.claude.md`, `.gemini.md`, `.codex.md`, `.copilot.md` filename patterns

## mdflow Key Concepts

| Concept | Description | Example |
|---------|-------------|---------|
| **Filename → Command** | `task.claude.md` runs `claude` | `review.gemini.md` → `gemini` |
| **Frontmatter → CLI Flags** | YAML keys become CLI flags | `model: opus` → `--model opus` |
| **Body → Prompt** | Markdown body is the final argument | Full prompt text |
| **`@file` imports** | Inline file contents | `@./src/**/*.ts` |
| **`!command` inlines** | Execute and inline output | `` !`git log -5` `` |
| **`{{_stdin}}`** | Piped input variable | Template substitution |
| **`{{_1}}`, `{{_2}}`** | Positional arguments | CLI arg placeholders |
| **`.i.` marker** | Interactive mode | `task.i.claude.md` |

## Current State Analysis

### Existing Patterns to Follow

| Feature | Scripts | Scriptlets | Agents (Planned) |
|---------|---------|------------|------------------|
| **Location** | `~/.scriptkit/*/scripts/*.ts` | `~/.scriptkit/*/scriptlets/*.md` | `~/.scriptkit/*/agents/*.md` |
| **Struct** | `Script` | `Scriptlet` | `Agent` |
| **Match type** | `ScriptMatch` | `ScriptletMatch` | `AgentMatch` |
| **Search result** | `SearchResult::Script` | `SearchResult::Scriptlet` | `SearchResult::Agent` |
| **File watcher** | `ScriptWatcher` | (same watcher) | Extend `ScriptWatcher` |
| **Loader fn** | `read_scripts()` | `load_scriptlets()` | `load_agents()` |
| **Fuzzy search** | `fuzzy_search_scripts()` | `fuzzy_search_scriptlets()` | `fuzzy_search_agents()` |

### Key Files to Modify

| File | Purpose | Changes Needed |
|------|---------|----------------|
| `src/scripts.rs` | Core types and search | Add `Agent`, `AgentMatch`, extend `SearchResult` |
| `src/watcher.rs` | File watching | Watch `agents/` directory, add `AgentReloadEvent` |
| `src/agents.rs` | **NEW** | Agent parsing, frontmatter extraction, backend detection |
| `src/executor.rs` | Script execution | Add `execute_agent()` function |
| `src/app_impl.rs` | App state | Add `agents: Vec<Arc<Agent>>` field |
| `src/render_script_list.rs` | UI rendering | Handle `SearchResult::Agent` |
| `src/frecency.rs` | Recency tracking | Include agents in frecency store |

---

## Phase 1: Data Model

### Agent Struct

```rust
// src/agents.rs

/// Represents an mdflow agent parsed from a .md file
#[derive(Clone, Debug)]
pub struct Agent {
    /// Display name (from frontmatter or filename)
    pub name: String,
    /// File path to the .md file
    pub path: PathBuf,
    /// Backend inferred from filename (claude, gemini, codex, copilot)
    pub backend: AgentBackend,
    /// Whether this is interactive mode (has .i. in filename)
    pub interactive: bool,
    /// Description from frontmatter
    pub description: Option<String>,
    /// Icon name
    pub icon: Option<String>,
    /// Keyboard shortcut
    pub shortcut: Option<String>,
    /// Alias for quick triggering
    pub alias: Option<String>,
    /// Model override from frontmatter
    pub model: Option<String>,
    /// Raw frontmatter for CLI flag generation
    pub frontmatter: AgentFrontmatter,
    /// The kit this agent belongs to (e.g., "main", "custom-kit")
    pub kit: Option<String>,
}

/// Supported AI backends
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AgentBackend {
    Claude,
    Gemini,
    Codex,
    Copilot,
    /// Generic - requires --_command flag
    Generic,
}

impl AgentBackend {
    /// Parse backend from filename pattern
    /// e.g., "review.claude.md" → Claude
    pub fn from_filename(filename: &str) -> Self {
        let name = filename.to_lowercase();
        if name.contains(".claude.") { Self::Claude }
        else if name.contains(".gemini.") { Self::Gemini }
        else if name.contains(".codex.") { Self::Codex }
        else if name.contains(".copilot.") { Self::Copilot }
        else { Self::Generic }
    }
    
    /// Get the CLI command for this backend
    pub fn command(&self) -> Option<&'static str> {
        match self {
            Self::Claude => Some("claude"),
            Self::Gemini => Some("gemini"),
            Self::Codex => Some("codex"),
            Self::Copilot => Some("copilot"),
            Self::Generic => None,
        }
    }
    
    /// Icon for this backend
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Claude => "anthropic",
            Self::Gemini => "google",
            Self::Codex => "openai",
            Self::Copilot => "github",
            Self::Generic => "bot",
        }
    }
}

/// Parsed frontmatter from agent .md file
#[derive(Clone, Debug, Default)]
pub struct AgentFrontmatter {
    /// Model to use (e.g., "opus", "sonnet", "gemini-2.0-flash")
    pub model: Option<String>,
    /// Print mode (non-interactive)
    pub print: Option<bool>,
    /// Interactive mode override
    pub interactive: Option<bool>,
    /// MCP config file path
    pub mcp_config: Option<String>,
    /// Skip permission prompts
    pub dangerously_skip_permissions: Option<bool>,
    /// Template variables (keys starting with _)
    pub variables: HashMap<String, String>,
    /// All other keys → CLI flags
    pub extra: HashMap<String, serde_yaml::Value>,
}
```

### Search Integration

```rust
// In src/scripts.rs

/// Represents a scored match result for fuzzy search on agents
#[derive(Clone, Debug)]
pub struct AgentMatch {
    pub agent: Arc<Agent>,
    pub score: i32,
    /// The display name for matching
    pub display_name: String,
    /// Indices of matched characters for UI highlighting
    pub match_indices: MatchIndices,
}

// Extend SearchResult enum
pub enum SearchResult {
    Script(ScriptMatch),
    Scriptlet(ScriptletMatch),
    BuiltIn(BuiltInMatch),
    App(AppMatch),
    Window(WindowMatch),
    Agent(AgentMatch),  // NEW
}

impl SearchResult {
    pub fn name(&self) -> &str {
        match self {
            // ... existing matches ...
            SearchResult::Agent(am) => &am.agent.name,
        }
    }
    
    pub fn description(&self) -> Option<&str> {
        match self {
            // ... existing matches ...
            SearchResult::Agent(am) => am.agent.description.as_deref(),
        }
    }
    
    pub fn type_label(&self) -> &'static str {
        match self {
            // ... existing matches ...
            SearchResult::Agent(_) => "Agent",
        }
    }
}
```

---

## Phase 2: Parsing

### Frontmatter Parser

```rust
// src/agents.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Parse agent frontmatter from markdown content
/// 
/// Frontmatter format:
/// ```yaml
/// ---
/// model: opus
/// _feature: "default value"
/// dangerously-skip-permissions: true
/// ---
/// ```
pub fn parse_agent_frontmatter(content: &str) -> Option<AgentFrontmatter> {
    let trimmed = content.trim_start();
    
    // Must start with ---
    if !trimmed.starts_with("---") {
        return None;
    }
    
    // Find closing ---
    let after_first = &trimmed[3..];
    let end_pos = after_first.find("\n---")?;
    let yaml_content = &after_first[..end_pos].trim();
    
    // Parse as generic YAML first
    let raw: HashMap<String, serde_yaml::Value> = 
        serde_yaml::from_str(yaml_content).ok()?;
    
    let mut frontmatter = AgentFrontmatter::default();
    
    for (key, value) in raw {
        match key.as_str() {
            "model" => {
                frontmatter.model = value.as_str().map(|s| s.to_string());
            }
            "print" | "_print" => {
                frontmatter.print = value.as_bool();
            }
            "_interactive" | "_i" => {
                frontmatter.interactive = value.as_bool().or(Some(true));
            }
            "mcp-config" | "mcp_config" => {
                frontmatter.mcp_config = value.as_str().map(|s| s.to_string());
            }
            "dangerously-skip-permissions" => {
                frontmatter.dangerously_skip_permissions = value.as_bool();
            }
            _ if key.starts_with('_') => {
                // Template variable
                if let Some(s) = value.as_str() {
                    frontmatter.variables.insert(key, s.to_string());
                }
            }
            _ => {
                // Other key → CLI flag
                frontmatter.extra.insert(key, value);
            }
        }
    }
    
    Some(frontmatter)
}

/// Parse agent metadata from file path and content
pub fn parse_agent(path: &Path, content: &str) -> Option<Agent> {
    let filename = path.file_name()?.to_str()?;
    
    // Skip hidden files
    if filename.starts_with('.') {
        return None;
    }
    
    // Must be .md file
    if !filename.ends_with(".md") {
        return None;
    }
    
    // Parse backend from filename
    let backend = AgentBackend::from_filename(filename);
    
    // Check for interactive marker (.i.)
    let interactive = filename.contains(".i.");
    
    // Parse frontmatter
    let frontmatter = parse_agent_frontmatter(content).unwrap_or_default();
    
    // Extract name: prefer frontmatter name, fall back to filename
    let name = frontmatter.extra
        .get("name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            // Remove .md and backend suffix for display name
            filename
                .trim_end_matches(".md")
                .split('.')
                .next()
                .unwrap_or(filename)
                .replace('-', " ")
                .replace('_', " ")
        });
    
    // Extract optional metadata
    let description = frontmatter.extra
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    let icon = frontmatter.extra
        .get("icon")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    let shortcut = frontmatter.extra
        .get("shortcut")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    let alias = frontmatter.extra
        .get("alias")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    Some(Agent {
        name,
        path: path.to_path_buf(),
        backend,
        interactive,
        description,
        icon,
        shortcut,
        alias,
        model: frontmatter.model.clone(),
        frontmatter,
        kit: None, // Set by loader
    })
}
```

### Loader Function

```rust
// src/agents.rs

use glob::glob;
use std::sync::Arc;
use tracing::{debug, warn};
use crate::setup::get_kit_path;

/// Load agents from all kits
/// 
/// Globs: ~/.scriptkit/*/agents/*.md
/// 
/// Returns Arc-wrapped agents sorted by name.
pub fn load_agents() -> Vec<Arc<Agent>> {
    let kit_path = get_kit_path();
    let mut agents = Vec::new();
    
    let pattern = kit_path.join("*/agents/*.md");
    let pattern_str = pattern.to_string_lossy().to_string();
    
    debug!(pattern = %pattern_str, "Globbing for agent files");
    
    match glob(&pattern_str) {
        Ok(paths) => {
            for entry in paths {
                match entry {
                    Ok(path) => {
                        debug!(path = %path.display(), "Parsing agent file");
                        
                        match std::fs::read_to_string(&path) {
                            Ok(content) => {
                                if let Some(mut agent) = parse_agent(&path, &content) {
                                    // Extract kit from path
                                    agent.kit = extract_kit_from_path(&path, &kit_path);
                                    agents.push(Arc::new(agent));
                                }
                            }
                            Err(e) => {
                                warn!(
                                    error = %e,
                                    path = %path.display(),
                                    "Failed to read agent file"
                                );
                            }
                        }
                    }
                    Err(e) => {
                        warn!(error = %e, "Failed to process glob entry");
                    }
                }
            }
        }
        Err(e) => {
            warn!(
                error = %e,
                pattern = %pattern_str,
                "Failed to glob agent files"
            );
        }
    }
    
    // Sort by name
    agents.sort_by(|a, b| a.name.cmp(&b.name));
    
    debug!(count = agents.len(), "Loaded agents");
    agents
}

/// Extract kit name from path
fn extract_kit_from_path(path: &Path, kit_root: &Path) -> Option<String> {
    let kit_prefix = format!("{}/", kit_root.display());
    let path_str = path.to_string_lossy();
    
    if path_str.starts_with(&kit_prefix) {
        let relative = &path_str[kit_prefix.len()..];
        relative.split('/').next().map(|s| s.to_string())
    } else {
        None
    }
}
```

---

## Phase 3: File Watching

### Extend ScriptWatcher

```rust
// src/watcher.rs

/// Event types for agent file changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentReloadEvent {
    /// A specific agent file was modified
    FileChanged(PathBuf),
    /// A new agent file was created
    FileCreated(PathBuf),
    /// An agent file was deleted
    FileDeleted(PathBuf),
    /// Full reload needed
    FullReload,
}

/// Check if path is a relevant agent file
fn is_agent_file(path: &Path) -> bool {
    // Skip hidden files
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if name.starts_with('.') {
            return false;
        }
    }
    
    // Must be .md file
    if path.extension().and_then(|e| e.to_str()) != Some("md") {
        return false;
    }
    
    // Must be in an agents/ directory
    path.parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map(|n| n == "agents")
        .unwrap_or(false)
}

// In ScriptWatcher::watch_loop(), add:
// 1. Watch ~/.scriptkit/*/agents/ directories
// 2. Filter for .md files in agents/ subdirs
// 3. Emit AgentReloadEvent for changes
```

---

## Phase 4: Fuzzy Search

```rust
// src/scripts.rs

/// Fuzzy search agents by query string
/// Searches across name, description, backend, and alias
pub fn fuzzy_search_agents(agents: &[Arc<Agent>], query: &str) -> Vec<AgentMatch> {
    if query.is_empty() {
        return agents
            .iter()
            .map(|a| AgentMatch {
                agent: Arc::clone(a),
                score: 0,
                display_name: a.name.clone(),
                match_indices: MatchIndices::default(),
            })
            .collect();
    }
    
    let query_lower = query.to_lowercase();
    let mut matches = Vec::new();
    
    let pattern = Pattern::parse(
        &query_lower,
        nucleo_matcher::pattern::CaseMatching::Ignore,
        nucleo_matcher::pattern::Normalization::Smart,
    );
    let mut matcher = Matcher::new(nucleo_matcher::Config::DEFAULT);
    
    for agent in agents {
        let mut score = 0i32;
        
        // Score by name match - highest priority
        if let Some(pos) = find_ignore_ascii_case(&agent.name, &query_lower) {
            score += if pos == 0 { 100 } else { 75 };
        }
        
        // Nucleo fuzzy matching on name
        if let Some(nucleo_s) = nucleo_score(&agent.name, &pattern, &mut matcher) {
            score += 50 + (nucleo_s / 20) as i32;
        }
        
        // Score by backend name
        if let Some(cmd) = agent.backend.command() {
            if contains_ignore_ascii_case(cmd, &query_lower) {
                score += 40;
            }
        }
        
        // Score by description
        if let Some(ref desc) = agent.description {
            if contains_ignore_ascii_case(desc, &query_lower) {
                score += 25;
            }
        }
        
        // Score by alias
        if let Some(ref alias) = agent.alias {
            if contains_ignore_ascii_case(alias, &query_lower) {
                score += 60;
            }
        }
        
        if score > 0 {
            matches.push(AgentMatch {
                agent: Arc::clone(agent),
                score,
                display_name: agent.name.clone(),
                match_indices: MatchIndices::default(),
            });
        }
    }
    
    matches.sort_by(|a, b| match b.score.cmp(&a.score) {
        Ordering::Equal => a.agent.name.cmp(&b.agent.name),
        other => other,
    });
    
    matches
}
```

---

## Phase 5: Execution

### Agent Executor

```rust
// src/agents.rs or src/executor.rs

use std::process::{Command, Stdio};

/// Execute an agent using mdflow
/// 
/// Flow:
/// 1. Build CLI args from frontmatter
/// 2. Spawn `mdflow <agent.md> [args...]`
/// 3. Pipe stdin if provided
/// 4. Return stdout/stderr
pub fn execute_agent(
    agent: &Agent,
    positional_args: &[String],
    stdin_input: Option<&str>,
) -> anyhow::Result<std::process::Child> {
    let mut cmd = Command::new("mdflow");
    
    // Add the agent file path
    cmd.arg(&agent.path);
    
    // Add positional arguments
    for arg in positional_args {
        cmd.arg(arg);
    }
    
    // Convert frontmatter to CLI flags
    for (key, value) in &agent.frontmatter.extra {
        add_frontmatter_flag(&mut cmd, key, value);
    }
    
    // Add template variable overrides
    for (key, value) in &agent.frontmatter.variables {
        cmd.arg(format!("--{}", key));
        cmd.arg(value);
    }
    
    // Set up I/O
    cmd.stdin(if stdin_input.is_some() { Stdio::piped() } else { Stdio::null() });
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    
    let mut child = cmd.spawn()?;
    
    // Write stdin if provided
    if let Some(input) = stdin_input {
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin.write_all(input.as_bytes())?;
        }
    }
    
    Ok(child)
}

/// Convert YAML value to CLI flag(s)
fn add_frontmatter_flag(cmd: &mut Command, key: &str, value: &serde_yaml::Value) {
    let flag = if key.len() == 1 {
        format!("-{}", key)
    } else {
        format!("--{}", key)
    };
    
    match value {
        serde_yaml::Value::Bool(true) => {
            cmd.arg(&flag);
        }
        serde_yaml::Value::Bool(false) => {
            // Omit false booleans
        }
        serde_yaml::Value::String(s) => {
            cmd.arg(&flag);
            cmd.arg(s);
        }
        serde_yaml::Value::Number(n) => {
            cmd.arg(&flag);
            cmd.arg(n.to_string());
        }
        serde_yaml::Value::Sequence(arr) => {
            // Repeated flags for arrays
            for item in arr {
                if let Some(s) = item.as_str() {
                    cmd.arg(&flag);
                    cmd.arg(s);
                }
            }
        }
        _ => {}
    }
}
```

### Interactive Agent Session

For interactive agents (`.i.` marker or `_interactive: true`), we need special handling:

```rust
/// Start an interactive agent session
/// 
/// This creates a pseudo-terminal for the AI CLI to interact with the user.
pub fn execute_agent_interactive(
    agent: &Agent,
    positional_args: &[String],
) -> anyhow::Result<AgentSession> {
    // Interactive agents need a PTY
    // Similar to how we handle terminal prompts in term_prompt.rs
    
    // For MVP, we can shell out to mdflow in a terminal window
    // Later: embed using portable-pty like we do for term()
    
    todo!("Implement interactive agent session")
}
```

---

## Phase 6: UI Integration

### Main Menu Rendering

```rust
// src/render_script_list.rs

// In the list item rendering closure, add handling for Agent:

GroupedListItem::Item(result_idx) => {
    if let Some(result) = flat_results_clone.get(*result_idx) {
        match result {
            // ... existing Script, Scriptlet, BuiltIn handlers ...
            
            SearchResult::Agent(am) => {
                // Render agent list item
                render_agent_item(
                    &am.agent,
                    ix,
                    is_selected,
                    is_hovered,
                    theme_colors,
                )
            }
        }
    }
}

fn render_agent_item(
    agent: &Agent,
    ix: usize,
    is_selected: bool,
    is_hovered: bool,
    colors: ListItemColors,
) -> AnyElement {
    let bg_color = if is_selected {
        colors.background_selected
    } else if is_hovered {
        colors.background_hovered
    } else {
        colors.background
    };
    
    let backend_badge = match agent.backend {
        AgentBackend::Claude => "Claude",
        AgentBackend::Gemini => "Gemini",
        AgentBackend::Codex => "Codex",
        AgentBackend::Copilot => "Copilot",
        AgentBackend::Generic => "Agent",
    };
    
    div()
        .id(ElementId::NamedInteger("agent-item".into(), ix as u64))
        .h(px(LIST_ITEM_HEIGHT))
        .w_full()
        .flex()
        .flex_row()
        .items_center()
        .px(px(16.))
        .gap(px(12.))
        .bg(rgb(bg_color))
        // Icon
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .size(px(24.))
                .child(
                    svg()
                        .path(format!("icons/{}.svg", agent.icon.as_deref()
                            .unwrap_or(agent.backend.icon())))
                        .size(px(16.))
                        .text_color(rgb(colors.text_secondary))
                )
        )
        // Name and description
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .overflow_hidden()
                .child(
                    div()
                        .text_color(rgb(colors.text_primary))
                        .text_ellipsis()
                        .child(agent.name.clone())
                )
                .when_some(agent.description.clone(), |d, desc| {
                    d.child(
                        div()
                            .text_color(rgb(colors.text_secondary))
                            .text_xs()
                            .text_ellipsis()
                            .child(desc)
                    )
                })
        )
        // Backend badge
        .child(
            div()
                .px(px(8.))
                .py(px(2.))
                .rounded(px(4.))
                .bg(rgba((colors.accent << 8) | 0x30))
                .text_xs()
                .text_color(rgb(colors.text_secondary))
                .child(backend_badge)
        )
        .into_any_element()
}
```

### Grouped Results Integration

```rust
// src/scripts.rs - extend get_grouped_results()

// Add agents to the unified search
pub fn fuzzy_search_unified_with_agents(
    scripts: &[Arc<Script>],
    scriptlets: &[Arc<Scriptlet>],
    builtins: &[BuiltInEntry],
    apps: &[AppInfo],
    agents: &[Arc<Agent>],  // NEW
    query: &str,
) -> Vec<SearchResult> {
    let mut results = Vec::new();
    
    // ... existing builtin, app, script, scriptlet searches ...
    
    // Search agents
    let agent_matches = fuzzy_search_agents(agents, query);
    for am in agent_matches {
        results.push(SearchResult::Agent(am));
    }
    
    // Sort with agents after apps but before scripts
    results.sort_by(|a, b| {
        match b.score().cmp(&a.score()) {
            Ordering::Equal => {
                let type_order = |r: &SearchResult| -> i32 {
                    match r {
                        SearchResult::BuiltIn(_) => 0,
                        SearchResult::App(_) => 1,
                        SearchResult::Agent(_) => 2,  // NEW
                        SearchResult::Window(_) => 3,
                        SearchResult::Script(_) => 4,
                        SearchResult::Scriptlet(_) => 5,
                    }
                };
                type_order(a).cmp(&type_order(b))
            }
            other => other,
        }
    });
    
    results
}
```

---

## Phase 7: App State Integration

```rust
// src/app_impl.rs

impl ScriptListApp {
    // Add agents field
    agents: Vec<Arc<Agent>>,
    
    // In new() or initialize:
    pub fn load_all_content(&mut self) {
        self.scripts = scripts::read_scripts();
        self.scriptlets = scripts::load_scriptlets();
        self.agents = agents::load_agents();  // NEW
        
        // Invalidate caches
        self.results_cache.dirty = true;
        self.grouped_cache.dirty = true;
    }
    
    // In execute_selected():
    pub fn execute_selected(&mut self, cx: &mut Context<Self>) {
        let (_, results) = self.get_grouped_results_cached();
        
        if let Some(grouped_item) = self.grouped_items.get(self.selected_index) {
            if let GroupedListItem::Item(result_idx) = grouped_item {
                if let Some(result) = results.get(*result_idx) {
                    match result {
                        SearchResult::Script(sm) => self.execute_script(&sm.script, cx),
                        SearchResult::Scriptlet(sm) => self.execute_scriptlet(&sm.scriptlet, cx),
                        SearchResult::Agent(am) => self.execute_agent(&am.agent, cx),  // NEW
                        // ... other handlers ...
                    }
                }
            }
        }
    }
    
    // New agent execution handler
    fn execute_agent(&mut self, agent: &Agent, cx: &mut Context<Self>) {
        logging::log("EXEC", &format!("Executing agent: {}", agent.name));
        
        // Record frecency
        let agent_key = format!("agent:{}", agent.path.display());
        self.frecency_store.record_access(&agent_key);
        
        // Execute via mdflow
        match agents::execute_agent(agent, &[], None) {
            Ok(child) => {
                // Handle output similar to script execution
                // For interactive agents, may need terminal integration
                self.handle_agent_process(child, agent, cx);
            }
            Err(e) => {
                logging::log("EXEC", &format!("Agent execution failed: {}", e));
                // Show error to user
            }
        }
    }
}
```

---

## Phase 8: Testing

### Test Cases

```rust
#[cfg(test)]
mod agent_tests {
    use super::*;
    
    #[test]
    fn test_backend_from_filename() {
        assert_eq!(
            AgentBackend::from_filename("review.claude.md"),
            AgentBackend::Claude
        );
        assert_eq!(
            AgentBackend::from_filename("task.gemini.md"),
            AgentBackend::Gemini
        );
        assert_eq!(
            AgentBackend::from_filename("analyze.i.codex.md"),
            AgentBackend::Codex
        );
        assert_eq!(
            AgentBackend::from_filename("generic.md"),
            AgentBackend::Generic
        );
    }
    
    #[test]
    fn test_interactive_detection() {
        let agent = parse_agent(
            Path::new("task.i.claude.md"),
            "---\nmodel: opus\n---\nPrompt here"
        ).unwrap();
        assert!(agent.interactive);
        
        let agent2 = parse_agent(
            Path::new("task.claude.md"),
            "---\nmodel: opus\n---\nPrompt here"
        ).unwrap();
        assert!(!agent2.interactive);
    }
    
    #[test]
    fn test_frontmatter_parsing() {
        let content = r#"---
model: opus
_feature_name: Authentication
dangerously-skip-permissions: true
add-dir:
  - ./src
  - ./tests
---
Build {{ _feature_name }}.
"#;
        
        let fm = parse_agent_frontmatter(content).unwrap();
        assert_eq!(fm.model, Some("opus".to_string()));
        assert_eq!(fm.dangerously_skip_permissions, Some(true));
        assert!(fm.variables.contains_key("_feature_name"));
        assert!(fm.extra.contains_key("add-dir"));
    }
}
```

### Smoke Test

```typescript
// tests/smoke/test-agents.ts
import '../../scripts/kit-sdk';

export const metadata = {
  name: "Agent Integration Test",
  description: "Tests agent loading and display",
};

console.error('[SMOKE] Testing agent integration...');

// Create a test agent file
const agentPath = join(process.env.HOME!, '.scriptkit/main/agents/test.claude.md');
await writeFile(agentPath, `---
model: sonnet
description: Test agent for smoke testing
---
Hello from test agent!
`);

// Verify it appears in search (would need SDK function)
// For now, just verify file was created
console.error('[SMOKE] Agent file created at:', agentPath);

process.exit(0);
```

---

## Implementation Order

1. **Phase 1**: Data model (`src/agents.rs` - structs only)
2. **Phase 2**: Parsing (frontmatter + file parsing)
3. **Phase 3**: Loader function (`load_agents()`)
4. **Phase 4**: Search integration (extend `SearchResult`, add `fuzzy_search_agents`)
5. **Phase 5**: File watcher (extend `ScriptWatcher`)
6. **Phase 6**: App state integration (add `agents` field, handle reload events)
7. **Phase 7**: UI rendering (`render_agent_item`)
8. **Phase 8**: Execution (spawn mdflow process)
9. **Phase 9**: Testing (unit tests + smoke tests)

---

## Open Questions

1. **Interactive mode handling**: Should interactive agents open in a terminal window or embed in the Script Kit UI?
   - Option A: Open system terminal (simple, reliable)
   - Option B: Embed with PTY (consistent UX, more complex)
   
2. **mdflow installation**: Should Script Kit bundle mdflow or require user installation?
   - Option A: Bundle as npm dependency
   - Option B: Auto-install on first use
   - Option C: Require manual installation, show helpful error
   
3. **Template variable UI**: When an agent has `_inputs`, should we show a form like scriptlets do?
   - Could leverage existing `fields()` prompt type
   
4. **Streaming output**: Should agent output stream to a div in real-time or wait for completion?
   - For print mode: stream to panel
   - For interactive mode: full terminal needed

5. **Backend availability**: How to handle missing backends (e.g., user doesn't have Claude CLI)?
   - Check on startup and show warning
   - Check on execution and show helpful install instructions

---

## Dependencies

- **mdflow**: `npm install -g mdflow` or bundled
- **AI CLIs**: claude, gemini, codex, copilot (user-installed)
- **Existing**: All current Script Kit infrastructure

## Success Criteria

- [ ] Agents in `~/.scriptkit/*/agents/*.md` appear in main menu
- [ ] Fuzzy search works across agent name, description, backend
- [ ] File watcher detects new/modified/deleted agents
- [ ] Executing an agent spawns mdflow with correct arguments
- [ ] Backend badge shows correct AI provider
- [ ] Frecency tracking includes agents
- [ ] Interactive agents work (basic terminal integration)

</file>

</files>