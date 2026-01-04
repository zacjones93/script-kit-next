//! Agent execution
//!
//! This module handles executing mdflow agents:
//! - Spawn `mdflow <file>` directly (let mdflow interpret frontmatter)
//! - Add `--_quiet --raw` for UI capture mode
//! - Pass `--_varname value` for user-provided template variables
//! - Pipe stdin for `{{ _stdin }}` support
//! - Set working directory to agent file's parent directory
//!
//! # Execution Model
//!
//! We do NOT convert frontmatter to CLI flags. mdflow handles that.
//! Script Kit only adds:
//! - Mode flags (`--_quiet --raw` for UI capture)
//! - Runtime variable overrides (`--_varname value`)
//! - stdin piping

// These functions are public API for future integration - allow them to be unused for now
#![allow(dead_code)]

use std::collections::HashMap;
use std::io::Write;
use std::process::{Child, Command, Stdio};

use anyhow::{Context, Result};

use crate::agents::types::{Agent, AgentAvailability, AgentExecutionMode};

/// Check if mdflow CLI is available in PATH
pub fn is_mdflow_available() -> bool {
    which::which("mdflow").is_ok() || which::which("md").is_ok()
}

/// Get the mdflow command name (prefers "mdflow", falls back to "md")
pub fn get_mdflow_command() -> Option<&'static str> {
    if which::which("mdflow").is_ok() {
        Some("mdflow")
    } else if which::which("md").is_ok() {
        Some("md")
    } else {
        None
    }
}

/// Check availability of an agent (mdflow + backend)
pub fn check_availability(agent: &Agent) -> AgentAvailability {
    let mdflow_available = is_mdflow_available();
    let backend_available = agent.backend.is_available();

    let error_message = if !mdflow_available {
        Some("mdflow not found. Install with: npm install -g mdflow".to_string())
    } else if !backend_available {
        agent.backend.command().map(|cmd| {
            format!(
                "{} CLI not found. Please install {} to use this agent.",
                agent.backend.label(),
                cmd
            )
        })
    } else {
        None
    };

    AgentAvailability {
        mdflow_available,
        backend_available,
        error_message,
    }
}

/// Execute an agent
///
/// # Arguments
///
/// * `agent` - The agent to execute
/// * `mode` - Execution mode (UI capture, interactive, dry run, explain)
/// * `variables` - Runtime variable overrides (passed as `--_varname value`)
/// * `positional_args` - Positional arguments appended after the file path
/// * `stdin_input` - Optional input to pipe to stdin (for `{{ _stdin }}`)
///
/// # Returns
///
/// A spawned child process. The caller is responsible for:
/// - Reading stdout/stderr
/// - Waiting for completion
/// - Handling errors
pub fn execute_agent(
    agent: &Agent,
    mode: AgentExecutionMode,
    variables: &HashMap<String, String>,
    positional_args: &[String],
    stdin_input: Option<&str>,
) -> Result<Child> {
    let mdflow_cmd =
        get_mdflow_command().context("mdflow not found. Install with: npm install -g mdflow")?;

    let mut cmd = Command::new(mdflow_cmd);

    // Add the agent file path
    cmd.arg(&agent.path);

    // Add mode-specific flags
    match mode {
        AgentExecutionMode::UiCapture => {
            // Suppress dashboard, clean output for embedding in UI
            cmd.arg("--_quiet");
            cmd.arg("--raw");
        }
        AgentExecutionMode::Interactive => {
            // No special flags - mdflow handles interactive mode
        }
        AgentExecutionMode::DryRun => {
            cmd.arg("--_dry-run");
        }
        AgentExecutionMode::Explain => {
            // Use "md explain" subcommand instead
            // Note: This requires special handling - we actually run "md explain <file>"
            // For now, we use --_context which shows context without running
            cmd.arg("--_context");
        }
    }

    // Add runtime variable overrides
    for (key, value) in variables {
        // Variables are passed as --_varname value (underscore prefix)
        let flag = if key.starts_with('_') {
            format!("--{}", key)
        } else {
            format!("--_{}", key)
        };
        cmd.arg(&flag);
        cmd.arg(value);
    }

    // Add positional arguments
    for arg in positional_args {
        cmd.arg(arg);
    }

    // Set working directory to agent file's parent
    // This is important for @./relative imports to work correctly
    if let Some(parent) = agent.path.parent() {
        cmd.current_dir(parent);
    }

    // Set up I/O
    cmd.stdin(if stdin_input.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    });
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    // Apply environment from frontmatter _env if present
    if let Some(ref env) = agent.frontmatter.env {
        for (key, value) in env {
            cmd.env(key, value);
        }
    }

    let mut child = cmd
        .spawn()
        .with_context(|| format!("Failed to spawn mdflow for agent: {}", agent.path.display()))?;

    // Write stdin if provided
    if let Some(input) = stdin_input {
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(input.as_bytes())
                .with_context(|| format!("Failed to write stdin for agent: {}", agent.name))?;
            // Drop stdin to close it
        }
    }

    Ok(child)
}

