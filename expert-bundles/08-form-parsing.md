# Expert Question 8: Form Parsing & Field Rendering

## The Problem

We parse HTML form strings from user scripts and render them as native GPUI components. The parser is regex-based (~440 LOC) and the field renderer (~1,450 LOC) handles TextField, TextArea, and Checkbox.

## Specific Concerns

1. **Regex-Based HTML Parsing**: We use multiple independent regexes for `<input>`, `<textarea>`, `<select>` with document-order preservation via position tracking. This is fragile and misses edge cases.

2. **Label Association Logic**: Must match `for` attribute to ID, then fallback to name matching. Two separate regex patterns for labels with/without other attributes.

3. **Char vs. Byte Indexing**: Form fields track char indices but GPUI uses byte offsets for cursor positioning. We have 30 lines of index conversion helpers.

4. **Click-to-Position Approximation**: Hardcoded `TEXTFIELD_CHAR_WIDTH_PX=8.0` and `TEXTAREA_LINE_HEIGHT_PX=24.0` that may not match actual font metrics.

5. **Checkbox Value Collision**: Maps `checked` attribute to value "true", but this conflicts with user-provided `value` attributes.

## Questions for Expert

1. Should we use a real HTML parser (html5ever, scraper) instead of regex? What's the tradeoff for this limited use case?
2. Is there a better pattern for char↔byte index conversion that's less error-prone?
3. How should we handle font metrics for click-to-position? Query GPUI at runtime?
4. What's the right abstraction for heterogeneous form fields (TextField/TextArea/Checkbox)?
5. Should we validate form HTML against a schema before parsing?

