//! Window management module
//!
//! This module provides a unified window registry for managing multiple windows
//! (Main, Notes, AI) in a consistent way. It replaces the per-window statics
//! with a single registry, ensuring consistent lifecycle handling.

mod registry;

pub use registry::{
    clear_window, get_window, is_window_open, notify_all_windows, register_window, WindowRole,
};
