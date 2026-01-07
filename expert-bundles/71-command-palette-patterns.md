# Command Palette Patterns - Expert Bundle

## Overview

The command palette is the core UX of Script Kit - a fast, keyboard-driven interface for discovering and executing commands.

## Palette Architecture

### Core Components

```rust
pub struct CommandPalette {
    // Input state
    query: String,
    input_focus: FocusHandle,
    
    // List state
    all_items: Vec<Arc<CommandItem>>,
    filtered_items: Vec<Arc<CommandItem>>,
    selected_index: usize,
    list_scroll_handle: UniformListScrollHandle,
    
    // Visual state
    is_loading: bool,
    placeholder: String,
    hint: Option<String>,
    
    // Actions
    actions: Vec<Action>,
    show_actions: bool,
}

pub struct CommandItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub shortcut: Option<String>,
    pub keywords: Vec<String>,
    pub category: Option<String>,
    pub frecency_score: f64,
}
```

### Item Rendering

```rust
const ITEM_HEIGHT: f32 = 52.0;

fn render_command_item(
    &self,
    item: &CommandItem,
    index: usize,
    is_selected: bool,
    theme: &Theme,
) -> impl IntoElement {
    let colors = theme.list_item_colors();
    
    div()
        .id(ElementId::from(index))
        .h(px(ITEM_HEIGHT))
        .w_full()
        .px_3()
        .flex()
        .items_center()
        .gap_3()
        .cursor_pointer()
        .when(is_selected, |d| d.bg(rgb(colors.selected_bg)))
        .hover(|d| d.bg(rgb(colors.hover_bg)))
        // Icon
        .child(
            div()
                .w(px(24.0))
                .h(px(24.0))
                .flex()
                .items_center()
                .justify_center()
                .child(self.render_icon(&item.icon))
        )
        // Text content
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .justify_center()
                .overflow_hidden()
                // Name
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(colors.text))
                        .truncate()
                        .child(&item.name)
                )
                // Description
                .when_some(item.description.as_ref(), |d, desc| {
                    d.child(
                        div()
                            .text_xs()
                            .text_color(rgb(colors.secondary_text))
                            .truncate()
                            .child(desc.clone())
                    )
                })
        )
        // Shortcut badge
        .when_some(item.shortcut.as_ref(), |d, shortcut| {
            d.child(
                div()
                    .px_2()
                    .py_0p5()
                    .rounded_md()
                    .bg(rgb(0x3F3F46))
                    .text_xs()
                    .text_color(rgb(0xA1A1AA))
                    .child(shortcut.clone())
            )
        })
}
```

## Fuzzy Search

### Multi-Field Matching

```rust
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};

pub struct CommandFilter {
    matcher: SkimMatcherV2,
}

impl CommandFilter {
    pub fn new() -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
        }
    }

    pub fn filter(&self, items: &[Arc<CommandItem>], query: &str) -> Vec<(Arc<CommandItem>, i64)> {
        if query.is_empty() {
            return items.iter()
                .map(|item| (Arc::clone(item), 0))
                .collect();
        }

        let query_lower = query.to_lowercase();
        
        items.iter()
            .filter_map(|item| {
                // Match against name (highest weight)
                let name_score = self.matcher
                    .fuzzy_match(&item.name.to_lowercase(), &query_lower)
                    .unwrap_or(0) * 3;
                
                // Match against keywords
                let keyword_score = item.keywords.iter()
                    .filter_map(|kw| self.matcher.fuzzy_match(&kw.to_lowercase(), &query_lower))
                    .max()
                    .unwrap_or(0) * 2;
                
                // Match against description
                let desc_score = item.description.as_ref()
                    .and_then(|d| self.matcher.fuzzy_match(&d.to_lowercase(), &query_lower))
                    .unwrap_or(0);
                
                let total_score = name_score + keyword_score + desc_score;
                
                if total_score > 0 {
                    Some((Arc::clone(item), total_score))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn filter_and_sort(&self, items: &[Arc<CommandItem>], query: &str) -> Vec<Arc<CommandItem>> {
        let mut results = self.filter(items, query);
        
        // Sort by score (descending), then by frecency
        results.sort_by(|a, b| {
            b.1.cmp(&a.1)
                .then_with(|| {
                    b.0.frecency_score.partial_cmp(&a.0.frecency_score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });
        
        results.into_iter().map(|(item, _)| item).collect()
    }
}
```

## Frecency Scoring

```rust
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrecencyData {
    pub access_count: u32,
    pub last_access: u64,  // Unix timestamp
}

impl FrecencyData {
    pub fn score(&self) -> f64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let age_hours = (now - self.last_access) as f64 / 3600.0;
        
        // Decay factor: halves every 24 hours
        let recency = 0.5_f64.powf(age_hours / 24.0);
        
        // Frequency component: log scale
        let frequency = (self.access_count as f64 + 1.0).ln();
        
        recency * frequency * 100.0
    }
    
    pub fn record_access(&mut self) {
        self.access_count += 1;
        self.last_access = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}
```

## Keyboard Navigation

