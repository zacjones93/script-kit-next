# Expert Bundle #89: Settings UI Patterns

## Overview

Settings UI allows users to customize Script Kit's behavior, appearance, and integrations. Good settings design is organized, discoverable, and provides clear feedback. Changes should be immediate or clearly require confirmation.

## Architecture

### Settings Structure

```rust
// src/settings.rs
use gpui::*;
use serde::{Deserialize, Serialize};

/// All user settings
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Settings {
    pub general: GeneralSettings,
    pub appearance: AppearanceSettings,
    pub keyboard: KeyboardSettings,
    pub editor: EditorSettings,
    pub integrations: IntegrationSettings,
    pub advanced: AdvancedSettings,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GeneralSettings {
    pub start_at_login: bool,
    pub check_for_updates: bool,
    pub send_analytics: bool,
    pub default_script_folder: Option<PathBuf>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AppearanceSettings {
    pub theme: ThemeChoice,
    pub vibrancy: bool,
    pub window_opacity: f32,
    pub font_size: u32,
    pub show_icons: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum ThemeChoice {
    #[default]
    System,
    Light,
    Dark,
    Custom(String),
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct KeyboardSettings {
    pub global_hotkey: Option<Shortcut>,
    pub vim_mode: bool,
    pub custom_shortcuts: HashMap<String, Shortcut>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EditorSettings {
    pub default_editor: String,
    pub editor_font_family: String,
    pub editor_font_size: u32,
    pub tab_size: u8,
    pub word_wrap: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct IntegrationSettings {
    pub github_token: Option<String>,
    pub openai_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AdvancedSettings {
    pub debug_mode: bool,
    pub log_level: LogLevel,
    pub bun_path: Option<PathBuf>,
    pub experimental_features: bool,
}
```

### Settings Categories

```rust
// src/settings/categories.rs

/// Settings category for navigation
#[derive(Clone)]
pub struct SettingsCategory {
    pub id: String,
    pub label: SharedString,
    pub icon: SharedString,
    pub description: Option<SharedString>,
}

pub fn settings_categories() -> Vec<SettingsCategory> {
    vec![
        SettingsCategory {
            id: "general".into(),
            label: "General".into(),
            icon: "settings".into(),
            description: Some("Startup, updates, and defaults".into()),
        },
        SettingsCategory {
            id: "appearance".into(),
            label: "Appearance".into(),
            icon: "palette".into(),
            description: Some("Theme, fonts, and visual options".into()),
        },
        SettingsCategory {
            id: "keyboard".into(),
            label: "Keyboard".into(),
            icon: "keyboard".into(),
            description: Some("Hotkeys and shortcuts".into()),
        },
        SettingsCategory {
            id: "editor".into(),
            label: "Editor".into(),
            icon: "code".into(),
            description: Some("Script editing preferences".into()),
        },
        SettingsCategory {
            id: "integrations".into(),
            label: "Integrations".into(),
            icon: "plug".into(),
            description: Some("API keys and external services".into()),
        },
        SettingsCategory {
            id: "advanced".into(),
            label: "Advanced".into(),
            icon: "wrench".into(),
            description: Some("Developer options and debugging".into()),
        },
    ]
}
```

### Settings Components

