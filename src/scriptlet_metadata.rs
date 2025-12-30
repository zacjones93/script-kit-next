//! Scriptlet codefence metadata parser
//!
//! Parses `\`\`\`metadata` and `\`\`\`schema` codefence blocks from markdown scriptlets.
//! These blocks provide an alternative to the HTML comment metadata format, using
//! JSON directly in labeled code fences.
//!
//! # Example scriptlet with codefences:
//! ````markdown
//! # Quick Todo
//!
//! ```metadata
//! { "name": "Quick Todo", "description": "Add a todo item" }
//! ```
//!
//! ```schema
//! { "input": { "item": { "type": "string", "required": true } } }
//! ```
//!
//! ```ts
//! const { item } = await input();
//! await addTodo(item);
//! ```
//! ````

use crate::metadata_parser::TypedMetadata;
use crate::schema_parser::Schema;
use tracing::debug;

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
    /// The language identifier (e.g., "ts", "bash", "python")
    pub language: String,
    /// The code content
    pub content: String,
}

/// Parse codefence blocks from markdown scriptlet content
///
/// Looks for:
/// - `\`\`\`metadata\n{...}\n\`\`\`` - JSON metadata block
/// - `\`\`\`schema\n{...}\n\`\`\`` - JSON schema block  
/// - `\`\`\`<lang>\n...\n\`\`\`` - Main code block
///
/// # Arguments
/// * `content` - The markdown content to parse
///
/// # Returns
/// `CodefenceParseResult` with parsed metadata, schema, code, and any errors
pub fn parse_codefence_metadata(content: &str) -> CodefenceParseResult {
    let mut result = CodefenceParseResult::default();
    
    let blocks = extract_all_codefence_blocks(content);
    
    for (language, block_content) in blocks {
        match language.as_str() {
            "metadata" => {
                match serde_json::from_str::<TypedMetadata>(&block_content) {
                    Ok(metadata) => {
                        debug!(
                            name = ?metadata.name,
                            description = ?metadata.description,
                            "Parsed codefence metadata"
                        );
                        result.metadata = Some(metadata);
                    }
                    Err(e) => {
                        result.errors.push(format!("Failed to parse metadata JSON: {}", e));
                    }
                }
            }
            "schema" => {
                match serde_json::from_str::<Schema>(&block_content) {
                    Ok(schema) => {
                        debug!(
                            input_fields = schema.input.len(),
                            output_fields = schema.output.len(),
                            "Parsed codefence schema"
                        );
                        result.schema = Some(schema);
                    }
                    Err(e) => {
                        result.errors.push(format!("Failed to parse schema JSON: {}", e));
                    }
                }
            }
            // Skip empty language specifier
            "" => {}
            // Any other language is treated as code
            lang => {
                // Only capture the first non-metadata/schema code block
                if result.code.is_none() {
                    result.code = Some(CodeBlock {
                        language: lang.to_string(),
                        content: block_content,
                    });
                }
            }
        }
    }
    
    result
}

/// Extract all codefence blocks from content
/// Returns Vec of (language, content) tuples
fn extract_all_codefence_blocks(content: &str) -> Vec<(String, String)> {
    let mut blocks = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim_start();
        
        // Check for opening fence (``` or ~~~)
        if let Some((fence_char, fence_count, language)) = detect_fence_opening(trimmed) {
            let mut block_lines = Vec::new();
            i += 1;
            
            // Collect content until closing fence
            while i < lines.len() {
                let current = lines[i].trim_start();
                if is_closing_fence(current, fence_char, fence_count) {
                    break;
                }
                block_lines.push(lines[i]);
                i += 1;
            }
            
            let block_content = block_lines.join("\n");
            blocks.push((language, block_content.trim().to_string()));
        }
        
        i += 1;
    }
    
    blocks
}

