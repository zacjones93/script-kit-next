# Expert Question 18: Frecency Scoring System

## The Problem

Frecency combines frequency + recency: `score = count * e^(-days_since_use / half_life)`. Used to rank scripts by usage patterns with exponential decay over time.

## Specific Concerns

1. **Exponential Decay Formula**: Half-life configurable (default 7 days). Old items decay toward zero but never fully disappear. Hard to explain to users why ordering changes.

2. **Cache Invalidation Bug**: When `record_use()` is called, the filter/group cache is NOT invalidated. Script list doesn't re-sort until next search. Known bug in comments.

3. **Timestamp Precision**: Uses Unix seconds. Can't track multiple uses within same second. Fine-grained usage patterns lost.

4. **Blocking I/O**: Entire frecency map saved synchronously on dirty write. No background save thread. Large files block main thread.

5. **Retroactive Half-Life Changes**: Changing half-life in config recalculates all scores retroactively. No way to preserve original decay curves.

## Questions for Expert

1. Is exponential decay the right model for frecency? Would linear decay or step functions be more predictable?
2. How do we properly invalidate caches when frecency changes without re-filtering everything?
3. Should frecency saves be async/batched to avoid blocking?
4. How do we handle the "rich get richer" problem where frequently used items dominate forever?
5. Should we expose frecency scores to users for transparency?

