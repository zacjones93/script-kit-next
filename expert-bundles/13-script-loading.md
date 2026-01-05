# Expert Question 13: Script/Scriptlet Loading & Caching

## The Problem

We load TypeScript scripts from `~/.sk/kit/scripts/` and scriptlets (markdown bundles) from `~/.sk/kit/snippets/`. Two-level cache: in-memory + file system mtime tracking. Scripts wrapped in `Arc<Script>` for zero-copy sharing.

## Specific Concerns

1. **Cache Staleness via mtime Only**: If file replaced with identical content (same mtime by coincidence), cache never invalidates. Content hash would be more reliable.

2. **No Arc Cycle Detection**: If scriptlet references another file's scriptlet, circular dependency creates memory leak (Arc::clone() never drops).

3. **Metadata Parsing Fragility**: Regex-based extraction of TS `export const metadata = {}` vs comment format `// Name: ...`. No schema validation.

4. **Full FS Scan on Load**: `read_scripts()` returns a list with no caching layer. Full filesystem scan on each call (expensive for large directories).

5. **Scheduled Script Staleness**: Scheduler stores `Arc<Script>` but doesn't re-load on script change. Scheduler keeps old Arc indefinitely.

## Questions for Expert

1. Should we use content hashing (SHA256) instead of mtime for cache invalidation?
2. How do we detect/prevent Arc cycles in a script dependency graph?
3. Should we use tree-sitter or a real TS parser for metadata extraction instead of regex?
4. What's the right caching strategy for script loading? LRU? Weak references?
5. How should scheduled tasks handle script updates? Re-read on each execution?

