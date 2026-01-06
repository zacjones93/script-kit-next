//! Notes Storage Layer
//!
//! SQLite-backed persistence for notes with CRUD operations.
//! Follows the same patterns as clipboard_history.rs for consistency.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use tracing::{debug, info};

use super::model::{Note, NoteId};

/// Global database connection for notes
static NOTES_DB: OnceLock<Arc<Mutex<Connection>>> = OnceLock::new();

/// Get the path to the notes database
fn get_notes_db_path() -> PathBuf {
    let kit_dir = dirs::home_dir()
        .map(|h| h.join(".scriptkit"))
        .unwrap_or_else(|| PathBuf::from(".scriptkit"));

    kit_dir.join("db").join("notes.sqlite")
}

/// Initialize the notes database
///
/// This function is idempotent - it's safe to call multiple times.
/// If the database is already initialized, it returns Ok(()) immediately.
pub fn init_notes_db() -> Result<()> {
    // Check if already initialized - this is the common case after first init
    if NOTES_DB.get().is_some() {
        debug!("Notes database already initialized, skipping");
        return Ok(());
    }

    let db_path = get_notes_db_path();

    // Ensure directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).context("Failed to create notes db directory")?;
    }

    let conn = Connection::open(&db_path).context("Failed to open notes database")?;

    // Enable WAL mode for better write performance
    conn.execute_batch("PRAGMA journal_mode=WAL;")
        .context("Failed to enable WAL mode")?;

    // Create tables
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS notes (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL DEFAULT '',
            content TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            deleted_at TEXT,
            is_pinned INTEGER NOT NULL DEFAULT 0,
            sort_order INTEGER NOT NULL DEFAULT 0
        );

        CREATE INDEX IF NOT EXISTS idx_notes_updated_at ON notes(updated_at DESC);
        CREATE INDEX IF NOT EXISTS idx_notes_deleted_at ON notes(deleted_at);
        CREATE INDEX IF NOT EXISTS idx_notes_is_pinned ON notes(is_pinned);

        -- Full-text search support
        CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(
            title,
            content,
            content='notes',
            content_rowid='rowid'
        );
        "#,
    )
    .context("Failed to create notes tables")?;

    // Drop old triggers if they exist (they fire on ALL updates)
    // and recreate them to only fire on title/content changes
    conn.execute_batch(
        r#"
        DROP TRIGGER IF EXISTS notes_ai;
        DROP TRIGGER IF EXISTS notes_ad;
        DROP TRIGGER IF EXISTS notes_au;

        -- Trigger for INSERT: always sync to FTS
        CREATE TRIGGER notes_ai AFTER INSERT ON notes BEGIN
            INSERT INTO notes_fts(rowid, title, content)
            VALUES (NEW.rowid, NEW.title, NEW.content);
        END;

        -- Trigger for DELETE: always sync to FTS
        CREATE TRIGGER notes_ad AFTER DELETE ON notes BEGIN
            INSERT INTO notes_fts(notes_fts, rowid, title, content)
            VALUES('delete', OLD.rowid, OLD.title, OLD.content);
        END;

        -- Trigger for UPDATE: only sync when title or content changes
        -- This prevents FTS churn when toggling pin or other metadata changes
        CREATE TRIGGER notes_au AFTER UPDATE OF title, content ON notes BEGIN
            INSERT INTO notes_fts(notes_fts, rowid, title, content)
            VALUES('delete', OLD.rowid, OLD.title, OLD.content);
            INSERT INTO notes_fts(rowid, title, content)
            VALUES (NEW.rowid, NEW.title, NEW.content);
        END;
        "#,
    )
    .context("Failed to create FTS triggers")?;

    info!(db_path = %db_path.display(), "Notes database initialized");

    // Use get_or_init pattern to handle race condition where another thread
    // might have initialized the DB between our check and set
    let _ = NOTES_DB.get_or_init(|| Arc::new(Mutex::new(conn)));

    Ok(())
}

/// Get a reference to the notes database connection
fn get_db() -> Result<Arc<Mutex<Connection>>> {
    NOTES_DB
        .get()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Notes database not initialized"))
}

