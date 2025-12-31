//! Semantic ID generation for AI-driven UX targeting
//!
//! Provides functions to generate semantic IDs for UI elements that can be
//! used by AI agents to target specific elements in the interface.

/// Generate a semantic ID for an element.
///
/// Format: {type}:{index}:{value_slug}
///
/// # Arguments
/// * `element_type` - The element type (e.g., "choice", "button", "input")
/// * `index` - The numeric index of the element
/// * `value` - The value to convert to a slug
///
/// # Returns
/// A semantic ID string in the format: type:index:slug
pub fn generate_semantic_id(element_type: &str, index: usize, value: &str) -> String {
    let slug = value_to_slug(value);
    format!("{}:{}:{}", element_type, index, slug)
}

/// Generate a semantic ID for named elements (no index).
///
/// Format: {type}:{name}
///
/// # Arguments
/// * `element_type` - The element type (e.g., "input", "panel", "window")
/// * `name` - The name of the element
///
/// # Returns
/// A semantic ID string in the format: type:name
pub fn generate_semantic_id_named(element_type: &str, name: &str) -> String {
    let slug = value_to_slug(name);
    format!("{}:{}", element_type, slug)
}

/// Convert a value string to a URL-safe slug suitable for semantic IDs.
///
/// - Converts to lowercase
/// - Replaces spaces and underscores with hyphens
/// - Removes non-alphanumeric characters (except hyphens)
/// - Collapses multiple hyphens to single
/// - Truncates to 20 characters
/// - Removes leading/trailing hyphens
pub fn value_to_slug(value: &str) -> String {
    let slug: String = value
        .to_lowercase()
        .chars()
        .map(|c| match c {
            ' ' | '_' => '-',
            c if c.is_alphanumeric() || c == '-' => c,
            _ => '-',
        })
        .collect();

    // Collapse multiple hyphens and trim
    let mut result = String::with_capacity(20);
    let mut prev_hyphen = false;

    for c in slug.chars() {
        if c == '-' {
            if !prev_hyphen && !result.is_empty() {
                result.push('-');
            }
            prev_hyphen = true;
        } else {
            result.push(c);
            prev_hyphen = false;
        }

        if result.len() >= 20 {
            break;
        }
    }

    // Remove trailing hyphen
    if result.ends_with('-') {
        result.pop();
    }

    // Ensure non-empty
    if result.is_empty() {
        result.push_str("item");
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_to_slug_basic() {
        assert_eq!(value_to_slug("apple"), "apple");
        assert_eq!(value_to_slug("Apple"), "apple");
        assert_eq!(value_to_slug("APPLE"), "apple");
    }

    #[test]
    fn test_value_to_slug_spaces() {
        assert_eq!(value_to_slug("red apple"), "red-apple");
        assert_eq!(value_to_slug("red  apple"), "red-apple"); // multiple spaces
        assert_eq!(value_to_slug("  apple  "), "apple"); // leading/trailing spaces become hyphens then trimmed
    }

    #[test]
    fn test_value_to_slug_special_chars() {
        assert_eq!(value_to_slug("apple_pie"), "apple-pie");
        assert_eq!(value_to_slug("apple@pie!"), "apple-pie");
        assert_eq!(value_to_slug("hello-world"), "hello-world");
    }

    #[test]
    fn test_value_to_slug_truncation() {
        let long_value = "this is a very long value that exceeds twenty characters";
        let slug = value_to_slug(long_value);
        assert!(slug.len() <= 20);
        assert_eq!(slug, "this-is-a-very-long");
    }

    #[test]
    fn test_value_to_slug_empty() {
        assert_eq!(value_to_slug(""), "item");
        assert_eq!(value_to_slug("   "), "item");
        assert_eq!(value_to_slug("@#$%"), "item"); // all special chars
    }

    #[test]
    fn test_generate_semantic_id() {
        assert_eq!(generate_semantic_id("choice", 0, "apple"), "choice:0:apple");
        assert_eq!(
            generate_semantic_id("choice", 5, "Red Apple"),
            "choice:5:red-apple"
        );
        assert_eq!(
            generate_semantic_id("button", 1, "Submit Form"),
            "button:1:submit-form"
        );
    }

    #[test]
    fn test_generate_semantic_id_named() {
        assert_eq!(
            generate_semantic_id_named("input", "filter"),
            "input:filter"
        );
        assert_eq!(
            generate_semantic_id_named("panel", "preview"),
            "panel:preview"
        );
        assert_eq!(
            generate_semantic_id_named("window", "Main Window"),
            "window:main-window"
        );
    }
}
