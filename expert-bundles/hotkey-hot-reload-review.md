# Hotkey Hot-Reload Implementation Expert Bundle

## Original Goal

> Implement hotkey hot-reload so that when `~/.scriptkit/kit/config.ts` changes, the main hotkey, notes hotkey, and AI hotkey are updated automatically without requiring an app restart.
>
> This was explicitly requested: "Well we absolutely need the hotkeys to hot reload! So do what it takes to get that working."

## Executive Summary

This bundle contains a complete implementation of hotkey hot-reload for a Rust/GPUI desktop application. The solution uses global state with atomic operations to allow the hotkey event loop (running on a separate thread) to recognize newly registered hotkeys after config changes.

### Key Design Decisions:
1. **Global `GlobalHotKeyManager`** stored in `MAIN_MANAGER` (OnceLock<Mutex<...>>) so `update_hotkeys()` can acquire it during config reload.
2. **Atomic IDs** (`AtomicU32`) for main/notes/AI hotkeys allow the event loop to read current IDs without locking.
3. **`CURRENT_HOTKEYS`** stores actual `HotKey` objects needed for proper OS-level unregistration before re-registration.
4. **Manager lock dropped** before event loop starts, so `update_hotkeys()` can acquire it when config changes.

### Key Files:
- `src/hotkeys.rs`: Core implementation - global state, `update_hotkeys()`, modified `start_hotkey_listener()`
- `src/app_impl.rs`: Calls `hotkeys::update_hotkeys(&self.config)` in `update_config()`
- `src/config/mod.rs`: Exports `HotkeyConfig` unconditionally (was test-only)

### Architecture:
```
┌─────────────────────────────────────────────────────────────┐
│                    Config Change Flow                        │
├─────────────────────────────────────────────────────────────┤
│  config.ts modified                                          │
│       │                                                      │
│       ▼                                                      │
│  ConfigWatcher triggers update_config()                      │
│       │                                                      │
│       ▼                                                      │
│  update_config() calls hotkeys::update_hotkeys(&config)      │
│       │                                                      │
│       ▼                                                      │
│  update_hotkeys():                                           │
│    1. Locks MAIN_MANAGER                                     │
│    2. For each hotkey (main/notes/ai):                       │
│       - Parse new config → new HotKey → new_id               │
│       - Read current_id from atomic                          │
│       - If new_id != current_id:                             │
│         a. Unregister old HotKey from CURRENT_HOTKEYS        │
│         b. Register new HotKey with manager                  │
│         c. Store new_id in atomic                            │
│         d. Store new HotKey in CURRENT_HOTKEYS               │
│    3. Unlock manager                                         │
│       │                                                      │
│       ▼                                                      │
│  Event loop (in separate thread):                            │
│    - Reads current IDs from atomics on each event            │
│    - Matches event.id against current_main_id, etc.          │
│    - Immediately recognizes new hotkeys                      │
└─────────────────────────────────────────────────────────────┘
```

### Files Included:
- `src/hotkeys.rs`: Hotkey registration, global state, hot-reload logic (~1350 lines)
- `src/app_impl.rs`: Main app implementation with `update_config()` (~2540 lines)
- `src/config/mod.rs`: Config module exports (~40 lines)

---
## Packx Output (verbatim)

🧩 Packing 3 file(s)...
📝 Files selected:
  • src/app_impl.rs
  • src/config/mod.rs
  • src/hotkeys.rs
This file is a merged representation of the filtered codebase, combined into a single document by packx.

<file_summary>
This section contains a summary of this file.

<purpose>
This file contains a packed representation of filtered repository contents.
It is designed to be easily consumable by AI systems for analysis, code review,
or other automated processes.
</purpose>

<usage_guidelines>
- Treat this file as a snapshot of the repository's state
- Be aware that this file may contain sensitive information
</usage_guidelines>

<notes>
- Files were filtered by packx based on content and extension matching
- Total files included: 3
</notes>
</file_summary>

<directory_structure>
src/app_impl.rs
src/config/mod.rs
src/hotkeys.rs
</directory_structure>

<files>
This section contains the contents of the repository's files.

<file path="src/app_impl.rs">
impl ScriptListApp {
    fn new(config: config::Config, bun_available: bool, window: &mut Window, cx: &mut Context<Self>) -> Self {
        // PERF: Measure script loading time
        let load_start = std::time::Instant::now();
        let scripts = scripts::read_scripts();
        let scripts_elapsed = load_start.elapsed();

        let scriptlets_start = std::time::Instant::now();
        let scriptlets = scripts::read_scriptlets();
        let scriptlets_elapsed = scriptlets_start.elapsed();

        let theme = theme::load_theme();
        // Config is now passed in from main() to avoid duplicate load (~100-300ms savings)

        // Load frecency data for suggested section tracking
        let suggested_config = config.get_suggested();
        let mut frecency_store = FrecencyStore::with_config(&suggested_config);
        frecency_store.load().ok(); // Ignore errors - starts fresh if file doesn't exist

        // Load built-in entries based on config
        let builtin_entries = builtins::get_builtin_entries(&config.get_builtins());

        // Apps are loaded in the background to avoid blocking startup
        // Start with empty list, will be populated asynchronously
        let apps = Vec::new();

        let total_elapsed = load_start.elapsed();
        logging::log("PERF", &format!(
            "Startup loading: {:.2}ms total ({} scripts in {:.2}ms, {} scriptlets in {:.2}ms, apps loading in background)",
            total_elapsed.as_secs_f64() * 1000.0,
            scripts.len(),
            scripts_elapsed.as_secs_f64() * 1000.0,
            scriptlets.len(),
            scriptlets_elapsed.as_secs_f64() * 1000.0
        ));
        logging::log(
            "APP",
            &format!("Loaded {} scripts from ~/.scriptkit/scripts", scripts.len()),
        );
        logging::log(
            "APP",
            &format!(
                "Loaded {} scriptlets from ~/.scriptkit/scriptlets/scriptlets.md",
                scriptlets.len()
            ),
        );
        logging::log(
            "APP",
            &format!("Loaded {} built-in features", builtin_entries.len()),
        );
        logging::log("APP", "Applications loading in background...");
        logging::log("APP", "Loaded theme with system appearance detection");
        logging::log(
            "APP",
            &format!(
                "Loaded config: hotkey={:?}+{}, bun_path={:?}",
                config.hotkey.modifiers, config.hotkey.key, config.bun_path
            ),
        );

        // Load apps in background thread to avoid blocking startup
        let app_launcher_enabled = config.get_builtins().app_launcher;
        if app_launcher_enabled {
            // Use a channel to send loaded apps back to main thread
            let (tx, rx) =
                std::sync::mpsc::channel::<(Vec<app_launcher::AppInfo>, std::time::Duration)>();

            // Spawn background thread for app scanning
            std::thread::spawn(move || {
                let start = std::time::Instant::now();
                let apps = app_launcher::scan_applications().clone();
                let elapsed = start.elapsed();
                let _ = tx.send((apps, elapsed));
            });

            // Poll for results using a spawned task
            cx.spawn(async move |this, cx| {
                // Poll the channel periodically
                loop {
                    Timer::after(std::time::Duration::from_millis(50)).await;
                    match rx.try_recv() {
                        Ok((apps, elapsed)) => {
                            let app_count = apps.len();
                            let _ = cx.update(|cx| {
                                this.update(cx, |app, cx| {
                                    app.apps = apps;
                                    // Invalidate caches since apps changed
                                    app.filter_cache_key = String::from("\0_APPS_LOADED_\0");
                                    app.grouped_cache_key = String::from("\0_APPS_LOADED_\0");
                                    logging::log(
                                        "APP",
                                        &format!(
                                            "Background app loading complete: {} apps in {:.2}ms",
                                            app_count,
                                            elapsed.as_secs_f64() * 1000.0
                                        ),
                                    );
                                    cx.notify();
                                })
                            });
                            break;
                        }
                        Err(std::sync::mpsc::TryRecvError::Empty) => continue,
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
                    }
                }
            })
            .detach();
        }
        logging::log("UI", "Script Kit logo SVG loaded for header rendering");

        // Start cursor blink timer - updates all inputs that track cursor visibility
        cx.spawn(async move |this, cx| {
            loop {
                Timer::after(std::time::Duration::from_millis(530)).await;
                let _ = cx.update(|cx| {
                    this.update(cx, |app, cx| {
                        // Skip cursor blink when window is hidden or no input is focused
                        if !script_kit_gpui::is_main_window_visible()
                            || app.focused_input == FocusedInput::None
                        {
                            return;
                        }

                        app.cursor_visible = !app.cursor_visible;
                        // Also update ActionsDialog cursor if it exists
                        if let Some(ref dialog) = app.actions_dialog {
                            dialog.update(cx, |d, _cx| {
                                d.set_cursor_visible(app.cursor_visible);
                            });
                        }
                        cx.notify();
                    })
                });
            }
        })
        .detach();

        let gpui_input_state =
            cx.new(|cx| InputState::new(window, cx).placeholder(DEFAULT_PLACEHOLDER));
        let gpui_input_subscription = cx.subscribe_in(&gpui_input_state, window, {
            move |this, _, event: &InputEvent, window, cx| match event {
                InputEvent::Focus => {
                    this.gpui_input_focused = true;
                    this.focused_input = FocusedInput::MainFilter;
                    cx.notify();
                }
                InputEvent::Blur => {
                    this.gpui_input_focused = false;
                    if this.focused_input == FocusedInput::MainFilter {
                        this.focused_input = FocusedInput::None;
                    }
                    cx.notify();
                }
                InputEvent::Change => {
                    this.handle_filter_input_change(window, cx);
                }
                InputEvent::PressEnter { .. } => {
                    if matches!(this.current_view, AppView::ScriptList) && !this.show_actions_popup
                    {
                        this.execute_selected(cx);
                    }
                }
            }
        });

        let mut app = ScriptListApp {
            scripts,
            scriptlets,
            builtin_entries,
            apps,
            selected_index: 0,
            filter_text: String::new(),
            gpui_input_state,
            gpui_input_focused: false,
            gpui_input_subscriptions: vec![gpui_input_subscription],
            suppress_filter_events: false,
            pending_filter_sync: false,
            last_output: None,
            focus_handle: cx.focus_handle(),
            show_logs: false,
            theme,
            config,
            // Scroll activity tracking: start with scrollbar hidden
            is_scrolling: false,
            last_scroll_time: None,
            current_view: AppView::ScriptList,
            script_session: Arc::new(ParkingMutex::new(None)),
            arg_input: TextInputState::new(),
            arg_selected_index: 0,
            prompt_receiver: None,
            response_sender: None,
            // Variable-height list state for main menu (section headers at 24px, items at 48px)
            // Start with 0 items, will be reset when grouped_items changes
            // .measure_all() ensures all items are measured upfront for correct scroll height
            main_list_state: ListState::new(0, ListAlignment::Top, px(100.)).measure_all(),
            list_scroll_handle: UniformListScrollHandle::new(),
            arg_list_scroll_handle: UniformListScrollHandle::new(),
            clipboard_list_scroll_handle: UniformListScrollHandle::new(),
            window_list_scroll_handle: UniformListScrollHandle::new(),
            design_gallery_scroll_handle: UniformListScrollHandle::new(),
            show_actions_popup: false,
            actions_dialog: None,
            cursor_visible: true,
            focused_input: FocusedInput::MainFilter,
            current_script_pid: None,
            // P1: Initialize filter cache
            cached_filtered_results: Vec::new(),
            filter_cache_key: String::from("\0_UNINITIALIZED_\0"), // Sentinel value to force initial compute
            // P1: Initialize grouped results cache (Arc for cheap clone)
            cached_grouped_items: Arc::from([]),
            cached_grouped_flat_results: Arc::from([]),
            grouped_cache_key: String::from("\0_UNINITIALIZED_\0"), // Sentinel value to force initial compute
            // P3: Two-stage filter coalescing
            computed_filter_text: String::new(),
            filter_coalescer: FilterCoalescer::new(),
            // Scroll stabilization: start with no last scrolled index
            last_scrolled_index: None,
            // Preview cache: start empty, will populate on first render
            preview_cache_path: None,
            preview_cache_lines: Vec::new(),
            // Design system: start with default design
            current_design: DesignVariant::default(),
            // Toast manager: initialize for error notifications
            toast_manager: ToastManager::new(),
            // Clipboard image cache: decoded RenderImages for thumbnails/preview
            clipboard_image_cache: std::collections::HashMap::new(),
            // Frecency store for tracking script usage
            frecency_store,
            // Mouse hover tracking - starts as None (no item hovered)
            hovered_index: None,
            // P0-2: Initialize hover debounce timer
            last_hover_notify: std::time::Instant::now(),
            // Pending path action - starts as None (Arc<Mutex<>> for callback access)
            pending_path_action: Arc::new(Mutex::new(None)),
            // Signal to close path actions dialog
            close_path_actions: Arc::new(Mutex::new(false)),
            // Shared state: path actions dialog visibility (for toggle behavior)
            path_actions_showing: Arc::new(Mutex::new(false)),
            // Shared state: path actions search text (for header display)
            path_actions_search_text: Arc::new(Mutex::new(String::new())),
            // Pending path action result - action_id + path_info to execute
            pending_path_action_result: Arc::new(Mutex::new(None)),
            // Alias/shortcut registries - populated below
            alias_registry: std::collections::HashMap::new(),
            shortcut_registry: std::collections::HashMap::new(),
            // SDK actions - starts empty, populated by setActions() from scripts
            sdk_actions: None,
            action_shortcuts: std::collections::HashMap::new(),
            // Debug grid overlay - check env var at startup
            grid_config: if std::env::var("SCRIPT_KIT_DEBUG_GRID").is_ok() {
                logging::log("DEBUG_GRID", "SCRIPT_KIT_DEBUG_GRID env var set - enabling grid overlay");
                Some(debug_grid::GridConfig::default())
            } else {
                None
            },
            // Navigation coalescing for rapid arrow key events
            nav_coalescer: NavCoalescer::new(),
            // Window focus tracking - for detecting focus lost and auto-dismissing prompts
            was_window_focused: false,
            // Scroll stabilization: track last scrolled index for each handle
            last_scrolled_main: None,
            last_scrolled_arg: None,
            last_scrolled_clipboard: None,
            last_scrolled_window: None,
            last_scrolled_design_gallery: None,
            // Show warning banner when bun is not available
            show_bun_warning: !bun_available,
            // Pending confirmation for dangerous actions
            pending_confirmation: None,
            // Menu bar integration: Now handled by frontmost_app_tracker module
            // which pre-fetches menu items in background when apps activate
        };

        // Build initial alias/shortcut registries (conflicts logged, not shown via HUD on startup)
        let conflicts = app.rebuild_registries();
        if !conflicts.is_empty() {
            logging::log(
                "STARTUP",
                &format!(
                    "Found {} alias/shortcut conflicts on startup",
                    conflicts.len()
                ),
            );
        }

        app
    }

    /// Switch to a different design variant
    ///
    /// Cycle to the next design variant.
    /// Use Cmd+1 to cycle through all designs.
    fn cycle_design(&mut self, cx: &mut Context<Self>) {
        let old_design = self.current_design;
        let new_design = old_design.next();
        let all_designs = DesignVariant::all();
        let old_idx = all_designs
            .iter()
            .position(|&v| v == old_design)
            .unwrap_or(0);
        let new_idx = all_designs
            .iter()
            .position(|&v| v == new_design)
            .unwrap_or(0);

        logging::log(
            "DESIGN",
            &format!(
                "Cycling design: {} ({}) -> {} ({}) [total: {}]",
                old_design.name(),
                old_idx,
                new_design.name(),
                new_idx,
                all_designs.len()
            ),
        );
        logging::log(
            "DESIGN",
            &format!(
                "Design '{}': {}",
                new_design.name(),
                new_design.description()
            ),
        );

        self.current_design = new_design;
        logging::log(
            "DESIGN",
            &format!("self.current_design is now: {:?}", self.current_design),
        );
        cx.notify();
    }

    fn update_theme(&mut self, cx: &mut Context<Self>) {
        self.theme = theme::load_theme();
        logging::log("APP", "Theme reloaded based on system appearance");
        cx.notify();
    }

    fn update_config(&mut self, cx: &mut Context<Self>) {
        self.config = config::load_config();
        clipboard_history::set_max_text_content_len(
            self.config.get_clipboard_history_max_text_length(),
        );
        // Hot-reload hotkeys from updated config
        hotkeys::update_hotkeys(&self.config);
        logging::log(
            "APP",
            &format!("Config reloaded: padding={:?}", self.config.get_padding()),
        );
        cx.notify();
    }

    fn refresh_scripts(&mut self, cx: &mut Context<Self>) {
        self.scripts = scripts::read_scripts();
        self.scriptlets = scripts::read_scriptlets();
        self.selected_index = 0;
        self.last_scrolled_index = None;
        // Use main_list_state for variable-height list (not the legacy list_scroll_handle)
        self.main_list_state.scroll_to_reveal_item(0);
        self.last_scrolled_index = Some(0);
        self.invalidate_filter_cache();
        self.invalidate_grouped_cache();

        // Rebuild alias/shortcut registries and show HUD for any conflicts
        let conflicts = self.rebuild_registries();
        for conflict in conflicts {
            self.show_hud(conflict, Some(4000), cx); // 4s for conflict messages
        }

        logging::log(
            "APP",
            &format!(
                "Scripts refreshed: {} scripts, {} scriptlets loaded",
                self.scripts.len(),
                self.scriptlets.len()
            ),
        );
        cx.notify();
    }

