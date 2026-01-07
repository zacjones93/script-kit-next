# Loading States - Expert Bundle

## Overview

Proper loading states prevent user confusion and provide confidence that the application is working. Script Kit uses progressive loading, skeletons, and spinners appropriately.

## Loading State Types

### Inline Spinner

```rust
fn render_inline_loading() -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(
            Icon::new(IconName::Loader2)
                .size_4()
                .text_color(rgb(0x71717A))
                .animate_spin()
        )
        .child(
            div()
                .text_sm()
                .text_color(rgb(0x71717A))
                .child("Loading...")
        )
}
```

### Full Page Loading

```rust
fn render_full_loading() -> impl IntoElement {
    div()
        .w_full()
        .h_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_4()
        .child(
            div()
                .w(px(48.0))
                .h(px(48.0))
                .rounded_full()
                .border_4()
                .border_color(rgb(0x3F3F46))
                .border_t_color(rgb(0x3B82F6))
                .animate_spin()
        )
        .child(
            div()
                .text_sm()
                .text_color(rgb(0x71717A))
                .child("Loading scripts...")
        )
}
```

### Skeleton Loading

```rust
fn render_skeleton_list(count: usize) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .children((0..count).map(|i| {
            render_skeleton_item(i)
        }))
}

fn render_skeleton_item(index: usize) -> impl IntoElement {
    // Stagger animation delay
    let delay = index as f32 * 100.0;
    
    div()
        .h(px(52.0))
        .px_3()
        .flex()
        .items_center()
        .gap_3()
        // Icon skeleton
        .child(
            div()
                .w(px(24.0))
                .h(px(24.0))
                .rounded_md()
                .bg(rgb(0x27272A))
                .animate_pulse()
                .animation_delay(Duration::from_millis(delay as u64))
        )
        // Text skeletons
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .gap_2()
                // Title
                .child(
                    div()
                        .h(px(14.0))
                        .w(px(120.0 + (index as f32 * 20.0) % 80.0))
                        .rounded()
                        .bg(rgb(0x27272A))
                        .animate_pulse()
                        .animation_delay(Duration::from_millis(delay as u64))
                )
                // Description
                .child(
                    div()
                        .h(px(10.0))
                        .w(px(180.0 + (index as f32 * 30.0) % 100.0))
                        .rounded()
                        .bg(rgb(0x27272A))
                        .animate_pulse()
                        .animation_delay(Duration::from_millis(delay as u64))
                )
        )
}
```

## Progress Indicators

### Determinate Progress

```rust
pub struct ProgressBar {
    progress: f32,  // 0.0 to 1.0
    label: Option<String>,
}

impl Render for ProgressBar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let percentage = (self.progress * 100.0) as u32;
        
        div()
            .flex()
            .flex_col()
            .gap_2()
            // Label
            .when_some(self.label.as_ref(), |d, label| {
                d.child(
                    div()
                        .flex()
                        .justify_between()
                        .text_xs()
                        .text_color(rgb(0xA1A1AA))
                        .child(label.clone())
                        .child(format!("{}%", percentage))
                )
            })
            // Track
            .child(
                div()
                    .h(px(4.0))
                    .w_full()
                    .rounded_full()
                    .bg(rgb(0x27272A))
                    .overflow_hidden()
                    // Fill
                    .child(
                        div()
                            .h_full()
                            .w(relative(self.progress))
                            .rounded_full()
                            .bg(rgb(0x3B82F6))
                            .transition_width()
                            .duration_300()
                    )
            )
    }
}
```

### Indeterminate Progress

```rust
fn render_indeterminate_progress() -> impl IntoElement {
    div()
        .h(px(4.0))
        .w_full()
        .rounded_full()
        .bg(rgb(0x27272A))
        .overflow_hidden()
        .child(
            div()
                .h_full()
                .w(relative(0.3))
                .rounded_full()
                .bg(rgb(0x3B82F6))
                .animate_indeterminate_progress()
        )
}
```

## Button Loading States

```rust
pub struct LoadingButton {
    label: String,
    is_loading: bool,
}

impl Render for LoadingButton {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_loading = self.is_loading;
        
        div()
            .px_4()
            .py_2()
            .rounded_md()
            .bg(rgb(0x3B82F6))
            .cursor(if is_loading { CursorStyle::Wait } else { CursorStyle::Pointer })
            .when(is_loading, |d| d.opacity(0.7))
            .flex()
            .items_center()
            .justify_center()
            .gap_2()
            // Spinner (when loading)
            .when(is_loading, |d| {
                d.child(
                    Icon::new(IconName::Loader2)
                        .size_4()
                        .text_color(rgb(0xFFFFFF))
                        .animate_spin()
                )
            })
            // Label
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(rgb(0xFFFFFF))
                    .child(if is_loading {
                        "Loading..."
                    } else {
                        &self.label
                    })
            )
    }
}
```

