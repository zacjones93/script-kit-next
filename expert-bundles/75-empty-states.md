# Expert Bundle #75: Empty States

## Overview

Empty states communicate when there's no data to display and guide users toward taking action. In Script Kit, empty states appear in script lists, search results, notes, and chat history. Well-designed empty states reduce confusion and encourage engagement.

## Architecture

### Empty State Types

```rust
// src/empty_states.rs
use gpui::*;

/// Categories of empty states with different intents
#[derive(Clone, Debug)]
pub enum EmptyStateType {
    /// No data exists yet (first-time user)
    NoData {
        title: SharedString,
        description: SharedString,
        action: Option<EmptyStateAction>,
        illustration: Option<EmptyStateIllustration>,
    },
    /// Search/filter returned no results
    NoResults {
        query: String,
        suggestions: Vec<String>,
    },
    /// Error prevented data loading
    LoadError {
        error: String,
        retry_action: Option<Box<dyn Fn(&mut WindowContext)>>,
    },
    /// Feature requires configuration
    NotConfigured {
        feature: SharedString,
        setup_steps: Vec<String>,
    },
    /// Content was intentionally cleared
    Cleared {
        message: SharedString,
        undo_available: bool,
    },
}

#[derive(Clone, Debug)]
pub struct EmptyStateAction {
    pub label: SharedString,
    pub icon: Option<SharedString>,
    pub handler: ActionHandler,
}

#[derive(Clone, Copy, Debug)]
pub enum EmptyStateIllustration {
    Scripts,
    Search,
    Notes,
    Chat,
    Settings,
    Error,
    Success,
}
```

### Empty State Component

