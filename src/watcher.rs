#![allow(dead_code)]
use notify::{recommended_watcher, RecursiveMode, Result as NotifyResult, Watcher};
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::{Duration, Instant};

use std::process::Command;
use tracing::{debug, info, warn};

/// Internal control messages for watcher threads
enum ControlMsg {
    /// Signal from notify callback with a file event
    Notify(notify::Result<notify::Event>),
    /// Signal to stop the watcher thread
    Stop,
}

/// Debounce configuration
const DEBOUNCE_MS: u64 = 500;
/// Storm threshold: if more than this many unique paths pending, collapse to FullReload
const STORM_THRESHOLD: usize = 200;

/// Check if an event kind is relevant (not just Access events)
fn is_relevant_event_kind(kind: &notify::EventKind) -> bool {
    !matches!(kind, notify::EventKind::Access(_))
}

/// Event emitted when config needs to be reloaded
#[derive(Debug, Clone)]
pub enum ConfigReloadEvent {
    Reload,
}

/// Event emitted when theme needs to be reloaded
#[derive(Debug, Clone)]
pub enum ThemeReloadEvent {
    Reload,
}

/// Event emitted when scripts need to be reloaded
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptReloadEvent {
    /// A specific file was modified
    FileChanged(PathBuf),
    /// A new file was created
    FileCreated(PathBuf),
    /// A file was deleted
    FileDeleted(PathBuf),
    /// Fallback for complex events (e.g., bulk changes, renames)
    FullReload,
}

/// Event emitted when system appearance changes (light/dark mode)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppearanceChangeEvent {
    /// Dark mode is now active
    Dark,
    /// Light mode is now active
    Light,
}

/// Watches ~/.scriptkit/kit/config.ts for changes and emits reload events
///
/// Uses trailing-edge debounce: each new event resets the deadline.
/// Handles atomic saves (rename/remove operations).
/// Properly shuts down via Stop control message.
pub struct ConfigWatcher {
    tx: Option<Sender<ConfigReloadEvent>>,
    control_tx: Option<Sender<ControlMsg>>,
    watcher_thread: Option<thread::JoinHandle<()>>,
}

impl ConfigWatcher {
    /// Create a new ConfigWatcher
    ///
    /// Returns a tuple of (watcher, receiver) where receiver will emit ConfigReloadEvent
    /// when the config file changes.
    pub fn new() -> (Self, Receiver<ConfigReloadEvent>) {
        let (tx, rx) = channel();
        let watcher = ConfigWatcher {
            tx: Some(tx),
            control_tx: None,
            watcher_thread: None,
        };
        (watcher, rx)
    }

    /// Start watching the config file for changes
    ///
    /// This spawns a background thread that watches ~/.scriptkit/kit/config.ts and sends
    /// reload events through the receiver when changes are detected.
    pub fn start(&mut self) -> NotifyResult<()> {
        let tx = self
            .tx
            .take()
            .ok_or_else(|| std::io::Error::other("watcher already started"))?;

        let (control_tx, control_rx) = channel::<ControlMsg>();
        let callback_tx = control_tx.clone();
        self.control_tx = Some(control_tx);

        let target_path = PathBuf::from(shellexpand::tilde("~/.scriptkit/kit/config.ts").as_ref());

        let thread_handle = thread::spawn(move || {
            if let Err(e) = Self::watch_loop(target_path, tx, control_rx, callback_tx) {
                warn!(error = %e, watcher = "config", "Config watcher error");
            }
        });

        self.watcher_thread = Some(thread_handle);
        Ok(())
    }

    /// Internal watch loop running in background thread
    fn watch_loop(
        target_path: PathBuf,
        out_tx: Sender<ConfigReloadEvent>,
        control_rx: Receiver<ControlMsg>,
        callback_tx: Sender<ControlMsg>,
    ) -> NotifyResult<()> {
        let target_name: OsString = target_path
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new(""))
            .to_owned();