    /// Dismiss the bun warning banner
    fn dismiss_bun_warning(&mut self, cx: &mut Context<Self>) {
        logging::log("APP", "Bun warning banner dismissed by user");
        self.show_bun_warning = false;
        cx.notify();
    }

    /// Open bun.sh in the default browser
    fn open_bun_website(&self) {
        logging::log("APP", "Opening https://bun.sh in default browser");
        if let Err(e) = std::process::Command::new("open")
            .arg("https://bun.sh")
            .spawn()
        {
            logging::log("APP", &format!("Failed to open bun.sh: {}", e));
        }
    }

    /// Handle incremental scriptlet file change
    ///
    /// Instead of reloading all scriptlets, this method:
    /// 1. Parses only the changed file
    /// 2. Diffs against cached state to find what changed
    /// 3. Updates hotkeys/expand triggers incrementally
    /// 4. Updates the scriptlets list
    ///
    /// # Arguments
    /// * `path` - Path to the changed/deleted scriptlet file
    /// * `is_deleted` - Whether the file was deleted (vs created/modified)
    /// * `cx` - The context for UI updates
    fn handle_scriptlet_file_change(
        &mut self,
        path: &std::path::Path,
        is_deleted: bool,
        cx: &mut Context<Self>,
    ) {
        use script_kit_gpui::scriptlet_cache::{diff_scriptlets, CachedScriptlet};

        logging::log(
            "APP",
            &format!(
                "Incremental scriptlet change: {} (deleted={})",
                path.display(),
                is_deleted
            ),
        );

        // Get old cached scriptlets for this file (if any)
        // Note: We're using a simple approach here - comparing name+shortcut+expand+alias
        let old_scriptlets: Vec<CachedScriptlet> = self
            .scriptlets
            .iter()
            .filter(|s| {
                s.file_path
                    .as_ref()
                    .map(|fp| fp.starts_with(&path.to_string_lossy().to_string()))
                    .unwrap_or(false)
            })
            .map(|s| {
                CachedScriptlet::new(
                    s.name.clone(),
                    s.shortcut.clone(),
                    s.expand.clone(),
                    s.alias.clone(),
                    s.file_path.clone().unwrap_or_default(),
                )
            })
            .collect();

        // Parse new scriptlets from file (empty if deleted)
        let new_scripts_scriptlets = if is_deleted {
            vec![]
        } else {
            scripts::read_scriptlets_from_file(path)
        };

        let new_scriptlets: Vec<CachedScriptlet> = new_scripts_scriptlets
            .iter()
            .map(|s| {
                CachedScriptlet::new(
                    s.name.clone(),
                    s.shortcut.clone(),
                    s.expand.clone(),
                    s.alias.clone(),
                    s.file_path.clone().unwrap_or_default(),
                )
            })
            .collect();

        // Compute diff
        let diff = diff_scriptlets(&old_scriptlets, &new_scriptlets);

        if diff.is_empty() {
            logging::log(
                "APP",
                &format!("No changes detected in {}", path.display()),
            );
            return;
        }

        logging::log(
            "APP",
            &format!(
                "Scriptlet diff: {} added, {} removed, {} shortcut changes, {} expand changes, {} alias changes",
                diff.added.len(),
                diff.removed.len(),
                diff.shortcut_changes.len(),
                diff.expand_changes.len(),
                diff.alias_changes.len()
            ),
        );

        // Apply hotkey changes
        for removed in &diff.removed {
            if removed.shortcut.is_some() {
                if let Err(e) = hotkeys::unregister_script_hotkey(&removed.file_path) {
                    logging::log(
                        "HOTKEY",
                        &format!("Failed to unregister hotkey for {}: {}", removed.name, e),
                    );
                }
            }
        }

        for added in &diff.added {
            if let Some(ref shortcut) = added.shortcut {
                if let Err(e) = hotkeys::register_script_hotkey(&added.file_path, shortcut) {
                    logging::log(
                        "HOTKEY",
                        &format!("Failed to register hotkey for {}: {}", added.name, e),
                    );
                }
            }
        }

        for change in &diff.shortcut_changes {
            if let Err(e) = hotkeys::update_script_hotkey(
                &change.file_path,
                change.old.as_deref(),
                change.new.as_deref(),
            ) {
                logging::log(
                    "HOTKEY",
                    &format!("Failed to update hotkey for {}: {}", change.name, e),
                );
            }
        }

        // Apply expand manager changes (macOS only)
        #[cfg(target_os = "macos")]
        {
            // For removed scriptlets, clear their triggers
            for removed in &diff.removed {
                if removed.expand.is_some() {
                    // We'd need access to the expand manager here
                    // For now, log that we would clear triggers
                    logging::log(
                        "EXPAND",
                        &format!("Would clear expand trigger for removed: {}", removed.name),
                    );
                }
            }

            // For added scriptlets with expand, register them
            for added in &diff.added {
                if added.expand.is_some() {
                    logging::log(
                        "EXPAND",
                        &format!("Would register expand trigger for added: {}", added.name),
                    );
                }
            }

            // For changed expand triggers, update them
            for change in &diff.expand_changes {
                logging::log(
                    "EXPAND",
                    &format!(
                        "Would update expand trigger for {}: {:?} -> {:?}",
                        change.name, change.old, change.new
                    ),
                );
            }
        }

        // Update the scriptlets list
        // Remove old scriptlets from this file
        let path_str = path.to_string_lossy().to_string();
        self.scriptlets
            .retain(|s| !s.file_path.as_ref().map(|fp| fp.starts_with(&path_str)).unwrap_or(false));

        // Add new scriptlets from this file
        self.scriptlets.extend(new_scripts_scriptlets);

        // Sort by name to maintain consistent ordering
        self.scriptlets.sort_by(|a, b| a.name.cmp(&b.name));

        // Invalidate caches
        self.invalidate_filter_cache();
        self.invalidate_grouped_cache();

        // Rebuild alias/shortcut registries for this file's scriptlets
        let conflicts = self.rebuild_registries();
        for conflict in conflicts {
            self.show_hud(conflict, Some(4000), cx);
        }

        logging::log(
            "APP",
            &format!(
                "Scriptlet file updated incrementally: {} now has {} total scriptlets",
                path.display(),
                self.scriptlets.len()
            ),
        );

        cx.notify();
    }

    /// Get unified filtered results combining scripts and scriptlets
    /// Helper to get filter text as string (for compatibility with existing code)
    fn filter_text(&self) -> &str {
        self.filter_text.as_str()
    }

    /// P1: Now uses caching - invalidates only when filter_text changes
    fn filtered_results(&self) -> Vec<scripts::SearchResult> {
        let filter_text = self.filter_text();
        // P1: Return cached results if filter hasn't changed
        if filter_text == self.filter_cache_key {
            logging::log_debug(
                "CACHE",
                &format!("Filter cache HIT for '{}'", filter_text),
            );
            return self.cached_filtered_results.clone();
        }

        // P1: Cache miss - need to recompute (will be done by get_filtered_results_mut)
        logging::log_debug(
            "CACHE",
            &format!(
                "Filter cache MISS - need recompute for '{}' (cached key: '{}')",
                filter_text, self.filter_cache_key
            ),
        );

        // PERF: Measure search time (only log when actually filtering)
        let search_start = std::time::Instant::now();
        let results =
            scripts::fuzzy_search_unified(&self.scripts, &self.scriptlets, filter_text);
        let search_elapsed = search_start.elapsed();

        // Only log search performance when there's an active filter
        if !filter_text.is_empty() {
            logging::log(
                "PERF",
                &format!(
                    "Search '{}' took {:.2}ms ({} results from {} total)",
                    filter_text,
                    search_elapsed.as_secs_f64() * 1000.0,
                    results.len(),
                    self.scripts.len() + self.scriptlets.len()
                ),
            );
        }
        results
    }

    /// P1: Get filtered results with cache update (mutable version)
    /// Call this when you need to ensure cache is updated
    fn get_filtered_results_cached(&mut self) -> &Vec<scripts::SearchResult> {
        if self.filter_text != self.filter_cache_key {
            logging::log_debug(
                "CACHE",
                &format!("Filter cache MISS - recomputing for '{}'", self.filter_text),
            );
            let search_start = std::time::Instant::now();
            self.cached_filtered_results = scripts::fuzzy_search_unified_all(
                &self.scripts,
                &self.scriptlets,
                &self.builtin_entries,
                &self.apps,
                &self.filter_text,
            );
            self.filter_cache_key = self.filter_text.clone();
            let search_elapsed = search_start.elapsed();

            if !self.filter_text.is_empty() {
                logging::log(
                    "PERF",
                    &format!(
                        "Search '{}' took {:.2}ms ({} results from {} total)",
                        self.filter_text,
                        search_elapsed.as_secs_f64() * 1000.0,
                        self.cached_filtered_results.len(),
                        self.scripts.len()
                            + self.scriptlets.len()
                            + self.builtin_entries.len()
                            + self.apps.len()
                    ),
                );
            }
        } else {
            logging::log_debug(
                "CACHE",
                &format!("Filter cache HIT for '{}'", self.filter_text),
            );
        }
        &self.cached_filtered_results
    }

    /// P1: Invalidate filter cache (call when scripts/scriptlets change)
    #[allow(dead_code)]
    fn invalidate_filter_cache(&mut self) {
        logging::log_debug("CACHE", "Filter cache INVALIDATED");
        self.filter_cache_key = String::from("\0_INVALIDATED_\0");
    }

    /// P1: Get grouped results with caching - avoids recomputing 9+ times per keystroke
    ///
    /// This is the ONLY place that should call scripts::get_grouped_results().
    /// P3: Cache is keyed off computed_filter_text (not filter_text) for two-stage filtering.
    ///
    /// P1-Arc: Returns Arc clones for cheap sharing with render closures.
    fn get_grouped_results_cached(
        &mut self,
    ) -> (Arc<[GroupedListItem]>, Arc<[scripts::SearchResult]>) {
        // P3: Key off computed_filter_text for two-stage filtering
        if self.computed_filter_text == self.grouped_cache_key {
            logging::log_debug(
                "CACHE",
                &format!("Grouped cache HIT for '{}'", self.computed_filter_text),
            );
            return (
                self.cached_grouped_items.clone(),
                self.cached_grouped_flat_results.clone(),
            );
        }

        // Cache miss - need to recompute
        logging::log_debug(
            "CACHE",
            &format!(
                "Grouped cache MISS - recomputing for '{}'",
                self.computed_filter_text
            ),
        );

        let start = std::time::Instant::now();
        let suggested_config = self.config.get_suggested();
        
        // Get menu bar items from the background tracker (pre-fetched when apps activate)
        #[cfg(target_os = "macos")]
        let (menu_bar_items, menu_bar_bundle_id): (Vec<menu_bar::MenuBarItem>, Option<String>) = {
            let cached = frontmost_app_tracker::get_cached_menu_items();
            let bundle_id = frontmost_app_tracker::get_last_real_app().map(|a| a.bundle_id);
            // No conversion needed - tracker is compiled as part of binary crate
            // so it already returns binary crate types
            (cached, bundle_id)
        };
        #[cfg(not(target_os = "macos"))]
        let (menu_bar_items, menu_bar_bundle_id): (Vec<menu_bar::MenuBarItem>, Option<String>) = (Vec::new(), None);
        
        logging::log("APP", &format!(
            "get_grouped_results: filter='{}', menu_bar_items={}, bundle_id={:?}",
            self.computed_filter_text,
            menu_bar_items.len(),
            menu_bar_bundle_id
        ));
        let (grouped_items, flat_results) = get_grouped_results(
            &self.scripts,
            &self.scriptlets,
            &self.builtin_entries,
            &self.apps,
            &self.frecency_store,
            &self.computed_filter_text,
            &suggested_config,
            &menu_bar_items,
            menu_bar_bundle_id.as_deref(),
        );
        let elapsed = start.elapsed();

        // P1-Arc: Convert to Arc<[T]> for cheap clone
        self.cached_grouped_items = grouped_items.into();
        self.cached_grouped_flat_results = flat_results.into();
        self.grouped_cache_key = self.computed_filter_text.clone();

        if !self.computed_filter_text.is_empty() {
            logging::log_debug(
                "CACHE",
                &format!(
                    "Grouped results computed in {:.2}ms for '{}' ({} items)",
                    elapsed.as_secs_f64() * 1000.0,
                    self.computed_filter_text,
                    self.cached_grouped_items.len()
                ),
            );
        }

        (
            self.cached_grouped_items.clone(),
            self.cached_grouped_flat_results.clone(),
        )
    }

    /// P1: Invalidate grouped results cache (call when scripts/scriptlets/apps change)
    fn invalidate_grouped_cache(&mut self) {
        logging::log_debug("CACHE", "Grouped cache INVALIDATED");
        self.grouped_cache_key = String::from("\0_INVALIDATED_\0");
        // Also reset computed_filter_text to force recompute
        self.computed_filter_text = String::from("\0_INVALIDATED_\0");
    }

    /// Get the currently selected search result, correctly mapping from grouped index.
    ///
    /// This function handles the mapping from `selected_index` (which is the visual
    /// position in the grouped list including section headers) to the actual
    /// `SearchResult` in the flat results array.
    ///
    /// Returns `None` if:
    /// - The selected index points to a section header (headers aren't selectable)
    /// - The selected index is out of bounds
    /// - No results exist
    fn get_selected_result(&mut self) -> Option<scripts::SearchResult> {
        let selected_index = self.selected_index;
        let (grouped_items, flat_results) = self.get_grouped_results_cached();

        match grouped_items.get(selected_index) {
            Some(GroupedListItem::Item(idx)) => flat_results.get(*idx).cloned(),
            _ => None,
        }
    }

    /// Get or update the preview cache for syntax-highlighted code lines.
    /// Only re-reads and re-highlights when the script path actually changes.
    /// Returns cached lines if path matches, otherwise updates cache and returns new lines.
    fn get_or_update_preview_cache(
        &mut self,
        script_path: &str,
        lang: &str,
    ) -> &[syntax::HighlightedLine] {
        // Check if cache is valid for this path
        if self.preview_cache_path.as_deref() == Some(script_path)
            && !self.preview_cache_lines.is_empty()
        {
            logging::log_debug("CACHE", &format!("Preview cache HIT for '{}'", script_path));
            return &self.preview_cache_lines;
        }

        // Cache miss - need to re-read and re-highlight
        logging::log_debug(
            "CACHE",
            &format!("Preview cache MISS - loading '{}'", script_path),
        );

        self.preview_cache_path = Some(script_path.to_string());
        self.preview_cache_lines = match std::fs::read_to_string(script_path) {
            Ok(content) => {
                // Only take first 15 lines for preview
                let preview: String = content.lines().take(15).collect::<Vec<_>>().join("\n");
                syntax::highlight_code_lines(&preview, lang)
            }
            Err(e) => {
                logging::log("ERROR", &format!("Failed to read preview: {}", e));
                Vec::new()
            }
        };

        &self.preview_cache_lines
    }

    /// Invalidate the preview cache (call when selection might change to different script)
    #[allow(dead_code)]
    fn invalidate_preview_cache(&mut self) {
        self.preview_cache_path = None;
        self.preview_cache_lines.clear();
    }

    #[allow(dead_code)]
    fn filtered_scripts(&self) -> Vec<Arc<scripts::Script>> {
        let filter_text = self.filter_text();
        if filter_text.is_empty() {
            self.scripts.clone()
        } else {
            let filter_lower = filter_text.to_lowercase();
            self.scripts
                .iter()
                .filter(|s| s.name.to_lowercase().contains(&filter_lower))
                .cloned()
                .collect()
        }
    }

    /// Find a script or scriptlet by alias (case-insensitive exact match)
    /// Uses O(1) registry lookup instead of O(n) iteration
    fn find_alias_match(&self, alias: &str) -> Option<AliasMatch> {
        let alias_lower = alias.to_lowercase();

        // O(1) lookup in registry
        if let Some(path) = self.alias_registry.get(&alias_lower) {
            // Find the script/scriptlet by path
            for script in &self.scripts {
                if script.path.to_string_lossy() == *path {
                    logging::log(
                        "ALIAS",
                        &format!("Found script match: '{}' -> '{}'", alias, script.name),
                    );
                    return Some(AliasMatch::Script(script.clone()));
                }
            }

            // Check scriptlets by file_path or name
            for scriptlet in &self.scriptlets {
                let scriptlet_path = scriptlet.file_path.as_ref().unwrap_or(&scriptlet.name);
                if scriptlet_path == path {
                    logging::log(
                        "ALIAS",
                        &format!("Found scriptlet match: '{}' -> '{}'", alias, scriptlet.name),
                    );
                    return Some(AliasMatch::Scriptlet(scriptlet.clone()));
                }
            }

            // Path in registry but not found in current scripts (stale entry)
            logging::log(
                "ALIAS",
                &format!(
                    "Stale registry entry: '{}' -> '{}' (not found)",
                    alias, path
                ),
            );
        }

        None
    }


