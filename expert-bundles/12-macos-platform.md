# Expert Question 12: macOS Platform Integration

## The Problem

We use macOS-specific APIs for: floating panel windows, vibrancy/blur effects, accessibility permissions, and global hotkey registration. No cross-platform fallbacks exist.

## Specific Concerns

1. **Hardcoded Layout Constants**: `HEADER_PADDING_Y = 8px` vs `design_spacing.padding_md = 12px`. Values must match exactly but aren't derived from a single source.

2. **Vibrancy Effect**: `WindowBackgroundAppearance::Blurred` + semi-transparent colors combine to create macOS blur. No Windows/Linux equivalent.

3. **Accessibility Permissions**: Required for global hotkeys and clipboard monitoring. User must grant in System Preferences. No graceful degradation.

4. **Cursor Sizing**: Manual calculation assumes San Francisco font metrics. Custom fonts or different system font sizes break alignment.

5. **NSWindow Level**: We set `NSFloatingWindowLevel` via unsafe objc calls. Must be called after window creation but timing is finicky.

## Questions for Expert

1. What's the recommended pattern for macOS-specific code in a cross-platform Rust app?
2. Should we abstract platform-specific features behind a trait for future Windows/Linux support?
3. How do other apps handle accessibility permission requirements gracefully?
4. Is there a way to query actual font metrics from GPUI instead of hardcoding?
5. Are there crates that wrap NSWindow manipulation more safely than raw objc calls?

