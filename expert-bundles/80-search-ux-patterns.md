# Expert Bundle #80: Search UX Patterns

## Overview

Search is fundamental to Script Kit's command palette experience. Users type to filter scripts, find actions, and navigate content. Good search UX includes instant feedback, smart matching, highlighted results, and helpful empty states.

## Architecture

### Search System

```rust
// src/search.rs
use gpui::*;
use std::time::Duration;

/// Search configuration options
#[derive(Clone)]
pub struct SearchConfig {
    /// Debounce delay before searching
    pub debounce: Duration,
    /// Minimum characters to trigger search
    pub min_chars: usize,
    /// Maximum results to display
    pub max_results: usize,
    /// Whether to show recent searches
    pub show_recent: bool,
    /// Whether to highlight matches
    pub highlight_matches: bool,
    /// Fuzzy matching threshold (0.0-1.0)
    pub fuzzy_threshold: f32,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            debounce: Duration::from_millis(100),
            min_chars: 0,
            max_results: 50,
            show_recent: true,
            highlight_matches: true,
            fuzzy_threshold: 0.6,
        }
    }
}

/// Search state management
pub struct SearchState {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub selected_index: usize,
    pub is_searching: bool,
    pub search_time: Option<Duration>,
    pub total_count: usize,
}

#[derive(Clone)]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub icon: Option<String>,
    pub score: f32,
    pub match_ranges: Vec<std::ops::Range<usize>>,
    pub category: Option<String>,
}
```

### Fuzzy Matching

```rust
// src/search/fuzzy.rs

/// Fuzzy string matching with scoring
pub struct FuzzyMatcher {
    case_sensitive: bool,
    require_prefix: bool,
}

impl FuzzyMatcher {
    pub fn new() -> Self {
        Self {
            case_sensitive: false,
            require_prefix: false,
        }
    }
    
    /// Match query against text, returning score and match positions
    pub fn match_with_positions(&self, query: &str, text: &str) -> Option<FuzzyMatch> {
        let query_chars: Vec<char> = if self.case_sensitive {
            query.chars().collect()
        } else {
            query.to_lowercase().chars().collect()
        };
        
        let text_chars: Vec<char> = if self.case_sensitive {
            text.chars().collect()
        } else {
            text.to_lowercase().chars().collect()
        };
        
        if query_chars.is_empty() {
            return Some(FuzzyMatch {
                score: 1.0,
                positions: vec![],
            });
        }
        
        let mut positions = Vec::new();
        let mut query_idx = 0;
        let mut prev_match_idx: Option<usize> = None;
        let mut score = 0.0;
        
        for (text_idx, &text_char) in text_chars.iter().enumerate() {
            if query_idx < query_chars.len() && text_char == query_chars[query_idx] {
                positions.push(text_idx);
                
                // Score bonuses
                if text_idx == 0 {
                    score += 15.0; // Start of string
                } else if text.chars().nth(text_idx - 1).map(|c| c == ' ' || c == '_' || c == '-').unwrap_or(false) {
                    score += 10.0; // Start of word
                } else if prev_match_idx == Some(text_idx - 1) {
                    score += 5.0; // Consecutive match
                }
                
                score += 1.0; // Base match
                prev_match_idx = Some(text_idx);
                query_idx += 1;
            }
        }
        
        if query_idx == query_chars.len() {
            // Normalize score
            let max_score = query_chars.len() as f32 * 21.0; // Max possible
            let normalized = score / max_score;
            
            Some(FuzzyMatch {
                score: normalized,
                positions,
            })
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct FuzzyMatch {
    pub score: f32,
    pub positions: Vec<usize>,
}
```

### Search Input Component