        let watch_path = target_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));

        // Create the watcher with a callback that forwards to control channel
        let mut watcher: Box<dyn Watcher> = Box::new(recommended_watcher(
            move |res: notify::Result<notify::Event>| {
                let _ = callback_tx.send(ControlMsg::Notify(res));
            },
        )?);

        watcher.watch(watch_path, RecursiveMode::NonRecursive)?;

        info!(
            path = %watch_path.display(),
            target = ?target_name,
            "Config watcher started"
        );

        let debounce = Duration::from_millis(DEBOUNCE_MS);
        let mut deadline: Option<Instant> = None;

        loop {
            let msg = match deadline {
                None => {
                    // No pending reload => wait indefinitely for next msg
                    match control_rx.recv() {
                        Ok(m) => m,
                        Err(_) => break,
                    }
                }
                Some(dl) => {
                    // Pending reload => wait until deadline, unless a new event comes in
                    let timeout = dl.saturating_duration_since(Instant::now());
                    match control_rx.recv_timeout(timeout) {
                        Ok(m) => m,
                        Err(RecvTimeoutError::Timeout) => {
                            // Quiet period ended => emit reload
                            debug!(file = ?target_name, "Config debounce complete, emitting reload");
                            let _ = out_tx.send(ConfigReloadEvent::Reload);
                            info!(file = ?target_name, "Config file changed, emitting reload event");
                            deadline = None;
                            continue;
                        }
                        Err(RecvTimeoutError::Disconnected) => break,
                    }
                }
            };

            match msg {
                ControlMsg::Stop => {
                    info!(watcher = "config", "Config watcher received stop signal");
                    break;
                }

                ControlMsg::Notify(Err(e)) => {
                    warn!(error = %e, watcher = "config", "notify delivered error");
                }

                ControlMsg::Notify(Ok(event)) => {
                    // Filter: does this event mention the target filename?
                    let touches_target = event.paths.iter().any(|p| {
                        p.file_name()
                            .map(|n| n == target_name.as_os_str())
                            .unwrap_or(false)
                    });

                    // Treat everything except Access as potentially relevant
                    // This covers atomic saves (remove/rename) too
                    let relevant_kind = is_relevant_event_kind(&event.kind);

                    if touches_target && relevant_kind {
                        // Trailing-edge debounce: reset deadline on every hit
                        debug!(
                            file = ?target_name,
                            event_kind = ?event.kind,
                            "Config change detected, resetting debounce"
                        );
                        deadline = Some(Instant::now() + debounce);
                    }
                }
            }
        }

        info!(watcher = "config", "Config watcher shutting down");
        Ok(())
    }
}

impl Drop for ConfigWatcher {
    fn drop(&mut self) {
        // Send stop signal first
        if let Some(tx) = self.control_tx.take() {
            let _ = tx.send(ControlMsg::Stop);
        }
        // Now join - the thread will exit because it received Stop
        if let Some(handle) = self.watcher_thread.take() {
            let _ = handle.join();
        }
    }
}

/// Watches ~/.scriptkit/kit/theme.json for changes and emits reload events
///
/// Uses trailing-edge debounce: each new event resets the deadline.
/// Handles atomic saves (rename/remove operations).
/// Properly shuts down via Stop control message.
pub struct ThemeWatcher {
    tx: Option<Sender<ThemeReloadEvent>>,
    control_tx: Option<Sender<ControlMsg>>,
    watcher_thread: Option<thread::JoinHandle<()>>,
}

impl ThemeWatcher {
    /// Create a new ThemeWatcher
    ///
    /// Returns a tuple of (watcher, receiver) where receiver will emit ThemeReloadEvent
    /// when the theme file changes.
    pub fn new() -> (Self, Receiver<ThemeReloadEvent>) {
        let (tx, rx) = channel();
        let watcher = ThemeWatcher {
            tx: Some(tx),
            control_tx: None,
            watcher_thread: None,
        };
        (watcher, rx)
    }

    /// Start watching the theme file for changes
    ///
    /// This spawns a background thread that watches ~/.scriptkit/kit/theme.json and sends
    /// reload events through the receiver when changes are detected.
    pub fn start(&mut self) -> NotifyResult<()> {
        let tx = self
            .tx
            .take()
            .ok_or_else(|| std::io::Error::other("watcher already started"))?;

        let (control_tx, control_rx) = channel::<ControlMsg>();
        let callback_tx = control_tx.clone();
        self.control_tx = Some(control_tx);

        let target_path = PathBuf::from(shellexpand::tilde("~/.scriptkit/kit/theme.json").as_ref());

        let thread_handle = thread::spawn(move || {
            if let Err(e) = Self::watch_loop(target_path, tx, control_rx, callback_tx) {
                warn!(error = %e, watcher = "theme", "Theme watcher error");
            }
        });

        self.watcher_thread = Some(thread_handle);
        Ok(())
    }

