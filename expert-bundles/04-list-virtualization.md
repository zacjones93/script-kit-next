# Expert Question 4: Variable-Height List Virtualization

## The Problem

We started with GPUI's `uniform_list` (fixed 52px rows) but needed:
- Section headers (24px) + script items (48px) = variable heights
- Manual scrollbar with fade-out animation
- Keyboard navigation that skips headers

We switched to `list()` which supports variable heights but requires manual height calculation and scroll offset tracking.

## Specific Concerns

1. **Scroll Math**: We manually calculate `visible_ratio = container_height / total_content_height` for scrollbar thumb size. This requires knowing total content height which means iterating all items.

2. **ListState Coordination**: `ListState` and `UniformListScrollHandle` have different APIs. When item count changes, we must call `list_state.reset(count)` or scrolling breaks.

3. **Section Header Navigation**: Arrow keys must skip headers. Current impl checks `is_header` and recursively calls move_up/down. Stack overflow possible with many consecutive headers?

4. **Scroll-to-Item Strategy**: `ScrollStrategy::Nearest` sometimes doesn't scroll enough when item is partially visible. `ScrollStrategy::Center` jumps too much.

5. **Performance**: 1000+ items with variable heights. Pre-calculating all heights defeats virtualization benefits.

## Questions for Expert

1. Is GPUI's `list()` the right tool, or should we roll our own virtualized list?
2. How do we avoid O(n) height calculation while supporting variable heights?
3. What's the right scroll strategy for keyboard navigation in a list?
4. Should we pre-flatten headers into the item list or keep them separate?
5. How does Zed's file tree handle variable-height items (files vs. directories)?

