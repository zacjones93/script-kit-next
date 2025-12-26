//! Alacritty terminal emulator integration for Script Kit GPUI.
//!
//! This module wraps Alacritty's terminal emulator library to provide
//! VT100/xterm compatible terminal emulation. It handles escape sequence
//! parsing, terminal grid management, and state tracking.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────┐     ┌──────────────┐     ┌─────────────┐
//! │  PTY Output │ ──▶ │ VTE Parser   │ ──▶ │ Term Grid   │
//! └─────────────┘     └──────────────┘     └─────────────┘
//!                                                 │
//!                                                 ▼
//!                                          ┌─────────────┐
//!                                          │ GPUI Render │
//!                                          └─────────────┘
//! ```
//!
//! The terminal processes incoming bytes through the VTE parser, which
//! interprets escape sequences and updates the terminal grid. The grid
//! state is then read by the GPUI rendering layer.
//!
//! # Thread Safety
//!
//! `TerminalHandle` uses `Arc<Mutex<>>` for the terminal state, allowing
//! safe access from multiple threads. The PTY I/O can run on a background
//! thread while the main thread reads terminal content for rendering.
//!
//! # Example
//!
//! ```rust,ignore
//! use script_kit_gpui::terminal::TerminalHandle;
//!
//! let mut terminal = TerminalHandle::new(80, 24)?;
//!
//! // Process incoming data from PTY
//! let events = terminal.process();
//!
//! // Send keyboard input
//! terminal.input(b"ls -la\n")?;
//!
//! // Get content for rendering
//! let content = terminal.content();
//! for line in &content.lines {
//!     println!("{}", line);
//! }
//! ```

use std::sync::{Arc, Mutex};

use alacritty_terminal::event::{Event as AlacrittyEvent, EventListener};
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::term::{Config as TermConfig, Term};
use anyhow::{Context, Result};
use tracing::{debug, info, instrument, trace, warn};
use vte::ansi::Processor;

use crate::terminal::pty::PtyManager;
use crate::terminal::theme_adapter::ThemeAdapter;
use crate::terminal::TerminalEvent;

/// Default scrollback buffer size in lines.
const DEFAULT_SCROLLBACK_LINES: usize = 10_000;

/// Maximum bytes to read from PTY in a single process() call.
const PTY_READ_BUFFER_SIZE: usize = 4096;

/// Event proxy for alacritty_terminal - handles terminal events.
///
/// This struct implements `EventListener` to receive events from the
/// Alacritty terminal emulator. Events are batched for efficient processing.
///
/// The EventProxy is cloneable because it shares the event queue via Arc.
#[derive(Debug, Clone)]
pub struct EventProxy {
    /// Batched events waiting to be processed.
    events: Arc<Mutex<Vec<TerminalEvent>>>,
}

impl EventProxy {
    /// Creates a new event proxy with an empty event queue.
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Takes all pending events, leaving an empty queue.
    pub fn take_events(&self) -> Vec<TerminalEvent> {
        let mut events = self.events.lock().unwrap();
        std::mem::take(&mut *events)
    }
}

impl Default for EventProxy {
    fn default() -> Self {
        Self::new()
    }
}

impl EventListener for EventProxy {
    fn send_event(&self, event: AlacrittyEvent) {
        let terminal_event = match event {
            AlacrittyEvent::Bell => {
                debug!("Terminal bell received");
                Some(TerminalEvent::Bell)
            }
            AlacrittyEvent::Title(title) => {
                debug!(title = %title, "Terminal title changed");
                Some(TerminalEvent::Title(title))
            }
            AlacrittyEvent::ResetTitle => {
                debug!("Terminal title reset");
                Some(TerminalEvent::Title(String::new()))
            }
            AlacrittyEvent::Exit => {
                info!("Terminal exit requested");
                Some(TerminalEvent::Exit(0))
            }
            AlacrittyEvent::ChildExit(code) => {
                info!(exit_code = code, "Child process exited");
                Some(TerminalEvent::Exit(code))
            }
            AlacrittyEvent::Wakeup => {
                trace!("Terminal wakeup event");
                // Wakeup events indicate new content is available
                None
            }
            AlacrittyEvent::PtyWrite(text) => {
                trace!(bytes = text.len(), "PTY write request");
                // PtyWrite events are handled internally
                None
            }
            AlacrittyEvent::MouseCursorDirty => {
                trace!("Mouse cursor dirty");
                None
            }
            AlacrittyEvent::CursorBlinkingChange => {
                trace!("Cursor blinking state changed");
                None
            }
            AlacrittyEvent::ClipboardStore(_, _) => {
                trace!("Clipboard store request");
                None
            }
            AlacrittyEvent::ClipboardLoad(_, _) => {
                trace!("Clipboard load request");
                None
            }
            AlacrittyEvent::ColorRequest(_, _) => {
                trace!("Color request");
                None
            }
            AlacrittyEvent::TextAreaSizeRequest(_) => {
                trace!("Text area size request");
                None
            }
        };

        if let Some(event) = terminal_event {
            let mut events = self.events.lock().unwrap();
            events.push(event);
        }
    }
}

