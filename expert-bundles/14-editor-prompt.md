# Expert Question 14: Editor Prompt & Snippet System

## The Problem

Our editor prompt (1,489 LOC) handles code editing with snippet/template support. Snippets have tabstops with placeholders (`${1:default}`) and choice lists (`${1|opt1,opt2|}`). Tab/Shift+Tab navigates between tabstops.

## Specific Concerns

1. **Deferred Initialization**: EditorPrompt created without Window reference. InputState entity created on first render. If render called twice before init, second render gets None = crash.

2. **Char vs. Byte Offset Conversion**: `char_offset_to_byte_offset()` must handle UTF-8 multibyte characters. `unwrap_or(text.len())` is correct but easy to accidentally reverse.

3. **SnippetState Synchronization**: Maintains three parallel Vec<> (tabstops, current_values, last_selection_ranges). Out-of-sync = rendering bugs.

4. **Position Conversion Performance**: Iterates chars to find line/col. O(N) on every cursor move. No cached line starts.

5. **Subscription Leaks**: Subscriptions stored but never unsubscribed. Comment suggests `#[allow(dead_code)]` means subscriptions may leak.

## Questions for Expert

1. Is deferred initialization a code smell? Should we require Window at construction time?
2. What's the idiomatic Rust pattern for char↔byte↔position conversion that's both safe and efficient?
3. Should SnippetState be a single struct with invariants enforced, rather than parallel vectors?
4. How do real editors (Zed, VS Code) handle line/column position lookups efficiently?
5. What's the GPUI-idiomatic way to manage subscriptions lifecycle?

