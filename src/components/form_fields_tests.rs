//! Unit tests for form field text indexing helpers.
//!
//! These tests verify the UTF-8 char/byte conversion functions used by form fields.
//! Separated from form_fields.rs due to GPUI macro recursion limit issues.

/// Count the number of Unicode scalar values (chars) in a string.
fn char_len(s: &str) -> usize {
    s.chars().count()
}

/// Convert a character index (0..=char_len) into a byte index (0..=s.len()).
/// If char_idx is past the end, returns s.len().
fn byte_idx_from_char_idx(s: &str, char_idx: usize) -> usize {
    if char_idx == 0 {
        return 0;
    }
    s.char_indices()
        .nth(char_idx)
        .map(|(byte_idx, _)| byte_idx)
        .unwrap_or_else(|| s.len())
}

/// Remove a char range [start_char, end_char) from a String (char indices).
fn drain_char_range(s: &mut String, start_char: usize, end_char: usize) {
    let start_b = byte_idx_from_char_idx(s, start_char);
    let end_b = byte_idx_from_char_idx(s, end_char);
    if start_b < end_b && start_b <= s.len() && end_b <= s.len() {
        s.drain(start_b..end_b);
    }
}

/// Slice a &str by char indices [start_char, end_char).
fn slice_by_char_range(s: &str, start_char: usize, end_char: usize) -> &str {
    let start_b = byte_idx_from_char_idx(s, start_char);
    let end_b = byte_idx_from_char_idx(s, end_char);
    &s[start_b..end_b]
}

// --- Text indexing helper tests ---

#[test]
fn test_char_len_ascii() {
    assert_eq!(char_len("hello"), 5);
    assert_eq!(char_len(""), 0);
}

#[test]
fn test_char_len_emoji() {
    // Each emoji is 1 char but 4 bytes
    assert_eq!(char_len("😀"), 1);
    assert_eq!(char_len("😀😀"), 2);
    assert_eq!(char_len("a😀b"), 3);
}

#[test]
fn test_char_len_multibyte() {
    // "•" (bullet) is 3 bytes, 1 char
    assert_eq!(char_len("•"), 1);
    assert_eq!(char_len("•••"), 3);
    // Japanese
    assert_eq!(char_len("日本語"), 3);
}

#[test]
fn test_byte_idx_from_char_idx_ascii() {
    let s = "hello";
    assert_eq!(byte_idx_from_char_idx(s, 0), 0);
    assert_eq!(byte_idx_from_char_idx(s, 1), 1);
    assert_eq!(byte_idx_from_char_idx(s, 5), 5);
    // Past end
    assert_eq!(byte_idx_from_char_idx(s, 10), 5);
}

#[test]
fn test_byte_idx_from_char_idx_emoji() {
    let s = "a😀b"; // a=1 byte, 😀=4 bytes, b=1 byte
    assert_eq!(byte_idx_from_char_idx(s, 0), 0); // before 'a'
    assert_eq!(byte_idx_from_char_idx(s, 1), 1); // before '😀'
    assert_eq!(byte_idx_from_char_idx(s, 2), 5); // before 'b' (1+4)
    assert_eq!(byte_idx_from_char_idx(s, 3), 6); // end
}

#[test]
fn test_byte_idx_from_char_idx_bullet() {
    let s = "•••"; // 3 bullets, each 3 bytes = 9 bytes total
    assert_eq!(byte_idx_from_char_idx(s, 0), 0);
    assert_eq!(byte_idx_from_char_idx(s, 1), 3);
    assert_eq!(byte_idx_from_char_idx(s, 2), 6);
    assert_eq!(byte_idx_from_char_idx(s, 3), 9);
}

#[test]
fn test_slice_by_char_range_ascii() {
    let s = "hello";
    assert_eq!(slice_by_char_range(s, 0, 2), "he");
    assert_eq!(slice_by_char_range(s, 2, 5), "llo");
    assert_eq!(slice_by_char_range(s, 0, 5), "hello");
}

#[test]
fn test_slice_by_char_range_emoji() {
    let s = "a😀b";
    assert_eq!(slice_by_char_range(s, 0, 1), "a");
    assert_eq!(slice_by_char_range(s, 1, 2), "😀");
    assert_eq!(slice_by_char_range(s, 2, 3), "b");
    assert_eq!(slice_by_char_range(s, 0, 3), "a😀b");
}

#[test]
fn test_slice_by_char_range_bullet() {
    let s = "•••";
    assert_eq!(slice_by_char_range(s, 0, 1), "•");
    assert_eq!(slice_by_char_range(s, 1, 2), "•");
    assert_eq!(slice_by_char_range(s, 0, 2), "••");
}

#[test]
fn test_drain_char_range_ascii() {
    let mut s = "hello".to_string();
    drain_char_range(&mut s, 1, 3);
    assert_eq!(s, "hlo");
}

#[test]
fn test_drain_char_range_emoji() {
    let mut s = "a😀b".to_string();
    drain_char_range(&mut s, 1, 2); // remove emoji
    assert_eq!(s, "ab");
}

#[test]
fn test_drain_char_range_bullet() {
    let mut s = "•••".to_string();
    drain_char_range(&mut s, 1, 2); // remove middle bullet
    assert_eq!(s, "••");
}

// --- Password bullet rendering tests ---

/// Test that password bullet string can be safely sliced by char index.
/// This test verifies the FIX for the bug where render() slices bullet
/// strings using cursor_position directly (which is a char index).
#[test]
fn test_password_bullet_slicing_safe() {
    let password = "abc"; // 3 chars
    let bullets = "•".repeat(char_len(password)); // "•••" = 9 bytes
    let cursor_pos: usize = 2; // char index

    // This is the CORRECT way to slice (using char indices):
    let before = slice_by_char_range(&bullets, 0, cursor_pos);
    let after = slice_by_char_range(&bullets, cursor_pos, char_len(&bullets));

    assert_eq!(before, "••");
    assert_eq!(after, "•");
}

/// This test documents the bug in the original render code.
/// It should panic because slicing "•••" at byte index 2 is not a char boundary.
#[test]
#[should_panic(expected = "byte index 2 is not a char boundary")]
fn test_password_bullet_slicing_bug_panics() {
    let password = "abc"; // 3 chars
    let bullets = "•".repeat(char_len(password)); // "•••" = 9 bytes
    let cursor_pos: usize = 2; // This is treated as byte index in buggy code

    // This is the BUGGY way (what the old render code does):
    // It treats cursor_pos (char index) as byte index
    let _before = &bullets[..cursor_pos]; // PANIC: byte 2 is not char boundary
}
