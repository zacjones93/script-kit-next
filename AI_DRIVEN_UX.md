# AI-Driven UX for Script Kit GPUI

## Executive Summary

**Script Kit GPUI** is a complete rewrite of Script Kit into the GPUI framework, designed with **AI agents as first-class citizens**. Unlike traditional GUI applications that require mouse clicks and human interaction, Script Kit GPUI accepts **JSONL commands via stdin**, enabling AI agents to:

- **Execute scripts** without manual interaction
- **Drive UI prompts** programmatically (select choices, enter text, submit forms)
- **Capture screenshots** for visual verification
- **Receive structured logs** optimized for token efficiency (~67% reduction)
- **Run autonomous tests** with auto-submit capabilities

The architecture treats AI automation as a primary use case, not an afterthought. Every UI component, every protocol message, and every logging format is designed with machine-readable output and programmatic control in mind.

**Core Principle:** If an AI agent can't reliably drive it, it's not done.

---

## Quick Reference Card

| Action | Command |
|--------|---------|
| **Run a script** | `echo '{"type":"run","path":"/path/to/script.ts"}' \| ./target/debug/script-kit-gpui` |
| **Enable AI-compact logs** | `SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1` |
| **Show window** | `echo '{"type":"show"}' \| ./target/debug/script-kit-gpui` |
| **Hide window** | `echo '{"type":"hide"}' \| ./target/debug/script-kit-gpui` |
| **Set search filter** | `echo '{"type":"setFilter","text":"search"}' \| ./target/debug/script-kit-gpui` |
| **Capture screenshot** | SDK: `await captureScreenshot()` returns `{data, width, height}` |
| **Auto-submit for testing** | `AUTO_SUBMIT=true ./target/debug/script-kit-gpui` |
| **Visual test** | `./scripts/visual-test.sh tests/smoke/<test>.ts 3` |
| **Build app** | `cargo build` |
| **Run verification** | `cargo check && cargo clippy && cargo test` |

### Log Format Quick Reference

When `SCRIPT_KIT_AI_LOG=1`:
```
SS.mmm|L|C|message
```
- `SS.mmm` = Seconds.milliseconds in current minute
- `L` = Level: `i`(INFO), `w`(WARN), `e`(ERROR), `d`(DEBUG), `t`(TRACE)
- `C` = Category: `P`(POSITION), `U`(UI), `K`(KEY), `Z`(RESIZE), `X`(ERROR), ...

---

## Document Index

| Document | Description |
|----------|-------------|
| [Protocol Reference](./AI_DRIVEN_UX_PROTOCOL.md) | Complete JSONL protocol specification with 59+ message types, request/response patterns, and SDK integration |
| [Testing Guide](./AI_DRIVEN_UX_TESTING.md) | Autonomous testing framework with AUTO_SUBMIT, screenshot capture, JSONL results, and API coverage matrix |
| [Agent Patterns](./AI_DRIVEN_UX_PATTERNS.md) | Token-efficient logging, error handling strategies, verification patterns, and agent workflow decision trees |
| [Future Roadmap](./AI_DRIVEN_UX_ROADMAP.md) | Proposed protocol extensions: getState, inspectElement, batch commands, and semantic targeting |

---

## Getting Started for AI Agents

### Step 1: Build the Application

```bash
cd /Users/johnlindquist/dev/script-kit-gpui
cargo build
```

### Step 2: Run a Script via stdin Protocol

```bash
# CORRECT - Use stdin JSON protocol
echo '{"type": "run", "path": "'$(pwd)'/tests/smoke/hello-world.ts"}' | ./target/debug/script-kit-gpui

# WRONG - Command line args don't work!
./target/debug/script-kit-gpui tests/smoke/hello-world.ts  # THIS DOES NOTHING
```

### Step 3: Enable AI-Compact Logs

```bash
# Compact logs save ~67% tokens compared to standard format
echo '{"type": "run", "path": "..."}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
```

### Step 4: Capture Screenshots for Visual Verification

```typescript
// In your test script
import '../../scripts/kit-sdk';

const screenshot = await captureScreenshot();
console.error(`Screenshot: ${screenshot.width}x${screenshot.height}`);
// screenshot.data contains base64-encoded PNG
```

Or use the visual test script:
```bash
./scripts/visual-test.sh tests/smoke/test-editor-height.ts 3
# Output: .test-screenshots/test-editor-height-<timestamp>.png
```

### Step 5: Run Autonomous Tests

```bash
# Enable auto-submit for testing (prompts auto-complete)
AUTO_SUBMIT=true echo '{"type": "run", "path": "..."}' | ./target/debug/script-kit-gpui

# Run the test harness
bun run scripts/test-harness.ts
```

### Step 6: Verify Before Committing

```bash
# MANDATORY before every commit
cargo check && cargo clippy --all-targets -- -D warnings && cargo test
```

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                     AI Agent (Claude, GPT, etc.)                     │
│                                                                      │
│  • Reads documentation                                               │
│  • Generates JSONL commands                                          │
│  • Parses structured responses                                       │
│  • Captures/analyzes screenshots                                     │
└───────────────────────────────┬─────────────────────────────────────┘
                                │
                                │ stdin: JSONL commands
                                │ {"type":"run","path":"..."} 
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    Script Kit GPUI (Rust/GPUI)                       │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                    Protocol Layer                             │   │
│  │  • Parses stdin JSONL                                         │   │
│  │  • Routes to appropriate handlers                             │   │
│  │  • Outputs structured responses                               │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                │                                     │
│                                ▼                                     │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                    GPUI Renderer                              │   │
│  │  • Prompts: arg(), div(), editor(), fields(), etc.           │   │
│  │  • Theme-aware styling                                        │   │
│  │  • Focus management                                           │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                │                                     │
│                                ▼                                     │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                    Script Executor (Bun)                      │   │
│  │  • Runs TypeScript scripts with SDK preload                   │   │
│  │  • Bidirectional JSONL communication                          │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
└───────────────────────────────┬─────────────────────────────────────┘
                                │
                                │ stdout: JSONL responses
                                │ stderr: AI-compact logs (with SCRIPT_KIT_AI_LOG=1)
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        AI Agent Feedback Loop                        │
│                                                                      │
│  • Parse JSONL responses                                             │
│  • Decode screenshots (base64 PNG)                                   │
│  • Filter logs by category: grep '|Z|' for resize                    │
│  • Iterate on failures                                               │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Key Protocol Messages

