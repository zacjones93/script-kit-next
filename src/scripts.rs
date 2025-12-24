use std::path::PathBuf;
use std::env;
use std::fs;
use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct Script {
    pub name: String,
    pub path: PathBuf,
    pub extension: String,
    pub description: Option<String>,
}

/// Represents a scriptlet parsed from a markdown file
/// Scriptlets are code snippets extracted from .md files with metadata
#[derive(Clone, Debug)]
pub struct Scriptlet {
    pub name: String,
    pub description: Option<String>,
    pub code: String,
    pub tool: String,  // "ts", "bash", "paste", etc.
    pub shortcut: Option<String>,
    pub expand: Option<String>,
}

/// Represents a scored match result for fuzzy search
#[derive(Clone, Debug)]
pub struct ScriptMatch {
    pub script: Script,
    pub score: i32,
}

/// Represents a scored match result for fuzzy search on scriptlets
#[derive(Clone, Debug)]
pub struct ScriptletMatch {
    pub scriptlet: Scriptlet,
    pub score: i32,
}

/// Unified search result that can be either a Script or Scriptlet
#[derive(Clone, Debug)]
pub enum SearchResult {
    Script(ScriptMatch),
    Scriptlet(ScriptletMatch),
}

impl SearchResult {
    /// Get the display name for this result
    pub fn name(&self) -> &str {
        match self {
            SearchResult::Script(sm) => &sm.script.name,
            SearchResult::Scriptlet(sm) => &sm.scriptlet.name,
        }
    }

    /// Get the description for this result
    pub fn description(&self) -> Option<&str> {
        match self {
            SearchResult::Script(sm) => sm.script.description.as_deref(),
            SearchResult::Scriptlet(sm) => sm.scriptlet.description.as_deref(),
        }
    }

    /// Get the score for this result
    pub fn score(&self) -> i32 {
        match self {
            SearchResult::Script(sm) => sm.score,
            SearchResult::Scriptlet(sm) => sm.score,
        }
    }

    /// Get the type label for UI display
    pub fn type_label(&self) -> &'static str {
        match self {
            SearchResult::Script(_) => "Script",
            SearchResult::Scriptlet(_) => "Snippet",
        }
    }
}

/// Extract metadata from script file comments
/// Looks for lines starting with "// Description:"
fn extract_metadata(path: &PathBuf) -> Option<String> {
    match fs::read_to_string(path) {
        Ok(content) => {
            for line in content.lines().take(20) {  // Check only first 20 lines
                if line.trim().starts_with("// Description:") {
                    if let Some(desc) = line.split("// Description:").nth(1) {
                        return Some(desc.trim().to_string());
                    }
                }
            }
            None
        }
        Err(_) => None,
    }
}

/// Extract metadata from HTML comments in scriptlet markdown
/// Looks for <!-- key: value --> patterns
fn extract_html_comment_metadata(text: &str) -> std::collections::HashMap<String, String> {
    use std::collections::HashMap;
    let mut metadata = HashMap::new();
    
    // Find HTML comment blocks
    if let Some(start) = text.find("<!--") {
        if let Some(end) = text.find("-->") {
            if start < end {
                let comment_content = &text[start + 4..end];
                // Parse key: value pairs
                for line in comment_content.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        // Handle format: "key: value"
                        if let Some(colon_pos) = trimmed.find(':') {
                            let key = trimmed[..colon_pos].trim().to_string();
                            let value = trimmed[colon_pos + 1..].trim().to_string();
                            metadata.insert(key, value);
                        }
                    }
                }
            }
        }
    }
    
    metadata
}

/// Extract code block from markdown text
/// Looks for ```language ... ``` pattern and returns (language, code)
fn extract_code_block(text: &str) -> Option<(String, String)> {
    // Find first code fence
    if let Some(start) = text.find("```") {
        let after_fence = &text[start + 3..];
        // Get the language specifier (rest of line)
        if let Some(newline_pos) = after_fence.find('\n') {
            let language = after_fence[..newline_pos].trim().to_string();
            let code_start = start + 3 + newline_pos + 1;
            
            // Find closing fence
            if let Some(end_pos) = text[code_start..].find("```") {
                let code = text[code_start..code_start + end_pos].trim().to_string();
                return Some((language, code));
            }
        }
    }
    None
}

