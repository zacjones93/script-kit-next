//! AI Chat Storage Layer
//!
//! SQLite-backed persistence for AI chats with CRUD operations and FTS5 search.
//! Follows the same patterns as src/notes/storage.rs for consistency.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use tracing::{debug, info};

use super::model::{Chat, ChatId, Message, MessageRole};

/// Global database connection for AI chats
static AI_DB: OnceLock<Arc<Mutex<Connection>>> = OnceLock::new();

/// Get the path to the AI chats database
fn get_ai_db_path() -> PathBuf {
    let kenv_dir = dirs::home_dir()
        .map(|h| h.join(".kenv"))
        .unwrap_or_else(|| PathBuf::from(".kenv"));

    kenv_dir.join("ai-chats.db")
}

/// Initialize the AI chats database
pub fn init_ai_db() -> Result<()> {
    let db_path = get_ai_db_path();

    // Ensure directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).context("Failed to create AI db directory")?;
    }

    let conn = Connection::open(&db_path).context("Failed to open AI chats database")?;

    // Create tables
    conn.execute_batch(
        r#"
        -- Chats table
        CREATE TABLE IF NOT EXISTS chats (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL DEFAULT 'New Chat',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            deleted_at TEXT,
            model_id TEXT NOT NULL,
            provider TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_chats_updated_at ON chats(updated_at DESC);
        CREATE INDEX IF NOT EXISTS idx_chats_deleted_at ON chats(deleted_at);
        CREATE INDEX IF NOT EXISTS idx_chats_provider ON chats(provider);

        -- Messages table
        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            chat_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL,
            tokens_used INTEGER,
            FOREIGN KEY (chat_id) REFERENCES chats(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_messages_chat_id ON messages(chat_id);
        CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages(created_at);

        -- Full-text search support for chats (searches titles and message content)
        CREATE VIRTUAL TABLE IF NOT EXISTS chats_fts USING fts5(
            title,
            content='chats',
            content_rowid='rowid'
        );

        -- Full-text search for messages
        CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
            content,
            content='messages',
            content_rowid='rowid'
        );

        -- Triggers to keep chat FTS in sync
        CREATE TRIGGER IF NOT EXISTS chats_ai AFTER INSERT ON chats BEGIN
            INSERT INTO chats_fts(rowid, title) 
            VALUES (NEW.rowid, NEW.title);
        END;

        CREATE TRIGGER IF NOT EXISTS chats_ad AFTER DELETE ON chats BEGIN
            INSERT INTO chats_fts(chats_fts, rowid, title) 
            VALUES('delete', OLD.rowid, OLD.title);
        END;

        CREATE TRIGGER IF NOT EXISTS chats_au AFTER UPDATE ON chats BEGIN
            INSERT INTO chats_fts(chats_fts, rowid, title) 
            VALUES('delete', OLD.rowid, OLD.title);
            INSERT INTO chats_fts(rowid, title) 
            VALUES (NEW.rowid, NEW.title);
        END;

        -- Triggers to keep message FTS in sync
        CREATE TRIGGER IF NOT EXISTS messages_ai AFTER INSERT ON messages BEGIN
            INSERT INTO messages_fts(rowid, content) 
            VALUES (NEW.rowid, NEW.content);
        END;

        CREATE TRIGGER IF NOT EXISTS messages_ad AFTER DELETE ON messages BEGIN
            INSERT INTO messages_fts(messages_fts, rowid, content) 
            VALUES('delete', OLD.rowid, OLD.content);
        END;

        CREATE TRIGGER IF NOT EXISTS messages_au AFTER UPDATE ON messages BEGIN
            INSERT INTO messages_fts(messages_fts, rowid, content) 
            VALUES('delete', OLD.rowid, OLD.content);
            INSERT INTO messages_fts(rowid, content) 
            VALUES (NEW.rowid, NEW.content);
        END;
        "#,
    )
    .context("Failed to create AI tables")?;

    info!(db_path = %db_path.display(), "AI chats database initialized");

    AI_DB
        .set(Arc::new(Mutex::new(conn)))
        .map_err(|_| anyhow::anyhow!("AI database already initialized"))?;

    Ok(())
}