```rust
// src/components/settings.rs
use crate::theme::Theme;
use gpui::*;

/// Settings page layout
pub struct SettingsPanel {
    settings: Settings,
    categories: Vec<SettingsCategory>,
    selected_category: String,
    theme: Arc<Theme>,
    has_changes: bool,
}

impl Render for SettingsPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        div()
            .w_full()
            .h_full()
            .flex()
            .flex_row()
            // Sidebar navigation
            .child(
                div()
                    .w(px(220.0))
                    .h_full()
                    .border_r_1()
                    .border_color(rgb(colors.ui.border))
                    .flex()
                    .flex_col()
                    .py_4()
                    // Header
                    .child(
                        div()
                            .px_4()
                            .pb_4()
                            .child(
                                div()
                                    .text_size(px(18.0))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(colors.text.primary))
                                    .child("Settings")
                            )
                    )
                    // Categories
                    .child(
                        div()
                            .flex_1()
                            .overflow_y_auto()
                            .children(self.categories.iter().map(|cat| {
                                self.render_category_item(cat, cx)
                            }))
                    )
            )
            // Content area
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .overflow_y_auto()
                    .p_6()
                    .child(self.render_category_content(cx))
            )
    }
}

impl SettingsPanel {
    fn render_category_item(&self, category: &SettingsCategory, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        let is_selected = self.selected_category == category.id;
        let cat_id = category.id.clone();
        
        div()
            .mx_2()
            .px_3()
            .py_2()
            .rounded_md()
            .cursor_pointer()
            .bg(rgb(if is_selected { colors.ui.selected } else { 0x00000000 }))
            .hover(|s| s.bg(rgb(colors.ui.hover)))
            .on_click(cx.listener(move |this, _, cx| {
                this.selected_category = cat_id.clone();
                cx.notify();
            }))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(
                        Icon::new(&category.icon)
                            .size(px(16.0))
                            .color(rgb(if is_selected {
                                colors.accent.primary
                            } else {
                                colors.text.secondary
                            }))
                    )
                    .child(
                        div()
                            .text_size(px(14.0))
                            .text_color(rgb(if is_selected {
                                colors.text.primary
                            } else {
                                colors.text.secondary
                            }))
                            .child(category.label.clone())
                    )
            )
    }
    
    fn render_category_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        match self.selected_category.as_str() {
            "general" => self.render_general_settings(cx).into_any_element(),
            "appearance" => self.render_appearance_settings(cx).into_any_element(),
            "keyboard" => self.render_keyboard_settings(cx).into_any_element(),
            "editor" => self.render_editor_settings(cx).into_any_element(),
            "integrations" => self.render_integrations_settings(cx).into_any_element(),
            "advanced" => self.render_advanced_settings(cx).into_any_element(),
            _ => div().into_any_element(),
        }
    }
}
```

### Setting Controls

