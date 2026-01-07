# Expert Bundle #81: Filtering & Sorting

## Overview

Filtering and sorting help users navigate large collections of scripts, actions, and content. Script Kit provides multiple ways to refine results including category filters, tags, sort options, and saved filter presets. Good filtering UX is quick to apply and easy to clear.

## Architecture

### Filter System

```rust
// src/filters.rs
use gpui::*;

/// Filter configuration for a list
#[derive(Clone, Default)]
pub struct FilterConfig {
    pub categories: Vec<CategoryFilter>,
    pub tags: Vec<TagFilter>,
    pub sort_options: Vec<SortOption>,
    pub show_counts: bool,
    pub allow_multiple: bool,
}

#[derive(Clone)]
pub struct CategoryFilter {
    pub id: String,
    pub label: SharedString,
    pub icon: Option<SharedString>,
    pub count: usize,
}

#[derive(Clone)]
pub struct TagFilter {
    pub id: String,
    pub label: SharedString,
    pub color: Option<u32>,
}

#[derive(Clone)]
pub struct SortOption {
    pub id: String,
    pub label: SharedString,
    pub direction: SortDirection,
    pub is_default: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum SortDirection {
    #[default]
    Ascending,
    Descending,
}

/// Current filter state
#[derive(Clone, Default)]
pub struct FilterState {
    pub selected_categories: HashSet<String>,
    pub selected_tags: HashSet<String>,
    pub current_sort: Option<String>,
    pub sort_direction: SortDirection,
    pub search_query: String,
}

impl FilterState {
    pub fn is_active(&self) -> bool {
        !self.selected_categories.is_empty()
            || !self.selected_tags.is_empty()
            || self.current_sort.is_some()
            || !self.search_query.is_empty()
    }
    
    pub fn clear(&mut self) {
        self.selected_categories.clear();
        self.selected_tags.clear();
        self.current_sort = None;
        self.sort_direction = SortDirection::default();
        self.search_query.clear();
    }
    
    pub fn toggle_category(&mut self, id: &str) {
        if self.selected_categories.contains(id) {
            self.selected_categories.remove(id);
        } else {
            self.selected_categories.insert(id.to_string());
        }
    }
    
    pub fn toggle_tag(&mut self, id: &str) {
        if self.selected_tags.contains(id) {
            self.selected_tags.remove(id);
        } else {
            self.selected_tags.insert(id.to_string());
        }
    }
}
```

### Filter Bar Component

