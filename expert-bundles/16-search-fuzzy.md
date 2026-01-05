# Expert Question 16: Search & Fuzzy Matching

## The Problem

We use multiple fuzzy matching backends: `nucleo` (fast, Unicode) and custom ASCII case-folding helpers. Search combines fuzzy scores with frecency decay for ranking.

## Specific Concerns

1. **Multiple Backends**: `nucleo` for Unicode-safe matching, custom `fuzzy_match_with_indices_ascii` for ASCII-only fast path. When to use which?

2. **Zero-Allocation ASCII Matching**: Byte-by-byte case-folding avoids heap allocation but assumes ASCII text. Non-ASCII chars break silently.

3. **Lazy Match Indices**: Match highlight indices only computed for visible rows (virtualized list), not during search phase. Stale if search changes mid-render.

4. **Complex Scoring Formula**: `score = fuzzy_score + count * e^(-days_since_use / half_life)`. Hard to tune, hard to explain to users.

5. **Spotlight Integration**: Uses macOS `mdfind` for file search with query escaping. Shell injection risk if escaping is wrong.

## Questions for Expert

1. Should we standardize on one fuzzy backend (nucleo everywhere) or keep the ASCII fast path?
2. How do we safely handle Unicode in fuzzy matching without performance regression?
3. Is lazy index computation the right approach, or should we compute indices upfront?
4. How should we expose scoring weights to users for customization?
5. Are there better alternatives to shelling out to `mdfind` for file search?