    fn execute_selected(&mut self, cx: &mut Context<Self>) {
        // Get grouped results to map from selected_index to actual result (cached)
        let (grouped_items, flat_results) = self.get_grouped_results_cached();
        // Clone to avoid borrow issues with self mutation below
        let grouped_items = grouped_items.clone();
        let flat_results = flat_results.clone();

        // Get the grouped item at selected_index and extract the result index
        let result_idx = match grouped_items.get(self.selected_index) {
            Some(GroupedListItem::Item(idx)) => Some(*idx),
            Some(GroupedListItem::SectionHeader(_)) => None, // Section headers are not selectable
            None => None,
        };

        if let Some(idx) = result_idx {
            if let Some(result) = flat_results.get(idx).cloned() {
                // Record frecency usage before executing (unless excluded)
                let frecency_path: Option<String> = match &result {
                    scripts::SearchResult::Script(sm) => {
                        Some(sm.script.path.to_string_lossy().to_string())
                    }
                    scripts::SearchResult::App(am) => {
                        Some(am.app.path.to_string_lossy().to_string())
                    }
                    scripts::SearchResult::BuiltIn(bm) => {
                        // Skip frecency tracking for excluded builtins (e.g., "Quit Script Kit")
                        let excluded = &self.config.get_suggested().excluded_commands;
                        if bm.entry.should_exclude_from_frecency(excluded) {
                            None
                        } else {
                            Some(format!("builtin:{}", bm.entry.name))
                        }
                    }
                    scripts::SearchResult::Scriptlet(sm) => {
                        Some(format!("scriptlet:{}", sm.scriptlet.name))
                    }
                    scripts::SearchResult::Window(wm) => {
                        Some(format!("window:{}:{}", wm.window.app, wm.window.title))
                    }
                    scripts::SearchResult::Agent(am) => {
                        Some(format!("agent:{}", am.agent.path.to_string_lossy()))
                    }
                };
                if let Some(path) = frecency_path {
                    self.frecency_store.record_use(&path);
                    self.frecency_store.save().ok(); // Best-effort save
                    self.invalidate_grouped_cache(); // Invalidate cache so next show reflects frecency
                }

                match result {
                    scripts::SearchResult::Script(script_match) => {
                        logging::log(
                            "EXEC",
                            &format!("Executing script: {}", script_match.script.name),
                        );
                        self.execute_interactive(&script_match.script, cx);
                    }
                    scripts::SearchResult::Scriptlet(scriptlet_match) => {
                        logging::log(
                            "EXEC",
                            &format!("Executing scriptlet: {}", scriptlet_match.scriptlet.name),
                        );
                        self.execute_scriptlet(&scriptlet_match.scriptlet, cx);
                    }
                    scripts::SearchResult::BuiltIn(builtin_match) => {
                        logging::log(
                            "EXEC",
                            &format!("Executing built-in: {}", builtin_match.entry.name),
                        );
                        self.execute_builtin(&builtin_match.entry, cx);
                    }
                    scripts::SearchResult::App(app_match) => {
                        logging::log("EXEC", &format!("Launching app: {}", app_match.app.name));
                        self.execute_app(&app_match.app, cx);
                    }
                    scripts::SearchResult::Window(window_match) => {
                        logging::log(
                            "EXEC",
                            &format!("Focusing window: {}", window_match.window.title),
                        );
                        self.execute_window_focus(&window_match.window, cx);
                    }
                    scripts::SearchResult::Agent(agent_match) => {
                        logging::log(
                            "EXEC",
                            &format!("Agent selected: {}", agent_match.agent.name),
                        );
                        // TODO: Implement agent execution via mdflow
                        self.last_output = Some(SharedString::from(format!(
                            "Agent execution not yet implemented: {}",
                            agent_match.agent.name
                        )));
                    }
                }
            }
        }
    }

    fn handle_filter_input_change(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.suppress_filter_events {
            return;
        }

        let new_text = self.gpui_input_state.read(cx).value().to_string();
        if new_text == self.filter_text {
            return;
        }

        // Clear pending confirmation when typing (user is changing context)
        if self.pending_confirmation.is_some() {
            self.pending_confirmation = None;
        }

        let previous_text = std::mem::replace(&mut self.filter_text, new_text.clone());
        self.selected_index = 0;
        self.last_scrolled_index = None;
        // Use main_list_state for variable-height list (not the legacy list_scroll_handle)
        self.main_list_state.scroll_to_reveal_item(0);
        self.last_scrolled_index = Some(0);

        if new_text.ends_with(' ') {
            let trimmed = new_text.trim_end_matches(' ');
            if !trimmed.is_empty() && trimmed == previous_text {
                if let Some(alias_match) = self.find_alias_match(trimmed) {
                    logging::log(
                        "ALIAS",
                        &format!("Alias '{}' triggered execution", trimmed),
                    );
                    match alias_match {
                        AliasMatch::Script(script) => {
                            self.execute_interactive(&script, cx);
                        }
                        AliasMatch::Scriptlet(scriptlet) => {
                            self.execute_scriptlet(&scriptlet, cx);
                        }
                    }
                    self.clear_filter(window, cx);
                    return;
                }
            }
        }

        // P3: Notify immediately so UI updates (responsive typing)
        cx.notify();

        // Menu bar items are now pre-fetched by frontmost_app_tracker
        // No lazy loading needed - items are already in cache when we open

        self.queue_filter_compute(new_text, cx);
    }

    fn queue_filter_compute(&mut self, value: String, cx: &mut Context<Self>) {
        // P3: Debounce expensive search/window resize work.
        // Use 8ms debounce (half a frame) to batch rapid keystrokes.
        if self.filter_coalescer.queue(value) {
            cx.spawn(async move |this, cx| {
                // Wait 8ms for coalescing window (half frame at 60fps)
                Timer::after(std::time::Duration::from_millis(8)).await;

                let _ = cx.update(|cx| {
                    this.update(cx, |app, cx| {
                        if let Some(latest) = app.filter_coalescer.take_latest() {
                            if app.computed_filter_text != latest {
                                app.computed_filter_text = latest;
                                // This will trigger cache recompute on next get_grouped_results_cached()
                                app.update_window_size();
                                cx.notify();
                            }
                        }
                    })
                });
            })
            .detach();
        }
    }

    fn set_filter_text_immediate(
        &mut self,
        text: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.suppress_filter_events = true;
        self.filter_text = text.clone();
        self.gpui_input_state.update(cx, |state, cx| {
            state.set_value(text.clone(), window, cx);
        });
        self.suppress_filter_events = false;
        self.pending_filter_sync = false;

        self.selected_index = 0;
        self.last_scrolled_index = None;
        self.main_list_state.scroll_to_reveal_item(0);
        self.last_scrolled_index = Some(0);

        // Menu bar items are now pre-fetched by frontmost_app_tracker
        // No lazy loading needed - items are already in cache when we open

        self.computed_filter_text = text;
        self.filter_coalescer.reset();
        self.update_window_size();
        cx.notify();
    }

    fn clear_filter(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.set_filter_text_immediate(String::new(), window, cx);
    }

    fn sync_filter_input_if_needed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.pending_filter_sync {
            return;
        }

        let desired = self.filter_text.clone();
        let current = self.gpui_input_state.read(cx).value().to_string();
        if current == desired {
            self.pending_filter_sync = false;
            return;
        }