```rust
impl CommandPalette {
    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.key.as_ref().map(|k| k.as_str()).unwrap_or("");
        
        match key {
            // Navigation
            "up" | "arrowup" => self.move_up(cx),
            "down" | "arrowdown" => self.move_down(cx),
            "pageup" | "PageUp" => self.page_up(cx),
            "pagedown" | "PageDown" => self.page_down(cx),
            "home" | "Home" if event.modifiers.command => self.jump_to_first(cx),
            "end" | "End" if event.modifiers.command => self.jump_to_last(cx),
            
            // Selection
            "enter" | "Enter" => self.submit(cx),
            "tab" | "Tab" => {
                if event.modifiers.shift {
                    self.show_actions(cx);
                } else {
                    self.autocomplete(cx);
                }
            }
            
            // Actions
            "escape" | "Escape" => {
                if self.show_actions {
                    self.hide_actions(cx);
                } else {
                    self.cancel(cx);
                }
            }
            
            // Quick actions via modifiers
            _ if event.modifiers.command => {
                if let Some(action) = self.get_cmd_action(key) {
                    self.execute_action(action, cx);
                }
            }
            
            _ => {}
        }
    }
}
```

## Quick Actions (Cmd+Key)

```rust
impl CommandPalette {
    fn get_cmd_action(&self, key: &str) -> Option<&Action> {
        match key {
            "1" => self.actions.get(0),
            "2" => self.actions.get(1),
            "3" => self.actions.get(2),
            "c" => self.find_action("copy"),
            "o" => self.find_action("open"),
            "e" => self.find_action("edit"),
            _ => None,
        }
    }
    
    fn find_action(&self, id: &str) -> Option<&Action> {
        self.actions.iter().find(|a| a.id == id)
    }
}
```

## Placeholder & Hints

```rust
fn render_input(&self, cx: &mut Context<Self>) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .h(px(44.0))
        .px_3()
        .gap_2()
        .border_b_1()
        .border_color(rgb(0x3F3F46))
        // Search icon
        .child(
            Icon::new(IconName::Search)
                .size_4()
                .text_color(rgb(0x71717A))
        )
        // Input
        .child(
            div()
                .flex_1()
                .child(
                    input()
                        .placeholder(&self.placeholder)
                        .value(&self.query)
                        .text_sm()
                        .on_change(cx.listener(Self::on_query_change))
                )
        )
        // Hint
        .when_some(self.hint.as_ref(), |d, hint| {
            d.child(
                div()
                    .text_xs()
                    .text_color(rgb(0x71717A))
                    .child(hint.clone())
            )
        })
        // Loading indicator
        .when(self.is_loading, |d| {
            d.child(
                Icon::new(IconName::Loader)
                    .size_4()
                    .animate_spin()
            )
        })
}
```

## Category Headers

```rust
fn render_with_categories(&self) -> impl IntoElement {
    let mut current_category: Option<&str> = None;
    let mut elements: Vec<AnyElement> = Vec::new();
    
    for (index, item) in self.filtered_items.iter().enumerate() {
        let item_category = item.category.as_deref();
        
        // Insert category header if changed
        if item_category != current_category {
            if let Some(cat) = item_category {
                elements.push(
                    div()
                        .px_3()
                        .py_2()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(0x71717A))
                        .child(cat.to_uppercase())
                        .into_any()
                );
            }
            current_category = item_category;
        }
        
        elements.push(
            self.render_command_item(item, index, index == self.selected_index, &self.theme)
                .into_any()
        );
    }
    
    div().flex().flex_col().children(elements)
}
```

## Empty & Loading States

```rust
fn render_list_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
    if self.is_loading {
        self.render_loading_state()
    } else if self.filtered_items.is_empty() {
        self.render_empty_state()
    } else {
        self.render_items(cx)
    }
}

fn render_empty_state(&self) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .py_12()
        .gap_3()
        .child(
            Icon::new(IconName::Search)
                .size_12()
                .text_color(rgb(0x52525B))
        )
        .child(
            div()
                .text_sm()
                .text_color(rgb(0x71717A))
                .child(if self.query.is_empty() {
                    "No commands available"
                } else {
                    "No matching commands"
                })
        )
        .when(!self.query.is_empty(), |d| {
            d.child(
                div()
                    .text_xs()
                    .text_color(rgb(0x52525B))
                    .child(format!("Try a different search term"))
            )
        })
}
```

## Best Practices

1. **Fixed 52px item height** - Required for virtualization
2. **Fuzzy match multiple fields** - Name, keywords, description
3. **Frecency scoring** - Balance recency and frequency
4. **Keyboard-first** - All actions accessible via keyboard
5. **Quick actions with Cmd+N** - First 3 items via Cmd+1/2/3
6. **Show loading states** - Never leave users guessing
7. **Meaningful empty states** - Help users understand why

## Summary

| Feature | Pattern |
|---------|---------|
| Search | Multi-field fuzzy matching |
| Ranking | Score + frecency |
| Navigation | Arrow keys + Page Up/Down |
| Actions | Tab for actions, Cmd+N shortcuts |
| Feedback | Loading, empty, error states |