## Search Loading

```rust
impl SearchInput {
    fn render(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap_2()
            .px_3()
            .h(px(44.0))
            .border_b_1()
            .border_color(rgb(0x3F3F46))
            // Search or loading icon
            .child(
                if self.is_searching {
                    Icon::new(IconName::Loader2)
                        .size_4()
                        .text_color(rgb(0x71717A))
                        .animate_spin()
                        .into_any()
                } else {
                    Icon::new(IconName::Search)
                        .size_4()
                        .text_color(rgb(0x71717A))
                        .into_any()
                }
            )
            // Input
            .child(
                input()
                    .placeholder("Search...")
                    .value(&self.query)
                    .flex_1()
            )
            // Clear button
            .when(!self.query.is_empty(), |d| {
                d.child(
                    IconButton::new("clear")
                        .icon(IconName::X)
                        .ghost()
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.clear(cx);
                        }))
                )
            })
    }
}
```

## Async Loading Pattern

```rust
impl App {
    fn load_scripts(&mut self, cx: &mut Context<Self>) {
        self.loading_state = LoadingState::Loading;
        cx.notify();
        
        cx.spawn(|this, mut cx| async move {
            // Simulate or perform actual loading
            let scripts = load_scripts_from_disk().await;
            
            // Update state on main thread
            let _ = this.update(&mut cx, |app, cx| {
                match scripts {
                    Ok(scripts) => {
                        app.scripts = scripts;
                        app.loading_state = LoadingState::Loaded;
                    }
                    Err(e) => {
                        app.loading_state = LoadingState::Error(e.to_string());
                    }
                }
                cx.notify();
            });
        }).detach();
    }
}

enum LoadingState {
    Idle,
    Loading,
    Loaded,
    Error(String),
}
```

## Render Based on Loading State

```rust
impl Render for App {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        match &self.loading_state {
            LoadingState::Loading => {
                self.render_loading(cx)
            }
            LoadingState::Error(msg) => {
                self.render_error(msg, cx)
            }
            LoadingState::Loaded | LoadingState::Idle => {
                self.render_content(cx)
            }
        }
    }
}
```

## Stale While Revalidate

```rust
impl App {
    fn refresh_scripts(&mut self, cx: &mut Context<Self>) {
        // Show stale data with loading indicator
        self.is_refreshing = true;
        cx.notify();
        
        cx.spawn(|this, mut cx| async move {
            let fresh_scripts = load_scripts_from_disk().await;
            
            let _ = this.update(&mut cx, |app, cx| {
                if let Ok(scripts) = fresh_scripts {
                    app.scripts = scripts;
                }
                app.is_refreshing = false;
                cx.notify();
            });
        }).detach();
    }

    fn render_header(&self) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap_2()
            .child("Scripts")
            // Subtle refresh indicator
            .when(self.is_refreshing, |d| {
                d.child(
                    Icon::new(IconName::RefreshCw)
                        .size_3()
                        .text_color(rgb(0x52525B))
                        .animate_spin()
                )
            })
    }
}
```

## Loading Timeouts

```rust
impl App {
    fn load_with_timeout(&mut self, cx: &mut Context<Self>) {
        self.loading_state = LoadingState::Loading;
        cx.notify();
        
        cx.spawn(|this, mut cx| async move {
            let result = tokio::time::timeout(
                Duration::from_secs(10),
                load_scripts_from_disk()
            ).await;
            
            let _ = this.update(&mut cx, |app, cx| {
                match result {
                    Ok(Ok(scripts)) => {
                        app.scripts = scripts;
                        app.loading_state = LoadingState::Loaded;
                    }
                    Ok(Err(e)) => {
                        app.loading_state = LoadingState::Error(e.to_string());
                    }
                    Err(_) => {
                        app.loading_state = LoadingState::Error(
                            "Loading timed out. Please try again.".to_string()
                        );
                    }
                }
                cx.notify();
            });
        }).detach();
    }
}
```

## Best Practices

1. **Show loading within 100ms** - Any slower feels broken
2. **Use skeletons for lists** - Better than spinners for known layouts
3. **Preserve layout** - Avoid content jumping
4. **Allow cancellation** - Escape should abort loading
5. **Show stale data** - While refreshing in background
6. **Add timeouts** - Don't hang forever
7. **Disable interactions** - When loading affects actions

## Loading State Reference

| Scenario | Indicator | Notes |
|----------|-----------|-------|
| Initial load | Full skeleton | Show expected layout |
| Search | Icon spinner | Replace search icon |
| Refresh | Subtle spinner | Keep showing data |
| Button action | Button spinner | Disable button |
| Form submit | Button spinner | Disable form |
| Slow operation | Progress bar | If progress knowable |
