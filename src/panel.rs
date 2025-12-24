// macOS Panel Configuration Module
// This module configures GPUI windows as macOS floating panels
// that appear above other applications with native vibrancy effects

/// Vibrancy configuration for GPUI window background appearance
/// 
/// GPUI supports three WindowBackgroundAppearance values:
/// - Opaque: Solid, no transparency
/// - Transparent: Fully transparent
/// - Blurred: macOS vibrancy effect (recommended for Spotlight/Raycast-like feel)
/// 
/// The actual vibrancy effect is achieved through:
/// 1. Setting `WindowBackgroundAppearance::Blurred` in WindowOptions (done in main.rs)
/// 2. Using semi-transparent background colors (controlled by theme opacity settings)
/// 
/// The blur shows through the transparent portions of the window background,
/// creating the native macOS vibrancy effect similar to Spotlight and Raycast.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowVibrancy {
    /// Solid, opaque background - no vibrancy effect
    Opaque,
    /// Transparent background without blur
    Transparent,
    /// macOS vibrancy/blur effect - the native feel
    /// This is the recommended setting for Spotlight/Raycast-like appearance
    Blurred,
}

impl Default for WindowVibrancy {
    fn default() -> Self {
        // Default to Blurred for native macOS feel
        WindowVibrancy::Blurred
    }
}

impl WindowVibrancy {
    /// Check if this vibrancy setting enables the blur effect
    pub fn is_blurred(&self) -> bool {
        matches!(self, WindowVibrancy::Blurred)
    }
    
    /// Check if this vibrancy setting is fully opaque
    pub fn is_opaque(&self) -> bool {
        matches!(self, WindowVibrancy::Opaque)
    }
}

#[cfg(target_os = "macos")]
/// Configure the current key window as a floating panel window that appears above other apps.
///
/// This function:
/// - Sets the window level to NSFloatingWindowLevel (3) so it floats above normal windows
/// - Sets collection behavior to appear on all spaces/desktops
/// - Keeps the window visible when switching between applications
///
/// Note: Vibrancy/blur effect is configured via WindowBackgroundAppearance::Blurred
/// in WindowOptions when the window is created (see main.rs).
///
/// Should be called immediately after the window is created and visible.
pub fn configure_as_floating_panel() {
    // This will be called from main.rs where objc macros are available
    // The actual implementation is in main.rs to avoid macro issues in lib code
    crate::logging::log("PANEL", "Panel configuration (implemented in main.rs)");
}

#[cfg(not(target_os = "macos"))]
/// No-op on non-macOS platforms
pub fn configure_as_floating_panel() {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_vibrancy() {
        assert_eq!(WindowVibrancy::default(), WindowVibrancy::Blurred);
    }
    
    #[test]
    fn test_vibrancy_is_blurred() {
        assert!(WindowVibrancy::Blurred.is_blurred());
        assert!(!WindowVibrancy::Opaque.is_blurred());
        assert!(!WindowVibrancy::Transparent.is_blurred());
    }
    
    #[test]
    fn test_vibrancy_is_opaque() {
        assert!(WindowVibrancy::Opaque.is_opaque());
        assert!(!WindowVibrancy::Blurred.is_opaque());
        assert!(!WindowVibrancy::Transparent.is_opaque());
    }
}
