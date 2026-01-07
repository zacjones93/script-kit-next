# Expert Bundle #87: Accessibility Patterns

## Overview

Accessibility ensures Script Kit is usable by everyone, including users with visual, motor, or cognitive differences. This includes keyboard navigation, screen reader support, high contrast modes, and reduced motion preferences. Good accessibility is built-in, not bolted on.

## Architecture

### Accessibility State

```rust
// src/accessibility.rs
use gpui::*;

/// Global accessibility preferences
#[derive(Clone, Debug)]
pub struct AccessibilityPreferences {
    /// Reduce animations
    pub reduce_motion: bool,
    /// High contrast mode
    pub high_contrast: bool,
    /// Larger text
    pub larger_text: bool,
    /// Screen reader active
    pub screen_reader_active: bool,
    /// Keyboard-only navigation
    pub keyboard_only: bool,
}

impl AccessibilityPreferences {
    /// Detect system preferences
    pub fn from_system() -> Self {
        Self {
            reduce_motion: Self::detect_reduce_motion(),
            high_contrast: Self::detect_high_contrast(),
            larger_text: Self::detect_larger_text(),
            screen_reader_active: Self::detect_screen_reader(),
            keyboard_only: false,
        }
    }
    
    #[cfg(target_os = "macos")]
    fn detect_reduce_motion() -> bool {
        use cocoa::appkit::NSWorkspace;
        use cocoa::base::id;
        use objc::{msg_send, sel, sel_impl};
        
        unsafe {
            let workspace: id = NSWorkspace::sharedWorkspace();
            let reduce: bool = msg_send![workspace, accessibilityDisplayShouldReduceMotion];
            reduce
        }
    }
    
    #[cfg(not(target_os = "macos"))]
    fn detect_reduce_motion() -> bool {
        false
    }
    
    // Similar detection for other preferences...
}

/// Accessibility context for components
#[derive(Clone)]
pub struct A11yContext {
    pub preferences: AccessibilityPreferences,
    pub focus_visible: bool, // Show focus rings
    pub announced_messages: Vec<String>, // For screen readers
}
```

### ARIA-like Labels

```rust
// src/accessibility/labels.rs
use gpui::*;

/// Semantic role for accessibility
#[derive(Clone, Copy, Debug)]
pub enum A11yRole {
    Button,
    Link,
    Checkbox,
    Radio,
    Slider,
    TextInput,
    ListBox,
    ListItem,
    Menu,
    MenuItem,
    Dialog,
    Alert,
    Status,
    Tab,
    TabPanel,
    Tree,
    TreeItem,
    Grid,
    Row,
    Cell,
    Heading,
    Region,
    Main,
    Navigation,
    Search,
    Form,
}

/// Accessibility properties for an element
#[derive(Clone, Default)]
pub struct A11yProps {
    pub role: Option<A11yRole>,
    pub label: Option<SharedString>,
    pub description: Option<SharedString>,
    pub value: Option<SharedString>,
    pub expanded: Option<bool>,
    pub selected: Option<bool>,
    pub checked: Option<bool>,
    pub disabled: bool,
    pub hidden: bool,
    pub live: Option<LiveRegion>,
    pub level: Option<u8>, // For headings
    pub pos_in_set: Option<usize>,
    pub set_size: Option<usize>,
}

#[derive(Clone, Copy, Debug)]
pub enum LiveRegion {
    Off,
    Polite,
    Assertive,
}

/// Extension trait for adding accessibility props
pub trait A11yExt: Sized {
    fn a11y_label(self, label: impl Into<SharedString>) -> Self;
    fn a11y_role(self, role: A11yRole) -> Self;
    fn a11y_description(self, desc: impl Into<SharedString>) -> Self;
    fn a11y_expanded(self, expanded: bool) -> Self;
    fn a11y_selected(self, selected: bool) -> Self;
    fn a11y_checked(self, checked: bool) -> Self;
    fn a11y_disabled(self, disabled: bool) -> Self;
    fn a11y_hidden(self) -> Self;
    fn a11y_live(self, region: LiveRegion) -> Self;
}
```

### Focus Management