```rust
// src/components/filter_bar.rs
use crate::theme::Theme;
use gpui::*;

pub struct FilterBar {
    config: FilterConfig,
    state: FilterState,
    theme: Arc<Theme>,
    expanded: bool,
}

impl FilterBar {
    pub fn new(config: FilterConfig, theme: Arc<Theme>) -> Self {
        Self {
            config,
            state: FilterState::default(),
            theme,
            expanded: false,
        }
    }
}

impl Render for FilterBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        let has_active_filters = self.state.is_active();
        
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_2()
            // Main filter row
            .child(
                div()
                    .px_4()
                    .py_2()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    // Category chips
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_2()
                            .overflow_x_auto()
                            .children(self.config.categories.iter().map(|cat| {
                                let is_selected = self.state.selected_categories.contains(&cat.id);
                                let cat_id = cat.id.clone();
                                
                                div()
                                    .px_3()
                                    .py_1()
                                    .rounded_full()
                                    .cursor_pointer()
                                    .bg(rgb(if is_selected {
                                        colors.accent.primary
                                    } else {
                                        colors.ui.surface
                                    }))
                                    .text_color(rgb(if is_selected {
                                        colors.background.main
                                    } else {
                                        colors.text.secondary
                                    }))
                                    .hover(|s| s.bg(rgb(if is_selected {
                                        colors.accent.hover
                                    } else {
                                        colors.ui.hover
                                    })))
                                    .on_click(cx.listener(move |this, _, cx| {
                                        this.state.toggle_category(&cat_id);
                                        cx.emit(FilterChanged(this.state.clone()));
                                        cx.notify();
                                    }))
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap_1()
                                            .when_some(cat.icon.clone(), |el, icon| {
                                                el.child(Icon::new(icon).size(px(12.0)))
                                            })
                                            .child(
                                                div()
                                                    .text_size(px(12.0))
                                                    .child(cat.label.clone())
                                            )
                                            .when(self.config.show_counts, |el| {
                                                el.child(
                                                    div()
                                                        .text_size(px(10.0))
                                                        .opacity(0.7)
                                                        .child(format!("({})", cat.count))
                                                )
                                            })
                                    )
                            }))
                    )
                    // Spacer
                    .child(div().flex_1())
                    // Sort dropdown
                    .when(!self.config.sort_options.is_empty(), |el| {
                        el.child(self.render_sort_dropdown(cx))
                    })
                    // Clear filters
                    .when(has_active_filters, |el| {
                        el.child(
                            div()
                                .px_2()
                                .py_1()
                                .rounded_sm()
                                .cursor_pointer()
                                .text_size(px(12.0))
                                .text_color(rgb(colors.text.muted))
                                .hover(|s| s.text_color(rgb(colors.accent.primary)))
                                .on_click(cx.listener(|this, _, cx| {
                                    this.state.clear();
                                    cx.emit(FilterChanged(this.state.clone()));
                                    cx.notify();
                                }))
                                .child("Clear all")
                        )
                    })
                    // Expand/collapse
                    .when(!self.config.tags.is_empty(), |el| {
                        el.child(
                            div()
                                .cursor_pointer()
                                .p_1()
                                .rounded_sm()
                                .hover(|s| s.bg(rgb(colors.ui.hover)))
                                .on_click(cx.listener(|this, _, cx| {
                                    this.expanded = !this.expanded;
                                    cx.notify();
                                }))
                                .child(
                                    Icon::new(if self.expanded { "chevron-up" } else { "filter" })
                                        .size(px(14.0))
                                        .color(rgb(colors.text.muted))
                                )
                        )
                    })
            )
            // Expanded tags section
            .when(self.expanded && !self.config.tags.is_empty(), |el| {
                el.child(
                    div()
                        .px_4()
                        .py_2()
                        .border_t_1()
                        .border_color(rgb(colors.ui.border))
                        .child(self.render_tags_section(cx))
                )
            })
    }
}

impl FilterBar {
    fn render_sort_dropdown(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        let current_sort = self.state.current_sort.as_ref()
            .and_then(|id| self.config.sort_options.iter().find(|s| &s.id == id))
            .or_else(|| self.config.sort_options.iter().find(|s| s.is_default));
        
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .child(
                div()
                    .text_size(px(11.0))
                    .text_color(rgb(colors.text.muted))
                    .child("Sort:")
            )
            .child(
                div()
                    .px_2()
                    .py_1()
                    .rounded_sm()
                    .cursor_pointer()
                    .bg(rgb(colors.ui.surface))
                    .hover(|s| s.bg(rgb(colors.ui.hover)))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_1()
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .text_color(rgb(colors.text.secondary))
                                    .child(current_sort.map(|s| s.label.clone()).unwrap_or_default())
                            )
                            .child(
                                Icon::new(match self.state.sort_direction {
                                    SortDirection::Ascending => "arrow-up",
                                    SortDirection::Descending => "arrow-down",
                                })
                                .size(px(12.0))
                                .color(rgb(colors.text.muted))
                            )
                    )
            )
    }
    
    fn render_tags_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .text_size(px(11.0))
                    .text_color(rgb(colors.text.muted))
                    .child("Tags")
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap_2()
                    .children(self.config.tags.iter().map(|tag| {
                        let is_selected = self.state.selected_tags.contains(&tag.id);
                        let tag_id = tag.id.clone();
                        let tag_color = tag.color.unwrap_or(colors.ui.surface);
                        
                        div()
                            .px_2()
                            .py_1()
                            .rounded_sm()
                            .cursor_pointer()
                            .border_1()
                            .border_color(rgb(if is_selected { tag_color } else { colors.ui.border }))
                            .bg(rgb(if is_selected {
                                with_alpha(tag_color, 0.2)
                            } else {
                                colors.background.main
                            }))
                            .text_size(px(11.0))
                            .text_color(rgb(if is_selected {
                                tag_color
                            } else {
                                colors.text.secondary
                            }))
                            .hover(|s| s.bg(rgb(colors.ui.hover)))
                            .on_click(cx.listener(move |this, _, cx| {
                                this.state.toggle_tag(&tag_id);
                                cx.emit(FilterChanged(this.state.clone()));
                                cx.notify();
                            }))
                            .child(tag.label.clone())
                    }))
            )
    }
}

#[derive(Clone)]
pub struct FilterChanged(pub FilterState);
```

### Filter Application

