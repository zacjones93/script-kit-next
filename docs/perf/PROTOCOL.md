# Protocol Parsing & JSON Serialization Performance Audit

**Auditor:** ProtocolAuditor  
**Cell:** cell--9bnr5-mjqv2hnrair  
**Date:** 2024-12-29  
**Source:** `src/protocol.rs` (4,965 lines)

---

## Executive Summary

The protocol module handles JSONL-based bidirectional communication between Script Kit scripts and the GPUI application. With **59+ message types** and heavy use of `serde_json`, this module has several performance-critical paths that warrant attention.

### Key Findings

| Area | Severity | Impact |
|------|----------|--------|
| Large enum variant sizes | Medium | Memory overhead, stack usage |
| Base64 encoding in hot paths | High | CPU + memory for screenshots/images |
| String allocations in message construction | Medium | GC pressure, allocation overhead |
| Message type dispatch (match arms) | Low | Branch prediction, code size |
| Graceful parsing double-parse | Medium | 2x parse cost for unknown types |

---

## 1. Message Type Analysis

### 1.1 Enum Size Concern

The `Message` enum has 59+ variants with widely varying sizes:

```rust
#[serde(tag = "type")]
#[allow(clippy::large_enum_variant)]  // <-- Clippy warning suppressed!
pub enum Message {
    // Small variants (~48 bytes)
    Beep {},
    Show {},
    Hide {},
    
    // Medium variants (~100-200 bytes)
    Arg { id: String, placeholder: String, choices: Vec<Choice> },
    
    // Large variants (~300+ bytes)
    StateResult { /* 11 fields */ },
    SetError { /* 7 fields */ },
    ScriptletList { scriptlets: Vec<ScriptletData> },
}
```

**Problem:** The `#[allow(clippy::large_enum_variant)]` annotation indicates awareness of this issue but no mitigation. Every `Message` instance occupies memory equal to the largest variant.

**Size estimates:**
- `Beep {}` actual: ~8 bytes, allocated: ~400+ bytes
- `StateResult` with 11 fields: ~300-400 bytes
- `ScriptletList` with `Vec<ScriptletData>`: unbounded

### 1.2 Message Categories by Frequency

| Category | Message Count | Hot Path? | Notes |
|----------|--------------|-----------|-------|
| Core Prompts | 5 | Yes | `Arg`, `Div`, `Submit`, `Update`, `Exit` |
| Text Input | 3 | Yes | `Editor`, `Mini`, `Micro` |
| Selection | 1 | Yes | `Select` with `Vec<Choice>` |
| Form | 2 | Medium | `Fields`, `Form` |
| System Control | 10+ | Medium | `Clipboard`, `Keyboard`, `Mouse`, etc. |
| Response Types | 15+ | Yes | `*Result` messages sent back to scripts |
| Scriptlet | 4 | Low | `RunScriptlet`, `GetScriptlets`, etc. |

---

## 2. Parsing Hot Spots

### 2.1 Primary Parsing Functions

```rust
// Line 2147 - Simple parse (returns error on unknown type)
pub fn parse_message(line: &str) -> Result<Message, serde_json::Error> {
    serde_json::from_str(line)
}

// Line 2182 - Graceful parse (handles unknown types)
pub fn parse_message_graceful(line: &str) -> ParseResult {
    match serde_json::from_str::<Message>(line) {
        Ok(msg) => ParseResult::Ok(msg),
        Err(e) => {
            // DOUBLE PARSE: If first parse fails, parse again as Value
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(line) {
                // Extract type field...
            }
            ParseResult::ParseError(e)
        }
    }
}
```

**Problem:** `parse_message_graceful` performs **two parses** on unknown message types:
1. First attempt: `serde_json::from_str::<Message>(line)`
2. Fallback: `serde_json::from_str::<serde_json::Value>(line)`

This doubles CPU cost for malformed or unknown messages.

### 2.2 JsonlReader Hot Path

