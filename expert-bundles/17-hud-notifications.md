# Expert Question 17: HUD Notifications

## The Problem

HUD notifications are separate NSWindow instances (740 LOC), not part of the main app window. Each HUD is created, tracked, and destroyed independently with platform-specific Cocoa calls.

## Specific Concerns

1. **Separate Window Instances**: Each HUD is its own NSWindow. Must coordinate creation, z-ordering, and cleanup across multiple simultaneous HUDs.

2. **Unsafe Cocoa Calls**: Heavy `msg_send!` usage for NSWindow methods (setLevel, setCollectionBehavior, orderFront, setIgnoresMouseEvents). Memory safety relies on correct calling conventions.

3. **Position Calculation**: Bottom-center of screen containing mouse, with vertical stacking offset when multiple HUDs present. Must account for display bounds.

4. **Timer + Cleanup Race**: Async timer schedules cleanup, but window handle scope and NSWindow release timing must coordinate. Race condition if HUD dismissed manually while timer pending.

5. **Click-Through Behavior**: `setIgnoresMouseEvents: true` means HUD is unclickable. Action button HUDs can't actually receive clicks (TODO in code).

## Questions for Expert

1. Is creating separate NSWindows the right approach, or should HUDs be rendered in a single overlay window?
2. How do we safely wrap Cocoa window manipulation without raw `msg_send!`?
3. What's the correct cleanup pattern for async-scheduled window destruction?
4. How do other apps implement clickable notifications with auto-dismiss?
5. Should we use macOS UserNotifications framework instead of custom HUD windows?