/// Save a note (insert or update)
pub fn save_note(note: &Note) -> Result<()> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    conn.execute(
        r#"
        INSERT INTO notes (id, title, content, created_at, updated_at, deleted_at, is_pinned, sort_order)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ON CONFLICT(id) DO UPDATE SET
            title = excluded.title,
            content = excluded.content,
            updated_at = excluded.updated_at,
            deleted_at = excluded.deleted_at,
            is_pinned = excluded.is_pinned,
            sort_order = excluded.sort_order
        "#,
        params![
            note.id.as_str(),
            note.title,
            note.content,
            note.created_at.to_rfc3339(),
            note.updated_at.to_rfc3339(),
            note.deleted_at.map(|dt| dt.to_rfc3339()),
            note.is_pinned as i32,
            note.sort_order,
        ],
    )
    .context("Failed to save note")?;

    debug!(note_id = %note.id, title = %note.title, "Note saved");
    Ok(())
}

/// Get a note by ID
pub fn get_note(id: NoteId) -> Result<Option<Note>> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, title, content, created_at, updated_at, deleted_at, is_pinned, sort_order
            FROM notes
            WHERE id = ?1
            "#,
        )
        .context("Failed to prepare get_note query")?;

    let result = stmt
        .query_row(params![id.as_str()], row_to_note)
        .optional()
        .context("Failed to get note")?;

    Ok(result)
}

/// Get all active notes (not deleted), sorted by pinned first then updated_at desc
pub fn get_all_notes() -> Result<Vec<Note>> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, title, content, created_at, updated_at, deleted_at, is_pinned, sort_order
            FROM notes
            WHERE deleted_at IS NULL
            ORDER BY is_pinned DESC, updated_at DESC
            "#,
        )
        .context("Failed to prepare get_all_notes query")?;

    let notes = stmt
        .query_map([], row_to_note)
        .context("Failed to query notes")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect notes")?;

    debug!(count = notes.len(), "Retrieved all notes");
    Ok(notes)
}

/// Get notes in trash (soft-deleted)
pub fn get_deleted_notes() -> Result<Vec<Note>> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, title, content, created_at, updated_at, deleted_at, is_pinned, sort_order
            FROM notes
            WHERE deleted_at IS NOT NULL
            ORDER BY deleted_at DESC
            "#,
        )
        .context("Failed to prepare get_deleted_notes query")?;

    let notes = stmt
        .query_map([], row_to_note)
        .context("Failed to query deleted notes")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect deleted notes")?;

    debug!(count = notes.len(), "Retrieved deleted notes");
    Ok(notes)
}

/// Sanitize a query string for FTS5 MATCH
///
/// FTS5 special characters that need escaping: * " ' ( ) : - ^
/// We wrap the query in double quotes for phrase matching and escape internal quotes.
fn sanitize_fts_query(query: &str) -> String {
    let escaped = query.replace('"', "\"\"");
    format!("\"{}\"", escaped)
}

/// Search notes using full-text search
///
/// Uses FTS5 search when possible with a fallback to LIKE queries for robustness
/// against special characters that break FTS5 MATCH syntax.
pub fn search_notes(query: &str) -> Result<Vec<Note>> {
    if query.trim().is_empty() {
        return get_all_notes();
    }

    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    // Try FTS search first with sanitized query
    let sanitized_query = sanitize_fts_query(query);

    // FTS5 search with BM25 ranking
    let fts_result: rusqlite::Result<Vec<Note>> = (|| {
        let mut stmt = conn.prepare(
            r#"
            SELECT n.id, n.title, n.content, n.created_at, n.updated_at,
                   n.deleted_at, n.is_pinned, n.sort_order
            FROM notes n
            INNER JOIN notes_fts fts ON n.rowid = fts.rowid
            WHERE notes_fts MATCH ?1 AND n.deleted_at IS NULL
            ORDER BY bm25(notes_fts)
            "#,
        )?;

        let notes = stmt
            .query_map(params![sanitized_query], row_to_note)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(notes)
    })();

    match fts_result {
        Ok(notes) => {
            debug!(query = %query, count = notes.len(), method = "fts", "Note search completed");
            Ok(notes)
        }
        Err(e) => {
            // FTS failed (possibly due to special characters), fall back to LIKE search
            debug!(
                query = %query,
                error = %e,
                method = "like_fallback",
                "FTS search failed, using LIKE fallback"
            );

            let like_pattern = format!("%{}%", query);
            let mut stmt = conn
                .prepare(
                    r#"
                    SELECT id, title, content, created_at, updated_at,
                           deleted_at, is_pinned, sort_order
                    FROM notes
                    WHERE deleted_at IS NULL
                      AND (title LIKE ?1 OR content LIKE ?1)
                    ORDER BY updated_at DESC
                    "#,
                )
                .context("Failed to prepare LIKE fallback query")?;

            let notes = stmt
                .query_map(params![like_pattern], row_to_note)
                .context("Failed to execute LIKE fallback search")?
                .collect::<Result<Vec<_>, _>>()
                .context("Failed to collect LIKE fallback results")?;

            debug!(query = %query, count = notes.len(), method = "like_fallback", "Note search completed");
            Ok(notes)
        }
    }
}