```rust
// src/accessibility/focus.rs
use gpui::*;

/// Focus ring styling
pub struct FocusRing {
    visible: bool,
    color: u32,
    width: Pixels,
    offset: Pixels,
}

impl FocusRing {
    pub fn new(theme: &Theme) -> Self {
        Self {
            visible: false,
            color: theme.colors.accent.primary,
            width: px(2.0),
            offset: px(2.0),
        }
    }
    
    pub fn show(&mut self) {
        self.visible = true;
    }
    
    pub fn hide(&mut self) {
        self.visible = false;
    }
}

/// Focus trap for dialogs
pub struct FocusTrap {
    container_id: String,
    first_focusable: Option<FocusHandle>,
    last_focusable: Option<FocusHandle>,
    return_focus: Option<FocusHandle>,
}

impl FocusTrap {
    pub fn new(container_id: impl Into<String>) -> Self {
        Self {
            container_id: container_id.into(),
            first_focusable: None,
            last_focusable: None,
            return_focus: None,
        }
    }
    
    /// Capture focus and store return target
    pub fn activate(&mut self, cx: &mut WindowContext) {
        // Store current focus for restoration
        self.return_focus = cx.focus_handle().is_focused(window).then(|| cx.focus_handle());
        
        // Focus first element
        if let Some(first) = &self.first_focusable {
            first.focus(window);
        }
    }
    
    /// Release trap and restore focus
    pub fn deactivate(&mut self, cx: &mut WindowContext) {
        if let Some(return_target) = self.return_focus.take() {
            return_target.focus(window);
        }
    }
    
    /// Handle Tab key to cycle focus
    pub fn handle_tab(&mut self, shift: bool, cx: &mut WindowContext) {
        let current = cx.focus_handle();
        
        if shift {
            // Shift+Tab: go backwards
            if current.id() == self.first_focusable.as_ref().map(|f| f.id()) {
                // Wrap to last
                if let Some(last) = &self.last_focusable {
                    last.focus(window);
                }
            }
        } else {
            // Tab: go forwards
            if current.id() == self.last_focusable.as_ref().map(|f| f.id()) {
                // Wrap to first
                if let Some(first) = &self.first_focusable {
                    first.focus(window);
                }
            }
        }
    }
}
```

## Keyboard Navigation

### Skip Links

```rust
// Allow keyboard users to skip to main content
impl MainMenu {
    fn render_skip_link(&self, cx: &mut WindowContext) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        // Only visible on focus
        div()
            .absolute()
            .top_0()
            .left_0()
            .z_index(1000)
            .opacity(0)
            .focus_visible(|el| {
                el.opacity(1.0)
                    .p_2()
                    .bg(rgb(colors.accent.primary))
                    .text_color(rgb(colors.background.main))
            })
            .child("Skip to main content")
            .on_click(cx.listener(|this, _, cx| {
                this.focus_main_content(cx);
            }))
    }
}
```

### Arrow Key Navigation

```rust
// Consistent arrow key navigation for lists
impl ScriptList {
    fn handle_arrow_keys(&mut self, key: &str, cx: &mut WindowContext) {
        match key {
            // Vertical lists
            "up" | "arrowup" => {
                self.move_selection(-1, cx);
            }
            "down" | "arrowdown" => {
                self.move_selection(1, cx);
            }
            // Page navigation
            "pageup" => {
                self.move_selection(-10, cx);
            }
            "pagedown" => {
                self.move_selection(10, cx);
            }
            // Boundaries
            "home" => {
                self.select_first(cx);
            }
            "end" => {
                self.select_last(cx);
            }
            _ => {}
        }
        
        // Announce selection to screen reader
        if let Some(selected) = self.get_selected() {
            self.announce(format!("{}, {} of {}", selected.name, self.selected_index + 1, self.items.len()), cx);
        }
    }
    
    fn announce(&self, message: String, cx: &mut WindowContext) {
        // Platform-specific screen reader announcement
        #[cfg(target_os = "macos")]
        {
            // Use NSAccessibility to announce
        }
    }
}
```

## Visual Accessibility

### High Contrast Mode

```rust
// src/accessibility/contrast.rs
impl Theme {
    /// Get high contrast version of colors
    pub fn high_contrast_colors(&self) -> ColorScheme {
        let colors = &self.colors;
        
        ColorScheme {
            background: BackgroundColors {
                main: 0x000000,      // Pure black
                secondary: 0x1A1A1A,
                tertiary: 0x333333,
            },
            text: TextColors {
                primary: 0xFFFFFF,   // Pure white
                secondary: 0xE5E5E5,
                muted: 0xA0A0A0,
                disabled: 0x666666,
            },
            accent: AccentColors {
                primary: 0xFFFF00,   // High visibility yellow
                hover: 0xFFFF66,
                active: 0xCCCC00,
            },
            ui: UIColors {
                border: 0xFFFFFF,    // Visible borders
                hover: 0x333333,
                selected: 0x444444,
                focus: 0xFFFF00,     // Bright focus ring
                ..colors.ui.clone()
            },
            semantic: SemanticColors {
                error: 0xFF6666,     // Brighter for visibility
                warning: 0xFFCC00,
                success: 0x66FF66,
                info: 0x66CCFF,
            },
        }
    }
}
```

