# Expert Question 5: Focus Management Across Components

## The Problem

Focus in GPUI requires:
1. Creating `FocusHandle` via `cx.focus_handle()` in each component
2. Calling `focus_handle.focus(window)` to grab focus
3. Checking `focus_handle.is_focused(window)` for styling
4. Re-rendering when focus changes

We have multiple focusable areas:
- Main filter input
- Arg prompt (choices list + text input)
- Div prompt (HTML content)
- Editor prompt (code editor)
- Form fields (multiple inputs in a form)
- Terminal (needs focus for keyboard input)

## Specific Concerns

1. **FocusHandle in Closures**: Can't use `self.focus_handle` in render closures because it doesn't implement `Copy`. Must extract `is_focused` before closure.

2. **Focus-Aware Colors**: Theme has `get_colors(is_focused)` returning different colors. Every focusable component must track focus state and re-render on change.

3. **Focus Transitions**: When user selects an item in ArgPrompt and presses Enter, focus should move to... where? Back to filter? To the next prompt?

4. **Tab Navigation in Forms**: Forms have multiple fields. Tab key must cycle through them. Each field has its own FocusHandle.

5. **Cursor Blink Timer**: Only ticks when an input is focused. Timer management interacts with focus state in confusing ways.

## Questions for Expert

1. Should we have a central focus manager instead of per-component FocusHandles?
2. How do we make focus-aware styling less boilerplate? A `Focusable` trait extension?
3. What's the GPUI-idiomatic way to do tab navigation between sibling components?
4. Should cursor blink be tied to focus, or a separate concern?
5. How does Zed handle focus transitions between panes/editors?