/// Permanently delete a note
pub fn delete_note_permanently(id: NoteId) -> Result<()> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    conn.execute("DELETE FROM notes WHERE id = ?1", params![id.as_str()])
        .context("Failed to delete note")?;

    info!(note_id = %id, "Note permanently deleted");
    Ok(())
}

/// Prune notes deleted more than `days` ago
pub fn prune_old_deleted_notes(days: u32) -> Result<usize> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let cutoff = Utc::now() - chrono::Duration::days(days as i64);

    let count = conn
        .execute(
            "DELETE FROM notes WHERE deleted_at IS NOT NULL AND deleted_at < ?1",
            params![cutoff.to_rfc3339()],
        )
        .context("Failed to prune old deleted notes")?;

    if count > 0 {
        info!(count, days, "Pruned old deleted notes");
    }

    Ok(count)
}

/// Get total note count (active only)
pub fn get_note_count() -> Result<usize> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM notes WHERE deleted_at IS NULL",
            [],
            |row| row.get(0),
        )
        .context("Failed to count notes")?;

    Ok(count as usize)
}

/// Convert a database row to a Note
fn row_to_note(row: &rusqlite::Row) -> rusqlite::Result<Note> {
    let id_str: String = row.get(0)?;
    let title: String = row.get(1)?;
    let content: String = row.get(2)?;
    let created_at_str: String = row.get(3)?;
    let updated_at_str: String = row.get(4)?;
    let deleted_at_str: Option<String> = row.get(5)?;
    let is_pinned: i32 = row.get(6)?;
    let sort_order: i32 = row.get(7)?;

    let id = NoteId::parse(&id_str).unwrap_or_default();

    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    let deleted_at = deleted_at_str.and_then(|s| {
        DateTime::parse_from_rfc3339(&s)
            .map(|dt| dt.with_timezone(&Utc))
            .ok()
    });

    Ok(Note {
        id,
        title,
        content,
        created_at,
        updated_at,
        deleted_at,
        is_pinned: is_pinned != 0,
        sort_order,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_path() {
        let path = get_notes_db_path();
        assert!(path.to_string_lossy().contains("notes.sqlite"));
    }

    #[test]
    fn test_search_notes_handles_special_characters() {
        // Ensure DB is initialized
        let _ = init_notes_db();

        // Search with special characters should not error (even if no results)
        // These are FTS5 special characters that can break MATCH queries
        let special_queries = [
            "test@example.com", // @ symbol
            "foo*bar",          // wildcard
            "hello\"world",     // quote
            "foo:bar",          // colon (FTS column prefix syntax)
            "(test)",           // parentheses
            "test^2",           // caret (boost syntax)
            "test-query",       // hyphen (can be operator)
            "'test'",           // single quotes
            "test AND OR NOT",  // operators
        ];

        for query in special_queries {
            let result = search_notes(query);
            assert!(
                result.is_ok(),
                "Search with '{}' should not error: {:?}",
                query,
                result.err()
            );
        }
    }
}
