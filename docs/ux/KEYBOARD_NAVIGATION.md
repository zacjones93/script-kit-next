# Keyboard Navigation & Focus States Audit

## Executive Summary

This document provides a comprehensive audit of keyboard navigation and focus handling in the Script Kit GPUI application. The codebase demonstrates solid keyboard navigation patterns with consistent arrow key handling across all components.

**Key Findings:**
- ✅ Arrow key handling properly matches BOTH variants (`"up" | "arrowup"`, etc.) across all components
- ✅ Focus management uses proper `FocusHandle` and `Focusable` trait implementations
- ✅ Cursor blinking implemented with centralized timer and focus-aware visibility
- ⚠️ No explicit tab navigation between UI elements (intentional for launcher UX)
- ⚠️ Limited focus ring/outline styling (uses background colors instead)
- ✅ Event coalescing via scroll stabilization prevents jitter

---

## 1. Key Event Handling Patterns

### 1.1 KeyDownEvent Handler Pattern

All components follow a consistent pattern for handling keyboard events:

```rust
let handle_key = cx.listener(move |this: &mut Self, event: &gpui::KeyDownEvent, window: &mut Window, cx: &mut Context<Self>| {
    let key_str = event.keystroke.key.to_lowercase();
    let has_cmd = event.keystroke.modifiers.platform;
    let has_shift = event.keystroke.modifiers.shift;
    
    match key_str.as_str() {
        "up" | "arrowup" => this.move_up(cx),
        "down" | "arrowdown" => this.move_down(cx),
        // ...
    }
});
```

**Location:** `src/main.rs:4611`, `src/actions.rs:462`, `src/prompts.rs:247`, `src/editor.rs:700+`

### 1.2 Event Registration

Events are registered using `.on_key_down()` chained with `.track_focus()`:

```rust
div()
    .key_context("prompt_name")
    .track_focus(&self.focus_handle)
    .on_key_down(handle_key)
```

This pattern is used consistently across:
- `ScriptListApp` (main.rs:4809-4810)
- `ActionsDialog` (actions.rs:863-864)
- `ArgPrompt` (prompts.rs:391-392)
- `DivPrompt` (prompts.rs:516-517)
- `TermPrompt` (term_prompt.rs:591-592)
- `EditorPrompt` (editor.rs - managed separately)

---

## 2. Arrow Key Navigation

### 2.1 Cross-Platform Arrow Key Matching

**CRITICAL:** The codebase correctly handles BOTH arrow key name variants per AGENTS.md requirements:

| Component | Location | Pattern Used |
|-----------|----------|--------------|
| ScriptListApp | main.rs:4726-4727 | `"up" \| "arrowup"`, `"down" \| "arrowdown"` |
| ScriptListApp (actions popup) | main.rs:4677-4684 | `"up" \| "arrowup"`, `"down" \| "arrowdown"` |
| ArgPrompt | prompts.rs:251-252 | `"up" \| "arrowup"`, `"down" \| "arrowdown"` |
| ActionsDialog | actions.rs:466-467 | `"up" \| "arrowup"`, `"down" \| "arrowdown"` |
| ClipboardHistory | main.rs:5745-5757 | `"up" \| "arrowup"`, `"down" \| "arrowdown"` |
| WindowSwitcher | main.rs:6611-6621 | `"up" \| "arrowup"`, `"down" \| "arrowdown"` |
| DesignGallery | main.rs:7180-7190 | `"up" \| "arrowup"`, `"down" \| "arrowdown"` |
| EditorPrompt | editor.rs:716-733 | All four directions with both variants |
| TermPrompt | term_prompt.rs:523-526 | All four directions with both variants |

### 2.2 Editor Full Arrow Key Support

The editor has comprehensive arrow key handling with modifier combinations:

```rust
// editor.rs:716-735
("left" | "arrowleft", false, _, false) => self.move_left(shift),
("right" | "arrowright", false, _, false) => self.move_right(shift),
("up" | "arrowup", false, _, false) => self.move_up(shift),
("down" | "arrowdown", false, _, false) => self.move_down(shift),

// Word navigation (Alt/Option + arrow)
("left" | "arrowleft", false, _, true) => self.move_word_left(shift),
("right" | "arrowright", false, _, true) => self.move_word_right(shift),

// Line start/end (Cmd+Left/Right)
("left" | "arrowleft", true, _, false) => self.move_to_line_start(shift),
("right" | "arrowright", true, _, false) => self.move_to_line_end(shift),

// Document start/end (Cmd+Up/Down)
("up" | "arrowup", true, _, false) => self.move_to_document_start(shift),
("down" | "arrowdown", true, _, false) => self.move_to_document_end(shift),
```

