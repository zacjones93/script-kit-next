//! Unit tests for UnifiedListItem component types and helpers.
//!
//! Tests verify:
//! - TextContent highlight range slicing
//! - UTF-8 boundary safety for highlight ranges
//! - Density layout calculations
//! - A11y label generation

#![allow(clippy::single_range_in_vec_init)]

use std::ops::Range;

use super::unified_list_item::{
    Density, ItemState, LeadingContent, ListItemLayout, TextContent, TrailingContent,
};

// =============================================================================
// TextContent Tests - Highlight Ranges
// =============================================================================

#[test]
fn test_text_content_plain_creation() {
    let text = TextContent::plain("Hello World");
    assert!(matches!(text, TextContent::Plain(_)));
}

#[test]
fn test_text_content_highlighted_creation() {
    let text = TextContent::highlighted("Hello World", vec![0..5]);
    match text {
        TextContent::Highlighted { text, ranges } => {
            assert_eq!(text.as_ref(), "Hello World");
            assert_eq!(ranges.len(), 1);
            assert_eq!(ranges[0], 0..5);
        }
        _ => panic!("Expected Highlighted variant"),
    }
}

#[test]
fn test_text_content_empty_ranges() {
    let text = TextContent::highlighted("Hello", vec![]);
    match text {
        TextContent::Highlighted { ranges, .. } => {
            assert!(ranges.is_empty());
        }
        _ => panic!("Expected Highlighted variant"),
    }
}

#[test]
fn test_text_content_multiple_ranges() {
    // "Hello World" - highlight "Hello" and "World"
    let text = TextContent::highlighted("Hello World", vec![0..5, 6..11]);
    match text {
        TextContent::Highlighted { ranges, .. } => {
            assert_eq!(ranges.len(), 2);
            assert_eq!(ranges[0], 0..5);
            assert_eq!(ranges[1], 6..11);
        }
        _ => panic!("Expected Highlighted variant"),
    }
}

// =============================================================================
// UTF-8 Boundary Safety Tests
// =============================================================================

/// Helper to split text into highlighted and non-highlighted spans.
/// Returns Vec<(text_slice, is_highlighted)>.
fn split_by_ranges<'a>(text: &'a str, ranges: &[Range<usize>]) -> Vec<(&'a str, bool)> {
    if ranges.is_empty() {
        return vec![(text, false)];
    }

    let mut result = Vec::new();
    let mut current_byte = 0;

    for range in ranges {
        // Non-highlighted portion before this range
        if range.start > current_byte {
            result.push((&text[current_byte..range.start], false));
        }
        // Highlighted portion
        if range.end > range.start && range.end <= text.len() {
            result.push((&text[range.start..range.end], true));
        }
        current_byte = range.end;
    }

    // Remaining non-highlighted portion
    if current_byte < text.len() {
        result.push((&text[current_byte..], false));
    }

    result
}

#[test]
fn test_split_by_ranges_ascii() {
    let text = "Hello World";
    let ranges = vec![0..5]; // "Hello"

    let spans = split_by_ranges(text, &ranges);
    assert_eq!(spans.len(), 2);
    assert_eq!(spans[0], ("Hello", true));
    assert_eq!(spans[1], (" World", false));
}

#[test]
fn test_split_by_ranges_multiple() {
    let text = "Hello World Test";
    let ranges = vec![0..5, 12..16]; // "Hello" and "Test"

    let spans = split_by_ranges(text, &ranges);
    // Expected: "Hello" (highlighted), " World " (not), "Test" (highlighted)
    // No trailing because "Test" ends at position 16 which is end of string
    assert_eq!(spans.len(), 3);
    assert_eq!(spans[0], ("Hello", true));
    assert_eq!(spans[1], (" World ", false));
    assert_eq!(spans[2], ("Test", true));
}

#[test]
fn test_split_by_ranges_emoji_safe() {
    // "a😀b" - highlight only the emoji
    // a = 1 byte, 😀 = 4 bytes, b = 1 byte
    // So emoji is at byte range 1..5
    let text = "a😀b";
    let ranges = vec![1..5]; // The emoji bytes

    let spans = split_by_ranges(text, &ranges);
    assert_eq!(spans[0], ("a", false));
    assert_eq!(spans[1], ("😀", true));
    assert_eq!(spans[2], ("b", false));
}

#[test]
fn test_split_by_ranges_japanese() {
    // "日本語" - each char is 3 bytes
    let text = "日本語";
    let ranges = vec![3..6]; // "本" (middle char)

    let spans = split_by_ranges(text, &ranges);
    assert_eq!(spans[0], ("日", false));
    assert_eq!(spans[1], ("本", true));
    assert_eq!(spans[2], ("語", false));
}

