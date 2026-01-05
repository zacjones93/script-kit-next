# Expert Question 11: File Watchers & Hot-Reload

## The Problem

We have four independent file watchers: ConfigWatcher, ThemeWatcher, ScriptWatcher, AppearanceChangeWatcher. Each uses debouncing (500ms delay) to coalesce rapid file changes.

## Specific Concerns

1. **Race Condition in Debounce**: Release lock before spawn, then re-acquire in spawned thread. Gap where second event can reset flag incorrectly.

2. **No Duplicate Event Suppression**: If file modified twice in 500ms, both events fire. Debounce only delays, not deduplicates.

3. **Watcher Thread Lifecycle**: ThreadHandle stored but never `.join()` called. Thread leaks if watcher dropped.

4. **notify Crate Quirks**: Emits duplicate events for fast edits. RecursiveMode vs. NonRecursive has different performance characteristics.

5. **Error Recovery**: If watch_loop fails, entire watcher thread exits silently. Only warns via tracing, no restart logic.

## Questions for Expert

1. What's the correct debounce pattern in Rust? Should we use a dedicated debounce crate?
2. How do we properly handle watcher thread lifecycle? Should watchers implement Drop?
3. Is the `notify` crate the right choice? Are there alternatives with better event deduplication?
4. Should all four watchers share a single notify::Watcher instance to reduce resource usage?
5. How do we make file watching robust against transient errors (permission changes, network drives)?