### 2.3 Terminal Arrow Key Handling

Terminal translates arrow keys to ANSI escape sequences:

```rust
// term_prompt.rs:523-526
"up" | "arrowup" => Some(b"\x1b[A"),
"down" | "arrowdown" => Some(b"\x1b[B"),
"right" | "arrowright" => Some(b"\x1b[C"),
"left" | "arrowleft" => Some(b"\x1b[D"),
```

### 2.4 Arrow Key Verification Test

The editor includes explicit tests for arrow key patterns:

```rust
// editor.rs:1230-1262
/// "arrowup"/"arrowdown"/"arrowleft"/"arrowright" for cross-platform compatibility.
#[test]
fn test_arrow_key_handling_patterns() {
    // Verifies patterns like:
    r#""up" | "arrowup""#,
    r#""down" | "arrowdown""#,
    r#""left" | "arrowleft""#,
    r#""right" | "arrowright""#,
}
```

---

## 3. Focus Handle Implementation

### 3.1 Focusable Trait Implementations

All interactive components implement the `Focusable` trait:

| Component | Location | Implementation |
|-----------|----------|----------------|
| ScriptListApp | main.rs:3529-3532 | `focus_handle.clone()` |
| ActionsDialog | actions.rs:448-451 | `focus_handle.clone()` |
| ArgPrompt | prompts.rs:231-234 | `focus_handle.clone()` |
| DivPrompt | prompts.rs:460-463 | `focus_handle.clone()` |
| TermPrompt | term_prompt.rs:431-434 | `focus_handle.clone()` |
| EditorPrompt | editor.rs:1058+ | `focus_handle.clone()` |

### 3.2 Focus Handle Creation

Focus handles are created in constructors using `cx.focus_handle()`:

```rust
// main.rs:918
focus_handle: cx.focus_handle(),

// prompts.rs:76
focus_handle,  // Passed from caller

// editor.rs:208
focus_handle: cx.focus_handle(),
```

### 3.3 Focus Management in Render

The main app manages focus in its render method based on current view:

```rust
// main.rs:3540-3558
match &current_view {
    AppView::EditorPrompt { focus_handle, .. } => {
        // EditorPrompt has its own focus handle
        let is_focused = focus_handle.is_focused(window);
        if !is_focused {
            window.focus(focus_handle, cx);
        }
    }
    _ => {
        // Other views use the parent's focus handle
        let is_focused = self.focus_handle.is_focused(window);
        if !is_focused {
            window.focus(&self.focus_handle, cx);
        }
    }
}
```

### 3.4 Focus Restoration on Dialog Close

When closing popups, focus is explicitly returned:

```rust
// main.rs:4693, 4702
window.focus(&this.focus_handle, cx);

// After actions dialog close
this.focused_input = FocusedInput::MainFilter;
window.focus(&this.focus_handle, cx);
```

---

## 4. FocusedInput State Tracking

### 4.1 FocusedInput Enum

The app tracks which input has focus for cursor visibility:

```rust
// main.rs:478-487
enum FocusedInput {
    MainFilter,      // Main script list filter input
    ActionsSearch,   // Actions dialog search input
    ArgPrompt,       // Arg prompt input (script prompts)
    None,            // No input focused (terminal, editor, div)
}
```

### 4.2 FocusedInput State Changes

| View Transition | FocusedInput State | Location |
|-----------------|-------------------|----------|
| Reset to script list | `MainFilter` | main.rs:3368 |
| Show actions popup | `ActionsSearch` | main.rs:1495 |
| Close actions popup | `MainFilter` | main.rs:4692, 4701 |
| Show ArgPrompt | `ArgPrompt` | main.rs:2781 |
| Show DivPrompt | `None` | main.rs:2794 |
| Show TermPrompt | `None` | main.rs:2836 |
| Show EditorPrompt | `None` | main.rs:2884 |

---

## 5. Focus Ring/Outline Styling

### 5.1 Current Approach: Background Colors

The application uses **background color changes** rather than focus rings/outlines for selection indication:

```rust
// List item selection styling (main.rs, actions.rs, prompts.rs)
.bg(if is_selected { 
    list_colors.background_selected 
} else { 
    list_colors.background 
})
```

### 5.2 Focus-Aware Color System

The theme system provides focus-aware colors:

