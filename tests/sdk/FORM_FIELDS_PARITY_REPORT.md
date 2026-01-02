# Form & Fields SDK Parity Report

**Date:** 2025-01-02  
**Epic:** cell--9bnr5-mjxautpwamg  
**Status:** Complete - Critical Gap Identified

---

## Executive Summary

This report documents the SDK parity testing results for `form()` and `fields()` functions in Script Kit GPUI. Testing revealed a **critical implementation gap**: while the SDK correctly sends `Message::Fields` to the GPUI backend, **the backend does NOT handle this message type**, causing all `fields()` calls to fail with "Unhandled message type: Fields".

### Key Findings

| Aspect | Status |
|--------|--------|
| `fields()` SDK Implementation | **Working** - Sends correct message |
| `fields()` GPUI Backend | **NOT IMPLEMENTED** - Falls through to catch-all |
| `form()` SDK Implementation | **Working** - Sends correct message |
| `form()` GPUI Backend | **Working** - Parses HTML to native components |
| SDK Type Coverage | **Partial** - Missing 9 HTML5 input types |

---

## 1. Test Coverage Matrix

### All 14 HTML5 Input Types from Documentation

| # | Input Type | `form()` Parser | `fields()` FieldDef | Native GPUI Component | Test File |
|---|------------|-----------------|---------------------|----------------------|-----------|
| 1 | `text` | **Supported** | **Supported** | `FormTextField` | `test-fields-basic.ts`, `test-form-all-types.ts` |
| 2 | `password` | **Supported** | **Supported** | `FormTextField` (masked) | `test-fields-basic.ts`, `test-form-all-types.ts` |
| 3 | `email` | **Supported** | **Supported** | `FormTextField` | `test-fields-basic.ts`, `test-form-all-types.ts` |
| 4 | `number` | **Supported** | **Supported** | `FormTextField` | `test-fields-basic.ts`, `test-form-all-types.ts` |
| 5 | `date` | **Pass-through** | **Supported** | `FormTextField` | `test-fields-datetime.ts`, `test-form-all-types.ts` |
| 6 | `time` | **Pass-through** | **Supported** | `FormTextField` | `test-fields-datetime.ts`, `test-form-all-types.ts` |
| 7 | `datetime-local` | **Pass-through** | **MISSING** | `FormTextField` | `test-fields-datetime.ts`, `test-form-all-types.ts` |
| 8 | `month` | **Pass-through** | **MISSING** | `FormTextField` | `test-fields-datetime.ts`, `test-form-all-types.ts` |
| 9 | `week` | **Pass-through** | **MISSING** | `FormTextField` | `test-fields-datetime.ts`, `test-form-all-types.ts` |
| 10 | `url` | **Pass-through** | **Supported** | `FormTextField` | `test-form-specialized.ts`, `test-form-all-types.ts` |
| 11 | `search` | **Pass-through** | **MISSING** | `FormTextField` | `test-form-specialized.ts`, `test-form-all-types.ts` |
| 12 | `tel` | **Pass-through** | **Supported** | `FormTextField` | `test-form-specialized.ts`, `test-form-all-types.ts` |
| 13 | `color` | **Pass-through** | **Supported** | `FormTextField` (no picker) | `test-form-specialized.ts`, `test-form-all-types.ts` |
| 14 | `textarea` | **Supported** | N/A | `FormTextArea` | `test-form-specialized.ts`, `test-form-all-types.ts` |

### Additional Form Elements

| Element | `form()` Parser | Native Component | Notes |
|---------|-----------------|------------------|-------|
| `checkbox` | **Supported** | `FormCheckbox` | Boolean values |
| `select` | **Supported** | Future: dropdown | Currently parsed but minimal rendering |
| `radio` | **NOT Supported** | None | Not implemented |
| `range` | **NOT Supported** | None | Not implemented |
| `file` | **NOT Supported** | None | Not implemented |
| `hidden` | **Skipped** | N/A | Intentionally ignored |

---

## 2. SDK Type Definitions Analysis

### Current FieldDef Type (`scripts/kit-sdk.ts` line 22-28)