/// Terminal dimensions for creating Term instance.
#[derive(Debug, Clone, Copy)]
pub struct TerminalSize {
    /// Number of columns.
    pub cols: usize,
    /// Number of rows.
    pub rows: usize,
}

impl TerminalSize {
    /// Creates a new terminal size.
    pub fn new(cols: u16, rows: u16) -> Self {
        Self {
            cols: cols as usize,
            rows: rows as usize,
        }
    }
}

impl Dimensions for TerminalSize {
    fn total_lines(&self) -> usize {
        self.rows
    }

    fn screen_lines(&self) -> usize {
        self.rows
    }

    fn columns(&self) -> usize {
        self.cols
    }
}

/// Thread-safe terminal state wrapper.
///
/// This struct bundles the terminal and its VTE processor together,
/// allowing thread-safe access to both.
struct TerminalState {
    term: Term<EventProxy>,
    processor: Processor,
}

impl TerminalState {
    fn new(config: TermConfig, size: &TerminalSize, event_proxy: EventProxy) -> Self {
        Self {
            term: Term::new(config, size, event_proxy),
            processor: Processor::new(),
        }
    }

    /// Process raw bytes from PTY through the VTE parser.
    fn process_bytes(&mut self, bytes: &[u8]) {
        // VTE 0.15 advance() takes a slice of bytes
        self.processor.advance(&mut self.term, bytes);
    }
}

/// Handle to an Alacritty terminal emulator instance.
///
/// `TerminalHandle` provides the core terminal emulation functionality:
///
/// - **Escape Sequence Parsing**: Full VT100/xterm/ANSI support via VTE
/// - **Grid Management**: Character grid with colors, attributes, and Unicode
/// - **Scrollback Buffer**: Configurable history for scrolling back
/// - **Selection**: Text selection support for copy operations
/// - **Thread Safety**: Safe concurrent access via Arc<Mutex<>>
///
/// # Thread Safety
///
/// The terminal state is wrapped in `Arc<Mutex<>>`, allowing safe access
/// from multiple threads. The main thread can render while a background
/// thread handles PTY I/O.
///
/// # Performance
///
/// The terminal uses event batching and damage tracking to minimize
/// unnecessary re-rendering. Only cells that have changed are marked dirty.
pub struct TerminalHandle {
    /// Thread-safe terminal state.
    state: Arc<Mutex<TerminalState>>,
    /// Event proxy for receiving terminal events (shared with Term).
    event_proxy: EventProxy,
    /// PTY manager for process I/O.
    pty: PtyManager,
    /// Theme adapter for colors.
    #[allow(dead_code)]
    theme: ThemeAdapter,
    /// Current terminal dimensions.
    cols: u16,
    rows: u16,
    /// Buffer for reading from PTY.
    read_buffer: Vec<u8>,
}

impl std::fmt::Debug for TerminalHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TerminalHandle")
            .field("cols", &self.cols)
            .field("rows", &self.rows)
            .finish_non_exhaustive()
    }
}

impl TerminalHandle {
    /// Creates a new terminal handle with the default shell.
    ///
    /// # Arguments
    ///
    /// * `cols` - Number of columns (character width)
    /// * `rows` - Number of rows (character height)
    ///
    /// # Errors
    ///
    /// Returns an error if PTY creation or shell spawning fails.
    #[instrument(level = "info", name = "terminal_new", fields(cols, rows))]
    pub fn new(cols: u16, rows: u16) -> Result<Self> {
        Self::with_scrollback(cols, rows, DEFAULT_SCROLLBACK_LINES)
    }