```rust
// theme.rs:484-501
pub fn get_colors(&self, is_focused: bool) -> ColorScheme {
    if let Some(ref focus_aware) = self.focus_aware {
        if is_focused {
            if let Some(ref focused) = focus_aware.focused {
                return focused.to_color_scheme();
            }
        } else if let Some(ref unfocused) = focus_aware.unfocused {
            return unfocused.to_color_scheme();
        }
    }
    
    // Fallback: automatic dimming for unfocused
    if is_focused {
        self.colors.clone()
    } else {
        self.colors.to_unfocused()
    }
}
```

### 5.3 Window Unfocused Styling

Windows dim when unfocused:

```rust
// theme.rs:527-539
pub fn get_opacity_for_focus(&self, is_focused: bool) -> BackgroundOpacity {
    let base = self.get_opacity();
    if is_focused {
        base
    } else {
        // Reduce opacity by 10% when unfocused
        BackgroundOpacity {
            main: (base.main * 0.9).clamp(0.0, 1.0),
            // ...
        }
    }
}
```

### 5.4 Search Input Focus Indicator

The actions dialog uses a border color change for search focus:

```rust
// actions.rs:510-511
let accent_color_hex = colors.accent;
let focus_border_color = rgba(hex_with_alpha(accent_color_hex, 0x60));

// actions.rs:565-569
.border_color(if !self.search_text.is_empty() { 
    focus_border_color
} else { 
    border_color
})
```

---

## 6. Tab Order & Focus Trapping

### 6.1 Tab Navigation

**Finding:** There is **no explicit tab navigation** between UI elements.

This is **intentional** for a launcher application where:
- Arrow keys navigate lists
- Enter selects/confirms
- Escape cancels/hides
- Characters filter the list

### 6.2 Focus Trapping

Focus is implicitly trapped within each view:
- Each view has its own `focus_handle`
- The render method ensures focus stays on the current view
- Modal dialogs (ActionsDialog) receive focus when shown

```rust
// main.rs:1513-1515 - Focus trapping for actions dialog
let dialog_focus_handle = dialog.read(cx).focus_handle.clone();
window.focus(&dialog_focus_handle, cx);
```

---

## 7. Keyboard Shortcuts Documentation

### 7.1 Global Shortcuts (with Cmd modifier)

| Shortcut | Action | Location |
|----------|--------|----------|
| `Cmd+;` | Toggle window visibility | config.ts (configurable) |
| `Cmd+L` | Toggle logs panel | main.rs:4619-4622 |
| `Cmd+K` | Toggle actions dialog | main.rs:4623-4626 |
| `Cmd+1` | Cycle through designs | main.rs:4628-4631 |
| `Cmd+E` | Edit selected script | main.rs:4633-4637 |
| `Cmd+Shift+F` | Reveal in Finder | main.rs:4638-4642 |
| `Cmd+Shift+C` | Copy script path | main.rs:4643-4647 |
| `Cmd+N` | Create new script | main.rs:4649-4653 |
| `Cmd+R` | Reload scripts | main.rs:4654-4658 |
| `Cmd+,` | Open settings | main.rs:4659-4663 |
| `Cmd+Q` | Quit application | main.rs:4664-4668 |

### 7.2 List Navigation Shortcuts

| Shortcut | Action | Location |
|----------|--------|----------|
| `Up/ArrowUp` | Move selection up | main.rs:4726 |
| `Down/ArrowDown` | Move selection down | main.rs:4727 |
| `Enter` | Execute/select item | main.rs:4728 |
| `Escape` | Clear filter or hide window | main.rs:4729-4748 |
| `Backspace` | Delete last filter character | main.rs:4749 |
| `a-z, 0-9, -, _, space` | Add to filter | main.rs:4750-4757 |

### 7.3 Editor Shortcuts

| Shortcut | Action | Location |
|----------|--------|----------|
| `Cmd+Z` | Undo | editor.rs:703 |
| `Cmd+Shift+Z` | Redo | editor.rs:704 |
| `Cmd+C` | Copy | editor.rs:707 |
| `Cmd+X` | Cut | editor.rs:708 |
| `Cmd+V` | Paste | editor.rs:709 |
| `Cmd+A` | Select all | editor.rs:712 |
| `Arrow keys` | Navigate | editor.rs:716-719 |
| `Alt+Arrow` | Word navigation | editor.rs:722-723 |
| `Cmd+Arrow` | Line/document navigation | editor.rs:726-735 |
| `Tab` | Insert 4 spaces | editor.rs:741 |

### 7.4 Terminal Special Keys

