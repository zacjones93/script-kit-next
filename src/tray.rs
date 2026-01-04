//! System tray icon management for Script Kit
//!
//! Provides a TrayManager that creates a macOS menu bar icon with a context menu.
//! The icon uses the Script Kit logo rendered as a template image for proper
//! light/dark mode adaptation.

use anyhow::{Context, Result};
use tray_icon::{
    menu::{
        CheckMenuItem, IconMenuItem, Menu, MenuEvent, MenuEventReceiver, MenuItem, NativeIcon,
        PredefinedMenuItem,
    },
    Icon, TrayIcon, TrayIconBuilder,
};

use crate::login_item;

/// SVG logo for Script Kit (32x32, monochrome)
/// This will be rendered as a template image on macOS for light/dark mode adaptation
const LOGO_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" fill="currentColor" viewBox="0 0 32 32">
  <path fill="currentColor" d="M14 25a2 2 0 0 1 2-2h14a2 2 0 1 1 0 4H16a2 2 0 0 1-2-2ZM0 7.381c0-1.796 1.983-2.884 3.498-1.92l13.728 8.736c1.406.895 1.406 2.946 0 3.84L3.498 26.775C1.983 27.738 0 26.649 0 24.854V7.38Z"/>
</svg>"#;

/// Menu item identifiers for matching events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayMenuAction {
    OpenScriptKit,
    OpenNotes,
    OpenAiChat,
    OpenOnGitHub,
    OpenManual,
    JoinCommunity,
    FollowUs,
    Settings,
    LaunchAtLogin,
    Quit,
}

/// Manages the system tray icon and menu
pub struct TrayManager {
    #[allow(dead_code)]
    tray_icon: TrayIcon,
    open_script_kit_id: String,
    open_notes_id: String,
    open_ai_chat_id: String,
    open_on_github_id: String,
    open_manual_id: String,
    join_community_id: String,
    follow_us_id: String,
    settings_id: String,
    launch_at_login_item: CheckMenuItem,
    #[allow(dead_code)]
    version_id: String,
    quit_id: String,
}

impl TrayManager {
    /// Creates a new TrayManager with the Script Kit logo and menu
    ///
    /// # Errors
    /// Returns an error if:
    /// - SVG parsing fails
    /// - PNG rendering fails
    /// - Tray icon creation fails
    pub fn new() -> Result<Self> {
        let icon = Self::create_icon_from_svg()?;
        let (
            menu,
            open_id,
            open_notes_id,
            open_ai_chat_id,
            open_on_github_id,
            open_manual_id,
            join_community_id,
            follow_us_id,
            settings_id,
            launch_at_login_item,
            version_id,
            quit_id,
        ) = Self::create_menu()?;

        let tray_icon = TrayIconBuilder::new()
            .with_icon(icon)
            .with_tooltip("Script Kit")
            .with_menu(Box::new(menu))
            .with_icon_as_template(true) // macOS: adapt to light/dark menu bar
            .build()
            .context("Failed to create tray icon")?;

        Ok(Self {
            tray_icon,
            open_script_kit_id: open_id,
            open_notes_id,
            open_ai_chat_id,
            open_on_github_id,
            open_manual_id,
            join_community_id,
            follow_us_id,
            settings_id,
            launch_at_login_item,
            version_id,
            quit_id,
        })
    }

    /// Converts the embedded SVG logo to a PNG icon
    fn create_icon_from_svg() -> Result<Icon> {
        // Parse SVG
        let opts = usvg::Options::default();
        let tree = usvg::Tree::from_str(LOGO_SVG, &opts).context("Failed to parse SVG")?;

        // Create pixmap for rendering (32x32)
        let size = tree.size();
        let width = size.width() as u32;
        let height = size.height() as u32;

        let mut pixmap =
            tiny_skia::Pixmap::new(width, height).context("Failed to create pixmap")?;

        // Render SVG to pixmap (white color for template image)
        // Fill with white since template images on macOS use the alpha channel
        // and the system will colorize based on menu bar appearance
        resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

        // Convert to RGBA bytes
        let rgba = pixmap.take();

        // Create tray icon from RGBA data
        Icon::from_rgba(rgba, width, height).context("Failed to create icon from RGBA data")
    }