    /// Internal watch loop running in background thread
    fn watch_loop(
        target_path: PathBuf,
        out_tx: Sender<ThemeReloadEvent>,
        control_rx: Receiver<ControlMsg>,
        callback_tx: Sender<ControlMsg>,
    ) -> NotifyResult<()> {
        let target_name: OsString = target_path
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new(""))
            .to_owned();

        let watch_path = target_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));

        let mut watcher: Box<dyn Watcher> = Box::new(recommended_watcher(
            move |res: notify::Result<notify::Event>| {
                let _ = callback_tx.send(ControlMsg::Notify(res));
            },
        )?);

        watcher.watch(watch_path, RecursiveMode::NonRecursive)?;

        info!(
            path = %watch_path.display(),
            target = ?target_name,
            "Theme watcher started"
        );

        let debounce = Duration::from_millis(DEBOUNCE_MS);
        let mut deadline: Option<Instant> = None;

        loop {
            let msg = match deadline {
                None => match control_rx.recv() {
                    Ok(m) => m,
                    Err(_) => break,
                },
                Some(dl) => {
                    let timeout = dl.saturating_duration_since(Instant::now());
                    match control_rx.recv_timeout(timeout) {
                        Ok(m) => m,
                        Err(RecvTimeoutError::Timeout) => {
                            debug!(file = ?target_name, "Theme debounce complete, emitting reload");
                            let _ = out_tx.send(ThemeReloadEvent::Reload);
                            info!(file = ?target_name, "Theme file changed, emitting reload event");
                            deadline = None;
                            continue;
                        }
                        Err(RecvTimeoutError::Disconnected) => break,
                    }
                }
            };

            match msg {
                ControlMsg::Stop => {
                    info!(watcher = "theme", "Theme watcher received stop signal");
                    break;
                }

                ControlMsg::Notify(Err(e)) => {
                    warn!(error = %e, watcher = "theme", "notify delivered error");
                }

                ControlMsg::Notify(Ok(event)) => {
                    let touches_target = event.paths.iter().any(|p| {
                        p.file_name()
                            .map(|n| n == target_name.as_os_str())
                            .unwrap_or(false)
                    });

                    let relevant_kind = is_relevant_event_kind(&event.kind);

                    if touches_target && relevant_kind {
                        debug!(
                            file = ?target_name,
                            event_kind = ?event.kind,
                            "Theme change detected, resetting debounce"
                        );
                        deadline = Some(Instant::now() + debounce);
                    }
                }
            }
        }

        info!(watcher = "theme", "Theme watcher shutting down");
        Ok(())
    }
}

impl Drop for ThemeWatcher {
    fn drop(&mut self) {
        if let Some(tx) = self.control_tx.take() {
            let _ = tx.send(ControlMsg::Stop);
        }
        if let Some(handle) = self.watcher_thread.take() {
            let _ = handle.join();
        }
    }
}

/// Check if a file path is a relevant script file (ts, js, or md)
fn is_relevant_script_file(path: &std::path::Path) -> bool {
    // Skip hidden files
    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
        if file_name.starts_with('.') {
            return false;
        }
    }

    // Check for relevant extensions
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("ts") | Some("js") | Some("md")
    )
}

/// Compute the next deadline from pending events
fn next_deadline(
    pending: &HashMap<PathBuf, (ScriptReloadEvent, Instant)>,
    debounce: Duration,
) -> Option<Instant> {
    pending.values().map(|(_, t)| *t + debounce).min()
}

/// Flush expired events from pending map
fn flush_expired(
    pending: &mut HashMap<PathBuf, (ScriptReloadEvent, Instant)>,
    debounce: Duration,
    out_tx: &Sender<ScriptReloadEvent>,
) {
    let now = Instant::now();
    let mut to_send: Vec<ScriptReloadEvent> = Vec::new();

    pending.retain(|path, (ev, t)| {
        if now.duration_since(*t) >= debounce {
            debug!(path = %path.display(), event = ?ev, "Script debounce complete, flushing");
            to_send.push(ev.clone());
            false
        } else {
            true
        }
    });

    for ev in to_send {
        info!(event = ?ev, "Emitting script reload event");
        let _ = out_tx.send(ev);
    }
}