        self.suppress_filter_events = true;
        self.gpui_input_state.update(cx, |state, cx| {
            state.set_value(desired.clone(), window, cx);
        });
        self.suppress_filter_events = false;
        self.pending_filter_sync = false;
    }

    fn toggle_logs(&mut self, cx: &mut Context<Self>) {
        self.show_logs = !self.show_logs;
        cx.notify();
    }

    /// Update window size based on current view and item count.
    /// This implements dynamic window resizing:
    /// - Script list: resize based on filtered results (including section headers)
    /// - Arg prompt: resize based on filtered choices
    /// - Div/Editor/Term: use full height
    fn update_window_size(&mut self) {
        let (view_type, item_count) = match &self.current_view {
            AppView::ScriptList => {
                // Get grouped results which includes section headers (cached)
                let (grouped_items, _) = self.get_grouped_results_cached();
                let count = grouped_items.len();
                (ViewType::ScriptList, count)
            }
            AppView::ArgPrompt { choices, .. } => {
                let filtered = self.get_filtered_arg_choices(choices);
                if filtered.is_empty() && choices.is_empty() {
                    (ViewType::ArgPromptNoChoices, 0)
                } else {
                    (ViewType::ArgPromptWithChoices, filtered.len())
                }
            }
            AppView::DivPrompt { .. } => (ViewType::DivPrompt, 0),
            AppView::FormPrompt { .. } => (ViewType::DivPrompt, 0), // Use DivPrompt size for forms
            AppView::EditorPrompt { .. } => {
                (ViewType::EditorPrompt, 0)
            }
            AppView::SelectPrompt { .. } => (ViewType::ArgPromptWithChoices, 0),
            AppView::PathPrompt { .. } => (ViewType::DivPrompt, 0),
            AppView::EnvPrompt { .. } => (ViewType::ArgPromptNoChoices, 0), // Env prompt is a simple input
            AppView::DropPrompt { .. } => (ViewType::DivPrompt, 0), // Drop prompt uses div size for drop zone
            AppView::TemplatePrompt { .. } => (ViewType::DivPrompt, 0), // Template prompt uses div size
            AppView::TermPrompt { .. } => (ViewType::TermPrompt, 0),
            AppView::ActionsDialog => {
                // Actions dialog is an overlay, don't resize
                return;
            }
            // Clipboard history and app launcher use standard height (same as script list)
            AppView::ClipboardHistoryView {
                entries, filter, ..
            } => {
                let filtered_count = if filter.is_empty() {
                    entries.len()
                } else {
                    let filter_lower = filter.to_lowercase();
                    entries
                        .iter()
                        .filter(|e| e.text_preview.to_lowercase().contains(&filter_lower))
                        .count()
                };
                (ViewType::ScriptList, filtered_count)
            }
            AppView::AppLauncherView { apps, filter, .. } => {
                let filtered_count = if filter.is_empty() {
                    apps.len()
                } else {
                    let filter_lower = filter.to_lowercase();
                    apps.iter()
                        .filter(|a| a.name.to_lowercase().contains(&filter_lower))
                        .count()
                };
                (ViewType::ScriptList, filtered_count)
            }
            AppView::WindowSwitcherView {
                windows, filter, ..
            } => {
                let filtered_count = if filter.is_empty() {
                    windows.len()
                } else {
                    let filter_lower = filter.to_lowercase();
                    windows
                        .iter()
                        .filter(|w| {
                            w.title.to_lowercase().contains(&filter_lower)
                                || w.app.to_lowercase().contains(&filter_lower)
                        })
                        .count()
                };
                (ViewType::ScriptList, filtered_count)
            }
            AppView::DesignGalleryView { filter, .. } => {
                // Calculate total gallery items (separators + icons)
                let total_items = designs::separator_variations::SeparatorStyle::count()
                    + designs::icon_variations::total_icon_count();
                let filtered_count = if filter.is_empty() {
                    total_items
                } else {
                    // For now, return total - filtering can be added later
                    total_items
                };
                (ViewType::ScriptList, filtered_count)
            }
        };

        let target_height = height_for_view(view_type, item_count);
        resize_first_window_to_height(target_height);
    }

    fn set_prompt_input(&mut self, text: String, cx: &mut Context<Self>) {
        match &mut self.current_view {
            AppView::ArgPrompt { .. } => {
                self.arg_input.set_text(text);
                self.arg_selected_index = 0;
                self.arg_list_scroll_handle
                    .scroll_to_item(0, ScrollStrategy::Top);
                self.update_window_size();
                cx.notify();
            }
            AppView::PathPrompt { entity, .. } => {
                entity.update(cx, |prompt, cx| prompt.set_input(text, cx));
            }
            AppView::SelectPrompt { entity, .. } => {
                entity.update(cx, |prompt, cx| prompt.set_input(text, cx));
            }
            AppView::EnvPrompt { entity, .. } => {
                entity.update(cx, |prompt, cx| prompt.set_input(text, cx));
            }
            AppView::TemplatePrompt { entity, .. } => {
                entity.update(cx, |prompt, cx| prompt.set_input(text, cx));
            }
            AppView::FormPrompt { entity, .. } => {
                entity.update(cx, |prompt, cx| prompt.set_input(text, cx));
            }
            _ => {}
        }
    }

    /// Helper to get filtered arg choices without cloning
    fn get_filtered_arg_choices<'a>(&self, choices: &'a [Choice]) -> Vec<&'a Choice> {
        if self.arg_input.is_empty() {
            choices.iter().collect()
        } else {
            let filter = self.arg_input.text().to_lowercase();
            choices
                .iter()
                .filter(|c| c.name.to_lowercase().contains(&filter))
                .collect()
        }
    }

    fn focus_main_filter(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.focused_input = FocusedInput::MainFilter;
        let input_state = self.gpui_input_state.clone();
        input_state.update(cx, |state, cx| {
            state.focus(window, cx);
        });
    }

    fn toggle_actions(&mut self, cx: &mut Context<Self>, window: &mut Window) {
        logging::log("KEY", "Toggling actions popup");
        if self.show_actions_popup {
            // Close - return focus to main filter
            self.show_actions_popup = false;
            self.actions_dialog = None;
            self.focus_main_filter(window, cx);
            logging::log("FOCUS", "Actions closed, focus returned to MainFilter");
        } else {
            // Open - create dialog entity
            self.show_actions_popup = true;
            self.focused_input = FocusedInput::ActionsSearch;
            let script_info = self.get_focused_script_info();

            let theme_arc = std::sync::Arc::new(self.theme.clone());
            let dialog = cx.new(|cx| {
                let focus_handle = cx.focus_handle();
                ActionsDialog::with_script(
                    focus_handle,
                    std::sync::Arc::new(|_action_id| {}), // Callback handled separately
                    script_info,
                    theme_arc,
                )
            });

            // Hide the dialog's built-in search input since header already has search
            dialog.update(cx, |d, _| d.set_hide_search(true));

            // Focus the dialog's internal focus handle
            let dialog_focus_handle = dialog.read(cx).focus_handle.clone();
            self.actions_dialog = Some(dialog.clone());
            window.focus(&dialog_focus_handle, cx);
            logging::log("FOCUS", "Actions opened, focus moved to ActionsSearch");
        }
        cx.notify();
    }

    /// Toggle actions dialog for arg prompts with SDK-defined actions
    fn toggle_arg_actions(&mut self, cx: &mut Context<Self>, window: &mut Window) {
        logging::log(
            "KEY",
            &format!(
                "toggle_arg_actions called: show_actions_popup={}, actions_dialog.is_some={}, sdk_actions.is_some={}",
                self.show_actions_popup,
                self.actions_dialog.is_some(),
                self.sdk_actions.is_some()
            ),
        );
        if self.show_actions_popup {
            // Close - return focus to arg prompt
            self.show_actions_popup = false;
            self.actions_dialog = None;
            self.focused_input = FocusedInput::ArgPrompt;
            window.focus(&self.focus_handle, cx);
            logging::log("FOCUS", "Arg actions closed, focus returned to ArgPrompt");
        } else {
            // Check if we have SDK actions
            if let Some(ref sdk_actions) = self.sdk_actions {
                logging::log("KEY", &format!("SDK actions count: {}", sdk_actions.len()));
                if !sdk_actions.is_empty() {
                    // Open - create dialog entity with SDK actions
                    self.show_actions_popup = true;
                    self.focused_input = FocusedInput::ActionsSearch;

                    let theme_arc = std::sync::Arc::new(self.theme.clone());
                    let sdk_actions_clone = sdk_actions.clone();
                    let dialog = cx.new(|cx| {
                        let focus_handle = cx.focus_handle();
                        let mut dialog = ActionsDialog::with_script(
                            focus_handle,
                            std::sync::Arc::new(|_action_id| {}), // Callback handled separately
                            None,                                 // No script info for arg prompts
                            theme_arc,
                        );
                        // Set SDK actions to replace built-in actions
                        dialog.set_sdk_actions(sdk_actions_clone);
                        dialog
                    });

                    // Hide the dialog's built-in search input since header already has search
                    dialog.update(cx, |d, _| d.set_hide_search(true));

                    // Focus the dialog's internal focus handle
                    let dialog_focus_handle = dialog.read(cx).focus_handle.clone();
                    self.actions_dialog = Some(dialog.clone());
                    window.focus(&dialog_focus_handle, cx);
                    logging::log(
                        "FOCUS",
                        &format!(
                            "Arg actions OPENED: show_actions_popup={}, actions_dialog.is_some={}",
                            self.show_actions_popup,
                            self.actions_dialog.is_some()
                        ),
                    );
                } else {
                    logging::log("KEY", "No SDK actions available to show (empty list)");
                }
            } else {
                logging::log("KEY", "No SDK actions defined for this arg prompt (None)");
            }
        }
        cx.notify();
    }


    /// Edit a script in configured editor (config.editor > $EDITOR > "code")
    #[allow(dead_code)]
    fn edit_script(&mut self, path: &std::path::Path) {
        let editor = self.config.get_editor();
        logging::log(
            "UI",
            &format!("Opening script in editor '{}': {}", editor, path.display()),
        );
        let path_str = path.to_string_lossy().to_string();

        std::thread::spawn(move || {
            use std::process::Command;
            match Command::new(&editor).arg(&path_str).spawn() {
                Ok(_) => logging::log("UI", &format!("Successfully spawned editor: {}", editor)),
                Err(e) => logging::log(
                    "ERROR",
                    &format!("Failed to spawn editor '{}': {}", editor, e),
                ),
            }
        });
    }

    /// Execute a path action from the actions dialog
    /// Handles actions like copy_path, open_in_finder, open_in_editor, etc.
    fn execute_path_action(
        &mut self,
        action_id: &str,
        path_info: &PathInfo,
        path_prompt_entity: &Entity<PathPrompt>,
        cx: &mut Context<Self>,
    ) {
        logging::log(
            "UI",
            &format!(
                "Executing path action '{}' for: {} (is_dir={})",
                action_id, path_info.path, path_info.is_dir
            ),
        );

        match action_id {
            "select_file" | "open_directory" => {
                // For select/open, trigger submission through the path prompt
                // We need to trigger the submit callback with this path
                path_prompt_entity.update(cx, |prompt, cx| {
                    // Find the index of this path in filtered_entries and submit it
                    if let Some(idx) = prompt
                        .filtered_entries
                        .iter()
                        .position(|e| e.path == path_info.path)
                    {
                        prompt.selected_index = idx;
                    }
                    // For directories, navigate into them; for files, submit
                    if path_info.is_dir && action_id == "open_directory" {
                        prompt.navigate_to(&path_info.path, cx);
                    } else {
                        // Submit the selected path
                        let id = prompt.id.clone();
                        let path = path_info.path.clone();
                        (prompt.on_submit)(id, Some(path));
                    }
                });
            }
            "copy_path" => {
                // Copy full path to clipboard
                #[cfg(target_os = "macos")]
                {
                    use std::io::Write;
                    use std::process::{Command, Stdio};

                    match Command::new("pbcopy").stdin(Stdio::piped()).spawn() {
                        Ok(mut child) => {
                            if let Some(ref mut stdin) = child.stdin {
                                if stdin.write_all(path_info.path.as_bytes()).is_ok() {
                                    let _ = child.wait();
                                    logging::log(
                                        "UI",
                                        &format!("Copied path to clipboard: {}", path_info.path),
                                    );
                                    self.last_output = Some(SharedString::from(format!(
                                        "Copied: {}",
                                        path_info.path
                                    )));
                                } else {
                                    logging::log("ERROR", "Failed to write to pbcopy stdin");
                                    self.last_output =
                                        Some(SharedString::from("Failed to copy path"));
                                }
                            }
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to spawn pbcopy: {}", e));
                            self.last_output = Some(SharedString::from("Failed to copy path"));
                        }
                    }
                }

                #[cfg(not(target_os = "macos"))]
                {
                    use arboard::Clipboard;
                    match Clipboard::new() {
                        Ok(mut clipboard) => match clipboard.set_text(&path_info.path) {
                            Ok(_) => {
                                logging::log(
                                    "UI",
                                    &format!("Copied path to clipboard: {}", path_info.path),
                                );
                                self.last_output =
                                    Some(SharedString::from(format!("Copied: {}", path_info.path)));
                            }
                            Err(e) => {
                                logging::log("ERROR", &format!("Failed to copy path: {}", e));
                                self.last_output = Some(SharedString::from("Failed to copy path"));
                            }
                        },
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to access clipboard: {}", e));
                            self.last_output =
                                Some(SharedString::from("Failed to access clipboard"));
                        }
                    }
                }
            }
            "copy_filename" => {
                // Copy just the filename to clipboard
                #[cfg(target_os = "macos")]
                {
                    use std::io::Write;
                    use std::process::{Command, Stdio};

                    match Command::new("pbcopy").stdin(Stdio::piped()).spawn() {
                        Ok(mut child) => {
                            if let Some(ref mut stdin) = child.stdin {
                                if stdin.write_all(path_info.name.as_bytes()).is_ok() {
                                    let _ = child.wait();
                                    logging::log(
                                        "UI",
                                        &format!(
                                            "Copied filename to clipboard: {}",
                                            path_info.name
                                        ),
                                    );
                                    self.last_output = Some(SharedString::from(format!(
                                        "Copied: {}",
                                        path_info.name
                                    )));
                                }
                            }
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to spawn pbcopy: {}", e));
                        }
                    }
                }
            }
            "open_in_finder" => {
                // Reveal in Finder (macOS)
                #[cfg(target_os = "macos")]
                {
                    use std::process::Command;
                    let path_to_reveal = if path_info.is_dir {
                        path_info.path.clone()
                    } else {
                        // For files, reveal the containing folder with the file selected
                        path_info.path.clone()
                    };

                    match Command::new("open").args(["-R", &path_to_reveal]).spawn() {
                        Ok(_) => {
                            logging::log("UI", &format!("Revealed in Finder: {}", path_info.path));
                            // Hide window and set reset flag after opening external app
                            script_kit_gpui::set_main_window_visible(false);
                            NEEDS_RESET.store(true, Ordering::SeqCst);
                            cx.hide();
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to reveal in Finder: {}", e));
                            self.last_output =
                                Some(SharedString::from("Failed to reveal in Finder"));
                        }
                    }
                }
            }
            "open_in_editor" => {
                // Open in configured editor
                let editor = self.config.get_editor();
                let path_str = path_info.path.clone();
                logging::log(
                    "UI",
                    &format!("Opening in editor '{}': {}", editor, path_str),
                );

                match std::process::Command::new(&editor).arg(&path_str).spawn() {
                    Ok(_) => {
                        logging::log("UI", &format!("Opened in editor: {}", path_str));
                        // Hide window and set reset flag after opening external app
                        script_kit_gpui::set_main_window_visible(false);
                        NEEDS_RESET.store(true, Ordering::SeqCst);
                        cx.hide();
                    }
                    Err(e) => {
                        logging::log("ERROR", &format!("Failed to open in editor: {}", e));
                        self.last_output = Some(SharedString::from("Failed to open in editor"));
                    }
                }
            }
            "open_in_terminal" => {
                // Open terminal at this location
                #[cfg(target_os = "macos")]
                {
                    use std::process::Command;
                    // Get the directory (if file, use parent directory)
                    let dir_path = if path_info.is_dir {
                        path_info.path.clone()
                    } else {
                        std::path::Path::new(&path_info.path)
                            .parent()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|| path_info.path.clone())
                    };

                    // Try iTerm first, fall back to Terminal.app
                    let script = format!(
                        r#"tell application "Terminal"
                            do script "cd '{}'"
                            activate
                        end tell"#,
                        dir_path
                    );

                    match Command::new("osascript").args(["-e", &script]).spawn() {
                        Ok(_) => {
                            logging::log("UI", &format!("Opened terminal at: {}", dir_path));
                            // Hide window and set reset flag after opening external app
                            script_kit_gpui::set_main_window_visible(false);
                            NEEDS_RESET.store(true, Ordering::SeqCst);
                            cx.hide();
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to open terminal: {}", e));
                            self.last_output = Some(SharedString::from("Failed to open terminal"));
                        }
                    }
                }
            }
            "move_to_trash" => {
                // Move to trash (macOS)
                #[cfg(target_os = "macos")]
                {
                    use std::process::Command;
                    let path_str = path_info.path.clone();
                    let name = path_info.name.clone();

                    // Use AppleScript to move to trash (preserves undo capability)
                    let script = format!(
                        r#"tell application "Finder"
                            delete POSIX file "{}"
                        end tell"#,
                        path_str
                    );

                    match Command::new("osascript").args(["-e", &script]).spawn() {
                        Ok(mut child) => {
                            // Wait for completion and check result
                            match child.wait() {
                                Ok(status) if status.success() => {
                                    logging::log("UI", &format!("Moved to trash: {}", path_str));
                                    self.last_output = Some(SharedString::from(format!(
                                        "Moved to Trash: {}",
                                        name
                                    )));
                                    // Refresh the path prompt to show the file is gone
                                    path_prompt_entity.update(cx, |prompt, cx| {
                                        let current = prompt.current_path.clone();
                                        prompt.navigate_to(&current, cx);
                                    });
                                }
                                _ => {
                                    logging::log(
                                        "ERROR",
                                        &format!("Failed to move to trash: {}", path_str),
                                    );
                                    self.last_output =
                                        Some(SharedString::from("Failed to move to Trash"));
                                }
                            }
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to spawn trash command: {}", e));
                            self.last_output = Some(SharedString::from("Failed to move to Trash"));
                        }
                    }
                }
            }
            _ => {
                logging::log("UI", &format!("Unknown path action: {}", action_id));
            }
        }

        cx.notify();
    }

    /// Execute a scriptlet (simple code snippet from .md file)
    fn execute_scriptlet(&mut self, scriptlet: &scripts::Scriptlet, cx: &mut Context<Self>) {
        logging::log(
            "EXEC",
            &format!(
                "Executing scriptlet: {} (tool: {})",
                scriptlet.name, scriptlet.tool
            ),
        );

        let tool = scriptlet.tool.to_lowercase();

        // TypeScript/Kit scriptlets need to run interactively (they may use SDK prompts)
        // These should be spawned like regular scripts, not run synchronously
        if matches!(tool.as_str(), "kit" | "ts" | "bun" | "deno" | "js") {
            logging::log(
                "EXEC",
                &format!(
                    "TypeScript scriptlet '{}' - running interactively",
                    scriptlet.name
                ),
            );

            // Write scriptlet content to a temp file
            let temp_dir = std::env::temp_dir();
            let temp_file = temp_dir.join(format!(
                "scriptlet-{}-{}.ts",
                scriptlet.name.to_lowercase().replace(' ', "-"),
                std::process::id()
            ));

            if let Err(e) = std::fs::write(&temp_file, &scriptlet.code) {
                logging::log(
                    "ERROR",
                    &format!("Failed to write temp scriptlet file: {}", e),
                );
                self.toast_manager.push(
                    components::toast::Toast::error(
                        format!("Failed to write scriptlet: {}", e),
                        &self.theme,
                    )
                    .duration_ms(Some(5000)),
                );
                cx.notify();
                return;
            }

            // Create a Script struct and run it interactively
            let script = scripts::Script {
                name: scriptlet.name.clone(),
                description: scriptlet.description.clone(),
                path: temp_file,
                extension: "ts".to_string(),
                icon: None,
                alias: None,
                shortcut: None,
                typed_metadata: None,
                schema: None,
            };

            self.execute_interactive(&script, cx);
            return;
        }

        // For non-TypeScript tools (bash, python, etc.), run synchronously
        // These don't use the SDK and won't block waiting for input

        // Convert scripts::Scriptlet to scriptlets::Scriptlet for executor
        let exec_scriptlet = scriptlets::Scriptlet {
            name: scriptlet.name.clone(),
            command: scriptlet.command.clone().unwrap_or_else(|| {
                // Generate command slug from name if not present
                scriptlet.name.to_lowercase().replace(' ', "-")
            }),
            tool: scriptlet.tool.clone(),
            scriptlet_content: scriptlet.code.clone(),
            inputs: vec![], // TODO: Parse inputs from code if needed
            group: scriptlet.group.clone().unwrap_or_default(),
            preview: None,
            metadata: scriptlets::ScriptletMetadata {
                shortcut: scriptlet.shortcut.clone(),
                expand: scriptlet.expand.clone(),
                description: scriptlet.description.clone(),
                ..Default::default()
            },
            typed_metadata: None,
            schema: None,
            kit: None,
            source_path: scriptlet.file_path.clone(),
        };

        // Execute with default options (no inputs for now)
        let options = executor::ScriptletExecOptions::default();

        match executor::run_scriptlet(&exec_scriptlet, options) {
            Ok(result) => {
                if result.success {
                    logging::log(
                        "EXEC",
                        &format!(
                            "Scriptlet '{}' succeeded: exit={}",
                            scriptlet.name, result.exit_code
                        ),
                    );

                    // Handle special tool types that need interactive prompts
                    if tool == "template" && !result.stdout.is_empty() {
                        // Template tool: show template prompt with the content
                        let id = format!("scriptlet-template-{}", uuid::Uuid::new_v4());
                        logging::log(
                            "EXEC",
                            &format!(
                                "Template scriptlet '{}' - showing template prompt",
                                scriptlet.name
                            ),
                        );
                        self.handle_prompt_message(
                            PromptMessage::ShowTemplate {
                                id,
                                template: result.stdout.clone(),
                            },
                            cx,
                        );
                        return;
                    }

                    // Store output if any
                    if !result.stdout.is_empty() {
                        self.last_output = Some(SharedString::from(result.stdout.clone()));
                    }

                    // Hide window after successful execution
                    script_kit_gpui::set_main_window_visible(false);
                    cx.hide();
                } else {
                    // Execution failed (non-zero exit code)
                    let error_msg = if !result.stderr.is_empty() {
                        result.stderr.clone()
                    } else {
                        format!("Exit code: {}", result.exit_code)
                    };

                    logging::log(
                        "ERROR",
                        &format!("Scriptlet '{}' failed: {}", scriptlet.name, error_msg),
                    );

                    self.toast_manager.push(
                        components::toast::Toast::error(
                            format!("Scriptlet failed: {}", error_msg),
                            &self.theme,
                        )
                        .duration_ms(Some(5000)),
                    );
                    cx.notify();
                }
            }
            Err(e) => {
                logging::log(
                    "ERROR",
                    &format!("Failed to execute scriptlet '{}': {}", scriptlet.name, e),
                );

                self.toast_manager.push(
                    components::toast::Toast::error(
                        format!("Failed to execute: {}", e),
                        &self.theme,
                    )
                    .duration_ms(Some(5000)),
                );
                cx.notify();
            }
        }
    }

    /// Execute a script or scriptlet by its file path
    /// Used by global shortcuts to directly invoke scripts
    #[allow(dead_code)]
    fn execute_script_by_path(&mut self, path: &str, cx: &mut Context<Self>) {
        logging::log("EXEC", &format!("Executing script by path: {}", path));

        // Check if it's a scriptlet (contains #)
        if path.contains('#') {
            // It's a scriptlet path like "/path/to/file.md#command"
            if let Some(scriptlet) = self
                .scriptlets
                .iter()
                .find(|s| s.file_path.as_ref().map(|p| p == path).unwrap_or(false))
            {
                let scriptlet_clone = scriptlet.clone();
                self.execute_scriptlet(&scriptlet_clone, cx);
                return;
            }
            logging::log("ERROR", &format!("Scriptlet not found: {}", path));
            return;
        }

        // It's a regular script - find by path
        if let Some(script) = self
            .scripts
            .iter()
            .find(|s| s.path.to_string_lossy() == path)
        {
            let script_clone = script.clone();
            self.execute_interactive(&script_clone, cx);
            return;
        }

        // Not found in loaded scripts - try to execute directly as a file
        let script_path = std::path::PathBuf::from(path);
        if script_path.exists() {
            let name = script_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("script")
                .to_string();

            let script = scripts::Script {
                name,
                path: script_path.clone(),
                extension: script_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("ts")
                    .to_string(),
                description: None,
                icon: None,
                alias: None,
                shortcut: None,
                typed_metadata: None,
                schema: None,
            };

            self.execute_interactive(&script, cx);
        } else {
            logging::log("ERROR", &format!("Script file not found: {}", path));
        }
    }

    /// Cancel the currently running script and clean up all state
    fn cancel_script_execution(&mut self, cx: &mut Context<Self>) {
        logging::log("EXEC", "=== Canceling script execution ===");

        // Send cancel message to script (Exit with cancel code)
        if let Some(ref sender) = self.response_sender {
            // Try to send Exit message to terminate the script cleanly
            let exit_msg = Message::Exit {
                code: Some(1), // Non-zero code indicates cancellation
                message: Some("Cancelled by user".to_string()),
            };
            match sender.send(exit_msg) {
                Ok(()) => logging::log("EXEC", "Sent Exit message to script"),
                Err(e) => logging::log(
                    "EXEC",
                    &format!("Failed to send Exit: {} (script may have exited)", e),
                ),
            }
        } else {
            logging::log("EXEC", "No response_sender - script may not be running");
        }

        // Belt-and-suspenders: Force-kill the process group using stored PID
        // This ensures cleanup even if Drop doesn't fire properly
        if let Some(pid) = self.current_script_pid.take() {
            logging::log(
                "CLEANUP",
                &format!("Force-killing script process group {}", pid),
            );
            #[cfg(unix)]
            {
                let _ = std::process::Command::new("kill")
                    .args(["-9", &format!("-{}", pid)])
                    .output();
            }
        }

        // Abort script session if it exists
        {
            let mut session_guard = self.script_session.lock();
            if let Some(_session) = session_guard.take() {
                logging::log("EXEC", "Cleared script session");
            }
        }

        // Reset to script list view
        self.reset_to_script_list(cx);
        logging::log("EXEC", "=== Script cancellation complete ===");
    }

    /// Flush pending toasts from ToastManager to gpui-component's NotificationList
    ///
    /// This should be called at the start of render() where we have window access.
    /// The ToastManager acts as a staging queue for toasts pushed from callbacks
    /// that don't have window access.
    fn flush_pending_toasts(&mut self, window: &mut gpui::Window, cx: &mut gpui::App) {
        use gpui_component::WindowExt;

        let pending = self.toast_manager.drain_pending();
        for toast in pending {
            let notification = pending_toast_to_notification(&toast);
            window.push_notification(notification, cx);
        }
    }

    /// Close window and reset to default state (Cmd+W global handler)
    ///
    /// This method handles the global Cmd+W shortcut which should work
    /// regardless of what prompt or view is currently active. It:
    /// 1. Cancels any running script
    /// 2. Resets state to the default script list
    /// 3. Hides the window
    fn close_and_reset_window(&mut self, cx: &mut Context<Self>) {
        logging::log("VISIBILITY", "=== Close and reset window ===");

        // Update visibility state FIRST to prevent race conditions
        script_kit_gpui::set_main_window_visible(false);
        logging::log("VISIBILITY", "WINDOW_VISIBLE set to: false");

        // If in a prompt, cancel the script execution
        if self.is_in_prompt() {
            logging::log("VISIBILITY", "In prompt mode - canceling script before hiding");
            self.cancel_script_execution(cx);
        } else {
            // Just reset to script list (clears filter, selection, scroll)
            self.reset_to_script_list(cx);
        }

        // Check if Notes or AI windows are open BEFORE hiding
        let notes_open = notes::is_notes_window_open();
        let ai_open = ai::is_ai_window_open();
        logging::log(
            "VISIBILITY",
            &format!(
                "Secondary windows: notes_open={}, ai_open={}",
                notes_open, ai_open
            ),
        );

        // CRITICAL: Only hide main window if Notes/AI are open
        // cx.hide() hides the ENTIRE app (all windows), so we use
        // platform::hide_main_window() to hide only the main window
        if notes_open || ai_open {
            logging::log(
                "VISIBILITY",
                "Using hide_main_window() - secondary windows are open",
            );
            platform::hide_main_window();
        } else {
            logging::log("VISIBILITY", "Using cx.hide() - no secondary windows");
            cx.hide();
        }
        logging::log("VISIBILITY", "=== Window closed ===");
    }

    /// Handle global keyboard shortcuts with configurable dismissability
    ///
    /// Returns `true` if the shortcut was handled (caller should return early)
    ///
    /// # Arguments
    /// * `event` - The key down event to check
    /// * `is_dismissable` - If true, ESC key will also close the window (for prompts like arg, div, form, etc.)
    ///   If false, only Cmd+W closes the window (for prompts like term, editor)
    /// * `cx` - The context
    ///
    /// # Handled shortcuts
    /// - Cmd+W: Always closes window and resets to default state
    /// - Escape: Only closes window if `is_dismissable` is true AND actions popup is not showing
    fn handle_global_shortcut_with_options(
        &mut self,
        event: &gpui::KeyDownEvent,
        is_dismissable: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        let key_str = event.keystroke.key.to_lowercase();
        let has_cmd = event.keystroke.modifiers.platform;

        // Cmd+W always closes window
        if has_cmd && key_str == "w" {
            logging::log("KEY", "Cmd+W - closing window");
            self.close_and_reset_window(cx);
            return true;
        }

        // ESC closes dismissable prompts (when actions popup is not showing)
        if is_dismissable && key_str == "escape" && !self.show_actions_popup {
            logging::log("KEY", "ESC in dismissable prompt - closing window");
            self.close_and_reset_window(cx);
            return true;
        }

        false
    }

    /// Check if the current view is a dismissable prompt
    ///
    /// Dismissable prompts are those that feel "closeable" with escape:
    /// - ArgPrompt, DivPrompt, FormPrompt, SelectPrompt, PathPrompt, EnvPrompt, DropPrompt, TemplatePrompt
    /// - Built-in views (ClipboardHistory, AppLauncher, WindowSwitcher, DesignGallery)
    /// - ScriptList
    ///
    /// Non-dismissable prompts:
    /// - TermPrompt, EditorPrompt (these require explicit Cmd+W to close)
    #[allow(dead_code)]
    fn is_dismissable_view(&self) -> bool {
        !matches!(
            self.current_view,
            AppView::TermPrompt { .. } | AppView::EditorPrompt { .. }
        )
    }

    /// Show a HUD (heads-up display) overlay message
    ///
    /// This creates a separate floating window positioned at bottom-center of the
    /// screen containing the mouse cursor. The HUD is independent of the main
    /// Script Kit window and will remain visible even when the main window is hidden.
    ///
    /// Position: Bottom-center (85% down screen)
    /// Duration: 2000ms default, configurable
    /// Shape: Pill (40px tall, variable width)
    fn show_hud(&mut self, text: String, duration_ms: Option<u64>, cx: &mut Context<Self>) {
        // Delegate to the HUD manager which creates a separate floating window
        // This ensures the HUD is visible even when the main app window is hidden
        hud_manager::show_hud(text, duration_ms, cx);
    }

    /// Show the debug grid overlay with specified options
    ///
    /// This method converts protocol::GridOptions to debug_grid::GridConfig
    /// and enables the grid overlay rendering.
    fn show_grid(&mut self, options: protocol::GridOptions, cx: &mut Context<Self>) {
        use debug_grid::{GridColorScheme, GridConfig, GridDepth};
        use protocol::GridDepthOption;

        // Convert protocol depth to debug_grid depth
        let depth = match &options.depth {
            GridDepthOption::Preset(s) if s == "all" => GridDepth::All,
            GridDepthOption::Preset(_) => GridDepth::Prompts,
            GridDepthOption::Components(names) => GridDepth::Components(names.clone()),
        };

        self.grid_config = Some(GridConfig {
            grid_size: options.grid_size,
            show_bounds: options.show_bounds,
            show_box_model: options.show_box_model,
            show_alignment_guides: options.show_alignment_guides,
            show_dimensions: options.show_dimensions,
            depth,
            color_scheme: GridColorScheme::default(),
        });

        logging::log(
            "DEBUG_GRID",
            &format!(
                "Grid overlay enabled: size={}, bounds={}, box_model={}, guides={}, dimensions={}",
                options.grid_size,
                options.show_bounds,
                options.show_box_model,
                options.show_alignment_guides,
                options.show_dimensions
            ),
        );

        cx.notify();
    }

    /// Hide the debug grid overlay
    fn hide_grid(&mut self, cx: &mut Context<Self>) {
        self.grid_config = None;
        logging::log("DEBUG_GRID", "Grid overlay hidden");
        cx.notify();
    }


    /// Rebuild alias and shortcut registries from current scripts/scriptlets.
    /// Returns a list of conflict messages (if any) for HUD display.
    /// Conflict rule: first-registered wins - duplicates are blocked.
    fn rebuild_registries(&mut self) -> Vec<String> {
        let mut conflicts = Vec::new();
        self.alias_registry.clear();
        self.shortcut_registry.clear();

        // Register script aliases
        for script in &self.scripts {
            if let Some(ref alias) = script.alias {
                let alias_lower = alias.to_lowercase();
                if let Some(existing_path) = self.alias_registry.get(&alias_lower) {
                    conflicts.push(format!(
                        "Alias conflict: '{}' already used by {}",
                        alias,
                        std::path::Path::new(existing_path)
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| existing_path.clone())
                    ));
                    logging::log(
                        "ALIAS",
                        &format!(
                            "Conflict: alias '{}' in {} blocked (already used by {})",
                            alias,
                            script.path.display(),
                            existing_path
                        ),
                    );
                } else {
                    self.alias_registry
                        .insert(alias_lower, script.path.to_string_lossy().to_string());
                }
            }
        }

        // Register scriptlet aliases
        for scriptlet in &self.scriptlets {
            if let Some(ref alias) = scriptlet.alias {
                let alias_lower = alias.to_lowercase();
                if let Some(existing_path) = self.alias_registry.get(&alias_lower) {
                    conflicts.push(format!(
                        "Alias conflict: '{}' already used by {}",
                        alias,
                        std::path::Path::new(existing_path)
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| existing_path.clone())
                    ));
                    logging::log(
                        "ALIAS",
                        &format!(
                            "Conflict: alias '{}' in {} blocked (already used by {})",
                            alias, scriptlet.name, existing_path
                        ),
                    );
                } else {
                    let path = scriptlet
                        .file_path
                        .clone()
                        .unwrap_or_else(|| scriptlet.name.clone());
                    self.alias_registry.insert(alias_lower, path);
                }
            }

            // Register scriptlet shortcuts
            if let Some(ref shortcut) = scriptlet.shortcut {
                let shortcut_lower = shortcut.to_lowercase();
                if let Some(existing_path) = self.shortcut_registry.get(&shortcut_lower) {
                    conflicts.push(format!(
                        "Shortcut conflict: '{}' already used by {}",
                        shortcut,
                        std::path::Path::new(existing_path)
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| existing_path.clone())
                    ));
                    logging::log(
                        "SHORTCUT",
                        &format!(
                            "Conflict: shortcut '{}' in {} blocked (already used by {})",
                            shortcut, scriptlet.name, existing_path
                        ),
                    );
                } else {
                    let path = scriptlet
                        .file_path
                        .clone()
                        .unwrap_or_else(|| scriptlet.name.clone());
                    self.shortcut_registry.insert(shortcut_lower, path);
                }
            }
        }

        logging::log(
            "REGISTRY",
            &format!(
                "Rebuilt registries: {} aliases, {} shortcuts, {} conflicts",
                self.alias_registry.len(),
                self.shortcut_registry.len(),
                conflicts.len()
            ),
        );

        conflicts
    }

    /// Reset all state and return to the script list view.
    /// This clears all prompt state and resizes the window appropriately.
    fn reset_to_script_list(&mut self, cx: &mut Context<Self>) {
        let old_view = match &self.current_view {
            AppView::ScriptList => "ScriptList",
            AppView::ActionsDialog => "ActionsDialog",
            AppView::ArgPrompt { .. } => "ArgPrompt",
            AppView::DivPrompt { .. } => "DivPrompt",
            AppView::FormPrompt { .. } => "FormPrompt",
            AppView::TermPrompt { .. } => "TermPrompt",
            AppView::EditorPrompt { .. } => "EditorPrompt",
            AppView::SelectPrompt { .. } => "SelectPrompt",
            AppView::PathPrompt { .. } => "PathPrompt",
            AppView::EnvPrompt { .. } => "EnvPrompt",
            AppView::DropPrompt { .. } => "DropPrompt",
            AppView::TemplatePrompt { .. } => "TemplatePrompt",
            AppView::ClipboardHistoryView { .. } => "ClipboardHistoryView",
            AppView::AppLauncherView { .. } => "AppLauncherView",
            AppView::WindowSwitcherView { .. } => "WindowSwitcherView",
            AppView::DesignGalleryView { .. } => "DesignGalleryView",
        };

        let old_focused_input = self.focused_input;
        logging::log(
            "UI",
            &format!(
                "Resetting to script list (was: {}, focused_input: {:?})",
                old_view, old_focused_input
            ),
        );

        // Belt-and-suspenders: Force-kill the process group using stored PID
        // This runs BEFORE clearing channels to ensure cleanup even if Drop doesn't fire
        if let Some(pid) = self.current_script_pid.take() {
            logging::log(
                "CLEANUP",
                &format!("Force-killing script process group {} during reset", pid),
            );
            #[cfg(unix)]
            {
                let _ = std::process::Command::new("kill")
                    .args(["-9", &format!("-{}", pid)])
                    .output();
            }
        }

        // Reset view
        self.current_view = AppView::ScriptList;

        // CRITICAL: Reset focused_input to MainFilter so the cursor appears
        // This was a bug where focused_input could remain as ArgPrompt/None after
        // script exit, causing the cursor to not show in the main filter.
        self.focused_input = FocusedInput::MainFilter;
        self.gpui_input_focused = false;
        logging::log(
            "FOCUS",
            "Reset focused_input to MainFilter for cursor display",
        );

        // Clear arg prompt state
        self.arg_input.clear();
        self.arg_selected_index = 0;
        // P0: Reset arg scroll handle
        self.arg_list_scroll_handle
            .scroll_to_item(0, ScrollStrategy::Top);

        // Clear filter and selection state for fresh menu
        self.filter_text.clear();
        self.computed_filter_text.clear();
        self.filter_coalescer.reset();
        self.pending_filter_sync = true;
        self.selected_index = 0;
        self.last_scrolled_index = None;
        // Use main_list_state for variable-height list (not the legacy list_scroll_handle)
        self.main_list_state.scroll_to_reveal_item(0);
        self.last_scrolled_index = Some(0);

        // Resize window for script list content
        let count = self.scripts.len() + self.scriptlets.len();
        resize_first_window_to_height(height_for_view(ViewType::ScriptList, count));

        // Clear output
        self.last_output = None;

        // Clear channels (they will be dropped, closing the connections)
        self.prompt_receiver = None;
        self.response_sender = None;

        // Clear script session (parking_lot mutex never poisons)
        *self.script_session.lock() = None;

        // Clear actions popup state (prevents stale actions dialog from persisting)
        self.show_actions_popup = false;
        self.actions_dialog = None;

        // Clear pending path action and close signal
        if let Ok(mut guard) = self.pending_path_action.lock() {
            *guard = None;
        }
        if let Ok(mut guard) = self.close_path_actions.lock() {
            *guard = false;
        }

        logging::log(
            "UI",
            "State reset complete - view is now ScriptList (filter, selection, scroll cleared)",
        );
        cx.notify();
    }

    /// Check if we're currently in a prompt view (script is running)
    fn is_in_prompt(&self) -> bool {
        matches!(
            self.current_view,
            AppView::ArgPrompt { .. }
                | AppView::DivPrompt { .. }
                | AppView::FormPrompt { .. }
                | AppView::TermPrompt { .. }
                | AppView::EditorPrompt { .. }
                | AppView::ClipboardHistoryView { .. }
                | AppView::AppLauncherView { .. }
                | AppView::WindowSwitcherView { .. }
                | AppView::DesignGalleryView { .. }
        )
    }

    /// Submit a response to the current prompt
    fn submit_prompt_response(
        &mut self,
        id: String,
        value: Option<String>,
        _cx: &mut Context<Self>,
    ) {
        logging::log(
            "UI",
            &format!("Submitting response for {}: {:?}", id, value),
        );

        let response = Message::Submit { id, value };

        if let Some(ref sender) = self.response_sender {
            match sender.send(response) {
                Ok(()) => {
                    logging::log("UI", "Response queued for script");
                }
                Err(e) => {
                    logging::log("UI", &format!("Failed to queue response: {}", e));
                }
            }
        } else {
            logging::log("UI", "No response sender available");
        }

        // Return to waiting state (script will send next prompt or exit)
        // Don't change view here - wait for next message from script
    }

    /// Get filtered choices for arg prompt
    fn filtered_arg_choices(&self) -> Vec<(usize, &Choice)> {
        if let AppView::ArgPrompt { choices, .. } = &self.current_view {
            if self.arg_input.is_empty() {
                choices.iter().enumerate().collect()
            } else {
                let filter = self.arg_input.text().to_lowercase();
                choices
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.name.to_lowercase().contains(&filter))
                    .collect()
            }
        } else {
            vec![]
        }
    }

    /// P0: Get filtered choices as owned data for uniform_list closure
    fn get_filtered_arg_choices_owned(&self) -> Vec<(usize, Choice)> {
        if let AppView::ArgPrompt { choices, .. } = &self.current_view {
            if self.arg_input.is_empty() {
                choices
                    .iter()
                    .enumerate()
                    .map(|(i, c)| (i, c.clone()))
                    .collect()
            } else {
                let filter = self.arg_input.text().to_lowercase();
                choices
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.name.to_lowercase().contains(&filter))
                    .map(|(i, c)| (i, c.clone()))
                    .collect()
            }
        } else {
            vec![]
        }
    }

    /// Convert hex color to rgba with opacity from theme
    fn hex_to_rgba_with_opacity(&self, hex: u32, opacity: f32) -> u32 {
        // Convert opacity (0.0-1.0) to alpha byte (0-255)
        let alpha = (opacity.clamp(0.0, 1.0) * 255.0) as u32;
        (hex << 8) | alpha
    }

    /// Create box shadows from theme configuration
    fn create_box_shadows(&self) -> Vec<BoxShadow> {
        let shadow_config = self.theme.get_drop_shadow();

        if !shadow_config.enabled {
            return vec![];
        }

        // Convert hex color to HSLA
        // For black (0x000000), we use h=0, s=0, l=0
        let r = ((shadow_config.color >> 16) & 0xFF) as f32 / 255.0;
        let g = ((shadow_config.color >> 8) & 0xFF) as f32 / 255.0;
        let b = (shadow_config.color & 0xFF) as f32 / 255.0;

        // Simple RGB to HSL conversion for shadow color
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;

        let (h, s) = if max == min {
            (0.0, 0.0) // achromatic
        } else {
            let d = max - min;
            let s = if l > 0.5 {
                d / (2.0 - max - min)
            } else {
                d / (max + min)
            };
            let h = if max == r {
                (g - b) / d + if g < b { 6.0 } else { 0.0 }
            } else if max == g {
                (b - r) / d + 2.0
            } else {
                (r - g) / d + 4.0
            };
            (h / 6.0, s)
        };

        vec![BoxShadow {
            color: hsla(h, s, l, shadow_config.opacity),
            offset: point(px(shadow_config.offset_x), px(shadow_config.offset_y)),
            blur_radius: px(shadow_config.blur_radius),
            spread_radius: px(shadow_config.spread_radius),
        }]
    }
}

