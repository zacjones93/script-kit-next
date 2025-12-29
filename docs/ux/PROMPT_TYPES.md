# Prompt Types UX Audit

**Audit Date:** December 29, 2025  
**Auditor:** prompt-types-auditor  
**Scope:** Deep UX analysis of all prompt types in Script Kit GPUI

---

## Executive Summary

Script Kit GPUI implements **11 prompt types** across a modular architecture. The system demonstrates solid foundational patterns but has gaps in implementation completeness. Core prompts (arg, div, editor, term) are fully functional with consistent UX, while several SDK-advertised prompt types remain unimplemented stubs.

### Implementation Status Matrix

| Prompt Type | Implementation Status | UX Completeness | Priority |
|------------|----------------------|-----------------|----------|
| `arg()` | ✅ Complete | 95% | P0 |
| `div()` | ✅ Complete | 90% | P0 |
| `editor()` | ✅ Complete | 95% | P0 |
| `term()` | ✅ Complete | 90% | P0 |
| `form()` | ⚠️ Partial | 60% | P1 |
| `fields()` | ⚠️ Protocol only | 40% | P1 |
| `select()` | ⚠️ Protocol only | 40% | P1 |
| `mini()`/`micro()` | ⚠️ Protocol only | 30% | P2 |
| `chat()` | ⚠️ Protocol only | 20% | P2 |
| `path()` | ⚠️ Protocol only | 20% | P2 |
| `hotkey()` | ⚠️ Protocol only | 20% | P2 |
| `drop()` | ⚠️ Protocol only | 10% | P3 |
| `template()` | ⚠️ Protocol only | 20% | P2 |
| `env()` | ⚠️ Protocol only | 20% | P2 |
| `webcam()`/`mic()` | ⚠️ Protocol only | 10% | P3 |
| `widget()` | ⚠️ Protocol only | 10% | P3 |

---

## 1. ArgPrompt (arg())

**File:** `src/prompts.rs` (lines 31-401), `src/main.rs` (lines 5104-5346)

### Overview
The primary selection prompt with searchable choices. Fully implemented with virtualized scrolling.

### Features Analysis

| Feature | Status | Notes |
|---------|--------|-------|
| Choice list rendering | ✅ | Uses `uniform_list` for virtualization |
| Fuzzy filtering | ✅ | Real-time filter as user types |
| Keyboard navigation | ✅ | Up/Down arrows, Enter/Escape |
| Selection highlighting | ✅ | Accent bar + background color |
| Description support | ✅ | Secondary line for choice descriptions |
| Free text input | ✅ | Submits input when no choices match |
| Empty state | ✅ | "No choices match your filter" |
| Dynamic window resize | ✅ | Resizes based on choice count |

### Keyboard Handling

```
↑/↓ (ArrowUp/ArrowDown) → Navigate selection
Enter                    → Submit selected choice or typed text
Escape                   → Cancel and return to script list
Backspace                → Delete last character from filter
Any printable char       → Append to filter
```

**Critical Pattern:** Arrow key matching handles BOTH short (`"up"`) and long (`"arrowup"`) key names for cross-platform compatibility.

### UX Strengths
1. **Blinking cursor** - Visual feedback matches native inputs
2. **Choice count display** - Shows total choices in header
3. **Footer hints** - "↑↓ navigate • ⏎ select • Esc cancel"
4. **Virtualized scrolling** - Performance with large lists (100+ items)
5. **Scroll-to-selection** - Auto-scrolls to keep selection visible

### UX Issues

| Issue | Severity | Description |
|-------|----------|-------------|
| No fuzzy matching | Medium | Filter is substring-only, not fuzzy |
| No icons | Low | Choice icons not rendered |
| No keyboard shortcuts | Low | No Cmd+K or other power-user shortcuts |
| Fixed input position | Low | Input is at top, could be configurable |

### Recommendations
1. **Add fuzzy matching** - Use `nucleo` or similar for better matching
2. **Add choice icons** - Support `icon` field in Choice struct
3. **Add Cmd+A select all** - For filter text manipulation

---

## 2. DivPrompt (div())

**File:** `src/prompts.rs` (lines 403-529), `src/main.rs` (lines 5348-5428)

### Overview
HTML content display for informational or custom UI. Renders stripped text (HTML parsing not implemented).

### Features Analysis

| Feature | Status | Notes |
|---------|--------|-------|
| HTML content display | ⚠️ Partial | Strips tags, shows plain text |
| Tailwind styling | ❌ | Ignored - not parsed |
| Keyboard handling | ✅ | Enter/Escape to continue |
| Full-height layout | ✅ | Uses STANDARD_HEIGHT (500px) |

### Keyboard Handling