/// Watches ~/.scriptkit/kit/*/scripts and ~/.scriptkit/kit/*/extensions directories for changes
///
/// Uses per-file trailing-edge debounce with storm coalescing.
/// No separate flush thread - all debouncing in single recv_timeout loop.
/// Properly shuts down via Stop control message.
pub struct ScriptWatcher {
    tx: Option<Sender<ScriptReloadEvent>>,
    control_tx: Option<Sender<ControlMsg>>,
    watcher_thread: Option<thread::JoinHandle<()>>,
}

impl ScriptWatcher {
    /// Create a new ScriptWatcher
    ///
    /// Returns a tuple of (watcher, receiver) where receiver will emit ScriptReloadEvent
    /// when files in the scripts directory change.
    pub fn new() -> (Self, Receiver<ScriptReloadEvent>) {
        let (tx, rx) = channel();
        let watcher = ScriptWatcher {
            tx: Some(tx),
            control_tx: None,
            watcher_thread: None,
        };
        (watcher, rx)
    }

    /// Start watching the scripts directory for changes
    ///
    /// This spawns a background thread that watches ~/.scriptkit/scripts recursively and sends
    /// reload events through the receiver when scripts are added, modified, or deleted.
    pub fn start(&mut self) -> NotifyResult<()> {
        let tx = self
            .tx
            .take()
            .ok_or_else(|| std::io::Error::other("watcher already started"))?;

        let (control_tx, control_rx) = channel::<ControlMsg>();
        let callback_tx = control_tx.clone();
        self.control_tx = Some(control_tx);

        let scripts_path =
            PathBuf::from(shellexpand::tilde("~/.scriptkit/kit/main/scripts").as_ref());
        let extensions_path =
            PathBuf::from(shellexpand::tilde("~/.scriptkit/kit/main/extensions").as_ref());

        let thread_handle = thread::spawn(move || {
            if let Err(e) =
                Self::watch_loop(scripts_path, extensions_path, tx, control_rx, callback_tx)
            {
                warn!(error = %e, watcher = "scripts", "Script watcher error");
            }
        });

        self.watcher_thread = Some(thread_handle);
        Ok(())
    }

    /// Internal watch loop running in background thread
    fn watch_loop(
        scripts_path: PathBuf,
        extensions_path: PathBuf,
        out_tx: Sender<ScriptReloadEvent>,
        control_rx: Receiver<ControlMsg>,
        callback_tx: Sender<ControlMsg>,
    ) -> NotifyResult<()> {
        let mut watcher: Box<dyn Watcher> = Box::new(recommended_watcher(
            move |res: notify::Result<notify::Event>| {
                let _ = callback_tx.send(ControlMsg::Notify(res));
            },
        )?);

        watcher.watch(&scripts_path, RecursiveMode::Recursive)?;

        if extensions_path.exists() {
            watcher.watch(&extensions_path, RecursiveMode::Recursive)?;
            info!(
                path = %extensions_path.display(),
                recursive = true,
                "Scriptlets watcher started"
            );
        }

        info!(
            path = %scripts_path.display(),
            recursive = true,
            "Script watcher started"
        );

        let debounce = Duration::from_millis(DEBOUNCE_MS);
        let mut pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();

        loop {
            let deadline = next_deadline(&pending, debounce);

            let msg = match deadline {
                None => match control_rx.recv() {
                    Ok(m) => m,
                    Err(_) => break,
                },
                Some(dl) => {
                    let timeout = dl.saturating_duration_since(Instant::now());
                    match control_rx.recv_timeout(timeout) {
                        Ok(m) => m,
                        Err(RecvTimeoutError::Timeout) => {
                            flush_expired(&mut pending, debounce, &out_tx);
                            continue;
                        }
                        Err(RecvTimeoutError::Disconnected) => break,
                    }
                }
            };

            match msg {
                ControlMsg::Stop => {
                    info!(watcher = "scripts", "Script watcher received stop signal");
                    break;
                }

                ControlMsg::Notify(Err(e)) => {
                    warn!(error = %e, watcher = "scripts", "notify delivered error");
                }

                ControlMsg::Notify(Ok(event)) => {
                    let kind = &event.kind;

                    for path in event.paths.iter() {
                        if !is_relevant_script_file(path) {
                            continue;
                        }

                        let reload_event = match kind {
                            notify::EventKind::Create(_) => {
                                ScriptReloadEvent::FileCreated(path.clone())
                            }
                            notify::EventKind::Modify(_) => {
                                ScriptReloadEvent::FileChanged(path.clone())
                            }
                            notify::EventKind::Remove(_) => {
                                ScriptReloadEvent::FileDeleted(path.clone())
                            }
                            // Access events are not relevant
                            notify::EventKind::Access(_) => continue,
                            // For Other/Any events, use FullReload as a safe fallback
                            _ => ScriptReloadEvent::FullReload,
                        };

                        debug!(
                            path = %path.display(),
                            event_kind = ?kind,
                            "Script change detected, updating pending"
                        );
                        pending.insert(path.clone(), (reload_event, Instant::now()));

                        // Storm coalescing: if too many pending events, collapse to FullReload
                        if pending.len() >= STORM_THRESHOLD {
                            warn!(
                                pending_count = pending.len(),
                                threshold = STORM_THRESHOLD,
                                "Event storm detected, collapsing to FullReload"
                            );
                            pending.clear();
                            let _ = out_tx.send(ScriptReloadEvent::FullReload);
                            break;
                        }
                    }
                }
            }
        }

        info!(watcher = "scripts", "Script watcher shutting down");
        Ok(())
    }
}