```rust
// src/components/search_input.rs
use crate::theme::Theme;
use gpui::*;

pub struct SearchInput {
    query: String,
    placeholder: SharedString,
    theme: Arc<Theme>,
    focus_handle: FocusHandle,
    is_loading: bool,
    debounce_timer: Option<Task<()>>,
}

impl SearchInput {
    pub fn new(placeholder: impl Into<SharedString>, theme: Arc<Theme>, cx: &mut WindowContext) -> Self {
        Self {
            query: String::new(),
            placeholder: placeholder.into(),
            theme,
            focus_handle: cx.focus_handle(),
            is_loading: false,
            debounce_timer: None,
        }
    }
    
    pub fn set_query(&mut self, query: &str, cx: &mut WindowContext) {
        self.query = query.to_string();
        cx.notify();
    }
    
    pub fn clear(&mut self, cx: &mut WindowContext) {
        self.query.clear();
        cx.emit(SearchQueryChanged("".to_string()));
        cx.notify();
    }
}

impl Render for SearchInput {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        let has_query = !self.query.is_empty();
        
        div()
            .w_full()
            .px_4()
            .py_3()
            .bg(rgb(colors.ui.input))
            .flex()
            .flex_row()
            .items_center()
            .gap_3()
            // Search icon
            .child(
                div()
                    .flex_shrink_0()
                    .child(
                        Icon::new("search")
                            .size(px(16.0))
                            .color(rgb(if has_query {
                                colors.text.primary
                            } else {
                                colors.text.muted
                            }))
                    )
            )
            // Input field
            .child(
                div()
                    .flex_1()
                    .child(
                        input()
                            .w_full()
                            .bg_transparent()
                            .border_none()
                            .text_size(px(14.0))
                            .text_color(rgb(colors.text.primary))
                            .placeholder(&self.placeholder)
                            .placeholder_color(rgb(colors.text.muted))
                            .value(&self.query)
                            .focus(&self.focus_handle)
                            .on_input(cx.listener(|this, value: &str, cx| {
                                this.query = value.to_string();
                                
                                // Debounce search
                                this.debounce_timer = Some(cx.spawn(|this, mut cx| async move {
                                    Timer::after(Duration::from_millis(100)).await;
                                    this.update(&mut cx, |this, cx| {
                                        cx.emit(SearchQueryChanged(this.query.clone()));
                                    }).ok();
                                }));
                                
                                cx.notify();
                            }))
                    )
            )
            // Loading spinner or clear button
            .child(
                div()
                    .flex_shrink_0()
                    .w(px(20.0))
                    .child(
                        if self.is_loading {
                            Spinner::new(px(16.0)).into_any_element()
                        } else if has_query {
                            div()
                                .cursor_pointer()
                                .rounded_full()
                                .p_1()
                                .hover(|s| s.bg(rgb(colors.ui.hover)))
                                .on_click(cx.listener(|this, _, cx| {
                                    this.clear(cx);
                                }))
                                .child(
                                    Icon::new("x")
                                        .size(px(14.0))
                                        .color(rgb(colors.text.muted))
                                )
                                .into_any_element()
                        } else {
                            div().into_any_element()
                        }
                    )
            )
    }
}

#[derive(Clone)]
pub struct SearchQueryChanged(pub String);
```

### Highlighted Results

```rust
// src/components/search_result.rs
use crate::theme::Theme;
use gpui::*;

pub struct HighlightedText {
    text: String,
    highlights: Vec<std::ops::Range<usize>>,
    theme: Arc<Theme>,
}

impl HighlightedText {
    pub fn new(text: String, highlights: Vec<std::ops::Range<usize>>, theme: Arc<Theme>) -> Self {
        Self { text, highlights, theme }
    }
}

impl Render for HighlightedText {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        // Split text into highlighted and non-highlighted segments
        let mut segments = Vec::new();
        let mut last_end = 0;
        
        for range in &self.highlights {
            if range.start > last_end {
                segments.push((false, &self.text[last_end..range.start]));
            }
            segments.push((true, &self.text[range.start..range.end]));
            last_end = range.end;
        }
        
        if last_end < self.text.len() {
            segments.push((false, &self.text[last_end..]));
        }
        
        div()
            .flex()
            .flex_row()
            .children(segments.iter().map(|(is_highlight, text)| {
                if *is_highlight {
                    div()
                        .text_color(rgb(colors.accent.primary))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(*text)
                } else {
                    div()
                        .text_color(rgb(colors.text.primary))
                        .child(*text)
                }
            }))
    }
}

pub struct SearchResultItem {
    result: SearchResult,
    is_selected: bool,
    theme: Arc<Theme>,
}

impl SearchResultItem {
    pub fn new(result: SearchResult, is_selected: bool, theme: Arc<Theme>) -> Self {
        Self { result, is_selected, theme }
    }
}

impl Render for SearchResultItem {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        div()
            .h(px(52.0))
            .px_4()
            .flex()
            .flex_row()
            .items_center()
            .gap_3()
            .bg(rgb(if self.is_selected {
                colors.ui.selected
            } else {
                colors.background.main
            }))
            .cursor_pointer()
            .hover(|s| s.bg(rgb(colors.ui.hover)))
            // Icon
            .when_some(self.result.icon.clone(), |el, icon| {
                el.child(
                    div()
                        .w(px(24.0))
                        .h(px(24.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded_md()
                        .bg(rgb(colors.ui.surface))
                        .child(Icon::new(icon).size(px(14.0)))
                )
            })
            // Content
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .justify_center()
                    .overflow_hidden()
                    // Title with highlights
                    .child(
                        div()
                            .text_size(px(14.0))
                            .truncate()
                            .child(
                                HighlightedText::new(
                                    self.result.title.clone(),
                                    ranges_from_positions(&self.result.match_ranges),
                                    self.theme.clone(),
                                )
                            )
                    )
                    // Subtitle
                    .when_some(self.result.subtitle.clone(), |el, subtitle| {
                        el.child(
                            div()
                                .text_size(px(12.0))
                                .text_color(rgb(colors.text.muted))
                                .truncate()
                                .child(subtitle)
                        )
                    })
            )
            // Category badge
            .when_some(self.result.category.clone(), |el, category| {
                el.child(
                    div()
                        .px_2()
                        .py_1()
                        .rounded_sm()
                        .bg(rgb(colors.ui.surface))
                        .text_size(px(10.0))
                        .text_color(rgb(colors.text.muted))
                        .child(category)
                )
            })
    }
}
```