```
Enter   → Submit/Continue
Escape  → Cancel/Continue (same as Enter)
```

### UX Issues

| Issue | Severity | Description |
|-------|----------|-------------|
| No HTML rendering | High | HTML tags are stripped, not rendered |
| No Tailwind support | High | CSS classes ignored |
| No scrolling | Medium | Long content gets clipped |
| No markdown support | Medium | Common use case not supported |

### Recommendations
1. **Implement HTML renderer** - Use a markdown-to-HTML or HTML parser
2. **Add scrollable content area** - For long content
3. **Support basic styling** - Text colors, sizes, alignment

---

## 3. EditorPrompt (editor())

**File:** `src/editor.rs` (complete file - 1276 lines)

### Overview
Full-featured code editor with syntax highlighting. Most complete prompt implementation.

### Features Analysis

| Feature | Status | Notes |
|---------|--------|-------|
| Text editing | ✅ | Insert, delete, backspace |
| Cursor navigation | ✅ | Arrows, home/end, word movement |
| Selection | ✅ | Shift+arrows, Cmd+A |
| Clipboard | ✅ | Cmd+C/V/X copy/paste/cut |
| Undo/Redo | ✅ | Cmd+Z, Cmd+Shift+Z |
| Syntax highlighting | ✅ | Multi-language support |
| Line numbers | ✅ | Gutter with line numbers |
| Status bar | ✅ | Line/column info, language |
| Configurable font size | ✅ | From config.ts |

### Keyboard Handling

```
Cmd+Enter              → Submit content
Escape                 → Cancel
Cmd+Z / Cmd+Shift+Z    → Undo/Redo
Cmd+C/X/V              → Copy/Cut/Paste
Cmd+A                  → Select all
Arrow keys             → Navigation
Shift+Arrows           → Extend selection
Alt+Arrows             → Word navigation
Cmd+Arrows             → Line start/end
Home/End               → Line start/end
Tab                    → Insert 4 spaces
```

### UX Strengths
1. **Real code editor** - Not just a textarea
2. **Syntax highlighting** - Proper token coloring
3. **Undo stack** - 100 operations max
4. **Virtualized rendering** - `uniform_list` for lines
5. **Status bar feedback** - "Cmd+Enter to submit, Escape to cancel"

### UX Issues

| Issue | Severity | Description |
|-------|----------|-------------|
| No line wrapping | Medium | Long lines overflow |
| No find/replace | Medium | Common editor feature |
| Fixed tab size | Low | Always 4 spaces |
| No minimap | Low | Nice-to-have |

### Recommendations
1. **Add soft wrapping option** - For prose/markdown editing
2. **Add find (Cmd+F)** - Essential for longer content
3. **Configurable tab size** - From config.ts

---

## 4. TermPrompt (term())

**File:** `src/term_prompt.rs` (complete file - 1013 lines)

### Overview
Full terminal emulator with PTY support. Runs commands interactively.

### Features Analysis

| Feature | Status | Notes |
|---------|--------|-------|
| PTY integration | ✅ | Real pseudo-terminal |
| Command execution | ✅ | Optional initial command |
| ANSI colors | ✅ | Per-cell foreground/background |
| Text attributes | ✅ | Bold, underline |
| Cursor rendering | ✅ | Block cursor with inversion |
| Dynamic resize | ✅ | Terminal resizes with window |
| Ctrl+key support | ✅ | Ctrl+C, Ctrl+D, etc. |
| Special keys | ✅ | Function keys, arrows, etc. |
| Configurable font | ✅ | From config.ts |

### Keyboard Handling

```
Escape         → Cancel/Close
Ctrl+A-Z       → Control characters (SIGINT, EOF, etc.)
Enter          → Carriage return
Backspace      → Delete character
Tab            → Tab character
Arrow keys     → ANSI escape sequences
Function keys  → ANSI escape sequences
```

### UX Strengths
1. **True terminal emulation** - Runs real commands
2. **30fps refresh** - Smooth output display
3. **Batched cell rendering** - Performance optimization
4. **Conservative column calculation** - Prevents line wrapping issues

### UX Issues

| Issue | Severity | Description |
|-------|----------|-------------|
| No scrollback | Medium | Can't scroll to previous output |
| No copy/paste | Medium | Standard terminal operations |
| No selection | Medium | Can't select text |
| Exit handling | Low | Auto-submits on exit |

### Recommendations
1. **Add scrollback buffer** - Store history for scrolling
2. **Add mouse selection** - Click-drag to select
3. **Add Cmd+C/V** - Copy/paste support

---

## 5. FormPrompt (form())

**File:** `src/main.rs` (lines 5430-5526)

### Overview
HTML form with submit button. Currently renders stripped text only.