```rust
// src/components/empty_state.rs
use crate::theme::Theme;
use gpui::*;

pub struct EmptyState {
    state_type: EmptyStateType,
    theme: Arc<Theme>,
    compact: bool,
}

impl EmptyState {
    pub fn new(state_type: EmptyStateType, theme: Arc<Theme>) -> Self {
        Self {
            state_type,
            theme,
            compact: false,
        }
    }
    
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }
    
    fn render_illustration(&self, illustration: EmptyStateIllustration, cx: &WindowContext) -> impl IntoElement {
        let colors = &self.theme.colors;
        let size = if self.compact { px(48.0) } else { px(80.0) };
        
        // SVG or icon-based illustration
        let icon_name = match illustration {
            EmptyStateIllustration::Scripts => "terminal",
            EmptyStateIllustration::Search => "search",
            EmptyStateIllustration::Notes => "file-text",
            EmptyStateIllustration::Chat => "message-circle",
            EmptyStateIllustration::Settings => "settings",
            EmptyStateIllustration::Error => "alert-circle",
            EmptyStateIllustration::Success => "check-circle",
        };
        
        div()
            .w(size)
            .h(size)
            .flex()
            .items_center()
            .justify_center()
            .rounded_full()
            .bg(rgb(colors.ui.surface))
            .child(
                Icon::new(icon_name)
                    .size(if self.compact { px(24.0) } else { px(40.0) })
                    .color(rgb(colors.text.muted))
            )
    }
    
    fn render_action(&self, action: &EmptyStateAction, cx: &mut WindowContext) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        div()
            .px_4()
            .py_2()
            .rounded_md()
            .bg(rgb(colors.accent.primary))
            .text_color(rgb(colors.background.main))
            .font_weight(FontWeight::MEDIUM)
            .cursor_pointer()
            .hover(|s| s.bg(rgb(colors.accent.hover)))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .when_some(action.icon.clone(), |el, icon| {
                        el.child(Icon::new(icon).size(px(16.0)))
                    })
                    .child(action.label.clone())
            )
    }
}

impl Render for EmptyState {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        let padding = if self.compact { px(16.0) } else { px(32.0) };
        
        match &self.state_type {
            EmptyStateType::NoData { title, description, action, illustration } => {
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .p(padding)
                    .gap_4()
                    .when_some(*illustration, |el, illust| {
                        el.child(self.render_illustration(illust, cx))
                    })
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_size(if self.compact { px(14.0) } else { px(16.0) })
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(colors.text.primary))
                                    .child(title.clone())
                            )
                            .child(
                                div()
                                    .text_size(px(13.0))
                                    .text_color(rgb(colors.text.muted))
                                    .text_align(TextAlign::Center)
                                    .max_w(px(300.0))
                                    .child(description.clone())
                            )
                    )
                    .when_some(action.clone(), |el, act| {
                        el.child(self.render_action(&act, cx))
                    })
            }
            
            EmptyStateType::NoResults { query, suggestions } => {
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .p(padding)
                    .gap_4()
                    .child(self.render_illustration(EmptyStateIllustration::Search, cx))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_size(px(14.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(rgb(colors.text.primary))
                                    .child(format!("No results for \"{}\"", query))
                            )
                            .when(!suggestions.is_empty(), |el| {
                                el.child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .items_center()
                                        .gap_1()
                                        .child(
                                            div()
                                                .text_size(px(12.0))
                                                .text_color(rgb(colors.text.muted))
                                                .child("Try searching for:")
                                        )
                                        .children(suggestions.iter().map(|s| {
                                            div()
                                                .text_size(px(12.0))
                                                .text_color(rgb(colors.accent.primary))
                                                .cursor_pointer()
                                                .child(s.clone())
                                        }))
                                )
                            })
                    )
            }
            
            EmptyStateType::LoadError { error, retry_action } => {
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .p(padding)
                    .gap_4()
                    .child(self.render_illustration(EmptyStateIllustration::Error, cx))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_size(px(14.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(rgb(colors.text.primary))
                                    .child("Failed to load")
                            )
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .text_color(rgb(colors.text.muted))
                                    .child(error.clone())
                            )
                    )
                    .when(retry_action.is_some(), |el| {
                        el.child(
                            div()
                                .px_4()
                                .py_2()
                                .rounded_md()
                                .border_1()
                                .border_color(rgb(colors.ui.border))
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(colors.ui.hover)))
                                .child("Retry")
                        )
                    })
            }
            
            EmptyStateType::NotConfigured { feature, setup_steps } => {
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .p(padding)
                    .gap_4()
                    .child(self.render_illustration(EmptyStateIllustration::Settings, cx))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_size(px(14.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(rgb(colors.text.primary))
                                    .child(format!("{} needs setup", feature))
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .p_3()
                                    .rounded_md()
                                    .bg(rgb(colors.ui.surface))
                                    .children(setup_steps.iter().enumerate().map(|(i, step)| {
                                        div()
                                            .flex()
                                            .flex_row()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .text_size(px(12.0))
                                                    .text_color(rgb(colors.accent.primary))
                                                    .child(format!("{}.", i + 1))
                                            )
                                            .child(
                                                div()
                                                    .text_size(px(12.0))
                                                    .text_color(rgb(colors.text.secondary))
                                                    .child(step.clone())
                                            )
                                    }))
                            )
                    )
            }
            
            EmptyStateType::Cleared { message, undo_available } => {
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .p(padding)
                    .gap_3()
                    .child(
                        div()
                            .text_size(px(13.0))
                            .text_color(rgb(colors.text.muted))
                            .child(message.clone())
                    )
                    .when(*undo_available, |el| {
                        el.child(
                            div()
                                .text_size(px(12.0))
                                .text_color(rgb(colors.accent.primary))
                                .cursor_pointer()
                                .child("Undo")
                        )
                    })
            }
        }
    }
}
```

## Usage Patterns

### Script List Empty State