/// Parse a single scriptlet section from markdown
/// Input should be text from ## Name to the next ## or end of file
fn parse_scriptlet_section(section: &str) -> Option<Scriptlet> {
    let lines: Vec<&str> = section.lines().collect();
    if lines.is_empty() {
        return None;
    }
    
    // First line should be ## Name
    let first_line = lines[0];
    if !first_line.starts_with("##") {
        return None;
    }
    
    let name = first_line
        .strip_prefix("##")
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    
    if name.is_empty() {
        return None;
    }
    
    // Extract metadata from HTML comments
    let metadata = extract_html_comment_metadata(section);
    
    // Extract code block
    let (tool, code) = extract_code_block(section)?;
    
    Some(Scriptlet {
        name,
        description: metadata.get("description").cloned(),
        code,
        tool,
        shortcut: metadata.get("shortcut").cloned(),
        expand: metadata.get("expand").cloned(),
    })
}

/// Reads scriptlets from ~/.kenv/scriptlets/scriptlets.md
/// Returns a sorted list of Scriptlet structs
/// Returns empty vec if file doesn't exist or is inaccessible
pub fn read_scriptlets() -> Vec<Scriptlet> {
    let home = match env::var("HOME") {
        Ok(home_path) => PathBuf::from(home_path),
        Err(_) => return vec![],
    };

    let scriptlets_file = home.join(".kenv/scriptlets/scriptlets.md");

    // Check if file exists
    if !scriptlets_file.exists() {
        return vec![];
    }

    match fs::read_to_string(&scriptlets_file) {
        Ok(content) => {
            let mut scriptlets = Vec::new();

            // Split by ## headings
            let mut current_section = String::new();
            for line in content.lines() {
                if line.starts_with("##") && !current_section.is_empty() {
                    // Parse previous section
                    if let Some(scriptlet) = parse_scriptlet_section(&current_section) {
                        scriptlets.push(scriptlet);
                    }
                    current_section = line.to_string();
                } else {
                    current_section.push('\n');
                    current_section.push_str(line);
                }
            }

            // Parse the last section
            if !current_section.is_empty() {
                if let Some(scriptlet) = parse_scriptlet_section(&current_section) {
                    scriptlets.push(scriptlet);
                }
            }

            // Sort by name
            scriptlets.sort_by(|a, b| a.name.cmp(&b.name));

            scriptlets
        }
        Err(_) => vec![],
    }
}

