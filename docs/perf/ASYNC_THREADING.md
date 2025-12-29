# Async/Threading Patterns & IPC Audit

> **Audit Date**: 2025-12-29  
> **Status**: READ-ONLY analysis  
> **Agent**: AsyncAuditor  
> **Cell**: cell--9bnr5-mjqv2hnipdb

## Executive Summary

This codebase shows **mature async patterns** with event-driven IPC using `async_channel`. The architecture has evolved from polling-based approaches to a hybrid model that prioritizes event-driven communication while using polling only for inherently polling operations (file watchers, clipboard monitoring).

**Key Findings**:
- 21 thread spawns identified (justified architectural split)
- Event-driven channels replace most polling loops
- Timer usage is appropriate for timeouts and UI refresh
- Some optimization opportunities in channel capacity tuning

---

## 1. Thread Inventory

### Total Thread Count: 21 `std::thread::spawn` calls

| Location | Purpose | Lifetime | Justification |
|----------|---------|----------|---------------|
| `main.rs:542` | **Stdin listener** | App lifetime | Blocking stdin read, must be off main thread |
| `main.rs:848` | App launcher scan | One-shot | Background I/O for app scanning |
| `main.rs:1533` | Open scripts folder | Fire-and-forget | External process spawn |
| `main.rs:1557` | Reveal in Finder | Fire-and-forget | External process spawn |
| `main.rs:1574` | Reveal app in Finder | Fire-and-forget | External process spawn |
| `main.rs:1730` | Edit script | Fire-and-forget | External editor spawn |
| `main.rs:1800` | Stderr reader | Script lifetime | Forward script stderr to logs |
| `main.rs:1827` | Stdin writer | Script lifetime | Write responses to script |
| `main.rs:1895` | Stdout reader | Script lifetime | Read script messages |
| `main.rs:5775` | Simulate paste | Fire-and-forget | Delayed keyboard simulation |
| `main.rs:8077` | Global hotkey listener | App lifetime | Platform hotkey monitoring |
| `clipboard_history.rs:224` | Image prewarm | One-shot | Background image decoding |
| `clipboard_history.rs:234` | Clipboard monitor | App lifetime | 500ms polling for clipboard changes |
| `watcher.rs:71` | Config watcher | App lifetime | File system notify loop |
| `watcher.rs:145` | Config debounce | Temporary | 500ms debounce window |
| `watcher.rs:211` | Theme watcher | App lifetime | File system notify loop |
| `watcher.rs:285` | Theme debounce | Temporary | 500ms debounce window |
| `watcher.rs:351` | Scripts watcher | App lifetime | File system notify loop |
| `watcher.rs:427` | Scripts debounce | Temporary | 500ms debounce window |
| `watcher.rs:491` | Appearance watcher | App lifetime | 2s polling for system appearance |
| `terminal/alacritty.rs:468` | PTY reader | Terminal lifetime | Blocking PTY read |

### Thread Classification

```
┌─────────────────────────────────────────────────────────────┐
│                    THREAD BREAKDOWN                         │
├─────────────────────────────────────────────────────────────┤
│  Long-lived (App/Feature lifetime):     10                  │
│  - Stdin listener (1)                                       │
│  - Script I/O threads (3: stdin, stdout, stderr)           │
│  - File watchers (3: config, theme, scripts)               │
│  - Appearance watcher (1)                                   │
│  - Clipboard monitor (1)                                    │
│  - Hotkey listener (1)                                      │
│  - PTY reader (1, per terminal)                             │
├─────────────────────────────────────────────────────────────┤
│  Short-lived (One-shot/Debounce):       11                  │
│  - Fire-and-forget process spawns (6)                       │
│  - Debounce timers (3)                                      │
│  - Image prewarm (1)                                        │
│  - App scanner (1)                                          │
└─────────────────────────────────────────────────────────────┘
```

### ✅ Thread Count Assessment: APPROPRIATE

The thread count is reasonable. Each thread serves a clear purpose:
- **Blocking I/O isolation**: Stdin, stdout, stderr, PTY reads must be off the main thread
- **Platform integration**: Global hotkey, file watchers, clipboard monitoring
- **Fire-and-forget**: Spawning external processes doesn't need tracking

**No unnecessary threads detected.**

---

## 2. Channel Analysis

### async_channel Usage (Event-Driven IPC)

| Location | Type | Capacity | Purpose |
|----------|------|----------|---------|
| `main.rs:401` | `bounded(10)` | 10 | Hotkey events |
| `main.rs:540` | `unbounded()` | ∞ | Stdin commands |
| `main.rs:1761` | `unbounded()` | ∞ | Prompt messages from script |
| `watcher.rs:476` | `bounded(100)` | 100 | Appearance change events |