```rust
// In MainMenu when no scripts exist
fn render_empty_scripts(&self, cx: &mut WindowContext) -> impl IntoElement {
    EmptyState::new(
        EmptyStateType::NoData {
            title: "No scripts yet".into(),
            description: "Create your first script to get started with Script Kit".into(),
            action: Some(EmptyStateAction {
                label: "Create Script".into(),
                icon: Some("plus".into()),
                handler: ActionHandler::new(|cx| {
                    // Navigate to script creation
                }),
            }),
            illustration: Some(EmptyStateIllustration::Scripts),
        },
        self.theme.clone(),
    )
}

// In MainMenu when search returns no results
fn render_no_results(&self, query: &str, cx: &mut WindowContext) -> impl IntoElement {
    let suggestions = self.get_search_suggestions(query);
    
    EmptyState::new(
        EmptyStateType::NoResults {
            query: query.to_string(),
            suggestions,
        },
        self.theme.clone(),
    )
    .compact(true) // Use compact mode in list context
}
```

### Notes Empty State

```rust
// In NotesApp sidebar when no notes
fn render_empty_notes(&self, cx: &mut WindowContext) -> impl IntoElement {
    EmptyState::new(
        EmptyStateType::NoData {
            title: "No notes".into(),
            description: "Press ⌘N to create your first note".into(),
            action: Some(EmptyStateAction {
                label: "New Note".into(),
                icon: Some("edit".into()),
                handler: ActionHandler::new(|cx| {
                    // Create new note
                }),
            }),
            illustration: Some(EmptyStateIllustration::Notes),
        },
        self.theme.clone(),
    )
}
```

### AI Chat Empty State

```rust
// In AIWindow when no API key configured
fn render_not_configured(&self, cx: &mut WindowContext) -> impl IntoElement {
    EmptyState::new(
        EmptyStateType::NotConfigured {
            feature: "AI Chat".into(),
            setup_steps: vec![
                "Get an API key from OpenAI or Anthropic".to_string(),
                "Set SCRIPT_KIT_OPENAI_API_KEY or SCRIPT_KIT_ANTHROPIC_API_KEY".to_string(),
                "Restart Script Kit".to_string(),
            ],
        },
        self.theme.clone(),
    )
}

// When chat history is empty
fn render_empty_chat(&self, cx: &mut WindowContext) -> impl IntoElement {
    EmptyState::new(
        EmptyStateType::NoData {
            title: "Start a conversation".into(),
            description: "Ask a question or describe what you'd like help with".into(),
            action: None, // Focus goes to input automatically
            illustration: Some(EmptyStateIllustration::Chat),
        },
        self.theme.clone(),
    )
}
```

## Best Practices

### Content Guidelines

```rust
/// Empty state content should follow these principles
pub struct EmptyStateContent {
    /// Title: Short, action-oriented (2-4 words)
    /// Good: "No scripts yet", "No results found"
    /// Bad: "There are currently no scripts in your collection"
    pub title: &'static str,
    
    /// Description: Helpful, not apologetic
    /// Good: "Create your first script to get started"
    /// Bad: "Sorry, we couldn't find any scripts"
    pub description: &'static str,
    
    /// Action: Clear verb + object
    /// Good: "Create Script", "Clear Filters"
    /// Bad: "Click here", "Go"
    pub action_label: Option<&'static str>,
}

// Predefined empty states for consistency
pub mod empty_states {
    pub const SCRIPTS_EMPTY: EmptyStateContent = EmptyStateContent {
        title: "No scripts yet",
        description: "Create your first script to automate your workflow",
        action_label: Some("Create Script"),
    };
    
    pub const SEARCH_NO_RESULTS: EmptyStateContent = EmptyStateContent {
        title: "No results",
        description: "Try different keywords or clear filters",
        action_label: Some("Clear Search"),
    };
    
    pub const NOTES_EMPTY: EmptyStateContent = EmptyStateContent {
        title: "No notes",
        description: "Capture ideas, code snippets, and more",
        action_label: Some("New Note"),
    };
    
    pub const HISTORY_EMPTY: EmptyStateContent = EmptyStateContent {
        title: "No history",
        description: "Scripts you run will appear here",
        action_label: None,
    };
    
    pub const FAVORITES_EMPTY: EmptyStateContent = EmptyStateContent {
        title: "No favorites",
        description: "Star scripts to access them quickly",
        action_label: None,
    };
}
```