```rust
// src/components/setting_controls.rs

/// Toggle switch setting
pub struct SettingToggle {
    label: SharedString,
    description: Option<SharedString>,
    value: bool,
    theme: Arc<Theme>,
}

impl Render for SettingToggle {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        div()
            .py_3()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            // Label
            .child(
                div()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .text_size(px(14.0))
                            .text_color(rgb(colors.text.primary))
                            .child(self.label.clone())
                    )
                    .when_some(self.description.clone(), |el, desc| {
                        el.child(
                            div()
                                .text_size(px(12.0))
                                .text_color(rgb(colors.text.muted))
                                .child(desc)
                        )
                    })
            )
            // Toggle
            .child(
                div()
                    .w(px(44.0))
                    .h(px(24.0))
                    .rounded_full()
                    .cursor_pointer()
                    .bg(rgb(if self.value { colors.accent.primary } else { colors.ui.border }))
                    .p(px(2.0))
                    .on_click(cx.listener(|this, _, cx| {
                        this.value = !this.value;
                        cx.emit(SettingChanged(this.value));
                        cx.notify();
                    }))
                    .child(
                        div()
                            .w(px(20.0))
                            .h(px(20.0))
                            .rounded_full()
                            .bg(rgb(0xFFFFFF))
                            .translate_x(px(if self.value { 20.0 } else { 0.0 }))
                    )
            )
    }
}

/// Dropdown select setting
pub struct SettingSelect<T: Clone + PartialEq> {
    label: SharedString,
    description: Option<SharedString>,
    value: T,
    options: Vec<(T, SharedString)>,
    theme: Arc<Theme>,
    expanded: bool,
}

impl<T: Clone + PartialEq + 'static> Render for SettingSelect<T> {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        let current_label = self.options.iter()
            .find(|(v, _)| v == &self.value)
            .map(|(_, l)| l.clone())
            .unwrap_or_default();
        
        div()
            .py_3()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            // Label
            .child(
                div()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .text_size(px(14.0))
                            .text_color(rgb(colors.text.primary))
                            .child(self.label.clone())
                    )
                    .when_some(self.description.clone(), |el, desc| {
                        el.child(
                            div()
                                .text_size(px(12.0))
                                .text_color(rgb(colors.text.muted))
                                .child(desc)
                        )
                    })
            )
            // Dropdown
            .child(
                div()
                    .relative()
                    .child(
                        div()
                            .px_3()
                            .py_2()
                            .min_w(px(150.0))
                            .rounded_md()
                            .bg(rgb(colors.ui.input))
                            .border_1()
                            .border_color(rgb(colors.ui.border))
                            .cursor_pointer()
                            .flex()
                            .items_center()
                            .justify_between()
                            .on_click(cx.listener(|this, _, cx| {
                                this.expanded = !this.expanded;
                                cx.notify();
                            }))
                            .child(
                                div()
                                    .text_size(px(13.0))
                                    .text_color(rgb(colors.text.primary))
                                    .child(current_label)
                            )
                            .child(
                                Icon::new("chevron-down")
                                    .size(px(14.0))
                                    .color(rgb(colors.text.muted))
                            )
                    )
                    // Dropdown options
                    .when(self.expanded, |el| {
                        el.child(
                            div()
                                .absolute()
                                .top_full()
                                .left_0()
                                .right_0()
                                .mt_1()
                                .py_1()
                                .rounded_md()
                                .bg(rgb(colors.ui.surface))
                                .border_1()
                                .border_color(rgb(colors.ui.border))
                                .shadow_lg()
                                .z_index(100)
                                .children(self.options.iter().map(|(value, label)| {
                                    let is_selected = value == &self.value;
                                    let value = value.clone();
                                    
                                    div()
                                        .px_3()
                                        .py_2()
                                        .cursor_pointer()
                                        .bg(rgb(if is_selected { colors.ui.selected } else { 0x00000000 }))
                                        .hover(|s| s.bg(rgb(colors.ui.hover)))
                                        .on_click(cx.listener(move |this, _, cx| {
                                            this.value = value.clone();
                                            this.expanded = false;
                                            cx.emit(SettingChanged(this.value.clone()));
                                            cx.notify();
                                        }))
                                        .child(
                                            div()
                                                .text_size(px(13.0))
                                                .text_color(rgb(colors.text.primary))
                                                .child(label.clone())
                                        )
                                }))
                        )
                    })
            )
    }
}

/// Text input setting
pub struct SettingInput {
    label: SharedString,
    description: Option<SharedString>,
    value: String,
    placeholder: Option<SharedString>,
    input_type: InputType,
    theme: Arc<Theme>,
}

#[derive(Clone, Copy)]
pub enum InputType {
    Text,
    Password,
    Number,
    Path,
}

/// Slider setting
pub struct SettingSlider {
    label: SharedString,
    description: Option<SharedString>,
    value: f32,
    min: f32,
    max: f32,
    step: f32,
    show_value: bool,
    theme: Arc<Theme>,
}
```

## Layout Patterns

### Section Groups

