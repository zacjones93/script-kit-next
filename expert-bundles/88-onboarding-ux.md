# Expert Bundle #88: Onboarding UX

## Overview

Onboarding introduces new users to Script Kit's features and capabilities. Good onboarding is progressive, contextual, and respects user time. It helps users succeed without overwhelming them.

## Architecture

### Onboarding State

```rust
// src/onboarding.rs
use gpui::*;
use std::collections::HashSet;

/// Tracks user's onboarding progress
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct OnboardingState {
    /// Whether initial setup is complete
    pub setup_complete: bool,
    /// Completed onboarding steps
    pub completed_steps: HashSet<String>,
    /// Dismissed hints
    pub dismissed_hints: HashSet<String>,
    /// Feature discovery progress
    pub discovered_features: HashSet<String>,
    /// Number of scripts created
    pub scripts_created: u32,
    /// Number of scripts run
    pub scripts_run: u32,
    /// First run timestamp
    pub first_run: Option<chrono::DateTime<chrono::Utc>>,
}

impl OnboardingState {
    pub fn is_new_user(&self) -> bool {
        !self.setup_complete || self.scripts_run < 3
    }
    
    pub fn should_show_hint(&self, hint_id: &str) -> bool {
        !self.dismissed_hints.contains(hint_id)
            && !self.completed_steps.contains(hint_id)
    }
    
    pub fn complete_step(&mut self, step_id: &str) {
        self.completed_steps.insert(step_id.to_string());
    }
    
    pub fn dismiss_hint(&mut self, hint_id: &str) {
        self.dismissed_hints.insert(hint_id.to_string());
    }
    
    pub fn discover_feature(&mut self, feature_id: &str) {
        self.discovered_features.insert(feature_id.to_string());
    }
}

/// Onboarding step definition
#[derive(Clone)]
pub struct OnboardingStep {
    pub id: String,
    pub title: SharedString,
    pub description: SharedString,
    pub action: Option<OnboardingAction>,
    pub completion_criteria: CompletionCriteria,
    pub highlight_element: Option<String>, // Element ID to highlight
}

#[derive(Clone)]
pub enum OnboardingAction {
    ShowModal,
    HighlightElement(String),
    RunScript(String),
    OpenUrl(String),
}

#[derive(Clone)]
pub enum CompletionCriteria {
    UserDismissed,
    ActionCompleted,
    FeatureUsed(String),
    ScriptRun,
    TimeElapsed(Duration),
}
```

### Welcome Flow

```rust
// src/onboarding/welcome.rs
use crate::theme::Theme;
use gpui::*;

pub struct WelcomeFlow {
    current_step: usize,
    steps: Vec<WelcomeStep>,
    theme: Arc<Theme>,
}

struct WelcomeStep {
    title: SharedString,
    description: SharedString,
    illustration: WelcomeIllustration,
    action_label: Option<SharedString>,
}

enum WelcomeIllustration {
    ScriptKit,
    Keyboard,
    Scripts,
    Complete,
}

impl WelcomeFlow {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            current_step: 0,
            steps: vec![
                WelcomeStep {
                    title: "Welcome to Script Kit".into(),
                    description: "Your shortcut to productivity. Automate tasks, launch apps, and build tools with simple scripts.".into(),
                    illustration: WelcomeIllustration::ScriptKit,
                    action_label: Some("Get Started".into()),
                },
                WelcomeStep {
                    title: "Quick Access".into(),
                    description: "Press ⌘; anytime to open Script Kit. Type to search your scripts instantly.".into(),
                    illustration: WelcomeIllustration::Keyboard,
                    action_label: Some("Next".into()),
                },
                WelcomeStep {
                    title: "Run Your First Script".into(),
                    description: "Select a script and press Enter to run it. Try the \"Hello World\" example!".into(),
                    illustration: WelcomeIllustration::Scripts,
                    action_label: Some("Let's Go".into()),
                },
            ],
            theme,
        }
    }
    
    fn next_step(&mut self, cx: &mut WindowContext) {
        if self.current_step < self.steps.len() - 1 {
            self.current_step += 1;
            cx.notify();
        } else {
            cx.emit(WelcomeComplete);
        }
    }
    
    fn skip(&mut self, cx: &mut WindowContext) {
        cx.emit(WelcomeComplete);
    }
}

impl Render for WelcomeFlow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        let step = &self.steps[self.current_step];
        
        div()
            .w(px(420.0))
            .rounded_xl()
            .bg(rgb(colors.ui.surface))
            .shadow_2xl()
            .overflow_hidden()
            .flex()
            .flex_col()
            // Illustration
            .child(
                div()
                    .h(px(200.0))
                    .bg(rgb(colors.accent.primary))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(self.render_illustration(&step.illustration))
            )
            // Content
            .child(
                div()
                    .p_6()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        div()
                            .text_size(px(20.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(colors.text.primary))
                            .child(step.title.clone())
                    )
                    .child(
                        div()
                            .text_size(px(14.0))
                            .text_color(rgb(colors.text.secondary))
                            .line_height(px(22.0))
                            .child(step.description.clone())
                    )
            )
            // Progress dots
            .child(
                div()
                    .px_6()
                    .pb_4()
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap_2()
                    .children((0..self.steps.len()).map(|i| {
                        div()
                            .w(px(8.0))
                            .h(px(8.0))
                            .rounded_full()
                            .bg(rgb(if i == self.current_step {
                                colors.accent.primary
                            } else {
                                colors.ui.border
                            }))
                    }))
            )
            // Actions
            .child(
                div()
                    .px_6()
                    .pb_6()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(13.0))
                            .text_color(rgb(colors.text.muted))
                            .cursor_pointer()
                            .hover(|s| s.text_color(rgb(colors.text.secondary)))
                            .on_click(cx.listener(|this, _, cx| {
                                this.skip(cx);
                            }))
                            .child("Skip")
                    )
                    .when_some(step.action_label.clone(), |el, label| {
                        el.child(
                            div()
                                .px_5()
                                .py_2()
                                .rounded_lg()
                                .bg(rgb(colors.accent.primary))
                                .text_color(rgb(colors.background.main))
                                .font_weight(FontWeight::MEDIUM)
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(colors.accent.hover)))
                                .on_click(cx.listener(|this, _, cx| {
                                    this.next_step(cx);
                                }))
                                .child(label)
                        )
                    })
            )
    }
}

pub struct WelcomeComplete;
```