impl Drop for ScriptWatcher {
    fn drop(&mut self) {
        if let Some(tx) = self.control_tx.take() {
            let _ = tx.send(ControlMsg::Stop);
        }
        if let Some(handle) = self.watcher_thread.take() {
            let _ = handle.join();
        }
    }
}

/// Watches system appearance (light/dark mode) for changes and emits events
///
/// This watcher polls the system appearance setting every 2 seconds by running
/// the `defaults read -g AppleInterfaceStyle` command on macOS.
///
/// Properly shuts down via stop flag.
pub struct AppearanceWatcher {
    tx: Option<async_channel::Sender<AppearanceChangeEvent>>,
    stop_flag: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    watcher_thread: Option<thread::JoinHandle<()>>,
}

impl AppearanceWatcher {
    /// Create a new AppearanceWatcher
    ///
    /// Returns a tuple of (watcher, receiver) where receiver will emit AppearanceChangeEvent
    /// when the system appearance changes.
    pub fn new() -> (Self, async_channel::Receiver<AppearanceChangeEvent>) {
        let (tx, rx) = async_channel::bounded(100);
        let watcher = AppearanceWatcher {
            tx: Some(tx),
            stop_flag: None,
            watcher_thread: None,
        };
        (watcher, rx)
    }

    /// Start watching the system appearance for changes
    ///
    /// This spawns a background thread that polls the system appearance every 2 seconds
    /// and sends appearance change events through the receiver when changes are detected.
    pub fn start(&mut self) -> Result<(), String> {
        let tx = self
            .tx
            .take()
            .ok_or_else(|| "watcher already started".to_string())?;

        let stop_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let thread_stop_flag = stop_flag.clone();
        self.stop_flag = Some(stop_flag);

        let thread_handle = thread::spawn(move || {
            if let Err(e) = Self::watch_loop(tx, thread_stop_flag) {
                warn!(error = %e, watcher = "appearance", "Appearance watcher error");
            }
        });

        self.watcher_thread = Some(thread_handle);
        Ok(())
    }

    /// Internal watch loop running in background thread
    fn watch_loop(
        tx: async_channel::Sender<AppearanceChangeEvent>,
        stop_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Result<(), String> {
        let mut last_appearance: Option<AppearanceChangeEvent> = None;

        info!(poll_interval_secs = 2, "Appearance watcher started");

        loop {
            // Check stop flag first
            if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                info!(
                    watcher = "appearance",
                    "Appearance watcher received stop signal"
                );
                break;
            }

            // Detect current system appearance
            let current_appearance = Self::detect_appearance();

            // Send event if appearance changed
            if last_appearance != Some(current_appearance.clone()) {
                let mode = match current_appearance {
                    AppearanceChangeEvent::Dark => "dark",
                    AppearanceChangeEvent::Light => "light",
                };
                info!(mode = mode, "System appearance changed");
                if tx.send_blocking(current_appearance.clone()).is_err() {
                    info!(
                        watcher = "appearance",
                        "Appearance watcher receiver dropped, shutting down"
                    );
                    break;
                }
                last_appearance = Some(current_appearance);
            }

            // Poll with interruptible sleep (check stop flag more frequently)
            for _ in 0..20 {
                // 20 * 100ms = 2s
                if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                    info!(
                        watcher = "appearance",
                        "Appearance watcher received stop signal during sleep"
                    );
                    return Ok(());
                }
                thread::sleep(Duration::from_millis(100));
            }
        }