### Features Analysis

| Feature | Status | Notes |
|---------|--------|-------|
| Form rendering | ⚠️ Partial | Strips HTML, shows text |
| Submit button | ✅ | Styled button component |
| Enter to submit | ✅ | Keyboard shortcut |

### Current Implementation
- Renders stripped HTML text (same as DivPrompt)
- Adds a Submit button in footer
- Enter key submits "submitted" string

### UX Issues

| Issue | Severity | Description |
|-------|----------|-------------|
| No form fields | Critical | Cannot render actual form inputs |
| No field validation | High | No client-side validation |
| Static response | High | Always submits "submitted" |

### Recommendations
1. **Implement form field rendering** - Input, textarea, checkbox, etc.
2. **Add field values to submission** - JSON object with field values
3. **Add validation** - Required fields, patterns

---

## 6. FieldsPrompt (fields())

**File:** `src/protocol.rs` (lines 36-74, 736-742)

### Overview
Multiple input fields prompt. **Protocol defined but UI not implemented.**

### Protocol Definition
```rust
pub struct Field {
    pub name: String,
    pub label: Option<String>,
    pub field_type: Option<String>,  // "text", "password", etc.
    pub placeholder: Option<String>,
    pub value: Option<String>,
}

// Message variant
Fields { id: String, fields: Vec<Field> }
```

### Current Status
- Protocol message type exists
- Field struct defined with builder pattern
- **NO UI rendering implemented**

### Recommendations
1. **Create FieldsPrompt component** - Vertical stack of labeled inputs
2. **Support field types** - Text, password, number, etc.
3. **Tab navigation** - Move between fields

---

## 7. SelectPrompt (select())

**File:** `src/protocol.rs` (lines 724-731)

### Overview
Selection with optional multiple choice. **Protocol only.**

### Protocol Definition
```rust
Select {
    id: String,
    placeholder: String,
    choices: Vec<Choice>,
    multiple: Option<bool>,  // Enables multi-select
}
```

### Current Status
- Treated as ArgPrompt (single selection)
- **Multiple selection not implemented**

### Recommendations
1. **Add checkbox indicators** - For multi-select mode
2. **Add Space to toggle** - Standard multi-select pattern
3. **Show selection count** - "3 selected" in header

---

## 8. Mini/Micro Prompts (mini(), micro())

**File:** `src/protocol.rs` (lines 704-717)

### Overview
Compact arg prompt variants. **Protocol only - render as standard arg.**

### Protocol Definition
```rust
Mini { id, placeholder, choices }   // Compact display
Micro { id, placeholder, choices }  // Tiny display
```

### Current Status
- Messages defined but not differentiated
- Would render as standard ArgPrompt

### Recommendations
1. **Reduce padding** - For mini variant
2. **Single line only** - For micro variant
3. **Smaller font sizes** - Proportionally scaled

---

## 9. Chat Prompt (chat())

**File:** `src/protocol.rs` (line 807)

### Overview
Chat/conversation interface. **Protocol stub only.**

### Protocol Definition
```rust
Chat { id: String }
```

### Current Status
- Minimal protocol definition
- **NO UI implementation**

### Recommendations
1. **Design chat UI** - Message bubbles, input at bottom
2. **Add message history** - Store conversation context
3. **Stream support** - For AI responses

---

## 10-15. Other Prompts (Stub Status)

| Prompt | Protocol Definition | Status |
|--------|-------------------|--------|
| `path()` | Lines 756-763 | File picker - stub only |
| `drop()` | Line 767 | Drop zone - stub only |
| `hotkey()` | Lines 774-779 | Key capture - stub only |
| `template()` | Lines 786-789 | Template editor - stub only |
| `env()` | Lines 793-799 | Env var prompt - stub only |
| `webcam()` | Line 828 | Camera capture - stub only |
| `mic()` | Line 832 | Audio recording - stub only |
| `widget()` | Lines 818-823 | Custom HTML - stub only |

---

## Cross-Prompt Consistency Analysis

### Consistent Patterns ✅

1. **Keyboard conventions**
   - Escape always cancels
   - Enter always submits/continues
   - Arrow keys for navigation

2. **Focus management**
   - Each prompt tracks its own focus handle
   - Cursor blink synchronized across prompts

3. **Theme integration**
   - All prompts use design tokens
   - Consistent color palette
   - Shared typography settings

4. **Window resizing**
   - `ViewType` enum for prompt-specific heights
   - Dynamic resize based on content

5. **Response protocol**
   - All use `Message::Submit { id, value }`
   - Consistent null handling for cancel

### Inconsistent Patterns ⚠️

