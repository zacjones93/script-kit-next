impl ScriptListApp {
    /// Read the first N lines of a script file for preview
    #[allow(dead_code)]
    fn read_script_preview(path: &std::path::Path, max_lines: usize) -> String {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                let preview: String = content
                    .lines()
                    .take(max_lines)
                    .collect::<Vec<_>>()
                    .join("\n");
                logging::log(
                    "UI",
                    &format!(
                        "Preview loaded: {} ({} lines read)",
                        path.file_name().unwrap_or_default().to_string_lossy(),
                        content.lines().count().min(max_lines)
                    ),
                );
                preview
            }
            Err(e) => {
                logging::log("UI", &format!("Preview error: {} - {}", path.display(), e));
                format!("Error reading file: {}", e)
            }
        }
    }

    /// Render toast notifications from the toast manager
    ///
    /// Toasts are positioned in the top-right corner and stack vertically.
    /// Each toast has its own dismiss callback that removes it from the manager.
    fn render_toasts(&mut self, _cx: &mut Context<Self>) -> Option<impl IntoElement> {
        // Tick the manager to handle auto-dismiss
        self.toast_manager.tick();

        // Clean up dismissed toasts
        self.toast_manager.cleanup();

        // Check if toasts need update (consume the flag to prevent repeated checks)
        // Note: We don't call cx.notify() here as it's an anti-pattern during render.
        // Toast updates are handled by timer-based refresh mechanisms.
        let _ = self.toast_manager.take_needs_notify();

        let visible = self.toast_manager.visible_toasts();
        if visible.is_empty() {
            return None;
        }

        // Use design tokens for consistent spacing
        let tokens = get_tokens(self.current_design);
        let spacing = tokens.spacing();

        // Build toast container (positioned in top-right via absolute positioning)
        let mut toast_container = div()
            .absolute()
            .top(px(spacing.padding_lg))
            .right(px(spacing.padding_lg))
            .flex()
            .flex_col()
            .gap(px(spacing.gap_sm))
            .w(px(380.0)); // Fixed width for toasts

        // Add each visible toast
        for notification in visible {
            // Clone the toast for rendering - unfortunately we need to recreate it
            // since Toast::render consumes self
            let toast_colors =
                ToastColors::from_theme(&self.theme, notification.toast.get_variant());
            let toast = Toast::new(notification.toast.get_message().clone(), toast_colors)
                .variant(notification.toast.get_variant())
                .duration_ms(notification.toast.get_duration_ms())
                .dismissible(true);

            // Add details if the toast has them
            let toast = toast.details_opt(notification.toast.get_details().cloned());

            toast_container = toast_container.child(toast);
        }

        Some(toast_container)
    }

    /// Render the preview panel showing details of the selected script/scriptlet
    fn render_preview_panel(&mut self, _cx: &mut Context<Self>) -> impl IntoElement {
        // Get grouped results to map from selected_index to actual result (cached)
        // Clone to avoid borrow issues with self.selected_index access
        let selected_index = self.selected_index;
        let (grouped_items, flat_results) = self.get_grouped_results_cached();
        let grouped_items = grouped_items.clone();
        let flat_results = flat_results.clone();

        // Get the result index from the grouped item
        let selected_result = match grouped_items.get(selected_index) {
            Some(GroupedListItem::Item(idx)) => flat_results.get(*idx).cloned(),
            _ => None,
        };

        // Use design tokens for GLOBAL theming - design applies to ALL components
        let tokens = get_tokens(self.current_design);
        let colors = tokens.colors();
        let spacing = tokens.spacing();
        let typography = tokens.typography();
        let visual = tokens.visual();

        // Map design tokens to local variables (all designs use tokens now)
        let bg_main = colors.background;
        let ui_border = colors.border;
        let text_primary = colors.text_primary;
        let text_muted = colors.text_muted;
        let text_secondary = colors.text_secondary;
        let bg_search_box = colors.background_tertiary;
        let border_radius = visual.radius_md;
        let font_family = typography.font_family;

        // Preview panel container with left border separator
        let mut panel = div()
            .w_full()
            .h_full()
            .bg(rgb(bg_main))
            .border_l_1()
            .border_color(rgba((ui_border << 8) | 0x80))
            .p(px(spacing.padding_lg))
            .flex()
            .flex_col()
            .overflow_y_hidden()
            .font_family(font_family);

        // P4: Compute match indices lazily for visible preview (only one result at a time)
        let computed_filter = self.computed_filter_text.clone();

        match selected_result {
            Some(ref result) => {
                // P4: Lazy match indices computation for preview panel
                let match_indices =
                    scripts::compute_match_indices_for_result(result, &computed_filter);

                match result {
                    scripts::SearchResult::Script(script_match) => {
                        let script = &script_match.script;

                        // Source indicator with match highlighting (e.g., "script: foo.ts")
                        let filename = &script_match.filename;
                        // P4: Use lazily computed indices instead of stored (empty) ones
                        let filename_indices = &match_indices.filename_indices;

                        // Render filename with highlighted matched characters
                        let path_segments =
                            render_path_with_highlights(filename, filename, filename_indices);
                        let accent_color = colors.accent;

                        let mut path_div = div()
                            .flex()
                            .flex_row()
                            .text_xs()
                            .font_family(typography.font_family_mono)
                            .pb(px(spacing.padding_xs))
                            .overflow_x_hidden()
                            .child(
                                div()
                                    .text_color(rgba((text_muted << 8) | 0x99))
                                    .child("script: "),
                            );

                        for (text, is_highlighted) in path_segments {
                            let color = if is_highlighted {
                                rgb(accent_color)
                            } else {
                                rgba((text_muted << 8) | 0x99)
                            };
                            path_div = path_div.child(div().text_color(color).child(text));
                        }

                        panel = panel.child(path_div);

                        // Script name header
                        panel = panel.child(
                            div()
                                .text_lg()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(text_primary))
                                .pb(px(spacing.padding_sm))
                                .child(format!("{}.{}", script.name, script.extension)),
                        );

                        // Description (if present)
                        if let Some(desc) = &script.description {
                            panel = panel.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .pb(px(spacing.padding_md))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_muted))
                                            .pb(px(spacing.padding_xs / 2.0))
                                            .child("Description"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(text_secondary))
                                            .child(desc.clone()),
                                    ),
                            );
                        }

                        // Divider
                        panel = panel.child(
                            div()
                                .w_full()
                                .h(px(visual.border_thin))
                                .bg(rgba((ui_border << 8) | 0x60))
                                .my(px(spacing.padding_sm)),
                        );

                        // Code preview header
                        panel = panel.child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_muted))
                                .pb(px(spacing.padding_sm))
                                .child("Code Preview"),
                        );

                        // Use cached syntax-highlighted lines (avoids file I/O and highlighting on every render)
                        let script_path = script.path.to_string_lossy().to_string();
                        let lang = script.extension.clone();
                        let lines = self
                            .get_or_update_preview_cache(&script_path, &lang)
                            .to_vec();

                        // Build code container - render line by line with monospace font
                        let mut code_container = div()
                            .w_full()
                            .min_w(px(280.))
                            .p(px(spacing.padding_md))
                            .rounded(px(border_radius))
                            .bg(rgba((bg_search_box << 8) | 0x80))
                            .overflow_hidden()
                            .flex()
                            .flex_col();

                        // Render each line as a row of spans with monospace font
                        for line in lines {
                            let mut line_div = div()
                                .flex()
                                .flex_row()
                                .w_full()
                                .font_family(typography.font_family_mono)
                                .text_xs()
                                .min_h(px(spacing.padding_lg)); // Line height

                            if line.spans.is_empty() {
                                // Empty line - add a space to preserve height
                                line_div = line_div.child(" ");
                            } else {
                                for span in line.spans {
                                    line_div = line_div
                                        .child(div().text_color(rgb(span.color)).child(span.text));
                                }
                            }

                            code_container = code_container.child(line_div);
                        }

                        panel = panel.child(code_container);
                    }
                    scripts::SearchResult::Scriptlet(scriptlet_match) => {
                        let scriptlet = &scriptlet_match.scriptlet;

                        // Source indicator with match highlighting (e.g., "scriptlet: foo.md")
                        if let Some(ref display_file_path) = scriptlet_match.display_file_path {
                            // P4: Use lazily computed indices instead of stored (empty) ones
                            let filename_indices = &match_indices.filename_indices;

                            // Render filename with highlighted matched characters
                            let path_segments = render_path_with_highlights(
                                display_file_path,
                                display_file_path,
                                filename_indices,
                            );
                            let accent_color = colors.accent;

                            let mut path_div = div()
                                .flex()
                                .flex_row()
                                .text_xs()
                                .font_family(typography.font_family_mono)
                                .pb(px(spacing.padding_xs))
                                .overflow_x_hidden()
                                .child(
                                    div()
                                        .text_color(rgba((text_muted << 8) | 0x99))
                                        .child("scriptlet: "),
                                );

                            for (text, is_highlighted) in path_segments {
                                let color = if is_highlighted {
                                    rgb(accent_color)
                                } else {
                                    rgba((text_muted << 8) | 0x99)
                                };
                                path_div = path_div.child(div().text_color(color).child(text));
                            }

                            panel = panel.child(path_div);
                        }

                        // Scriptlet name header
                        panel = panel.child(
                            div()
                                .text_lg()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(text_primary))
                                .pb(px(spacing.padding_sm))
                                .child(scriptlet.name.clone()),
                        );

                        // Description (if present)
                        if let Some(desc) = &scriptlet.description {
                            panel = panel.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .pb(px(spacing.padding_md))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_muted))
                                            .pb(px(spacing.padding_xs / 2.0))
                                            .child("Description"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(text_secondary))
                                            .child(desc.clone()),
                                    ),
                            );
                        }

                        // Shortcut (if present)
                        if let Some(shortcut) = &scriptlet.shortcut {
                            panel = panel.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .pb(px(spacing.padding_md))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_muted))
                                            .pb(px(spacing.padding_xs / 2.0))
                                            .child("Hotkey"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(text_secondary))
                                            .child(shortcut.clone()),
                                    ),
                            );
                        }

                        // Divider
                        panel = panel.child(
                            div()
                                .w_full()
                                .h(px(visual.border_thin))
                                .bg(rgba((ui_border << 8) | 0x60))
                                .my(px(spacing.padding_sm)),
                        );

                        // Content preview header
                        panel = panel.child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_muted))
                                .pb(px(spacing.padding_sm))
                                .child("Content Preview"),
                        );

                        // Display scriptlet code with syntax highlighting (first 15 lines)
                        // Note: Scriptlets store code in memory, no file I/O needed (no cache benefit)
                        let code_preview: String = scriptlet
                            .code
                            .lines()
                            .take(15)
                            .collect::<Vec<_>>()
                            .join("\n");

                        // Determine language from tool (bash, js, etc.)
                        let lang = match scriptlet.tool.as_str() {
                            "bash" | "zsh" | "sh" => "bash",
                            "node" | "bun" => "js",
                            _ => &scriptlet.tool,
                        };
                        let lines = highlight_code_lines(&code_preview, lang);

                        // Build code container - render line by line with monospace font
                        let mut code_container = div()
                            .w_full()
                            .min_w(px(280.))
                            .p(px(spacing.padding_md))
                            .rounded(px(border_radius))
                            .bg(rgba((bg_search_box << 8) | 0x80))
                            .overflow_hidden()
                            .flex()
                            .flex_col();

                        // Render each line as a row of spans with monospace font
                        for line in lines {
                            let mut line_div = div()
                                .flex()
                                .flex_row()
                                .w_full()
                                .font_family(typography.font_family_mono)
                                .text_xs()
                                .min_h(px(spacing.padding_lg)); // Line height

                            if line.spans.is_empty() {
                                // Empty line - add a space to preserve height
                                line_div = line_div.child(" ");
                            } else {
                                for span in line.spans {
                                    line_div = line_div
                                        .child(div().text_color(rgb(span.color)).child(span.text));
                                }
                            }

                            code_container = code_container.child(line_div);
                        }

                        panel = panel.child(code_container);
                    }
                    scripts::SearchResult::BuiltIn(builtin_match) => {
                        let builtin = &builtin_match.entry;

                        // Built-in name header
                        panel = panel.child(
                            div()
                                .text_lg()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(text_primary))
                                .pb(px(spacing.padding_sm))
                                .child(builtin.name.clone()),
                        );

                        // Description
                        panel = panel.child(
                            div()
                                .flex()
                                .flex_col()
                                .pb(px(spacing.padding_md))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .pb(px(spacing.padding_xs / 2.0))
                                        .child("Description"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_secondary))
                                        .child(builtin.description.clone()),
                                ),
                        );

                        // Keywords
                        if !builtin.keywords.is_empty() {
                            panel = panel.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .pb(px(spacing.padding_md))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_muted))
                                            .pb(px(spacing.padding_xs / 2.0))
                                            .child("Keywords"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(text_secondary))
                                            .child(builtin.keywords.join(", ")),
                                    ),
                            );
                        }

                        // Divider
                        panel = panel.child(
                            div()
                                .w_full()
                                .h(px(visual.border_thin))
                                .bg(rgba((ui_border << 8) | 0x60))
                                .my(px(spacing.padding_sm)),
                        );

                        // Feature type indicator
                        let feature_type: String = match &builtin.feature {
                            builtins::BuiltInFeature::ClipboardHistory => {
                                "Clipboard History Manager".to_string()
                            }
                            builtins::BuiltInFeature::AppLauncher => {
                                "Application Launcher".to_string()
                            }
                            builtins::BuiltInFeature::App(name) => name.clone(),
                            builtins::BuiltInFeature::WindowSwitcher => {
                                "Window Manager".to_string()
                            }
                            builtins::BuiltInFeature::DesignGallery => "Design Gallery".to_string(),
                        };
                        panel = panel.child(
                            div()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .pb(px(spacing.padding_xs / 2.0))
                                        .child("Feature Type"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_secondary))
                                        .child(feature_type),
                                ),
                        );
                    }
                    scripts::SearchResult::App(app_match) => {
                        let app = &app_match.app;

                        // App name header
                        panel = panel.child(
                            div()
                                .text_lg()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(text_primary))
                                .pb(px(spacing.padding_sm))
                                .child(app.name.clone()),
                        );

                        // Path
                        panel = panel.child(
                            div()
                                .flex()
                                .flex_col()
                                .pb(px(spacing.padding_md))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .pb(px(spacing.padding_xs / 2.0))
                                        .child("Path"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_secondary))
                                        .child(app.path.to_string_lossy().to_string()),
                                ),
                        );

                        // Bundle ID (if available)
                        if let Some(bundle_id) = &app.bundle_id {
                            panel = panel.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .pb(px(spacing.padding_md))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_muted))
                                            .pb(px(spacing.padding_xs / 2.0))
                                            .child("Bundle ID"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(text_secondary))
                                            .child(bundle_id.clone()),
                                    ),
                            );
                        }

                        // Divider
                        panel = panel.child(
                            div()
                                .w_full()
                                .h(px(visual.border_thin))
                                .bg(rgba((ui_border << 8) | 0x60))
                                .my(px(spacing.padding_sm)),
                        );

                        // Type indicator
                        panel = panel.child(
                            div()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .pb(px(spacing.padding_xs / 2.0))
                                        .child("Type"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_secondary))
                                        .child("Application"),
                                ),
                        );
                    }
                    scripts::SearchResult::Window(window_match) => {
                        let window = &window_match.window;

                        // Window title header
                        panel = panel.child(
                            div()
                                .text_lg()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(text_primary))
                                .pb(px(spacing.padding_sm))
                                .child(window.title.clone()),
                        );

                        // App name
                        panel = panel.child(
                            div()
                                .flex()
                                .flex_col()
                                .pb(px(spacing.padding_md))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .pb(px(spacing.padding_xs / 2.0))
                                        .child("Application"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_secondary))
                                        .child(window.app.clone()),
                                ),
                        );

                        // Bounds
                        panel = panel.child(
                            div()
                                .flex()
                                .flex_col()
                                .pb(px(spacing.padding_md))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .pb(px(spacing.padding_xs / 2.0))
                                        .child("Position & Size"),
                                )
                                .child(div().text_sm().text_color(rgb(text_secondary)).child(
                                    format!(
                                        "{}×{} at ({}, {})",
                                        window.bounds.width,
                                        window.bounds.height,
                                        window.bounds.x,
                                        window.bounds.y
                                    ),
                                )),
                        );

                        // Divider
                        panel = panel.child(
                            div()
                                .w_full()
                                .h(px(visual.border_thin))
                                .bg(rgba((ui_border << 8) | 0x60))
                                .my(px(spacing.padding_sm)),
                        );

                        // Type indicator
                        panel = panel.child(
                            div()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .pb(px(spacing.padding_xs / 2.0))
                                        .child("Type"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_secondary))
                                        .child("Window"),
                                ),
                        );
                    }
                }
            }
            None => {
                logging::log("UI", "Preview panel: No selection");
                // Empty state
                panel = panel.child(
                    div()
                        .w_full()
                        .h_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(rgb(text_muted))
                        .child(
                            if self.filter_text.is_empty()
                                && self.scripts.is_empty()
                                && self.scriptlets.is_empty()
                            {
                                "No scripts or snippets found"
                            } else if !self.filter_text.is_empty() {
                                "No matching scripts"
                            } else {
                                "Select a script to preview"
                            },
                        ),
                );
            }
        }

        panel
    }

    /// Get the ScriptInfo for the currently focused/selected script
    fn get_focused_script_info(&mut self) -> Option<ScriptInfo> {
        // Get grouped results to map from selected_index to actual result (cached)
        let (grouped_items, flat_results) = self.get_grouped_results_cached();
        // Clone to avoid borrow issues
        let grouped_items = grouped_items.clone();
        let flat_results = flat_results.clone();

        // Get the result index from the grouped item
        let result_idx = match grouped_items.get(self.selected_index) {
            Some(GroupedListItem::Item(idx)) => Some(*idx),
            _ => None,
        };

        if let Some(idx) = result_idx {
            if let Some(result) = flat_results.get(idx) {
                match result {
                    scripts::SearchResult::Script(m) => Some(ScriptInfo::new(
                        &m.script.name,
                        m.script.path.to_string_lossy(),
                    )),
                    scripts::SearchResult::Scriptlet(m) => {
                        // Scriptlets don't have a path, use name as identifier
                        Some(ScriptInfo::new(
                            &m.scriptlet.name,
                            format!("scriptlet:{}", &m.scriptlet.name),
                        ))
                    }
                    scripts::SearchResult::BuiltIn(m) => {
                        // Built-ins use their id as identifier
                        Some(ScriptInfo::new(
                            &m.entry.name,
                            format!("builtin:{}", &m.entry.id),
                        ))
                    }
                    scripts::SearchResult::App(m) => {
                        // Apps use their path as identifier
                        Some(ScriptInfo::new(
                            &m.app.name,
                            m.app.path.to_string_lossy().to_string(),
                        ))
                    }
                    scripts::SearchResult::Window(m) => {
                        // Windows use their id as identifier
                        Some(ScriptInfo::new(
                            &m.window.title,
                            format!("window:{}", m.window.id),
                        ))
                    }
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    fn render_script_list(&mut self, cx: &mut Context<Self>) -> AnyElement {
        // Get grouped or flat results based on filter state (cached) - MUST come first
        // to avoid borrow conflicts with theme access below
        // When filter is empty, use frecency-grouped results with RECENT/MAIN sections
        // When filtering, use flat fuzzy search results
        let (grouped_items, flat_results) = self.get_grouped_results_cached();
        // Clone for use in closures and to avoid borrow issues
        let grouped_items = grouped_items.clone();
        let flat_results = flat_results.clone();

        // Get design tokens for current design variant
        let tokens = get_tokens(self.current_design);
        let design_colors = tokens.colors();
        let design_spacing = tokens.spacing();
        let design_visual = tokens.visual();
        let design_typography = tokens.typography();
        let theme = &self.theme;

        // For Default design, use theme.colors for backward compatibility
        // For other designs, use design tokens
        let is_default_design = self.current_design == DesignVariant::Default;

        // P4: Pre-compute theme values using ListItemColors
        let _list_colors = ListItemColors::from_theme(theme);
        logging::log_debug("PERF", "P4: Using ListItemColors for render closure");

        let item_count = grouped_items.len();
        let _total_len = self.scripts.len() + self.scriptlets.len();

        // Handle edge cases - keep selected_index in valid bounds
        // Also skip section headers when adjusting bounds
        if item_count > 0 {
            if self.selected_index >= item_count {
                self.selected_index = item_count.saturating_sub(1);
            }
            // If we land on a section header, move to first valid item
            if let Some(GroupedListItem::SectionHeader(_)) = grouped_items.get(self.selected_index)
            {
                // Move down to find first Item
                for (i, item) in grouped_items.iter().enumerate().skip(self.selected_index) {
                    if matches!(item, GroupedListItem::Item(_)) {
                        self.selected_index = i;
                        break;
                    }
                }
            }
        }

        // Build script list using uniform_list for proper virtualized scrolling
        // Use design tokens for empty state styling
        let empty_text_color = if is_default_design {
            theme.colors.text.muted
        } else {
            design_colors.text_muted
        };
        let empty_font_family = if is_default_design {
            ".AppleSystemUIFont"
        } else {
            design_typography.font_family
        };

        let list_element: AnyElement = if item_count == 0 {
            div()
                .w_full()
                .h_full()
                .flex()
                .items_center()
                .justify_center()
                .text_color(rgb(empty_text_color))
                .font_family(empty_font_family)
                .child(if self.filter_text.is_empty() {
                    "No scripts or snippets found".to_string()
                } else {
                    format!("No results match '{}'", self.filter_text)
                })
                .into_any_element()
        } else {
            // Use GPUI's list() component for variable-height items
            // Section headers render at 24px, regular items at 48px
            // This gives true visual compression for headers without the uniform_list hack

            // Clone grouped_items and flat_results for the closure
            let grouped_items_clone = grouped_items.clone();
            let flat_results_clone = flat_results.clone();

            // Calculate scrollbar parameters
            // Estimate visible items based on typical container height
            // Note: With variable heights, this is approximate
            let estimated_container_height = 400.0_f32; // Typical visible height
            let visible_items = (estimated_container_height / LIST_ITEM_HEIGHT) as usize;

            // Use selected_index as approximate scroll offset
            let scroll_offset = if self.selected_index > visible_items.saturating_sub(1) {
                self.selected_index.saturating_sub(visible_items / 2)
            } else {
                0
            };

            // Get scrollbar colors from theme or design
            let scrollbar_colors = if is_default_design {
                ScrollbarColors::from_theme(theme)
            } else {
                ScrollbarColors::from_design(&design_colors)
            };

            // Create scrollbar (only visible if content overflows and scrolling is active)
            let scrollbar =
                Scrollbar::new(item_count, visible_items, scroll_offset, scrollbar_colors)
                    .container_height(estimated_container_height)
                    .visible(self.is_scrolling);

            // Update list state if item count changed
            if self.main_list_state.item_count() != item_count {
                self.main_list_state.reset(item_count);
            }

            // Scroll to reveal selected item
            self.main_list_state
                .scroll_to_reveal_item(self.selected_index);

            // Capture entity handle for use in the render closure
            let entity = cx.entity();

            // Clone values needed in the closure (can't access self in FnMut)
            let theme_colors = ListItemColors::from_theme(&self.theme);
            let current_design = self.current_design;

            let variable_height_list =
                list(self.main_list_state.clone(), move |ix, _window, cx| {
                    // Access entity state inside the closure
                    entity.update(cx, |this, cx| {
                        let current_selected = this.selected_index;
                        let current_hovered = this.hovered_index;

                        if let Some(grouped_item) = grouped_items_clone.get(ix) {
                            match grouped_item {
                                GroupedListItem::SectionHeader(label) => {
                                    // Section header at 24px height (SECTION_HEADER_HEIGHT)
                                    div()
                                        .id(ElementId::NamedInteger(
                                            "section-header".into(),
                                            ix as u64,
                                        ))
                                        .h(px(SECTION_HEADER_HEIGHT))
                                        .child(render_section_header(label, theme_colors))
                                        .into_any_element()
                                }
                                GroupedListItem::Item(result_idx) => {
                                    // Regular item at 48px height (LIST_ITEM_HEIGHT)
                                    if let Some(result) = flat_results_clone.get(*result_idx) {
                                        let is_selected = ix == current_selected;
                                        let is_hovered = current_hovered == Some(ix);

                                        // Create hover handler
                                        let hover_handler = cx.listener(
                                            move |this: &mut ScriptListApp,
                                                  hovered: &bool,
                                                  _window,
                                                  cx| {
                                                let now = std::time::Instant::now();
                                                const HOVER_DEBOUNCE_MS: u64 = 16;

                                                if *hovered {
                                                    // Mouse entered - set hovered_index with debounce
                                                    if this.hovered_index != Some(ix)
                                                        && now
                                                            .duration_since(this.last_hover_notify)
                                                            .as_millis()
                                                            >= HOVER_DEBOUNCE_MS as u128
                                                    {
                                                        this.hovered_index = Some(ix);
                                                        this.last_hover_notify = now;
                                                        cx.notify();
                                                    }
                                                } else if this.hovered_index == Some(ix) {
                                                    // Mouse left - clear hovered_index if it was this item
                                                    this.hovered_index = None;
                                                    this.last_hover_notify = now;
                                                    cx.notify();
                                                }
                                            },
                                        );

                                        // Create click handler
                                        let click_handler = cx.listener(
                                            move |this: &mut ScriptListApp,
                                                  _event: &gpui::ClickEvent,
                                                  _window,
                                                  cx| {
                                                if this.selected_index != ix {
                                                    this.selected_index = ix;
                                                    cx.notify();
                                                }
                                            },
                                        );

                                        // Dispatch to design-specific item renderer
                                        let item_element = render_design_item(
                                            current_design,
                                            result,
                                            ix,
                                            is_selected,
                                            is_hovered,
                                            theme_colors,
                                        );

                                        div()
                                            .id(ElementId::NamedInteger(
                                                "script-item".into(),
                                                ix as u64,
                                            ))
                                            .h(px(LIST_ITEM_HEIGHT)) // Explicit 48px height
                                            .on_hover(hover_handler)
                                            .on_click(click_handler)
                                            .child(item_element)
                                            .into_any_element()
                                    } else {
                                        // Fallback for missing result
                                        div().h(px(LIST_ITEM_HEIGHT)).into_any_element()
                                    }
                                }
                            }
                        } else {
                            // Fallback for out-of-bounds index
                            div().h(px(LIST_ITEM_HEIGHT)).into_any_element()
                        }
                    })
                })
                // Enable proper scroll handling for mouse wheel/trackpad
                // ListSizingBehavior::Infer sets overflow.y = Overflow::Scroll internally
                // which is required for the list's hitbox to capture scroll wheel events
                .with_sizing_behavior(ListSizingBehavior::Infer)
                .h_full();

            // Wrap list in a relative container with scrollbar overlay
            // The list() component with ListSizingBehavior::Infer handles scroll internally
            // No custom on_scroll_wheel handler needed - let GPUI handle it natively
            div()
                .relative()
                .flex()
                .flex_col()
                .flex_1()
                .w_full()
                .h_full()
                .child(variable_height_list)
                .child(scrollbar)
                .into_any_element()
        };

        // Log panel
        let log_panel = if self.show_logs {
            let logs = logging::get_last_logs(10);
            let mut log_container = div()
                .flex()
                .flex_col()
                .w_full()
                .bg(rgb(theme.colors.background.log_panel))
                .border_t_1()
                .border_color(rgb(theme.colors.ui.border))
                .p(px(design_spacing.padding_md))
                .max_h(px(120.))
                .font_family("SF Mono");

            for log_line in logs.iter().rev() {
                log_container = log_container.child(
                    div()
                        .text_color(rgb(theme.colors.ui.success))
                        .text_xs()
                        .child(log_line.clone()),
                );
            }
            Some(log_container)
        } else {
            None
        };

        let filter_display = if self.filter_text.is_empty() {
            SharedString::from(DEFAULT_PLACEHOLDER)
        } else {
            SharedString::from(self.filter_text.clone())
        };
        let filter_is_empty = self.filter_text.is_empty();

        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                let key_str = event.keystroke.key.to_lowercase();
                let has_cmd = event.keystroke.modifiers.platform;

                // Check SDK action shortcuts FIRST (before built-in shortcuts)
                // This allows scripts to override default shortcuts via setActions()
                if !this.action_shortcuts.is_empty() {
                    let key_combo =
                        shortcuts::keystroke_to_shortcut(&key_str, &event.keystroke.modifiers);
                    if let Some(action_name) = this.action_shortcuts.get(&key_combo).cloned() {
                        logging::log(
                            "ACTIONS",
                            &format!(
                                "SDK action shortcut matched: '{}' -> '{}'",
                                key_combo, action_name
                            ),
                        );
                        if this.trigger_action_by_name(&action_name, cx) {
                            return;
                        }
                    }
                }

                if has_cmd {
                    let has_shift = event.keystroke.modifiers.shift;

                    match key_str.as_str() {
                        "l" => {
                            this.toggle_logs(cx);
                            return;
                        }
                        "k" => {
                            this.toggle_actions(cx, window);
                            return;
                        }
                        // Cmd+1 cycles through all designs
                        "1" => {
                            this.cycle_design(cx);
                            return;
                        }
                        // Script context shortcuts (require a selected script)
                        "e" => {
                            // Cmd+E - Edit Script
                            this.handle_action("edit_script".to_string(), cx);
                            return;
                        }
                        "f" if has_shift => {
                            // Cmd+Shift+F - Reveal in Finder
                            this.handle_action("reveal_in_finder".to_string(), cx);
                            return;
                        }
                        "c" if has_shift => {
                            // Cmd+Shift+C - Copy Path
                            this.handle_action("copy_path".to_string(), cx);
                            return;
                        }
                        // Global shortcuts
                        "n" => {
                            // Cmd+N - Create Script
                            this.handle_action("create_script".to_string(), cx);
                            return;
                        }
                        "r" => {
                            // Cmd+R - Reload Scripts
                            this.handle_action("reload_scripts".to_string(), cx);
                            return;
                        }
                        "," => {
                            // Cmd+, - Settings
                            this.handle_action("settings".to_string(), cx);
                            return;
                        }
                        "q" => {
                            // Cmd+Q - Quit
                            this.handle_action("quit".to_string(), cx);
                            return;
                        }
                        _ => {}
                    }
                }

                // If actions popup is open, route keyboard events to it
                if this.show_actions_popup {
                    if let Some(ref dialog) = this.actions_dialog {
                        match key_str.as_str() {
                            "up" | "arrowup" => {
                                dialog.update(cx, |d, cx| d.move_up(cx));
                                return;
                            }
                            "down" | "arrowdown" => {
                                dialog.update(cx, |d, cx| d.move_down(cx));
                                return;
                            }
                            "enter" => {
                                // Get the selected action and execute it
                                let action_id = dialog.read(cx).get_selected_action_id();
                                let should_close = dialog.read(cx).selected_action_should_close();
                                if let Some(action_id) = action_id {
                                    logging::log(
                                        "ACTIONS",
                                        &format!(
                                            "Executing action: {} (close={})",
                                            action_id, should_close
                                        ),
                                    );
                                    // Only close if action has close: true (default)
                                    if should_close {
                                        this.show_actions_popup = false;
                                        this.actions_dialog = None;
                                        this.focused_input = FocusedInput::MainFilter;
                                        window.focus(&this.focus_handle, cx);
                                    }
                                    this.handle_action(action_id, cx);
                                }
                                return;
                            }
                            "escape" => {
                                this.show_actions_popup = false;
                                this.actions_dialog = None;
                                this.focused_input = FocusedInput::MainFilter;
                                window.focus(&this.focus_handle, cx);
                                cx.notify();
                                return;
                            }
                            "backspace" => {
                                dialog.update(cx, |d, cx| d.handle_backspace(cx));
                                return;
                            }
                            _ => {
                                // Route character input to the dialog for search
                                if let Some(ref key_char) = event.keystroke.key_char {
                                    if let Some(ch) = key_char.chars().next() {
                                        if !ch.is_control() {
                                            dialog.update(cx, |d, cx| d.handle_char(ch, cx));
                                        }
                                    }
                                }
                                return;
                            }
                        }
                    }
                }

                match key_str.as_str() {
                    "up" | "arrowup" => {
                        let _key_perf = crate::perf::KeyEventPerfGuard::new();
                        match this.nav_coalescer.record(NavDirection::Up) {
                            NavRecord::ApplyImmediate => this.move_selection_up(cx),
                            NavRecord::Coalesced => {}
                            NavRecord::FlushOld { dir, delta } => {
                                if delta != 0 {
                                    this.apply_nav_delta(dir, delta, cx);
                                }
                                this.move_selection_up(cx);
                            }
                        }
                        this.ensure_nav_flush_task(cx);
                    }
                    "down" | "arrowdown" => {
                        let _key_perf = crate::perf::KeyEventPerfGuard::new();
                        match this.nav_coalescer.record(NavDirection::Down) {
                            NavRecord::ApplyImmediate => this.move_selection_down(cx),
                            NavRecord::Coalesced => {}
                            NavRecord::FlushOld { dir, delta } => {
                                if delta != 0 {
                                    this.apply_nav_delta(dir, delta, cx);
                                }
                                this.move_selection_down(cx);
                            }
                        }
                        this.ensure_nav_flush_task(cx);
                    }
                    "enter" => this.execute_selected(cx),
                    "escape" => {
                        if !this.filter_text.is_empty() {
                            this.update_filter(None, false, true, cx);
                        } else {
                            // Update visibility state for hotkey toggle
                            WINDOW_VISIBLE.store(false, Ordering::SeqCst);
                            // Reset UI state before hiding (clears selection, scroll position, filter)
                            logging::log("UI", "Resetting to script list before hiding via Escape");
                            this.reset_to_script_list(cx);
                            logging::log("HOTKEY", "Window hidden via Escape key");
                            // PERF: Measure window hide latency
                            let hide_start = std::time::Instant::now();
                            cx.hide();
                            let hide_elapsed = hide_start.elapsed();
                            logging::log(
                                "PERF",
                                &format!(
                                    "Window hide (Escape) took {:.2}ms",
                                    hide_elapsed.as_secs_f64() * 1000.0
                                ),
                            );
                        }
                    }
                    "backspace" => this.update_filter(None, true, false, cx),
                    "space" | " " => {
                        // Check if current filter text matches an alias
                        // If so, execute the matching script/scriptlet immediately
                        if !this.filter_text.is_empty() {
                            if let Some(alias_match) = this.find_alias_match(&this.filter_text) {
                                logging::log(
                                    "ALIAS",
                                    &format!("Alias '{}' triggered execution", this.filter_text),
                                );
                                match alias_match {
                                    AliasMatch::Script(script) => {
                                        this.execute_interactive(&script, cx);
                                    }
                                    AliasMatch::Scriptlet(scriptlet) => {
                                        this.execute_scriptlet(&scriptlet, cx);
                                    }
                                }
                                // Clear filter after alias execution
                                this.update_filter(None, false, true, cx);
                                return;
                            }
                        }
                        // No alias match - add space to filter as normal character
                        this.update_filter(Some(' '), false, false, cx);
                    }
                    _ => {
                        // Allow all printable characters (not control chars like Tab, Escape)
                        // This enables searching for filenames with special chars like ".ts", ".md"
                        if let Some(ref key_char) = event.keystroke.key_char {
                            if let Some(ch) = key_char.chars().next() {
                                if !ch.is_control() {
                                    this.update_filter(Some(ch), false, false, cx);
                                }
                            }
                        }
                    }
                }
            },
        );

        // Main container with system font and transparency
        // Use theme opacity settings for background transparency
        let opacity = self.theme.get_opacity();

        // Use design tokens for background color (or theme for Default design)
        let bg_hex = if is_default_design {
            theme.colors.background.main
        } else {
            design_colors.background
        };
        let bg_with_alpha = self.hex_to_rgba_with_opacity(bg_hex, opacity.main);

        // Create box shadows from theme
        let box_shadows = self.create_box_shadows();

        // Use design tokens for border radius
        let border_radius = if is_default_design {
            12.0 // Default radius
        } else {
            design_visual.radius_lg
        };

        // Use design tokens for text color
        let text_primary = if is_default_design {
            theme.colors.text.primary
        } else {
            design_colors.text_primary
        };

        // Use design tokens for font family
        let font_family = if is_default_design {
            ".AppleSystemUIFont"
        } else {
            design_typography.font_family
        };

        let mut main_div = div()
            .flex()
            .flex_col()
            .bg(rgba(bg_with_alpha))
            .shadow(box_shadows)
            .w_full()
            .h_full()
            .rounded(px(border_radius))
            .text_color(rgb(text_primary))
            .font_family(font_family)
            .key_context("script_list")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            // Header: Search Input + Run + Actions + Logo
            // Use design tokens for spacing and colors
            .child({
                // Design token values for header
                let header_padding_x = if is_default_design {
                    16.0
                } else {
                    design_spacing.padding_lg
                };
                let header_padding_y = if is_default_design {
                    8.0
                } else {
                    design_spacing.padding_sm
                };
                let header_gap = if is_default_design {
                    12.0
                } else {
                    design_spacing.gap_md
                };
                let text_muted = if is_default_design {
                    theme.colors.text.muted
                } else {
                    design_colors.text_muted
                };
                let text_dimmed = if is_default_design {
                    theme.colors.text.dimmed
                } else {
                    design_colors.text_dimmed
                };
                let accent_color = if is_default_design {
                    theme.colors.accent.selected
                } else {
                    design_colors.accent
                };

                div()
                    .w_full()
                    .px(px(header_padding_x))
                    .py(px(header_padding_y))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(header_gap))
                    // Search input with blinking cursor
                    // Cursor appears at LEFT when input is empty (before placeholder text)
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_row()
                            .items_center()
                            .text_lg()
                            .text_color(if filter_is_empty {
                                rgb(text_muted)
                            } else {
                                rgb(text_primary)
                            })
                            // When empty: cursor FIRST (at left), then placeholder
                            // When typing: text, then cursor at end
                            //
                            // ALIGNMENT FIX: The left cursor (when empty) takes up space
                            // (CURSOR_WIDTH + CURSOR_GAP_X). We apply a negative margin to the
                            // placeholder text to pull it back by that amount, so placeholder
                            // and typed text share the same starting x-position.
                            .when(filter_is_empty, |d| {
                                d.child(
                                    div()
                                        .w(px(CURSOR_WIDTH))
                                        .h(px(CURSOR_HEIGHT_LG))
                                        .my(px(CURSOR_MARGIN_Y))
                                        .mr(px(CURSOR_GAP_X))
                                        .when(
                                            self.focused_input == FocusedInput::MainFilter
                                                && self.cursor_visible,
                                            |d| d.bg(rgb(text_primary)),
                                        ),
                                )
                            })
                            // Display text - with negative margin for placeholder alignment
                            .when(filter_is_empty, |d| {
                                d.child(
                                    div()
                                        .ml(px(-(CURSOR_WIDTH + CURSOR_GAP_X)))
                                        .child(filter_display.clone()),
                                )
                            })
                            .when(!filter_is_empty, |d| d.child(filter_display.clone()))
                            .when(!filter_is_empty, |d| {
                                d.child(
                                    div()
                                        .w(px(CURSOR_WIDTH))
                                        .h(px(CURSOR_HEIGHT_LG))
                                        .my(px(CURSOR_MARGIN_Y))
                                        .ml(px(CURSOR_GAP_X))
                                        .when(
                                            self.focused_input == FocusedInput::MainFilter
                                                && self.cursor_visible,
                                            |d| d.bg(rgb(text_primary)),
                                        ),
                                )
                            }),
                    )
                    // CLS-FREE ACTIONS AREA: Fixed-size relative container with stacked children
                    // Both states are always rendered at the same position, visibility toggled via opacity
                    // This prevents any layout shift when toggling between Run/Actions and search input
                    .child({
                        let button_colors = ButtonColors::from_theme(&self.theme);
                        let handle_run = cx.entity().downgrade();
                        let handle_actions = cx.entity().downgrade();
                        let show_actions = self.show_actions_popup;

                        // Get actions search text from the dialog
                        let search_text = self
                            .actions_dialog
                            .as_ref()
                            .map(|dialog| dialog.read(cx).search_text.clone())
                            .unwrap_or_default();
                        let search_is_empty = search_text.is_empty();
                        let search_display = if search_is_empty {
                            SharedString::from("Search actions...")
                        } else {
                            SharedString::from(search_text.clone())
                        };

                        // Outer container: relative positioned, fixed height to match header
                        div()
                            .relative()
                            .h(px(28.)) // Fixed height to prevent vertical CLS
                            .flex()
                            .items_center()
                            // Run + Actions buttons - absolute positioned, hidden when actions shown
                            .child(
                                div()
                                    .absolute()
                                    .inset_0()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .justify_end()
                                    // Visibility: hidden when actions popup is shown
                                    .when(show_actions, |d| d.opacity(0.).invisible())
                                    // Run button with click handler
                                    .child(
                                        Button::new("Run", button_colors)
                                            .variant(ButtonVariant::Ghost)
                                            .shortcut("↵")
                                            .on_click(Box::new(move |_, _window, cx| {
                                                if let Some(app) = handle_run.upgrade() {
                                                    app.update(cx, |this, cx| {
                                                        this.execute_selected(cx);
                                                    });
                                                }
                                            })),
                                    )
                                    .child(
                                        div()
                                            .mx(px(4.)) // Horizontal margin for spacing
                                            .text_color(rgba((text_dimmed << 8) | 0x60)) // Reduced opacity (60%)
                                            .text_sm() // Slightly smaller text
                                            .child("|"),
                                    )
                                    // Actions button with click handler
                                    .child(
                                        Button::new("Actions", button_colors)
                                            .variant(ButtonVariant::Ghost)
                                            .shortcut("⌘ K")
                                            .on_click(Box::new(move |_, window, cx| {
                                                if let Some(app) = handle_actions.upgrade() {
                                                    app.update(cx, |this, cx| {
                                                        this.toggle_actions(cx, window);
                                                    });
                                                }
                                            })),
                                    )
                                    .child(
                                        div()
                                            .mx(px(4.)) // Horizontal margin for spacing
                                            .text_color(rgba((text_dimmed << 8) | 0x60)) // Reduced opacity (60%)
                                            .text_sm() // Slightly smaller text
                                            .child("|"),
                                    ),
                            )
                            // Actions search input - absolute positioned, visible when actions shown
                            .child(
                                div()
                                    .absolute()
                                    .inset_0()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .justify_end()
                                    .gap(px(8.))
                                    // Visibility: hidden when actions popup is NOT shown
                                    .when(!show_actions, |d| d.opacity(0.).invisible())
                                    // ⌘K indicator
                                    .child(div().text_color(rgb(text_dimmed)).text_xs().child("⌘K"))
                                    // Search input display - compact style matching buttons
                                    // CRITICAL: Fixed width prevents resize when typing
                                    .child(
                                        div()
                                            .flex_shrink_0() // PREVENT flexbox from shrinking this
                                            .w(px(130.0)) // Compact width
                                            .min_w(px(130.0))
                                            .max_w(px(130.0))
                                            .h(px(24.0)) // Comfortable height with padding
                                            .min_h(px(24.0))
                                            .max_h(px(24.0))
                                            .overflow_hidden()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .px(px(8.)) // Comfortable horizontal padding
                                            .rounded(px(4.)) // Match button border radius
                                            // ALWAYS show background - just vary intensity
                                            .bg(rgba(
                                                (theme.colors.background.search_box << 8)
                                                    | if search_is_empty { 0x40 } else { 0x80 },
                                            ))
                                            .border_1()
                                            // ALWAYS show border - just vary intensity
                                            .border_color(rgba(
                                                (accent_color << 8)
                                                    | if search_is_empty { 0x20 } else { 0x40 },
                                            ))
                                            .text_sm()
                                            .text_color(if search_is_empty {
                                                rgb(text_muted)
                                            } else {
                                                rgb(text_primary)
                                            })
                                            // Cursor before placeholder when empty
                                            .when(search_is_empty, |d| {
                                                d.child(
                                                    div()
                                                        .w(px(2.))
                                                        .h(px(14.)) // Cursor height for comfortable input
                                                        .mr(px(2.))
                                                        .rounded(px(1.))
                                                        .when(
                                                            self.focused_input
                                                                == FocusedInput::ActionsSearch
                                                                && self.cursor_visible,
                                                            |d| d.bg(rgb(accent_color)),
                                                        ),
                                                )
                                            })
                                            .child(search_display)
                                            // Cursor after text when not empty
                                            .when(!search_is_empty, |d| {
                                                d.child(
                                                    div()
                                                        .w(px(2.))
                                                        .h(px(14.)) // Cursor height for comfortable input
                                                        .ml(px(2.))
                                                        .rounded(px(1.))
                                                        .when(
                                                            self.focused_input
                                                                == FocusedInput::ActionsSearch
                                                                && self.cursor_visible,
                                                            |d| d.bg(rgb(accent_color)),
                                                        ),
                                                )
                                            }),
                                    )
                                    .child(
                                        div()
                                            .mx(px(4.)) // Horizontal margin for spacing
                                            .text_color(rgba((text_dimmed << 8) | 0x60)) // Reduced opacity (60%)
                                            .text_sm() // Slightly smaller text
                                            .child("|"),
                                    ),
                            )
                    })
                    // Script Kit Logo - ALWAYS visible
                    // Size slightly larger than text for visual presence
                    .child(
                        svg()
                            .external_path(utils::get_logo_path())
                            .size(px(16.)) // Slightly larger than text_sm for visual presence
                            .text_color(rgb(accent_color)),
                    )
            })
            // Subtle divider - semi-transparent
            // Use design tokens for border color and spacing
            .child({
                let divider_margin = if is_default_design {
                    16.0
                } else {
                    design_spacing.margin_lg
                };
                let border_color = if is_default_design {
                    theme.colors.ui.border
                } else {
                    design_colors.border
                };
                let border_width = if is_default_design {
                    1.0
                } else {
                    design_visual.border_thin
                };

                div()
                    .mx(px(divider_margin))
                    .h(px(border_width))
                    .bg(rgba((border_color << 8) | 0x60))
            });

        // Main content area - 50/50 split: List on left, Preview on right
        main_div = main_div
            // Uses min_h(px(0.)) to prevent flex children from overflowing
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .min_h(px(0.)) // Critical: allows flex container to shrink properly
                    .w_full()
                    .overflow_hidden()
                    // Left side: Script list (50% width) - uses uniform_list for auto-scrolling
                    .child(
                        div()
                            .w_1_2() // 50% width
                            .h_full() // Take full height
                            .min_h(px(0.)) // Allow shrinking
                            .child(list_element),
                    )
                    // Right side: Preview panel (50% width) with actions overlay
                    // Preview ALWAYS renders, actions panel overlays on top when visible
                    .child(
                        div()
                            .relative() // Enable absolute positioning for overlay
                            .w_1_2() // 50% width
                            .h_full() // Take full height
                            .min_h(px(0.)) // Allow shrinking
                            .overflow_hidden()
                            // Preview panel ALWAYS renders (visible behind actions overlay)
                            .child(self.render_preview_panel(cx))
                            // Actions dialog overlays on top using absolute positioning
                            // Includes a backdrop to capture clicks outside the dialog
                            .when_some(
                                if self.show_actions_popup {
                                    self.actions_dialog.clone()
                                } else {
                                    None
                                },
                                |d, dialog| {
                                    // Create click handler for backdrop to dismiss dialog
                                    let backdrop_click = cx.listener(|this: &mut Self, _event: &gpui::ClickEvent, window: &mut Window, cx: &mut Context<Self>| {
                                        logging::log("FOCUS", "Actions backdrop clicked - dismissing dialog");
                                        this.show_actions_popup = false;
                                        this.actions_dialog = None;
                                        this.focused_input = FocusedInput::MainFilter;
                                        window.focus(&this.focus_handle, cx);
                                        cx.notify();
                                    });

                                    d.child(
                                        div()
                                            .absolute()
                                            .inset_0() // Cover entire preview area
                                            // Backdrop layer - captures clicks outside the dialog
                                            .child(
                                                div()
                                                    .id("actions-backdrop")
                                                    .absolute()
                                                    .inset_0()
                                                    .on_click(backdrop_click)
                                            )
                                            // Dialog container - positioned at top-right
                                            .child(
                                                div()
                                                    .absolute()
                                                    .inset_0()
                                                    .flex()
                                                    .justify_end()
                                                    .pr(px(8.)) // Small padding from right edge
                                                    .pt(px(8.)) // Small padding from top
                                                    .child(dialog),
                                            ),
                                    )
                                },
                            ),
                    ),
            );

        if let Some(panel) = log_panel {
            main_div = main_div.child(panel);
        }

        // Wrap in relative container for toast overlay positioning
        let mut container = div().relative().w_full().h_full().child(main_div);

        // Add toast notifications overlay (top-right)
        if let Some(toasts) = self.render_toasts(cx) {
            container = container.child(toasts);
        }

        // Note: HUD overlay is added at the top-level render() method for all views

        container.into_any_element()
    }

    fn render_arg_prompt(
        &mut self,
        id: String,
        placeholder: String,
        choices: Vec<Choice>,
        actions: Option<Vec<ProtocolAction>>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let _theme = &self.theme;
        let _filtered = self.filtered_arg_choices();
        let has_actions = actions.is_some() && !actions.as_ref().unwrap().is_empty();
        let has_choices = !choices.is_empty();

        // Use design tokens for GLOBAL theming - all prompts use current design
        let tokens = get_tokens(self.current_design);
        let design_colors = tokens.colors();
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();
        let design_visual = tokens.visual();

        // Key handler for arg prompt
        let prompt_id = id.clone();
        let has_actions_for_handler = has_actions;
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                let key_str = event.keystroke.key.to_lowercase();
                let has_cmd = event.keystroke.modifiers.platform;
                logging::log(
                    "KEY",
                    &format!("ArgPrompt key: '{}' cmd={}", key_str, has_cmd),
                );

                // Check for Cmd+K to toggle actions popup (if actions are available)
                if has_cmd && key_str == "k" && has_actions_for_handler {
                    logging::log("KEY", "Cmd+K in ArgPrompt - calling toggle_arg_actions");
                    this.toggle_arg_actions(cx, window);
                    return;
                }

                // If actions popup is open, route keyboard events to it (same as main menu)
                if this.show_actions_popup {
                    if let Some(ref dialog) = this.actions_dialog {
                        match key_str.as_str() {
                            "up" | "arrowup" => {
                                dialog.update(cx, |d, cx| d.move_up(cx));
                                return;
                            }
                            "down" | "arrowdown" => {
                                dialog.update(cx, |d, cx| d.move_down(cx));
                                return;
                            }
                            "enter" => {
                                // Get the selected action and execute it
                                let action_id = dialog.read(cx).get_selected_action_id();
                                let should_close = dialog.read(cx).selected_action_should_close();
                                if let Some(action_id) = action_id {
                                    logging::log(
                                        "ACTIONS",
                                        &format!(
                                            "ArgPrompt executing action: {} (close={})",
                                            action_id, should_close
                                        ),
                                    );
                                    // Only close if action has close: true (default)
                                    if should_close {
                                        this.show_actions_popup = false;
                                        this.actions_dialog = None;
                                        this.focused_input = FocusedInput::ArgPrompt;
                                        window.focus(&this.focus_handle, cx);
                                    }
                                    // Trigger the SDK action by name
                                    this.trigger_action_by_name(&action_id, cx);
                                }
                                return;
                            }
                            "escape" => {
                                this.show_actions_popup = false;
                                this.actions_dialog = None;
                                this.focused_input = FocusedInput::ArgPrompt;
                                window.focus(&this.focus_handle, cx);
                                cx.notify();
                                return;
                            }
                            "backspace" => {
                                dialog.update(cx, |d, cx| d.handle_backspace(cx));
                                return;
                            }
                            _ => {
                                // Route character input to the dialog for search
                                if let Some(ref key_char) = event.keystroke.key_char {
                                    if let Some(ch) = key_char.chars().next() {
                                        if !ch.is_control() {
                                            dialog.update(cx, |d, cx| d.handle_char(ch, cx));
                                        }
                                    }
                                }
                                return;
                            }
                        }
                    }
                }

                // Check for SDK action shortcuts (only when actions popup is NOT open)
                let shortcut_key =
                    shortcuts::keystroke_to_shortcut(&key_str, &event.keystroke.modifiers);
                if let Some(action_name) = this.action_shortcuts.get(&shortcut_key).cloned() {
                    logging::log(
                        "KEY",
                        &format!("SDK action shortcut matched: {}", action_name),
                    );
                    this.trigger_action_by_name(&action_name, cx);
                    return;
                }

                match key_str.as_str() {
                    "up" | "arrowup" => {
                        if this.arg_selected_index > 0 {
                            this.arg_selected_index -= 1;
                            // P0: Scroll to keep selection visible
                            this.arg_list_scroll_handle
                                .scroll_to_item(this.arg_selected_index, ScrollStrategy::Nearest);
                            logging::log_debug(
                                "SCROLL",
                                &format!("P0: Arg up: selected_index={}", this.arg_selected_index),
                            );
                            cx.notify();
                        }
                    }
                    "down" | "arrowdown" => {
                        let filtered = this.filtered_arg_choices();
                        if this.arg_selected_index < filtered.len().saturating_sub(1) {
                            this.arg_selected_index += 1;
                            // P0: Scroll to keep selection visible
                            this.arg_list_scroll_handle
                                .scroll_to_item(this.arg_selected_index, ScrollStrategy::Nearest);
                            logging::log_debug(
                                "SCROLL",
                                &format!(
                                    "P0: Arg down: selected_index={}",
                                    this.arg_selected_index
                                ),
                            );
                            cx.notify();
                        }
                    }
                    "enter" => {
                        let filtered = this.filtered_arg_choices();
                        if let Some((_, choice)) = filtered.get(this.arg_selected_index) {
                            // Case 1: There are filtered choices - submit the selected one
                            let value = choice.value.clone();
                            this.submit_prompt_response(prompt_id.clone(), Some(value), cx);
                        } else if !this.arg_input_text.is_empty() {
                            // Case 2: No choices but user typed something - submit input_text
                            let value = this.arg_input_text.clone();
                            this.submit_prompt_response(prompt_id.clone(), Some(value), cx);
                        }
                        // Case 3: No choices and no input - do nothing (prevent empty submissions)
                    }
                    "escape" => {
                        logging::log("KEY", "ESC in ArgPrompt - canceling script");
                        // Send cancel response and clean up fully
                        this.submit_prompt_response(prompt_id.clone(), None, cx);
                        this.cancel_script_execution(cx);
                    }
                    "backspace" => {
                        if !this.arg_input_text.is_empty() {
                            this.arg_input_text.pop();
                            this.arg_selected_index = 0;
                            this.update_window_size();
                            cx.notify();
                        }
                    }
                    _ => {
                        if let Some(ref key_char) = event.keystroke.key_char {
                            if let Some(ch) = key_char.chars().next() {
                                if !ch.is_control() {
                                    this.arg_input_text.push(ch);
                                    this.arg_selected_index = 0;
                                    this.update_window_size();
                                    cx.notify();
                                }
                            }
                        }
                    }
                }
            },
        );

        let input_display = if self.arg_input_text.is_empty() {
            SharedString::from(placeholder.clone())
        } else {
            SharedString::from(self.arg_input_text.clone())
        };
        let input_is_empty = self.arg_input_text.is_empty();

        // P4: Pre-compute theme values for arg prompt using design tokens for GLOBAL theming
        let arg_list_colors = ListItemColors::from_design(&design_colors);
        let text_primary = design_colors.text_primary;
        let text_muted = design_colors.text_muted;

        // P0: Clone data needed for uniform_list closure
        let arg_selected_index = self.arg_selected_index;
        let filtered_choices = self.get_filtered_arg_choices_owned();
        let filtered_choices_len = filtered_choices.len();
        logging::log_debug(
            "UI",
            &format!(
                "P0: Arg prompt has {} filtered choices",
                filtered_choices_len
            ),
        );

        // P0: Build virtualized choice list using uniform_list
        let list_element: AnyElement = if filtered_choices_len == 0 {
            div()
                .w_full()
                .py(px(design_spacing.padding_xl))
                .text_center()
                .text_color(rgb(design_colors.text_muted))
                .font_family(design_typography.font_family)
                .child("No choices match your filter")
                .into_any_element()
        } else {
            // P0: Use uniform_list for virtualized scrolling of arg choices
            // Now uses shared ListItem component for consistent design with script list
            uniform_list(
                "arg-choices",
                filtered_choices_len,
                move |visible_range, _window, _cx| {
                    logging::log_debug(
                        "SCROLL",
                        &format!("P0: Arg choices visible range: {:?}", visible_range.clone()),
                    );
                    visible_range
                        .map(|ix| {
                            if let Some((_, choice)) = filtered_choices.get(ix) {
                                let is_selected = ix == arg_selected_index;

                                // Use shared ListItem component for consistent design
                                div().id(ix).child(
                                    ListItem::new(choice.name.clone(), arg_list_colors)
                                        .description_opt(choice.description.clone())
                                        .selected(is_selected)
                                        .with_accent_bar(true)
                                        .index(ix),
                                )
                            } else {
                                div().id(ix).h(px(LIST_ITEM_HEIGHT))
                            }
                        })
                        .collect()
                },
            )
            .h_full()
            .track_scroll(&self.arg_list_scroll_handle)
            .into_any_element()
        };

        // Use design tokens for global theming
        let opacity = self.theme.get_opacity();
        let bg_hex = design_colors.background;
        let bg_with_alpha = self.hex_to_rgba_with_opacity(bg_hex, opacity.main);
        let box_shadows = self.create_box_shadows();

        // P4: Pre-compute more theme values for the main container using design tokens
        let ui_border = design_colors.border;
        let text_dimmed = design_colors.text_dimmed;

        div()
            .relative() // Needed for absolute positioned actions dialog overlay
            .flex()
            .flex_col()
            .bg(rgba(bg_with_alpha))
            .shadow(box_shadows)
            .w_full()
            .h_full()
            .rounded(px(design_visual.radius_lg))
            .text_color(rgb(text_primary))
            .font_family(design_typography.font_family)
            .key_context("arg_prompt")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            // Header with input
            .child(
                div()
                    .w_full()
                    .px(px(design_spacing.padding_lg))
                    .py(px(design_spacing.padding_md))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    // Search input with blinking cursor (same as main menu)
                    // Cursor appears at LEFT when input is empty (before placeholder text)
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_row()
                            .items_center()
                            .text_lg()
                            .text_color(if input_is_empty {
                                rgb(text_muted)
                            } else {
                                rgb(text_primary)
                            })
                            // When empty: cursor FIRST (at left), then placeholder
                            // When typing: text, then cursor at end
                            //
                            // ALIGNMENT FIX: The left cursor (when empty) takes up space
                            // (CURSOR_WIDTH + CURSOR_GAP_X). We apply a negative margin to the
                            // placeholder text to pull it back by that amount.
                            .when(input_is_empty, |d| {
                                d.child(
                                    div()
                                        .w(px(CURSOR_WIDTH))
                                        .h(px(CURSOR_HEIGHT_LG))
                                        .my(px(CURSOR_MARGIN_Y))
                                        .mr(px(CURSOR_GAP_X))
                                        .when(
                                            self.focused_input == FocusedInput::ArgPrompt
                                                && self.cursor_visible,
                                            |d| d.bg(rgb(text_primary)),
                                        ),
                                )
                            })
                            // Display text - with negative margin for placeholder alignment
                            .when(input_is_empty, |d| {
                                d.child(
                                    div()
                                        .ml(px(-(CURSOR_WIDTH + CURSOR_GAP_X)))
                                        .child(input_display.clone()),
                                )
                            })
                            .when(!input_is_empty, |d| d.child(input_display.clone()))
                            .when(!input_is_empty, |d| {
                                d.child(
                                    div()
                                        .w(px(CURSOR_WIDTH))
                                        .h(px(CURSOR_HEIGHT_LG))
                                        .my(px(CURSOR_MARGIN_Y))
                                        .ml(px(CURSOR_GAP_X))
                                        .when(
                                            self.focused_input == FocusedInput::ArgPrompt
                                                && self.cursor_visible,
                                            |d| d.bg(rgb(text_primary)),
                                        ),
                                )
                            }),
                    )
                    // CLS-FREE ACTIONS AREA: Matches main menu pattern exactly
                    // Both states are always rendered at the same position, visibility toggled via opacity
                    .when(has_actions, |d| {
                        let button_colors = ButtonColors::from_theme(&self.theme);
                        let handle_actions = cx.entity().downgrade();
                        let show_actions = self.show_actions_popup;

                        // Get actions search text from the dialog
                        let search_text = self
                            .actions_dialog
                            .as_ref()
                            .map(|dialog| dialog.read(cx).search_text.clone())
                            .unwrap_or_default();
                        let search_is_empty = search_text.is_empty();
                        let search_display: SharedString = if search_is_empty {
                            "Search actions...".into()
                        } else {
                            search_text.into()
                        };
                        let accent_color = design_colors.accent;
                        let search_box_bg = self.theme.colors.background.search_box;
                        let cursor_visible_for_search = self.focused_input
                            == FocusedInput::ActionsSearch
                            && self.cursor_visible;

                        d.child(
                            div()
                                .relative()
                                .h(px(28.)) // Fixed height to prevent vertical CLS
                                .flex()
                                .items_center()
                                // Layer 1: Actions button - visible when NOT showing actions search
                                .child(
                                    div()
                                        .absolute()
                                        .inset_0()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .justify_end()
                                        .when(show_actions, |d| d.opacity(0.).invisible())
                                        .child(
                                            Button::new("Actions", button_colors)
                                                .variant(ButtonVariant::Ghost)
                                                .shortcut("⌘ K")
                                                .on_click(Box::new(move |_, window, cx| {
                                                    if let Some(app) = handle_actions.upgrade() {
                                                        app.update(cx, |this, cx| {
                                                            this.toggle_arg_actions(cx, window);
                                                        });
                                                    }
                                                })),
                                        )
                                        .child(
                                            div()
                                                .mx(px(4.))
                                                .text_color(rgba((text_dimmed << 8) | 0x60))
                                                .text_sm()
                                                .child("|"),
                                        ),
                                )
                                // Layer 2: Actions search input - visible when showing actions search
                                .child(
                                    div()
                                        .absolute()
                                        .inset_0()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .justify_end()
                                        .gap(px(8.))
                                        .when(!show_actions, |d| d.opacity(0.).invisible())
                                        // ⌘K indicator
                                        .child(
                                            div()
                                                .text_color(rgb(text_dimmed))
                                                .text_xs()
                                                .child("⌘K"),
                                        )
                                        // Search input display - compact style matching buttons
                                        .child(
                                            div()
                                                .id("arg-actions-search")
                                                .flex_shrink_0()
                                                .w(px(130.))
                                                .min_w(px(130.))
                                                .max_w(px(130.))
                                                .h(px(24.))
                                                .min_h(px(24.))
                                                .max_h(px(24.))
                                                .overflow_hidden()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .px(px(8.))
                                                .rounded(px(4.))
                                                .bg(rgba(
                                                    (search_box_bg << 8)
                                                        | if search_is_empty { 0x40 } else { 0x80 },
                                                ))
                                                .border_1()
                                                .border_color(rgba(
                                                    (accent_color << 8)
                                                        | if search_is_empty { 0x20 } else { 0x40 },
                                                ))
                                                .text_sm()
                                                .text_color(if search_is_empty {
                                                    rgb(text_muted)
                                                } else {
                                                    rgb(text_primary)
                                                })
                                                // Cursor before placeholder when empty
                                                .when(search_is_empty, |d| {
                                                    d.child(
                                                        div()
                                                            .w(px(2.))
                                                            .h(px(14.))
                                                            .mr(px(2.))
                                                            .rounded(px(1.))
                                                            .when(cursor_visible_for_search, |d| {
                                                                d.bg(rgb(accent_color))
                                                            }),
                                                    )
                                                })
                                                .child(search_display)
                                                // Cursor after text when not empty
                                                .when(!search_is_empty, |d| {
                                                    d.child(
                                                        div()
                                                            .w(px(2.))
                                                            .h(px(14.))
                                                            .ml(px(2.))
                                                            .rounded(px(1.))
                                                            .when(cursor_visible_for_search, |d| {
                                                                d.bg(rgb(accent_color))
                                                            }),
                                                    )
                                                }),
                                        )
                                        .child(
                                            div()
                                                .mx(px(4.))
                                                .text_color(rgba((text_dimmed << 8) | 0x60))
                                                .text_sm()
                                                .child("|"),
                                        ),
                                ),
                        )
                    })
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(text_dimmed))
                            .child(format!("{} choices", choices.len())),
                    ),
            )
            // Choices list (only when prompt has choices)
            .when(has_choices, |d| {
                d.child(
                    div()
                        .mx(px(design_spacing.padding_lg))
                        .h(px(design_visual.border_thin))
                        .bg(rgba((ui_border << 8) | 0x60)),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .min_h(px(0.)) // P0: Allow flex container to shrink
                        .w_full()
                        .py(px(design_spacing.padding_xs))
                        .child(list_element),
                )
            })
            // Actions dialog overlay (when Cmd+K is pressed with SDK actions)
            // Uses same pattern as main menu: check BOTH show_actions_popup AND actions_dialog
            .when_some(
                if self.show_actions_popup {
                    self.actions_dialog.clone()
                } else {
                    None
                },
                |d, dialog| {
                    // Create click handler for backdrop to dismiss dialog
                    let backdrop_click = cx.listener(
                        |this: &mut Self,
                         _event: &gpui::ClickEvent,
                         window: &mut Window,
                         cx: &mut Context<Self>| {
                            logging::log(
                                "FOCUS",
                                "Arg actions backdrop clicked - dismissing dialog",
                            );
                            this.show_actions_popup = false;
                            this.actions_dialog = None;
                            this.focused_input = FocusedInput::ArgPrompt;
                            window.focus(&this.focus_handle, cx);
                            cx.notify();
                        },
                    );

                    d.child(
                        div()
                            .absolute()
                            .inset_0() // Cover entire arg prompt area
                            // Backdrop layer - captures clicks outside the dialog
                            .child(
                                div()
                                    .id("arg-actions-backdrop")
                                    .absolute()
                                    .inset_0()
                                    .on_click(backdrop_click),
                            )
                            // Dialog positioned at top-right
                            .child(
                                div()
                                    .absolute()
                                    .top(px(52.)) // Clear the header bar (~44px header + 8px margin)
                                    .right(px(8.))
                                    .child(dialog),
                            ),
                    )
                },
            )
            .into_any_element()
    }

    fn render_div_prompt(
        &mut self,
        entity: Entity<DivPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let has_actions =
            self.sdk_actions.is_some() && !self.sdk_actions.as_ref().unwrap().is_empty();

        // Use design tokens for GLOBAL theming
        let tokens = get_tokens(self.current_design);
        let design_colors = tokens.colors();
        let design_spacing = tokens.spacing();
        let design_visual = tokens.visual();

        // Key handler for Cmd+K actions toggle (at parent level to intercept before DivPrompt)
        let has_actions_for_handler = has_actions;
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                let key_str = event.keystroke.key.to_lowercase();
                let has_cmd = event.keystroke.modifiers.platform;

                // Check for Cmd+K to toggle actions popup (if actions are available)
                if has_cmd && key_str == "k" && has_actions_for_handler {
                    logging::log("KEY", "Cmd+K in DivPrompt - calling toggle_arg_actions");
                    this.toggle_arg_actions(cx, window);
                    return;
                }

                // If actions popup is open, route keyboard events to it
                if this.show_actions_popup {
                    if let Some(ref dialog) = this.actions_dialog {
                        match key_str.as_str() {
                            "up" | "arrowup" => {
                                dialog.update(cx, |d, cx| d.move_up(cx));
                            }
                            "down" | "arrowdown" => {
                                dialog.update(cx, |d, cx| d.move_down(cx));
                            }
                            "enter" => {
                                let action_id = dialog.read(cx).get_selected_action_id();
                                let should_close = dialog.read(cx).selected_action_should_close();
                                if let Some(action_id) = action_id {
                                    logging::log(
                                        "ACTIONS",
                                        &format!(
                                            "DivPrompt executing action: {} (close={})",
                                            action_id, should_close
                                        ),
                                    );
                                    if should_close {
                                        this.show_actions_popup = false;
                                        this.actions_dialog = None;
                                        this.focused_input = FocusedInput::None;
                                        window.focus(&this.focus_handle, cx);
                                    }
                                    this.trigger_action_by_name(&action_id, cx);
                                }
                            }
                            "escape" => {
                                this.show_actions_popup = false;
                                this.actions_dialog = None;
                                this.focused_input = FocusedInput::None;
                                window.focus(&this.focus_handle, cx);
                                cx.notify();
                            }
                            "backspace" => {
                                dialog.update(cx, |d, cx| d.handle_backspace(cx));
                            }
                            _ => {
                                if let Some(ref key_char) = event.keystroke.key_char {
                                    if let Some(ch) = key_char.chars().next() {
                                        if !ch.is_control() {
                                            dialog.update(cx, |d, cx| d.handle_char(ch, cx));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // SDK action shortcuts are handled by DivPrompt's own key handler
            },
        );

        // Use design tokens for global theming
        let opacity = self.theme.get_opacity();
        let bg_hex = design_colors.background;
        let bg_with_alpha = self.hex_to_rgba_with_opacity(bg_hex, opacity.main);
        let box_shadows = self.create_box_shadows();

        // Use explicit height from layout constants
        let content_height = window_resize::layout::STANDARD_HEIGHT;

        div()
            .relative() // Needed for absolute positioned actions dialog overlay
            .flex()
            .flex_col()
            .bg(rgba(bg_with_alpha))
            .shadow(box_shadows)
            .w_full()
            .h(content_height)
            .overflow_hidden()
            .rounded(px(design_visual.radius_lg))
            .track_focus(&self.focus_handle) // Required to receive key events
            .on_key_down(handle_key)
            // CLS-FREE ACTIONS AREA: Matches main menu and ArgPrompt pattern
            .when(has_actions, |d| {
                let button_colors = ButtonColors::from_theme(&self.theme);
                let handle_actions = cx.entity().downgrade();
                let show_actions = self.show_actions_popup;

                // Get actions search text from the dialog
                let search_text = self
                    .actions_dialog
                    .as_ref()
                    .map(|dialog| dialog.read(cx).search_text.clone())
                    .unwrap_or_default();
                let search_is_empty = search_text.is_empty();
                let search_display: SharedString = if search_is_empty {
                    "Search actions...".into()
                } else {
                    search_text.into()
                };
                let accent_color = design_colors.accent;
                let text_primary = design_colors.text_primary;
                let text_muted = design_colors.text_muted;
                let text_dimmed = design_colors.text_dimmed;
                let search_box_bg = self.theme.colors.background.search_box;
                let cursor_visible_for_search =
                    self.focused_input == FocusedInput::ActionsSearch && self.cursor_visible;

                d.child(
                    div()
                        .w_full()
                        .px(px(design_spacing.padding_lg))
                        .py(px(design_spacing.padding_md))
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_end()
                        .child(
                            div()
                                .relative()
                                .h(px(28.))
                                .flex()
                                .items_center()
                                // Layer 1: Actions button - visible when NOT showing actions search
                                .child(
                                    div()
                                        .absolute()
                                        .inset_0()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .justify_end()
                                        .when(show_actions, |d| d.opacity(0.).invisible())
                                        .child(
                                            Button::new("Actions", button_colors)
                                                .variant(ButtonVariant::Ghost)
                                                .shortcut("⌘ K")
                                                .on_click(Box::new(move |_, window, cx| {
                                                    if let Some(app) = handle_actions.upgrade() {
                                                        app.update(cx, |this, cx| {
                                                            this.toggle_arg_actions(cx, window);
                                                        });
                                                    }
                                                })),
                                        )
                                        .child(
                                            div()
                                                .mx(px(4.))
                                                .text_color(rgba((text_dimmed << 8) | 0x60))
                                                .text_sm()
                                                .child("|"),
                                        ),
                                )
                                // Layer 2: Actions search input - visible when showing actions search
                                .child(
                                    div()
                                        .absolute()
                                        .inset_0()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .justify_end()
                                        .gap(px(8.))
                                        .when(!show_actions, |d| d.opacity(0.).invisible())
                                        .child(
                                            div()
                                                .text_color(rgb(text_dimmed))
                                                .text_xs()
                                                .child("⌘K"),
                                        )
                                        .child(
                                            div()
                                                .id("div-actions-search")
                                                .flex_shrink_0()
                                                .w(px(130.))
                                                .min_w(px(130.))
                                                .max_w(px(130.))
                                                .h(px(24.))
                                                .min_h(px(24.))
                                                .max_h(px(24.))
                                                .overflow_hidden()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .px(px(8.))
                                                .rounded(px(4.))
                                                .bg(rgba(
                                                    (search_box_bg << 8)
                                                        | if search_is_empty { 0x40 } else { 0x80 },
                                                ))
                                                .border_1()
                                                .border_color(rgba(
                                                    (accent_color << 8)
                                                        | if search_is_empty { 0x20 } else { 0x40 },
                                                ))
                                                .text_sm()
                                                .text_color(if search_is_empty {
                                                    rgb(text_muted)
                                                } else {
                                                    rgb(text_primary)
                                                })
                                                .when(search_is_empty, |d| {
                                                    d.child(
                                                        div()
                                                            .w(px(2.))
                                                            .h(px(14.))
                                                            .mr(px(2.))
                                                            .rounded(px(1.))
                                                            .when(cursor_visible_for_search, |d| {
                                                                d.bg(rgb(accent_color))
                                                            }),
                                                    )
                                                })
                                                .child(search_display)
                                                .when(!search_is_empty, |d| {
                                                    d.child(
                                                        div()
                                                            .w(px(2.))
                                                            .h(px(14.))
                                                            .ml(px(2.))
                                                            .rounded(px(1.))
                                                            .when(cursor_visible_for_search, |d| {
                                                                d.bg(rgb(accent_color))
                                                            }),
                                                    )
                                                }),
                                        )
                                        .child(
                                            div()
                                                .mx(px(4.))
                                                .text_color(rgba((text_dimmed << 8) | 0x60))
                                                .text_sm()
                                                .child("|"),
                                        ),
                                ),
                        ),
                )
            })
            // Content area - render the DivPrompt entity which handles HTML parsing and rendering
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .min_h(px(0.)) // Critical: allows flex children to size properly
                    .overflow_hidden()
                    .child(entity.clone()),
            )
            // Actions dialog overlay (when Cmd+K is pressed with SDK actions)
            .when_some(
                if self.show_actions_popup {
                    self.actions_dialog.clone()
                } else {
                    None
                },
                |d, dialog| {
                    let backdrop_click = cx.listener(
                        |this: &mut Self,
                         _event: &gpui::ClickEvent,
                         window: &mut Window,
                         cx: &mut Context<Self>| {
                            logging::log(
                                "FOCUS",
                                "Div actions backdrop clicked - dismissing dialog",
                            );
                            this.show_actions_popup = false;
                            this.actions_dialog = None;
                            this.focused_input = FocusedInput::None;
                            window.focus(&this.focus_handle, cx);
                            cx.notify();
                        },
                    );

                    d.child(
                        div()
                            .absolute()
                            .inset_0()
                            .child(
                                div()
                                    .id("div-actions-backdrop")
                                    .absolute()
                                    .inset_0()
                                    .on_click(backdrop_click),
                            )
                            .child(div().absolute().top(px(52.)).right(px(8.)).child(dialog)),
                    )
                },
            )
            .into_any_element()
    }

    fn render_form_prompt(
        &mut self,
        entity: Entity<FormPromptState>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let has_actions =
            self.sdk_actions.is_some() && !self.sdk_actions.as_ref().unwrap().is_empty();

        // Use design tokens for GLOBAL theming
        let tokens = get_tokens(self.current_design);
        let design_colors = tokens.colors();
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();
        let design_visual = tokens.visual();

        // Get prompt ID and field count from entity
        let prompt_id = entity.read(cx).id.clone();
        let field_count = entity.read(cx).fields.len();

        // Clone entity for closures
        let entity_for_submit = entity.clone();
        let entity_for_tab = entity.clone();
        let entity_for_shift_tab = entity.clone();
        let entity_for_input = entity.clone();

        let prompt_id_for_key = prompt_id.clone();
        // Key handler for form navigation (Enter/Tab/Escape) and Cmd+K actions
        let has_actions_for_handler = has_actions;
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                let key_str = event.keystroke.key.to_lowercase();
                let has_shift = event.keystroke.modifiers.shift;
                let has_cmd = event.keystroke.modifiers.platform;

                logging::log(
                    "KEY",
                    &format!(
                        "FormPrompt key: '{}' (shift: {}, cmd: {}, key_char: {:?})",
                        key_str, has_shift, has_cmd, event.keystroke.key_char
                    ),
                );

                // Check for Cmd+K to toggle actions popup (if actions are available)
                if has_cmd && key_str == "k" && has_actions_for_handler {
                    logging::log("KEY", "Cmd+K in FormPrompt - calling toggle_arg_actions");
                    this.toggle_arg_actions(cx, window);
                    return;
                }

                // If actions popup is open, route keyboard events to it
                if this.show_actions_popup {
                    if let Some(ref dialog) = this.actions_dialog {
                        match key_str.as_str() {
                            "up" | "arrowup" => {
                                dialog.update(cx, |d, cx| d.move_up(cx));
                                return;
                            }
                            "down" | "arrowdown" => {
                                dialog.update(cx, |d, cx| d.move_down(cx));
                                return;
                            }
                            "enter" => {
                                let action_id = dialog.read(cx).get_selected_action_id();
                                let should_close = dialog.read(cx).selected_action_should_close();
                                if let Some(action_id) = action_id {
                                    logging::log(
                                        "ACTIONS",
                                        &format!(
                                            "FormPrompt executing action: {} (close={})",
                                            action_id, should_close
                                        ),
                                    );
                                    if should_close {
                                        this.show_actions_popup = false;
                                        this.actions_dialog = None;
                                        this.focused_input = FocusedInput::None;
                                        window.focus(&this.focus_handle, cx);
                                    }
                                    this.trigger_action_by_name(&action_id, cx);
                                }
                                return;
                            }
                            "escape" => {
                                this.show_actions_popup = false;
                                this.actions_dialog = None;
                                this.focused_input = FocusedInput::None;
                                window.focus(&this.focus_handle, cx);
                                cx.notify();
                                return;
                            }
                            "backspace" => {
                                dialog.update(cx, |d, cx| d.handle_backspace(cx));
                                return;
                            }
                            _ => {
                                if let Some(ref key_char) = event.keystroke.key_char {
                                    if let Some(ch) = key_char.chars().next() {
                                        if !ch.is_control() {
                                            dialog.update(cx, |d, cx| d.handle_char(ch, cx));
                                        }
                                    }
                                }
                                return;
                            }
                        }
                    }
                }

                // Check for SDK action shortcuts
                let shortcut_key =
                    shortcuts::keystroke_to_shortcut(&key_str, &event.keystroke.modifiers);
                if let Some(action_name) = this.action_shortcuts.get(&shortcut_key).cloned() {
                    logging::log(
                        "KEY",
                        &format!("SDK action shortcut matched: {}", action_name),
                    );
                    this.trigger_action_by_name(&action_name, cx);
                    return;
                }

                // Handle form-level keys (Enter, Escape, Tab) at this level
                // Forward all other keys to the focused form field for text input
                match key_str.as_str() {
                    "enter" => {
                        // Enter submits the form - collect all field values
                        logging::log("KEY", "Enter in FormPrompt - submitting form");
                        let values = entity_for_submit.read(cx).collect_values(cx);
                        logging::log("FORM", &format!("Form values: {}", values));
                        this.submit_prompt_response(prompt_id_for_key.clone(), Some(values), cx);
                    }
                    "escape" => {
                        // ESC cancels the script completely
                        logging::log("KEY", "ESC in FormPrompt - canceling script");
                        this.submit_prompt_response(prompt_id_for_key.clone(), None, cx);
                        this.cancel_script_execution(cx);
                    }
                    "tab" => {
                        // Tab navigation between fields
                        if has_shift {
                            entity_for_shift_tab.update(cx, |form, cx| {
                                form.focus_previous(cx);
                            });
                        } else {
                            entity_for_tab.update(cx, |form, cx| {
                                form.focus_next(cx);
                            });
                        }
                    }
                    _ => {
                        // Forward all other keys (characters, backspace, arrows, etc.) to the focused field
                        // This is necessary because GPUI requires track_focus() to receive key events,
                        // and we need the parent to have focus to handle Enter/Escape/Tab.
                        // The form fields' individual on_key_down handlers don't fire when parent has focus.
                        entity_for_input.update(cx, |form, cx| {
                            form.handle_key_input(event, cx);
                        });
                    }
                }
            },
        );

        // Use design tokens for global theming
        let opacity = self.theme.get_opacity();
        let bg_hex = design_colors.background;
        let bg_with_alpha = self.hex_to_rgba_with_opacity(bg_hex, opacity.main);
        let box_shadows = self.create_box_shadows();

        // Dynamic height based on field count
        // Base height (150px) + per-field height (60px per field)
        // Minimum of calculated height and MAX_HEIGHT (700px)
        let base_height = 150.0;
        let field_height = 60.0;
        let calculated_height = base_height + (field_count as f32 * field_height);
        let max_height = 700.0; // Same as window_resize::layout::MAX_HEIGHT
        let content_height = px(calculated_height.min(max_height));

        // Button colors from theme
        let button_colors = ButtonColors::from_theme(&self.theme);
        let actions_button_colors = ButtonColors::from_theme(&self.theme);

        // Form fields have their own focus handles and on_key_down handlers.
        // We DO NOT track_focus on the container - the fields handle their own focus.
        // Enter/Escape/Tab are handled by the handle_key listener above.
        div()
            .relative() // Needed for absolute positioned actions dialog overlay
            .flex()
            .flex_col()
            .bg(rgba(bg_with_alpha))
            .shadow(box_shadows)
            .w_full()
            .h(content_height)
            .overflow_hidden()
            .rounded(px(design_visual.radius_lg))
            .text_color(rgb(design_colors.text_primary))
            .font_family(design_typography.font_family)
            .key_context("form_prompt")
            .on_key_down(handle_key)
            // Content area with form fields
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .min_h(px(0.))
                    .overflow_y_hidden() // Clip content at container boundary
                    .p(px(design_spacing.padding_xl))
                    // Render the form entity (contains all fields)
                    .child(entity.clone()),
            )
            // Header with CLS-FREE Actions area and Submit button
            .child(
                div()
                    .w_full()
                    .px(px(design_spacing.padding_lg))
                    .py(px(design_spacing.padding_md))
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_end()
                    .gap_2()
                    // CLS-FREE ACTIONS AREA
                    .when(has_actions, |d| {
                        let handle_actions = cx.entity().downgrade();
                        let show_actions_state = self.show_actions_popup;

                        let search_text = self
                            .actions_dialog
                            .as_ref()
                            .map(|dialog| dialog.read(cx).search_text.clone())
                            .unwrap_or_default();
                        let search_is_empty = search_text.is_empty();
                        let search_display: SharedString = if search_is_empty {
                            "Search actions...".into()
                        } else {
                            search_text.into()
                        };
                        let accent_color = design_colors.accent;
                        let text_primary = design_colors.text_primary;
                        let text_muted = design_colors.text_muted;
                        let text_dimmed = design_colors.text_dimmed;
                        let search_box_bg = self.theme.colors.background.search_box;
                        let cursor_visible_for_search = self.focused_input
                            == FocusedInput::ActionsSearch
                            && self.cursor_visible;

                        d.child(
                            div()
                                .relative()
                                .h(px(28.))
                                .flex()
                                .items_center()
                                .child(
                                    div()
                                        .absolute()
                                        .inset_0()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .justify_end()
                                        .when(show_actions_state, |d| d.opacity(0.).invisible())
                                        .child(
                                            Button::new("Actions", actions_button_colors)
                                                .variant(ButtonVariant::Ghost)
                                                .shortcut("⌘ K")
                                                .on_click(Box::new(move |_, window, cx| {
                                                    if let Some(app) = handle_actions.upgrade() {
                                                        app.update(cx, |this, cx| {
                                                            this.toggle_arg_actions(cx, window);
                                                        });
                                                    }
                                                })),
                                        ),
                                )
                                .child(
                                    div()
                                        .absolute()
                                        .inset_0()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .justify_end()
                                        .gap(px(8.))
                                        .when(!show_actions_state, |d| d.opacity(0.).invisible())
                                        .child(
                                            div()
                                                .text_color(rgb(text_dimmed))
                                                .text_xs()
                                                .child("⌘K"),
                                        )
                                        .child(
                                            div()
                                                .id("form-actions-search")
                                                .flex_shrink_0()
                                                .w(px(130.))
                                                .min_w(px(130.))
                                                .max_w(px(130.))
                                                .h(px(24.))
                                                .min_h(px(24.))
                                                .max_h(px(24.))
                                                .overflow_hidden()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .px(px(8.))
                                                .rounded(px(4.))
                                                .bg(rgba(
                                                    (search_box_bg << 8)
                                                        | if search_is_empty { 0x40 } else { 0x80 },
                                                ))
                                                .border_1()
                                                .border_color(rgba(
                                                    (accent_color << 8)
                                                        | if search_is_empty { 0x20 } else { 0x40 },
                                                ))
                                                .text_sm()
                                                .text_color(if search_is_empty {
                                                    rgb(text_muted)
                                                } else {
                                                    rgb(text_primary)
                                                })
                                                .when(search_is_empty, |d| {
                                                    d.child(
                                                        div()
                                                            .w(px(2.))
                                                            .h(px(14.))
                                                            .mr(px(2.))
                                                            .rounded(px(1.))
                                                            .when(cursor_visible_for_search, |d| {
                                                                d.bg(rgb(accent_color))
                                                            }),
                                                    )
                                                })
                                                .child(search_display)
                                                .when(!search_is_empty, |d| {
                                                    d.child(
                                                        div()
                                                            .w(px(2.))
                                                            .h(px(14.))
                                                            .ml(px(2.))
                                                            .rounded(px(1.))
                                                            .when(cursor_visible_for_search, |d| {
                                                                d.bg(rgb(accent_color))
                                                            }),
                                                    )
                                                }),
                                        ),
                                ),
                        )
                    })
                    // Submit button
                    .child(
                        Button::new("Submit", button_colors)
                            .variant(ButtonVariant::Primary)
                            .shortcut("↵"),
                    ),
            )
            // Actions dialog overlay
            .when_some(
                if self.show_actions_popup {
                    self.actions_dialog.clone()
                } else {
                    None
                },
                |d, dialog| {
                    let backdrop_click = cx.listener(
                        |this: &mut Self,
                         _event: &gpui::ClickEvent,
                         window: &mut Window,
                         cx: &mut Context<Self>| {
                            logging::log(
                                "FOCUS",
                                "Form actions backdrop clicked - dismissing dialog",
                            );
                            this.show_actions_popup = false;
                            this.actions_dialog = None;
                            this.focused_input = FocusedInput::None;
                            window.focus(&this.focus_handle, cx);
                            cx.notify();
                        },
                    );

                    d.child(
                        div()
                            .absolute()
                            .inset_0()
                            .child(
                                div()
                                    .id("form-actions-backdrop")
                                    .absolute()
                                    .inset_0()
                                    .on_click(backdrop_click),
                            )
                            .child(div().absolute().top(px(52.)).right(px(8.)).child(dialog)),
                    )
                },
            )
            .into_any_element()
    }

    fn render_term_prompt(
        &mut self,
        entity: Entity<term_prompt::TermPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let has_actions =
            self.sdk_actions.is_some() && !self.sdk_actions.as_ref().unwrap().is_empty();

        // Sync suppress_keys with actions popup state so terminal ignores keys when popup is open
        let show_actions = self.show_actions_popup;
        entity.update(cx, |term, _| {
            term.suppress_keys = show_actions;
        });

        // Use design tokens for GLOBAL theming
        let tokens = get_tokens(self.current_design);
        let design_colors = tokens.colors();
        let design_spacing = tokens.spacing();

        // Use design tokens for global theming
        let opacity = self.theme.get_opacity();
        let bg_hex = design_colors.background;
        let bg_with_alpha = self.hex_to_rgba_with_opacity(bg_hex, opacity.main);
        let box_shadows = self.create_box_shadows();

        // Use explicit height from layout constants instead of h_full()
        // h_full() doesn't work at the root level because there's no parent to fill
        let content_height = window_resize::layout::MAX_HEIGHT;

        // Key handler for Cmd+K actions toggle
        let has_actions_for_handler = has_actions;
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                let key_str = event.keystroke.key.to_lowercase();
                let has_cmd = event.keystroke.modifiers.platform;

                // Check for Cmd+K to toggle actions popup (if actions are available)
                if has_cmd && key_str == "k" && has_actions_for_handler {
                    logging::log("KEY", "Cmd+K in TermPrompt - calling toggle_arg_actions");
                    this.toggle_arg_actions(cx, window);
                    return;
                }

                // If actions popup is open, route keyboard events to it
                if this.show_actions_popup {
                    if let Some(ref dialog) = this.actions_dialog {
                        match key_str.as_str() {
                            "up" | "arrowup" => {
                                dialog.update(cx, |d, cx| d.move_up(cx));
                                return;
                            }
                            "down" | "arrowdown" => {
                                dialog.update(cx, |d, cx| d.move_down(cx));
                                return;
                            }
                            "enter" => {
                                let action_id = dialog.read(cx).get_selected_action_id();
                                let should_close = dialog.read(cx).selected_action_should_close();
                                if let Some(action_id) = action_id {
                                    logging::log(
                                        "ACTIONS",
                                        &format!(
                                            "TermPrompt executing action: {} (close={})",
                                            action_id, should_close
                                        ),
                                    );
                                    if should_close {
                                        this.show_actions_popup = false;
                                        this.actions_dialog = None;
                                        this.focused_input = FocusedInput::None;
                                        window.focus(&this.focus_handle, cx);
                                    }
                                    this.trigger_action_by_name(&action_id, cx);
                                }
                                return;
                            }
                            "escape" => {
                                this.show_actions_popup = false;
                                this.actions_dialog = None;
                                this.focused_input = FocusedInput::None;
                                window.focus(&this.focus_handle, cx);
                                cx.notify();
                                return;
                            }
                            "backspace" => {
                                dialog.update(cx, |d, cx| d.handle_backspace(cx));
                                return;
                            }
                            _ => {
                                if let Some(ref key_char) = event.keystroke.key_char {
                                    if let Some(ch) = key_char.chars().next() {
                                        if !ch.is_control() {
                                            dialog.update(cx, |d, cx| d.handle_char(ch, cx));
                                        }
                                    }
                                }
                                return;
                            }
                        }
                    }
                }

                // Check for SDK action shortcuts
                let shortcut_key =
                    shortcuts::keystroke_to_shortcut(&key_str, &event.keystroke.modifiers);
                if let Some(action_name) = this.action_shortcuts.get(&shortcut_key).cloned() {
                    logging::log(
                        "KEY",
                        &format!("SDK action shortcut matched: {}", action_name),
                    );
                    this.trigger_action_by_name(&action_name, cx);
                }
                // Let other keys fall through to the terminal
            },
        );

        // Container with explicit height. We wrap the entity in a sized div because
        // GPUI entities don't automatically inherit parent flex sizing.
        // NOTE: No rounded corners for terminal - it should fill edge-to-edge
        div()
            .relative() // Needed for absolute positioned actions dialog overlay
            .flex()
            .flex_col()
            .bg(rgba(bg_with_alpha))
            .shadow(box_shadows)
            .w_full()
            .h(content_height)
            .overflow_hidden()
            .on_key_down(handle_key)
            .child(div().size_full().child(entity))
            // CLS-FREE ACTIONS AREA in top-right corner
            .when(has_actions, |d| {
                let button_colors = ButtonColors::from_theme(&self.theme);
                let handle_actions = cx.entity().downgrade();
                let show_actions_state = self.show_actions_popup;

                // Get actions search text from the dialog
                let search_text = self
                    .actions_dialog
                    .as_ref()
                    .map(|dialog| dialog.read(cx).search_text.clone())
                    .unwrap_or_default();
                let search_is_empty = search_text.is_empty();
                let search_display: SharedString = if search_is_empty {
                    "Search actions...".into()
                } else {
                    search_text.into()
                };
                let accent_color = design_colors.accent;
                let text_primary = design_colors.text_primary;
                let text_muted = design_colors.text_muted;
                let text_dimmed = design_colors.text_dimmed;
                let search_box_bg = self.theme.colors.background.search_box;
                let cursor_visible_for_search =
                    self.focused_input == FocusedInput::ActionsSearch && self.cursor_visible;

                d.child(
                    div()
                        .absolute()
                        .top(px(design_spacing.padding_md))
                        .right(px(design_spacing.padding_md))
                        .child(
                            div()
                                .relative()
                                .h(px(28.))
                                .flex()
                                .items_center()
                                // Layer 1: Actions button
                                .child(
                                    div()
                                        .absolute()
                                        .inset_0()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .justify_end()
                                        .when(show_actions_state, |d| d.opacity(0.).invisible())
                                        .child(
                                            Button::new("Actions", button_colors)
                                                .variant(ButtonVariant::Ghost)
                                                .shortcut("⌘ K")
                                                .on_click(Box::new(move |_, window, cx| {
                                                    if let Some(app) = handle_actions.upgrade() {
                                                        app.update(cx, |this, cx| {
                                                            this.toggle_arg_actions(cx, window);
                                                        });
                                                    }
                                                })),
                                        ),
                                )
                                // Layer 2: Actions search input
                                .child(
                                    div()
                                        .absolute()
                                        .inset_0()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .justify_end()
                                        .gap(px(8.))
                                        .when(!show_actions_state, |d| d.opacity(0.).invisible())
                                        .child(
                                            div()
                                                .text_color(rgb(text_dimmed))
                                                .text_xs()
                                                .child("⌘K"),
                                        )
                                        .child(
                                            div()
                                                .id("term-actions-search")
                                                .flex_shrink_0()
                                                .w(px(130.))
                                                .min_w(px(130.))
                                                .max_w(px(130.))
                                                .h(px(24.))
                                                .min_h(px(24.))
                                                .max_h(px(24.))
                                                .overflow_hidden()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .px(px(8.))
                                                .rounded(px(4.))
                                                .bg(rgba(
                                                    (search_box_bg << 8)
                                                        | if search_is_empty { 0x40 } else { 0x80 },
                                                ))
                                                .border_1()
                                                .border_color(rgba(
                                                    (accent_color << 8)
                                                        | if search_is_empty { 0x20 } else { 0x40 },
                                                ))
                                                .text_sm()
                                                .text_color(if search_is_empty {
                                                    rgb(text_muted)
                                                } else {
                                                    rgb(text_primary)
                                                })
                                                .when(search_is_empty, |d| {
                                                    d.child(
                                                        div()
                                                            .w(px(2.))
                                                            .h(px(14.))
                                                            .mr(px(2.))
                                                            .rounded(px(1.))
                                                            .when(cursor_visible_for_search, |d| {
                                                                d.bg(rgb(accent_color))
                                                            }),
                                                    )
                                                })
                                                .child(search_display)
                                                .when(!search_is_empty, |d| {
                                                    d.child(
                                                        div()
                                                            .w(px(2.))
                                                            .h(px(14.))
                                                            .ml(px(2.))
                                                            .rounded(px(1.))
                                                            .when(cursor_visible_for_search, |d| {
                                                                d.bg(rgb(accent_color))
                                                            }),
                                                    )
                                                }),
                                        ),
                                ),
                        ),
                )
            })
            // Actions dialog overlay
            .when_some(
                if self.show_actions_popup {
                    self.actions_dialog.clone()
                } else {
                    None
                },
                |d, dialog| {
                    let backdrop_click = cx.listener(
                        |this: &mut Self,
                         _event: &gpui::ClickEvent,
                         window: &mut Window,
                         cx: &mut Context<Self>| {
                            logging::log(
                                "FOCUS",
                                "Term actions backdrop clicked - dismissing dialog",
                            );
                            this.show_actions_popup = false;
                            this.actions_dialog = None;
                            this.focused_input = FocusedInput::None;
                            window.focus(&this.focus_handle, cx);
                            cx.notify();
                        },
                    );

                    d.child(
                        div()
                            .absolute()
                            .inset_0()
                            .child(
                                div()
                                    .id("term-actions-backdrop")
                                    .absolute()
                                    .inset_0()
                                    .on_click(backdrop_click),
                            )
                            .child(div().absolute().top(px(52.)).right(px(8.)).child(dialog)),
                    )
                },
            )
            .into_any_element()
    }

    fn render_editor_prompt(
        &mut self,
        entity: Entity<EditorPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let has_actions =
            self.sdk_actions.is_some() && !self.sdk_actions.as_ref().unwrap().is_empty();

        // Sync suppress_keys with actions popup state so editor ignores keys when popup is open
        let show_actions = self.show_actions_popup;
        entity.update(cx, |editor, _| {
            editor.suppress_keys = show_actions;
        });

        // Use design tokens for GLOBAL theming
        let tokens = get_tokens(self.current_design);
        let design_colors = tokens.colors();
        let design_spacing = tokens.spacing();
        let design_visual = tokens.visual();

        // Use design tokens for global theming
        let opacity = self.theme.get_opacity();
        let bg_hex = design_colors.background;
        let bg_with_alpha = self.hex_to_rgba_with_opacity(bg_hex, opacity.main);
        let box_shadows = self.create_box_shadows();

        // Use explicit height from layout constants instead of h_full()
        // h_full() doesn't work at the root level because there's no parent to fill
        let content_height = window_resize::layout::MAX_HEIGHT;

        // Key handler for Cmd+K actions toggle (at parent level to intercept before editor)
        let has_actions_for_handler = has_actions;
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                let key_str = event.keystroke.key.to_lowercase();
                let has_cmd = event.keystroke.modifiers.platform;

                // Check for Cmd+K to toggle actions popup (if actions are available)
                if has_cmd && key_str == "k" && has_actions_for_handler {
                    logging::log("KEY", "Cmd+K in EditorPrompt - calling toggle_arg_actions");
                    this.toggle_arg_actions(cx, window);
                    return;
                }

                // If actions popup is open, route keyboard events to it
                if this.show_actions_popup {
                    if let Some(ref dialog) = this.actions_dialog {
                        match key_str.as_str() {
                            "up" | "arrowup" => {
                                dialog.update(cx, |d, cx| d.move_up(cx));
                                return;
                            }
                            "down" | "arrowdown" => {
                                dialog.update(cx, |d, cx| d.move_down(cx));
                                return;
                            }
                            "enter" => {
                                let action_id = dialog.read(cx).get_selected_action_id();
                                let should_close = dialog.read(cx).selected_action_should_close();
                                if let Some(action_id) = action_id {
                                    logging::log(
                                        "ACTIONS",
                                        &format!(
                                            "EditorPrompt executing action: {} (close={})",
                                            action_id, should_close
                                        ),
                                    );
                                    if should_close {
                                        this.show_actions_popup = false;
                                        this.actions_dialog = None;
                                        this.focused_input = FocusedInput::None;
                                        window.focus(&this.focus_handle, cx);
                                    }
                                    this.trigger_action_by_name(&action_id, cx);
                                }
                                return;
                            }
                            "escape" => {
                                this.show_actions_popup = false;
                                this.actions_dialog = None;
                                this.focused_input = FocusedInput::None;
                                window.focus(&this.focus_handle, cx);
                                cx.notify();
                                return;
                            }
                            "backspace" => {
                                dialog.update(cx, |d, cx| d.handle_backspace(cx));
                                return;
                            }
                            _ => {
                                if let Some(ref key_char) = event.keystroke.key_char {
                                    if let Some(ch) = key_char.chars().next() {
                                        if !ch.is_control() {
                                            dialog.update(cx, |d, cx| d.handle_char(ch, cx));
                                        }
                                    }
                                }
                                return;
                            }
                        }
                    }
                }

                // Check for SDK action shortcuts
                let shortcut_key =
                    shortcuts::keystroke_to_shortcut(&key_str, &event.keystroke.modifiers);
                if let Some(action_name) = this.action_shortcuts.get(&shortcut_key).cloned() {
                    logging::log(
                        "KEY",
                        &format!("SDK action shortcut matched: {}", action_name),
                    );
                    this.trigger_action_by_name(&action_name, cx);
                }
                // Let other keys fall through to the editor
            },
        );

        // NOTE: The EditorPrompt entity has its own track_focus and on_key_down in its render method.
        // We do NOT add track_focus here to avoid duplicate focus tracking on the same handle.
        //
        // Container with explicit height. We wrap the entity in a sized div because
        // GPUI entities don't automatically inherit parent flex sizing.
        div()
            .relative() // Needed for absolute positioned actions dialog overlay
            .flex()
            .flex_col()
            .bg(rgba(bg_with_alpha))
            .shadow(box_shadows)
            .w_full()
            .h(content_height)
            .overflow_hidden()
            .rounded(px(design_visual.radius_lg))
            .on_key_down(handle_key)
            .child(div().size_full().child(entity))
            // CLS-FREE ACTIONS AREA in top-right corner
            .when(has_actions, |d| {
                let button_colors = ButtonColors::from_theme(&self.theme);
                let handle_actions = cx.entity().downgrade();
                let show_actions_state = self.show_actions_popup;

                // Get actions search text from the dialog
                let search_text = self
                    .actions_dialog
                    .as_ref()
                    .map(|dialog| dialog.read(cx).search_text.clone())
                    .unwrap_or_default();
                let search_is_empty = search_text.is_empty();
                let search_display: SharedString = if search_is_empty {
                    "Search actions...".into()
                } else {
                    search_text.into()
                };
                let accent_color = design_colors.accent;
                let text_primary = design_colors.text_primary;
                let text_muted = design_colors.text_muted;
                let text_dimmed = design_colors.text_dimmed;
                let search_box_bg = self.theme.colors.background.search_box;
                let cursor_visible_for_search =
                    self.focused_input == FocusedInput::ActionsSearch && self.cursor_visible;

                d.child(
                    div()
                        .absolute()
                        .top(px(design_spacing.padding_md))
                        .right(px(design_spacing.padding_md))
                        .child(
                            div()
                                .relative()
                                .h(px(28.))
                                .flex()
                                .items_center()
                                // Layer 1: Actions button
                                .child(
                                    div()
                                        .absolute()
                                        .inset_0()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .justify_end()
                                        .when(show_actions_state, |d| d.opacity(0.).invisible())
                                        .child(
                                            Button::new("Actions", button_colors)
                                                .variant(ButtonVariant::Ghost)
                                                .shortcut("⌘ K")
                                                .on_click(Box::new(move |_, window, cx| {
                                                    if let Some(app) = handle_actions.upgrade() {
                                                        app.update(cx, |this, cx| {
                                                            this.toggle_arg_actions(cx, window);
                                                        });
                                                    }
                                                })),
                                        ),
                                )
                                // Layer 2: Actions search input
                                .child(
                                    div()
                                        .absolute()
                                        .inset_0()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .justify_end()
                                        .gap(px(8.))
                                        .when(!show_actions_state, |d| d.opacity(0.).invisible())
                                        .child(
                                            div()
                                                .text_color(rgb(text_dimmed))
                                                .text_xs()
                                                .child("⌘K"),
                                        )
                                        .child(
                                            div()
                                                .id("editor-actions-search")
                                                .flex_shrink_0()
                                                .w(px(130.))
                                                .min_w(px(130.))
                                                .max_w(px(130.))
                                                .h(px(24.))
                                                .min_h(px(24.))
                                                .max_h(px(24.))
                                                .overflow_hidden()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .px(px(8.))
                                                .rounded(px(4.))
                                                .bg(rgba(
                                                    (search_box_bg << 8)
                                                        | if search_is_empty { 0x40 } else { 0x80 },
                                                ))
                                                .border_1()
                                                .border_color(rgba(
                                                    (accent_color << 8)
                                                        | if search_is_empty { 0x20 } else { 0x40 },
                                                ))
                                                .text_sm()
                                                .text_color(if search_is_empty {
                                                    rgb(text_muted)
                                                } else {
                                                    rgb(text_primary)
                                                })
                                                .when(search_is_empty, |d| {
                                                    d.child(
                                                        div()
                                                            .w(px(2.))
                                                            .h(px(14.))
                                                            .mr(px(2.))
                                                            .rounded(px(1.))
                                                            .when(cursor_visible_for_search, |d| {
                                                                d.bg(rgb(accent_color))
                                                            }),
                                                    )
                                                })
                                                .child(search_display)
                                                .when(!search_is_empty, |d| {
                                                    d.child(
                                                        div()
                                                            .w(px(2.))
                                                            .h(px(14.))
                                                            .ml(px(2.))
                                                            .rounded(px(1.))
                                                            .when(cursor_visible_for_search, |d| {
                                                                d.bg(rgb(accent_color))
                                                            }),
                                                    )
                                                }),
                                        ),
                                ),
                        ),
                )
            })
            // Actions dialog overlay
            .when_some(
                if self.show_actions_popup {
                    self.actions_dialog.clone()
                } else {
                    None
                },
                |d, dialog| {
                    let backdrop_click = cx.listener(
                        |this: &mut Self,
                         _event: &gpui::ClickEvent,
                         window: &mut Window,
                         cx: &mut Context<Self>| {
                            logging::log(
                                "FOCUS",
                                "Editor actions backdrop clicked - dismissing dialog",
                            );
                            this.show_actions_popup = false;
                            this.actions_dialog = None;
                            this.focused_input = FocusedInput::None;
                            window.focus(&this.focus_handle, cx);
                            cx.notify();
                        },
                    );

                    d.child(
                        div()
                            .absolute()
                            .inset_0()
                            .child(
                                div()
                                    .id("editor-actions-backdrop")
                                    .absolute()
                                    .inset_0()
                                    .on_click(backdrop_click),
                            )
                            .child(div().absolute().top(px(52.)).right(px(8.)).child(dialog)),
                    )
                },
            )
            .into_any_element()
    }

    fn render_select_prompt(
        &mut self,
        entity: Entity<SelectPrompt>,
        _cx: &mut Context<Self>,
    ) -> AnyElement {
        // Use design tokens for GLOBAL theming
        let tokens = get_tokens(self.current_design);
        let design_colors = tokens.colors();
        let design_visual = tokens.visual();

        // Use design tokens for global theming
        let opacity = self.theme.get_opacity();
        let bg_hex = design_colors.background;
        let bg_with_alpha = self.hex_to_rgba_with_opacity(bg_hex, opacity.main);
        let box_shadows = self.create_box_shadows();

        // SelectPrompt entity has its own track_focus and on_key_down in its render method.
        div()
            .flex()
            .flex_col()
            .bg(rgba(bg_with_alpha))
            .shadow(box_shadows)
            .w_full()
            .h_full()
            .overflow_hidden()
            .rounded(px(design_visual.radius_lg))
            .child(div().size_full().child(entity))
            .into_any_element()
    }

    fn render_path_prompt(
        &mut self,
        entity: Entity<PathPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        // Use design tokens for GLOBAL theming
        let tokens = get_tokens(self.current_design);
        let design_colors = tokens.colors();
        let design_visual = tokens.visual();

        // Use design tokens for global theming
        let opacity = self.theme.get_opacity();
        let bg_hex = design_colors.background;
        let bg_with_alpha = self.hex_to_rgba_with_opacity(bg_hex, opacity.main);
        let box_shadows = self.create_box_shadows();

        // Check if we should close the actions dialog
        if let Ok(mut close_guard) = self.close_path_actions.lock() {
            if *close_guard {
                *close_guard = false;
                self.show_actions_popup = false;
                self.actions_dialog = None;
                // Update shared showing state for toggle behavior
                if let Ok(mut showing_guard) = self.path_actions_showing.lock() {
                    *showing_guard = false;
                }
                logging::log("UI", "Closed path actions dialog");
            }
        }

        // Check for pending path action result and execute it
        // Extract data first, then drop the lock, then execute
        let pending_action = self
            .pending_path_action_result
            .lock()
            .ok()
            .and_then(|mut guard| guard.take());

        if let Some((action_id, path_info)) = pending_action {
            self.execute_path_action(&action_id, &path_info, &entity, cx);
        }

        // Check for pending path action and create ActionsDialog if needed
        let actions_dialog = if let Ok(mut guard) = self.pending_path_action.lock() {
            if let Some(path_info) = guard.take() {
                // Create ActionsDialog for this path
                let theme_arc = std::sync::Arc::new(self.theme.clone());
                let close_signal = self.close_path_actions.clone();
                let action_result_signal = self.pending_path_action_result.clone();
                let path_info_for_callback = path_info.clone();
                let action_callback: std::sync::Arc<dyn Fn(String) + Send + Sync> =
                    std::sync::Arc::new(move |action_id| {
                        logging::log(
                            "UI",
                            &format!(
                                "Path action selected: {} for path: {}",
                                action_id, path_info_for_callback.path
                            ),
                        );
                        // Store the action result for execution
                        if action_id != "__cancel__" {
                            if let Ok(mut guard) = action_result_signal.lock() {
                                *guard = Some((action_id.clone(), path_info_for_callback.clone()));
                            }
                        }
                        // Signal to close dialog on cancel or action selection
                        if let Ok(mut guard) = close_signal.lock() {
                            *guard = true;
                        }
                    });
                let dialog = cx.new(|cx| {
                    let focus_handle = cx.focus_handle();
                    let mut dialog = ActionsDialog::with_path(
                        focus_handle,
                        action_callback,
                        &path_info,
                        theme_arc,
                    );
                    // Hide search in the dialog - we show it in the header instead
                    dialog.set_hide_search(true);
                    dialog
                });
                self.actions_dialog = Some(dialog.clone());
                self.show_actions_popup = true;
                // Update shared showing state for toggle behavior
                if let Ok(mut showing_guard) = self.path_actions_showing.lock() {
                    *showing_guard = true;
                }
                Some(dialog)
            } else if self.show_actions_popup {
                self.actions_dialog.clone()
            } else {
                None
            }
        } else {
            None
        };

        // Sync the actions search text from the dialog to the shared state
        // This allows PathPrompt to display the search text in its header
        if let Some(ref dialog) = actions_dialog {
            let search_text = dialog.read(cx).search_text.clone();
            if let Ok(mut guard) = self.path_actions_search_text.lock() {
                *guard = search_text;
            }
        } else {
            // Clear search text when dialog is not showing
            if let Ok(mut guard) = self.path_actions_search_text.lock() {
                guard.clear();
            }
        }

        // Key handler for when actions dialog is showing
        // This intercepts keys and routes them to the dialog (like main menu does)
        let path_entity = entity.clone();
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                let key_str = event.keystroke.key.to_lowercase();
                let has_cmd = event.keystroke.modifiers.platform;

                logging::log(
                    "KEY",
                    &format!(
                        "PathPrompt OUTER handler: key='{}', show_actions_popup={}",
                        key_str, this.show_actions_popup
                    ),
                );

                // Cmd+K toggles actions from anywhere
                if has_cmd && key_str == "k" {
                    // Toggle the actions dialog
                    if this.show_actions_popup {
                        // Close actions
                        this.show_actions_popup = false;
                        this.actions_dialog = None;
                        if let Ok(mut guard) = this.path_actions_showing.lock() {
                            *guard = false;
                        }
                        cx.notify();
                    } else {
                        // Open actions - trigger the callback in PathPrompt
                        path_entity.update(cx, |prompt, cx| {
                            prompt.toggle_actions(cx);
                        });
                    }
                    return;
                }

                // If actions popup is open, route keyboard events to it
                if this.show_actions_popup {
                    if let Some(ref dialog) = this.actions_dialog {
                        match key_str.as_str() {
                            "up" | "arrowup" => {
                                dialog.update(cx, |d, cx| d.move_up(cx));
                            }
                            "down" | "arrowdown" => {
                                dialog.update(cx, |d, cx| d.move_down(cx));
                            }
                            "enter" => {
                                // Get the selected action and execute it
                                let action_id = dialog.read(cx).get_selected_action_id();
                                let should_close = dialog.read(cx).selected_action_should_close();
                                if let Some(action_id) = action_id {
                                    logging::log(
                                        "ACTIONS",
                                        &format!(
                                            "Path action selected via Enter: {} (close={})",
                                            action_id, should_close
                                        ),
                                    );

                                    // Get path info from PathPrompt
                                    let path_info = path_entity.read(cx).get_selected_path_info();

                                    // Close dialog if action says so (built-in path actions always close)
                                    if should_close {
                                        this.show_actions_popup = false;
                                        this.actions_dialog = None;
                                        if let Ok(mut guard) = this.path_actions_showing.lock() {
                                            *guard = false;
                                        }

                                        // Focus back to PathPrompt
                                        if let AppView::PathPrompt { focus_handle, .. } =
                                            &this.current_view
                                        {
                                            window.focus(focus_handle, cx);
                                        }
                                    }

                                    // Execute the action if we have path info
                                    if let Some(info) = path_info {
                                        this.execute_path_action(
                                            &action_id,
                                            &info,
                                            &path_entity,
                                            cx,
                                        );
                                    }
                                }
                            }
                            "escape" => {
                                this.show_actions_popup = false;
                                this.actions_dialog = None;
                                if let Ok(mut guard) = this.path_actions_showing.lock() {
                                    *guard = false;
                                }
                                // Focus back to PathPrompt
                                if let AppView::PathPrompt { focus_handle, .. } = &this.current_view
                                {
                                    window.focus(focus_handle, cx);
                                }
                                cx.notify();
                            }
                            "backspace" => {
                                dialog.update(cx, |d, cx| d.handle_backspace(cx));
                            }
                            _ => {
                                // Route character input to the dialog for search
                                if let Some(ref key_char) = event.keystroke.key_char {
                                    if let Some(ch) = key_char.chars().next() {
                                        if !ch.is_control() {
                                            dialog.update(cx, |d, cx| d.handle_char(ch, cx));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                // If actions not showing, let PathPrompt handle the keys via its own handler
            },
        );

        // PathPrompt entity has its own track_focus and on_key_down in its render method.
        // We add an outer key handler to intercept events when actions are showing.
        div()
            .relative()
            .flex()
            .flex_col()
            .bg(rgba(bg_with_alpha))
            .shadow(box_shadows)
            .w_full()
            .h_full()
            .overflow_hidden()
            .rounded(px(design_visual.radius_lg))
            .key_context("path_prompt_container")
            .on_key_down(handle_key)
            .child(div().size_full().child(entity))
            // Actions dialog overlays on top (upper-right corner, below the header bar)
            .when_some(actions_dialog, |d, dialog| {
                d.child(
                    div()
                        .absolute()
                        .inset_0()
                        .flex()
                        .justify_end()
                        .pt(px(52.)) // Clear the header bar (~44px header + 8px margin)
                        .pr(px(8.))
                        .child(dialog),
                )
            })
            .into_any_element()
    }

    fn render_env_prompt(
        &mut self,
        entity: Entity<EnvPrompt>,
        _cx: &mut Context<Self>,
    ) -> AnyElement {
        // Use design tokens for GLOBAL theming
        let tokens = get_tokens(self.current_design);
        let design_colors = tokens.colors();
        let design_visual = tokens.visual();

        // Use design tokens for global theming
        let opacity = self.theme.get_opacity();
        let bg_hex = design_colors.background;
        let bg_with_alpha = self.hex_to_rgba_with_opacity(bg_hex, opacity.main);
        let box_shadows = self.create_box_shadows();

        // EnvPrompt entity has its own track_focus and on_key_down in its render method.
        div()
            .flex()
            .flex_col()
            .bg(rgba(bg_with_alpha))
            .shadow(box_shadows)
            .w_full()
            .h_full()
            .overflow_hidden()
            .rounded(px(design_visual.radius_lg))
            .child(div().size_full().child(entity))
            .into_any_element()
    }

    fn render_drop_prompt(
        &mut self,
        entity: Entity<DropPrompt>,
        _cx: &mut Context<Self>,
    ) -> AnyElement {
        // Use design tokens for GLOBAL theming
        let tokens = get_tokens(self.current_design);
        let design_colors = tokens.colors();
        let design_visual = tokens.visual();

        // Use design tokens for global theming
        let opacity = self.theme.get_opacity();
        let bg_hex = design_colors.background;
        let bg_with_alpha = self.hex_to_rgba_with_opacity(bg_hex, opacity.main);
        let box_shadows = self.create_box_shadows();

        // DropPrompt entity has its own track_focus and on_key_down in its render method.
        div()
            .flex()
            .flex_col()
            .bg(rgba(bg_with_alpha))
            .shadow(box_shadows)
            .w_full()
            .h_full()
            .overflow_hidden()
            .rounded(px(design_visual.radius_lg))
            .child(div().size_full().child(entity))
            .into_any_element()
    }

    fn render_template_prompt(
        &mut self,
        entity: Entity<TemplatePrompt>,
        _cx: &mut Context<Self>,
    ) -> AnyElement {
        // Use design tokens for GLOBAL theming
        let tokens = get_tokens(self.current_design);
        let design_colors = tokens.colors();
        let design_visual = tokens.visual();

        // Use design tokens for global theming
        let opacity = self.theme.get_opacity();
        let bg_hex = design_colors.background;
        let bg_with_alpha = self.hex_to_rgba_with_opacity(bg_hex, opacity.main);
        let box_shadows = self.create_box_shadows();

        // TemplatePrompt entity has its own track_focus and on_key_down in its render method.
        div()
            .flex()
            .flex_col()
            .bg(rgba(bg_with_alpha))
            .shadow(box_shadows)
            .w_full()
            .h_full()
            .overflow_hidden()
            .rounded(px(design_visual.radius_lg))
            .child(div().size_full().child(entity))
            .into_any_element()
    }

    fn render_actions_dialog(&mut self, cx: &mut Context<Self>) -> AnyElement {
        // Use design tokens for GLOBAL theming
        let tokens = get_tokens(self.current_design);
        let design_colors = tokens.colors();
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();
        let design_visual = tokens.visual();

        // Use design tokens for global theming
        let opacity = self.theme.get_opacity();
        let bg_hex = design_colors.background;
        let bg_with_alpha = self.hex_to_rgba_with_opacity(bg_hex, opacity.main);
        let box_shadows = self.create_box_shadows();

        // Key handler for actions dialog
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  _window: &mut Window,
                  cx: &mut Context<Self>| {
                let key_str = event.keystroke.key.to_lowercase();
                logging::log("KEY", &format!("ActionsDialog key: '{}'", key_str));

                if key_str.as_str() == "escape" {
                    logging::log("KEY", "ESC in ActionsDialog - returning to script list");
                    this.current_view = AppView::ScriptList;
                    cx.notify();
                }
            },
        );

        // Simple actions dialog stub with design tokens
        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .bg(rgba(bg_with_alpha))
            .shadow(box_shadows)
            .rounded(px(design_visual.radius_lg))
            .p(px(design_spacing.padding_xl))
            .text_color(rgb(design_colors.text_primary))
            .font_family(design_typography.font_family)
            .key_context("actions_dialog")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .child(div().text_lg().child("Actions (Cmd+K)"))
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(design_colors.text_muted))
                    .mt(px(design_spacing.margin_md))
                    .child("• Create script\n• Edit script\n• Reload\n• Settings\n• Quit"),
            )
            .child(
                div()
                    .mt(px(design_spacing.margin_lg))
                    .text_xs()
                    .text_color(rgb(design_colors.text_dimmed))
                    .child("Press Esc to close"),
            )
            .into_any_element()
    }
}

/// Helper function to render a group header style item with actual visual styling
fn render_group_header_item(
    ix: usize,
    is_selected: bool,
    style: &designs::group_header_variations::GroupHeaderStyle,
    spacing: &designs::DesignSpacing,
    typography: &designs::DesignTypography,
    visual: &designs::DesignVisual,
    colors: &designs::DesignColors,
) -> AnyElement {
    use designs::group_header_variations::GroupHeaderStyle;

    let name_owned = style.name().to_string();
    let desc_owned = style.description().to_string();

    let mut item_div = div()
        .id(ElementId::NamedInteger("gallery-header".into(), ix as u64))
        .w_full()
        .h(px(LIST_ITEM_HEIGHT))
        .px(px(spacing.padding_lg))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(spacing.gap_md));

    if is_selected {
        item_div = item_div.bg(rgb(colors.background_selected));
    }

    // Create the preview element based on the style
    let preview = match style {
        // Text Only styles - vary font weight and style
        GroupHeaderStyle::UppercaseLeft => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(rgb(colors.text_secondary))
            .child("MAIN"),
        GroupHeaderStyle::UppercaseCenter => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .justify_center()
            .text_xs()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(rgb(colors.text_secondary))
            .child("MAIN"),
        GroupHeaderStyle::SmallCapsLeft => {
            div()
                .w(px(140.0))
                .h(px(28.0))
                .rounded(px(visual.radius_sm))
                .bg(rgba((colors.background_secondary << 8) | 0x60))
                .flex()
                .items_center()
                .px(px(8.0))
                .text_xs()
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(rgb(colors.text_secondary))
                .child("MAIN") // Would use font-variant: small-caps if available
        }
        GroupHeaderStyle::BoldLeft => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .font_weight(gpui::FontWeight::BOLD)
            .text_color(rgb(colors.text_primary))
            .child("MAIN"),
        GroupHeaderStyle::LightLeft => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .font_weight(gpui::FontWeight::LIGHT)
            .text_color(rgb(colors.text_muted))
            .child("MAIN"),
        GroupHeaderStyle::MonospaceLeft => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .font_family(typography.font_family_mono)
            .text_color(rgb(colors.text_secondary))
            .child("MAIN"),

        // With Lines styles
        GroupHeaderStyle::LineLeft => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .child(div().w(px(24.0)).h(px(1.0)).bg(rgb(colors.border)))
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            ),
        GroupHeaderStyle::LineRight => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            )
            .child(div().flex_1().h(px(1.0)).bg(rgb(colors.border))),
        GroupHeaderStyle::LineBothSides => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .child(div().flex_1().h(px(1.0)).bg(rgb(colors.border)))
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            )
            .child(div().flex_1().h(px(1.0)).bg(rgb(colors.border))),
        GroupHeaderStyle::LineBelow => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_col()
            .justify_center()
            .px(px(8.0))
            .gap(px(2.0))
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            )
            .child(div().w(px(40.0)).h(px(1.0)).bg(rgb(colors.border))),
        GroupHeaderStyle::LineAbove => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_col()
            .justify_center()
            .px(px(8.0))
            .gap(px(2.0))
            .child(div().w(px(40.0)).h(px(1.0)).bg(rgb(colors.border)))
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            ),
        GroupHeaderStyle::DoubleLine => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_col()
            .justify_center()
            .items_center()
            .gap(px(1.0))
            .child(div().w(px(100.0)).h(px(1.0)).bg(rgb(colors.border)))
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            )
            .child(div().w(px(100.0)).h(px(1.0)).bg(rgb(colors.border))),

        // With Background styles
        GroupHeaderStyle::PillBackground => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .child(
                div()
                    .px(px(8.0))
                    .py(px(2.0))
                    .rounded(px(10.0))
                    .bg(rgba((colors.accent << 8) | 0x30))
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.accent))
                    .child("MAIN"),
            ),
        GroupHeaderStyle::FullWidthBackground => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.accent << 8) | 0x20))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(rgb(colors.text_primary))
            .child("MAIN"),
        GroupHeaderStyle::SubtleBackground => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x90))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(rgb(colors.text_secondary))
            .child("MAIN"),
        GroupHeaderStyle::GradientFade => {
            // Simulated with opacity fade
            div()
                .w(px(140.0))
                .h(px(28.0))
                .rounded(px(visual.radius_sm))
                .bg(rgba((colors.background_secondary << 8) | 0x60))
                .flex()
                .items_center()
                .px(px(8.0))
                .child(
                    div()
                        .px(px(16.0))
                        .text_xs()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(rgb(colors.text_secondary))
                        .child("~  MAIN  ~"),
                )
        }
        GroupHeaderStyle::BorderedBox => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .child(
                div()
                    .px(px(8.0))
                    .py(px(2.0))
                    .border_1()
                    .border_color(rgb(colors.border))
                    .rounded(px(2.0))
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            ),

        // Minimal styles
        GroupHeaderStyle::DotPrefix => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .child(
                div()
                    .w(px(4.0))
                    .h(px(4.0))
                    .rounded(px(2.0))
                    .bg(rgb(colors.text_muted)),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            ),
        GroupHeaderStyle::DashPrefix => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .text_color(rgb(colors.text_secondary))
            .child("- MAIN"),
        GroupHeaderStyle::BulletPrefix => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .child(
                div()
                    .w(px(6.0))
                    .h(px(6.0))
                    .rounded(px(3.0))
                    .bg(rgb(colors.accent)),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            ),
        GroupHeaderStyle::ArrowPrefix => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .text_color(rgb(colors.text_secondary))
            .child("\u{25B8} MAIN"),
        GroupHeaderStyle::ChevronPrefix => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .text_color(rgb(colors.text_secondary))
            .child("\u{203A} MAIN"),
        GroupHeaderStyle::Dimmed => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .opacity(0.5)
            .text_color(rgb(colors.text_muted))
            .child("MAIN"),

        // Decorative styles
        GroupHeaderStyle::Bracketed => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .text_color(rgb(colors.text_secondary))
            .child("[MAIN]"),
        GroupHeaderStyle::Quoted => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .text_color(rgb(colors.text_secondary))
            .child("\"MAIN\""),
        GroupHeaderStyle::Tagged => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .child(
                div()
                    .px(px(6.0))
                    .py(px(1.0))
                    .bg(rgba((colors.accent << 8) | 0x40))
                    .rounded(px(2.0))
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.accent))
                    .child("MAIN"),
            ),
        GroupHeaderStyle::Numbered => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(rgb(colors.accent))
                    .child("01."),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            ),
        GroupHeaderStyle::IconPrefix => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .child(
                div()
                    .w(px(8.0))
                    .h(px(8.0))
                    .bg(rgb(colors.accent))
                    .rounded(px(1.0)),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            ),
    };

    item_div
        // Preview element
        .child(preview)
        // Name and description
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .gap(px(2.0))
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(rgb(colors.text_primary))
                        .child(name_owned),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(colors.text_muted))
                        .child(desc_owned),
                ),
        )
        .into_any_element()
}