### std::sync::mpsc Usage (Blocking IPC)

| Location | Purpose |
|----------|---------|
| `main.rs:845` | App launcher results (one-shot) |
| `main.rs:1820` | Response sender to script stdin |
| `watcher.rs:98,238,378` | notify crate callback channels |
| `terminal/alacritty.rs:460` | PTY output to main thread |

### Channel Capacity Recommendations

```
┌─────────────────────────────────────────────────────────────┐
│                CHANNEL CAPACITY TUNING                       │
├─────────────────────────────────────────────────────────────┤
│  GOOD:                                                       │
│  ✓ Hotkey channel bounded(10) - prevents memory growth      │
│  ✓ Appearance channel bounded(100) - reasonable buffer      │
│                                                              │
│  REVIEW NEEDED:                                              │
│  ⚠ Stdin command channel unbounded - low risk (external)    │
│  ⚠ Prompt message channel unbounded - script-controlled     │
├─────────────────────────────────────────────────────────────┤
│  RECOMMENDATION: Make unbounded channels bounded with        │
│  generous capacity (e.g., 1000) to prevent memory issues    │
│  from misbehaving scripts.                                   │
└─────────────────────────────────────────────────────────────┘
```

### Event-Driven vs Polling Evolution

The codebase shows intentional migration from polling to event-driven:

```rust
// OLD PATTERN (commented out):
// loop {
//     Timer::after(Duration::from_millis(100)).await;
//     // check AtomicBool
// }

// NEW PATTERN (current):
while let Ok(()) = hotkey_channel().1.recv().await {
    // Event-driven - no polling
}
```

**Evidence of evolution**:
- Comment at line 591: "No polling - replaces 100ms Timer::after loop"
- Comment at line 593: "event-driven via async_channel"
- Comment at line 3560: "Prompt messages are now handled via event-driven async_channel listener"

---

## 3. Timer/Polling Overhead

### Timer::after Usage (12 occurrences)

| Location | Delay | Purpose | Assessment |
|----------|-------|---------|------------|
| `main.rs:859` | 50ms | App launcher poll | ⚠ Could use oneshot channel |
| `main.rs:889` | 530ms | Cursor blink | ✅ UI animation timer |
| `main.rs:1015` | 5s | Timeout (context unclear) | ✅ Timeout pattern |
| `main.rs:1279` | 1000ms | Delayed operation | ✅ Debounce |
| `main.rs:8389` | 200ms | Config reload poll | ⚠ Try-recv anti-pattern |
| `main.rs:8406` | 200ms | Script reload poll | ⚠ Try-recv anti-pattern |
| `main.rs:8427` | 500ms | Test command poll | ⚠ Try-recv anti-pattern |
| `main.rs:8466` | 2s | Stdin timeout warning | ✅ One-shot timeout |
| `main.rs:8579` | 100ms | Tray event poll | ⚠ Platform limitation |
| `term_prompt.rs:220` | 16ms | Terminal refresh | ✅ Animation frame rate |
| `window_resize.rs:128` | 16ms | Resize debounce | ✅ Frame-aligned |

### Polling Anti-Patterns Identified

```rust
// ANTI-PATTERN: Timer + try_recv loop (lines 8387-8400)
cx.spawn(async move |cx| {
    loop {
        Timer::after(Duration::from_millis(200)).await;
        if config_rx.try_recv().is_ok() {
            // Handle event
        }
    }
});
```

**Why it's suboptimal**:
- Wastes CPU cycles checking empty channel 5x/second
- Adds 0-200ms latency to event handling
- Inconsistent with event-driven pattern used elsewhere

**Better pattern (already used for hotkey/stdin)**:
```rust
cx.spawn(async move |cx| {
    while let Ok(event) = rx.recv().await {
        // Immediate handling, no polling
    }
});
```

### Timer Overhead Estimate

```
┌─────────────────────────────────────────────────────────────┐
│                  TIMER CPU OVERHEAD                          │
├─────────────────────────────────────────────────────────────┤
│  Polling timers (suboptimal):                                │
│  - Config reload:  200ms interval = 5 wakeups/sec           │
│  - Script reload:  200ms interval = 5 wakeups/sec           │
│  - Test command:   500ms interval = 2 wakeups/sec           │
│  - Tray events:    100ms interval = 10 wakeups/sec          │
│  Total: ~22 wakeups/second = ~0.02% CPU (negligible)        │
├─────────────────────────────────────────────────────────────┤
│  Legitimate animation timers:                                │
│  - Cursor blink:   530ms = 1.9 wakeups/sec                  │
│  - Terminal:       16ms = 60 wakeups/sec (only when active) │
│  - Window resize:  One-shot (no ongoing cost)                │
└─────────────────────────────────────────────────────────────┘
```