```rust
// Line 2280 - Used in main.rs line 1904
pub fn next_message_graceful(&mut self) -> Result<Option<Message>, std::io::Error> {
    loop {
        let mut line = String::new();  // ALLOCATION per line
        match self.reader.read_line(&mut line)? {
            // ...
            _ => {
                let trimmed = line.trim();
                match parse_message_graceful(trimmed) {
                    ParseResult::Ok(msg) => return Ok(Some(msg)),
                    ParseResult::UnknownType { .. } => continue,  // Skip, read next
                    ParseResult::ParseError(_) => continue,       // Skip, read next
                }
            }
        }
    }
}
```

**Allocations per message:**
1. `String::new()` for line buffer
2. Intermediate serde allocations during parsing
3. Final `Message` struct with owned Strings

---

## 3. Serialization Overhead

### 3.1 Serialize Function

```rust
// Line 2225
pub fn serialize_message(msg: &Message) -> Result<String, serde_json::Error> {
    serde_json::to_string(msg)
}
```

**Usage in hot paths:**

| Location | Context | Frequency |
|----------|---------|-----------|
| `main.rs:1852` | Response writer thread | Per response |
| `executor.rs:605` | Script session send | Per message |

### 3.2 Response Serialization Pattern

```rust
// main.rs:1849-1887 - Writer thread loop
loop {
    match response_rx.recv() {
        Ok(response) => {
            let json = protocol::serialize_message(&response)?;  // ALLOCATION
            let bytes = format!("{}\n", json);                   // ALLOCATION #2
            stdin.write_all(bytes.as_bytes())?;
            stdin.flush()?;
        }
        // ...
    }
}
```

**Issue:** Two allocations per response:
1. `serialize_message()` returns owned `String`
2. `format!("{}\n", json)` allocates again to add newline

---

## 4. Large Payload Handling

### 4.1 Screenshot Data (Base64)

```rust
// main.rs:2303-2319
if let Message::CaptureScreenshot { request_id } = &msg {
    // Capture to PNG
    let png_data: Vec<u8> = /* screenshot bytes */;
    
    // Base64 encode - MAJOR ALLOCATION
    use base64::Engine;
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&png_data);
    
    // ScreenshotResult message with large data field
    Message::screenshot_result(
        request_id.clone(),
        base64_data,  // Large string, potentially MB+
        width,
        height,
    )
}
```

**Size analysis:**
- 1920x1080 screenshot: ~8MB raw RGBA
- PNG compressed: ~1-3MB
- Base64 overhead: +33% → **~1.3-4MB string**

**Problem:** This large string is:
1. Allocated in memory
2. Serialized to JSON (quoted, escaped)
3. Written to pipe
4. Parsed by script (another copy)

### 4.2 Clipboard Image Data

```rust
// clipboard_history.rs:384-389
fn encode_image_as_base64(image: &arboard::ImageData) -> Result<String> {
    // Format: "rgba:{width}:{height}:{base64_data}"
    let base64_data = BASE64.encode(&image.bytes);
    Ok(format!("rgba:{}:{}:{}", image.width, image.height, base64_data))
}
```

Similar issue: clipboard images can be multi-MB base64 strings.

### 4.3 Clipboard History Truncation (Existing Mitigation)

```rust
// main.rs:1932-1947 - Truncation for large entries
let content = match e.content_type {
    ContentType::Image => {
        // For images, send a placeholder with metadata
        format!("[image:{}]", e.id)  // GOOD: Avoids sending MB of data
    }
    ContentType::Text => {
        // Truncate very long text entries
        if e.content.len() > 1000 {
            format!("{}...", &e.content[..1000])  // GOOD: Caps at 1KB
        } else {
            e.content
        }
    }
};
```

This is a **good pattern** that should be extended to other large payloads.

---

## 5. String Allocation in Message Construction

### 5.1 Constructor Methods

```rust
// Lines 1326-2137 contain 60+ constructor methods
impl Message {
    pub fn arg(id: String, placeholder: String, choices: Vec<Choice>) -> Self {
        Message::Arg { id, placeholder, choices }
    }
    
    pub fn clipboard_history_list_response(
        request_id: String,
        entries: Vec<ClipboardHistoryEntryData>,
    ) -> Self {
        Message::ClipboardHistoryList { request_id, entries }
    }
    // ... 58+ more constructors
}
```