## Search Patterns

### Instant Search

```rust
// Search as you type with debouncing
impl MainMenu {
    fn handle_search_input(&mut self, query: &str, cx: &mut WindowContext) {
        self.search_query = query.to_string();
        
        // Cancel previous search
        if let Some(task) = self.search_task.take() {
            task.abort();
        }
        
        if query.is_empty() {
            // Show all items immediately
            self.filtered_scripts = self.all_scripts.clone();
            cx.notify();
            return;
        }
        
        // Debounce search
        self.search_task = Some(cx.spawn(|this, mut cx| async move {
            Timer::after(Duration::from_millis(50)).await;
            
            this.update(&mut cx, |this, cx| {
                let start = Instant::now();
                this.filtered_scripts = this.search_scripts(&this.search_query);
                this.search_time = Some(start.elapsed());
                this.selected_index = 0;
                cx.notify();
            }).ok();
        }));
    }
    
    fn search_scripts(&self, query: &str) -> Vec<SearchResult> {
        let matcher = FuzzyMatcher::new();
        
        let mut results: Vec<_> = self.all_scripts
            .iter()
            .filter_map(|script| {
                matcher.match_with_positions(query, &script.name).map(|m| {
                    SearchResult {
                        id: script.id.clone(),
                        title: script.name.clone(),
                        subtitle: script.description.clone(),
                        icon: script.icon.clone(),
                        score: m.score,
                        match_ranges: positions_to_ranges(&m.positions),
                        category: script.category.clone(),
                    }
                })
            })
            .collect();
        
        // Sort by score (highest first)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));
        
        // Limit results
        results.truncate(self.config.max_results);
        
        results
    }
}
```

### Categorized Results

```rust
// Group results by category
impl SearchResults {
    fn render_grouped(&self, cx: &mut WindowContext) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        // Group by category
        let mut groups: HashMap<Option<String>, Vec<&SearchResult>> = HashMap::new();
        for result in &self.results {
            groups.entry(result.category.clone()).or_default().push(result);
        }
        
        // Sort groups
        let mut sorted_groups: Vec<_> = groups.into_iter().collect();
        sorted_groups.sort_by_key(|(cat, _)| cat.clone().unwrap_or_default());
        
        div()
            .flex()
            .flex_col()
            .children(sorted_groups.into_iter().map(|(category, results)| {
                div()
                    .flex()
                    .flex_col()
                    // Category header
                    .when_some(category, |el, cat| {
                        el.child(
                            div()
                                .h(px(28.0))
                                .px_4()
                                .flex()
                                .items_center()
                                .bg(rgb(colors.ui.surface))
                                .child(
                                    div()
                                        .text_size(px(11.0))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(rgb(colors.text.muted))
                                        .text_transform(TextTransform::Uppercase)
                                        .child(cat)
                                )
                        )
                    })
                    // Results
                    .children(results.into_iter().map(|result| {
                        SearchResultItem::new(
                            result.clone(),
                            self.selected_id.as_ref() == Some(&result.id),
                            self.theme.clone(),
                        )
                    }))
            }))
    }
}
```