/// Run `md explain` to get context preview for an agent
///
/// This is useful for showing what the agent will send to the AI
/// before actually running it.
pub fn explain_agent(agent: &Agent) -> Result<String> {
    let mdflow_cmd =
        get_mdflow_command().context("mdflow not found. Install with: npm install -g mdflow")?;

    let mut cmd = Command::new(mdflow_cmd);
    cmd.arg("explain");
    cmd.arg(&agent.path);

    // Set working directory
    if let Some(parent) = agent.path.parent() {
        cmd.current_dir(parent);
    }

    let output = cmd
        .output()
        .with_context(|| format!("Failed to run md explain for agent: {}", agent.name))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("md explain failed: {}", stderr)
    }
}

/// Run `md --_dry-run` to see what would be executed
///
/// Shows the full command that would be run without actually running it.
pub fn dry_run_agent(agent: &Agent) -> Result<String> {
    let mdflow_cmd =
        get_mdflow_command().context("mdflow not found. Install with: npm install -g mdflow")?;

    let mut cmd = Command::new(mdflow_cmd);
    cmd.arg(&agent.path);
    cmd.arg("--_dry-run");

    // Set working directory
    if let Some(parent) = agent.path.parent() {
        cmd.current_dir(parent);
    }

    let output = cmd
        .output()
        .with_context(|| format!("Failed to run dry-run for agent: {}", agent.name))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("dry-run failed: {}", stderr)
    }
}

/// Build command args for external terminal execution
///
/// For interactive agents, we may want to open a system terminal.
/// This returns the command and arguments to run.
pub fn build_terminal_command(agent: &Agent) -> (String, Vec<String>) {
    let mdflow_cmd = get_mdflow_command().unwrap_or("mdflow");
    let args = vec![agent.path.to_string_lossy().to_string()];

    // For interactive terminal, don't add --_quiet or --raw
    // Let mdflow show its full terminal UX

    (mdflow_cmd.to_string(), args)
}

/// Get install instructions for missing dependencies
pub fn get_install_instructions(availability: &AgentAvailability) -> String {
    if !availability.mdflow_available {
        return "mdflow is not installed.\n\n\
            Install with npm:\n  npm install -g mdflow\n\n\
            Or with bun:\n  bun install -g mdflow\n\n\
            For more info: https://github.com/johnlindquist/mdflow"
            .to_string();
    }

    if !availability.backend_available {
        if let Some(ref error) = availability.error_message {
            return error.clone();
        }
    }

    "Unknown installation issue".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::types::{AgentBackend, AgentFrontmatter};
    use std::path::PathBuf;

    fn create_test_agent(backend: AgentBackend) -> Agent {
        Agent {
            name: "Test Agent".to_string(),
            path: PathBuf::from("/tmp/test.claude.md"),
            backend,
            interactive: false,
            description: Some("Test description".to_string()),
            icon: None,
            shortcut: None,
            alias: None,
            frontmatter: AgentFrontmatter::default(),
            kit: Some("main".to_string()),
            has_shell_inlines: false,
            has_remote_imports: false,
        }
    }

    #[test]
    fn test_check_availability_mdflow_missing() {
        // This test will pass if mdflow is not installed
        // or behave correctly if it is installed
        let agent = create_test_agent(AgentBackend::Claude);
        let availability = check_availability(&agent);

        // We can't guarantee mdflow is installed, so just check the struct is valid
        assert!(availability.mdflow_available || availability.error_message.is_some());
    }

    #[test]
    fn test_availability_struct() {
        let avail = AgentAvailability {
            mdflow_available: true,
            backend_available: true,
            error_message: None,
        };
        assert!(avail.is_available());

        let avail2 = AgentAvailability {
            mdflow_available: false,
            backend_available: true,
            error_message: Some("mdflow not found".to_string()),
        };
        assert!(!avail2.is_available());
    }

    #[test]
    fn test_get_install_instructions_mdflow() {
        let avail = AgentAvailability {
            mdflow_available: false,
            backend_available: true,
            error_message: Some("mdflow not found".to_string()),
        };

        let instructions = get_install_instructions(&avail);
        assert!(instructions.contains("npm install -g mdflow"));
        assert!(instructions.contains("bun install -g mdflow"));
    }

    #[test]
    fn test_get_install_instructions_backend() {
        let avail = AgentAvailability {
            mdflow_available: true,
            backend_available: false,
            error_message: Some("claude CLI not found".to_string()),
        };

        let instructions = get_install_instructions(&avail);
        assert!(instructions.contains("claude CLI not found"));
    }

    #[test]
    fn test_build_terminal_command() {
        let agent = create_test_agent(AgentBackend::Claude);
        let (cmd, args) = build_terminal_command(&agent);

        assert!(cmd == "mdflow" || cmd == "md");
        assert_eq!(args.len(), 1);
        assert!(args[0].contains("test.claude.md"));
    }

    // Note: We can't easily test execute_agent without mdflow installed
    // and an actual agent file. These would be integration tests.

    #[test]
    fn test_execution_mode_default() {
        assert_eq!(AgentExecutionMode::default(), AgentExecutionMode::UiCapture);
    }
}