```rust
// src/filters/apply.rs

/// Apply filters to a list of items
pub fn apply_filters<T: Filterable>(items: &[T], state: &FilterState) -> Vec<&T> {
    let mut filtered: Vec<&T> = items.iter()
        .filter(|item| {
            // Category filter
            if !state.selected_categories.is_empty() {
                if let Some(cat) = item.category() {
                    if !state.selected_categories.contains(cat) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            
            // Tag filter
            if !state.selected_tags.is_empty() {
                let item_tags: HashSet<&str> = item.tags().into_iter().collect();
                if !state.selected_tags.iter().any(|t| item_tags.contains(t.as_str())) {
                    return false;
                }
            }
            
            // Search filter
            if !state.search_query.is_empty() {
                if !item.matches_search(&state.search_query) {
                    return false;
                }
            }
            
            true
        })
        .collect();
    
    // Apply sorting
    if let Some(sort_id) = &state.current_sort {
        filtered.sort_by(|a, b| {
            let cmp = a.compare_by(sort_id, b);
            match state.sort_direction {
                SortDirection::Ascending => cmp,
                SortDirection::Descending => cmp.reverse(),
            }
        });
    }
    
    filtered
}

/// Trait for filterable items
pub trait Filterable {
    fn category(&self) -> Option<&str>;
    fn tags(&self) -> Vec<&str>;
    fn matches_search(&self, query: &str) -> bool;
    fn compare_by(&self, sort_key: &str, other: &Self) -> std::cmp::Ordering;
}

impl Filterable for Script {
    fn category(&self) -> Option<&str> {
        self.category.as_deref()
    }
    
    fn tags(&self) -> Vec<&str> {
        self.tags.iter().map(|s| s.as_str()).collect()
    }
    
    fn matches_search(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.name.to_lowercase().contains(&query_lower)
            || self.description.as_ref()
                .map(|d| d.to_lowercase().contains(&query_lower))
                .unwrap_or(false)
    }
    
    fn compare_by(&self, sort_key: &str, other: &Self) -> std::cmp::Ordering {
        match sort_key {
            "name" => self.name.cmp(&other.name),
            "recent" => other.last_run.cmp(&self.last_run),
            "frequent" => other.run_count.cmp(&self.run_count),
            "created" => other.created_at.cmp(&self.created_at),
            _ => std::cmp::Ordering::Equal,
        }
    }
}
```

## Usage Patterns

### Script List Filters

```rust
impl MainMenu {
    fn setup_filters(&mut self) {
        self.filter_config = FilterConfig {
            categories: vec![
                CategoryFilter {
                    id: "all".into(),
                    label: "All".into(),
                    icon: None,
                    count: self.all_scripts.len(),
                },
                CategoryFilter {
                    id: "scripts".into(),
                    label: "Scripts".into(),
                    icon: Some("terminal".into()),
                    count: self.count_by_category("scripts"),
                },
                CategoryFilter {
                    id: "tools".into(),
                    label: "Tools".into(),
                    icon: Some("wrench".into()),
                    count: self.count_by_category("tools"),
                },
                CategoryFilter {
                    id: "snippets".into(),
                    label: "Snippets".into(),
                    icon: Some("code".into()),
                    count: self.count_by_category("snippets"),
                },
            ],
            sort_options: vec![
                SortOption {
                    id: "recent".into(),
                    label: "Recent".into(),
                    direction: SortDirection::Descending,
                    is_default: true,
                },
                SortOption {
                    id: "name".into(),
                    label: "Name".into(),
                    direction: SortDirection::Ascending,
                    is_default: false,
                },
                SortOption {
                    id: "frequent".into(),
                    label: "Most Used".into(),
                    direction: SortDirection::Descending,
                    is_default: false,
                },
            ],
            show_counts: true,
            allow_multiple: false,
            tags: vec![], // Tags loaded dynamically
        };
    }
    
    fn handle_filter_changed(&mut self, state: FilterState, cx: &mut WindowContext) {
        self.filter_state = state;
        self.update_filtered_list(cx);
        cx.notify();
    }
    
    fn update_filtered_list(&mut self) {
        self.filtered_scripts = apply_filters(&self.all_scripts, &self.filter_state)
            .into_iter()
            .cloned()
            .collect();
    }
}
```

### Active Filter Badges