    /// Creates a new terminal handle running a specific command.
    ///
    /// # Arguments
    ///
    /// * `cmd` - The command to execute
    /// * `cols` - Number of columns
    /// * `rows` - Number of rows
    ///
    /// # Errors
    ///
    /// Returns an error if PTY creation or command spawning fails.
    #[instrument(level = "info", name = "terminal_with_command", fields(cmd = %cmd, cols, rows))]
    pub fn with_command(cmd: &str, cols: u16, rows: u16) -> Result<Self> {
        Self::create_internal(Some(cmd), cols, rows, DEFAULT_SCROLLBACK_LINES)
    }

    /// Creates a new terminal handle with custom scrollback size.
    ///
    /// # Arguments
    ///
    /// * `cols` - Number of columns
    /// * `rows` - Number of rows
    /// * `scrollback_lines` - Maximum lines to keep in scrollback buffer
    ///
    /// # Errors
    ///
    /// Returns an error if PTY creation or shell spawning fails.
    #[instrument(level = "info", name = "terminal_with_scrollback", fields(cols, rows, scrollback_lines))]
    pub fn with_scrollback(cols: u16, rows: u16, scrollback_lines: usize) -> Result<Self> {
        Self::create_internal(None, cols, rows, scrollback_lines)
    }

    /// Internal creation method.
    fn create_internal(
        cmd: Option<&str>,
        cols: u16,
        rows: u16,
        scrollback_lines: usize,
    ) -> Result<Self> {
        // Create PTY manager
        // When a command is provided, wrap it in a shell to handle:
        // - Argument parsing (e.g., "ls -la" -> ls with -la arg)
        // - Tilde expansion (e.g., ~ -> /Users/name)
        // - Environment variable expansion
        // - Pipes, redirects, and other shell features
        let pty = if let Some(cmd) = cmd {
            let shell = Self::detect_shell();
            info!(
                shell = %shell,
                original_cmd = %cmd,
                "Wrapping command in shell for proper execution"
            );
            // Use shell -c "command" to let the shell parse the command string
            PtyManager::with_command_and_size(&shell, &["-c", cmd], cols, rows)
                .with_context(|| format!("Failed to create PTY with command: {} -c '{}'", shell, cmd))?
        } else {
            PtyManager::with_size(cols, rows).context("Failed to create PTY")?
        };

        // Create terminal configuration
        let config = TermConfig {
            scrolling_history: scrollback_lines,
            ..TermConfig::default()
        };

        // Create event proxy - we'll share it with the Term
        let event_proxy = EventProxy::new();

        // Create terminal dimensions
        let size = TerminalSize::new(cols, rows);

        // Create the terminal state (Term + Processor)
        let state = TerminalState::new(config, &size, event_proxy.clone());
        let state = Arc::new(Mutex::new(state));

        // Create theme adapter with defaults
        let theme = ThemeAdapter::dark_default();

        info!(cols, rows, scrollback_lines, "Terminal created successfully");

        Ok(Self {
            state,
            event_proxy,
            pty,
            theme,
            cols,
            rows,
            read_buffer: vec![0u8; PTY_READ_BUFFER_SIZE],
        })
    }