### Contextual Hints

```rust
// src/onboarding/hints.rs
use crate::theme::Theme;
use gpui::*;

/// Contextual hint that appears near relevant UI
pub struct ContextualHint {
    id: String,
    message: SharedString,
    position: HintPosition,
    action: Option<HintAction>,
    theme: Arc<Theme>,
    visible: bool,
}

#[derive(Clone, Copy)]
pub enum HintPosition {
    Above,
    Below,
    Left,
    Right,
}

#[derive(Clone)]
pub struct HintAction {
    label: SharedString,
    handler: Arc<dyn Fn(&mut WindowContext)>,
}

impl ContextualHint {
    pub fn new(
        id: impl Into<String>,
        message: impl Into<SharedString>,
        theme: Arc<Theme>,
    ) -> Self {
        Self {
            id: id.into(),
            message: message.into(),
            position: HintPosition::Below,
            action: None,
            theme,
            visible: true,
        }
    }
    
    pub fn position(mut self, position: HintPosition) -> Self {
        self.position = position;
        self
    }
    
    pub fn action(mut self, label: impl Into<SharedString>, handler: impl Fn(&mut WindowContext) + 'static) -> Self {
        self.action = Some(HintAction {
            label: label.into(),
            handler: Arc::new(handler),
        });
        self
    }
}

impl Render for ContextualHint {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.visible {
            return div().into_any_element();
        }
        
        let colors = &self.theme.colors;
        let id = self.id.clone();
        
        div()
            .p_3()
            .rounded_lg()
            .bg(rgb(colors.accent.primary))
            .shadow_lg()
            .max_w(px(280.0))
            .flex()
            .flex_col()
            .gap_2()
            // Arrow (based on position)
            .child(self.render_arrow())
            // Content
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_start()
                    .gap_2()
                    .child(
                        Icon::new("lightbulb")
                            .size(px(16.0))
                            .color(rgb(colors.background.main))
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(13.0))
                            .text_color(rgb(colors.background.main))
                            .child(self.message.clone())
                    )
            )
            // Actions
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_end()
                    .gap_2()
                    // "Got it" dismiss
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(rgba(colors.background.main, 0.8))
                            .cursor_pointer()
                            .hover(|s| s.text_color(rgb(colors.background.main)))
                            .on_click(cx.listener(move |this, _, cx| {
                                cx.emit(DismissHint(id.clone()));
                                this.visible = false;
                                cx.notify();
                            }))
                            .child("Got it")
                    )
                    // Optional action
                    .when_some(self.action.clone(), |el, action| {
                        el.child(
                            div()
                                .px_2()
                                .py_1()
                                .rounded_sm()
                                .bg(rgba(colors.background.main, 0.2))
                                .text_size(px(12.0))
                                .text_color(rgb(colors.background.main))
                                .cursor_pointer()
                                .hover(|s| s.bg(rgba(colors.background.main, 0.3)))
                                .on_click(cx.listener(move |_, _, cx| {
                                    (action.handler)(cx);
                                }))
                                .child(action.label.clone())
                        )
                    })
            )
            .into_any_element()
    }
}

#[derive(Clone)]
pub struct DismissHint(pub String);
```

