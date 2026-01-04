// Path prompt render method - extracted from render_prompts.rs
// This file is included via include!() macro in main.rs

impl ScriptListApp {
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
                // Global shortcuts (Cmd+W, ESC for dismissable prompts)
                if this.handle_global_shortcut_with_options(event, true, cx) {
                    return;
                }

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
}