### Recent Searches

```rust
// Show recent searches when input is empty
impl MainMenu {
    fn render_recent_searches(&self, cx: &mut WindowContext) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        div()
            .flex()
            .flex_col()
            // Header
            .child(
                div()
                    .h(px(28.0))
                    .px_4()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(11.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(colors.text.muted))
                            .child("Recent")
                    )
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(rgb(colors.text.muted))
                            .cursor_pointer()
                            .hover(|s| s.text_color(rgb(colors.accent.primary)))
                            .on_click(cx.listener(|this, _, cx| {
                                this.clear_recent_searches(cx);
                            }))
                            .child("Clear")
                    )
            )
            // Recent items
            .children(self.recent_searches.iter().take(5).map(|query| {
                div()
                    .h(px(40.0))
                    .px_4()
                    .flex()
                    .items_center()
                    .gap_3()
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(colors.ui.hover)))
                    .on_click(cx.listener(move |this, _, cx| {
                        this.set_search_query(query, cx);
                    }))
                    .child(
                        Icon::new("clock")
                            .size(px(14.0))
                            .color(rgb(colors.text.muted))
                    )
                    .child(
                        div()
                            .text_size(px(13.0))
                            .text_color(rgb(colors.text.secondary))
                            .child(query.clone())
                    )
            }))
    }
}
```

## Search Feedback

### Result Count

```rust
// Show result count and search time
impl SearchResults {
    fn render_status(&self, cx: &mut WindowContext) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        div()
            .h(px(32.0))
            .px_4()
            .flex()
            .items_center()
            .justify_between()
            .border_t_1()
            .border_color(rgb(colors.ui.border))
            // Result count
            .child(
                div()
                    .text_size(px(11.0))
                    .text_color(rgb(colors.text.muted))
                    .child(format!(
                        "{} result{}",
                        self.results.len(),
                        if self.results.len() == 1 { "" } else { "s" }
                    ))
            )
            // Search time (debug mode)
            .when_some(self.search_time, |el, time| {
                el.child(
                    div()
                        .text_size(px(10.0))
                        .text_color(rgb(colors.text.muted))
                        .child(format!("{:.1}ms", time.as_secs_f64() * 1000.0))
                )
            })
    }
}
```

### Keyboard Hints

```rust
// Show keyboard navigation hints
impl SearchResults {
    fn render_keyboard_hints(&self, cx: &mut WindowContext) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        div()
            .h(px(28.0))
            .px_4()
            .flex()
            .items_center()
            .gap_4()
            .bg(rgb(colors.ui.surface))
            // Navigate
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(
                        div()
                            .flex()
                            .gap_1()
                            .child(kbd("↑"))
                            .child(kbd("↓"))
                    )
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(rgb(colors.text.muted))
                            .child("navigate")
                    )
            )
            // Select
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(kbd("↵"))
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(rgb(colors.text.muted))
                            .child("select")
                    )
            )
            // Actions
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(kbd("Tab"))
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(rgb(colors.text.muted))
                            .child("actions")
                    )
            )
    }
}

fn kbd(key: &str) -> impl IntoElement {
    div()
        .px_1()
        .py_px()
        .rounded_sm()
        .bg(rgb(0x3F3F46))
        .text_size(px(10.0))
        .font_family("monospace")
        .text_color(rgb(0xA1A1AA))
        .child(key)
}
```

## Testing

### Search Test Script

```typescript
// tests/smoke/test-search-ux.ts
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Test 1: Search input states
await div(`
  <div class="p-4 flex flex-col gap-4">
    <!-- Empty state -->
    <div class="px-4 py-3 bg-zinc-800 rounded-md flex items-center gap-3">
      <span class="text-zinc-500">🔍</span>
      <input type="text" placeholder="Search scripts..." class="flex-1 bg-transparent text-white outline-none" />
    </div>
    
    <!-- With query -->
    <div class="px-4 py-3 bg-zinc-800 rounded-md flex items-center gap-3">
      <span class="text-white">🔍</span>
      <input type="text" value="clipboard" class="flex-1 bg-transparent text-white outline-none" />
      <span class="text-zinc-500 cursor-pointer text-sm">×</span>
    </div>
    
    <!-- Loading -->
    <div class="px-4 py-3 bg-zinc-800 rounded-md flex items-center gap-3">
      <span class="text-white">🔍</span>
      <input type="text" value="search..." class="flex-1 bg-transparent text-white outline-none" />
      <div class="w-4 h-4 border-2 border-amber-500 border-t-transparent rounded-full animate-spin"></div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot1 = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
