# Feature Bundle 24: AI Chat Window Actions (Cmd+K)

## Goal

Implement the actions system in the AI Chat window so Cmd+K opens an actions dialog with context-aware actions for messages and chats.

## Current State

**UI Placeholder Exists:**
```rust
// ai/window.rs line 1534-1546
.child("Actions ⌘K") // placeholder for future actions menu
```

**Main Window Actions System is Complete:**
- ActionsDialog component with search/filter
- Keyboard navigation (arrows, enter, escape)
- SDK-provided actions via protocol
- Context-aware actions based on selection type

**What's Missing in AI Window:**
- No Cmd+K key handler
- No ActionsDialog integration
- No AI-specific action types defined
- No action handlers for messages/chats

## Proposed AI Actions

### Message-Level Actions
| Action | Shortcut | Description |
|--------|----------|-------------|
| Copy Message | ⌘C | Copy message content to clipboard |
| Copy as Markdown | ⌘⇧C | Copy with markdown formatting |
| Regenerate | ⌘R | Re-send prompt for new response |
| Edit & Resubmit | ⌘E | Edit user message and resubmit |
| Delete Message | ⌫ | Remove message from chat |
| Branch from Here | ⌘B | Create new chat starting from this point |

### Chat-Level Actions
| Action | Shortcut | Description |
|--------|----------|-------------|
| New Chat | ⌘N | Start a new conversation |
| Rename Chat | ⌘⇧R | Rename current chat |
| Delete Chat | ⌘⇧⌫ | Delete entire conversation |
| Export Chat | ⌘⇧E | Export as markdown/JSON |
| Search in Chat | ⌘F | Find text in conversation |

### Provider Actions
| Action | Shortcut | Description |
|--------|----------|-------------|
| Switch Model | ⌘M | Change AI model |
| View API Usage | ⌘U | Show token/cost stats |
| Configure API Key | - | Open API key settings |

## Key Questions

1. **Reuse vs. Fork**: Should we reuse `ActionsDialog` directly, or create `AiActionsDialog` variant? Main window actions are script-focused; AI actions are message-focused.

2. **Context Detection**: How do we know which message is "selected" for message-level actions? Hover state? Click state? Last interacted?

3. **Action Routing**: Main window routes to SDK or built-in handlers. AI window needs to route to message operations. Same pattern or different?

4. **Keyboard Conflicts**: Cmd+C already means copy. Should Cmd+K → actions be the only entry point, or add direct shortcuts too?

5. **State Management**: After "Regenerate", should we replace the response or append? After "Edit & Resubmit", delete subsequent messages?

## Implementation Steps

1. Add `AIAction` enum to `src/ai/actions.rs`:
   ```rust
   enum AIAction {
       CopyMessage(MessageId),
       Regenerate(MessageId),
       EditAndResubmit(MessageId),
       DeleteMessage(MessageId),
       NewChat,
       DeleteChat(ChatId),
       // ...
   }
   ```

2. Add key handler in `ai/window.rs`:
   ```rust
   "k" if modifiers.command => self.toggle_actions_dialog(cx),
   ```

3. Integrate ActionsDialog component:
   - Pass AI-specific actions based on context
   - Handle action selection
   - Route to message/chat operations

4. Add action handlers in `ai/window.rs`:
   - `handle_copy_message()`
   - `handle_regenerate()`
   - `handle_delete_chat()`
   - etc.

5. Update UI to show selection state for context