### Focus Indicators

```rust
// Clear focus indicators for keyboard users
impl FocusableElement {
    fn render_with_focus(&self, cx: &mut WindowContext) -> impl IntoElement {
        let colors = &self.theme.colors;
        let is_focused = self.focus_handle.is_focused(window);
        let a11y = &cx.global::<A11yContext>();
        
        div()
            // Always show focus ring when using keyboard
            .when(is_focused && a11y.focus_visible, |el| {
                el.rounded_md()
                    .outline_2()
                    .outline_offset(px(2.0))
                    .outline_color(rgb(colors.accent.primary))
            })
            // High contrast: stronger focus ring
            .when(is_focused && a11y.preferences.high_contrast, |el| {
                el.outline_3()
                    .outline_color(rgb(0xFFFF00))
            })
            .child(/* content */)
    }
}
```

### Reduced Motion

```rust
// Respect reduce motion preference
impl Animation {
    fn with_reduced_motion(self, a11y: &A11yContext) -> Self {
        if a11y.preferences.reduce_motion {
            // No animation or instant transition
            self.duration(Duration::ZERO)
        } else {
            self
        }
    }
}

// Component example
impl Notification {
    fn render_animated(&self, cx: &mut WindowContext) -> impl IntoElement {
        let a11y = cx.global::<A11yContext>();
        
        div()
            .when(!a11y.preferences.reduce_motion, |el| {
                el.with_animation(
                    "notification-enter",
                    Animation::new()
                        .duration(Duration::from_millis(200))
                        .easing(ease_out_cubic),
                    |el, t| el.opacity(t).translate_y(px((1.0 - t) * 20.0)),
                )
            })
            .when(a11y.preferences.reduce_motion, |el| {
                // Instant appearance
                el.opacity(1.0)
            })
            .child(/* notification content */)
    }
}
```

## Screen Reader Support

### Live Regions

```rust
// Announce dynamic content changes
impl NotificationManager {
    fn show_notification(&mut self, notification: Notification, cx: &mut WindowContext) {
        self.notifications.push(notification.clone());
        
        // Announce to screen reader
        self.announce_live(
            &notification.message,
            match notification.level {
                NotificationLevel::Error => LiveRegion::Assertive,
                _ => LiveRegion::Polite,
            },
            cx,
        );
        
        cx.notify();
    }
    
    fn announce_live(&self, message: &str, region: LiveRegion, cx: &mut WindowContext) {
        // Platform-specific implementation
        #[cfg(target_os = "macos")]
        {
            use cocoa::appkit::NSAccessibilityPostNotification;
            // Post accessibility notification
        }
    }
}
```

### Descriptive Labels

```rust
// Provide meaningful labels for interactive elements
impl IconButton {
    fn render(&self, cx: &mut WindowContext) -> impl IntoElement {
        div()
            .a11y_role(A11yRole::Button)
            .a11y_label(&self.label) // "Close dialog" not just "X"
            .a11y_description(self.description.as_deref())
            .cursor_pointer()
            .child(Icon::new(&self.icon).size(px(16.0)))
    }
}

// List items with context
impl ScriptListItem {
    fn render(&self, index: usize, total: usize, cx: &mut WindowContext) -> impl IntoElement {
        div()
            .a11y_role(A11yRole::ListItem)
            .a11y_label(&self.script.name)
            .a11y_description(self.script.description.as_deref())
            .a11y_props(A11yProps {
                pos_in_set: Some(index + 1),
                set_size: Some(total),
                selected: Some(self.is_selected),
                ..Default::default()
            })
            .child(/* item content */)
    }
}
```

## Testing

### Accessibility Test Script

```typescript
// tests/smoke/test-accessibility.ts
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Test 1: Focus ring visibility
await div(`
  <div class="p-4 flex flex-col gap-4">
    <div class="text-xs text-zinc-500 mb-2">Focus states (keyboard navigation)</div>
    
    <!-- Unfocused button -->
    <button class="px-4 py-2 rounded-md bg-zinc-700 text-white">
      Unfocused Button
    </button>
    
    <!-- Focused button with ring -->
    <button class="px-4 py-2 rounded-md bg-zinc-700 text-white outline outline-2 outline-offset-2 outline-amber-500">
      Focused Button
    </button>
    
    <!-- High contrast focused -->
    <button class="px-4 py-2 rounded-md bg-black text-white border border-white outline outline-3 outline-offset-2 outline-yellow-400">
      High Contrast Focus
    </button>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot1 = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
