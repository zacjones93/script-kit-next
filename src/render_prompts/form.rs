// Form prompt render method - extracted from render_prompts.rs
// This file is included via include!() macro in main.rs

impl ScriptListApp {
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
                // Global shortcuts (Cmd+W, ESC for dismissable prompts)
                if this.handle_global_shortcut_with_options(event, true, cx) {
                    return;
                }

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
                    // Note: "escape" is handled by handle_global_shortcut_with_options above
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
        let box_shadows = self.create_box_shadows();

        // VIBRANCY: Use foundation helper - returns None when vibrancy enabled (let Root handle bg)
        let vibrancy_bg = get_vibrancy_background(&self.theme);

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
            .when_some(vibrancy_bg, |d, bg| d.bg(bg)) // VIBRANCY: Only apply bg when vibrancy disabled
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
}