// Note: convert_menu_bar_items/convert_menu_bar_item functions were removed
// because frontmost_app_tracker is now compiled as part of the binary crate
// (via `mod frontmost_app_tracker` in main.rs) so it returns binary types directly.

</file>

<file path="src/config/mod.rs">
//! Configuration module - Application settings and user preferences
//!
//! This module provides functionality for:
//! - Loading configuration from ~/.scriptkit/kit/config.ts
//! - Default values for all settings
//! - Type definitions for config structures
//!
//! # Module Structure
//!
//! - `defaults` - All default constant values
//! - `types` - Configuration struct definitions (Config, HotkeyConfig, etc.)
//! - `loader` - File system loading and parsing

mod defaults;
mod loader;
mod types;

// Re-export defaults that are used externally
pub use defaults::DEFAULT_SUGGESTED_HALF_LIFE_DAYS;

// Re-export types that are used externally
pub use types::{BuiltInConfig, Config, HotkeyConfig, SuggestedConfig};

// Re-export loader
pub use loader::load_config;

// Additional exports for tests
#[cfg(test)]
pub use defaults::{
    DEFAULT_CLIPBOARD_HISTORY_MAX_TEXT_LENGTH, DEFAULT_CONFIRMATION_COMMANDS,
    DEFAULT_EDITOR_FONT_SIZE, DEFAULT_HEALTH_CHECK_INTERVAL_MS, DEFAULT_PADDING_LEFT,
    DEFAULT_PADDING_RIGHT, DEFAULT_PADDING_TOP, DEFAULT_TERMINAL_FONT_SIZE, DEFAULT_UI_SCALE,
};

