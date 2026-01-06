// Editor prompt render method - extracted from render_prompts.rs
// This file is included via include!() macro in main.rs

impl ScriptListApp {
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
        let box_shadows = self.create_box_shadows();

        // VIBRANCY: Use foundation helper - returns None when vibrancy enabled (let Root handle bg)
        let vibrancy_bg = get_vibrancy_background(&self.theme);

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
                // Global shortcuts (Cmd+W only - editor is NOT dismissable with ESC)
                // Note: When actions popup is open, ESC should close the popup
                if !this.show_actions_popup
                    && this.handle_global_shortcut_with_options(event, false, cx)
                {
                    return;
                }

                let key = event.keystroke.key.as_str();
                let key_char = event.keystroke.key_char.as_deref();
                let has_cmd = event.keystroke.modifiers.platform;

                // Check for Cmd+K to toggle actions popup (if actions are available)
                if has_cmd && ui_foundation::is_key_k(key) && has_actions_for_handler {
                    logging::log("KEY", "Cmd+K in EditorPrompt - calling toggle_arg_actions");
                    this.toggle_arg_actions(cx, window);
                    return;
                }

                // Route to shared actions dialog handler (modal when open)
                match this.route_key_to_actions_dialog(
                    key,
                    key_char,
                    ActionsDialogHost::EditorPrompt,
                    window,
                    cx,
                ) {
                    ActionsRoute::Execute { action_id } => {
                        this.trigger_action_by_name(&action_id, cx);
                        return;
                    }
                    ActionsRoute::Handled => {
                        // Key consumed by actions dialog
                        return;
                    }
                    ActionsRoute::NotHandled => {
                        // Actions popup not open - continue with normal handling
                    }
                }

                // Check for SDK action shortcuts (only when popup is NOT open)
                let key_lower = key.to_lowercase();
                let shortcut_key =
                    shortcuts::keystroke_to_shortcut(&key_lower, &event.keystroke.modifiers);
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
            .when_some(vibrancy_bg, |d, bg| d.bg(bg)) // VIBRANCY: Only apply bg when vibrancy disabled
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
                            this.close_actions_popup(ActionsDialogHost::EditorPrompt, window, cx);
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
}
