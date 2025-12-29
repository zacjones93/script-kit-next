# IPC Protocol Security Audit

**Auditor:** security-ipc-auditor  
**Date:** 2024-12-29  
**Files Reviewed:** `src/protocol.rs`, `src/main.rs`  
**Severity Scale:** Critical | High | Medium | Low | Informational

---

## Executive Summary

This audit examined the IPC (Inter-Process Communication) protocol used by Script Kit GPUI for communication between the Rust application and child script processes (bun/node). The protocol uses JSONL (newline-delimited JSON) over stdin/stdout pipes.

### Overall Risk Rating: **Medium-High**

The protocol implementation shows good defensive coding practices for JSON parsing with graceful error handling. However, several areas require attention:

1. **No message size limits** - potential DoS vector via memory exhaustion
2. **No rate limiting** on message processing
3. **Shell command injection risks** in some message handlers
4. **Type confusion possible** in some message variants with flexible `serde_json::Value` fields
5. **No message authentication** - any process with pipe access can send messages

### Positive Findings

- Graceful parsing with `ParseResult` enum handles unknown message types safely
- Buffer reuse in `JsonlReader` prevents allocation-per-message
- Strong type system via Rust enums provides message validation
- Empty lines and malformed JSON are skipped rather than crashing

---

## Findings Table

| ID | Severity | Title | Location | Status |
|----|----------|-------|----------|--------|
| IPC-001 | Medium | No Message Size Limits | `protocol.rs:2276-2294` | Open |
| IPC-002 | Medium | Unbounded String Fields | `protocol.rs` (various) | Open |
| IPC-003 | Medium | Shell Command Injection via Exec | `protocol.rs:921-926` | Open |
| IPC-004 | Low | Type Confusion in serde_json::Value Fields | `protocol.rs:664-668` | Open |
| IPC-005 | Low | No Rate Limiting on Message Processing | `main.rs:1774-1788` | Open |
| IPC-006 | Informational | Process Keeps Running on Malformed Messages | `protocol.rs:2333-2340` | By Design |
| IPC-007 | Medium | Deserialization Stack Overflow Potential | `protocol.rs:2147-2156` | Open |
| IPC-008 | Low | Clipboard Content Not Sanitized | `main.rs:2027-2033` | Open |
| IPC-009 | Informational | No Message Authentication | Protocol design | By Design |
| IPC-010 | Medium | Large Choices Array Memory Exhaustion | `protocol.rs:643-647` | Open |

---

## Detailed Analysis

### IPC-001: No Message Size Limits

**Severity:** Medium  
**Location:** `protocol.rs:2276-2294` (`JsonlReader::next_message`)

**Description:**
The `JsonlReader` uses `BufReader::read_line()` which will read an entire line into memory regardless of size. A malicious or buggy script could send a single JSON line containing gigabytes of data, causing memory exhaustion.

**Code:**
```rust
pub fn next_message(&mut self) -> Result<Option<Message>, Box<dyn std::error::Error>> {
    self.line_buffer.clear();
    match self.reader.read_line(&mut self.line_buffer)? {
        0 => Ok(None),
        bytes_read => {
            // No check on bytes_read size
            let trimmed = self.line_buffer.trim();
            // ...
        }
    }
}
```

**Impact:**
- Memory exhaustion leading to application crash
- Denial of Service (DoS) by sending large messages

**Remediation:**
```rust
const MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024; // 10 MB

pub fn next_message(&mut self) -> Result<Option<Message>, Box<dyn std::error::Error>> {
    self.line_buffer.clear();
    
    // Read with size limit
    let mut total_read = 0;
    loop {
        let buf = self.reader.fill_buf()?;
        if buf.is_empty() {
            return Ok(None);
        }
        
        if let Some(newline_pos) = buf.iter().position(|&b| b == b'\n') {
            let line_bytes = newline_pos + 1;
            if total_read + line_bytes > MAX_MESSAGE_SIZE {
                return Err("Message exceeds maximum size".into());
            }
            self.line_buffer.push_str(
                std::str::from_utf8(&buf[..newline_pos])?
            );
            self.reader.consume(line_bytes);
            break;
        }
        
        total_read += buf.len();
        if total_read > MAX_MESSAGE_SIZE {
            return Err("Message exceeds maximum size".into());
        }
        
        let buf_str = std::str::from_utf8(buf)?;
        self.line_buffer.push_str(buf_str);
        let len = buf.len();
        self.reader.consume(len);
    }
    // ... rest of parsing
}
```

---

### IPC-002: Unbounded String Fields

**Severity:** Medium  
**Location:** Multiple locations in `protocol.rs`

**Description:**
Many message variants contain `String` fields without size validation:

- `html` in `Div`, `Form`, `Widget`, `SetPanel`, `SetPreview`, `SetPrompt`
- `content` in `Editor`, `Clipboard`
- `text` in `Say`
- `template` in `Template`
- `command` in `Exec`, `Term`
- `data` in `ScreenshotResult` (base64 PNG)

A malicious script could send extremely large HTML content or base64 data, consuming excessive memory.

**Code Example:**
```rust
Message::Div {
    id: String,
    html: String,  // No size limit
    tailwind: Option<String>,
}
```

**Impact:**
- Memory exhaustion
- UI rendering performance degradation with large HTML

**Remediation:**
Add field-level validation after deserialization:

```rust
impl Message {
    pub fn validate(&self) -> Result<(), ValidationError> {
        const MAX_HTML_SIZE: usize = 1024 * 1024; // 1 MB
        const MAX_BASE64_SIZE: usize = 50 * 1024 * 1024; // 50 MB for screenshots
        
        match self {
            Message::Div { html, .. } |
            Message::Form { html, .. } |
            Message::Widget { html, .. } |
            Message::SetPanel { html } |
            Message::SetPreview { html } |
            Message::SetPrompt { html } => {
                if html.len() > MAX_HTML_SIZE {
                    return Err(ValidationError::FieldTooLarge("html"));
                }
            }
            Message::ScreenshotResult { data, .. } => {
                if data.len() > MAX_BASE64_SIZE {
                    return Err(ValidationError::FieldTooLarge("data"));
                }
            }
            // ... other variants
            _ => {}
        }
        Ok(())
    }
}
```

---

### IPC-003: Shell Command Injection via Exec

**Severity:** Medium  
**Location:** `protocol.rs:921-926`

**Description:**
The `Exec` message type allows scripts to request shell command execution:

```rust
#[serde(rename = "exec")]
Exec {
    command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<serde_json::Value>,
}
```

While this is intentional functionality, a compromised or malicious script could execute arbitrary system commands. The current implementation does not appear to sanitize or validate the command string.

**Impact:**
- Arbitrary command execution with user privileges
- System compromise if scripts can be injected

**Context:**
This is somewhat expected behavior for Script Kit (scripts ARE supposed to be able to execute commands). However, consider:

1. **Allowlist approach:** Only allow specific commands or command patterns
2. **Sandboxing:** Run scripts in a sandboxed environment
3. **User confirmation:** Prompt for confirmation on dangerous commands

**Remediation (Defense in Depth):**
```rust
// Add command validation
fn validate_exec_command(command: &str) -> Result<(), SecurityError> {
    // Block common dangerous patterns
    let dangerous_patterns = [
        "rm -rf /",
        "mkfs",
        "> /dev/",
        "dd if=",
        ":(){ :|:& };:",  // Fork bomb
    ];
    
    for pattern in dangerous_patterns {
        if command.contains(pattern) {
            return Err(SecurityError::DangerousCommand(pattern.to_string()));
        }
    }
    Ok(())
}
```

---

### IPC-004: Type Confusion in serde_json::Value Fields

**Severity:** Low  
**Location:** `protocol.rs:664-668` and other locations

**Description:**
Several message variants use `serde_json::Value` for flexible data:

```rust
#[serde(rename = "update")]
Update {
    id: String,
    #[serde(flatten)]
    data: serde_json::Value,
}

#[serde(rename = "mouse")]
Mouse {
    action: MouseAction,
    data: Option<serde_json::Value>,
}
```

This allows any JSON structure, bypassing type safety. Handlers must carefully validate the structure before use.

**Impact:**
- Type confusion bugs if handlers assume specific structure
- Potential crashes on unexpected data types

