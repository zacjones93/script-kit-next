# Expert Question 20: System Tray Integration

## The Problem

System tray (403 LOC) requires SVG → PNG → Icon pipeline. Menu items have inline SVG icons. Launch at Login toggle requires OS integration. macOS template images colorize automatically.

## Specific Concerns

1. **SVG Rendering Pipeline**: Embedded 32x32 SVG logo → rendered via usvg + tiny_skia → PNG RGBA bytes → `tray_icon::Icon`. Pipeline can fail silently.

2. **Template Image Mode**: On macOS, icon rendered as template image (alpha channel only). System colorizes based on menu bar appearance. Non-macOS needs different handling.

3. **Menu Icon SVGs**: 8 menu items with inline 16x16 SVG icons, each rendered independently. Lots of boilerplate for small icons.

4. **Menu Event Matching**: Menu item IDs are opaque strings captured at creation. No type safety for matching events to handlers.

5. **Launch at Login State**: CheckMenuItem toggles login item + updates visual checkmark. But OS can change login item externally; our UI doesn't sync.

## Questions for Expert

1. Is SVG → PNG → Icon the right pipeline, or should we use pre-rendered PNGs?
2. How do we handle template images correctly on non-macOS platforms?
3. Should menu items use an enum for type-safe ID matching instead of strings?
4. How do we keep Launch at Login checkbox in sync with OS state?
5. Is `tray_icon` crate the right choice, or are there better alternatives?

