# Selection Feedback - Expert Bundle

## Overview

Clear visual feedback for selection states is critical for keyboard-driven interfaces. Users must always know what's selected and what will happen when they press Enter.

## Selection States

### State Hierarchy

```
Normal → Hovered → Focused → Selected → Selected + Focused
```

### Visual Treatments

```rust
#[derive(Clone, Copy)]
pub struct SelectionColors {
    pub normal_bg: u32,
    pub hover_bg: u32,
    pub selected_bg: u32,
    pub selected_focused_bg: u32,
    pub selected_border: u32,
}

impl Theme {
    pub fn selection_colors(&self) -> SelectionColors {
        SelectionColors {
            normal_bg: 0x00000000,        // Transparent
            hover_bg: 0x3F3F4620,          // 12% white
            selected_bg: 0x3B82F620,       // 12% blue
            selected_focused_bg: 0x3B82F640, // 25% blue
            selected_border: 0x3B82F6,     // Solid blue
        }
    }
}
```

## List Item Selection

### Basic Pattern

```rust
fn render_list_item(
    &self,
    item: &Item,
    index: usize,
    cx: &mut Context<Self>,
) -> impl IntoElement {
    let is_selected = index == self.selected_index;
    let is_window_focused = self.focus_handle.is_focused(cx.window());
    let colors = self.theme.selection_colors();
    
    div()
        .id(ElementId::from(index))
        .h(px(52.0))
        .w_full()
        .px_3()
        .flex()
        .items_center()
        .cursor_pointer()
        // Background based on state
        .when(is_selected && is_window_focused, |d| {
            d.bg(rgb(colors.selected_focused_bg))
        })
        .when(is_selected && !is_window_focused, |d| {
            d.bg(rgb(colors.selected_bg))
        })
        .when(!is_selected, |d| {
            d.hover(|h| h.bg(rgb(colors.hover_bg)))
        })
        // Selection indicator (left border)
        .when(is_selected, |d| {
            d.border_l_2()
             .border_color(rgb(colors.selected_border))
        })
        .child(/* content */)
}
```

### Selection Indicator Variants

```rust
// Variant 1: Left border accent
.when(is_selected, |d| {
    d.border_l_2()
     .border_color(rgb(0x3B82F6))
})

// Variant 2: Background highlight
.when(is_selected, |d| {
    d.bg(rgb(0x3B82F6).opacity(0.2))
})

// Variant 3: Full border
.when(is_selected, |d| {
    d.border_1()
     .border_color(rgb(0x3B82F6))
     .rounded_md()
})

// Variant 4: Checkmark icon
.when(is_selected, |d| {
    d.child(
        Icon::new(IconName::Check)
            .size_4()
            .text_color(rgb(0x3B82F6))
    )
})
```

## Multi-Select Pattern

```rust
pub struct MultiSelectList {
    items: Vec<Item>,
    selected_indices: HashSet<usize>,
    focused_index: usize,
}

fn render_multi_select_item(
    &self,
    item: &Item,
    index: usize,
) -> impl IntoElement {
    let is_selected = self.selected_indices.contains(&index);
    let is_focused = index == self.focused_index;
    
    div()
        .h(px(44.0))
        .px_3()
        .flex()
        .items_center()
        .gap_3()
        // Focus ring (keyboard navigation)
        .when(is_focused, |d| {
            d.outline_2()
             .outline_color(rgb(0x3B82F6))
             .outline_offset(px(-2.0))
        })
        // Checkbox
        .child(
            div()
                .w(px(18.0))
                .h(px(18.0))
                .rounded_sm()
                .border_1()
                .flex()
                .items_center()
                .justify_center()
                .when(is_selected, |d| {
                    d.bg(rgb(0x3B82F6))
                     .border_color(rgb(0x3B82F6))
                     .child(
                        Icon::new(IconName::Check)
                            .size_3()
                            .text_color(rgb(0xFFFFFF))
                     )
                })
                .when(!is_selected, |d| {
                    d.bg(rgb(0x27272A))
                     .border_color(rgb(0x52525B))
                })
        )
        // Item content
        .child(
            div()
                .flex_1()
                .text_sm()
                .child(&item.name)
        )
}
```

## Selection Count Badge

```rust
fn render_selection_status(&self) -> impl IntoElement {
    let count = self.selected_indices.len();
    
    if count == 0 {
        return div().into_any();
    }
    
    div()
        .flex()
        .items_center()
        .gap_2()
        .px_3()
        .py_2()
        .bg(rgb(0x3B82F6).opacity(0.1))
        .rounded_md()
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(0x3B82F6))
                .child(format!("{} selected", count))
        )
        .child(
            Button::new("clear-selection")
                .label("Clear")
                .ghost()
                .small()
                .on_click(cx.listener(|this, _, _, cx| {
                    this.clear_selection(cx);
                }))
        )
        .into_any()
}
```