```rust
// Group related settings
impl SettingsPanel {
    fn render_general_settings(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        div()
            .flex()
            .flex_col()
            .gap_6()
            // Section: Startup
            .child(
                SettingsSection::new("Startup", self.theme.clone())
                    .child(SettingToggle::new(
                        "Launch at login",
                        Some("Start Script Kit when you log in"),
                        self.settings.general.start_at_login,
                        self.theme.clone(),
                    ))
                    .child(SettingToggle::new(
                        "Check for updates",
                        Some("Automatically check for new versions"),
                        self.settings.general.check_for_updates,
                        self.theme.clone(),
                    ))
            )
            // Section: Privacy
            .child(
                SettingsSection::new("Privacy", self.theme.clone())
                    .child(SettingToggle::new(
                        "Send anonymous usage data",
                        Some("Help improve Script Kit by sharing anonymous analytics"),
                        self.settings.general.send_analytics,
                        self.theme.clone(),
                    ))
            )
            // Section: Paths
            .child(
                SettingsSection::new("Paths", self.theme.clone())
                    .child(SettingInput::new(
                        "Script folder",
                        Some("Default location for new scripts"),
                        self.settings.general.default_script_folder
                            .as_ref()
                            .map(|p| p.display().to_string())
                            .unwrap_or_default(),
                        InputType::Path,
                        self.theme.clone(),
                    ))
            )
    }
}

pub struct SettingsSection {
    title: SharedString,
    description: Option<SharedString>,
    children: Vec<AnyElement>,
    theme: Arc<Theme>,
}

impl SettingsSection {
    pub fn new(title: impl Into<SharedString>, theme: Arc<Theme>) -> Self {
        Self {
            title: title.into(),
            description: None,
            children: vec![],
            theme,
        }
    }
    
    pub fn description(mut self, desc: impl Into<SharedString>) -> Self {
        self.description = Some(desc.into());
        self
    }
    
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }
}

impl Render for SettingsSection {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        div()
            .flex()
            .flex_col()
            // Section header
            .child(
                div()
                    .pb_2()
                    .mb_2()
                    .border_b_1()
                    .border_color(rgb(colors.ui.border))
                    .child(
                        div()
                            .text_size(px(13.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(colors.text.muted))
                            .text_transform(TextTransform::Uppercase)
                            .child(self.title.clone())
                    )
                    .when_some(self.description.clone(), |el, desc| {
                        el.child(
                            div()
                                .text_size(px(12.0))
                                .text_color(rgb(colors.text.muted))
                                .mt_1()
                                .child(desc)
                        )
                    })
            )
            // Settings
            .children(std::mem::take(&mut self.children))
    }
}
```

## Testing

### Settings UI Test Script

```typescript
// tests/smoke/test-settings-ui.ts
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Test 1: Settings sidebar navigation
await div(`
  <div class="w-[700px] h-[500px] flex bg-zinc-900 rounded-lg overflow-hidden">
    <!-- Sidebar -->
    <div class="w-52 border-r border-zinc-700 py-4">
      <div class="px-4 pb-4">
        <h1 class="text-lg font-bold text-white">Settings</h1>
      </div>
      <div class="space-y-1">
        <div class="mx-2 px-3 py-2 rounded-md bg-zinc-800 flex items-center gap-3">
          <span class="text-amber-500">⚙️</span>
          <span class="text-sm text-white">General</span>
        </div>
        <div class="mx-2 px-3 py-2 rounded-md flex items-center gap-3 hover:bg-zinc-800 cursor-pointer">
          <span class="text-zinc-500">🎨</span>
          <span class="text-sm text-zinc-400">Appearance</span>
        </div>
        <div class="mx-2 px-3 py-2 rounded-md flex items-center gap-3 hover:bg-zinc-800 cursor-pointer">
          <span class="text-zinc-500">⌨️</span>
          <span class="text-sm text-zinc-400">Keyboard</span>
        </div>
        <div class="mx-2 px-3 py-2 rounded-md flex items-center gap-3 hover:bg-zinc-800 cursor-pointer">
          <span class="text-zinc-500">📝</span>
          <span class="text-sm text-zinc-400">Editor</span>
        </div>
        <div class="mx-2 px-3 py-2 rounded-md flex items-center gap-3 hover:bg-zinc-800 cursor-pointer">
          <span class="text-zinc-500">🔌</span>
          <span class="text-sm text-zinc-400">Integrations</span>
        </div>
      </div>
    </div>
    <!-- Content -->
    <div class="flex-1 p-6 overflow-y-auto">
      <div class="text-lg font-semibold text-white mb-4">General</div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot1 = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
writeFileSync(join(dir, 'settings-sidebar.png'), Buffer.from(shot1.data, 'base64'));

