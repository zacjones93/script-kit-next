//! Script creation utilities
//!
//! Functions for validating script names, generating templates,
//! and creating script files.

use crate::logging;

/// Validates a script name - only alphanumeric and hyphens allowed
///
/// # Rules
/// - Cannot be empty
/// - Only letters, numbers, and hyphens allowed
/// - Cannot start or end with a hyphen
///
/// # Examples
/// ```
/// assert!(validate_script_name("hello-world").is_ok());
/// assert!(validate_script_name("myScript").is_ok());
/// assert!(validate_script_name("").is_err());
/// assert!(validate_script_name("-hello").is_err());
/// ```
pub fn validate_script_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Script name cannot be empty".to_string());
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return Err("Script name can only contain letters, numbers, and hyphens".to_string());
    }
    if name.starts_with('-') || name.ends_with('-') {
        return Err("Script name cannot start or end with a hyphen".to_string());
    }
    Ok(())
}

/// Generates a script template with the given name
///
/// Converts kebab-case names to Title Case for the display name.
/// Creates a basic TypeScript script with Name and Description metadata.
///
/// # Example
/// ```
/// let template = generate_script_template("hello-world");
/// // Returns:
/// // // Name: Hello World
/// // // Description:
/// //
/// // console.log("Hello from hello-world!")
/// ```
pub fn generate_script_template(name: &str) -> String {
    // Convert kebab-case to Title Case for display
    let display_name = name
        .split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    format!(
        r#"// Name: {}
// Description: 

console.log("Hello from {}!")
"#,
        display_name, name
    )
}

/// Creates a new script file at ~/.sk/kit/scripts/{name}.ts
///
/// # Arguments
/// * `name` - The script name (will be validated)
///
/// # Returns
/// * `Ok(PathBuf)` - Path to the created script file
/// * `Err(String)` - Error message if creation failed
///
/// # Errors
/// - Invalid script name (see `validate_script_name`)
/// - Script already exists
/// - Failed to create directory or write file
pub fn create_script_file(name: &str) -> Result<std::path::PathBuf, String> {
    use std::fs;
    use std::path::PathBuf;

    validate_script_name(name)?;

    let scripts_dir = PathBuf::from(shellexpand::tilde("~/.sk/kit/scripts").as_ref());

    // Ensure directory exists
    if !scripts_dir.exists() {
        fs::create_dir_all(&scripts_dir)
            .map_err(|e| format!("Failed to create scripts directory: {}", e))?;
    }

    let file_path = scripts_dir.join(format!("{}.ts", name));

    // Check if file already exists
    if file_path.exists() {
        return Err(format!("Script '{}' already exists", name));
    }

    // Write template
    let template = generate_script_template(name);
    fs::write(&file_path, template).map_err(|e| format!("Failed to write script file: {}", e))?;

    logging::log(
        "SCRIPT_CREATE",
        &format!("Created new script: {}", file_path.display()),
    );

    Ok(file_path)
}

/// Returns the path where a script would be created (without creating it)
/// Useful for checking if a script already exists or for UI display
pub fn get_script_path(name: &str) -> std::path::PathBuf {
    use std::path::PathBuf;
    let scripts_dir = PathBuf::from(shellexpand::tilde("~/.sk/kit/scripts").as_ref());
    scripts_dir.join(format!("{}.ts", name))
}

/// Checks if a script with the given name already exists
pub fn script_exists(name: &str) -> bool {
    get_script_path(name).exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_script_name_valid() {
        assert!(validate_script_name("hello-world").is_ok());
        assert!(validate_script_name("myScript").is_ok());
        assert!(validate_script_name("test123").is_ok());
        assert!(validate_script_name("a").is_ok());
        assert!(validate_script_name("ABC").is_ok());
        assert!(validate_script_name("my-cool-script").is_ok());
    }

    #[test]
    fn test_validate_script_name_empty() {
        let result = validate_script_name("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Script name cannot be empty");
    }

    #[test]
    fn test_validate_script_name_invalid_chars() {
        // Spaces not allowed
        let result = validate_script_name("hello world");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("only contain letters"));

        // Underscores not allowed
        let result = validate_script_name("hello_world");
        assert!(result.is_err());

        // Special characters not allowed
        assert!(validate_script_name("hello!").is_err());
        assert!(validate_script_name("hello@script").is_err());
        assert!(validate_script_name("hello.ts").is_err());
        assert!(validate_script_name("path/to/script").is_err());
    }

    #[test]
    fn test_validate_script_name_hyphen_position() {
        // Leading hyphen not allowed
        let result = validate_script_name("-hello");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot start or end"));

        // Trailing hyphen not allowed
        let result = validate_script_name("hello-");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot start or end"));

        // Just a hyphen not allowed
        assert!(validate_script_name("-").is_err());
    }

    #[test]
    fn test_generate_script_template_simple() {
        let template = generate_script_template("hello");
        assert!(template.contains("// Name: Hello"));
        assert!(template.contains("// Description:"));
        assert!(template.contains("Hello from hello!"));
    }

    #[test]
    fn test_generate_script_template_kebab_case() {
        let template = generate_script_template("hello-world");
        assert!(template.contains("// Name: Hello World"));
        assert!(template.contains("Hello from hello-world!"));
    }

    #[test]
    fn test_generate_script_template_multi_word() {
        let template = generate_script_template("my-cool-script");
        assert!(template.contains("// Name: My Cool Script"));
        assert!(template.contains("Hello from my-cool-script!"));
    }

    #[test]
    fn test_generate_script_template_structure() {
        let template = generate_script_template("test");

        // Should have proper structure
        assert!(template.starts_with("// Name:"));
        assert!(template.contains("// Description:"));
        assert!(template.contains("console.log"));

        // Template should be valid TypeScript (basic check)
        assert!(template.contains("\"Hello from test!\""));
    }

    #[test]
    fn test_get_script_path() {
        let path = get_script_path("hello-world");

        // Should end with the correct filename
        assert!(path.to_string_lossy().ends_with("hello-world.ts"));

        // Should be in ~/.sk/kit/scripts/
        assert!(path.to_string_lossy().contains(".sk/kit/scripts"));
    }

    #[test]
    fn test_get_script_path_various_names() {
        assert!(get_script_path("a").to_string_lossy().ends_with("a.ts"));
        assert!(get_script_path("my-script")
            .to_string_lossy()
            .ends_with("my-script.ts"));
        assert!(get_script_path("Test123")
            .to_string_lossy()
            .ends_with("Test123.ts"));
    }
}