### Running Scripts

```json
{"type": "run", "path": "/absolute/path/to/script.ts"}
```

### Prompt Types (Script → App)

| Type | Purpose | Response |
|------|---------|----------|
| `arg` | Single choice selection | `{"type":"submit","value":"..."}` |
| `div` | HTML/Markdown display | None (display only) |
| `editor` | Code/text editing | `{"type":"submit","value":"..."}` |
| `fields` | Multi-field forms | `{"type":"submit","value":["..."]}` |
| `select` | Multi-select | `{"type":"submit","value":["..."]}` |
| `path` | File/folder picker | `{"type":"submit","value":"/path"}` |
| `hotkey` | Key capture | `{"type":"submit","value":"cmd+k"}` |
| `term` | Terminal session | Process output |

### Screenshot Capture (Script → App)

```json
{"type": "captureScreenshot", "requestId": "req-001"}
```

**Response:**
```json
{
  "type": "screenshotResult",
  "requestId": "req-001",
  "data": "iVBORw0KGgo...",  // Base64 PNG
  "width": 800,
  "height": 600
}
```

See [Protocol Reference](./AI_DRIVEN_UX_PROTOCOL.md) for complete message catalog.

---

## Testing Capabilities

### Auto-Submit Mode

When `AUTO_SUBMIT=true`, prompts automatically submit after rendering:
- **arg()** - Submits first choice
- **editor()** - Submits initial content
- **fields()** - Submits default values

```bash
AUTO_SUBMIT=true echo '{"type":"run",...}' | ./target/debug/script-kit-gpui
```

### Test Result Format (JSONL)

```json
{"test": "arg-string-choices", "status": "running", "timestamp": "2025-12-27T..."}
{"test": "arg-string-choices", "status": "pass", "result": "Apple", "duration_ms": 45}
```

### API Coverage Matrix

The testing framework tracks coverage of all 59+ SDK methods. See [Testing Guide](./AI_DRIVEN_UX_TESTING.md#6-api-coverage-matrix) for the complete matrix.

---

## Token Efficiency

### Standard vs Compact Logs

| Format | Example | Characters |
|--------|---------|------------|
| Standard | `2025-12-27T15:22:13.150Z INFO script_kit_gpui::logging: Selected display origin=(0,0)` | 85 |
| Compact | `13.150\|i\|P\|Selected display origin=(0,0)` | 28 |
| **Savings** | | **67%** |

### Category Filter Examples

```bash
# Resize events only
echo '...' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1 | grep '|Z|'

# Errors only
echo '...' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1 | grep '|e|'

# UI rendering
echo '...' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1 | grep '|U|'
```

See [Agent Patterns](./AI_DRIVEN_UX_PATTERNS.md) for complete category reference and parsing examples.

---

## Future Extensions (Roadmap)

The [Future Roadmap](./AI_DRIVEN_UX_ROADMAP.md) proposes:

| Extension | Priority | Description |
|-----------|----------|-------------|
| `getState` | P0 | Query current UI state without modifying it |
| `inspectElement` | P0 | Get element properties by selector |
| `batch` | P1 | Send multiple commands atomically |
| `subscribe` | P1 | Stream events without polling |
| `semanticId` | P2 | Stable element identifiers |
| `accessibilityTree` | P2 | Traverse UI hierarchy |

---

## Contributing

### For Human Developers

1. Read [AGENTS.md](./AGENTS.md) for development guidelines
2. Follow TDD workflow: Write failing test → Implement → Verify
3. Run verification gate before committing

### For AI Agents

1. Initialize with `swarmmail_init()`
2. Query semantic memory for past learnings
3. Reserve files before editing
4. Report progress at 25/50/75%
5. Complete with `swarm_complete()` (not `hive_close`)

See [AGENTS.md](./AGENTS.md) for complete agent workflow protocol.

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-12-27 | Initial documentation suite: Protocol, Testing, Patterns, Roadmap |

---

## Quick Links

- **Main AGENTS.md**: [AGENTS.md](./AGENTS.md) - Development guidelines and agent protocol
- **Protocol Reference**: [AI_DRIVEN_UX_PROTOCOL.md](./AI_DRIVEN_UX_PROTOCOL.md)
- **Testing Guide**: [AI_DRIVEN_UX_TESTING.md](./AI_DRIVEN_UX_TESTING.md)
- **Agent Patterns**: [AI_DRIVEN_UX_PATTERNS.md](./AI_DRIVEN_UX_PATTERNS.md)
- **Future Roadmap**: [AI_DRIVEN_UX_ROADMAP.md](./AI_DRIVEN_UX_ROADMAP.md)
- **Test Scripts**: `tests/smoke/`, `tests/sdk/`, `tests/autonomous/`
- **Visual Testing**: `./scripts/visual-test.sh`

---

*Generated by AI Documentation Swarm | 2025-12-27*
