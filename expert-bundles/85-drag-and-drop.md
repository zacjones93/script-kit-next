# Expert Bundle #85: Drag and Drop

## Overview

Drag and drop enables intuitive content manipulation through direct manipulation. In Script Kit, it supports script reordering, file dropping, and content organization. Good drag-and-drop UX provides clear visual feedback and graceful handling of edge cases.

## Architecture

### Drag Types

```rust
// src/drag_drop.rs
use gpui::*;

/// Data being dragged
#[derive(Clone, Debug)]
pub enum DragData {
    /// Script being reordered
    Script {
        id: String,
        name: SharedString,
        source_index: usize,
    },
    /// External file being dropped
    ExternalFile {
        path: PathBuf,
        mime_type: Option<String>,
    },
    /// Text content
    Text {
        content: String,
    },
    /// List item reordering
    ListItem {
        id: String,
        source_list: String,
        source_index: usize,
    },
}

/// Current drag state
#[derive(Default)]
pub struct DragState {
    pub active_drag: Option<ActiveDrag>,
    pub drop_target: Option<DropTarget>,
}

pub struct ActiveDrag {
    pub data: DragData,
    pub position: Point<Pixels>,
    pub offset: Point<Pixels>, // Offset from drag start point
    pub ghost_element: Option<AnyElement>,
}

#[derive(Clone)]
pub struct DropTarget {
    pub id: String,
    pub kind: DropTargetKind,
    pub bounds: Bounds<Pixels>,
    pub accepts: fn(&DragData) -> bool,
}

#[derive(Clone, Copy, Debug)]
pub enum DropTargetKind {
    /// Insert before this item
    Before,
    /// Insert after this item  
    After,
    /// Drop onto this item (e.g., into a folder)
    Onto,
    /// Drop into this zone
    Zone,
}

/// Drop position indicator
#[derive(Clone, Copy, Debug)]
pub enum DropIndicator {
    Line {
        orientation: Orientation,
        position: Point<Pixels>,
        length: Pixels,
    },
    Highlight {
        bounds: Bounds<Pixels>,
    },
    Ghost {
        bounds: Bounds<Pixels>,
        opacity: f32,
    },
}

#[derive(Clone, Copy, Debug)]
pub enum Orientation {
    Horizontal,
    Vertical,
}
```

### Drag Handler

```rust
// src/drag_drop/handler.rs
use gpui::*;

pub struct DragDropManager {
    state: DragState,
    drop_targets: Vec<DropTarget>,
    theme: Arc<Theme>,
}

impl DragDropManager {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            state: DragState::default(),
            drop_targets: vec![],
            theme,
        }
    }
    
    pub fn start_drag(&mut self, data: DragData, position: Point<Pixels>, cx: &mut WindowContext) {
        self.state.active_drag = Some(ActiveDrag {
            data,
            position,
            offset: Point::default(),
            ghost_element: None,
        });
        
        // Change cursor
        cx.set_cursor_style(CursorStyle::Grabbing);
        cx.notify();
    }
    
    pub fn update_drag(&mut self, position: Point<Pixels>, cx: &mut WindowContext) {
        if let Some(drag) = &mut self.state.active_drag {
            drag.position = position;
            
            // Find drop target under cursor
            self.state.drop_target = self.drop_targets
                .iter()
                .find(|target| target.bounds.contains(&position) && (target.accepts)(&drag.data))
                .cloned();
            
            cx.notify();
        }
    }
    
    pub fn end_drag(&mut self, cx: &mut WindowContext) -> Option<DropResult> {
        let drag = self.state.active_drag.take()?;
        let target = self.state.drop_target.take();
        
        cx.set_cursor_style(CursorStyle::default());
        cx.notify();
        
        target.map(|t| DropResult {
            data: drag.data,
            target_id: t.id,
            target_kind: t.kind,
        })
    }
    
    pub fn cancel_drag(&mut self, cx: &mut WindowContext) {
        self.state.active_drag = None;
        self.state.drop_target = None;
        cx.set_cursor_style(CursorStyle::default());
        cx.notify();
    }
    
    pub fn register_target(&mut self, target: DropTarget) {
        self.drop_targets.push(target);
    }
    
    pub fn unregister_target(&mut self, id: &str) {
        self.drop_targets.retain(|t| t.id != id);
    }
}

pub struct DropResult {
    pub data: DragData,
    pub target_id: String,
    pub target_kind: DropTargetKind,
}
```

### Draggable Component

