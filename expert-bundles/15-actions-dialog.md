# Expert Question 15: Actions Dialog System

## The Problem

We have a context-aware actions dialog (600+ LOC) that acts as both a global overlay popup (Cmd+K) AND as an inline header search. Actions route to either SDK callbacks or built-in app functions.

## Specific Concerns

1. **Dual-Mode Rendering**: Same component renders as popup overlay OR inline header. State machine tracks which mode is active, but transitions are complex.

2. **Dynamic Action Routing**: Two paths:
   - SDK actions with `has_action=true` → send `ActionTriggered` back to script
   - Built-in actions → trigger directly (Open Files, Reveal in Finder, etc.)

3. **Context-Aware Filtering**: Actions change based on focused result type (Script vs Scriptlet vs App vs BuiltIn vs Window). Generated on-the-fly.

4. **Focus Management**: Dialog must capture focus when visible, pass focus back to parent when dismissing. Cursor blink timer controlled externally.

5. **Search State Machine**: Animated toggle between action button and search input. Search must filter actions WITHOUT triggering parent's own search.

## Questions for Expert

1. Should the popup and inline modes be separate components instead of one dual-mode component?
2. What's the right pattern for action routing that's extensible but type-safe?
3. How do we properly manage focus handoff between dialog and parent?
4. Should action filtering be push-based (regenerate on context change) or pull-based (compute on render)?
5. Is there a GPUI-idiomatic way to do animated visibility toggles?