#[cfg(test)]
pub use types::{CommandConfig, ContentPadding, ProcessLimits};

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;

</file>

<file path="src/hotkeys.rs">
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    Error as HotkeyError, GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use crate::{config, logging, scripts, shortcuts};

// =============================================================================
// Global state for hotkey hot-reload
// =============================================================================

/// The main GlobalHotKeyManager - stored globally so update_hotkeys can access it
static MAIN_MANAGER: OnceLock<Mutex<GlobalHotKeyManager>> = OnceLock::new();

/// Current hotkey IDs - read by the event loop via atomics
static MAIN_HOTKEY_ID: AtomicU32 = AtomicU32::new(0);
static NOTES_HOTKEY_ID: AtomicU32 = AtomicU32::new(0);
static AI_HOTKEY_ID: AtomicU32 = AtomicU32::new(0);

/// Stores the actual HotKey objects (needed for unregistration)
struct CurrentHotkeys {
    main: Option<HotKey>,
    notes: Option<HotKey>,
    ai: Option<HotKey>,
}

static CURRENT_HOTKEYS: OnceLock<Mutex<CurrentHotkeys>> = OnceLock::new();

fn get_current_hotkeys() -> &'static Mutex<CurrentHotkeys> {
    CURRENT_HOTKEYS.get_or_init(|| {
        Mutex::new(CurrentHotkeys {
            main: None,
            notes: None,
            ai: None,
        })
    })
}

/// Parse a HotkeyConfig into (Modifiers, Code)
fn parse_hotkey_config(hk: &config::HotkeyConfig) -> Option<(Modifiers, Code)> {
    let code = match hk.key.as_str() {
        "Semicolon" => Code::Semicolon,
        "KeyK" => Code::KeyK,
        "KeyP" => Code::KeyP,
        "Space" => Code::Space,
        "Enter" => Code::Enter,
        "Digit0" => Code::Digit0,
        "Digit1" => Code::Digit1,
        "Digit2" => Code::Digit2,
        "Digit3" => Code::Digit3,
        "Digit4" => Code::Digit4,
        "Digit5" => Code::Digit5,
        "Digit6" => Code::Digit6,
        "Digit7" => Code::Digit7,
        "Digit8" => Code::Digit8,
        "Digit9" => Code::Digit9,
        "KeyA" => Code::KeyA,
        "KeyB" => Code::KeyB,
        "KeyC" => Code::KeyC,
        "KeyD" => Code::KeyD,
        "KeyE" => Code::KeyE,
        "KeyF" => Code::KeyF,
        "KeyG" => Code::KeyG,
        "KeyH" => Code::KeyH,
        "KeyI" => Code::KeyI,
        "KeyJ" => Code::KeyJ,
        "KeyL" => Code::KeyL,
        "KeyM" => Code::KeyM,
        "KeyN" => Code::KeyN,
        "KeyO" => Code::KeyO,
        "KeyQ" => Code::KeyQ,
        "KeyR" => Code::KeyR,
        "KeyS" => Code::KeyS,
        "KeyT" => Code::KeyT,
        "KeyU" => Code::KeyU,
        "KeyV" => Code::KeyV,
        "KeyW" => Code::KeyW,
        "KeyX" => Code::KeyX,
        "KeyY" => Code::KeyY,
        "KeyZ" => Code::KeyZ,
        "F1" => Code::F1,
        "F2" => Code::F2,
        "F3" => Code::F3,
        "F4" => Code::F4,
        "F5" => Code::F5,
        "F6" => Code::F6,
        "F7" => Code::F7,
        "F8" => Code::F8,
        "F9" => Code::F9,
        "F10" => Code::F10,
        "F11" => Code::F11,
        "F12" => Code::F12,
        _ => return None,
    };

    let mut modifiers = Modifiers::empty();
    for modifier in &hk.modifiers {
        match modifier.as_str() {
            "meta" => modifiers |= Modifiers::META,
            "ctrl" => modifiers |= Modifiers::CONTROL,
            "alt" => modifiers |= Modifiers::ALT,
            "shift" => modifiers |= Modifiers::SHIFT,
            _ => {}
        }
    }

    Some((modifiers, code))
}

/// Convert a HotkeyConfig to a display string (e.g., "meta+shift+N")
fn hotkey_config_to_display(hk: &config::HotkeyConfig) -> String {
    format!(
        "{}{}{}",
        hk.modifiers.join("+"),
        if hk.modifiers.is_empty() { "" } else { "+" },
        hk.key
    )
}

/// Update hotkeys from config - call this when config changes
pub fn update_hotkeys(cfg: &config::Config) {
    let manager_guard = match MAIN_MANAGER.get() {
        Some(m) => match m.lock() {
            Ok(g) => g,
            Err(e) => {
                logging::log("HOTKEY", &format!("Failed to lock manager: {}", e));
                return;
            }
        },
        None => {
            logging::log("HOTKEY", "Manager not initialized - hotkeys not updated");
            return;
        }
    };

    let mut hotkeys_guard = match get_current_hotkeys().lock() {
        Ok(g) => g,
        Err(e) => {
            logging::log("HOTKEY", &format!("Failed to lock current hotkeys: {}", e));
            return;
        }
    };

    // Update main hotkey
    let main_config = &cfg.hotkey;
    if let Some((mods, code)) = parse_hotkey_config(main_config) {
        let new_hotkey = HotKey::new(Some(mods), code);
        let new_id = new_hotkey.id();
        let current_id = MAIN_HOTKEY_ID.load(Ordering::SeqCst);

        if new_id != current_id {
            let display = hotkey_config_to_display(main_config);

            // Unregister old
            if let Some(old_hotkey) = hotkeys_guard.main.take() {
                if let Err(e) = manager_guard.unregister(old_hotkey) {
                    logging::log(
                        "HOTKEY",
                        &format!("Failed to unregister old main hotkey: {}", e),
                    );
                }
            }

            // Register new
            match manager_guard.register(new_hotkey) {
                Ok(()) => {
                    MAIN_HOTKEY_ID.store(new_id, Ordering::SeqCst);
                    hotkeys_guard.main = Some(HotKey::new(Some(mods), code));
                    MAIN_HOTKEY_REGISTERED.store(true, Ordering::SeqCst);
                    logging::log(
                        "HOTKEY",
                        &format!("Hot-reloaded main hotkey: {} (id: {})", display, new_id),
                    );
                }
                Err(e) => {
                    MAIN_HOTKEY_REGISTERED.store(false, Ordering::SeqCst);
                    logging::log(
                        "HOTKEY",
                        &format!("Failed to register main hotkey {}: {}", display, e),
                    );
                }
            }
        }
    }

    // Update notes hotkey
    let notes_config = cfg.get_notes_hotkey();
    if let Some((mods, code)) = parse_hotkey_config(&notes_config) {
        let new_hotkey = HotKey::new(Some(mods), code);
        let new_id = new_hotkey.id();
        let current_id = NOTES_HOTKEY_ID.load(Ordering::SeqCst);

        if new_id != current_id {
            let display = hotkey_config_to_display(&notes_config);

            // Unregister old
            if let Some(old_hotkey) = hotkeys_guard.notes.take() {
                if let Err(e) = manager_guard.unregister(old_hotkey) {
                    logging::log(
                        "HOTKEY",
                        &format!("Failed to unregister old notes hotkey: {}", e),
                    );
                }
            }

            // Register new
            match manager_guard.register(new_hotkey) {
                Ok(()) => {
                    NOTES_HOTKEY_ID.store(new_id, Ordering::SeqCst);
                    hotkeys_guard.notes = Some(HotKey::new(Some(mods), code));
                    logging::log(
                        "HOTKEY",
                        &format!("Hot-reloaded notes hotkey: {} (id: {})", display, new_id),
                    );
                }
                Err(e) => {
                    logging::log(
                        "HOTKEY",
                        &format!("Failed to register notes hotkey {}: {}", display, e),
                    );
                }
            }
        }
    }

    // Update AI hotkey
    let ai_config = cfg.get_ai_hotkey();
    if let Some((mods, code)) = parse_hotkey_config(&ai_config) {
        let new_hotkey = HotKey::new(Some(mods), code);
        let new_id = new_hotkey.id();
        let current_id = AI_HOTKEY_ID.load(Ordering::SeqCst);

        if new_id != current_id {
            let display = hotkey_config_to_display(&ai_config);

            // Unregister old
            if let Some(old_hotkey) = hotkeys_guard.ai.take() {
                if let Err(e) = manager_guard.unregister(old_hotkey) {
                    logging::log(
                        "HOTKEY",
                        &format!("Failed to unregister old AI hotkey: {}", e),
                    );
                }
            }

            // Register new
            match manager_guard.register(new_hotkey) {
                Ok(()) => {
                    AI_HOTKEY_ID.store(new_id, Ordering::SeqCst);
                    hotkeys_guard.ai = Some(HotKey::new(Some(mods), code));
                    logging::log(
                        "HOTKEY",
                        &format!("Hot-reloaded AI hotkey: {} (id: {})", display, new_id),
                    );
                }
                Err(e) => {
                    logging::log(
                        "HOTKEY",
                        &format!("Failed to register AI hotkey {}: {}", display, e),
                    );
                }
            }
        }
    }
}

// =============================================================================
// Dynamic Script Hotkey Manager
// =============================================================================

/// Manages dynamic registration/unregistration of script hotkeys.
/// Uses a thread-safe global singleton pattern for access from multiple contexts.
pub struct ScriptHotkeyManager {
    /// The underlying global hotkey manager
    manager: GlobalHotKeyManager,
    /// Maps hotkey ID -> script path
    hotkey_map: HashMap<u32, String>,
    /// Maps script path -> hotkey ID (reverse lookup for unregistration)
    path_to_id: HashMap<String, u32>,
    /// Maps script path -> HotKey object (needed for proper unregistration)
    path_to_hotkey: HashMap<String, HotKey>,
}

impl ScriptHotkeyManager {
    /// Create a new ScriptHotkeyManager.
    /// NOTE: Must be created on the main thread.
    fn new(manager: GlobalHotKeyManager) -> Self {
        Self {
            manager,
            hotkey_map: HashMap::new(),
            path_to_id: HashMap::new(),
            path_to_hotkey: HashMap::new(),
        }
    }

    /// Register a hotkey for a script.
    /// Returns the hotkey ID on success.
    pub fn register(&mut self, path: &str, shortcut: &str) -> anyhow::Result<u32> {
        // Parse the shortcut
        let (mods, code) = shortcuts::parse_shortcut(shortcut)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse shortcut: {}", shortcut))?;

        let hotkey = HotKey::new(Some(mods), code);
        let hotkey_id = hotkey.id();

        // Register with the OS - provide specific error messages based on error type
        if let Err(e) = self.manager.register(hotkey) {
            return Err(match e {
                HotkeyError::AlreadyRegistered(hk) => {
                    anyhow::anyhow!(
                        "Hotkey '{}' is already registered (conflict with another app or script). Hotkey ID: {}",
                        shortcut,
                        hk.id()
                    )
                }
                HotkeyError::FailedToRegister(msg) => {
                    anyhow::anyhow!(
                        "System rejected hotkey '{}': {}. This may be reserved by macOS or another app.",
                        shortcut,
                        msg
                    )
                }
                HotkeyError::OsError(os_err) => {
                    anyhow::anyhow!("OS error registering hotkey '{}': {}", shortcut, os_err)
                }
                other => {
                    anyhow::anyhow!("Failed to register hotkey '{}': {}", shortcut, other)
                }
            });
        }

        // Track the mapping
        self.hotkey_map.insert(hotkey_id, path.to_string());
        self.path_to_id.insert(path.to_string(), hotkey_id);
        self.path_to_hotkey.insert(path.to_string(), hotkey);

        logging::log(
            "HOTKEY",
            &format!(
                "Registered script hotkey '{}' for {} (id: {})",
                shortcut, path, hotkey_id
            ),
        );

        Ok(hotkey_id)
    }

    /// Unregister a hotkey for a script by path.
    /// Returns Ok(()) even if the path wasn't registered (no-op).
    pub fn unregister(&mut self, path: &str) -> anyhow::Result<()> {
        if let Some(hotkey_id) = self.path_to_id.remove(path) {
            // Remove from hotkey_map
            self.hotkey_map.remove(&hotkey_id);

            // Unregister from OS using stored HotKey object
            if let Some(hotkey) = self.path_to_hotkey.remove(path) {
                if let Err(e) = self.manager.unregister(hotkey) {
                    logging::log(
                        "HOTKEY",
                        &format!(
                            "Warning: Failed to unregister hotkey for {} (id: {}): {}",
                            path, hotkey_id, e
                        ),
                    );
                    // Continue anyway - the internal tracking is already updated
                }
            }

            logging::log(
                "HOTKEY",
                &format!(
                    "Unregistered script hotkey for {} (id: {})",
                    path, hotkey_id
                ),
            );
        }
        // If path wasn't registered, this is a no-op (success)
        Ok(())
    }

    /// Update a script's hotkey.
    /// Handles add (old=None, new=Some), remove (old=Some, new=None), and change (both Some).
    pub fn update(
        &mut self,
        path: &str,
        old_shortcut: Option<&str>,
        new_shortcut: Option<&str>,
    ) -> anyhow::Result<()> {
        match (old_shortcut, new_shortcut) {
            (None, None) => {
                // No change needed
                Ok(())
            }
            (None, Some(new)) => {
                // Add new hotkey
                self.register(path, new)?;
                Ok(())
            }
            (Some(_old), None) => {
                // Remove old hotkey
                self.unregister(path)
            }
            (Some(_old), Some(new)) => {
                // Change: unregister old, register new
                self.unregister(path)?;
                self.register(path, new)?;
                Ok(())
            }
        }
    }

    /// Get the script path for a given hotkey ID.
    pub fn get_script_path(&self, hotkey_id: u32) -> Option<&String> {
        self.hotkey_map.get(&hotkey_id)
    }

    /// Get all registered hotkeys as (path, hotkey_id) pairs.
    pub fn get_registered_hotkeys(&self) -> Vec<(String, u32)> {
        self.path_to_id
            .iter()
            .map(|(path, id)| (path.clone(), *id))
            .collect()
    }

    /// Check if a script has a registered hotkey.
    #[allow(dead_code)]
    pub fn is_registered(&self, path: &str) -> bool {
        self.path_to_id.contains_key(path)
    }
}

/// Global singleton for the ScriptHotkeyManager.
/// Initialized when start_hotkey_listener is called.
static SCRIPT_HOTKEY_MANAGER: OnceLock<Mutex<ScriptHotkeyManager>> = OnceLock::new();

/// Initialize the global ScriptHotkeyManager.
/// Must be called from the main thread.
/// Returns an error if already initialized.
#[allow(dead_code)]
pub fn init_script_hotkey_manager(manager: GlobalHotKeyManager) -> anyhow::Result<()> {
    SCRIPT_HOTKEY_MANAGER
        .set(Mutex::new(ScriptHotkeyManager::new(manager)))
        .map_err(|_| anyhow::anyhow!("ScriptHotkeyManager already initialized"))
}

/// Register a script hotkey dynamically.
/// Returns the hotkey ID on success.
pub fn register_script_hotkey(path: &str, shortcut: &str) -> anyhow::Result<u32> {
    let manager = SCRIPT_HOTKEY_MANAGER
        .get()
        .ok_or_else(|| anyhow::anyhow!("ScriptHotkeyManager not initialized"))?;

    let mut guard = manager
        .lock()
        .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
    guard.register(path, shortcut)
}

/// Unregister a script hotkey by path.
/// Returns Ok(()) even if the path wasn't registered (no-op).
pub fn unregister_script_hotkey(path: &str) -> anyhow::Result<()> {
    let manager = SCRIPT_HOTKEY_MANAGER
        .get()
        .ok_or_else(|| anyhow::anyhow!("ScriptHotkeyManager not initialized"))?;

    let mut guard = manager
        .lock()
        .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
    guard.unregister(path)
}