```rust
// Show active filters as dismissible badges
impl FilterBar {
    fn render_active_filters(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        if !self.state.is_active() {
            return div().into_any_element();
        }
        
        div()
            .px_4()
            .py_2()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            // Active category badges
            .children(self.state.selected_categories.iter().map(|cat_id| {
                let cat = self.config.categories.iter().find(|c| &c.id == cat_id);
                let cat_id = cat_id.clone();
                
                div()
                    .px_2()
                    .py_1()
                    .rounded_full()
                    .bg(rgb(colors.accent.primary))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(rgb(colors.background.main))
                            .child(cat.map(|c| c.label.clone()).unwrap_or_default())
                    )
                    .child(
                        div()
                            .cursor_pointer()
                            .on_click(cx.listener(move |this, _, cx| {
                                this.state.selected_categories.remove(&cat_id);
                                cx.emit(FilterChanged(this.state.clone()));
                                cx.notify();
                            }))
                            .child(
                                Icon::new("x")
                                    .size(px(10.0))
                                    .color(rgb(colors.background.main))
                            )
                    )
            }))
            // Active tag badges
            .children(self.state.selected_tags.iter().map(|tag_id| {
                let tag = self.config.tags.iter().find(|t| &t.id == tag_id);
                let tag_id = tag_id.clone();
                let tag_color = tag.and_then(|t| t.color).unwrap_or(colors.text.secondary);
                
                div()
                    .px_2()
                    .py_1()
                    .rounded_sm()
                    .bg(rgb(with_alpha(tag_color, 0.2)))
                    .border_1()
                    .border_color(rgb(tag_color))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(rgb(tag_color))
                            .child(tag.map(|t| t.label.clone()).unwrap_or_default())
                    )
                    .child(
                        div()
                            .cursor_pointer()
                            .on_click(cx.listener(move |this, _, cx| {
                                this.state.selected_tags.remove(&tag_id);
                                cx.emit(FilterChanged(this.state.clone()));
                                cx.notify();
                            }))
                            .child(
                                Icon::new("x")
                                    .size(px(10.0))
                                    .color(rgb(tag_color))
                            )
                    )
            }))
            .into_any_element()
    }
}
```

### Quick Filters

```rust
// Keyboard shortcuts for common filters
impl MainMenu {
    fn handle_key(&mut self, key: &str, cx: &mut WindowContext) {
        match key {
            // Quick category filters
            "1" if self.modifiers.cmd => {
                self.filter_state.clear();
                self.filter_state.selected_categories.insert("scripts".into());
                self.update_filtered_list(cx);
            }
            "2" if self.modifiers.cmd => {
                self.filter_state.clear();
                self.filter_state.selected_categories.insert("tools".into());
                self.update_filtered_list(cx);
            }
            "3" if self.modifiers.cmd => {
                self.filter_state.clear();
                self.filter_state.selected_categories.insert("snippets".into());
                self.update_filtered_list(cx);
            }
            // Clear filters
            "backspace" if self.filter_state.search_query.is_empty() && self.filter_state.is_active() => {
                self.filter_state.clear();
                self.update_filtered_list(cx);
            }
            _ => {}
        }
        cx.notify();
    }
}
```

## Testing

### Filter Test Script

```typescript
// tests/smoke/test-filtering.ts
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Test 1: Filter bar with categories
await div(`
  <div class="px-4 py-2 flex items-center gap-3 border-b border-zinc-700">
    <div class="flex gap-2 overflow-x-auto">
      <button class="px-3 py-1 rounded-full bg-amber-500 text-black text-xs">All (42)</button>
      <button class="px-3 py-1 rounded-full bg-zinc-700 text-zinc-300 text-xs hover:bg-zinc-600">Scripts (24)</button>
      <button class="px-3 py-1 rounded-full bg-zinc-700 text-zinc-300 text-xs hover:bg-zinc-600">Tools (12)</button>
      <button class="px-3 py-1 rounded-full bg-zinc-700 text-zinc-300 text-xs hover:bg-zinc-600">Snippets (6)</button>
    </div>
    <div class="flex-1"></div>
    <div class="flex items-center gap-2 text-xs">
      <span class="text-zinc-500">Sort:</span>
      <button class="px-2 py-1 rounded bg-zinc-700 text-zinc-300 flex items-center gap-1">
        Recent <span class="text-zinc-500">↓</span>
      </button>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot1 = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
writeFileSync(join(dir, 'filter-bar-categories.png'), Buffer.from(shot1.data, 'base64'));