**CPU impact is minimal**, but converting poll loops to event-driven would:
- Reduce latency (0-200ms → immediate)
- Improve code consistency
- Remove unnecessary wakeups when idle

---

## 4. cx.spawn Async Task Lifecycle

### Task Inventory (15 cx.spawn calls)

| Location | Pattern | Lifetime Management |
|----------|---------|---------------------|
| `main.rs:592` | Hotkey listener | `.detach()` - runs until channel closes |
| `main.rs:856` | App launcher poll | `.detach()` - breaks after result |
| `main.rs:887` | Cursor blink | `.detach()` - infinite loop |
| `main.rs:1014` | Timeout task | `.detach()` - one-shot |
| `main.rs:1278` | Debounce task | `.detach()` - one-shot |
| `main.rs:1766` | Prompt listener | `.detach()` - runs until channel closes |
| `main.rs:8372` | Appearance listener | `.detach()` - runs until channel closes |
| `main.rs:8387` | Config poll | `.detach()` - infinite loop |
| `main.rs:8404` | Script poll | `.detach()` - infinite loop |
| `main.rs:8424` | Test command poll | `.detach()` - infinite loop |
| `main.rs:8465` | Stdin timeout | `.detach()` - one-shot |
| `main.rs:8482` | Stdin handler | `.detach()` - runs until channel closes |
| `main.rs:8574` | Tray handler | `.detach()` - infinite loop |
| `term_prompt.rs:218` | Terminal refresh | `.detach()` - breaks on exit |
| `window_resize.rs:126` | Resize debounce | `.detach()` - one-shot |

### Lifecycle Issues

**All tasks use `.detach()`** which means:
- ✅ Tasks run independently of the spawning context
- ✅ No join handles to manage
- ⚠ No explicit cancellation mechanism for infinite loops

**Graceful shutdown consideration**:
- Tasks using channels will exit when sender is dropped
- Infinite poll loops (cursor blink, config poll) run until app exit
- This is acceptable for an app that runs until user quits

---

## 5. Arc<Mutex<T>> Patterns

### Usage Inventory

| Location | Type | Purpose | Contention Risk |
|----------|------|---------|-----------------|
| `main.rs:474` | `Arc<Mutex<Option<ScriptSession>>>` | Active script session | Low - accessed serially |
| `clipboard_history.rs:75` | `Arc<Mutex<Connection>>` | SQLite connection | Medium - DB operations |
| `clipboard_history.rs:78` | `Arc<Mutex<bool>>` | Stop flag | Low - simple flag |
| `watcher.rs:94,234,372` | `Arc<Mutex<bool>>` | Debounce flags | Low - brief holds |
| `terminal/alacritty.rs:80` | `Arc<Mutex<Vec<TerminalEvent>>>` | Event queue | Medium - terminal I/O |
| `terminal/alacritty.rs:352` | `Arc<Mutex<TerminalState>>` | Terminal grid state | High - frequent access |

### Potential Bottlenecks

```
┌─────────────────────────────────────────────────────────────┐
│              MUTEX CONTENTION ANALYSIS                       │
├─────────────────────────────────────────────────────────────┤
│  HIGH CONTENTION RISK:                                       │
│  - terminal/alacritty.rs:352 (TerminalState)                │
│    PTY reader thread + UI render thread both access         │
│    Mitigation: Consider RwLock for read-heavy access        │
│                                                              │
│  MEDIUM CONTENTION:                                          │
│  - clipboard_history.rs:75 (DB connection)                   │
│    Monitor thread + UI queries compete                       │
│    Mitigation: Connection pooling or separate connections   │
│                                                              │
│  LOW CONTENTION (acceptable):                                │
│  - All debounce flags (brief lock/unlock)                    │
│  - Script session (serialized access pattern)                │
└─────────────────────────────────────────────────────────────┘
```

### Lock Usage in Critical Paths

Checking for locks in render paths:

```rust
// terminal/alacritty.rs:656 - render_grid() acquires lock
let state = self.state.lock().unwrap();
```

This lock in the render path could cause frame drops if:
- PTY reader is holding the lock during a large write
- Workaround: Try-lock with fallback to cached state

---

## 6. Blocking Operations in Async Contexts

### Potential Issues Found