        info!(watcher = "appearance", "Appearance watcher shutting down");
        Ok(())
    }

    /// Detect the current system appearance
    fn detect_appearance() -> AppearanceChangeEvent {
        match Command::new("defaults")
            .args(["read", "-g", "AppleInterfaceStyle"])
            .output()
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.to_lowercase().contains("dark") {
                    AppearanceChangeEvent::Dark
                } else {
                    AppearanceChangeEvent::Light
                }
            }
            Err(_) => {
                // Command failed, likely in light mode on macOS
                AppearanceChangeEvent::Light
            }
        }
    }
}

impl Drop for AppearanceWatcher {
    fn drop(&mut self) {
        // Signal stop
        if let Some(flag) = self.stop_flag.take() {
            flag.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        // Wait for thread to finish
        if let Some(handle) = self.watcher_thread.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_watcher_creation() {
        let (_watcher, _rx) = ConfigWatcher::new();
        // Watcher should be created without panicking
    }

    #[test]
    fn test_config_reload_event_clone() {
        let event = ConfigReloadEvent::Reload;
        let _cloned = event.clone();
        // Event should be cloneable
    }

    #[test]
    fn test_theme_watcher_creation() {
        let (_watcher, _rx) = ThemeWatcher::new();
        // Watcher should be created without panicking
    }

    #[test]
    fn test_theme_reload_event_clone() {
        let event = ThemeReloadEvent::Reload;
        let _cloned = event.clone();
        // Event should be cloneable
    }

    #[test]
    fn test_script_watcher_creation() {
        let (_watcher, _rx) = ScriptWatcher::new();
        // Watcher should be created without panicking
    }

    #[test]
    fn test_script_reload_event_clone() {
        let event = ScriptReloadEvent::FullReload;
        let _cloned = event.clone();
        // Event should be cloneable
    }

    #[test]
    fn test_script_reload_event_file_changed() {
        let path = PathBuf::from("/test/path/script.ts");
        let event = ScriptReloadEvent::FileChanged(path.clone());

        // Verify the event contains the correct path
        if let ScriptReloadEvent::FileChanged(event_path) = event {
            assert_eq!(event_path, path);
        } else {
            panic!("Expected FileChanged variant");
        }
    }

    #[test]
    fn test_script_reload_event_file_created() {
        let path = PathBuf::from("/test/path/new-script.ts");
        let event = ScriptReloadEvent::FileCreated(path.clone());

        // Verify the event contains the correct path
        if let ScriptReloadEvent::FileCreated(event_path) = event {
            assert_eq!(event_path, path);
        } else {
            panic!("Expected FileCreated variant");
        }
    }

    #[test]
    fn test_script_reload_event_file_deleted() {
        let path = PathBuf::from("/test/path/deleted-script.ts");
        let event = ScriptReloadEvent::FileDeleted(path.clone());

        // Verify the event contains the correct path
        if let ScriptReloadEvent::FileDeleted(event_path) = event {
            assert_eq!(event_path, path);
        } else {
            panic!("Expected FileDeleted variant");
        }
    }

    #[test]
    fn test_script_reload_event_equality() {
        let path1 = PathBuf::from("/test/path/script.ts");
        let path2 = PathBuf::from("/test/path/script.ts");
        let path3 = PathBuf::from("/test/path/other.ts");

        // Same path should be equal
        assert_eq!(
            ScriptReloadEvent::FileChanged(path1.clone()),
            ScriptReloadEvent::FileChanged(path2.clone())
        );

        // Different paths should not be equal
        assert_ne!(
            ScriptReloadEvent::FileChanged(path1.clone()),
            ScriptReloadEvent::FileChanged(path3.clone())
        );

        // Different event types should not be equal
        assert_ne!(
            ScriptReloadEvent::FileChanged(path1.clone()),
            ScriptReloadEvent::FileCreated(path1.clone())
        );

        // FullReload should equal itself
        assert_eq!(ScriptReloadEvent::FullReload, ScriptReloadEvent::FullReload);
    }

    #[test]
    fn test_extract_file_path_from_event() {
        // Test helper function for extracting paths from notify events
        use notify::event::{CreateKind, ModifyKind, RemoveKind};

        let test_path = PathBuf::from("/Users/test/.scriptkit/scripts/hello.ts");

        // Test Create event
        let create_event = notify::Event {
            kind: notify::EventKind::Create(CreateKind::File),
            paths: vec![test_path.clone()],
            attrs: Default::default(),
        };
        assert_eq!(create_event.paths.first(), Some(&test_path));

        // Test Modify event
        let modify_event = notify::Event {
            kind: notify::EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Content)),
            paths: vec![test_path.clone()],
            attrs: Default::default(),
        };
        assert_eq!(modify_event.paths.first(), Some(&test_path));

        // Test Remove event
        let remove_event = notify::Event {
            kind: notify::EventKind::Remove(RemoveKind::File),
            paths: vec![test_path.clone()],
            attrs: Default::default(),
        };
        assert_eq!(remove_event.paths.first(), Some(&test_path));
    }

