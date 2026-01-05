#![allow(dead_code)]
//! Unified Window Registry
//!
//! Provides a single registry for all application windows, replacing separate
//! `OnceLock<Mutex<Option<WindowHandle>>>` statics in each window module.
//!
//! # Benefits
//!
//! - Consistent lifecycle handling across all windows
//! - Single place for cross-window operations (e.g., notify_all_windows for theme changes)
//! - Easier to reason about window state
//! - Prevents drift between window implementations
//!
//! # Usage
//!
//! ```rust,ignore
//! use crate::windows::{WindowRole, register_window, get_window, clear_window};
//!
//! // Register a window after creation
//! let handle = cx.open_window(options, |window, cx| { ... })?;
//! register_window(WindowRole::Notes, handle);
//!
//! // Get a window handle (returns Copy)
//! if let Some(handle) = get_window(WindowRole::Notes) {
//!     handle.update(cx, |root, window, cx| { ... });
//! }
//!
//! // Clear when window closes
//! clear_window(WindowRole::Notes);
//!
//! // Notify all windows (e.g., for theme changes)
//! notify_all_windows(cx);
//! ```

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use gpui::{App, WindowHandle};
use gpui_component::Root;

/// Identifies the role/purpose of a window in the application.
///
/// Each role corresponds to a distinct window type with its own behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowRole {
    /// The main Script Kit launcher window
    Main,
    /// The Notes window (floating panel)
    Notes,
    /// The AI Chat window
    AiChat,
}

impl WindowRole {
    /// Get a human-readable name for this role (for logging)
    pub fn name(&self) -> &'static str {
        match self {
            WindowRole::Main => "Main",
            WindowRole::Notes => "Notes",
            WindowRole::AiChat => "AI",
        }
    }
}

/// Internal registry state
struct WindowRegistry {
    /// Map of window roles to their handles
    handles: HashMap<WindowRole, WindowHandle<Root>>,
}

impl WindowRegistry {
    fn new() -> Self {
        Self {
            handles: HashMap::new(),
        }
    }
}

/// Global singleton for the window registry
static REGISTRY: OnceLock<Mutex<WindowRegistry>> = OnceLock::new();

/// Get or initialize the global registry
fn registry() -> &'static Mutex<WindowRegistry> {
    REGISTRY.get_or_init(|| Mutex::new(WindowRegistry::new()))
}

/// Register a window with a specific role.
///
/// Call this after GPUI creates a window to track it by role.
/// Subsequent calls with the same role will overwrite the previous registration.
///
/// # Arguments
/// * `role` - The purpose/role of this window
/// * `handle` - The GPUI window handle
pub fn register_window(role: WindowRole, handle: WindowHandle<Root>) {
    if let Ok(mut reg) = registry().lock() {
        crate::logging::log(
            "WINDOW_REG",
            &format!("Registering window: {:?}", role.name()),
        );
        reg.handles.insert(role, handle);
    }
}

/// Get a window by its role.
///
/// # Arguments
/// * `role` - The role to look up
///
/// # Returns
/// The window handle if registered (WindowHandle is Copy)
pub fn get_window(role: WindowRole) -> Option<WindowHandle<Root>> {
    registry()
        .lock()
        .ok()
        .and_then(|reg| reg.handles.get(&role).copied())
}

/// Clear a window registration.
///
/// Call this when a window is closed to clean up the registry.
///
/// # Arguments
/// * `role` - The role to clear
pub fn clear_window(role: WindowRole) {
    if let Ok(mut reg) = registry().lock() {
        if reg.handles.remove(&role).is_some() {
            crate::logging::log(
                "WINDOW_REG",
                &format!("Cleared window registration: {:?}", role.name()),
            );
        }
    }
}

/// Check if a window is currently registered and open.
///
/// # Arguments
/// * `role` - The role to check
///
/// # Returns
/// `true` if a window is registered for this role
pub fn is_window_open(role: WindowRole) -> bool {
    registry()
        .lock()
        .ok()
        .map(|reg| reg.handles.contains_key(&role))
        .unwrap_or(false)
}

/// Notify all registered windows to re-render.
///
/// This is useful for broadcasting changes like theme updates.
///
/// # Arguments
/// * `cx` - The GPUI App context
pub fn notify_all_windows(cx: &mut App) {
    // Collect handles first (release lock before calling update)
    let handles: Vec<WindowHandle<Root>> = registry()
        .lock()
        .ok()
        .map(|reg| reg.handles.values().copied().collect())
        .unwrap_or_default();

    let count = handles.len();

    // Notify each window (WindowHandle is Copy so we can iterate directly)
    for handle in handles {
        let _ = handle.update(cx, |_root, _window, cx| {
            cx.notify();
        });
    }

    if count > 0 {
        crate::logging::log("WINDOW_REG", &format!("Notified {} window(s)", count));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_role_name() {
        assert_eq!(WindowRole::Main.name(), "Main");
        assert_eq!(WindowRole::Notes.name(), "Notes");
        assert_eq!(WindowRole::AiChat.name(), "AI");
    }

    #[test]
    fn test_window_role_eq() {
        assert_eq!(WindowRole::Main, WindowRole::Main);
        assert_ne!(WindowRole::Main, WindowRole::Notes);
    }

    #[test]
    fn test_window_role_copy() {
        let role = WindowRole::Main;
        let role2 = role; // Copy
        assert_eq!(role, role2);
    }
}