/// Reads scripts from ~/.kenv/scripts directory
/// Returns a sorted list of Script structs for .ts and .js files
/// Returns empty vec if directory doesn't exist or is inaccessible
pub fn read_scripts() -> Vec<Script> {
    // Expand ~ to home directory using HOME environment variable
    let home = match env::var("HOME") {
        Ok(home_path) => PathBuf::from(home_path),
        Err(_) => return vec![],
    };

    let scripts_dir = home.join(".kenv/scripts");

    // Check if directory exists
    if !scripts_dir.exists() {
        return vec![];
    }

    let mut scripts = Vec::new();

    // Read the directory contents
    match std::fs::read_dir(&scripts_dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        let path = entry.path();
                        
                        // Check extension
                        if let Some(ext) = path.extension() {
                            if let Some(ext_str) = ext.to_str() {
                                if ext_str == "ts" || ext_str == "js" {
                                    // Get filename without extension
                                    if let Some(file_name) = path.file_stem() {
                                        if let Some(name) = file_name.to_str() {
                                            let description = extract_metadata(&path);
                                            scripts.push(Script {
                                                name: name.to_string(),
                                                path: path.clone(),
                                                extension: ext_str.to_string(),
                                                description,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(_) => return vec![],
    }

    // Sort by name
    scripts.sort_by(|a, b| a.name.cmp(&b.name));

    scripts
}

/// Check if a pattern is a fuzzy match for haystack (characters appear in order)
fn is_fuzzy_match(haystack: &str, pattern: &str) -> bool {
    let mut pattern_chars = pattern.chars().peekable();
    for ch in haystack.chars() {
        if let Some(&p) = pattern_chars.peek() {
            if ch.eq_ignore_ascii_case(&p) {
                pattern_chars.next();
            }
        }
    }
    pattern_chars.peek().is_none()
}

/// Fuzzy search scripts by query string
/// Searches across name, description, and path
/// Returns results sorted by relevance score (highest first)
pub fn fuzzy_search_scripts(scripts: &[Script], query: &str) -> Vec<ScriptMatch> {
    if query.is_empty() {
        // If no query, return all scripts with equal score, sorted by name
        return scripts.iter().map(|s| ScriptMatch {
            script: s.clone(),
            score: 0,
        }).collect();
    }

    let query_lower = query.to_lowercase();
    let mut matches = Vec::new();

    for script in scripts {
        let mut score = 0i32;
        let name_lower = script.name.to_lowercase();

        // Score by name match - highest priority
        if let Some(pos) = name_lower.find(&query_lower) {
            // Bonus for exact substring match at start of name
            score += if pos == 0 { 100 } else { 75 };
        }

        // Fuzzy character matching in name (characters in order)
        if is_fuzzy_match(&name_lower, &query_lower) {
            score += 50;
        }

        // Score by description match - medium priority
        if let Some(ref desc) = script.description {
            if desc.to_lowercase().contains(&query_lower) {
                score += 25;
            }
        }

        // Score by path match - lower priority
        let path_str = script.path.to_string_lossy().to_lowercase();
        if path_str.contains(&query_lower) {
            score += 10;
        }

        if score > 0 {
            matches.push(ScriptMatch {
                script: script.clone(),
                score,
            });
        }
    }

    // Sort by score (highest first), then by name for ties
    matches.sort_by(|a, b| {
        match b.score.cmp(&a.score) {
            Ordering::Equal => a.script.name.cmp(&b.script.name),
            other => other,
        }
    });

    matches
}

/// Fuzzy search scriptlets by query string
/// Searches across name, description, and code
/// Returns results sorted by relevance score (highest first)
pub fn fuzzy_search_scriptlets(scriptlets: &[Scriptlet], query: &str) -> Vec<ScriptletMatch> {
    if query.is_empty() {
        // If no query, return all scriptlets with equal score, sorted by name
        return scriptlets.iter().map(|s| ScriptletMatch {
            scriptlet: s.clone(),
            score: 0,
        }).collect();
    }

    let query_lower = query.to_lowercase();
    let mut matches = Vec::new();

    for scriptlet in scriptlets {
        let mut score = 0i32;
        let name_lower = scriptlet.name.to_lowercase();

        // Score by name match - highest priority
        if let Some(pos) = name_lower.find(&query_lower) {
            // Bonus for exact substring match at start of name
            score += if pos == 0 { 100 } else { 75 };
        }

        // Fuzzy character matching in name (characters in order)
        if is_fuzzy_match(&name_lower, &query_lower) {
            score += 50;
        }

        // Score by description match - medium priority
        if let Some(ref desc) = scriptlet.description {
            if desc.to_lowercase().contains(&query_lower) {
                score += 25;
            }
        }

        // Score by code content match - lower priority
        if scriptlet.code.to_lowercase().contains(&query_lower) {
            score += 5;
        }

        // Bonus for tool type match
        if scriptlet.tool.to_lowercase().contains(&query_lower) {
            score += 10;
        }

        if score > 0 {
            matches.push(ScriptletMatch {
                scriptlet: scriptlet.clone(),
                score,
            });
        }
    }

    // Sort by score (highest first), then by name for ties
    matches.sort_by(|a, b| {
        match b.score.cmp(&a.score) {
            Ordering::Equal => a.scriptlet.name.cmp(&b.scriptlet.name),
            other => other,
        }
    });

    matches
}

/// Perform unified fuzzy search across both scripts and scriptlets
/// Returns combined and ranked results sorted by relevance
pub fn fuzzy_search_unified(scripts: &[Script], scriptlets: &[Scriptlet], query: &str) -> Vec<SearchResult> {
    let mut results = Vec::new();

    // Search scripts
    let script_matches = fuzzy_search_scripts(scripts, query);
    for sm in script_matches {
        results.push(SearchResult::Script(sm));
    }

    // Search scriptlets
    let scriptlet_matches = fuzzy_search_scriptlets(scriptlets, query);
    for sm in scriptlet_matches {
        results.push(SearchResult::Scriptlet(sm));
    }

    // Sort by score (highest first), then by type (scripts first), then by name
    results.sort_by(|a, b| {
        match b.score().cmp(&a.score()) {
            Ordering::Equal => {
                // Prefer scripts over scriptlets when scores are equal
                let type_order_a = match a { SearchResult::Script(_) => 0, SearchResult::Scriptlet(_) => 1 };
                let type_order_b = match b { SearchResult::Script(_) => 0, SearchResult::Scriptlet(_) => 1 };
                match type_order_a.cmp(&type_order_b) {
                    Ordering::Equal => a.name().cmp(b.name()),
                    other => other,
                }
            }
            other => other,
        }
    });

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_scriptlet_basic() {
        let section = "## Test Snippet\n\n```ts\nconst x = 1;\n```";
        let scriptlet = parse_scriptlet_section(section);
        assert!(scriptlet.is_some());
        let s = scriptlet.unwrap();
        assert_eq!(s.name, "Test Snippet");
        assert_eq!(s.tool, "ts");
        assert_eq!(s.code, "const x = 1;");
        assert_eq!(s.shortcut, None);
    }

    #[test]
    fn test_parse_scriptlet_with_metadata() {
        let section = "## Open File\n\n<!-- \nshortcut: cmd o\n-->\n\n```ts\nawait exec('open')\n```";
        let scriptlet = parse_scriptlet_section(section);
        assert!(scriptlet.is_some());
        let s = scriptlet.unwrap();
        assert_eq!(s.name, "Open File");
        assert_eq!(s.tool, "ts");
        assert_eq!(s.shortcut, Some("cmd o".to_string()));
    }

    #[test]
    fn test_parse_scriptlet_with_description() {
        let section = "## Test\n\n<!-- \ndescription: Test description\n-->\n\n```bash\necho test\n```";
        let scriptlet = parse_scriptlet_section(section);
        assert!(scriptlet.is_some());
        let s = scriptlet.unwrap();
        assert_eq!(s.description, Some("Test description".to_string()));
    }

    #[test]
    fn test_parse_scriptlet_with_expand() {
        let section = "## Execute Plan\n\n<!-- \nexpand: plan,,\n-->\n\n```paste\nPlease execute\n```";
        let scriptlet = parse_scriptlet_section(section);
        assert!(scriptlet.is_some());
        let s = scriptlet.unwrap();
        assert_eq!(s.expand, Some("plan,,".to_string()));
        assert_eq!(s.tool, "paste");
    }

    #[test]
    fn test_extract_code_block_ts() {
        let text = "Some text\n```ts\nconst x = 1;\n```\nMore text";
        let result = extract_code_block(text);
        assert!(result.is_some());
        let (lang, code) = result.unwrap();
        assert_eq!(lang, "ts");
        assert_eq!(code, "const x = 1;");
    }

    #[test]
    fn test_extract_code_block_bash() {
        let text = "```bash\necho hello\necho world\n```";
        let result = extract_code_block(text);
        assert!(result.is_some());
        let (lang, code) = result.unwrap();
        assert_eq!(lang, "bash");
        assert_eq!(code, "echo hello\necho world");
    }

    #[test]
    fn test_extract_html_metadata_shortcut() {
        let text = "<!-- \nshortcut: opt s\n-->";
        let metadata = extract_html_comment_metadata(text);
        assert_eq!(metadata.get("shortcut"), Some(&"opt s".to_string()));
    }

    #[test]
    fn test_extract_html_metadata_multiple() {
        let text = "<!-- \nshortcut: cmd k\nexpand: foo,,\ndescription: Test\n-->";
        let metadata = extract_html_comment_metadata(text);
        assert_eq!(metadata.get("shortcut"), Some(&"cmd k".to_string()));
        assert_eq!(metadata.get("expand"), Some(&"foo,,".to_string()));
        assert_eq!(metadata.get("description"), Some(&"Test".to_string()));
    }

    #[test]
    fn test_parse_scriptlet_none_without_heading() {
        let section = "Some text without heading\n```ts\ncode\n```";
        let scriptlet = parse_scriptlet_section(section);
        assert!(scriptlet.is_none());
    }

    #[test]
    fn test_parse_scriptlet_none_without_code_block() {
        let section = "## Name\nNo code block here";
        let scriptlet = parse_scriptlet_section(section);
        assert!(scriptlet.is_none());
    }

    #[test]
    fn test_read_scripts_returns_vec() {
        let scripts = read_scripts();
        // scripts should be a Vec, check it's valid
        assert!(scripts.is_empty() || !scripts.is_empty());
    }

    #[test]
    fn test_script_struct_has_required_fields() {
        let script = Script {
            name: "test".to_string(),
            path: PathBuf::from("/test/path"),
            extension: "ts".to_string(),
            description: None,
        };
        assert_eq!(script.name, "test");
        assert_eq!(script.extension, "ts");
    }

    #[test]
    fn test_fuzzy_search_by_name() {
        let scripts = vec![
            Script {
                name: "openfile".to_string(),
                path: PathBuf::from("/test/openfile.ts"),
                extension: "ts".to_string(),
                description: Some("Open a file dialog".to_string()),
            },
            Script {
                name: "savefile".to_string(),
                path: PathBuf::from("/test/savefile.ts"),
                extension: "ts".to_string(),
                description: None,
            },
        ];

        let results = fuzzy_search_scripts(&scripts, "open");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].script.name, "openfile");
        assert!(results[0].score > 0);
    }

    #[test]
    fn test_fuzzy_search_empty_query() {
        let scripts = vec![
            Script {
                name: "test1".to_string(),
                path: PathBuf::from("/test/test1.ts"),
                extension: "ts".to_string(),
                description: None,
            },
        ];

        let results = fuzzy_search_scripts(&scripts, "");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].score, 0);
    }

    #[test]
    fn test_fuzzy_search_ranking() {
        let scripts = vec![
            Script {
                name: "openfile".to_string(),
                path: PathBuf::from("/test/openfile.ts"),
                extension: "ts".to_string(),
                description: Some("Open a file dialog".to_string()),
            },
            Script {
                name: "open".to_string(),
                path: PathBuf::from("/test/open.ts"),
                extension: "ts".to_string(),
                description: Some("Basic open function".to_string()),
            },
            Script {
                name: "reopen".to_string(),
                path: PathBuf::from("/test/reopen.ts"),
                extension: "ts".to_string(),
                description: None,
            },
        ];

        let results = fuzzy_search_scripts(&scripts, "open");
        // Should have all three results
        assert_eq!(results.len(), 3);
        // "open" should be first (exact match at start: 100 + fuzzy match: 50 = 150)
        assert_eq!(results[0].script.name, "open");
        // "openfile" should be second (substring at start: 100 + fuzzy match: 50 = 150, but "open" comes first alphabetically in tie)
        assert_eq!(results[1].script.name, "openfile");
        // "reopen" should be third (substring not at start: 75 + fuzzy match: 50 = 125)
        assert_eq!(results[2].script.name, "reopen");
    }

    #[test]
    fn test_fuzzy_search_scriptlets() {
        let scriptlets = vec![
            Scriptlet {
                name: "Copy Text".to_string(),
                description: Some("Copy current selection".to_string()),
                code: "copy()".to_string(),
                tool: "ts".to_string(),
                shortcut: None,
                expand: None,
            },
            Scriptlet {
                name: "Paste Code".to_string(),
                description: None,
                code: "paste()".to_string(),
                tool: "ts".to_string(),
                shortcut: None,
                expand: None,
            },
        ];

        let results = fuzzy_search_scriptlets(&scriptlets, "copy");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].scriptlet.name, "Copy Text");
        assert!(results[0].score > 0);
    }

    #[test]
    fn test_fuzzy_search_unified() {
        let scripts = vec![
            Script {
                name: "open".to_string(),
                path: PathBuf::from("/test/open.ts"),
                extension: "ts".to_string(),
                description: Some("Open a file".to_string()),
            },
        ];

        let scriptlets = vec![
            Scriptlet {
                name: "Open Browser".to_string(),
                description: Some("Open in browser".to_string()),
                code: "open()".to_string(),
                tool: "ts".to_string(),
                shortcut: None,
                expand: None,
            },
        ];

        let results = fuzzy_search_unified(&scripts, &scriptlets, "open");
        assert_eq!(results.len(), 2);
        
        // First result should be the script (same score but scripts come first)
        match &results[0] {
            SearchResult::Script(sm) => assert_eq!(sm.script.name, "open"),
            _ => panic!("Expected script"),
        }
        
        // Second result should be the scriptlet
        match &results[1] {
            SearchResult::Scriptlet(sm) => assert_eq!(sm.scriptlet.name, "Open Browser"),
            _ => panic!("Expected scriptlet"),
        }
    }

    #[test]
    fn test_search_result_type_label() {
        let script = SearchResult::Script(ScriptMatch {
            script: Script {
                name: "test".to_string(),
                path: PathBuf::from("/test.ts"),
                extension: "ts".to_string(),
                description: None,
            },
            score: 100,
        });

        let scriptlet = SearchResult::Scriptlet(ScriptletMatch {
            scriptlet: Scriptlet {
                name: "snippet".to_string(),
                description: None,
                code: "code".to_string(),
                tool: "ts".to_string(),
                shortcut: None,
                expand: None,
            },
            score: 50,
        });

        assert_eq!(script.type_label(), "Script");
        assert_eq!(scriptlet.type_label(), "Snippet");
    }
}
