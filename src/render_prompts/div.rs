// Div prompt render method - extracted from render_prompts.rs
// This file is included via include!() macro in main.rs

impl ScriptListApp {
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
                // Global shortcuts (Cmd+W, ESC for dismissable prompts)
                if this.handle_global_shortcut_with_options(event, true, cx) {
                    return;
                }

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

        // Pre-build the overlay header (needs to reference captured variables)
        let header_overlay = if has_actions {
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

            Some(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
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
                                    .child(div().text_color(rgb(text_dimmed)).text_xs().child("⌘K"))
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
        } else {
            None
        };

        div()
            .relative() // Needed for absolute positioned overlays
            .bg(rgba(bg_with_alpha))
            .shadow(box_shadows)
            .w_full()
            .h(content_height)
            .overflow_hidden()
            .rounded(px(design_visual.radius_lg))
            .track_focus(&self.focus_handle) // Required to receive key events
            .on_key_down(handle_key)
            // Content area FIRST - render the DivPrompt entity (full height)
            .child(
                div()
                    .size_full()
                    .min_h(px(0.)) // Critical: allows flex children to size properly
                    .overflow_hidden()
                    .child(entity.clone()),
            )
            // Header overlay SECOND - absolute positioned, renders ON TOP of content
            .when_some(header_overlay, |d, header| d.child(header))
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
}