```rust
// src/components/draggable.rs
use gpui::*;

/// Makes a child element draggable
pub struct Draggable<E> {
    child: E,
    data: DragData,
    drag_handle: Option<DragHandle>,
    disabled: bool,
}

pub enum DragHandle {
    /// Entire element is drag handle
    Full,
    /// Only specific area is drag handle
    Partial {
        render: Box<dyn Fn(&mut WindowContext) -> AnyElement>,
    },
}

impl<E: IntoElement> Draggable<E> {
    pub fn new(child: E, data: DragData) -> Self {
        Self {
            child,
            data,
            drag_handle: Some(DragHandle::Full),
            disabled: false,
        }
    }
    
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    
    pub fn with_handle(mut self, handle: DragHandle) -> Self {
        self.drag_handle = Some(handle);
        self
    }
}

impl<E: IntoElement> IntoElement for Draggable<E> {
    fn into_element(self) -> impl Element {
        let data = self.data.clone();
        let disabled = self.disabled;
        
        div()
            .cursor(if disabled { CursorStyle::default() } else { CursorStyle::Grab })
            .when(!disabled, |el| {
                el.on_drag(move |drag_event, window, cx| {
                    if let DragEvent::Start { position } = drag_event {
                        cx.emit(StartDrag {
                            data: data.clone(),
                            position: *position,
                        });
                    }
                })
                .on_drag_move(|position, window, cx| {
                    cx.emit(UpdateDrag { position });
                })
                .on_drag_end(|_, window, cx| {
                    cx.emit(EndDrag);
                })
            })
            .child(self.child)
    }
}
```

### Drop Zone Component

```rust
// src/components/drop_zone.rs
use crate::theme::Theme;
use gpui::*;

pub struct DropZone<E> {
    child: E,
    id: String,
    accepts: fn(&DragData) -> bool,
    theme: Arc<Theme>,
    highlight_on_hover: bool,
}

impl<E: IntoElement> DropZone<E> {
    pub fn new(id: impl Into<String>, child: E, theme: Arc<Theme>) -> Self {
        Self {
            child,
            id: id.into(),
            accepts: |_| true,
            theme,
            highlight_on_hover: true,
        }
    }
    
    pub fn accepts(mut self, predicate: fn(&DragData) -> bool) -> Self {
        self.accepts = predicate;
        self
    }
}

impl<E: IntoElement> Render for DropZone<E> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        let is_drop_target = /* check if currently targeted */;
        
        div()
            .relative()
            .child(self.child.clone())
            // Drop highlight overlay
            .when(is_drop_target && self.highlight_on_hover, |el| {
                el.child(
                    div()
                        .absolute()
                        .inset_0()
                        .rounded_md()
                        .border_2()
                        .border_dashed()
                        .border_color(rgb(colors.accent.primary))
                        .bg(with_alpha(colors.accent.primary, 0.1))
                        .pointer_events_none()
                )
            })
    }
}
```

## Visual Feedback

### Drop Indicator

```rust
// src/components/drop_indicator.rs
use crate::theme::Theme;
use gpui::*;

pub struct DropIndicatorView {
    indicator: DropIndicator,
    theme: Arc<Theme>,
}

impl Render for DropIndicatorView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        match self.indicator {
            DropIndicator::Line { orientation, position, length } => {
                div()
                    .absolute()
                    .left(position.x)
                    .top(position.y)
                    .map(|el| match orientation {
                        Orientation::Horizontal => el.w(length).h(px(2.0)),
                        Orientation::Vertical => el.w(px(2.0)).h(length),
                    })
                    .bg(rgb(colors.accent.primary))
                    .rounded_full()
                    // Animated pulse
                    .with_animation(
                        "drop-indicator-pulse",
                        Animation::new()
                            .duration(Duration::from_millis(1000))
                            .repeat()
                            .easing(ease_in_out_sine),
                        |el, t| {
                            let opacity = 0.6 + (t * 0.4);
                            el.opacity(opacity)
                        },
                    )
            }
            
            DropIndicator::Highlight { bounds } => {
                div()
                    .absolute()
                    .left(bounds.origin.x)
                    .top(bounds.origin.y)
                    .w(bounds.size.width)
                    .h(bounds.size.height)
                    .rounded_md()
                    .border_2()
                    .border_color(rgb(colors.accent.primary))
                    .bg(with_alpha(colors.accent.primary, 0.1))
            }
            
            DropIndicator::Ghost { bounds, opacity } => {
                div()
                    .absolute()
                    .left(bounds.origin.x)
                    .top(bounds.origin.y)
                    .w(bounds.size.width)
                    .h(bounds.size.height)
                    .rounded_md()
                    .bg(rgb(colors.ui.surface))
                    .border_1()
                    .border_dashed()
                    .border_color(rgb(colors.ui.border))
                    .opacity(opacity)
            }
        }
    }
}
```

