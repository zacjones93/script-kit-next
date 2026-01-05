# Feature Bundle 32: System Events for Scripts & Commands

## Goal

Enable scripts and built-in commands to trigger on macOS system events: sleep, wake, battery changes, network changes, display changes, and more.

## Current State

### What's Implemented
- **Appearance changes**: Polls `defaults read -g AppleInterfaceStyle` every 2s
- **Frontmost app tracking**: NSWorkspace observer for app activation
- **File watching**: notify crate for config/scripts/theme

### What's NOT Implemented
- Sleep/wake detection
- Battery/power state changes
- Network connectivity changes
- Display connect/disconnect
- Volume mount/unmount
- USB device events
- Bluetooth connection events
- Screen lock/unlock

## macOS System Event APIs

### NSWorkspace Notifications
```objc
// Available through NSWorkspaceNotificationCenter
NSWorkspaceWillSleepNotification
NSWorkspaceDidWakeNotification
NSWorkspaceWillPowerOffNotification
NSWorkspaceDidMountNotification
NSWorkspaceDidUnmountNotification
NSWorkspaceDidActivateApplicationNotification
NSWorkspaceActiveSpaceDidChangeNotification
NSWorkspaceScreensDidSleepNotification
NSWorkspaceScreensDidWakeNotification
```

### IOKit Power Management
```rust
// Sleep/wake events via IOKit
IORegisterForSystemPower()
kIOMessageSystemWillSleep
kIOMessageSystemHasPoweredOn
kIOMessageCanSystemSleep
```

### Network Reachability
```rust
// Network change detection
SCNetworkReachabilityCreateWithAddress()
SCNetworkReachabilitySetCallback()
```

### Battery State (sysinfo crate - ALREADY IN Cargo.toml)
```rust
use sysinfo::{System, Components};
// Battery percentage, charging status, time remaining
```

## Proposed Architecture

### 1. System Event Bus
```rust
pub enum SystemEvent {
    // Power
    WillSleep,
    DidWake,
    BatteryLevelChanged { percent: u8, charging: bool },
    PowerSourceChanged { on_battery: bool },

    // Display
    DisplayConnected { display_id: u32 },
    DisplayDisconnected { display_id: u32 },
    ScreenLocked,
    ScreenUnlocked,

    // Network
    NetworkChanged { connected: bool, interface: String },
    WifiChanged { ssid: Option<String> },

    // Storage
    VolumeMount { path: String, name: String },
    VolumeUnmount { path: String },

    // Apps
    AppLaunched { bundle_id: String },
    AppQuit { bundle_id: String },
    SpaceChanged { space_id: u32 },
}
```

### 2. Event → Script Binding
```typescript
// ~/.sk/kit/config.ts
export default {
  systemEvents: {
    onWake: "scripts/on-wake.ts",
    onSleep: "scripts/on-sleep.ts",
    onBatteryLow: {
      script: "scripts/battery-warning.ts",
      threshold: 20
    },
    onNetworkChange: "scripts/network-changed.ts",
    onDisplayConnect: "scripts/display-connected.ts",
  }
}
```

### 3. Built-in Commands as Triggers
```typescript
// Not just scripts - can trigger built-in commands
systemEvents: {
  onWake: {
    command: "clipboard-history",  // Built-in
    action: "clear-sensitive"      // Sub-action
  },
  onScreenLock: {
    command: "hide-all-windows"    // Built-in
  }
}
```

## Key Questions

1. **Event Granularity**: Should battery events fire on every 1% change, or only at thresholds (20%, 10%, 5%)?

2. **Blocking vs Async**: Should `onWillSleep` block sleep until script completes (with timeout)?

3. **Built-in Commands**: Which built-ins should be triggerable?
   - Clear clipboard history
   - Hide/show windows
   - Toggle features
   - Run maintenance tasks

4. **SDK API**: Should scripts be able to register their own event listeners?
   ```typescript
   onSystemEvent("wake", async () => { ... })
   ```

5. **Event Debouncing**: Network events can be noisy. What's the right debounce strategy?

## Implementation Checklist

- [ ] Add `sysinfo` battery monitoring (crate already available)
- [ ] Implement IOKit sleep/wake registration
- [ ] Add NSWorkspace notification observer
- [ ] Create SystemEventBus module
- [ ] Add config.ts event binding support
- [ ] Implement built-in command triggers
- [ ] Add event logging for debugging
- [ ] Create example system event scripts

