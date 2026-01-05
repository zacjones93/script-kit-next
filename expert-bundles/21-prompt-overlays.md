# Expert Question 21: Prompt Overlay Architecture

## The Problem

Prompts (Div, Term, Path) have a 3-layer rendering stack: Content + Header overlay + Dialog overlay. Absolute positioning, animated visibility, and keyboard routing must coordinate across layers.

## Specific Concerns

1. **3-Layer Stack**: Content (prompt entity) + Header (action button OR search input) + Dialog (backdrop + actions menu). Each layer has different z-index and interaction semantics.

2. **Absolute Positioning**: Header floats over content with `inset_0()`. Dialog at `top(52px).right(8px)`. Backdrop covers entire prompt. Positions hardcoded, not responsive.

3. **Animated Visibility**: Button/search use `opacity(0).invisible()` to hide. Not removed from DOM—just invisible. Animation timing controlled externally.

4. **Keyboard Event Routing**: Cmd+K toggles popup, ESC dismisses, arrows navigate, Enter selects, backspace filters. All handled by parent listener, not dialog itself.

5. **Focus Delegation**: When popup open, ALL keyboard input routed to dialog. When closed, focus returns to parent. Transition timing can cause lost keystrokes.

## Questions for Expert

1. Is 3-layer absolute positioning the right approach, or should we use GPUI's built-in overlay primitives?
2. How do we make overlay positions responsive to window size changes?
3. Should keyboard routing be centralized (parent handles all) or distributed (each layer handles its own)?
4. What's the GPUI pattern for animated show/hide without removing from DOM?
5. How do we prevent lost keystrokes during focus transitions?