    /// Detects the default shell for the current platform.
    ///
    /// On Unix, uses `$SHELL` environment variable, falling back to `/bin/sh`.
    /// On Windows, uses `%COMSPEC%`, falling back to `cmd.exe`.
    fn detect_shell() -> String {
        #[cfg(unix)]
        {
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
        }
        #[cfg(windows)]
        {
            std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string())
        }
    }

    /// Processes PTY output through terminal parser.
    ///
    /// Reads available data from the PTY and processes it through the
    /// VTE parser to update the terminal grid. Returns any events
    /// generated during processing.
    ///
    /// This method is non-blocking - it reads whatever data is available
    /// without waiting.
    ///
    /// # Returns
    ///
    /// A vector of terminal events generated during processing.
    #[instrument(level = "trace", skip(self))]
    pub fn process(&mut self) -> Vec<TerminalEvent> {
        // Try to read from PTY (non-blocking would be ideal, but we'll handle errors)
        match self.pty.read(&mut self.read_buffer) {
            Ok(0) => {
                // EOF - terminal closed
                trace!("PTY EOF");
            }
            Ok(n) => {
                trace!(bytes = n, "Read from PTY");

                // Process bytes through VTE parser
                let mut state = self.state.lock().unwrap();
                state.process_bytes(&self.read_buffer[..n]);
            }
            Err(e) => {
                // WouldBlock is expected for non-blocking reads
                if e.kind() != std::io::ErrorKind::WouldBlock {
                    warn!(error = %e, "Error reading from PTY");
                }
            }
        }

        // Collect and return events
        self.event_proxy.take_events()
    }

    /// Sends keyboard input bytes to the terminal.
    ///
    /// # Arguments
    ///
    /// * `bytes` - Raw bytes to send (e.g., UTF-8 encoded text)
    ///
    /// # Errors
    ///
    /// Returns an error if writing to the PTY fails.
    #[instrument(level = "debug", skip(self, bytes), fields(bytes_len = bytes.len()))]
    pub fn input(&mut self, bytes: &[u8]) -> Result<()> {
        self.pty
            .write_all(bytes)
            .context("Failed to write to PTY")?;
        self.pty.flush().context("Failed to flush PTY")?;
        debug!(bytes_len = bytes.len(), "Sent input to terminal");
        Ok(())
    }

    /// Resizes the terminal grid.
    ///
    /// Content is reflowed according to terminal resize semantics:
    /// - Lines longer than the new width are wrapped
    /// - The cursor position is adjusted to stay visible
    /// - Scrollback content is preserved
    ///
    /// # Arguments
    ///
    /// * `cols` - New number of columns
    /// * `rows` - New number of rows
    ///
    /// # Errors
    ///
    /// Returns an error if the PTY resize fails.
    #[instrument(level = "debug", skip(self), fields(cols, rows))]
    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()> {
        // Resize PTY first
        self.pty
            .resize(cols, rows)
            .context("Failed to resize PTY")?;

        // Resize terminal grid
        let size = TerminalSize::new(cols, rows);
        {
            let mut state = self.state.lock().unwrap();
            state.term.resize(size);
        }

        self.cols = cols;
        self.rows = rows;

        debug!(cols, rows, "Terminal resized");
        Ok(())
    }

    /// Returns the current terminal dimensions as (columns, rows).
    #[inline]
    pub fn size(&self) -> (u16, u16) {
        (self.cols, self.rows)
    }

    /// Checks if the terminal process is still running.
    ///
    /// # Returns
    ///
    /// `true` if the child process is still running, `false` otherwise.
    pub fn is_running(&mut self) -> bool {
        self.pty.is_running()
    }

    /// Gets the current terminal content for rendering.
    ///
    /// This method creates a snapshot of the visible terminal content,
    /// including the cursor position. It's designed to be called from
    /// the render loop.
    ///
    /// # Returns
    ///
    /// A `TerminalContent` struct containing lines and cursor info.
    #[instrument(level = "trace", skip(self))]
    pub fn content(&self) -> TerminalContent {
        let state = self.state.lock().unwrap();
        let grid = state.term.grid();

        let mut lines = Vec::with_capacity(state.term.screen_lines());

        // Iterate over visible lines
        for line_idx in 0..state.term.screen_lines() {
            let row = &grid[alacritty_terminal::index::Line(line_idx as i32)];
            let mut line_str = String::with_capacity(state.term.columns());

            for col_idx in 0..state.term.columns() {
                let cell = &row[alacritty_terminal::index::Column(col_idx)];
                line_str.push(cell.c);
            }

            // Trim trailing spaces for cleaner output
            let trimmed = line_str.trim_end();
            lines.push(trimmed.to_string());
        }

        // Get cursor position
        let cursor = grid.cursor.point;

        TerminalContent {
            lines,
            cursor_line: cursor.line.0 as usize,
            cursor_col: cursor.column.0,
        }
    }

    /// Gets the configured scrollback buffer size.
    #[inline]
    pub fn scrollback_lines(&self) -> usize {
        let state = self.state.lock().unwrap();
        state.term.history_size()
    }

    /// Scrolls the terminal display.
    ///
    /// # Arguments
    ///
    /// * `delta` - Number of lines to scroll (positive = up, negative = down)
    pub fn scroll(&mut self, delta: i32) {
        let mut state = self.state.lock().unwrap();
        let scroll = alacritty_terminal::grid::Scroll::Delta(delta);
        state.term.scroll_display(scroll);
        debug!(delta, "Scrolled terminal display");
    }

    /// Gets the current selection as a string, if any.
    ///
    /// # Returns
    ///
    /// The selected text, or `None` if there is no selection.
    pub fn selection_to_string(&self) -> Option<String> {
        let state = self.state.lock().unwrap();
        state.term.selection_to_string()
    }

    /// Clears the current selection.
    pub fn clear_selection(&mut self) {
        let mut state = self.state.lock().unwrap();
        state.term.selection = None;
        debug!("Selection cleared");
    }

    /// Updates the theme adapter for focus state.
    ///
    /// # Arguments
    ///
    /// * `is_focused` - Whether the terminal window is focused.
    pub fn update_focus(&mut self, is_focused: bool) {
        self.theme.update_for_focus(is_focused);
        let mut state = self.state.lock().unwrap();
        state.term.is_focused = is_focused;
        debug!(is_focused, "Terminal focus updated");
    }

    /// Gets a reference to the theme adapter.
    pub fn theme(&self) -> &ThemeAdapter {
        &self.theme
    }
}