```rust
// CONCERN: stdin.lock() in sync thread context - OK (dedicated thread)
// main.rs:545
let reader = stdin.lock();

// CONCERN: .lock().unwrap() could panic - acceptable risk
// main.rs:1758
*self.script_session.lock().unwrap() = Some(session);
```

### Blocking Analysis

| Code Location | Blocking Call | Context | Assessment |
|--------------|---------------|---------|------------|
| `stdin.lock()` | Blocking | Dedicated thread | ✅ Correct |
| `Mutex::lock()` | Blocking | Main thread | ✅ Brief holds |
| `Command::spawn()` | Non-blocking | Fire-and-forget | ✅ Correct |
| `recv_blocking()` | Blocking | Dedicated threads | ✅ Correct |
| `thread::sleep()` | Blocking | Debounce threads | ✅ Correct |

**No async-blocking antipatterns found.** All blocking operations are properly isolated to dedicated threads.

---

## 7. Synchronization Bottlenecks

### Global Static Analysis

```rust
// main.rs - Global state using atomics (good)
static WINDOW_VISIBLE: AtomicBool = AtomicBool::new(false);
static NEEDS_RESET: AtomicBool = AtomicBool::new(false);
static PANEL_CONFIGURED: AtomicBool = AtomicBool::new(false);
static STDIN_RECEIVED: AtomicBool = AtomicBool::new(false);

// main.rs:397 - Channel via OnceLock (good)
static HOTKEY_CHANNEL: OnceLock<(Sender<()>, Receiver<()>)> = OnceLock::new();
```

**Assessment**:
- AtomicBool for simple flags: ✅ Correct
- OnceLock for one-time initialization: ✅ Correct
- No global Mutex bottlenecks detected

### Ordering Semantics

All AtomicBool operations use `Ordering::SeqCst`:
- ✅ Correct for visibility flags that need cross-thread consistency
- ⚠ Could use `Ordering::Relaxed` for some flags where strict ordering isn't needed
- Impact: Negligible (atomic operations are fast)

---

## 8. Recommendations

### Priority 1: Convert Poll Loops to Event-Driven

**Files**: `main.rs` lines 8387-8400, 8404-8416, 8424-8455

```rust
// CURRENT (polling)
loop {
    Timer::after(Duration::from_millis(200)).await;
    if rx.try_recv().is_ok() { /* handle */ }
}

// RECOMMENDED (event-driven)
while let Ok(event) = rx.recv().await {
    /* handle immediately */
}
```

**Benefits**:
- Eliminates 200ms max latency
- Reduces unnecessary wakeups
- Consistent with rest of codebase

### Priority 2: Add Capacity Bounds to Unbounded Channels

**Files**: `main.rs:540`, `main.rs:1761`

```rust
// CURRENT
let (tx, rx) = async_channel::unbounded();

// RECOMMENDED
let (tx, rx) = async_channel::bounded(1000);
```

**Benefits**:
- Prevents memory growth from misbehaving scripts
- 1000 is generous enough to not cause backpressure in normal use

### Priority 3: Consider RwLock for Terminal State

**File**: `terminal/alacritty.rs:352`

```rust
// CURRENT
state: Arc<Mutex<TerminalState>>

// CONSIDER
state: Arc<RwLock<TerminalState>>
```

**Benefits**:
- UI renders can read concurrently
- Only PTY writer needs exclusive access
- May reduce frame drops during heavy output

### Priority 4: Document Thread Shutdown

Add explicit shutdown signals for infinite loops:

```rust
// RECOMMENDED: Add stop flags to long-running loops
static SHUTDOWN_FLAG: AtomicBool = AtomicBool::new(false);

// In cursor blink loop:
while !SHUTDOWN_FLAG.load(Ordering::Relaxed) {
    Timer::after(Duration::from_millis(530)).await;
    // ...
}
```

**Benefits**:
- Clean app shutdown
- Testability
- Resource cleanup

---

## Summary

| Category | Status | Notes |
|----------|--------|-------|
| Thread Count | ✅ Good | 21 threads, all justified |
| Channel Patterns | ✅ Good | Event-driven architecture |
| Timer Usage | ⚠ Review | Some poll loops could be event-driven |
| Mutex Patterns | ⚠ Review | Terminal state could use RwLock |
| Async Blocking | ✅ Good | No blocking in async contexts |
| Global State | ✅ Good | Proper atomic usage |

**Overall Assessment**: The async/threading architecture is **well-designed** with intentional evolution toward event-driven patterns. The recommendations above are optimizations, not critical fixes.

---

Skills: [N/A - read-only audit] | Cmds: [grep, read] | Changed: [docs/perf/ASYNC_THREADING.md] | Risks: [none]