    #[test]
    fn test_is_relevant_script_file() {
        use std::path::Path;

        // Test that we correctly identify relevant script files
        let ts_path = Path::new("/Users/test/.scriptkit/scripts/hello.ts");
        let js_path = Path::new("/Users/test/.scriptkit/scripts/hello.js");
        let md_path = Path::new("/Users/test/.scriptkit/scriptlets/hello.md");
        let txt_path = Path::new("/Users/test/.scriptkit/scripts/readme.txt");
        let hidden_path = Path::new("/Users/test/.scriptkit/scripts/.hidden.ts");

        // TypeScript files should be relevant
        assert!(is_relevant_script_file(ts_path));

        // JavaScript files should be relevant
        assert!(is_relevant_script_file(js_path));

        // Markdown files in scriptlets should be relevant
        assert!(is_relevant_script_file(md_path));

        // Other file types should not be relevant
        assert!(!is_relevant_script_file(txt_path));

        // Hidden files should not be relevant
        assert!(!is_relevant_script_file(hidden_path));
    }

    #[test]
    fn test_appearance_change_event_clone() {
        let event_dark = AppearanceChangeEvent::Dark;
        let _cloned = event_dark.clone();
        let event_light = AppearanceChangeEvent::Light;
        let _cloned = event_light.clone();
        // Events should be cloneable
    }

    #[test]
    fn test_appearance_change_event_equality() {
        let dark1 = AppearanceChangeEvent::Dark;
        let dark2 = AppearanceChangeEvent::Dark;
        let light = AppearanceChangeEvent::Light;

        assert_eq!(dark1, dark2);
        assert_ne!(dark1, light);
    }

    #[test]
    fn test_appearance_watcher_creation() {
        let (_watcher, _rx) = AppearanceWatcher::new();
        // Watcher should be created without panicking
    }

    #[test]
    fn test_is_relevant_event_kind() {
        use notify::event::{AccessKind, CreateKind, ModifyKind, RemoveKind};

        // Access events should NOT be relevant
        assert!(!is_relevant_event_kind(&notify::EventKind::Access(
            AccessKind::Read
        )));

        // Create events SHOULD be relevant
        assert!(is_relevant_event_kind(&notify::EventKind::Create(
            CreateKind::File
        )));

        // Modify events SHOULD be relevant
        assert!(is_relevant_event_kind(&notify::EventKind::Modify(
            ModifyKind::Any
        )));

        // Remove events SHOULD be relevant
        assert!(is_relevant_event_kind(&notify::EventKind::Remove(
            RemoveKind::File
        )));

        // Other/Any events SHOULD be relevant (includes renames)
        assert!(is_relevant_event_kind(&notify::EventKind::Other));
        assert!(is_relevant_event_kind(&notify::EventKind::Any));
    }

    #[test]
    fn test_next_deadline_empty() {
        let pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        let debounce = Duration::from_millis(500);

        assert!(next_deadline(&pending, debounce).is_none());
    }

    #[test]
    fn test_next_deadline_single() {
        let mut pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        let now = Instant::now();
        let debounce = Duration::from_millis(500);

        pending.insert(
            PathBuf::from("/test/script.ts"),
            (
                ScriptReloadEvent::FileChanged(PathBuf::from("/test/script.ts")),
                now,
            ),
        );

        let deadline = next_deadline(&pending, debounce);
        assert!(deadline.is_some());
        // Deadline should be approximately now + debounce
        let expected = now + debounce;
        let actual = deadline.unwrap();
        // Allow 1ms tolerance for timing
        assert!(actual >= expected && actual <= expected + Duration::from_millis(1));
    }