/// Content snapshot for rendering.
///
/// This struct contains a snapshot of the terminal content at a point
/// in time, suitable for rendering in GPUI.
#[derive(Debug, Clone)]
pub struct TerminalContent {
    /// Lines of text in the terminal.
    pub lines: Vec<String>,
    /// Cursor line position (0-indexed from top).
    pub cursor_line: usize,
    /// Cursor column position (0-indexed from left).
    pub cursor_col: usize,
}

impl TerminalContent {
    /// Returns `true` if the terminal is empty (no content).
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty() || self.lines.iter().all(|l| l.is_empty())
    }

    /// Returns the number of non-empty lines.
    pub fn line_count(&self) -> usize {
        self.lines.iter().filter(|l| !l.is_empty()).count()
    }
}

/// Cursor position in the terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorPosition {
    /// Line index (0-indexed from top).
    pub line: usize,
    /// Column index (0-indexed from left).
    pub col: usize,
}

impl From<&TerminalContent> for CursorPosition {
    fn from(content: &TerminalContent) -> Self {
        Self {
            line: content.cursor_line,
            col: content.cursor_col,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_proxy_creation() {
        let proxy = EventProxy::new();
        assert!(proxy.take_events().is_empty());
    }

    #[test]
    fn test_event_proxy_batching() {
        let proxy = EventProxy::new();

        // Simulate sending events
        proxy.send_event(AlacrittyEvent::Bell);
        proxy.send_event(AlacrittyEvent::Title("Test".to_string()));

        let events = proxy.take_events();
        assert_eq!(events.len(), 2);
        assert!(matches!(events[0], TerminalEvent::Bell));
        assert!(matches!(events[1], TerminalEvent::Title(_)));

        // Events should be cleared after take
        assert!(proxy.take_events().is_empty());
    }

    #[test]
    fn test_terminal_size() {
        let size = TerminalSize::new(80, 24);
        assert_eq!(size.columns(), 80);
        assert_eq!(size.screen_lines(), 24);
        assert_eq!(size.total_lines(), 24);
    }

    #[test]
    fn test_terminal_content_is_empty() {
        let empty_content = TerminalContent {
            lines: vec![],
            cursor_line: 0,
            cursor_col: 0,
        };
        assert!(empty_content.is_empty());

        let whitespace_content = TerminalContent {
            lines: vec!["".to_string(), "".to_string()],
            cursor_line: 0,
            cursor_col: 0,
        };
        assert!(whitespace_content.is_empty());

        let content_with_text = TerminalContent {
            lines: vec!["hello".to_string()],
            cursor_line: 0,
            cursor_col: 5,
        };
        assert!(!content_with_text.is_empty());
    }

    #[test]
    fn test_terminal_content_line_count() {
        let content = TerminalContent {
            lines: vec!["hello".to_string(), "".to_string(), "world".to_string()],
            cursor_line: 0,
            cursor_col: 0,
        };
        assert_eq!(content.line_count(), 2);
    }

    #[test]
    fn test_cursor_position_from_content() {
        let content = TerminalContent {
            lines: vec!["hello world".to_string()],
            cursor_line: 0,
            cursor_col: 6,
        };
        let cursor: CursorPosition = (&content).into();
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.col, 6);
    }

    #[test]
    fn test_detect_shell() {
        let shell = TerminalHandle::detect_shell();
        assert!(!shell.is_empty(), "Shell should not be empty");

        #[cfg(unix)]
        {
            // On Unix, should be a valid path or common shell name
            assert!(
                shell.starts_with('/') || shell == "sh" || shell == "bash" || shell == "zsh",
                "Unix shell should be absolute path or known shell, got: {}",
                shell
            );
        }

        #[cfg(windows)]
        {
            let lower = shell.to_lowercase();
            assert!(
                lower.contains("cmd") || lower.contains("powershell"),
                "Windows shell should be cmd or powershell, got: {}",
                shell
            );
        }
    }

    #[test]
    fn test_terminal_with_simple_command() {
        // Test that a simple command like "echo hello" works when wrapped in shell
        let result = TerminalHandle::with_command("echo hello", 80, 24);
        
        // Skip if PTY creation fails (e.g., in CI without PTY support)
        if let Ok(mut terminal) = result {
            // Give it a moment to produce output
            std::thread::sleep(std::time::Duration::from_millis(100));
            
            // Process should have run
            terminal.process();
            
            let content = terminal.content();
            // The output should contain "hello" somewhere
            let all_text: String = content.lines.join("\n");
            assert!(
                all_text.contains("hello"),
                "Output should contain 'hello', got: {}",
                all_text
            );
        }
    }

    #[test]
    fn test_terminal_with_command_and_args() {
        // Test command with multiple arguments: "ls -la"
        let result = TerminalHandle::with_command("ls -la", 80, 24);
        
        if let Ok(mut terminal) = result {
            std::thread::sleep(std::time::Duration::from_millis(200));
            terminal.process();
            
            let content = terminal.content();
            let all_text: String = content.lines.join("\n");
            
            // ls -la should show "total" or "drwx" or similar
            assert!(
                all_text.contains("total") || all_text.contains("drwx") || all_text.contains("rw"),
                "ls -la output should contain directory listing, got: {}",
                all_text
            );
        }
    }

    #[test]
    fn test_terminal_with_tilde_expansion() {
        // Test that ~ is expanded by the shell
        let result = TerminalHandle::with_command("echo ~", 80, 24);
        
        if let Ok(mut terminal) = result {
            std::thread::sleep(std::time::Duration::from_millis(100));
            terminal.process();
            
            let content = terminal.content();
            let all_text: String = content.lines.join("\n");
            
            // ~ should be expanded to home directory (starts with /)
            // It should NOT literally contain "~" as the output
            assert!(
                all_text.contains("/Users") || all_text.contains("/home") || all_text.contains("/root"),
                "~ should be expanded to home directory path, got: {}",
                all_text
            );
        }
    }

    #[test]
    fn test_terminal_with_env_var_expansion() {
        // Test that environment variables are expanded
        let result = TerminalHandle::with_command("echo $HOME", 80, 24);
        
        if let Ok(mut terminal) = result {
            std::thread::sleep(std::time::Duration::from_millis(100));
            terminal.process();
            
            let content = terminal.content();
            let all_text: String = content.lines.join("\n");
            
            // $HOME should be expanded to home directory
            assert!(
                all_text.contains("/Users") || all_text.contains("/home") || all_text.contains("/root"),
                "$HOME should be expanded to home directory path, got: {}",
                all_text
            );
        }
    }

    #[test]
    fn test_terminal_with_pipe() {
        // Test that pipes work
        let result = TerminalHandle::with_command("echo hello | tr a-z A-Z", 80, 24);
        
        if let Ok(mut terminal) = result {
            std::thread::sleep(std::time::Duration::from_millis(100));
            terminal.process();
            
            let content = terminal.content();
            let all_text: String = content.lines.join("\n");
            
            // Should contain "HELLO" (uppercase)
            assert!(
                all_text.contains("HELLO"),
                "Pipe should work, expected 'HELLO', got: {}",
                all_text
            );
        }
    }
}