writeFileSync(join(dir, 'search-input-states.png'), Buffer.from(shot1.data, 'base64'));

// Test 2: Highlighted results
await div(`
  <div class="flex flex-col">
    <div class="h-[52px] px-4 flex items-center gap-3 bg-zinc-800">
      <div class="w-6 h-6 rounded-md bg-zinc-700 flex items-center justify-center text-xs">📋</div>
      <div class="flex-1">
        <div class="text-sm">
          <span class="text-amber-400 font-semibold">Clip</span><span class="text-white">board History</span>
        </div>
        <div class="text-xs text-zinc-500">View and paste from clipboard history</div>
      </div>
    </div>
    <div class="h-[52px] px-4 flex items-center gap-3 hover:bg-zinc-800/50">
      <div class="w-6 h-6 rounded-md bg-zinc-700 flex items-center justify-center text-xs">📎</div>
      <div class="flex-1">
        <div class="text-sm">
          <span class="text-white">Copy </span><span class="text-amber-400 font-semibold">Clip</span><span class="text-white">board</span>
        </div>
        <div class="text-xs text-zinc-500">Copy current selection</div>
      </div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot2 = await captureScreenshot();
writeFileSync(join(dir, 'search-highlights.png'), Buffer.from(shot2.data, 'base64'));

// Test 3: No results
await div(`
  <div class="p-4">
    <div class="px-4 py-3 bg-zinc-800 rounded-md flex items-center gap-3 mb-4">
      <span class="text-white">🔍</span>
      <input type="text" value="xyznonexistent" class="flex-1 bg-transparent text-white outline-none" />
      <span class="text-zinc-500 cursor-pointer text-sm">×</span>
    </div>
    <div class="flex flex-col items-center justify-center py-8">
      <div class="w-12 h-12 rounded-full bg-zinc-800 flex items-center justify-center mb-3">
        <span class="text-2xl opacity-50">🔍</span>
      </div>
      <div class="text-sm text-white mb-1">No results for "xyznonexistent"</div>
      <div class="text-xs text-zinc-500">Try a different search term</div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot3 = await captureScreenshot();
writeFileSync(join(dir, 'search-no-results.png'), Buffer.from(shot3.data, 'base64'));

// Test 4: Keyboard hints footer
await div(`
  <div class="border-t border-zinc-700 h-7 px-4 flex items-center gap-4 bg-zinc-800/50">
    <div class="flex items-center gap-1">
      <span class="px-1 py-0.5 rounded bg-zinc-700 text-[10px] font-mono text-zinc-400">↑</span>
      <span class="px-1 py-0.5 rounded bg-zinc-700 text-[10px] font-mono text-zinc-400">↓</span>
      <span class="text-[11px] text-zinc-500 ml-1">navigate</span>
    </div>
    <div class="flex items-center gap-1">
      <span class="px-1 py-0.5 rounded bg-zinc-700 text-[10px] font-mono text-zinc-400">↵</span>
      <span class="text-[11px] text-zinc-500 ml-1">select</span>
    </div>
    <div class="flex items-center gap-1">
      <span class="px-1 py-0.5 rounded bg-zinc-700 text-[10px] font-mono text-zinc-400">Tab</span>
      <span class="text-[11px] text-zinc-500 ml-1">actions</span>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot4 = await captureScreenshot();
writeFileSync(join(dir, 'search-keyboard-hints.png'), Buffer.from(shot4.data, 'base64'));

console.error('[SEARCH UX] Test screenshots saved');
process.exit(0);
```

## Related Bundles

- Bundle #71: Command Palette Patterns - Search in command palette
- Bundle #75: Empty States - No results state
- Bundle #81: Filtering & Sorting - Advanced filtering
- Bundle #64: List Virtualization - Rendering large result sets
