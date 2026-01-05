# Expert Question 2: State Management Explosion

## The Problem

Our main `ScriptListApp` struct has **150+ fields** including:
- Script/scriptlet data (Arc-wrapped)
- UI state (9 scroll handles, multiple input states)
- 3 filter caches with string-based invalidation keys
- Channel management (SyncSender, receivers)
- Async coordination (subscriptions Vec)
- Focus tracking per-component

Every state mutation requires `cx.notify()` (264 occurrences in codebase). Missing one = silent UI staleness.

## Specific Concerns

1. **Cache Invalidation**: We use magic sentinel strings like `"\0_APPS_LOADED_\0"` to invalidate caches. This is fragile.

2. **cx.notify() Everywhere**: Easy to forget, causes hard-to-debug "UI doesn't update" bugs. No compile-time enforcement.

3. **Double-Borrow in Closures**: Render closures can't capture `&self` and also mutate. We pre-clone data into `Arc<[T]>` which feels wasteful.

4. **Multiple Scroll Handles**: We have 5+ scroll handles for different views. They share no abstraction and each has different lifecycle.

5. **Focus State Duplication**: `FocusedInput` enum tracks which input has focus, but each component also has its own `FocusHandle`.

## Questions for Expert

1. Should we split `ScriptListApp` into multiple smaller views? How does GPUI's composition model support this?
2. Is there a pattern to auto-notify on state changes (like Elm's update function)?
3. What's the idiomatic way to share state between parent and child views in GPUI?
4. Should we use a single source of truth for scroll state or is per-view acceptable?
5. How do large GPUI apps (Zed) manage this complexity?