### Drag Ghost

```rust
// Ghost element that follows cursor during drag
impl DragDropManager {
    fn render_drag_ghost(&self, cx: &mut WindowContext) -> Option<impl IntoElement> {
        let drag = self.state.active_drag.as_ref()?;
        let colors = &self.theme.colors;
        
        let position = Point::new(
            drag.position.x - drag.offset.x,
            drag.position.y - drag.offset.y,
        );
        
        Some(
            div()
                .absolute()
                .left(position.x)
                .top(position.y)
                .z_index(1000)
                .pointer_events_none()
                // Semi-transparent card
                .px_4()
                .py_2()
                .rounded_lg()
                .bg(rgb(colors.ui.surface))
                .shadow_xl()
                .opacity(0.9)
                .transform(Transform::scale(1.02))
                .child(self.render_ghost_content(&drag.data, cx))
        )
    }
    
    fn render_ghost_content(&self, data: &DragData, cx: &mut WindowContext) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        match data {
            DragData::Script { name, .. } => {
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(Icon::new("terminal").size(px(14.0)))
                    .child(
                        div()
                            .text_size(px(13.0))
                            .text_color(rgb(colors.text.primary))
                            .child(name.clone())
                    )
            }
            DragData::ExternalFile { path, .. } => {
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(Icon::new("file").size(px(14.0)))
                    .child(
                        div()
                            .text_size(px(13.0))
                            .text_color(rgb(colors.text.primary))
                            .child(path.file_name().unwrap_or_default().to_string_lossy().to_string())
                    )
            }
            _ => div(),
        }
        .into_any_element()
    }
}
```

## Usage Patterns

### Reorderable List

```rust
// src/components/reorderable_list.rs
impl ReorderableList {
    fn render(&mut self, cx: &mut WindowContext) -> impl IntoElement {
        let colors = &self.theme.colors;
        
        div()
            .flex()
            .flex_col()
            .children(self.items.iter().enumerate().map(|(i, item)| {
                let item_id = item.id.clone();
                let is_dragging = self.drag_index == Some(i);
                let is_drop_target = self.drop_index == Some(i);
                
                div()
                    .relative()
                    // Drop indicator line
                    .when(is_drop_target && self.drop_position == DropPosition::Before, |el| {
                        el.child(
                            div()
                                .absolute()
                                .left_0()
                                .right_0()
                                .top_0()
                                .h(px(2.0))
                                .bg(rgb(colors.accent.primary))
                                .rounded_full()
                        )
                    })
                    // Item
                    .child(
                        Draggable::new(
                            self.render_item(item, cx),
                            DragData::ListItem {
                                id: item_id.clone(),
                                source_list: self.id.clone(),
                                source_index: i,
                            },
                        )
                        .disabled(self.reorder_disabled)
                    )
                    // Style changes during drag
                    .when(is_dragging, |el| {
                        el.opacity(0.3)
                    })
            }))
    }
    
    fn handle_drop(&mut self, result: DropResult, cx: &mut WindowContext) {
        if let DragData::ListItem { source_index, .. } = result.data {
            let target_index = self.items.iter()
                .position(|i| i.id == result.target_id)
                .unwrap_or(self.items.len());
            
            // Reorder items
            if source_index != target_index {
                let item = self.items.remove(source_index);
                let insert_index = if source_index < target_index {
                    target_index - 1
                } else {
                    target_index
                };
                self.items.insert(insert_index, item);
                
                cx.emit(ItemsReordered(self.items.clone()));
            }
        }
        
        cx.notify();
    }
}
```

### File Drop Zone

```rust
// Accept external file drops
impl MainMenu {
    fn setup_file_drop(&mut self, cx: &mut WindowContext) {
        self.drop_zone = Some(DropZone::new(
            "main-drop-zone",
            self.render_content(cx),
            self.theme.clone(),
        )
        .accepts(|data| matches!(data, DragData::ExternalFile { .. })));
    }
    
    fn handle_file_drop(&mut self, result: DropResult, cx: &mut WindowContext) {
        if let DragData::ExternalFile { path, mime_type } = result.data {
            if self.is_script_file(&path) {
                self.import_script(path, cx);
            } else {
                self.notifications.warning("Unsupported file type", cx);
            }
        }
    }
    
    fn is_script_file(&self, path: &Path) -> bool {
        matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("ts") | Some("js") | Some("mjs")
        )
    }
}
```