| Key | Escape Sequence | Location |
|-----|-----------------|----------|
| `Enter` | `\r` | term_prompt.rs:520 |
| `Backspace` | `\x7f` | term_prompt.rs:521 |
| `Tab` | `\t` | term_prompt.rs:522 |
| `Arrow Up` | `\x1b[A` | term_prompt.rs:523 |
| `Arrow Down` | `\x1b[B` | term_prompt.rs:524 |
| `Arrow Right` | `\x1b[C` | term_prompt.rs:525 |
| `Arrow Left` | `\x1b[D` | term_prompt.rs:526 |
| `Home` | `\x1b[H` | term_prompt.rs:527 |
| `End` | `\x1b[F` | term_prompt.rs:528 |
| `PageUp` | `\x1b[5~` | term_prompt.rs:529 |
| `PageDown` | `\x1b[6~` | term_prompt.rs:530 |
| `Delete` | `\x1b[3~` | term_prompt.rs:531 |
| `F1-F12` | Various | term_prompt.rs:533-544 |

---

## 8. Event Coalescing & Scroll Stabilization

### 8.1 Scroll Stabilization Pattern

The app prevents scroll jitter by tracking the last scrolled-to index:

```rust
// main.rs:1252-1265
fn scroll_to_selected_if_needed(&mut self, _reason: &str) {
    let target = self.selected_index;
    
    // Check if we've already scrolled to this index
    if self.last_scrolled_index == Some(target) {
        return;
    }
    
    // Perform the scroll
    self.list_scroll_handle.scroll_to_item(target, ScrollStrategy::Nearest);
    self.last_scrolled_index = Some(target);
}
```

### 8.2 Section Header Skipping

Navigation skips section headers when moving through lists:

```rust
// main.rs:1194-1206
// Skip section headers when moving up
while new_index > 0 {
    if let Some(GroupedListItem::SectionHeader(_)) = grouped_items.get(new_index) {
        new_index -= 1;
    } else {
        break;
    }
}

// Make sure we didn't land on a section header
if let Some(GroupedListItem::SectionHeader(_)) = grouped_items.get(new_index) {
    return;  // Stay at current position
}
```

### 8.3 Scroll Activity Tracking

Scrollbar visibility is managed with timed fade-out:

```rust
// main.rs:1273-1290
fn trigger_scroll_activity(&mut self, cx: &mut Context<Self>) {
    self.is_scrolling = true;
    self.last_scroll_time = Some(std::time::Instant::now());
    
    // Schedule fade-out after 1000ms of inactivity
    cx.spawn(async move |this, cx| {
        Timer::after(Duration::from_millis(1000)).await;
        // Only hide if no new scroll activity occurred
        if let Some(last_time) = app.last_scroll_time {
            if last_time.elapsed() >= Duration::from_millis(1000) {
                app.is_scrolling = false;
                cx.notify();
            }
        }
    });
}
```

### 8.4 Batch Coalescing Logging

The logging module supports tracking coalesced scroll events:

```rust
// logging.rs:1001-1022
pub fn log_scroll_batch(batch_size: usize, coalesced_from: usize) {
    if coalesced_from > batch_size {
        // Log when events were coalesced
        info!(
            action = "batch_coalesce",
            batch_size = batch_size,
            coalesced_from = coalesced_from,
            "Coalesced {} scroll events to {}", coalesced_from, batch_size
        );
    }
}
```

---

## 9. Focus-Aware Color Transitions

### 9.1 Theme Focus-Aware Colors

The theme system supports separate color schemes for focused/unfocused states:

```rust
// theme.rs:structure
pub struct Theme {
    pub colors: ColorScheme,
    pub focus_aware: Option<FocusAwareColorScheme>,
    // ...
}

pub struct FocusAwareColorScheme {
    pub focused: Option<FocusedThemeSettings>,
    pub unfocused: Option<UnfocusedThemeSettings>,
}
```

### 9.2 Cursor Style for Focus

Cursor style changes based on focus state:

```rust
// theme.rs:504-517
pub fn get_cursor_style(&self, is_focused: bool) -> Option<CursorStyle> {
    if !is_focused {
        return None;  // No cursor when unfocused
    }
    
    if let Some(ref focus_aware) = self.focus_aware {
        if let Some(ref focused) = focus_aware.focused {
            return focused.cursor.clone();
        }
    }
    
    // Return default blinking cursor if focused
    Some(CursorStyle::default_focused())
}
```

### 9.3 Terminal Focus Handling

The terminal updates its theme based on focus:

```rust
// terminal/alacritty.rs:742-747
pub fn update_focus(&mut self, is_focused: bool) {
    self.theme.update_for_focus(is_focused);
    state.term.is_focused = is_focused;
    debug!(is_focused, "Terminal focus updated");
}
```

---

## 10. Cursor Blinking

### 10.1 Centralized Blink Timer

A single timer in the main app controls cursor blinking for all inputs:

```rust
// main.rs:886-903
// Start cursor blink timer - updates all inputs that track cursor visibility
cx.spawn(|app, mut cx| async move {
    loop {
        Timer::after(Duration::from_millis(530)).await;
        let _ = cx.update(|cx| {
            app.update(cx, |app, cx| {
                // Skip cursor blink when window is hidden or no input is focused
                if !WINDOW_VISIBLE.load(Ordering::SeqCst) || app.focused_input == FocusedInput::None {
                    return;
                }
                app.cursor_visible = !app.cursor_visible;
                
                // Also update ActionsDialog cursor if present
                if let Some(ref dialog) = app.actions_dialog {
                    dialog.update(cx, |d, _cx| {
                        d.set_cursor_visible(app.cursor_visible);
                    });
                }
                cx.notify();
            })
        });
    }
}).detach();
```

### 10.2 Cursor Rendering

Cursors are rendered conditionally based on `cursor_visible` and `focused_input`:

```rust
// main.rs:4849 - MainFilter cursor
.when(self.focused_input == FocusedInput::MainFilter && self.cursor_visible, |d| d.bg(rgb(text_primary)))

// main.rs:4989 - ActionsSearch cursor
.when(self.focused_input == FocusedInput::ActionsSearch && self.cursor_visible, |d| d.bg(rgb(accent_color)))

// main.rs:5296 - ArgPrompt cursor
.when(self.focused_input == FocusedInput::ArgPrompt && self.cursor_visible, |d| d.bg(rgb(text_primary)))
```

### 10.3 Cursor Style Configuration

The theme supports configurable cursor blink intervals:

```rust
// theme.rs:325-331
pub fn default_focused() -> Self {
    Self {
        color: 0xffffff,
        blink_interval_ms: 500,
    }
}
```

---

## 11. UniformList Scroll Handles

### 11.1 Multiple Scroll Handles

The app maintains separate scroll handles for different list contexts:

```rust
// main.rs:757-765
list_scroll_handle: UniformListScrollHandle,        // Main script list
arg_list_scroll_handle: UniformListScrollHandle,    // Arg prompt choices
clipboard_list_scroll_handle: UniformListScrollHandle,  // Clipboard history
window_list_scroll_handle: UniformListScrollHandle,     // Window switcher
design_gallery_scroll_handle: UniformListScrollHandle,  // Design gallery
```

### 11.2 Scroll Handle Usage

Each view uses its appropriate scroll handle with `uniform_list`:

```rust
uniform_list(
    "script-list",
    items.len(),
    cx.listener(|this, range, _window, _cx| {
        this.render_list_items(range)
    }),
)
.h_full()
.track_scroll(&self.list_scroll_handle)
```

---

## 12. Recommendations

### 12.1 Potential Improvements

1. **Visual Focus Indicators**: Consider adding subtle focus ring/outline styling for accessibility beyond background colors

2. **Keyboard Shortcut Reference**: Add a help dialog (`Cmd+?`) showing all available shortcuts

3. **Tab Navigation (Optional)**: For complex forms, consider adding tab navigation between form fields

4. **Focus Trap Escape**: Ensure modal dialogs have clear keyboard escape routes (already implemented with Escape key)

### 12.2 Accessibility Considerations

1. The current background-color-based selection indication may be insufficient for some users with color vision deficiencies

2. Consider adding ARIA-like semantic hints via element IDs (already partially implemented)

3. The lack of visible focus rings may make keyboard navigation less discoverable

### 12.3 Testing Recommendations

1. Add automated tests for all keyboard shortcuts
2. Test arrow key handling on different platforms (macOS, Windows, Linux)
3. Verify focus restoration after dialog close/cancel operations
4. Test screen reader compatibility (if applicable)

---

## Appendix: File Reference

| File | Key Keyboard/Focus Code |
|------|------------------------|
| `src/main.rs` | Main keyboard handler (4611-4760), focus management (3540-3558), cursor blink (886-903) |
| `src/actions.rs` | ActionsDialog keyboard (462-482), Focusable impl (448-451) |
| `src/prompts.rs` | ArgPrompt keyboard (247-267), DivPrompt keyboard (475-482), Focusable impls |
| `src/editor.rs` | Full editor keyboard (700-756), arrow key patterns (716-733) |
| `src/term_prompt.rs` | Terminal key translation (519-556), escape sequences |
| `src/theme.rs` | Focus-aware colors (484-501), cursor styles (504-517) |
| `src/logging.rs` | Scroll coalescing logs (1001-1022) |

---

*Audit completed: December 2024*
*Codebase version: Script Kit GPUI (Rust/GPUI framework)*