**Observation:** All constructors take owned `String` values, forcing callers to allocate even if they have `&str`.

### 5.2 Clone Patterns

```rust
// main.rs:1962
Message::clipboard_history_list_response(request_id.clone(), entry_data)

// main.rs:1967
Message::clipboard_history_success(request_id.clone())

// Pattern repeats dozens of times
```

**Issue:** `request_id.clone()` on every response creates redundant String allocations.

---

## 6. Message Type Dispatch Efficiency

### 6.1 Large Match Statements

```rust
// Line 1364-1456 - Message::id() method
pub fn id(&self) -> Option<&str> {
    match self {
        Message::Arg { id, .. } => Some(id),
        Message::Div { id, .. } => Some(id),
        Message::Submit { id, .. } => Some(id),
        // ... 56 more arms
        Message::SetError { .. } => None,
    }
}
```

With 59+ arms, this generates a large jump table. Modern CPUs handle this efficiently, but code size is bloated.

### 6.2 Dispatch in Hot Path

```rust
// main.rs:2347-2383 - Message routing
let prompt_msg = match &msg {
    Message::Arg { id, placeholder, choices } => { ... }
    Message::Div { id, html, tailwind } => { ... }
    Message::Form { id, html } => { ... }
    Message::Term { id, command } => { ... }
    Message::Editor { id, content, language, .. } => { ... }
    Message::Exit { .. } => { ... }
    Message::ForceSubmit { value } => { ... }
    Message::Hide {} => { ... }
    Message::Browse { url } => { ... }
    _ => { /* Unhandled */ }
};
```

**Observation:** Only ~10 message types are handled in the main dispatch. The large enum is mostly unused in the hot path.

---

## 7. Optimization Recommendations

### 7.1 High Priority (P0)

#### A. Box Large Enum Variants

```rust
// Current (all variants same size)
pub enum Message {
    StateResult { /* 11 fields, ~400 bytes */ },
    ScriptletList { scriptlets: Vec<ScriptletData> },
    // ...
}

// Recommended (box large variants)
pub enum Message {
    StateResult(Box<StateResultData>),
    ScriptletList(Box<ScriptletListData>),
    // Small variants remain inline
    Beep {},
    Show {},
}
```

**Impact:** Reduces base enum size from ~400 bytes to ~40 bytes.

#### B. Streaming Base64 for Large Payloads

```rust
// Current - allocates full base64 string
let base64_data = BASE64.encode(&png_data);
Message::screenshot_result(request_id, base64_data, w, h)

// Recommended - stream directly to writer
struct StreamingScreenshot<'a> {
    request_id: &'a str,
    png_data: &'a [u8],
    width: u32,
    height: u32,
}

impl Serialize for StreamingScreenshot<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Custom serialization that base64-encodes during write
    }
}
```

**Impact:** Eliminates ~4MB allocation for large screenshots.

### 7.2 Medium Priority (P1)

#### C. Single-Parse Graceful Handling

```rust
// Current - double parse on unknown types
pub fn parse_message_graceful(line: &str) -> ParseResult {
    match serde_json::from_str::<Message>(line) {
        Ok(msg) => ParseResult::Ok(msg),
        Err(e) => {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(line) {
                // ...
            }
        }
    }
}

// Recommended - parse to Value first, then convert
pub fn parse_message_graceful(line: &str) -> ParseResult {
    let value: serde_json::Value = serde_json::from_str(line)?;
    let msg_type = value.get("type").and_then(|t| t.as_str());
    
    // Use from_value which is cheaper than re-parsing
    match serde_json::from_value::<Message>(value) {
        Ok(msg) => ParseResult::Ok(msg),
        Err(_) if msg_type.is_some() => ParseResult::UnknownType { ... },
        Err(e) => ParseResult::ParseError(e),
    }
}
```

**Impact:** Eliminates second parse for unknown message types.

#### D. Reuse Line Buffer in JsonlReader