/// Get a reference to the AI database connection
fn get_db() -> Result<Arc<Mutex<Connection>>> {
    AI_DB
        .get()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("AI database not initialized"))
}

// ============================================================================
// Chat Operations
// ============================================================================

/// Create a new chat
pub fn create_chat(chat: &Chat) -> Result<()> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    conn.execute(
        r#"
        INSERT INTO chats (id, title, created_at, updated_at, deleted_at, model_id, provider)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
        params![
            chat.id.as_str(),
            chat.title,
            chat.created_at.to_rfc3339(),
            chat.updated_at.to_rfc3339(),
            chat.deleted_at.map(|dt| dt.to_rfc3339()),
            chat.model_id,
            chat.provider,
        ],
    )
    .context("Failed to create chat")?;

    debug!(chat_id = %chat.id, title = %chat.title, "Chat created");
    Ok(())
}

/// Update an existing chat
pub fn update_chat(chat: &Chat) -> Result<()> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    conn.execute(
        r#"
        UPDATE chats 
        SET title = ?2, updated_at = ?3, deleted_at = ?4, model_id = ?5, provider = ?6
        WHERE id = ?1
        "#,
        params![
            chat.id.as_str(),
            chat.title,
            chat.updated_at.to_rfc3339(),
            chat.deleted_at.map(|dt| dt.to_rfc3339()),
            chat.model_id,
            chat.provider,
        ],
    )
    .context("Failed to update chat")?;

    debug!(chat_id = %chat.id, "Chat updated");
    Ok(())
}

/// Update chat title
pub fn update_chat_title(chat_id: &ChatId, title: &str) -> Result<()> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let now = Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE chats SET title = ?2, updated_at = ?3 WHERE id = ?1",
        params![chat_id.as_str(), title, now],
    )
    .context("Failed to update chat title")?;

    debug!(chat_id = %chat_id, title = %title, "Chat title updated");
    Ok(())
}

/// Get a chat by ID
pub fn get_chat(id: &ChatId) -> Result<Option<Chat>> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, title, created_at, updated_at, deleted_at, model_id, provider
            FROM chats
            WHERE id = ?1
            "#,
        )
        .context("Failed to prepare get_chat query")?;

    let result = stmt
        .query_row(params![id.as_str()], row_to_chat)
        .optional()
        .context("Failed to get chat")?;

    Ok(result)
}

/// Get all active chats (not deleted), sorted by updated_at desc
pub fn get_all_chats() -> Result<Vec<Chat>> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, title, created_at, updated_at, deleted_at, model_id, provider
            FROM chats
            WHERE deleted_at IS NULL
            ORDER BY updated_at DESC
            "#,
        )
        .context("Failed to prepare get_all_chats query")?;

    let chats = stmt
        .query_map([], row_to_chat)
        .context("Failed to query chats")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect chats")?;

    debug!(count = chats.len(), "Retrieved all chats");
    Ok(chats)
}

/// Get chats in trash (soft-deleted)
pub fn get_deleted_chats() -> Result<Vec<Chat>> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, title, created_at, updated_at, deleted_at, model_id, provider
            FROM chats
            WHERE deleted_at IS NOT NULL
            ORDER BY deleted_at DESC
            "#,
        )
        .context("Failed to prepare get_deleted_chats query")?;

    let chats = stmt
        .query_map([], row_to_chat)
        .context("Failed to query deleted chats")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect deleted chats")?;

    debug!(count = chats.len(), "Retrieved deleted chats");
    Ok(chats)
}