// Test 2: Toggle settings
await div(`
  <div class="p-6 w-96 bg-zinc-900 rounded-lg">
    <div class="space-y-4">
      <!-- Toggle ON -->
      <div class="py-3 flex items-center justify-between">
        <div>
          <div class="text-sm text-white">Launch at login</div>
          <div class="text-xs text-zinc-500">Start Script Kit when you log in</div>
        </div>
        <div class="w-11 h-6 rounded-full bg-amber-500 p-0.5 cursor-pointer">
          <div class="w-5 h-5 rounded-full bg-white translate-x-5"></div>
        </div>
      </div>
      
      <!-- Toggle OFF -->
      <div class="py-3 flex items-center justify-between">
        <div>
          <div class="text-sm text-white">Check for updates</div>
          <div class="text-xs text-zinc-500">Automatically check for new versions</div>
        </div>
        <div class="w-11 h-6 rounded-full bg-zinc-600 p-0.5 cursor-pointer">
          <div class="w-5 h-5 rounded-full bg-white"></div>
        </div>
      </div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot2 = await captureScreenshot();
writeFileSync(join(dir, 'settings-toggles.png'), Buffer.from(shot2.data, 'base64'));

// Test 3: Dropdown select
await div(`
  <div class="p-6 w-96 bg-zinc-900 rounded-lg">
    <div class="py-3 flex items-center justify-between">
      <div>
        <div class="text-sm text-white">Theme</div>
        <div class="text-xs text-zinc-500">Choose your preferred color scheme</div>
      </div>
      <div class="relative">
        <div class="px-3 py-2 min-w-[150px] rounded-md bg-zinc-800 border border-zinc-700 flex items-center justify-between cursor-pointer">
          <span class="text-sm text-white">Dark</span>
          <span class="text-zinc-500 text-xs">▼</span>
        </div>
        <div class="absolute top-full left-0 right-0 mt-1 py-1 rounded-md bg-zinc-800 border border-zinc-700 shadow-lg">
          <div class="px-3 py-2 hover:bg-zinc-700 cursor-pointer text-sm text-white">System</div>
          <div class="px-3 py-2 bg-zinc-700 cursor-pointer text-sm text-white">Dark</div>
          <div class="px-3 py-2 hover:bg-zinc-700 cursor-pointer text-sm text-white">Light</div>
        </div>
      </div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot3 = await captureScreenshot();
writeFileSync(join(dir, 'settings-dropdown.png'), Buffer.from(shot3.data, 'base64'));

// Test 4: Settings section groups
await div(`
  <div class="p-6 w-[500px] bg-zinc-900 rounded-lg">
    <!-- Section: Startup -->
    <div class="mb-6">
      <div class="pb-2 mb-3 border-b border-zinc-700">
        <div class="text-xs font-semibold text-zinc-500 uppercase">Startup</div>
      </div>
      <div class="space-y-3">
        <div class="flex items-center justify-between">
          <span class="text-sm text-white">Launch at login</span>
          <div class="w-10 h-5 rounded-full bg-amber-500 p-0.5">
            <div class="w-4 h-4 rounded-full bg-white translate-x-5"></div>
          </div>
        </div>
        <div class="flex items-center justify-between">
          <span class="text-sm text-white">Check for updates</span>
          <div class="w-10 h-5 rounded-full bg-amber-500 p-0.5">
            <div class="w-4 h-4 rounded-full bg-white translate-x-5"></div>
          </div>
        </div>
      </div>
    </div>
    
    <!-- Section: Privacy -->
    <div>
      <div class="pb-2 mb-3 border-b border-zinc-700">
        <div class="text-xs font-semibold text-zinc-500 uppercase">Privacy</div>
      </div>
      <div class="flex items-center justify-between">
        <div>
          <div class="text-sm text-white">Send analytics</div>
          <div class="text-xs text-zinc-500">Help improve Script Kit</div>
        </div>
        <div class="w-10 h-5 rounded-full bg-zinc-600 p-0.5">
          <div class="w-4 h-4 rounded-full bg-white"></div>
        </div>
      </div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot4 = await captureScreenshot();
writeFileSync(join(dir, 'settings-sections.png'), Buffer.from(shot4.data, 'base64'));

console.error('[SETTINGS UI] Test screenshots saved');
process.exit(0);
```

## Related Bundles

- Bundle #63: Config System - Config persistence
- Bundle #60: Form & Input Handling - Form controls
- Bundle #86: Responsive Layouts - Responsive settings panels
