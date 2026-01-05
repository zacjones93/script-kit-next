# Expert Question 3: Multi-Window Architecture & Coordination

## The Problem

We have 3 independent windows:
1. **Main launcher** - script list, prompts, search
2. **Notes window** - floating markdown editor
3. **AI chat window** - floating BYOK chat

Each is single-instance enforced via `OnceLock<Mutex<Option<WindowHandle<Root>>>>`.

## Specific Concerns

1. **OnceLock Limitations**: `OnceLock` can't be reset. When window closes, we set the inner `Option` to `None`, but the lock itself persists. Is this the right pattern?

2. **Window Discovery**: macOS creates tray popups and menu bar windows alongside our windows. Index-based `cx.windows()[0]` fails. We solved this with `WindowManager` struct but it feels hacky.

3. **Theme Synchronization**: All 3 windows + tray must sync theme from `~/.sk/kit/theme.json`. Currently each window has its own `ThemeWatcher`. Should there be a central theme bus?

4. **Focus Management**: Each window has independent focus handles. When switching between windows, focus state can get confused.

5. **Root Wrapper Requirement**: Every window must use gpui-component's `Root` wrapper for theme provision. Easy to forget during window creation.

## Questions for Expert

1. Is `OnceLock<Mutex<Option<WindowHandle>>>` the right pattern for single-instance windows in GPUI?
2. How should we discover our own windows reliably on macOS? Window titles? Tags?
3. Should theme be global state or per-window state? How do other multi-window apps handle this?
4. Is there a better abstraction than 3 separate `OnceLock`s for our windows?
5. Any patterns for cross-window communication in GPUI?