### Feature Spotlight

```rust
// Highlight a feature for discovery
impl MainMenu {
    fn render_with_spotlight(&self, cx: &mut WindowContext) -> impl IntoElement {
        let onboarding = cx.global::<OnboardingState>();
        let show_actions_spotlight = onboarding.should_show_hint("actions-tab");
        
        div()
            .relative()
            .child(self.render_content(cx))
            // Spotlight overlay for Tab key hint
            .when(show_actions_spotlight, |el| {
                el.child(
                    FeatureSpotlight::new(
                        "actions-tab",
                        "Quick Tip",
                        "Press Tab to see available actions for the selected item",
                        self.theme.clone(),
                    )
                    .target_element("actions-button")
                    .position(SpotlightPosition::Right)
                )
            })
    }
}

pub struct FeatureSpotlight {
    id: String,
    title: SharedString,
    description: SharedString,
    target_element: Option<String>,
    position: SpotlightPosition,
    theme: Arc<Theme>,
}

#[derive(Clone, Copy)]
pub enum SpotlightPosition {
    Top,
    Bottom,
    Left,
    Right,
}

impl FeatureSpotlight {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<SharedString>,
        description: impl Into<SharedString>,
        theme: Arc<Theme>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: description.into(),
            target_element: None,
            position: SpotlightPosition::Bottom,
            theme,
        }
    }
    
    pub fn target_element(mut self, element: impl Into<String>) -> Self {
        self.target_element = Some(element.into());
        self
    }
    
    pub fn position(mut self, position: SpotlightPosition) -> Self {
        self.position = position;
        self
    }
}
```

### Progress Indicator

```rust
// Show onboarding progress
impl OnboardingProgress {
    fn render(&self, cx: &mut WindowContext) -> impl IntoElement {
        let colors = &self.theme.colors;
        let onboarding = cx.global::<OnboardingState>();
        
        let total_steps = 5;
        let completed = onboarding.completed_steps.len();
        let progress = (completed as f32) / (total_steps as f32);
        
        if completed >= total_steps {
            return div().into_any_element();
        }
        
        div()
            .p_4()
            .rounded_lg()
            .bg(rgb(colors.ui.surface))
            .flex()
            .flex_col()
            .gap_3()
            // Header
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(14.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgb(colors.text.primary))
                            .child("Getting Started")
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(rgb(colors.text.muted))
                            .child(format!("{}/{}", completed, total_steps))
                    )
            )
            // Progress bar
            .child(
                div()
                    .w_full()
                    .h(px(4.0))
                    .rounded_full()
                    .bg(rgb(colors.ui.border))
                    .child(
                        div()
                            .h_full()
                            .rounded_full()
                            .bg(rgb(colors.accent.primary))
                            .w(Percentage(progress * 100.0))
                    )
            )
            // Next step
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(rgb(colors.text.secondary))
                    .child(self.get_next_step_message(onboarding))
            )
            .into_any_element()
    }
    
    fn get_next_step_message(&self, onboarding: &OnboardingState) -> String {
        if !onboarding.completed_steps.contains("run-script") {
            "Run your first script".to_string()
        } else if !onboarding.completed_steps.contains("create-script") {
            "Create a new script".to_string()
        } else if !onboarding.completed_steps.contains("use-shortcut") {
            "Try the global hotkey (⌘;)".to_string()
        } else if !onboarding.completed_steps.contains("explore-actions") {
            "Explore script actions (Tab)".to_string()
        } else {
            "Complete!".to_string()
        }
    }
}
```

## Testing

### Onboarding Test Script

