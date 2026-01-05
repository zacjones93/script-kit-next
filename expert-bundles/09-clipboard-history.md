# Expert Question 9: Clipboard History System

## The Problem

We maintain a clipboard history database with SQLite, including text and images. Three concurrent threads: Monitor (500ms polling), Prune (hourly), and Image prewarming—all racing to access shared SQLite DB.

## Specific Concerns

1. **Polling-Based Change Detection**: Compares `last_text` string equality each poll. No hash-based dedup for large text. Config allows up to 100MB clipboard entries.

2. **DB Lock Contention**: Three threads all call `get_connection()` → `Mutex::lock()` without backoff. Monitor thread on 500ms poll may block pruning.

3. **Image Storage**: Base64 encoding for SQLite storage (3x size overhead). PNG re-encoding for clipboard set operations.

4. **No Deduplication**: Each poll stores new entry even if content is identical to previous (only updates timestamp on copy-back).

5. **Oversized Text Handling**: Trimming runs once at init. If text grows beyond limit mid-session, entry is never trimmed until restart.

## Questions for Expert

1. Should we switch to event-based clipboard monitoring instead of polling? Platform-specific APIs?
2. Is SQLite the right choice, or should we use a simpler file-based store for images?
3. How should we handle DB lock contention? Connection pooling? Separate DBs per thread?
4. Should we hash-deduplicate clipboard entries? What hashing strategy for mixed text/images?
5. Is WAL mode + periodic VACUUM the right approach for this write-heavy workload?