| Pattern | Issue | Prompts Affected |
|---------|-------|-----------------|
| Footer text | Varies in format | All prompts |
| Padding values | Minor inconsistencies | Div, Form |
| Scroll behavior | Some have scroll, some don't | Div vs Arg |
| Focus input enum | Some use `None` | Term, Editor, Div |

---

## Transition Between Prompts

### Flow Analysis

```
ScriptList → (run script) → ArgPrompt/DivPrompt/... → (submit) → Next prompt or Exit
              ↓
           execute_interactive()
              ↓
           handle_prompt_message()
              ↓
           AppView transition + window resize
```

### Transition Handling

| Transition | Method | Notes |
|------------|--------|-------|
| Script → Prompt | `handle_prompt_message()` | Async via channel |
| Prompt → Prompt | Script sends next message | Sequential |
| Prompt → Exit | `ScriptExit` message | Resets to ScriptList |
| Any → Cancel | `cancel_script_execution()` | Cleans up fully |

### UX During Transitions
- **No loading indicator** - Instant transition
- **Window resize** - Smooth via debounce
- **Focus management** - Auto-focused on transition
- **State reset** - Arg input cleared on new prompt

---

## Default Values and Placeholders

### ArgPrompt
- **Placeholder:** From script's `placeholder` parameter
- **Default value:** None (starts empty)
- **Choices:** From script's `choices` array

### EditorPrompt
- **Content:** Optional `content` parameter
- **Language:** Defaults to "typescript"
- **Default:** Empty editor if no content

### TermPrompt
- **Command:** Optional initial command
- **Default:** Interactive shell if none

### DivPrompt
- **HTML:** Required - no default
- **Tailwind:** Optional - currently ignored

---

## Validation Feedback

### Current State
- **No inline validation** - All validation is server-side
- **Error toasts** - Script errors shown via toast manager
- **No field-level errors** - Form fields have no error states

### Recommendations

1. **Add required field indicators** - * for required fields
2. **Add inline validation messages** - Below fields
3. **Add error styling** - Red borders on invalid fields
4. **Add success states** - Green checkmarks on valid

---

## Accessibility Analysis

| Aspect | Current State | Recommendation |
|--------|--------------|----------------|
| Keyboard navigation | ✅ Good | Add more shortcuts |
| Screen reader support | ❌ Missing | Add ARIA labels |
| Focus indicators | ⚠️ Partial | Add focus rings |
| Color contrast | ✅ Good | Verify WCAG AA |
| Reduced motion | ❌ Missing | Add preference |

---

## Performance Considerations

### Virtualization
- ✅ ArgPrompt uses `uniform_list`
- ✅ EditorPrompt uses `uniform_list`
- ❌ DivPrompt has no virtualization (could overflow)

### Rendering Efficiency
- ✅ Fixed item height (LIST_ITEM_HEIGHT)
- ✅ Terminal cell batching
- ⚠️ Editor re-highlights on every change (could cache)

---

## Recommendations Summary

### P0 - Critical
1. Implement HTML rendering for DivPrompt
2. Add fuzzy matching to ArgPrompt

### P1 - High Priority
1. Implement FieldsPrompt UI
2. Add multi-select to SelectPrompt
3. Add scrollback to TermPrompt

### P2 - Medium Priority
1. Implement path picker (PathPrompt)
2. Add find/replace to EditorPrompt
3. Implement mini/micro variants

### P3 - Lower Priority
1. Add webcam/mic prompts
2. Add widget/drop prompts
3. Add chat interface

---

## Appendix: AppView Enum

```rust
enum AppView {
    ScriptList,
    ActionsDialog,
    ArgPrompt { id, placeholder, choices },
    DivPrompt { id, html, tailwind },
    FormPrompt { id, html },
    TermPrompt { id, entity },
    EditorPrompt { id, entity, focus_handle },
    ClipboardHistoryView { entries, filter, selected_index },
    AppLauncherView { apps, filter, selected_index },
    WindowSwitcherView { windows, filter, selected_index },
    DesignGalleryView { filter, selected_index },
}
```

---

## Appendix: Protocol Message Types (Prompts)

```rust
// Core prompts
Arg { id, placeholder, choices }
Div { id, html, tailwind? }
Editor { id, content?, language?, on_init?, on_submit? }
Term { id, command? }
Form { id, html }
Fields { id, fields }
Select { id, placeholder, choices, multiple? }
Mini { id, placeholder, choices }
Micro { id, placeholder, choices }

// Input capture
Hotkey { id, placeholder? }
Path { id, start_path?, hint? }
Drop { id }
Template { id, template }
Env { id, key, secret? }

// Media
Chat { id }
Webcam { id }
Mic { id }
Widget { id, html, options? }
```