/// Soft delete a chat
pub fn delete_chat(chat_id: &ChatId) -> Result<()> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let now = Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE chats SET deleted_at = ?2, updated_at = ?2 WHERE id = ?1",
        params![chat_id.as_str(), now],
    )
    .context("Failed to soft delete chat")?;

    info!(chat_id = %chat_id, "Chat soft deleted");
    Ok(())
}

/// Restore a chat from trash
pub fn restore_chat(chat_id: &ChatId) -> Result<()> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let now = Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE chats SET deleted_at = NULL, updated_at = ?2 WHERE id = ?1",
        params![chat_id.as_str(), now],
    )
    .context("Failed to restore chat")?;

    info!(chat_id = %chat_id, "Chat restored from trash");
    Ok(())
}

/// Permanently delete a chat and all its messages
pub fn delete_chat_permanently(chat_id: &ChatId) -> Result<()> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    // Delete messages first (foreign key constraint)
    conn.execute(
        "DELETE FROM messages WHERE chat_id = ?1",
        params![chat_id.as_str()],
    )
    .context("Failed to delete chat messages")?;

    conn.execute("DELETE FROM chats WHERE id = ?1", params![chat_id.as_str()])
        .context("Failed to delete chat")?;

    info!(chat_id = %chat_id, "Chat permanently deleted");
    Ok(())
}

/// Search chats by title or message content
pub fn search_chats(query: &str) -> Result<Vec<Chat>> {
    if query.trim().is_empty() {
        return get_all_chats();
    }

    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    // Search in chat titles
    let mut stmt = conn
        .prepare(
            r#"
            SELECT DISTINCT c.id, c.title, c.created_at, c.updated_at, 
                   c.deleted_at, c.model_id, c.provider
            FROM chats c
            LEFT JOIN chats_fts fts ON c.rowid = fts.rowid
            LEFT JOIN messages m ON c.id = m.chat_id
            LEFT JOIN messages_fts mfts ON m.rowid = mfts.rowid
            WHERE c.deleted_at IS NULL 
              AND (chats_fts MATCH ?1 OR messages_fts MATCH ?1)
            ORDER BY c.updated_at DESC
            "#,
        )
        .context("Failed to prepare search query")?;

    let chats = stmt
        .query_map(params![query], row_to_chat)
        .context("Failed to search chats")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect search results")?;

    debug!(query = %query, count = chats.len(), "Chat search completed");
    Ok(chats)
}

// ============================================================================
// Message Operations
// ============================================================================

/// Save a message
pub fn save_message(message: &Message) -> Result<()> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    conn.execute(
        r#"
        INSERT INTO messages (id, chat_id, role, content, created_at, tokens_used)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        ON CONFLICT(id) DO UPDATE SET
            content = excluded.content,
            tokens_used = excluded.tokens_used
        "#,
        params![
            message.id,
            message.chat_id.as_str(),
            message.role.as_str(),
            message.content,
            message.created_at.to_rfc3339(),
            message.tokens_used,
        ],
    )
    .context("Failed to save message")?;

    // Update the chat's updated_at timestamp
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE chats SET updated_at = ?2 WHERE id = ?1",
        params![message.chat_id.as_str(), now],
    )
    .context("Failed to update chat timestamp")?;

    debug!(
        message_id = %message.id,
        chat_id = %message.chat_id,
        role = %message.role,
        "Message saved"
    );
    Ok(())
}

/// Get all messages for a chat, ordered by creation time
pub fn get_chat_messages(chat_id: &ChatId) -> Result<Vec<Message>> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, chat_id, role, content, created_at, tokens_used
            FROM messages
            WHERE chat_id = ?1
            ORDER BY created_at ASC
            "#,
        )
        .context("Failed to prepare get_chat_messages query")?;

    let messages = stmt
        .query_map(params![chat_id.as_str()], row_to_message)
        .context("Failed to query messages")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect messages")?;

    debug!(chat_id = %chat_id, count = messages.len(), "Retrieved chat messages");
    Ok(messages)
}