/// Detect opening fence, returns (fence_char, count, language)
fn detect_fence_opening(line: &str) -> Option<(char, usize, String)> {
    // Try backticks
    let backtick_count = line.chars().take_while(|&c| c == '`').count();
    if backtick_count >= 3 {
        let rest = &line[backtick_count..];
        let language = rest.split_whitespace().next().unwrap_or("").to_string();
        return Some(('`', backtick_count, language));
    }
    
    // Try tildes
    let tilde_count = line.chars().take_while(|&c| c == '~').count();
    if tilde_count >= 3 {
        let rest = &line[tilde_count..];
        let language = rest.split_whitespace().next().unwrap_or("").to_string();
        return Some(('~', tilde_count, language));
    }
    
    None
}

/// Check if line is a closing fence
fn is_closing_fence(line: &str, fence_char: char, min_count: usize) -> bool {
    let count = line.chars().take_while(|&c| c == fence_char).count();
    if count < min_count {
        return false;
    }
    
    // Rest of line should be empty or whitespace
    let rest = &line[count..];
    rest.chars().all(|c| c.is_whitespace())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema_parser::FieldType;

    // ========================================
    // Core Test Cases (from requirements)
    // ========================================

    #[test]
    fn test_parse_metadata_codefence() {
        let content = r#"
# Quick Todo

```metadata
{ "name": "Quick Todo", "description": "Add a todo item" }
```

```ts
const item = await arg("Todo item");
```
"#;
        let result = parse_codefence_metadata(content);
        
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
        assert!(result.metadata.is_some());
        
        let metadata = result.metadata.unwrap();
        assert_eq!(metadata.name, Some("Quick Todo".to_string()));
        assert_eq!(metadata.description, Some("Add a todo item".to_string()));
    }

    #[test]
    fn test_parse_schema_codefence() {
        let content = r#"
```schema
{
    "input": {
        "item": { "type": "string", "required": true }
    }
}
```

```ts
const { item } = await input();
```
"#;
        let result = parse_codefence_metadata(content);
        
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
        assert!(result.schema.is_some());
        
        let schema = result.schema.unwrap();
        assert_eq!(schema.input.len(), 1);
        assert!(schema.input.contains_key("item"));
        
        let item_field = schema.input.get("item").unwrap();
        assert_eq!(item_field.field_type, FieldType::String);
        assert!(item_field.required);
    }

    #[test]
    fn test_parse_both_metadata_and_schema() {
        let content = r#"
```metadata
{ "name": "Quick Todo", "description": "Add a todo item", "icon": "CheckSquare" }
```

```schema
{
    "input": {
        "item": { "type": "string", "required": true, "description": "The todo item text" }
    },
    "output": {
        "id": { "type": "string" }
    }
}
```

```ts
const { item } = await input();
const id = await addTodo(item);
output({ id });
```
"#;
        let result = parse_codefence_metadata(content);
        
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
        
        // Check metadata
        assert!(result.metadata.is_some());
        let metadata = result.metadata.unwrap();
        assert_eq!(metadata.name, Some("Quick Todo".to_string()));
        assert_eq!(metadata.description, Some("Add a todo item".to_string()));
        assert_eq!(metadata.icon, Some("CheckSquare".to_string()));
        
        // Check schema
        assert!(result.schema.is_some());
        let schema = result.schema.unwrap();
        assert_eq!(schema.input.len(), 1);
        assert_eq!(schema.output.len(), 1);
        
        let item_field = schema.input.get("item").unwrap();
        assert_eq!(item_field.description, Some("The todo item text".to_string()));
        
        assert!(schema.output.contains_key("id"));
    }

    #[test]
    fn test_no_codefence_returns_none() {
        let content = r#"
# Just a Regular Markdown

Some text here with no code fences at all.

- List item 1
- List item 2
"#;
        let result = parse_codefence_metadata(content);
        
        assert!(result.metadata.is_none());
        assert!(result.schema.is_none());
        assert!(result.code.is_none());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_malformed_json_returns_error() {
        let content = r#"
```metadata
{ "name": "Bad JSON, "description": missing quote }
```

```ts
console.log("test");
```
"#;
        let result = parse_codefence_metadata(content);
        
        assert!(result.metadata.is_none());
        assert!(!result.errors.is_empty());
        assert!(result.errors[0].contains("Failed to parse metadata JSON"));
    }

    #[test]
    fn test_code_block_extracted_correctly() {
        let content = r#"
```metadata
{ "name": "Test Script" }
```

```ts
const result = await arg("Pick one");
console.log(result);
```
"#;
        let result = parse_codefence_metadata(content);
        
        assert!(result.code.is_some());
        let code = result.code.unwrap();
        assert_eq!(code.language, "ts");
        assert!(code.content.contains("const result = await arg"));
        assert!(code.content.contains("console.log(result)"));
    }

    // ========================================
    // Additional Test Cases
    // ========================================

    #[test]
    fn test_multiple_code_blocks_first_wins() {
        let content = r#"
```ts
first code block
```

```ts
second code block
```
"#;
        let result = parse_codefence_metadata(content);
        
        assert!(result.code.is_some());
        let code = result.code.unwrap();
        assert_eq!(code.content, "first code block");
    }

    #[test]
    fn test_tilde_fences_supported() {
        let content = r#"
~~~metadata
{ "name": "Tilde Test" }
~~~

~~~ts
console.log("tilde fences");
~~~
"#;
        let result = parse_codefence_metadata(content);
        
        assert!(result.metadata.is_some());
        assert_eq!(result.metadata.unwrap().name, Some("Tilde Test".to_string()));
        
        assert!(result.code.is_some());
        assert_eq!(result.code.unwrap().language, "ts");
    }

    #[test]
    fn test_mixed_fence_types() {
        let content = r#"
```metadata
{ "name": "Mixed Fences" }
```

~~~ts
const x = 1;
~~~
"#;
        let result = parse_codefence_metadata(content);
        
        assert!(result.metadata.is_some());
        assert!(result.code.is_some());
        assert_eq!(result.code.unwrap().language, "ts");
    }

    #[test]
    fn test_metadata_with_all_fields() {
        let content = r#"
```metadata
{
    "name": "Full Script",
    "description": "A complete script",
    "author": "Test Author",
    "enter": "Execute",
    "alias": "fs",
    "icon": "Star",
    "shortcut": "cmd shift f",
    "tags": ["productivity", "utility"],
    "hidden": true,
    "placeholder": "Type something...",
    "cron": "0 9 * * *",
    "watch": ["*.ts", "*.js"],
    "background": true,
    "system": false
}
```
"#;
        let result = parse_codefence_metadata(content);
        
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
        assert!(result.metadata.is_some());
        
        let meta = result.metadata.unwrap();
        assert_eq!(meta.name, Some("Full Script".to_string()));
        assert_eq!(meta.description, Some("A complete script".to_string()));
        assert_eq!(meta.author, Some("Test Author".to_string()));
        assert_eq!(meta.enter, Some("Execute".to_string()));
        assert_eq!(meta.alias, Some("fs".to_string()));
        assert_eq!(meta.icon, Some("Star".to_string()));
        assert_eq!(meta.shortcut, Some("cmd shift f".to_string()));
        assert_eq!(meta.tags, vec!["productivity", "utility"]);
        assert!(meta.hidden);
        assert_eq!(meta.placeholder, Some("Type something...".to_string()));
        assert_eq!(meta.cron, Some("0 9 * * *".to_string()));
        assert_eq!(meta.watch, vec!["*.ts", "*.js"]);
        assert!(meta.background);
        assert!(!meta.system);
    }

    #[test]
    fn test_schema_with_all_field_types() {
        let content = r#"
```schema
{
    "input": {
        "name": { "type": "string" },
        "count": { "type": "number" },
        "enabled": { "type": "boolean" },
        "items": { "type": "array", "items": "string" },
        "config": { "type": "object" },
        "anything": { "type": "any" }
    }
}
```
"#;
        let result = parse_codefence_metadata(content);
        
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
        assert!(result.schema.is_some());
        
        let schema = result.schema.unwrap();
        assert_eq!(schema.input.get("name").unwrap().field_type, FieldType::String);
        assert_eq!(schema.input.get("count").unwrap().field_type, FieldType::Number);
        assert_eq!(schema.input.get("enabled").unwrap().field_type, FieldType::Boolean);
        assert_eq!(schema.input.get("items").unwrap().field_type, FieldType::Array);
        assert_eq!(schema.input.get("config").unwrap().field_type, FieldType::Object);
        assert_eq!(schema.input.get("anything").unwrap().field_type, FieldType::Any);
    }

    #[test]
    fn test_malformed_schema_json_returns_error() {
        let content = r#"
```schema
{ "input": { not valid json } }
```
"#;
        let result = parse_codefence_metadata(content);
        
        assert!(result.schema.is_none());
        assert!(!result.errors.is_empty());
        assert!(result.errors[0].contains("Failed to parse schema JSON"));
    }

    #[test]
    fn test_different_code_languages() {
        let languages = vec!["bash", "python", "ruby", "js", "kit", "template"];
        
        for lang in languages {
            let content = format!(r#"
```{}
code content
```
"#, lang);
            let result = parse_codefence_metadata(&content);
            
            assert!(result.code.is_some(), "Failed for language: {}", lang);
            assert_eq!(result.code.unwrap().language, lang);
        }
    }

    #[test]
    fn test_empty_codefence_blocks() {
        let content = r#"
```metadata
```

```ts
```
"#;
        let result = parse_codefence_metadata(content);
        
        // Empty metadata should fail to parse as JSON
        assert!(!result.errors.is_empty() || result.metadata.is_none());
        
        // Empty code block should still be captured
        assert!(result.code.is_some());
        assert_eq!(result.code.unwrap().content, "");
    }

    #[test]
    fn test_whitespace_handling() {
        let content = r#"
```metadata
  { "name": "Whitespace Test" }  
```

```ts
  const x = 1;  
  const y = 2;  
```
"#;
        let result = parse_codefence_metadata(content);
        
        assert!(result.metadata.is_some());
        assert_eq!(result.metadata.unwrap().name, Some("Whitespace Test".to_string()));
        
        assert!(result.code.is_some());
        let code = result.code.unwrap();
        // Content should preserve internal whitespace but trim outer
        assert!(code.content.contains("const x = 1"));
        assert!(code.content.contains("const y = 2"));
    }

    #[test]
    fn test_nested_code_in_markdown() {
        // Simulate a scriptlet that contains a markdown code example
        let content = r#"
```metadata
{ "name": "Code Example Generator" }
```

```ts
const example = `
\`\`\`js
console.log("hello");
\`\`\`
`;
```
"#;
        let result = parse_codefence_metadata(content);
        
        assert!(result.metadata.is_some());
        assert!(result.code.is_some());
        // The code block should contain the nested fence example
        assert!(result.code.unwrap().content.contains("console.log"));
    }

    #[test]
    fn test_order_independence() {
        // Schema before metadata should still work
        let content = r#"
```schema
{ "input": { "x": { "type": "string" } } }
```

```metadata
{ "name": "Order Test" }
```

```ts
code here
```
"#;
        let result = parse_codefence_metadata(content);
        
        assert!(result.metadata.is_some());
        assert!(result.schema.is_some());
        assert!(result.code.is_some());
        
        assert_eq!(result.metadata.unwrap().name, Some("Order Test".to_string()));
        assert!(result.schema.unwrap().input.contains_key("x"));
    }

    #[test]
    fn test_code_block_without_metadata_or_schema() {
        let content = r#"
## Simple Script

```bash
echo "Hello World"
```
"#;
        let result = parse_codefence_metadata(content);
        
        assert!(result.metadata.is_none());
        assert!(result.schema.is_none());
        assert!(result.code.is_some());
        
        let code = result.code.unwrap();
        assert_eq!(code.language, "bash");
        assert_eq!(code.content, "echo \"Hello World\"");
    }

    #[test]
    fn test_multiple_errors_collected() {
        let content = r#"
```metadata
{ invalid json 1 }
```

```schema
{ invalid json 2 }
```
"#;
        let result = parse_codefence_metadata(content);
        
        assert_eq!(result.errors.len(), 2);
        assert!(result.errors[0].contains("metadata"));
        assert!(result.errors[1].contains("schema"));
    }
}