```typescript
export interface FieldDef {
  name: string;
  label: string;
  type?: 'text' | 'password' | 'email' | 'number' | 'date' | 'time' | 'url' | 'tel' | 'color';
  placeholder?: string;
  value?: string;
}
```

### Supported Types (9)
- `text`, `password`, `email`, `number`
- `date`, `time`
- `url`, `tel`, `color`

### Missing Types (9)
- `datetime-local` - Combined date+time picker
- `month` - Month/year picker
- `week` - Week picker  
- `range` - Slider input
- `search` - Search with clear button
- `hidden` - Hidden form field
- `file` - File upload
- `checkbox` - Boolean toggle
- `radio` - Radio button group

### Recommendation

Extend the `FieldDef` type union:

```typescript
type?: 
  // Basic
  | 'text' | 'password' | 'email' | 'number'
  // Date/Time (HTML5)
  | 'date' | 'time' | 'datetime-local' | 'month' | 'week'
  // Specialized
  | 'url' | 'tel' | 'search' | 'color'
  // Interactive
  | 'range' | 'checkbox' | 'file'
  // Hidden
  | 'hidden';
```

---

## 3. GPUI Backend Implementation Status

### Message Flow Analysis

```
SDK fields() call
    |
    v
Message::Fields { id, fields, actions }
    |
    v
execute_script.rs match statement
    |
    v
Falls through to "other" => UnhandledMessage  <-- CRITICAL GAP
```

### What's Missing

1. **No `ShowFields` variant in `PromptMessage` enum** (`src/main.rs`)
2. **No `Message::Fields` handler** in `execute_script.rs` match statement
3. **No `FieldsPrompt` GPUI view** (like `FormPrompt` exists)

### Comparison: `form()` vs `fields()`

| Step | `form()` | `fields()` |
|------|----------|------------|
| SDK sends message | `Message::Form` | `Message::Fields` |
| Backend match | `Message::Form { id, html, .. }` | **MISSING** |
| Prompt message | `PromptMessage::ShowForm` | **MISSING** |
| Handler case | `PromptMessage::ShowForm =>` | **MISSING** |
| GPUI View | `FormPrompt` | **MISSING** |
| Form parser | `form_parser::parse_form_html()` | N/A (uses Field structs) |

### form() Implementation (Working)

From `src/execute_script.rs`:
```rust
Message::Form { id, html, actions } => {
    Some(PromptMessage::ShowForm { id, html, actions })
}
```

From `src/prompt_handler.rs`:
```rust
PromptMessage::ShowForm { id, html, actions } => {
    // Parses HTML and renders native GPUI components
}
```

---

## 4. Form Parser Capabilities (`src/form_parser.rs`)

### Explicitly Supported

| Element | Type Attribute | Native Component |
|---------|---------------|------------------|
| `<input>` | `text` | `FormTextField` |
| `<input>` | `password` | `FormTextField` (masked) |
| `<input>` | `email` | `FormTextField` |
| `<input>` | `number` | `FormTextField` |
| `<input>` | `checkbox` | `FormCheckbox` |
| `<textarea>` | N/A | `FormTextArea` |
| `<select>` | N/A | Native select (limited) |

### Pass-through Types

These types are parsed and their `type` attribute is preserved, but they render as basic text fields:
- `date`, `time`, `datetime-local`, `month`, `week`
- `url`, `search`, `tel`, `color`

### Intentionally Skipped

- `<input type="hidden">` - Skipped (security/UX)
- `<input type="submit">` - Skipped (form has its own submit)
- `<input type="button">` - Skipped (not form data)

---

## 5. Test Files Created

| Test File | Purpose | Status |
|-----------|---------|--------|
| `test-fields-basic.ts` | Basic field types (text, password, email, number) | **Blocked** - fields() unimplemented |
| `test-fields-datetime.ts` | Date/time types (date, time, datetime-local, month, week) | **Blocked** - fields() unimplemented |
| `test-form-all-types.ts` | All 14 input types via form() HTML | **Working** |
| `test-form-specialized.ts` | Specialized types (url, search, tel, color, textarea) | **Working** |