/// Get the last N messages for a chat
pub fn get_recent_messages(chat_id: &ChatId, limit: usize) -> Result<Vec<Message>> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, chat_id, role, content, created_at, tokens_used
            FROM messages
            WHERE chat_id = ?1
            ORDER BY created_at DESC
            LIMIT ?2
            "#,
        )
        .context("Failed to prepare get_recent_messages query")?;

    let mut messages: Vec<Message> = stmt
        .query_map(params![chat_id.as_str(), limit as i64], row_to_message)
        .context("Failed to query recent messages")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect recent messages")?;

    // Reverse to get chronological order
    messages.reverse();

    Ok(messages)
}

/// Get total token usage for a chat
pub fn get_chat_token_usage(chat_id: &ChatId) -> Result<u64> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let total: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(tokens_used), 0) FROM messages WHERE chat_id = ?1",
            params![chat_id.as_str()],
            |row| row.get(0),
        )
        .context("Failed to get token usage")?;

    Ok(total as u64)
}

/// Get chat count (active only)
pub fn get_chat_count() -> Result<usize> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM chats WHERE deleted_at IS NULL",
            [],
            |row| row.get(0),
        )
        .context("Failed to count chats")?;

    Ok(count as usize)
}

/// Prune chats deleted more than `days` ago
pub fn prune_old_deleted_chats(days: u32) -> Result<usize> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let cutoff = Utc::now() - chrono::Duration::days(days as i64);

    // Get IDs of chats to delete
    let chat_ids: Vec<String> = {
        let mut stmt = conn
            .prepare("SELECT id FROM chats WHERE deleted_at IS NOT NULL AND deleted_at < ?1")
            .context("Failed to prepare prune query")?;

        let results = stmt
            .query_map(params![cutoff.to_rfc3339()], |row| row.get(0))
            .context("Failed to query chats to prune")?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to collect chat IDs")?;
        results
    };

    // Delete messages for these chats
    for chat_id in &chat_ids {
        conn.execute("DELETE FROM messages WHERE chat_id = ?1", params![chat_id])
            .context("Failed to delete messages for pruned chat")?;
    }

    // Delete the chats
    let count = conn
        .execute(
            "DELETE FROM chats WHERE deleted_at IS NOT NULL AND deleted_at < ?1",
            params![cutoff.to_rfc3339()],
        )
        .context("Failed to prune old deleted chats")?;

    if count > 0 {
        info!(count, days, "Pruned old deleted chats");
    }

    Ok(count)
}

// ============================================================================
// Row Converters
// ============================================================================

/// Convert a database row to a Chat
fn row_to_chat(row: &rusqlite::Row) -> rusqlite::Result<Chat> {
    let id_str: String = row.get(0)?;
    let title: String = row.get(1)?;
    let created_at_str: String = row.get(2)?;
    let updated_at_str: String = row.get(3)?;
    let deleted_at_str: Option<String> = row.get(4)?;
    let model_id: String = row.get(5)?;
    let provider: String = row.get(6)?;

    let id = ChatId::parse(&id_str).unwrap_or_default();

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

    Ok(Chat {
        id,
        title,
        created_at,
        updated_at,
        deleted_at,
        model_id,
        provider,
    })
}

/// Convert a database row to a Message
fn row_to_message(row: &rusqlite::Row) -> rusqlite::Result<Message> {
    let id: String = row.get(0)?;
    let chat_id_str: String = row.get(1)?;
    let role_str: String = row.get(2)?;
    let content: String = row.get(3)?;
    let created_at_str: String = row.get(4)?;
    let tokens_used: Option<u32> = row.get(5)?;

    let chat_id = ChatId::parse(&chat_id_str).unwrap_or_default();

    let role = MessageRole::parse(&role_str).unwrap_or(MessageRole::User);

    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    Ok(Message {
        id,
        chat_id,
        role,
        content,
        created_at,
        tokens_used,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_path() {
        let path = get_ai_db_path();
        assert!(path.to_string_lossy().contains("ai-chats.db"));
    }
}