    #[test]
    fn test_next_deadline_multiple_picks_earliest() {
        let mut pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        let now = Instant::now();
        let debounce = Duration::from_millis(500);

        // Add an older event
        let older_time = now - Duration::from_millis(200);
        pending.insert(
            PathBuf::from("/test/old.ts"),
            (
                ScriptReloadEvent::FileChanged(PathBuf::from("/test/old.ts")),
                older_time,
            ),
        );

        // Add a newer event
        pending.insert(
            PathBuf::from("/test/new.ts"),
            (
                ScriptReloadEvent::FileChanged(PathBuf::from("/test/new.ts")),
                now,
            ),
        );

        let deadline = next_deadline(&pending, debounce);
        assert!(deadline.is_some());
        // Should pick the older event's deadline (earlier)
        let expected = older_time + debounce;
        let actual = deadline.unwrap();
        assert!(actual >= expected && actual <= expected + Duration::from_millis(1));
    }

    #[test]
    fn test_flush_expired_none_expired() {
        let (tx, _rx) = channel::<ScriptReloadEvent>();
        let mut pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        let now = Instant::now();
        let debounce = Duration::from_millis(500);

        // Add a fresh event (not expired)
        pending.insert(
            PathBuf::from("/test/script.ts"),
            (
                ScriptReloadEvent::FileChanged(PathBuf::from("/test/script.ts")),
                now,
            ),
        );

        flush_expired(&mut pending, debounce, &tx);

        // Event should still be pending
        assert_eq!(pending.len(), 1);
    }

    #[test]
    fn test_flush_expired_some_expired() {
        let (tx, rx) = channel::<ScriptReloadEvent>();
        let mut pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        let debounce = Duration::from_millis(500);

        // Add an expired event (from 600ms ago)
        let old_time = Instant::now() - Duration::from_millis(600);
        pending.insert(
            PathBuf::from("/test/old.ts"),
            (
                ScriptReloadEvent::FileChanged(PathBuf::from("/test/old.ts")),
                old_time,
            ),
        );

        // Add a fresh event
        pending.insert(
            PathBuf::from("/test/new.ts"),
            (
                ScriptReloadEvent::FileChanged(PathBuf::from("/test/new.ts")),
                Instant::now(),
            ),
        );

        flush_expired(&mut pending, debounce, &tx);

        // Only fresh event should remain
        assert_eq!(pending.len(), 1);
        assert!(pending.contains_key(&PathBuf::from("/test/new.ts")));

        // Should have received the expired event
        let received = rx.try_recv().unwrap();
        assert_eq!(
            received,
            ScriptReloadEvent::FileChanged(PathBuf::from("/test/old.ts"))
        );
    }

    #[test]
    fn test_config_watcher_shutdown_no_hang() {
        // Create and start a watcher
        let (mut watcher, _rx) = ConfigWatcher::new();

        // This may fail if the watch directory doesn't exist, but that's fine
        // We're testing that drop doesn't hang, not that watching works
        let _ = watcher.start();

        // Drop should complete within a reasonable time (not hang)
        // The Drop implementation sends Stop and then joins
        drop(watcher);

        // If we get here, the test passed (didn't hang)
    }

    #[test]
    fn test_theme_watcher_shutdown_no_hang() {
        let (mut watcher, _rx) = ThemeWatcher::new();
        let _ = watcher.start();
        drop(watcher);
        // If we get here, the test passed (didn't hang)
    }

    #[test]
    fn test_script_watcher_shutdown_no_hang() {
        let (mut watcher, _rx) = ScriptWatcher::new();
        let _ = watcher.start();
        drop(watcher);
        // If we get here, the test passed (didn't hang)
    }

    #[test]
    fn test_appearance_watcher_shutdown_no_hang() {
        let (mut watcher, _rx) = AppearanceWatcher::new();
        let _ = watcher.start();
        drop(watcher);
        // If we get here, the test passed (didn't hang)
    }

    #[test]
    fn test_storm_threshold_constant() {
        // Verify storm threshold is a reasonable value (compile-time checks)
        const { assert!(STORM_THRESHOLD > 0) };
        const { assert!(STORM_THRESHOLD <= 1000) }; // Not too high
    }

    #[test]
    fn test_debounce_constant() {
        // Verify debounce is a reasonable value (compile-time checks)
        const { assert!(DEBOUNCE_MS >= 100) }; // At least 100ms
        const { assert!(DEBOUNCE_MS <= 2000) }; // At most 2s
    }
}
