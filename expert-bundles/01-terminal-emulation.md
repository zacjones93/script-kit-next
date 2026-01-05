# Expert Question 1: Terminal Emulation & PTY Management

## The Problem

We have a built-in terminal emulator for running interactive scripts. It's ~3,200 LOC across two files and handles:
- Cell-based monospace grid rendering with ANSI colors
- Escape sequence parsing (CSI, OSC sequences)
- PTY size negotiation (SIGWINCH on resize)
- Mouse selection with multi-click detection
- Cursor positioning through UTF-8 boundaries
- 30fps refresh throttling

## Specific Concerns

1. **Multi-byte Character Handling**: Cursor positioning uses byte offsets but display uses character offsets. We've had bugs where multi-byte UTF-8 characters cause cursor drift.

2. **Refresh Timer vs Event-Driven**: We poll at 30fps (33ms) instead of event-driven updates. This burns CPU even when terminal is idle.

3. **Selection Persistence**: Selection state persists across re-renders but can get stale if terminal content scrolls. Should selection be line-based or character-offset-based?

4. **Bell Flash Animation**: We flash the terminal on bell character but the timing interacts poorly with refresh timer.

5. **OSC Sequence State Machine**: Title extraction from `OSC 0;title BEL` requires partial parse state between reads.

## Questions for Expert

1. Is our cell-based rendering approach correct for GPUI, or should we use a different model?
2. How do real terminal emulators (Alacritty, WezTerm) handle the byte-vs-char offset problem?
3. Should we switch to event-driven refresh? What's the tradeoff?
4. Is our escape sequence parser (nested match statements) idiomatic Rust, or should we use a parser combinator?
5. Any recommendations for test coverage of terminal edge cases?