```typescript
// tests/smoke/test-onboarding.ts
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Test 1: Welcome screen
await div(`
  <div class="w-[420px] rounded-xl bg-zinc-800 shadow-2xl overflow-hidden">
    <!-- Illustration -->
    <div class="h-48 bg-gradient-to-br from-amber-500 to-orange-600 flex items-center justify-center">
      <span class="text-6xl">⚡</span>
    </div>
    <!-- Content -->
    <div class="p-6 flex flex-col gap-3">
      <h1 class="text-xl font-bold text-white">Welcome to Script Kit</h1>
      <p class="text-sm text-zinc-400 leading-relaxed">
        Your shortcut to productivity. Automate tasks, launch apps, and build tools with simple scripts.
      </p>
    </div>
    <!-- Progress dots -->
    <div class="px-6 pb-4 flex justify-center gap-2">
      <div class="w-2 h-2 rounded-full bg-amber-500"></div>
      <div class="w-2 h-2 rounded-full bg-zinc-600"></div>
      <div class="w-2 h-2 rounded-full bg-zinc-600"></div>
    </div>
    <!-- Actions -->
    <div class="px-6 pb-6 flex justify-between items-center">
      <span class="text-sm text-zinc-500 cursor-pointer hover:text-zinc-400">Skip</span>
      <button class="px-5 py-2 rounded-lg bg-amber-500 text-black font-medium">
        Get Started
      </button>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot1 = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
writeFileSync(join(dir, 'onboarding-welcome.png'), Buffer.from(shot1.data, 'base64'));

// Test 2: Contextual hint
await div(`
  <div class="p-4 relative">
    <div class="h-10 px-4 flex items-center bg-zinc-800 rounded-md">
      <span class="text-white text-sm">Script Item</span>
    </div>
    <!-- Hint below -->
    <div class="mt-2 p-3 rounded-lg bg-amber-500 shadow-lg max-w-[280px]">
      <div class="flex items-start gap-2">
        <span class="text-black">💡</span>
        <div class="flex-1 text-sm text-black">
          Press Tab to see available actions for this script
        </div>
      </div>
      <div class="flex justify-end gap-2 mt-2">
        <span class="text-xs text-black/70 cursor-pointer">Got it</span>
        <span class="px-2 py-1 rounded-sm bg-black/20 text-xs text-black cursor-pointer">Try it</span>
      </div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot2 = await captureScreenshot();
writeFileSync(join(dir, 'onboarding-hint.png'), Buffer.from(shot2.data, 'base64'));

// Test 3: Progress indicator
await div(`
  <div class="p-4">
    <div class="p-4 rounded-lg bg-zinc-800 flex flex-col gap-3">
      <div class="flex items-center justify-between">
        <span class="text-sm font-medium text-white">Getting Started</span>
        <span class="text-xs text-zinc-500">2/5</span>
      </div>
      <div class="w-full h-1 rounded-full bg-zinc-700">
        <div class="h-full rounded-full bg-amber-500" style="width: 40%"></div>
      </div>
      <span class="text-xs text-zinc-400">Next: Create a new script</span>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot3 = await captureScreenshot();
writeFileSync(join(dir, 'onboarding-progress.png'), Buffer.from(shot3.data, 'base64'));

// Test 4: Feature spotlight
await div(`
  <div class="p-4 relative">
    <div class="flex items-center gap-2">
      <div class="h-10 px-4 flex items-center bg-zinc-800 rounded-md flex-1">
        <span class="text-white text-sm">Script Item</span>
      </div>
      <div class="relative">
        <button class="w-10 h-10 rounded-md bg-amber-500 text-black flex items-center justify-center ring-2 ring-amber-300 ring-offset-2 ring-offset-zinc-900">
          ⚡
        </button>
        <!-- Spotlight popup -->
        <div class="absolute left-full ml-2 top-1/2 -translate-y-1/2 p-3 rounded-lg bg-zinc-700 shadow-lg w-48">
          <div class="absolute left-0 top-1/2 -translate-x-1/2 -translate-y-1/2 w-2 h-2 bg-zinc-700 rotate-45"></div>
          <div class="text-sm font-medium text-white mb-1">Quick Actions</div>
          <div class="text-xs text-zinc-400">Click here or press Tab to see actions for this item</div>
        </div>
      </div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot4 = await captureScreenshot();
writeFileSync(join(dir, 'onboarding-spotlight.png'), Buffer.from(shot4.data, 'base64'));

console.error('[ONBOARDING] Test screenshots saved');
process.exit(0);
```

## Related Bundles

- Bundle #79: Tooltips & Hints - Hint delivery mechanism
- Bundle #75: Empty States - First-time user states
- Bundle #63: Config System - Persisting onboarding state
