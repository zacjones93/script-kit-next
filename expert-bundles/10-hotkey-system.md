# Expert Question 10: Hotkey Registration & Hot-Reload

## The Problem

We have a three-tier hotkey system (1,347 LOC): Main app hotkey + Notes hotkey + AI hotkey + dynamic script shortcuts. Hot-reload updates hotkeys without restarting the app.

## Specific Concerns

1. **Global Singleton Complexity**: `OnceLock<Mutex<GlobalHotKeyManager>>` + three `AtomicU32` for hotkey IDs. Atomics use mixed `Ordering::Relaxed` and `SeqCst`—potential race conditions.

2. **Unsafe FFI for macOS**: Direct FFI to `dispatch_async_f` with `Box::into_raw` trampoline. Must be perfect or memory corruption.

3. **Massive Code Duplication**: Hotkey config parsing repeated 3x for main/notes/AI windows. Same logic, different targets.

4. **Script Hotkey Map Management**: Must maintain three maps (hotkey_id→path, path→id, path→HotKey). Inconsistency = memory leak on hot-reload.

5. **Error Handling Asymmetry**: Main hotkey failure exits listener thread; script shortcuts log but continue. Inconsistent behavior.

## Questions for Expert

1. Is `OnceLock<Mutex<Option<T>>>` the right pattern for global mutable singletons in Rust?
2. Are the mixed `Ordering` semantics on our atomics correct? Should we use SeqCst everywhere?
3. How do we safely wrap macOS GCD dispatch in Rust? Are there crates that do this better?
4. Should we use a trait/generic for hotkey registration to eliminate the 3x duplication?
5. What's the right cleanup strategy when hot-reloading script hotkeys?