## Testing

### Drag and Drop Test Script

```typescript
// tests/smoke/test-drag-drop.ts
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Test 1: List with drag handle
await div(`
  <div class="p-4 flex flex-col gap-1">
    <div class="h-[52px] px-4 flex items-center gap-3 bg-zinc-800 rounded-md cursor-grab">
      <span class="text-zinc-500">⋮⋮</span>
      <span class="text-white">Script A</span>
    </div>
    <div class="h-[52px] px-4 flex items-center gap-3 bg-zinc-700 rounded-md opacity-30">
      <span class="text-zinc-500">⋮⋮</span>
      <span class="text-white">Script B (dragging)</span>
    </div>
    <div class="relative h-[52px]">
      <div class="absolute left-0 right-0 top-0 h-0.5 bg-amber-500 rounded-full"></div>
      <div class="h-full px-4 flex items-center gap-3 bg-zinc-800 rounded-md">
        <span class="text-zinc-500">⋮⋮</span>
        <span class="text-white">Script C</span>
      </div>
    </div>
    <div class="h-[52px] px-4 flex items-center gap-3 bg-zinc-800 rounded-md">
      <span class="text-zinc-500">⋮⋮</span>
      <span class="text-white">Script D</span>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot1 = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
writeFileSync(join(dir, 'drag-reorder.png'), Buffer.from(shot1.data, 'base64'));

// Test 2: Drag ghost
await div(`
  <div class="p-8 relative">
    <div class="flex flex-col gap-1 opacity-50">
      <div class="h-10 px-4 flex items-center bg-zinc-800 rounded-md">
        <span class="text-white text-sm">Item 1</span>
      </div>
      <div class="h-10 px-4 flex items-center bg-zinc-800 rounded-md">
        <span class="text-white text-sm">Item 2</span>
      </div>
    </div>
    <!-- Ghost element -->
    <div class="absolute top-16 left-24 px-4 py-2 rounded-lg bg-zinc-700 shadow-xl flex items-center gap-2" style="transform: scale(1.02)">
      <span class="text-sm">📄</span>
      <span class="text-white text-sm">Dragging Item</span>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot2 = await captureScreenshot();
writeFileSync(join(dir, 'drag-ghost.png'), Buffer.from(shot2.data, 'base64'));

// Test 3: File drop zone
await div(`
  <div class="p-4">
    <div class="w-full h-48 rounded-lg border-2 border-dashed border-amber-500 bg-amber-500/10 flex flex-col items-center justify-center gap-3">
      <div class="w-16 h-16 rounded-full bg-amber-500/20 flex items-center justify-center">
        <span class="text-3xl">📁</span>
      </div>
      <div class="text-center">
        <div class="text-white font-medium">Drop files here</div>
        <div class="text-sm text-zinc-400">Supports .ts, .js files</div>
      </div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot3 = await captureScreenshot();
writeFileSync(join(dir, 'drag-drop-zone.png'), Buffer.from(shot3.data, 'base64'));

// Test 4: Multiple item selection drag
await div(`
  <div class="p-4">
    <div class="flex flex-col gap-1">
      <div class="h-10 px-4 flex items-center bg-amber-500/20 border border-amber-500 rounded-md">
        <span class="text-white text-sm">✓ Selected Item 1</span>
      </div>
      <div class="h-10 px-4 flex items-center bg-amber-500/20 border border-amber-500 rounded-md">
        <span class="text-white text-sm">✓ Selected Item 2</span>
      </div>
      <div class="h-10 px-4 flex items-center bg-zinc-800 rounded-md">
        <span class="text-white text-sm">Item 3</span>
      </div>
    </div>
    <div class="mt-4 text-xs text-zinc-500">Drag to move 2 items</div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot4 = await captureScreenshot();
writeFileSync(join(dir, 'drag-multi-select.png'), Buffer.from(shot4.data, 'base64'));

console.error('[DRAG AND DROP] Test screenshots saved');
process.exit(0);
```

## Related Bundles

- Bundle #73: Selection Feedback - Multi-select drag
- Bundle #64: List Virtualization - Reorderable virtualized lists
- Bundle #55: Animation & Transitions - Drag animations