## Animated Selection

### Smooth Transitions

```rust
fn render_list_item_animated(
    &self,
    item: &Item,
    index: usize,
) -> impl IntoElement {
    let is_selected = index == self.selected_index;
    
    div()
        .h(px(52.0))
        .w_full()
        .relative()
        // Animated selection background
        .child(
            div()
                .absolute()
                .inset_0()
                .rounded_md()
                .bg(rgb(0x3B82F6))
                // Animate opacity
                .opacity(if is_selected { 0.15 } else { 0.0 })
                .transition_opacity()
                .duration_150()
        )
        // Content
        .child(
            div()
                .relative()
                .px_3()
                .flex()
                .items_center()
                .h_full()
                .child(&item.name)
        )
}
```

### Selection Indicator Animation

```rust
// Sliding selection indicator
fn render_selection_indicator(&self) -> impl IntoElement {
    let y_offset = self.selected_index as f32 * ITEM_HEIGHT;
    
    div()
        .absolute()
        .left_0()
        .w(px(3.0))
        .h(px(ITEM_HEIGHT))
        .bg(rgb(0x3B82F6))
        .rounded_r_full()
        // Animated position
        .top(px(y_offset))
        .transition_top()
        .duration_150()
        .ease_out()
}
```

## Keyboard Selection Feedback

### Visual Key Press Feedback

```rust
fn render_with_key_feedback(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
    let show_key_hint = self.last_key_time.elapsed() < Duration::from_millis(500);
    
    div()
        .relative()
        .child(self.render_list(cx))
        // Floating key indicator
        .when(show_key_hint, |d| {
            d.child(
                div()
                    .absolute()
                    .bottom_4()
                    .right_4()
                    .px_3()
                    .py_2()
                    .rounded_lg()
                    .bg(rgb(0x000000).opacity(0.8))
                    .text_sm()
                    .text_color(rgb(0xFFFFFF))
                    .child(&self.last_key_name)
            )
        })
}
```

## Selection Sound Feedback

```rust
#[cfg(target_os = "macos")]
fn play_selection_sound() {
    use cocoa::appkit::NSSound;
    use cocoa::base::nil;
    use cocoa::foundation::NSString;
    
    unsafe {
        let sound_name = NSString::alloc(nil).init_str("Pop");
        let sound = NSSound::soundNamed_(nil, sound_name);
        if sound != nil {
            let _: () = msg_send![sound, play];
        }
    }
}

impl List {
    fn move_selection(&mut self, delta: i32, cx: &mut Context<Self>) {
        let new_index = (self.selected_index as i32 + delta)
            .max(0)
            .min(self.items.len() as i32 - 1) as usize;
        
        if new_index != self.selected_index {
            self.selected_index = new_index;
            play_selection_sound();
            cx.notify();
        }
    }
}
```

## Scroll to Selection

```rust
impl List {
    fn ensure_selected_visible(&self) {
        self.scroll_handle.scroll_to_item(
            self.selected_index,
            ScrollStrategy::Nearest,
        );
    }

    fn move_selection_down(&mut self, cx: &mut Context<Self>) {
        if self.selected_index < self.items.len() - 1 {
            self.selected_index += 1;
            self.ensure_selected_visible();
            cx.notify();
        }
    }
}
```

## Focus-Aware Selection

```rust
impl Render for List {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Check focus state
        let is_focused = self.focus_handle.is_focused(window);
        
        // Update cached state for styling
        if self.was_focused != is_focused {
            self.was_focused = is_focused;
            cx.notify();
        }
        
        let colors = if is_focused {
            self.theme.selection_colors_focused()
        } else {
            self.theme.selection_colors_unfocused()
        };
        
        // Render with appropriate colors
        self.render_list_with_colors(colors, cx)
    }
}
```

## Best Practices

1. **Always show selection** - Never leave users guessing
2. **Differentiate focused/unfocused** - Dimmer selection when unfocused
3. **Use multiple cues** - Background + border + icon
4. **Animate transitions** - 150ms ease-out is snappy
5. **Keep selection visible** - Auto-scroll on keyboard nav
6. **Support multi-select** - Space to toggle, Cmd+A to select all
7. **Show selection count** - Badge when multi-selecting

## State Reference

| State | Background | Border | Other |
|-------|------------|--------|-------|
| Normal | Transparent | None | - |
| Hover | 12% white | None | Cursor pointer |
| Selected | 12% blue | 2px left blue | - |
| Selected + Focused | 25% blue | 2px left blue | Brighter |
| Multi-selected | Checkbox filled | - | Check icon |