```rust
// Current - allocates new String per line
pub fn next_message_graceful(&mut self) -> Result<Option<Message>, std::io::Error> {
    loop {
        let mut line = String::new();  // NEW ALLOCATION
        self.reader.read_line(&mut line)?;
        // ...
    }
}

// Recommended - reuse buffer
pub struct JsonlReader<R: Read> {
    reader: BufReader<R>,
    line_buffer: String,  // Reusable buffer
}

impl<R: Read> JsonlReader<R> {
    pub fn next_message_graceful(&mut self) -> Result<Option<Message>, std::io::Error> {
        loop {
            self.line_buffer.clear();  // Clear but keep capacity
            self.reader.read_line(&mut self.line_buffer)?;
            // ...
        }
    }
}
```

**Impact:** Eliminates String allocation per message in steady state.

### 7.3 Lower Priority (P2)

#### E. Accept `&str` in Constructors

```rust
// Current - requires owned String
pub fn arg(id: String, placeholder: String, choices: Vec<Choice>) -> Self

// Recommended - accept Into<String>
pub fn arg(id: impl Into<String>, placeholder: impl Into<String>, choices: Vec<Choice>) -> Self
```

**Impact:** Allows callers to pass `&str` when they have static strings.

#### F. Avoid Double Format in Writer

```rust
// Current - two allocations
let json = serialize_message(&response)?;  // Alloc #1
let bytes = format!("{}\n", json);          // Alloc #2

// Recommended - serialize directly to Vec<u8>
let mut bytes = serde_json::to_vec(&response)?;
bytes.push(b'\n');
stdin.write_all(&bytes)?;
```

**Impact:** Eliminates one allocation per response.

---

## 8. Benchmarking Recommendations

Before implementing optimizations, establish baselines:

```rust
#[cfg(test)]
mod benchmarks {
    use super::*;
    use std::time::Instant;
    
    #[test]
    fn bench_parse_arg_message() {
        let json = r#"{"type":"arg","id":"1","placeholder":"Pick","choices":[{"name":"A","value":"a"}]}"#;
        let start = Instant::now();
        for _ in 0..10000 {
            let _ = parse_message(json);
        }
        eprintln!("10k parses: {:?}", start.elapsed());
    }
    
    #[test]
    fn bench_serialize_state_result() {
        let msg = Message::state_result(/* ... */);
        let start = Instant::now();
        for _ in 0..10000 {
            let _ = serialize_message(&msg);
        }
        eprintln!("10k serializes: {:?}", start.elapsed());
    }
    
    #[test]
    fn bench_message_enum_size() {
        eprintln!("Message enum size: {} bytes", std::mem::size_of::<Message>());
        eprintln!("Choice size: {} bytes", std::mem::size_of::<Choice>());
        eprintln!("ScriptletData size: {} bytes", std::mem::size_of::<ScriptletData>());
    }
}
```

---

## 9. Summary

| Issue | Fix | Effort | Impact |
|-------|-----|--------|--------|
| Large enum variants | Box large variants | Medium | High - memory savings |
| Base64 allocation | Streaming encode | High | High - eliminates MB allocations |
| Double parse | Parse to Value first | Low | Medium - CPU savings |
| Line buffer reuse | Add buffer to reader | Low | Medium - reduces GC pressure |
| Constructor signatures | Accept `impl Into<String>` | Low | Low - ergonomics |
| Writer double alloc | Serialize to Vec | Low | Low - one less alloc |

**Recommended order:** P0-A, P1-C, P1-D, P0-B (if screenshots are slow)

---

## Appendix: Message Type Catalog

| Type | Direction | Fields | Size Est. |
|------|-----------|--------|-----------|
| `Arg` | Script→App | 3 | ~150B + choices |
| `Div` | Script→App | 3 | ~100B + html |
| `Submit` | App→Script | 2 | ~60B |
| `Editor` | Script→App | 5 | ~150B |
| `ScreenshotResult` | App→Script | 4 | ~100B + data (MB!) |
| `StateResult` | App→Script | 11 | ~400B |
| `ElementsResult` | App→Script | 3 | ~80B + elements |
| `ScriptletList` | App→Script | 2 | ~60B + scriptlets |
| `ClipboardHistoryList` | App→Script | 2 | ~60B + entries |

For complete message catalog, see `docs/PROTOCOL.md`.