### Sizing and Placement

```rust
/// Empty state sizing based on context
impl EmptyState {
    /// Full-page empty state (script list, notes main area)
    pub fn full_page(state_type: EmptyStateType, theme: Arc<Theme>) -> Self {
        Self {
            state_type,
            theme,
            compact: false,
            // Full illustrations, generous padding
        }
    }
    
    /// Inline empty state (sidebar, dropdown)
    pub fn inline(state_type: EmptyStateType, theme: Arc<Theme>) -> Self {
        Self {
            state_type,
            theme,
            compact: true,
            // Smaller icons, tighter spacing
        }
    }
    
    /// List empty state (replaces list content)
    pub fn list(state_type: EmptyStateType, theme: Arc<Theme>) -> Self {
        Self {
            state_type,
            theme,
            compact: true,
            // Centered in list area
        }
    }
}
```

### Transition from Empty to Content

```rust
// Smooth transition when content appears
impl MainMenu {
    fn render_list_or_empty(&self, cx: &mut WindowContext) -> impl IntoElement {
        let filtered = self.get_filtered_scripts();
        
        if filtered.is_empty() {
            if self.filter_text.is_empty() {
                // No scripts at all
                EmptyState::new(
                    EmptyStateType::NoData { /* ... */ },
                    self.theme.clone(),
                )
                .into_any_element()
            } else {
                // Search returned nothing
                EmptyState::new(
                    EmptyStateType::NoResults {
                        query: self.filter_text.clone(),
                        suggestions: self.get_suggestions(),
                    },
                    self.theme.clone(),
                )
                .compact(true)
                .into_any_element()
            }
        } else {
            // Show the list
            uniform_list(/* ... */)
                .into_any_element()
        }
    }
}
```

## Testing

### Empty State Test Script

```typescript
// tests/smoke/test-empty-states.ts
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Test 1: Empty script list (no scripts)
await div(`
  <div class="flex flex-col items-center justify-center p-8 gap-4">
    <div class="w-20 h-20 rounded-full bg-zinc-800 flex items-center justify-center">
      <span class="text-4xl opacity-50">📜</span>
    </div>
    <div class="text-center">
      <div class="text-base font-semibold text-white">No scripts yet</div>
      <div class="text-sm text-zinc-400 mt-1">Create your first script to get started</div>
    </div>
    <button class="px-4 py-2 bg-amber-500 text-black rounded-md font-medium">
      Create Script
    </button>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot1 = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
writeFileSync(join(dir, 'empty-state-no-data.png'), Buffer.from(shot1.data, 'base64'));

// Test 2: No search results
await div(`
  <div class="flex flex-col items-center justify-center p-6 gap-3">
    <div class="w-12 h-12 rounded-full bg-zinc-800 flex items-center justify-center">
      <span class="text-2xl opacity-50">🔍</span>
    </div>
    <div class="text-center">
      <div class="text-sm font-medium text-white">No results for "xyzabc"</div>
      <div class="text-xs text-zinc-500 mt-2">Try searching for:</div>
      <div class="text-xs text-amber-400 cursor-pointer">clipboard</div>
      <div class="text-xs text-amber-400 cursor-pointer">window</div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot2 = await captureScreenshot();
writeFileSync(join(dir, 'empty-state-no-results.png'), Buffer.from(shot2.data, 'base64'));

console.error('[EMPTY STATES] Test screenshots saved');
process.exit(0);
```

## Related Bundles

- Bundle #74: Loading States - What to show while loading
- Bundle #76: Error States - When something goes wrong
- Bundle #77: Success Feedback - Positive confirmations
- Bundle #80: Search UX Patterns - Search-specific empty states