    /// Creates the tray menu with standard items
    ///
    /// Menu structure (Raycast-style):
    /// 1. Open Script Kit
    /// 2. ---
    /// 3. Open Notes
    /// 4. Open AI Chat
    /// 5. ---
    /// 6. Open on GitHub
    /// 7. Manual
    /// 8. Join Community
    /// 9. Follow Us
    /// 10. ---
    /// 11. Settings
    /// 12. ---
    /// 13. Launch at Login (checkmark)
    /// 14. Version 0.1.0 (disabled)
    /// 15. ---
    /// 16. Quit Script Kit
    #[allow(clippy::type_complexity)]
    fn create_menu() -> Result<(
        Menu,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        CheckMenuItem,
        String,
        String,
    )> {
        let menu = Menu::new();

        // Create menu items with native icons (macOS only)
        // Native icons are ignored on Windows/Linux but the menu items still work
        let open_item =
            IconMenuItem::with_native_icon("Open Script Kit", true, Some(NativeIcon::Home), None);
        let open_notes_item =
            IconMenuItem::with_native_icon("Open Notes", true, Some(NativeIcon::Bookmarks), None);
        let open_ai_chat_item = IconMenuItem::with_native_icon(
            "Open AI Chat",
            true,
            Some(NativeIcon::IChatTheater),
            None,
        );

        // External links
        let open_on_github_item =
            IconMenuItem::with_native_icon("Open on GitHub", true, Some(NativeIcon::Network), None);
        let open_manual_item =
            IconMenuItem::with_native_icon("Manual", true, Some(NativeIcon::Info), None);
        let join_community_item = IconMenuItem::with_native_icon(
            "Join Community",
            true,
            Some(NativeIcon::UserGroup),
            None,
        );
        let follow_us_item =
            IconMenuItem::with_native_icon("Follow Us", true, Some(NativeIcon::Share), None);

        // Settings
        let settings_item = IconMenuItem::with_native_icon(
            "Settings",
            true,
            Some(NativeIcon::PreferencesGeneral),
            None,
        );

        // Create check menu item for Launch at Login with current state
        let launch_at_login_item = CheckMenuItem::new(
            "Launch at Login",
            true, // enabled
            login_item::is_login_item_enabled(),
            None, // no accelerator
        );

        // Version display (disabled, informational only)
        let version_item = MenuItem::new("Version 0.1.0", false, None);

        let quit_item =
            IconMenuItem::with_native_icon("Quit Script Kit", true, Some(NativeIcon::Remove), None);

        // Store IDs for event matching
        let open_id = open_item.id().0.clone();
        let open_notes_id = open_notes_item.id().0.clone();
        let open_ai_chat_id = open_ai_chat_item.id().0.clone();
        let open_on_github_id = open_on_github_item.id().0.clone();
        let open_manual_id = open_manual_item.id().0.clone();
        let join_community_id = join_community_item.id().0.clone();
        let follow_us_id = follow_us_item.id().0.clone();
        let settings_id = settings_item.id().0.clone();
        let version_id = version_item.id().0.clone();
        let quit_id = quit_item.id().0.clone();

        // Add items to menu in Raycast-style order
        // Section 1: Main action
        menu.append(&open_item).context("Failed to add Open item")?;
        menu.append(&PredefinedMenuItem::separator())
            .context("Failed to add separator")?;

        // Section 2: App features
        menu.append(&open_notes_item)
            .context("Failed to add Open Notes item")?;
        menu.append(&open_ai_chat_item)
            .context("Failed to add Open AI Chat item")?;
        menu.append(&PredefinedMenuItem::separator())
            .context("Failed to add separator")?;

        // Section 3: External links
        menu.append(&open_on_github_item)
            .context("Failed to add Open on GitHub item")?;
        menu.append(&open_manual_item)
            .context("Failed to add Manual item")?;
        menu.append(&join_community_item)
            .context("Failed to add Join Community item")?;
        menu.append(&follow_us_item)
            .context("Failed to add Follow Us item")?;
        menu.append(&PredefinedMenuItem::separator())
            .context("Failed to add separator")?;

        // Section 4: Settings
        menu.append(&settings_item)
            .context("Failed to add Settings item")?;
        menu.append(&PredefinedMenuItem::separator())
            .context("Failed to add separator")?;

        // Section 5: App state
        menu.append(&launch_at_login_item)
            .context("Failed to add Launch at Login item")?;
        menu.append(&version_item)
            .context("Failed to add Version item")?;
        menu.append(&PredefinedMenuItem::separator())
            .context("Failed to add separator")?;

        // Section 6: Quit
        menu.append(&quit_item).context("Failed to add Quit item")?;

        Ok((
            menu,
            open_id,
            open_notes_id,
            open_ai_chat_id,
            open_on_github_id,
            open_manual_id,
            join_community_id,
            follow_us_id,
            settings_id,
            launch_at_login_item,
            version_id,
            quit_id,
        ))
    }

    /// Returns the menu event receiver for handling menu clicks
    pub fn menu_event_receiver(&self) -> &MenuEventReceiver {
        MenuEvent::receiver()
    }

    /// Matches a menu event to a TrayMenuAction
    ///
    /// Returns `Some(action)` if the event matches a known menu item,
    /// or `None` if the event is from an unknown source.
    pub fn match_menu_event(&self, event: &MenuEvent) -> Option<TrayMenuAction> {
        let id = &event.id.0;
        if id == &self.open_script_kit_id {
            Some(TrayMenuAction::OpenScriptKit)
        } else if id == &self.open_notes_id {
            Some(TrayMenuAction::OpenNotes)
        } else if id == &self.open_ai_chat_id {
            Some(TrayMenuAction::OpenAiChat)
        } else if id == &self.open_on_github_id {
            Some(TrayMenuAction::OpenOnGitHub)
        } else if id == &self.open_manual_id {
            Some(TrayMenuAction::OpenManual)
        } else if id == &self.join_community_id {
            Some(TrayMenuAction::JoinCommunity)
        } else if id == &self.follow_us_id {
            Some(TrayMenuAction::FollowUs)
        } else if id == &self.settings_id {
            Some(TrayMenuAction::Settings)
        } else if id == &self.launch_at_login_item.id().0 {
            // Toggle login item and update checkmark
            if let Ok(new_state) = login_item::toggle_login_item() {
                self.launch_at_login_item.set_checked(new_state);
            }
            Some(TrayMenuAction::LaunchAtLogin)
        } else if id == &self.quit_id {
            Some(TrayMenuAction::Quit)
        } else {
            None
        }
    }

    /// Returns the menu item ID for "Open Script Kit"
    #[allow(dead_code)]
    pub fn open_script_kit_id(&self) -> &str {
        &self.open_script_kit_id
    }

    /// Returns the menu item ID for "Settings"
    #[allow(dead_code)]
    pub fn settings_id(&self) -> &str {
        &self.settings_id
    }

    /// Returns the menu item ID for "Quit"
    #[allow(dead_code)]
    pub fn quit_id(&self) -> &str {
        &self.quit_id
    }
}
