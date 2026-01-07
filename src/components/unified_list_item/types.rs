//! Type definitions for UnifiedListItem.

// Allow dead_code - this is new code not yet integrated into the main app
#![allow(dead_code)]

use gpui::*;
use std::ops::Range;
use std::sync::Arc;

// =============================================================================
// TextContent - Title/Subtitle with optional highlight ranges
// =============================================================================

/// Text content that can be plain or have highlight ranges (for fuzzy match display).
///
/// Ranges are **byte offsets** into the text. The fuzzy matcher must return valid
/// UTF-8 boundaries. In debug builds, we assert that ranges land on char boundaries.
///
/// Note: Custom variant prevents Clone; build fresh per render (normal for virtualized lists).
pub enum TextContent {
    /// Plain text with no highlighting.
    Plain(SharedString),

    /// Text with highlighted ranges (e.g., fuzzy match results).
    Highlighted {
        text: SharedString,
        ranges: Vec<Range<usize>>,
    },

    /// Custom element (for special rendering needs).
    Custom(AnyElement),
}

impl TextContent {
    /// Create plain text content.
    pub fn plain(text: impl Into<SharedString>) -> Self {
        Self::Plain(text.into())
    }

    /// Create highlighted text content with byte ranges.
    pub fn highlighted(text: impl Into<SharedString>, ranges: Vec<Range<usize>>) -> Self {
        let text = text.into();

        #[cfg(debug_assertions)]
        {
            let s = text.as_ref();
            for range in &ranges {
                assert!(
                    s.is_char_boundary(range.start),
                    "Range start {} is not a char boundary in '{}'",
                    range.start,
                    s
                );
                assert!(
                    s.is_char_boundary(range.end),
                    "Range end {} is not a char boundary in '{}'",
                    range.end,
                    s
                );
            }
        }

        Self::Highlighted { text, ranges }
    }

    /// Create custom element content.
    pub fn custom(element: impl IntoElement) -> Self {
        Self::Custom(element.into_any_element())
    }

    /// Get the text string (for a11y labels). Returns None for Custom.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Plain(s) => Some(s.as_ref()),
            Self::Highlighted { text, .. } => Some(text.as_ref()),
            Self::Custom(_) => None,
        }
    }
}

// =============================================================================
// ItemState - Visual state (passed in, not owned)
// =============================================================================

/// Visual state for a list item.
#[derive(Clone, Copy, Default)]
pub struct ItemState {
    pub is_selected: bool,
    pub is_hovered: bool,
    pub is_disabled: bool,
}

// =============================================================================
// LeadingContent - Left-side content
// =============================================================================

/// Content displayed on the left side of the list item.
///
/// Note: Custom variant prevents Clone; use standard variants when possible.
pub enum LeadingContent {
    /// Emoji string (e.g., "📋").
    Emoji(SharedString),

    /// SVG icon by name with optional color override.
    Icon {
        name: SharedString,
        color: Option<u32>,
    },

    /// Pre-decoded app icon image.
    AppIcon(Arc<RenderImage>),

    /// Placeholder while app icon loads.
    AppIconPlaceholder,

    /// Custom element (use sparingly).
    Custom(AnyElement),
}

// =============================================================================
// TrailingContent - Right-side content
// =============================================================================

/// Content displayed on the right side of the list item.
///
/// Note: Custom variant prevents Clone; use standard variants when possible.
pub enum TrailingContent {
    /// Keyboard shortcut badge (e.g., "⌘O").
    Shortcut(SharedString),

    /// Navigation hint (e.g., "Enter").
    Hint(SharedString),

    /// Item count badge.
    Count(usize),

    /// Right chevron for navigation.
    Chevron,

    /// Checkmark for selected items.
    Checkmark,

    /// Custom element (use sparingly).
    Custom(AnyElement),
}

// =============================================================================
// Density - Single knob for layout sizing
// =============================================================================

/// Layout density for list items.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Density {
    /// Comfortable spacing (48px height).
    #[default]
    Comfortable,

    /// Compact spacing (40px height).
    Compact,
}

// =============================================================================
// ListItemLayout - Computed layout values
// =============================================================================

/// Pre-computed layout values based on density.
#[derive(Clone, Copy)]
pub struct ListItemLayout {
    pub height: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
    pub leading_size: f32,
    pub radius: f32,
}

impl ListItemLayout {
    /// Compute layout from density.
    pub fn from_density(density: Density) -> Self {
        match density {
            Density::Comfortable => Self {
                height: 48.0,
                padding_x: 12.0,
                padding_y: 6.0,
                gap: 8.0,
                leading_size: 20.0,
                radius: 6.0,
            },
            Density::Compact => Self {
                height: 40.0,
                padding_x: 8.0,
                padding_y: 4.0,
                gap: 6.0,
                leading_size: 16.0,
                radius: 4.0,
            },
        }
    }
}

// =============================================================================
// UnifiedListItemColors - Pre-computed colors
// =============================================================================

/// Pre-computed colors for UnifiedListItem rendering.
#[derive(Clone, Copy)]
pub struct UnifiedListItemColors {
    pub text_primary: u32,
    pub text_secondary: u32,
    pub text_muted: u32,
    pub text_dimmed: u32,
    pub text_highlight: u32,
    pub accent: u32,
    pub accent_subtle: u32,
    pub background: u32,
    pub selected_opacity: f32,
    pub hover_opacity: f32,
}

impl Default for UnifiedListItemColors {
    fn default() -> Self {
        Self {
            text_primary: 0xFFFFFF,
            text_secondary: 0xCCCCCC,
            text_muted: 0x888888,
            text_dimmed: 0x666666,
            text_highlight: 0x4A90D9,
            accent: 0x4A90D9,
            accent_subtle: 0x4A90D9,
            background: 0x1E1E1E,
            selected_opacity: 0.20,
            hover_opacity: 0.10,
        }
    }
}

impl UnifiedListItemColors {
    /// Create from theme reference.
    pub fn from_theme(theme: &crate::theme::Theme) -> Self {
        let opacity = theme.get_opacity();
        Self {
            text_primary: theme.colors.text.primary,
            text_secondary: theme.colors.text.secondary,
            text_muted: theme.colors.text.muted,
            text_dimmed: theme.colors.text.dimmed,
            text_highlight: theme.colors.accent.selected,
            accent: theme.colors.accent.selected,
            accent_subtle: theme.colors.accent.selected_subtle,
            background: theme.colors.background.main,
            selected_opacity: opacity.selected,
            hover_opacity: opacity.hover,
        }
    }
}

/// Height for section headers in grouped lists.
pub const SECTION_HEADER_HEIGHT: f32 = 24.0;