/// Update a script's hotkey.
/// Handles add (old=None, new=Some), remove (old=Some, new=None), and change (both Some).
pub fn update_script_hotkey(
    path: &str,
    old_shortcut: Option<&str>,
    new_shortcut: Option<&str>,
) -> anyhow::Result<()> {
    let manager = SCRIPT_HOTKEY_MANAGER
        .get()
        .ok_or_else(|| anyhow::anyhow!("ScriptHotkeyManager not initialized"))?;

    let mut guard = manager
        .lock()
        .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
    guard.update(path, old_shortcut, new_shortcut)
}

/// Get the script path for a given hotkey ID.
#[allow(dead_code)]
pub fn get_script_for_hotkey(hotkey_id: u32) -> Option<String> {
    let manager = SCRIPT_HOTKEY_MANAGER.get()?;
    let guard = manager.lock().ok()?;
    guard.get_script_path(hotkey_id).cloned()
}

/// Get all registered script hotkeys.
#[allow(dead_code)]
pub fn get_registered_hotkeys() -> Vec<(String, u32)> {
    SCRIPT_HOTKEY_MANAGER
        .get()
        .and_then(|m| m.lock().ok())
        .map(|guard| guard.get_registered_hotkeys())
        .unwrap_or_default()
}

// =============================================================================
// GCD dispatch for immediate main-thread execution (bypasses async runtime)
// =============================================================================

use std::sync::Arc;

/// Callback type for hotkey actions - uses Arc<dyn Fn()> for repeated invocation
pub type HotkeyHandler = Arc<dyn Fn() + Send + Sync>;

/// Static storage for handlers to be invoked on main thread
static NOTES_HANDLER: OnceLock<std::sync::Mutex<Option<HotkeyHandler>>> = OnceLock::new();
static AI_HANDLER: OnceLock<std::sync::Mutex<Option<HotkeyHandler>>> = OnceLock::new();

/// Register a handler to be invoked when the Notes hotkey is pressed.
/// This handler will be executed on the main thread via GCD dispatch_async.
/// The handler can be called multiple times (it's not consumed).
#[allow(dead_code)]
pub fn set_notes_hotkey_handler<F: Fn() + Send + Sync + 'static>(handler: F) {
    let storage = NOTES_HANDLER.get_or_init(|| std::sync::Mutex::new(None));
    *storage.lock().unwrap() = Some(Arc::new(handler));
}

/// Register a handler to be invoked when the AI hotkey is pressed.
/// This handler will be executed on the main thread via GCD dispatch_async.
/// The handler can be called multiple times (it's not consumed).
#[allow(dead_code)]
pub fn set_ai_hotkey_handler<F: Fn() + Send + Sync + 'static>(handler: F) {
    let storage = AI_HANDLER.get_or_init(|| std::sync::Mutex::new(None));
    *storage.lock().unwrap() = Some(Arc::new(handler));
}

#[cfg(target_os = "macos")]
mod gcd {
    use std::ffi::c_void;

    // Link to libSystem for GCD functions
    // Note: dispatch_get_main_queue is actually a macro that returns &_dispatch_main_q
    // We use the raw symbol directly instead
    #[link(name = "System", kind = "framework")]
    extern "C" {
        fn dispatch_async_f(
            queue: *const c_void,
            context: *mut c_void,
            work: extern "C" fn(*mut c_void),
        );
        // The main dispatch queue is a global static symbol, not a function
        #[link_name = "_dispatch_main_q"]
        static DISPATCH_MAIN_QUEUE: c_void;
    }

    /// Dispatch a closure to the main thread via GCD.
    /// This is the key to making hotkeys work before the GPUI event loop is "warmed up".
    pub fn dispatch_to_main<F: FnOnce() + Send + 'static>(f: F) {
        let boxed: Box<dyn FnOnce() + Send> = Box::new(f);
        let raw = Box::into_raw(Box::new(boxed));

        extern "C" fn trampoline(context: *mut c_void) {
            unsafe {
                let boxed: Box<Box<dyn FnOnce() + Send>> = Box::from_raw(context as *mut _);
                boxed();
            }
        }

        unsafe {
            let main_queue = &DISPATCH_MAIN_QUEUE as *const c_void;
            dispatch_async_f(main_queue, raw as *mut c_void, trampoline);
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod gcd {
    /// Fallback for non-macOS: just call the closure directly (in the current thread)
    pub fn dispatch_to_main<F: FnOnce() + Send + 'static>(f: F) {
        f();
    }
}

/// Dispatch the Notes hotkey handler to the main thread.
///
/// Strategy (mutually exclusive to prevent double-fire):
/// - If a handler is registered: use it directly via GCD dispatch
/// - Otherwise: send to channel for async polling
///
/// This works even before the main window is activated because GCD dispatch
/// directly integrates with the NSApplication run loop that GPUI uses.
fn dispatch_notes_hotkey() {
    // Check if a direct handler is registered (takes priority over channel)
    let handler = NOTES_HANDLER
        .get_or_init(|| std::sync::Mutex::new(None))
        .lock()
        .unwrap()
        .clone();

    if let Some(handler) = handler {
        // Handler is set - use direct GCD dispatch (skip channel to avoid double-fire)
        gcd::dispatch_to_main(move || {
            handler();
        });
    } else {
        // No handler - use channel approach for async polling
        if notes_hotkey_channel().0.try_send(()).is_err() {
            logging::log("HOTKEY", "Notes hotkey channel full/closed");
        }
        // Dispatch an empty closure to wake GPUI's event loop
        // This ensures the channel message gets processed even if GPUI was idle
        gcd::dispatch_to_main(|| {
            // Empty closure - just wakes the run loop
        });
    }
}

/// Dispatch the AI hotkey handler to the main thread.
///
/// Strategy (mutually exclusive to prevent double-fire):
/// - If a handler is registered: use it directly via GCD dispatch
/// - Otherwise: send to channel for async polling
fn dispatch_ai_hotkey() {
    // Check if a direct handler is registered (takes priority over channel)
    let handler = AI_HANDLER
        .get_or_init(|| std::sync::Mutex::new(None))
        .lock()
        .unwrap()
        .clone();

    if let Some(handler) = handler {
        // Handler is set - use direct GCD dispatch (skip channel to avoid double-fire)
        gcd::dispatch_to_main(move || {
            handler();
        });
    } else {
        // No handler - use channel approach for async polling
        if ai_hotkey_channel().0.try_send(()).is_err() {
            logging::log("HOTKEY", "AI hotkey channel full/closed");
        }
        // Dispatch an empty closure to wake GPUI's event loop
        gcd::dispatch_to_main(|| {
            // Empty closure - just wakes the run loop
        });
    }
}

// HOTKEY_CHANNEL: Event-driven async_channel for hotkey events (replaces AtomicBool polling)
#[allow(dead_code)]
static HOTKEY_CHANNEL: OnceLock<(async_channel::Sender<()>, async_channel::Receiver<()>)> =
    OnceLock::new();

/// Get the hotkey channel, initializing it on first access.
#[allow(dead_code)]
pub(crate) fn hotkey_channel() -> &'static (async_channel::Sender<()>, async_channel::Receiver<()>)
{
    HOTKEY_CHANNEL.get_or_init(|| async_channel::bounded(10))
}

// SCRIPT_HOTKEY_CHANNEL: Channel for script shortcut events (sends script path)
#[allow(dead_code)]
static SCRIPT_HOTKEY_CHANNEL: OnceLock<(
    async_channel::Sender<String>,
    async_channel::Receiver<String>,
)> = OnceLock::new();

/// Get the script hotkey channel, initializing it on first access.
#[allow(dead_code)]
pub(crate) fn script_hotkey_channel() -> &'static (
    async_channel::Sender<String>,
    async_channel::Receiver<String>,
) {
    SCRIPT_HOTKEY_CHANNEL.get_or_init(|| async_channel::bounded(10))
}

// NOTES_HOTKEY_CHANNEL: Channel for notes hotkey events
#[allow(dead_code)]
static NOTES_HOTKEY_CHANNEL: OnceLock<(async_channel::Sender<()>, async_channel::Receiver<()>)> =
    OnceLock::new();

/// Get the notes hotkey channel, initializing it on first access.
#[allow(dead_code)]
pub(crate) fn notes_hotkey_channel(
) -> &'static (async_channel::Sender<()>, async_channel::Receiver<()>) {
    NOTES_HOTKEY_CHANNEL.get_or_init(|| async_channel::bounded(10))
}

// AI_HOTKEY_CHANNEL: Channel for AI hotkey events
#[allow(dead_code)]
static AI_HOTKEY_CHANNEL: OnceLock<(async_channel::Sender<()>, async_channel::Receiver<()>)> =
    OnceLock::new();

/// Get the AI hotkey channel, initializing it on first access.
#[allow(dead_code)]
pub(crate) fn ai_hotkey_channel(
) -> &'static (async_channel::Sender<()>, async_channel::Receiver<()>) {
    AI_HOTKEY_CHANNEL.get_or_init(|| async_channel::bounded(10))
}

/// Tracks whether the main hotkey was successfully registered
/// Used by main.rs to detect if the app has an alternate entry point
static MAIN_HOTKEY_REGISTERED: AtomicBool = AtomicBool::new(false);

/// Check if the main hotkey was successfully registered
pub fn is_main_hotkey_registered() -> bool {
    MAIN_HOTKEY_REGISTERED.load(Ordering::SeqCst)
}

#[allow(dead_code)]
static HOTKEY_TRIGGER_COUNT: AtomicU64 = AtomicU64::new(0);

/// Format a hotkey registration error with helpful context
fn format_hotkey_error(e: &HotkeyError, shortcut_display: &str) -> String {
    match e {
        HotkeyError::AlreadyRegistered(hk) => {
            format!(
                "Hotkey '{}' is already registered by another application or script (ID: {}). \
                 Try a different shortcut or close the conflicting app.",
                shortcut_display,
                hk.id()
            )
        }
        HotkeyError::FailedToRegister(msg) => {
            format!(
                "System rejected hotkey '{}': {}. This shortcut may be reserved by macOS.",
                shortcut_display, msg
            )
        }
        HotkeyError::OsError(os_err) => {
            format!(
                "OS error registering '{}': {}. Check system hotkey settings.",
                shortcut_display, os_err
            )
        }
        other => format!(
            "Failed to register hotkey '{}': {}",
            shortcut_display, other
        ),
    }
}

