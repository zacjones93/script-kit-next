# Expert Question 7: Stdin JSON Protocol Design

## The Problem

Our protocol has 59+ message variants for script ↔ app communication:

**Prompts (script → app):**
- `arg`, `div`, `editor`, `fields`, `form`, `path`, `drop`, `hotkey`, `term`, `chat`, `mic`, `webcam`

**Responses (app → script):**
- `submit`, `update` (live updates during editing)

**System control:**
- `exit`, `show`, `hide`, `setPosition`, `setSize`, `setFilter`, `setActions`

**State queries:**
- `getState`, `getSelectedText`, `captureScreenshot`, `getWindowBounds`, `clipboardHistory`

## Specific Concerns

1. **Enum Explosion**: 59+ variants means large match statements. Adding a new prompt type touches many files.

2. **No Versioning**: Adding new fields could break old scripts. We rely on serde's `#[serde(default)]` but this isn't explicit versioning.

3. **Semantic IDs**: For AI-driven UX, each prompt needs a stable UUID. Currently we generate these ad-hoc.

4. **Type Safety Across Boundary**: TypeScript SDK and Rust app must agree on message shapes. No shared schema.

5. **Bidirectional Complexity**: Some messages are request/response (getState), others are fire-and-forget (show/hide). Mixed paradigms.

## Questions for Expert

1. Should we split the monolithic enum into trait objects for better extensibility?
2. What's the right versioning strategy for a JSONL protocol?
3. Should we use a schema definition language (protobuf, JSON Schema, TypeSpec) as source of truth?
4. How do we handle backwards compatibility when removing deprecated variants?
5. Is request/response correlation (matching responses to requests) worth implementing?