writeFileSync(join(dir, 'a11y-focus-rings.png'), Buffer.from(shot1.data, 'base64'));

// Test 2: High contrast mode
await div(`
  <div class="flex gap-4 p-4">
    <!-- Normal contrast -->
    <div class="w-48 bg-zinc-800 rounded-lg p-4">
      <div class="text-xs text-zinc-500 mb-2">Normal</div>
      <div class="h-10 px-3 flex items-center bg-zinc-700 rounded-md text-white text-sm">
        List Item
      </div>
      <button class="mt-2 w-full py-2 rounded-md bg-amber-500 text-black text-sm font-medium">
        Action
      </button>
    </div>
    
    <!-- High contrast -->
    <div class="w-48 bg-black rounded-lg p-4 border border-white">
      <div class="text-xs text-gray-400 mb-2">High Contrast</div>
      <div class="h-10 px-3 flex items-center bg-zinc-900 border border-white rounded-md text-white text-sm">
        List Item
      </div>
      <button class="mt-2 w-full py-2 rounded-md bg-yellow-400 text-black text-sm font-bold">
        Action
      </button>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot2 = await captureScreenshot();
writeFileSync(join(dir, 'a11y-contrast.png'), Buffer.from(shot2.data, 'base64'));

// Test 3: Skip link
await div(`
  <div class="relative">
    <!-- Skip link (normally hidden, shown on focus) -->
    <a href="#main" class="absolute top-0 left-0 z-50 px-4 py-2 bg-amber-500 text-black font-medium text-sm">
      Skip to main content
    </a>
    
    <nav class="h-12 px-4 flex items-center gap-4 bg-zinc-800 border-b border-zinc-700">
      <span class="text-white text-sm">Navigation</span>
      <a href="#" class="text-zinc-400 text-sm">Link 1</a>
      <a href="#" class="text-zinc-400 text-sm">Link 2</a>
    </nav>
    
    <main id="main" class="p-4">
      <div class="text-white">Main Content</div>
    </main>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot3 = await captureScreenshot();
writeFileSync(join(dir, 'a11y-skip-link.png'), Buffer.from(shot3.data, 'base64'));

// Test 4: Larger text mode
await div(`
  <div class="flex gap-4 p-4">
    <!-- Normal text -->
    <div class="w-48">
      <div class="text-xs text-zinc-500 mb-2">Normal</div>
      <div class="h-[52px] px-4 flex items-center gap-3 bg-zinc-800 rounded-md">
        <span class="text-sm">📄</span>
        <div>
          <div class="text-sm text-white">Script Name</div>
          <div class="text-xs text-zinc-500">Description</div>
        </div>
      </div>
    </div>
    
    <!-- Larger text -->
    <div class="w-64">
      <div class="text-sm text-zinc-500 mb-2">Larger Text</div>
      <div class="h-16 px-4 flex items-center gap-4 bg-zinc-800 rounded-md">
        <span class="text-lg">📄</span>
        <div>
          <div class="text-base text-white">Script Name</div>
          <div class="text-sm text-zinc-500">Description</div>
        </div>
      </div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot4 = await captureScreenshot();
writeFileSync(join(dir, 'a11y-larger-text.png'), Buffer.from(shot4.data, 'base64'));

console.error('[ACCESSIBILITY] Test screenshots saved');
process.exit(0);
```

## Accessibility Checklist

```rust
/// Accessibility requirements checklist
pub mod a11y_checklist {
    // Keyboard
    // [ ] All interactive elements focusable with Tab
    // [ ] Focus order matches visual order
    // [ ] Focus visible at all times
    // [ ] Escape closes modals/menus
    // [ ] Arrow keys navigate lists
    // [ ] Enter/Space activates buttons
    
    // Screen readers
    // [ ] All images have alt text (or are decorative)
    // [ ] Form fields have labels
    // [ ] Buttons have accessible names
    // [ ] Dynamic content announced
    // [ ] Heading hierarchy logical
    
    // Visual
    // [ ] Color contrast >= 4.5:1 for text
    // [ ] Color contrast >= 3:1 for UI
    // [ ] Don't rely on color alone
    // [ ] Support high contrast mode
    // [ ] Support larger text
    
    // Motion
    // [ ] Respect prefers-reduced-motion
    // [ ] No flashing content
    // [ ] Animations can be paused
}
```

## Related Bundles

- Bundle #65: Focus Management - Focus system details
- Bundle #82: Keyboard Shortcuts UX - Keyboard navigation
- Bundle #10: Theme System - High contrast themes
