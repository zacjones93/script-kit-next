# Feature Bundle 23: Background Scripts System

## Goal

Create a system where users can define TypeScript files in `~/.scriptkit/kit/main/background/*.ts` that:
- Can be toggled on/off
- Receive app events: "clipboard changed", "app focused", "key pressed", etc.
- Run in the background without UI

## Current State

**Event Infrastructure Already Exists:**

1. **Clipboard Monitoring** (monitor.rs):
   - Polls every 500ms in background thread
   - Detects text and image changes
   - Ready to trigger scripts

2. **App Focus Tracking** (frontmost_app_tracker.rs):
   - NSWorkspace observer for app activation
   - Tracks last real app (excludes Script Kit)
   - Ready to trigger scripts

3. **Global Hotkeys** (hotkeys.rs):
   - Unified routing table
   - Can register arbitrary hotkeys
   - Ready to trigger scripts

4. **File Watchers** (watcher.rs):
   - Watches config, theme, scripts directories
   - Can add custom watch paths
   - Ready to trigger scripts

5. **System Appearance** (watcher.rs):
   - Polls appearance every 2 seconds
   - Detects light/dark mode changes

**What's Missing:**
- No event → script binding registry
- No background script loading mechanism
- No SDK APIs for scripts to register listeners
- No UI to toggle background scripts on/off
- No event data serialization to pass to scripts

## Proposed Architecture

### 1. Background Script Configuration

```typescript
// ~/.sk/kit/config.ts
export default {
  backgroundScripts: {
    enabled: true,
    scripts: [
      {
        path: "background/on-clipboard.ts",
        trigger: "clipboardChanged",
        enabled: true
      },
      {
        path: "background/on-app-focus.ts",
        trigger: "appFocused",
        filter: { bundleId: "com.apple.*" },
        enabled: true
      }
    ]
  }
}
```

### 2. Event Types

```typescript
type BackgroundEvent =
  | { type: "clipboardChanged"; text?: string; hasImage: boolean }
  | { type: "appFocused"; bundleId: string; name: string }
  | { type: "appDeactivated"; bundleId: string }
  | { type: "hotkeyPressed"; shortcut: string }
  | { type: "fileChanged"; path: string; event: "created" | "modified" | "deleted" }
  | { type: "appearanceChanged"; appearance: "light" | "dark" }
```

### 3. Script Interface

```typescript
// ~/.sk/kit/background/on-clipboard.ts
import "@scriptkit/sdk"

export const metadata = {
  name: "Clipboard Logger",
  trigger: "clipboardChanged",
  enabled: true
}

export default async function(event: ClipboardChangedEvent) {
  if (event.text?.includes("secret")) {
    await notify("Sensitive data detected in clipboard!")
  }
}
```

## Key Questions

1. **Script Lifecycle**: Should background scripts be long-running processes or spawned per-event? Long-running is more efficient but harder to manage.

2. **Event Filtering**: Should filtering happen in Rust (efficient) or in the script (flexible)? E.g., "only trigger for Safari focus"

3. **Concurrency**: What if clipboard changes faster than script can handle? Queue events? Drop old ones?

4. **Error Handling**: If a background script crashes, how do we notify the user? Auto-disable the script?

5. **SDK Design**: Should scripts export a default function, or use `onClipboardChanged()` registration?

## Implementation Steps

1. Add `BackgroundScriptConfig` to config.rs
2. Create `src/background_scripts/` module
3. Add event listeners to clipboard monitor, app tracker, etc.
4. Create script spawning logic (non-blocking)
5. Add toggle UI to main menu or system tray
6. Update SDK with background script types