**Remediation:**
Define explicit types for known data structures:

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MouseMoveData {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MouseClickData {
    pub button: u8,
    pub x: i32,
    pub y: i32,
}

#[serde(rename = "mouse")]
Mouse {
    action: MouseAction,
    #[serde(flatten)]
    data: MouseData,  // Enum with typed variants
}
```

---

### IPC-005: No Rate Limiting on Message Processing

**Severity:** Low  
**Location:** `main.rs:1774-1788`

**Description:**
The message processing loop processes messages as fast as they arrive:

```rust
// Event-driven: recv().await yields until a message arrives
while let Ok(msg) = rx_for_listener.recv().await {
    logging::log("EXEC", &format!("Prompt message received: {:?}", msg));
    let _ = cx.update(|cx| {
        this.update(cx, |app, cx| {
            app.handle_prompt_message(msg, cx);
        })
    });
}
```

A malicious script could flood the UI thread with messages, causing UI freezing.

**Impact:**
- UI thread starvation
- Application unresponsiveness

**Remediation:**
Add message coalescing or rate limiting:

```rust
const MAX_MESSAGES_PER_SECOND: usize = 100;
let mut message_count = 0;
let mut window_start = Instant::now();

while let Ok(msg) = rx_for_listener.recv().await {
    // Rate limiting
    message_count += 1;
    if window_start.elapsed() < Duration::from_secs(1) {
        if message_count > MAX_MESSAGES_PER_SECOND {
            logging::log("WARN", "Message rate limit exceeded, dropping message");
            continue;
        }
    } else {
        window_start = Instant::now();
        message_count = 1;
    }
    
    // Process message...
}
```

---

### IPC-006: Process Keeps Running on Malformed Messages

**Severity:** Informational  
**Location:** `protocol.rs:2333-2340`

**Description:**
The `next_message_graceful` method logs and skips malformed messages rather than failing:

```rust
ParseResult::ParseError(e) => {
    warn!(
        error = %e,
        "Skipping malformed message, continuing to next message"
    );
    continue;
}
```

**Assessment:**
This is the correct behavior for robustness. The application should not crash due to a single malformed message from a script.

**Status:** By Design - No change needed

---

### IPC-007: Deserialization Stack Overflow Potential

**Severity:** Medium  
**Location:** `protocol.rs:2147-2156`

**Description:**
Deeply nested JSON objects can cause stack overflow during serde deserialization. This is a known issue with recursive deserialization.

**Example Attack:**
```json
{"type":"update","id":"1","data":{"a":{"b":{"c":{"d":{"e":{"f":{"g":{"h":...deeply nested...}}}}}}}}}
```

**Impact:**
- Stack overflow crash
- Denial of Service

**Remediation:**
Use serde's `#[serde(deserialize_with = "...")]` with a depth-limited deserializer:

```rust
use serde::de::{Deserializer, Error as DeError};

fn deserialize_with_depth_limit<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    // Use serde_json::StreamDeserializer with recursion limit
    // Or implement custom visitor with depth tracking
}
```

Alternatively, configure serde_json to limit recursion depth (if building with custom feature).

---

### IPC-008: Clipboard Content Not Sanitized

**Severity:** Low  
**Location:** `main.rs:2027-2033`

**Description:**
Clipboard write operations accept arbitrary content without sanitization:

```rust
protocol::ClipboardAction::Write => {
    if let Some(text) = content {
        use arboard::Clipboard;
        if let Ok(mut clipboard) = Clipboard::new() {
            let _ = clipboard.set_text(text.clone());  // No sanitization
        }
    }
}
```

**Impact:**
- Scripts can write any content to system clipboard
- Potential for clipboard hijacking attacks

**Assessment:**
This is expected Script Kit behavior. Scripts should be able to write to clipboard. However, consider logging clipboard writes for audit purposes.

**Remediation (Optional Audit Trail):**
```rust
if let Some(text) = content {
    // Log truncated content for audit
    let preview = if text.len() > 100 {
        format!("{}... ({} bytes)", &text[..100], text.len())
    } else {
        text.clone()
    };
    logging::log("CLIPBOARD", &format!("Script writing to clipboard: {}", preview));
    // ... write to clipboard
}
```

---

### IPC-009: No Message Authentication

**Severity:** Informational  
**Location:** Protocol design

**Description:**
There is no mechanism to authenticate that messages are coming from a legitimate child process. Any process with access to the pipe can inject messages.

**Assessment:**
In the current design, Script Kit spawns child processes and communicates via pipes. The pipe handles are private to the parent-child relationship, so in practice only the spawned script can send messages.

However, if:
- The script forks or spawns subprocesses
- Another process gains access to the pipe (via /proc on Linux)
- The script passes pipe handles to other processes

Then unauthorized message injection is possible.

**Status:** By Design - The trust model assumes scripts are trusted

**Recommendation for Future:**
If higher security is needed, consider:
1. Message signing with shared secret
2. Process ID validation
3. Capability-based security model

---

### IPC-010: Large Choices Array Memory Exhaustion

**Severity:** Medium  
**Location:** `protocol.rs:643-647`

**Description:**
The `Arg`, `Mini`, `Micro`, and `Select` messages contain a `choices: Vec<Choice>` field with no size limit:

```rust
#[serde(rename = "arg")]
Arg {
    id: String,
    placeholder: String,
    choices: Vec<Choice>,  // Unbounded
}
```

A script could send millions of choices, exhausting memory.

**Impact:**
- Memory exhaustion
- UI hang during rendering

**Remediation:**
Add choice count validation:

```rust
const MAX_CHOICES: usize = 10_000;

impl Message {
    pub fn validate(&self) -> Result<(), ValidationError> {
        match self {
            Message::Arg { choices, .. } |
            Message::Mini { choices, .. } |
            Message::Micro { choices, .. } |
            Message::Select { choices, .. } => {
                if choices.len() > MAX_CHOICES {
                    return Err(ValidationError::TooManyChoices(choices.len()));
                }
            }
            _ => {}
        }
        Ok(())
    }
}
```

---

## Race Conditions Analysis

### Message Processing Thread Safety

The current architecture uses multiple threads:
1. **Reader thread** - reads from script stdout
2. **Writer thread** - writes to script stdin  
3. **UI thread** - handles messages via async_channel

**Findings:**

1. **Channel-based communication is safe:** The use of `async_channel::bounded(100)` provides back-pressure and prevents unbounded queue growth.

2. **Arc<Mutex<Option<ScriptSession>>>** provides safe shared access to the session state.

3. **Potential race in visibility tracking:**
   ```rust
   WINDOW_VISIBLE.store(false, Ordering::SeqCst);
   ```
   Uses `SeqCst` ordering which is safe but may be overly conservative.

4. **Response sender clone pattern is correct:**
   ```rust
   let reader_response_tx = response_tx.clone();
   ```
   Both reader and writer can send responses without data races.

**Assessment:** No significant race conditions identified. The thread safety patterns are appropriate.

---

## Recommendations Summary

### Immediate (High Priority)

1. **Implement message size limits** (IPC-001)
   - Add maximum line length check in `JsonlReader`
   - Default to 10 MB max message size

2. **Add field validation** (IPC-002, IPC-010)
   - Create `Message::validate()` method
   - Call after deserialization, before processing
   - Limit HTML to 1 MB, choices to 10,000

### Short-term (Medium Priority)

3. **Add rate limiting** (IPC-005)
   - Implement message coalescing for rapid updates
   - Add per-second rate limit (~100 msg/sec)

4. **Define typed data structures** (IPC-004)
   - Replace `serde_json::Value` with typed enums where possible
   - Reduces type confusion risks

### Long-term (Low Priority)

5. **Audit logging enhancement** (IPC-008)
   - Log security-relevant operations (clipboard writes, exec commands)
   - Enable forensic analysis

6. **Consider depth-limited deserialization** (IPC-007)
   - Protect against deeply nested JSON attacks

---

## Test Coverage Recommendations

Add the following test cases to ensure security robustness:

```rust
#[cfg(test)]
mod security_tests {
    use super::*;
    
    #[test]
    fn test_large_message_rejection() {
        let large_json = format!(
            r#"{{"type":"div","id":"1","html":"{}"}}"#,
            "x".repeat(100_000_000)  // 100 MB
        );
        // Should reject or handle gracefully
    }
    
    #[test]
    fn test_deeply_nested_json() {
        let mut nested = String::from(r#"{"type":"update","id":"1","data":"#);
        for _ in 0..1000 {
            nested.push_str(r#"{"a":"#);
        }
        nested.push_str("1");
        for _ in 0..1000 {
            nested.push('}');
        }
        nested.push('}');
        // Should handle without stack overflow
    }
    
    #[test]
    fn test_many_choices() {
        let choices: Vec<_> = (0..100_000)
            .map(|i| Choice::new(format!("Choice {}", i), i.to_string()))
            .collect();
        let msg = Message::arg("1".to_string(), "Pick".to_string(), choices);
        // Validation should reject
    }
    
    #[test]
    fn test_malformed_json_graceful() {
        let inputs = [
            "not json at all",
            "{}",  // Missing type
            r#"{"type":"unknown_future_type"}"#,
            r#"{"type":"arg"}"#,  // Missing required fields
            "",
            "\n\n\n",
        ];
        
        for input in inputs {
            let result = parse_message_graceful(input);
            // Should not panic, should return appropriate error
        }
    }
}
```

---

## Conclusion

The Script Kit GPUI IPC protocol implementation demonstrates solid Rust practices with strong typing and graceful error handling. The main security concerns relate to resource exhaustion attacks (large messages, many choices) rather than classic injection vulnerabilities.

The trust model assumes scripts are trusted code running on behalf of the user, which is appropriate for Script Kit's design. However, defense-in-depth measures for message size and rate limiting would improve resilience against buggy or malicious scripts.

**Risk Rating Justification:**
- No critical vulnerabilities (no RCE beyond intentional exec functionality)
- Medium-severity DoS vectors via resource exhaustion
- Good defensive coding practices already in place
- Protocol design is appropriate for use case

**Final Rating: Medium-High Risk** - Primarily due to DoS potential, mitigated by local-only nature of the protocol.