### Test Results Summary

```
form() tests:     PASS - HTML parsing and rendering works
fields() tests:   BLOCKED - Backend doesn't handle Message::Fields
```

---

## 6. Parity Gaps and Severity

### Critical (Blocks Core Functionality)

| Gap | Impact | Priority |
|-----|--------|----------|
| `Message::Fields` not handled | `fields()` completely broken | **P0** |
| No `ShowFields` PromptMessage | Can't route to handler | **P0** |
| No `FieldsPrompt` view | Can't render fields | **P0** |

### Moderate (Missing Features)

| Gap | Impact | Priority |
|-----|--------|----------|
| SDK missing `datetime-local` type | TypeScript error for datetime-local fields | P1 |
| SDK missing `month`, `week` types | TypeScript error for date picker variants | P1 |
| SDK missing `search` type | TypeScript error for search inputs | P2 |
| SDK missing `range` type | Can't create slider inputs | P2 |
| SDK missing `checkbox` type | Must use form() for checkboxes | P2 |
| SDK missing `radio` type | Must use form() for radio groups | P2 |
| SDK missing `file` type | No file upload via fields() | P3 |

### Low (Cosmetic/Enhancement)

| Gap | Impact | Priority |
|-----|--------|----------|
| Color inputs render as text | No native color picker | P3 |
| Date inputs render as text | No native date picker | P3 |
| Range inputs not supported | Must use custom UI | P3 |

---

## 7. Recommended Implementation Path

### Phase 1: Core Fields Implementation (P0)

1. **Add `ShowFields` to `PromptMessage` enum** in `src/main.rs`:
   ```rust
   PromptMessage::ShowFields {
       id: String,
       fields: Vec<Field>,
       actions: Option<Vec<Action>>,
   }
   ```

2. **Add `Message::Fields` handler** in `src/execute_script.rs`:
   ```rust
   Message::Fields { id, fields, actions } => {
       Some(PromptMessage::ShowFields { id, fields, actions })
   }
   ```

3. **Create `FieldsPrompt` view** - Similar to `FormPrompt` but uses `Field` structs directly instead of HTML parsing

4. **Add handler case** in `src/prompt_handler.rs`:
   ```rust
   PromptMessage::ShowFields { id, fields, actions } => {
       // Render FieldsPrompt view
   }
   ```

### Phase 2: SDK Type Expansion (P1-P2)

1. Update `FieldDef.type` union in `scripts/kit-sdk.ts`
2. Add corresponding handlers for new types in GPUI

### Phase 3: Native Input Components (P3)

1. Native date picker component
2. Native time picker component
3. Native color picker component
4. Range slider component

---

## 8. Workarounds (Current)

Until `fields()` is implemented, use `form()` with HTML:

```typescript
// Instead of:
const result = await fields([
  { name: "email", label: "Email", type: "email" },
  { name: "password", label: "Password", type: "password" }
]);

// Use:
const result = await form(`
  <input type="email" name="email" placeholder="Email" />
  <input type="password" name="password" placeholder="Password" />
`);
```

### Limitations of Workaround

- No TypeScript type checking for field definitions
- Must write HTML manually
- Less structured API

---

## 9. Test Artifacts

Screenshots are saved to `.test-screenshots/` directory:
- `fields-basic-*.png` - Basic field type tests
- `fields-datetime-*.png` - Date/time field tests  
- `form-all-types-*.png` - All 14 types via form()
- `form-specialized-*.png` - Specialized input types

---

## 10. Conclusion

The `form()` function is **production-ready** with HTML parsing and native GPUI rendering. However, the `fields()` function is **completely broken** due to missing backend implementation. This represents a significant parity gap between the SDK API and the GPUI backend.

### Action Items

1. **Immediate**: Implement `Message::Fields` handler (blocks all fields() usage)
2. **Short-term**: Expand SDK `FieldDef` type union
3. **Medium-term**: Add native pickers for date/time/color
4. **Long-term**: Consider checkbox/radio/file support in fields()

---

*Report generated by swarm worker `cell--9bnr5-mjxautq8okn`*