// Test 2: Active filter badges
await div(`
  <div class="flex flex-col">
    <div class="px-4 py-2 flex items-center gap-3 border-b border-zinc-700">
      <div class="flex gap-2">
        <button class="px-3 py-1 rounded-full bg-zinc-700 text-zinc-300 text-xs">All</button>
        <button class="px-3 py-1 rounded-full bg-amber-500 text-black text-xs">Scripts</button>
        <button class="px-3 py-1 rounded-full bg-zinc-700 text-zinc-300 text-xs">Tools</button>
      </div>
      <div class="flex-1"></div>
      <span class="text-xs text-amber-400 cursor-pointer">Clear all</span>
    </div>
    <div class="px-4 py-2 flex items-center gap-2 bg-zinc-800/50">
      <div class="px-2 py-1 rounded-full bg-amber-500 flex items-center gap-1">
        <span class="text-xs text-black">Scripts</span>
        <span class="text-black cursor-pointer">×</span>
      </div>
      <div class="px-2 py-1 rounded-sm bg-blue-500/20 border border-blue-500 flex items-center gap-1">
        <span class="text-xs text-blue-400">automation</span>
        <span class="text-blue-400 cursor-pointer">×</span>
      </div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot2 = await captureScreenshot();
writeFileSync(join(dir, 'filter-active-badges.png'), Buffer.from(shot2.data, 'base64'));

// Test 3: Expanded tags section
await div(`
  <div class="flex flex-col">
    <div class="px-4 py-2 flex items-center gap-3 border-b border-zinc-700">
      <div class="flex gap-2">
        <button class="px-3 py-1 rounded-full bg-amber-500 text-black text-xs">All</button>
        <button class="px-3 py-1 rounded-full bg-zinc-700 text-zinc-300 text-xs">Scripts</button>
      </div>
      <div class="flex-1"></div>
      <button class="p-1 rounded hover:bg-zinc-700">
        <span class="text-zinc-500 text-sm">▲</span>
      </button>
    </div>
    <div class="px-4 py-2 border-t border-zinc-700/50 flex flex-col gap-2">
      <span class="text-xs text-zinc-500">Tags</span>
      <div class="flex flex-wrap gap-2">
        <span class="px-2 py-1 rounded-sm border border-blue-500 bg-blue-500/20 text-xs text-blue-400">automation</span>
        <span class="px-2 py-1 rounded-sm border border-zinc-600 text-xs text-zinc-400">clipboard</span>
        <span class="px-2 py-1 rounded-sm border border-zinc-600 text-xs text-zinc-400">window</span>
        <span class="px-2 py-1 rounded-sm border border-zinc-600 text-xs text-zinc-400">git</span>
        <span class="px-2 py-1 rounded-sm border border-green-500 bg-green-500/20 text-xs text-green-400">productivity</span>
      </div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot3 = await captureScreenshot();
writeFileSync(join(dir, 'filter-tags-expanded.png'), Buffer.from(shot3.data, 'base64'));

// Test 4: Sort dropdown
await div(`
  <div class="p-4">
    <div class="relative inline-block">
      <button class="px-2 py-1 rounded bg-zinc-700 text-zinc-300 text-xs flex items-center gap-1">
        Recent <span class="text-zinc-500">↓</span>
      </button>
      <div class="absolute top-full mt-1 w-32 bg-zinc-800 rounded-md shadow-lg border border-zinc-700 py-1">
        <div class="px-3 py-1.5 text-xs text-white bg-zinc-700 flex items-center justify-between">
          Recent <span>✓</span>
        </div>
        <div class="px-3 py-1.5 text-xs text-zinc-400 hover:bg-zinc-700 cursor-pointer">
          Name
        </div>
        <div class="px-3 py-1.5 text-xs text-zinc-400 hover:bg-zinc-700 cursor-pointer">
          Most Used
        </div>
        <div class="px-3 py-1.5 text-xs text-zinc-400 hover:bg-zinc-700 cursor-pointer">
          Created
        </div>
      </div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot4 = await captureScreenshot();
writeFileSync(join(dir, 'filter-sort-dropdown.png'), Buffer.from(shot4.data, 'base64'));

console.error('[FILTERING] Test screenshots saved');
process.exit(0);
```

## Related Bundles

- Bundle #80: Search UX Patterns - Search complements filtering
- Bundle #71: Command Palette Patterns - Filtered command lists
- Bundle #64: List Virtualization - Rendering filtered results
