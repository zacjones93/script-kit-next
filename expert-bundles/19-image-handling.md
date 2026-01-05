# Expert Question 19: Image Encoding & Rendering

## The Problem

We handle images in clipboard history and app launcher icons. Dual format support: new PNG format (compressed) and legacy RGBA format (uncompressed). Images cached as `Arc<RenderImage>` for sharing.

## Specific Concerns

1. **Dual Format Support**: New `png:{base64}` format for compression (~90% smaller), legacy `rgba:{W}:{H}:{base64}` for backward compat. Must detect format by prefix.

2. **Lazy Decoding**: `decode_to_render_image()` expensive (PNG decompress + frame creation). Only done once per entry, cached in Arc. But when to evict cache?

3. **Dimension Extraction**: `get_png_dimensions()` reads PNG header only to avoid full decode. But header parsing has edge cases (interlaced PNGs, etc.).

4. **Hash Collision Risk**: `compute_image_hash()` only hashes first 1KB of pixels. Two different large images with same first 1KB will collide.

5. **Arc<RenderImage> Semantics**: RenderImage must be immutable + shareable, but GPUI frame semantics assume owned frames. Unclear if Arc sharing is safe long-term.

## Questions for Expert

1. Should we standardize on PNG format only and drop legacy RGBA support?
2. What's the right cache eviction strategy for decoded images? LRU? Weak references?
3. Is partial hashing (first 1KB) acceptable, or should we hash the entire image?
4. How do we handle image decoding errors gracefully (corrupted PNG, invalid base64)?
5. Is Arc<RenderImage> the right abstraction for GPUI, or should we use a different pattern?