#[allow(dead_code)]
pub(crate) fn start_hotkey_listener(config: config::Config) {
    std::thread::spawn(move || {
        let manager = match GlobalHotKeyManager::new() {
            Ok(m) => m,
            Err(e) => {
                logging::log("HOTKEY", &format!("Failed to create hotkey manager: {}", e));
                return;
            }
        };

        // Store manager globally for hot-reload access
        if MAIN_MANAGER.set(Mutex::new(manager)).is_err() {
            logging::log("HOTKEY", "Manager already initialized (unexpected)");
            return;
        }

        // Get a reference to the manager for initial registration
        let manager_guard = match MAIN_MANAGER.get().unwrap().lock() {
            Ok(g) => g,
            Err(e) => {
                logging::log("HOTKEY", &format!("Failed to lock manager: {}", e));
                return;
            }
        };

        // Convert config hotkey to global_hotkey::Code
        let code = match config.hotkey.key.as_str() {
            "Semicolon" => Code::Semicolon,
            "KeyK" => Code::KeyK,
            "KeyP" => Code::KeyP,
            "Space" => Code::Space,
            "Enter" => Code::Enter,
            "Digit0" => Code::Digit0,
            "Digit1" => Code::Digit1,
            "Digit2" => Code::Digit2,
            "Digit3" => Code::Digit3,
            "Digit4" => Code::Digit4,
            "Digit5" => Code::Digit5,
            "Digit6" => Code::Digit6,
            "Digit7" => Code::Digit7,
            "Digit8" => Code::Digit8,
            "Digit9" => Code::Digit9,
            "KeyA" => Code::KeyA,
            "KeyB" => Code::KeyB,
            "KeyC" => Code::KeyC,
            "KeyD" => Code::KeyD,
            "KeyE" => Code::KeyE,
            "KeyF" => Code::KeyF,
            "KeyG" => Code::KeyG,
            "KeyH" => Code::KeyH,
            "KeyI" => Code::KeyI,
            "KeyJ" => Code::KeyJ,
            "KeyL" => Code::KeyL,
            "KeyM" => Code::KeyM,
            "KeyN" => Code::KeyN,
            "KeyO" => Code::KeyO,
            "KeyQ" => Code::KeyQ,
            "KeyR" => Code::KeyR,
            "KeyS" => Code::KeyS,
            "KeyT" => Code::KeyT,
            "KeyU" => Code::KeyU,
            "KeyV" => Code::KeyV,
            "KeyW" => Code::KeyW,
            "KeyX" => Code::KeyX,
            "KeyY" => Code::KeyY,
            "KeyZ" => Code::KeyZ,
            // Function keys
            "F1" => Code::F1,
            "F2" => Code::F2,
            "F3" => Code::F3,
            "F4" => Code::F4,
            "F5" => Code::F5,
            "F6" => Code::F6,
            "F7" => Code::F7,
            "F8" => Code::F8,
            "F9" => Code::F9,
            "F10" => Code::F10,
            "F11" => Code::F11,
            "F12" => Code::F12,
            other => {
                logging::log(
                    "HOTKEY",
                    &format!(
                        "Unknown key code: '{}'. Valid keys: KeyA-KeyZ, Digit0-Digit9, F1-F12, Space, Enter, Semicolon. Falling back to Semicolon",
                        other
                    ),
                );
                Code::Semicolon
            }
        };

        // Convert modifiers from config strings to Modifiers flags
        let mut modifiers = Modifiers::empty();
        for modifier in &config.hotkey.modifiers {
            match modifier.as_str() {
                "meta" => modifiers |= Modifiers::META,
                "ctrl" => modifiers |= Modifiers::CONTROL,
                "alt" => modifiers |= Modifiers::ALT,
                "shift" => modifiers |= Modifiers::SHIFT,
                other => {
                    logging::log("HOTKEY", &format!("Unknown modifier: {}", other));
                }
            }
        }

        let hotkey = HotKey::new(Some(modifiers), code);
        let main_hotkey_id = hotkey.id();

        let hotkey_display = format!(
            "{}{}",
            config.hotkey.modifiers.join("+"),
            if config.hotkey.modifiers.is_empty() {
                String::new()
            } else {
                "+".to_string()
            }
        ) + &config.hotkey.key;

        if let Err(e) = manager_guard.register(hotkey) {
            logging::log("HOTKEY", &format_hotkey_error(&e, &hotkey_display));
            // Main hotkey registration failed - flag stays false
            return;
        }

        // Store ID in atomic for event loop and HotKey for unregistration
        MAIN_HOTKEY_ID.store(main_hotkey_id, Ordering::SeqCst);
        {
            let mut hotkeys = get_current_hotkeys().lock().unwrap();
            hotkeys.main = Some(HotKey::new(Some(modifiers), code));
        }

        // Mark main hotkey as successfully registered
        MAIN_HOTKEY_REGISTERED.store(true, Ordering::SeqCst);

        logging::log(
            "HOTKEY",
            &format!(
                "Registered global hotkey {} (id: {})",
                hotkey_display, main_hotkey_id
            ),
        );

        // Register notes hotkey (Cmd+Shift+N by default)
        let notes_config = config.get_notes_hotkey();
        let notes_code = match notes_config.key.as_str() {
            "KeyN" => Code::KeyN,
            "KeyM" => Code::KeyM,
            "KeyO" => Code::KeyO,
            "KeyP" => Code::KeyP,
            _ => Code::KeyN, // Default to N
        };

        let mut notes_modifiers = Modifiers::empty();
        for modifier in &notes_config.modifiers {
            match modifier.as_str() {
                "meta" => notes_modifiers |= Modifiers::META,
                "ctrl" => notes_modifiers |= Modifiers::CONTROL,
                "alt" => notes_modifiers |= Modifiers::ALT,
                "shift" => notes_modifiers |= Modifiers::SHIFT,
                _ => {}
            }
        }

        let notes_hotkey = HotKey::new(Some(notes_modifiers), notes_code);
        let notes_hotkey_id = notes_hotkey.id();

        let notes_display = format!(
            "{}{}",
            notes_config.modifiers.join("+"),
            if notes_config.modifiers.is_empty() {
                String::new()
            } else {
                "+".to_string()
            }
        ) + &notes_config.key;

        if let Err(e) = manager_guard.register(notes_hotkey) {
            logging::log("HOTKEY", &format_hotkey_error(&e, &notes_display));
        } else {
            // Store ID in atomic and HotKey for unregistration
            NOTES_HOTKEY_ID.store(notes_hotkey_id, Ordering::SeqCst);
            {
                let mut hotkeys = get_current_hotkeys().lock().unwrap();
                hotkeys.notes = Some(HotKey::new(Some(notes_modifiers), notes_code));
            }
            logging::log(
                "HOTKEY",
                &format!(
                    "Registered notes hotkey {} (id: {})",
                    notes_display, notes_hotkey_id
                ),
            );
        }

        // Register AI hotkey (Cmd+Shift+Space by default)
        let ai_config = config.get_ai_hotkey();
        let ai_code = match ai_config.key.as_str() {
            "Space" => Code::Space,
            "KeyA" => Code::KeyA,
            "KeyI" => Code::KeyI,
            _ => Code::Space, // Default to Space
        };

        let mut ai_modifiers = Modifiers::empty();
        for modifier in &ai_config.modifiers {
            match modifier.as_str() {
                "meta" => ai_modifiers |= Modifiers::META,
                "ctrl" => ai_modifiers |= Modifiers::CONTROL,
                "alt" => ai_modifiers |= Modifiers::ALT,
                "shift" => ai_modifiers |= Modifiers::SHIFT,
                _ => {}
            }
        }

        let ai_hotkey = HotKey::new(Some(ai_modifiers), ai_code);
        let ai_hotkey_id = ai_hotkey.id();

        let ai_display = format!(
            "{}{}",
            ai_config.modifiers.join("+"),
            if ai_config.modifiers.is_empty() {
                String::new()
            } else {
                "+".to_string()
            }
        ) + &ai_config.key;

        if let Err(e) = manager_guard.register(ai_hotkey) {
            logging::log("HOTKEY", &format_hotkey_error(&e, &ai_display));
        } else {
            // Store ID in atomic and HotKey for unregistration
            AI_HOTKEY_ID.store(ai_hotkey_id, Ordering::SeqCst);
            {
                let mut hotkeys = get_current_hotkeys().lock().unwrap();
                hotkeys.ai = Some(HotKey::new(Some(ai_modifiers), ai_code));
            }
            logging::log(
                "HOTKEY",
                &format!("Registered AI hotkey {} (id: {})", ai_display, ai_hotkey_id),
            );
        }

        // Register script shortcuts
        // Map from hotkey ID to script path
        let mut script_hotkey_map: std::collections::HashMap<u32, String> =
            std::collections::HashMap::new();

        // Load scripts with shortcuts
        let all_scripts = scripts::read_scripts();
        for script in &all_scripts {
            if let Some(ref shortcut) = script.shortcut {
                if let Some((mods, key_code)) = shortcuts::parse_shortcut(shortcut) {
                    let script_hotkey = HotKey::new(Some(mods), key_code);
                    let script_hotkey_id = script_hotkey.id();

                    match manager_guard.register(script_hotkey) {
                        Ok(()) => {
                            script_hotkey_map.insert(
                                script_hotkey_id,
                                script.path.to_string_lossy().to_string(),
                            );
                            logging::log(
                                "HOTKEY",
                                &format!(
                                    "Registered script shortcut '{}' for {} (id: {})",
                                    shortcut, script.name, script_hotkey_id
                                ),
                            );
                        }
                        Err(e) => {
                            logging::log(
                                "HOTKEY",
                                &format!(
                                    "{} (script: {})",
                                    format_hotkey_error(&e, shortcut),
                                    script.name
                                ),
                            );
                        }
                    }
                } else {
                    logging::log(
                        "HOTKEY",
                        &format!(
                            "Failed to parse shortcut '{}' for script {}",
                            shortcut, script.name
                        ),
                    );
                }
            }
        }

        // Load scriptlets with shortcuts
        let all_scriptlets = scripts::load_scriptlets();
        for scriptlet in &all_scriptlets {
            if let Some(ref shortcut) = scriptlet.shortcut {
                if let Some((mods, key_code)) = shortcuts::parse_shortcut(shortcut) {
                    let scriptlet_hotkey = HotKey::new(Some(mods), key_code);
                    let scriptlet_hotkey_id = scriptlet_hotkey.id();

                    // Use file_path as the identifier (already includes #command)
                    let scriptlet_path = scriptlet
                        .file_path
                        .clone()
                        .unwrap_or_else(|| scriptlet.name.clone());

                    match manager_guard.register(scriptlet_hotkey) {
                        Ok(()) => {
                            script_hotkey_map.insert(scriptlet_hotkey_id, scriptlet_path.clone());
                            logging::log(
                                "HOTKEY",
                                &format!(
                                    "Registered scriptlet shortcut '{}' for {} (id: {})",
                                    shortcut, scriptlet.name, scriptlet_hotkey_id
                                ),
                            );
                        }
                        Err(e) => {
                            logging::log(
                                "HOTKEY",
                                &format!(
                                    "{} (scriptlet: {})",
                                    format_hotkey_error(&e, shortcut),
                                    scriptlet.name
                                ),
                            );
                        }
                    }
                }
            }
        }

        logging::log(
            "HOTKEY",
            &format!(
                "Registered {} script/scriptlet shortcuts",
                script_hotkey_map.len()
            ),
        );

        // Drop the manager guard before entering event loop
        // This allows update_hotkeys to acquire the lock for hot-reload
        drop(manager_guard);

        let receiver = GlobalHotKeyEvent::receiver();

        // Log all registered hotkey IDs for debugging
        logging::log(
            "HOTKEY",
            &format!(
                "Hotkey ID map: main={}, notes={}, ai={}",
                main_hotkey_id, notes_hotkey_id, ai_hotkey_id
            ),
        );

        loop {
            if let Ok(event) = receiver.recv() {
                // Only respond to key PRESS, not release
                if event.state != HotKeyState::Pressed {
                    continue;
                }

                // Read current IDs from atomics (support hot-reload)
                let current_main_id = MAIN_HOTKEY_ID.load(Ordering::SeqCst);
                let current_notes_id = NOTES_HOTKEY_ID.load(Ordering::SeqCst);
                let current_ai_id = AI_HOTKEY_ID.load(Ordering::SeqCst);

                // Log EVERY hotkey event with its ID for debugging
                logging::log(
                    "HOTKEY",
                    &format!(
                        "Received event id={} (main={}, notes={}, ai={})",
                        event.id, current_main_id, current_notes_id, current_ai_id
                    ),
                );

                // Check if it's the main app hotkey
                if event.id == current_main_id {
                    let count = HOTKEY_TRIGGER_COUNT.fetch_add(1, Ordering::SeqCst);
                    // Send via async_channel for immediate event-driven handling
                    if hotkey_channel().0.send_blocking(()).is_err() {
                        logging::log("HOTKEY", "Hotkey channel closed, cannot send");
                    }
                    logging::log(
                        "HOTKEY",
                        &format!("Main hotkey pressed (trigger #{})", count + 1),
                    );
                }
                // Check if it's the notes hotkey - dispatch directly to main thread via GCD
                else if event.id == current_notes_id {
                    logging::log(
                        "HOTKEY",
                        "Notes hotkey pressed - dispatching to main thread",
                    );
                    dispatch_notes_hotkey();
                }
                // Check if it's the AI hotkey - dispatch directly to main thread via GCD
                else if event.id == current_ai_id {
                    logging::log("HOTKEY", "AI hotkey pressed - dispatching to main thread");
                    dispatch_ai_hotkey();
                }
                // Check if it's a script shortcut
                else if let Some(script_path) = script_hotkey_map.get(&event.id) {
                    logging::log(
                        "HOTKEY",
                        &format!("Script shortcut triggered: {}", script_path),
                    );
                    // Send the script path to be executed
                    if script_hotkey_channel()
                        .0
                        .send_blocking(script_path.clone())
                        .is_err()
                    {
                        logging::log("HOTKEY", "Script hotkey channel closed, cannot send");
                    }
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_channel::TryRecvError;

    #[test]
    fn hotkey_channels_are_independent() {
        while hotkey_channel().1.try_recv().is_ok() {}
        while script_hotkey_channel().1.try_recv().is_ok() {}

        hotkey_channel().0.send_blocking(()).expect("send hotkey");
        assert!(matches!(
            script_hotkey_channel().1.try_recv(),
            Err(TryRecvError::Empty)
        ));
        assert!(hotkey_channel().1.try_recv().is_ok());

        script_hotkey_channel()
            .0
            .send_blocking("script".to_string())
            .expect("send script hotkey");
        assert_eq!(
            script_hotkey_channel()
                .1
                .try_recv()
                .expect("recv script hotkey"),
            "script"
        );
    }

    // =============================================================================
    // ScriptHotkeyManager Unit Tests
    // =============================================================================
    // Note: These tests cannot actually register system hotkeys in the test environment
    // because GlobalHotKeyManager requires a running event loop and proper OS permissions.
    // Instead, we test the logic of the manager's internal tracking.

    mod script_hotkey_manager_tests {
        use super::*;

        /// Helper to create a manager for testing.
        /// Note: Registration will fail without an event loop, but we can test tracking logic.
        fn create_test_manager() -> Option<ScriptHotkeyManager> {
            // GlobalHotKeyManager::new() may fail in test environment
            GlobalHotKeyManager::new()
                .ok()
                .map(ScriptHotkeyManager::new)
        }

        #[test]
        fn test_manager_creation() {
            // Just verify we can create the struct (manager creation may fail in CI)
            if let Some(manager) = create_test_manager() {
                assert!(manager.hotkey_map.is_empty());
                assert!(manager.path_to_id.is_empty());
            }
        }

        #[test]
        fn test_get_registered_hotkeys_empty() {
            if let Some(manager) = create_test_manager() {
                assert!(manager.get_registered_hotkeys().is_empty());
            }
        }

        #[test]
        fn test_is_registered_false_for_unknown_path() {
            if let Some(manager) = create_test_manager() {
                assert!(!manager.is_registered("/some/unknown/path.ts"));
            }
        }

        #[test]
        fn test_unregister_nonexistent_is_noop() {
            if let Some(mut manager) = create_test_manager() {
                // Should not error when unregistering a path that was never registered
                let result = manager.unregister("/nonexistent/path.ts");
                assert!(result.is_ok());
            }
        }

        #[test]
        fn test_update_none_to_none_is_noop() {
            if let Some(mut manager) = create_test_manager() {
                // No old, no new -> no-op, should succeed
                let result = manager.update("/some/path.ts", None, None);
                assert!(result.is_ok());
            }
        }

        // Note: The following tests would require a working GlobalHotKeyManager
        // which may not be available in all test environments.
        // In a real CI environment, these would be integration tests.

        #[test]
        fn test_register_tracks_mapping() {
            if let Some(mut manager) = create_test_manager() {
                // Try to register - this may fail in test environment, that's OK
                let result = manager.register("/test/script.ts", "cmd+shift+t");
                if result.is_ok() {
                    // If registration succeeded, verify tracking
                    assert!(manager.is_registered("/test/script.ts"));
                    let hotkeys = manager.get_registered_hotkeys();
                    assert_eq!(hotkeys.len(), 1);
                    assert_eq!(hotkeys[0].0, "/test/script.ts");
                }
                // If it failed (no event loop), that's expected in test env
            }
        }

        #[test]
        fn test_unregister_removes_tracking() {
            if let Some(mut manager) = create_test_manager() {
                // Try to register first
                if manager.register("/test/script.ts", "cmd+shift+u").is_ok() {
                    assert!(manager.is_registered("/test/script.ts"));

                    // Now unregister
                    let result = manager.unregister("/test/script.ts");
                    assert!(result.is_ok());
                    assert!(!manager.is_registered("/test/script.ts"));
                }
            }
        }

        #[test]
        fn test_update_add_hotkey() {
            if let Some(mut manager) = create_test_manager() {
                // None -> Some = add
                let result = manager.update("/test/add.ts", None, Some("cmd+shift+a"));
                if result.is_ok() {
                    assert!(manager.is_registered("/test/add.ts"));
                }
            }
        }

        #[test]
        fn test_update_remove_hotkey() {
            if let Some(mut manager) = create_test_manager() {
                // First register
                if manager.register("/test/remove.ts", "cmd+shift+r").is_ok() {
                    // Some -> None = remove
                    let result = manager.update("/test/remove.ts", Some("cmd+shift+r"), None);
                    assert!(result.is_ok());
                    assert!(!manager.is_registered("/test/remove.ts"));
                }
            }
        }

        #[test]
        fn test_update_change_hotkey() {
            if let Some(mut manager) = create_test_manager() {
                // First register with old shortcut
                if manager.register("/test/change.ts", "cmd+shift+c").is_ok() {
                    // Some -> Some (different) = change
                    let result =
                        manager.update("/test/change.ts", Some("cmd+shift+c"), Some("cmd+alt+c"));
                    if result.is_ok() {
                        // Should still be registered (with new shortcut)
                        assert!(manager.is_registered("/test/change.ts"));
                    }
                }
            }
        }

        #[test]
        fn test_get_script_path() {
            if let Some(mut manager) = create_test_manager() {
                if let Ok(hotkey_id) = manager.register("/test/lookup.ts", "cmd+shift+l") {
                    let path = manager.get_script_path(hotkey_id);
                    assert_eq!(path, Some(&"/test/lookup.ts".to_string()));

                    // Unknown ID returns None
                    assert!(manager.get_script_path(99999).is_none());
                }
            }
        }
    }
}

</file>

</files>
📊 Pack Summary:
────────────────
  Total Files: 3 files
  Search Mode: ripgrep (fast)
  Total Tokens: ~31.3K (31,265 exact)
  Total Chars: 155,179 chars
       Output: -

📁 Extensions Found:
────────────────────
  .rs

📂 Top 10 Files (by tokens):
──────────────────────────
     20.1K - src/app_impl.rs
     10.8K - src/hotkeys.rs
       275 - src/config/mod.rs

---
## Implementation Guide

### Key Code Locations

#### 1. Global State (src/hotkeys.rs, lines 15-40)
```rust
static MAIN_MANAGER: OnceLock<Mutex<GlobalHotKeyManager>> = OnceLock::new();
static MAIN_HOTKEY_ID: AtomicU32 = AtomicU32::new(0);
static NOTES_HOTKEY_ID: AtomicU32 = AtomicU32::new(0);
static AI_HOTKEY_ID: AtomicU32 = AtomicU32::new(0);

struct CurrentHotkeys {
    main: Option<HotKey>,
    notes: Option<HotKey>,
    ai: Option<HotKey>,
}
static CURRENT_HOTKEYS: OnceLock<Mutex<CurrentHotkeys>> = OnceLock::new();
```

#### 2. update_hotkeys() (src/hotkeys.rs, lines 124-250)
The core hot-reload function:
- Acquires manager lock from `MAIN_MANAGER`
- For each hotkey type, compares new ID vs atomic ID
- If different: unregisters old, registers new, updates atomic + storage

#### 3. Manager Storage (src/hotkeys.rs, lines 757-770)
In `start_hotkey_listener()`:
```rust
if MAIN_MANAGER.set(Mutex::new(manager)).is_err() {
    logging::log("HOTKEY", "Manager already initialized");
    return;
}
let manager_guard = MAIN_MANAGER.get().unwrap().lock()...
```

#### 4. Atomic Storage After Registration (src/hotkeys.rs, lines 871-876)
```rust
MAIN_HOTKEY_ID.store(main_hotkey_id, Ordering::SeqCst);
{
    let mut hotkeys = get_current_hotkeys().lock().unwrap();
    hotkeys.main = Some(HotKey::new(Some(modifiers), code));
}
```

#### 5. Lock Drop Before Event Loop (src/hotkeys.rs, line 1089)
```rust
drop(manager_guard);  // Allow update_hotkeys() to acquire lock
```

#### 6. Event Loop Reading Atomics (src/hotkeys.rs, lines 1109-1112)
```rust
let current_main_id = MAIN_HOTKEY_ID.load(Ordering::SeqCst);
let current_notes_id = NOTES_HOTKEY_ID.load(Ordering::SeqCst);
let current_ai_id = AI_HOTKEY_ID.load(Ordering::SeqCst);
```

#### 7. Config Reload Call (src/app_impl.rs, line 347)
```rust
hotkeys::update_hotkeys(&self.config);
```

### Review Checklist

- [ ] **Thread Safety**: AtomicU32 with SeqCst ordering for cross-thread visibility
- [ ] **Mutex Deadlock**: Manager lock dropped before event loop to prevent deadlock
- [ ] **HotKey Lifecycle**: Old HotKey objects stored for proper OS unregistration
- [ ] **Error Handling**: All lock acquisitions have error handling with logging
- [ ] **ID Comparison**: Uses hotkey.id() which is deterministic (same mods+code = same ID)

### Potential Concerns to Review

1. **SeqCst Ordering**: Could potentially use Acquire/Release for better performance, but SeqCst is safer
2. **Lock Contention**: If config changes rapidly, could have brief contention on MAIN_MANAGER
3. **Script Hotkeys**: Currently NOT hot-reloaded (only main/notes/AI) - by design for simplicity

---
## Instructions for Next AI Agent

You are reviewing a hotkey hot-reload implementation for a Rust desktop application using GPUI framework.

### Your Review Tasks:
1. **Verify thread safety** of the atomic operations and mutex usage
2. **Check for potential deadlocks** in the lock acquisition patterns
3. **Validate the unregister→register sequence** ensures no hotkey leaks
4. **Review error handling** - are failures logged appropriately?
5. **Consider edge cases**: What if config reload happens during hotkey press?

### Key Questions to Answer:
- Is `SeqCst` ordering necessary, or would `Acquire/Release` suffice?
- Is the manager lock drop timing correct (line 1089)?
- Could rapid config changes cause issues?
- Are there any race conditions between the event loop and update_hotkeys()?

### If You Find Issues:
Provide exact file paths and line numbers with corrected code snippets.

### Testing Approach:
The implementation can be tested by:
1. Running the app with `SCRIPT_KIT_AI_LOG=1`
2. Modifying `~/.scriptkit/kit/config.ts` hotkey settings
3. Checking logs for "Hot-reloaded main/notes/AI hotkey" messages
4. Verifying new hotkeys work without app restart