#[test]
fn test_split_by_ranges_empty() {
    let text = "Hello";
    let ranges: Vec<Range<usize>> = vec![];

    let spans = split_by_ranges(text, &ranges);
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0], ("Hello", false));
}

// =============================================================================
// ItemState Tests
// =============================================================================

#[test]
fn test_item_state_default() {
    let state = ItemState::default();
    assert!(!state.is_selected);
    assert!(!state.is_hovered);
    assert!(!state.is_disabled);
}

#[test]
fn test_item_state_selected() {
    let state = ItemState {
        is_selected: true,
        ..Default::default()
    };
    assert!(state.is_selected);
    assert!(!state.is_hovered);
}

#[test]
fn test_item_state_hovered() {
    let state = ItemState {
        is_hovered: true,
        ..Default::default()
    };
    assert!(!state.is_selected);
    assert!(state.is_hovered);
}

// =============================================================================
// Density & Layout Tests
// =============================================================================

#[test]
fn test_density_comfortable_layout() {
    let layout = ListItemLayout::from_density(Density::Comfortable);
    assert_eq!(layout.height, 48.0);
    assert!(layout.padding_x >= 12.0);
    assert!(layout.leading_size >= 20.0);
}

#[test]
fn test_density_compact_layout() {
    let layout = ListItemLayout::from_density(Density::Compact);
    assert_eq!(layout.height, 40.0);
    assert!(layout.padding_x >= 8.0);
    assert!(layout.leading_size >= 16.0);
}

#[test]
fn test_layout_height_is_fixed() {
    // Verify both densities have fixed heights (important for uniform_list)
    let comfortable = ListItemLayout::from_density(Density::Comfortable);
    let compact = ListItemLayout::from_density(Density::Compact);

    // Heights should be fixed, non-zero values
    assert!(comfortable.height > 0.0);
    assert!(compact.height > 0.0);
    assert!(comfortable.height > compact.height);
}

// =============================================================================
// LeadingContent Tests
// =============================================================================

#[test]
fn test_leading_content_emoji() {
    let leading = LeadingContent::Emoji("📋".into());
    assert!(matches!(leading, LeadingContent::Emoji(_)));
}

#[test]
fn test_leading_content_icon() {
    let leading = LeadingContent::Icon {
        name: "terminal".into(),
        color: None,
    };
    match leading {
        LeadingContent::Icon { name, color } => {
            assert_eq!(name.as_ref(), "terminal");
            assert!(color.is_none());
        }
        _ => panic!("Expected Icon variant"),
    }
}

#[test]
fn test_leading_content_icon_with_color() {
    let leading = LeadingContent::Icon {
        name: "file".into(),
        color: Some(0xFF0000),
    };
    match leading {
        LeadingContent::Icon { color, .. } => {
            assert_eq!(color, Some(0xFF0000));
        }
        _ => panic!("Expected Icon variant"),
    }
}

// =============================================================================
// TrailingContent Tests
// =============================================================================

#[test]
fn test_trailing_content_shortcut() {
    let trailing = TrailingContent::Shortcut("⌘O".into());
    assert!(matches!(trailing, TrailingContent::Shortcut(_)));
}

#[test]
fn test_trailing_content_count() {
    let trailing = TrailingContent::Count(42);
    match trailing {
        TrailingContent::Count(n) => assert_eq!(n, 42),
        _ => panic!("Expected Count variant"),
    }
}

#[test]
fn test_trailing_content_chevron() {
    let trailing = TrailingContent::Chevron;
    assert!(matches!(trailing, TrailingContent::Chevron));
}

#[test]
fn test_trailing_content_checkmark() {
    let trailing = TrailingContent::Checkmark;
    assert!(matches!(trailing, TrailingContent::Checkmark));
}

// =============================================================================
// A11y Label Generation Tests
// =============================================================================

/// Generate accessibility label from title and optional subtitle.
fn generate_a11y_label(title: &str, subtitle: Option<&str>) -> String {
    match subtitle {
        Some(sub) => format!("{}, {}", title, sub),
        None => title.to_string(),
    }
}

#[test]
fn test_a11y_label_title_only() {
    let label = generate_a11y_label("Open File", None);
    assert_eq!(label, "Open File");
}

#[test]
fn test_a11y_label_with_subtitle() {
    let label = generate_a11y_label("Open File", Some("Opens a file dialog"));
    assert_eq!(label, "Open File, Opens a file dialog");
}

#[test]
fn test_a11y_label_empty_subtitle() {
    let label = generate_a11y_label("Save", Some(""));
    assert_eq!(label, "Save, ");
}
