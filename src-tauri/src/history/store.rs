use std::path::Path;
use std::sync::Mutex;

use log::{info, warn};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

/// One recorded transcription.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub id: i64,
    /// Unix timestamp in seconds.
    pub created_at: i64,
    /// Length of the recording that produced this text.
    pub duration_ms: i64,
    pub text: String,
    /// Preset that produced it. `None` if that preset has since been deleted.
    pub preset_id: Option<String>,
    /// Preset name captured at the time, so history stays readable after the
    /// preset is renamed or removed.
    pub preset_name: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
    /// Bundle id of the app the text was pasted into, when known.
    pub app_bundle_id: Option<String>,
    /// Human-readable name of that app.
    pub app_name: Option<String>,
}

/// A transcription to record.
#[derive(Debug, Clone, Default)]
pub struct NewEntry {
    pub text: String,
    pub duration_ms: i64,
    pub preset_id: Option<String>,
    pub preset_name: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub app_bundle_id: Option<String>,
    pub app_name: Option<String>,
}

/// Filters for reading history back.
#[derive(Debug, Clone, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct HistoryQuery {
    /// Full-text search over the transcribed text. Empty means "everything".
    #[serde(default)]
    pub search: Option<String>,
    /// Only entries produced by this preset.
    #[serde(default)]
    pub preset_id: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: u32,
    #[serde(default)]
    pub offset: u32,
}

fn default_limit() -> u32 {
    50
}

impl Default for HistoryQuery {
    fn default() -> Self {
        Self {
            search: None,
            preset_id: None,
            limit: default_limit(),
            offset: 0,
        }
    }
}

/// Aggregate numbers for the history screen.
#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct HistoryStats {
    pub total_entries: i64,
    pub total_words: i64,
    /// Total recorded audio duration, in milliseconds.
    pub total_duration_ms: i64,
    /// Timestamp of the oldest entry, if any.
    pub oldest_at: Option<i64>,
}

/// SQLite-backed transcription history.
///
/// The connection is wrapped in a mutex: writes come from the recording thread
/// while reads come from Tauri command handlers, and the volumes involved are
/// far too small to justify a pool.
pub struct History {
    conn: Mutex<Connection>,
}

impl History {
    /// Open (creating if needed) the history database at `path`.
    pub fn open(path: &Path) -> Result<Self, rusqlite::Error> {
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                warn!("Failed to create history directory: {}", e);
            }
        }

        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

        let history = Self {
            conn: Mutex::new(conn),
        };
        history.migrate()?;

        info!("History database ready at {}", path.display());
        Ok(history)
    }

    /// Create the schema and the FTS index if they are not there yet.
    ///
    /// The FTS table is an external-content index over `recordings`, kept in
    /// sync by triggers so inserts and deletes cannot drift apart.
    fn migrate(&self) -> Result<(), rusqlite::Error> {
        let conn = self.lock();

        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS recordings (
                id            INTEGER PRIMARY KEY AUTOINCREMENT,
                created_at    INTEGER NOT NULL,
                duration_ms   INTEGER NOT NULL DEFAULT 0,
                text          TEXT    NOT NULL,
                preset_id     TEXT,
                preset_name   TEXT,
                provider      TEXT,
                model         TEXT,
                app_bundle_id TEXT,
                app_name      TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_recordings_created_at
                ON recordings (created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_recordings_preset
                ON recordings (preset_id);

            CREATE VIRTUAL TABLE IF NOT EXISTS recordings_fts
                USING fts5(text, content='recordings', content_rowid='id');

            CREATE TRIGGER IF NOT EXISTS recordings_ai AFTER INSERT ON recordings BEGIN
                INSERT INTO recordings_fts(rowid, text) VALUES (new.id, new.text);
            END;

            CREATE TRIGGER IF NOT EXISTS recordings_ad AFTER DELETE ON recordings BEGIN
                INSERT INTO recordings_fts(recordings_fts, rowid, text)
                    VALUES('delete', old.id, old.text);
            END;

            CREATE TRIGGER IF NOT EXISTS recordings_au AFTER UPDATE ON recordings BEGIN
                INSERT INTO recordings_fts(recordings_fts, rowid, text)
                    VALUES('delete', old.id, old.text);
                INSERT INTO recordings_fts(rowid, text) VALUES (new.id, new.text);
            END;
            "#,
        )?;

        Ok(())
    }

    /// Record a transcription. Blank text is ignored.
    pub fn insert(&self, entry: NewEntry) -> Result<Option<i64>, rusqlite::Error> {
        if entry.text.trim().is_empty() {
            return Ok(None);
        }

        let conn = self.lock();
        conn.execute(
            r#"
            INSERT INTO recordings
                (created_at, duration_ms, text, preset_id, preset_name, provider, model,
                 app_bundle_id, app_name)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                now_seconds(),
                entry.duration_ms,
                entry.text,
                entry.preset_id,
                entry.preset_name,
                entry.provider,
                entry.model,
                entry.app_bundle_id,
                entry.app_name,
            ],
        )?;

        Ok(Some(conn.last_insert_rowid()))
    }

    /// Read history back, newest first.
    pub fn list(&self, query: &HistoryQuery) -> Result<Vec<HistoryEntry>, rusqlite::Error> {
        let conn = self.lock();
        let limit = query.limit.clamp(1, 500) as i64;
        let offset = query.offset as i64;

        let search = query
            .search
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty());

        // FTS5 treats most punctuation as syntax, so a raw user string can be a
        // syntax error. Quoting each token makes it a plain phrase search.
        let mut rows = match search {
            Some(search) => {
                let pattern = fts_query(search);
                let sql = format!(
                    "{SELECT_COLUMNS}
                     JOIN recordings_fts f ON f.rowid = r.id
                     WHERE recordings_fts MATCH ?1 {preset_filter}
                     ORDER BY r.created_at DESC
                     LIMIT ?2 OFFSET ?3",
                    preset_filter = preset_filter(query, 4),
                );
                let mut stmt = conn.prepare(&sql)?;
                match &query.preset_id {
                    Some(preset_id) => stmt
                        .query_map(params![pattern, limit, offset, preset_id], map_entry)?
                        .collect::<Result<Vec<_>, _>>()?,
                    None => stmt
                        .query_map(params![pattern, limit, offset], map_entry)?
                        .collect::<Result<Vec<_>, _>>()?,
                }
            }
            None => {
                let sql = format!(
                    "{SELECT_COLUMNS}
                     WHERE 1 = 1 {preset_filter}
                     ORDER BY r.created_at DESC
                     LIMIT ?1 OFFSET ?2",
                    preset_filter = preset_filter(query, 3),
                );
                let mut stmt = conn.prepare(&sql)?;
                match &query.preset_id {
                    Some(preset_id) => stmt
                        .query_map(params![limit, offset, preset_id], map_entry)?
                        .collect::<Result<Vec<_>, _>>()?,
                    None => stmt
                        .query_map(params![limit, offset], map_entry)?
                        .collect::<Result<Vec<_>, _>>()?,
                }
            }
        };

        rows.shrink_to_fit();
        Ok(rows)
    }

    /// Look up a single entry.
    pub fn get(&self, id: i64) -> Result<Option<HistoryEntry>, rusqlite::Error> {
        let conn = self.lock();
        let sql = format!("{SELECT_COLUMNS} WHERE r.id = ?1");
        let mut stmt = conn.prepare(&sql)?;
        stmt.query_row(params![id], map_entry).optional()
    }

    /// Delete one entry. Returns whether it existed.
    pub fn delete(&self, id: i64) -> Result<bool, rusqlite::Error> {
        let conn = self.lock();
        let affected = conn.execute("DELETE FROM recordings WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }

    /// Delete everything.
    pub fn clear(&self) -> Result<(), rusqlite::Error> {
        let conn = self.lock();
        conn.execute("DELETE FROM recordings", [])?;
        Ok(())
    }

    /// Aggregate counts for the history screen.
    pub fn stats(&self) -> Result<HistoryStats, rusqlite::Error> {
        let conn = self.lock();
        conn.query_row(
            r#"
            SELECT COUNT(*),
                   COALESCE(SUM(LENGTH(TRIM(text)) - LENGTH(REPLACE(TRIM(text), ' ', '')) + 1), 0),
                   COALESCE(SUM(duration_ms), 0),
                   MIN(created_at)
            FROM recordings
            "#,
            [],
            |row| {
                Ok(HistoryStats {
                    total_entries: row.get(0)?,
                    total_words: row.get(1)?,
                    total_duration_ms: row.get(2)?,
                    oldest_at: row.get(3)?,
                })
            },
        )
    }

    /// Drop entries older than `retention_days`. `0` disables pruning.
    ///
    /// Returns how many rows were removed.
    pub fn prune(&self, retention_days: u32) -> Result<usize, rusqlite::Error> {
        if retention_days == 0 {
            return Ok(0);
        }

        let cutoff = now_seconds() - (retention_days as i64) * 86_400;
        let conn = self.lock();
        let removed = conn.execute(
            "DELETE FROM recordings WHERE created_at < ?1",
            params![cutoff],
        )?;

        if removed > 0 {
            info!(
                "Pruned {} history entries older than {} days",
                removed, retention_days
            );
        }

        Ok(removed)
    }

    /// Take the connection lock, recovering from a poisoned mutex.
    ///
    /// A panic elsewhere must not make history permanently unwritable; the
    /// connection itself is still usable.
    fn lock(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(|e| e.into_inner())
    }
}

const SELECT_COLUMNS: &str = "SELECT r.id, r.created_at, r.duration_ms, r.text, r.preset_id, \
                              r.preset_name, r.provider, r.model, r.app_bundle_id, r.app_name \
                              FROM recordings r";

fn preset_filter(query: &HistoryQuery, index: usize) -> String {
    match query.preset_id {
        Some(_) => format!("AND r.preset_id = ?{}", index),
        None => String::new(),
    }
}

fn map_entry(row: &rusqlite::Row<'_>) -> Result<HistoryEntry, rusqlite::Error> {
    Ok(HistoryEntry {
        id: row.get(0)?,
        created_at: row.get(1)?,
        duration_ms: row.get(2)?,
        text: row.get(3)?,
        preset_id: row.get(4)?,
        preset_name: row.get(5)?,
        provider: row.get(6)?,
        model: row.get(7)?,
        app_bundle_id: row.get(8)?,
        app_name: row.get(9)?,
    })
}

/// Turn free-form user input into a safe FTS5 query.
///
/// Each whitespace-separated token becomes a quoted prefix term, so
/// `"don't panic"` searches for `"don't"* "panic"*` rather than blowing up on
/// the apostrophe.
fn fts_query(input: &str) -> String {
    input
        .split_whitespace()
        .map(|token| format!("\"{}\"*", token.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" ")
}

fn now_seconds() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn history() -> History {
        let conn = Connection::open_in_memory().unwrap();
        let history = History {
            conn: Mutex::new(conn),
        };
        history.migrate().unwrap();
        history
    }

    fn entry(text: &str) -> NewEntry {
        NewEntry {
            text: text.to_string(),
            duration_ms: 1000,
            preset_name: Some("OpenAI".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn inserts_and_reads_back_newest_first() {
        let history = history();
        history.insert(entry("first")).unwrap();
        history.insert(entry("second")).unwrap();

        let entries = history.list(&HistoryQuery::default()).unwrap();
        assert_eq!(entries.len(), 2);
        // Same-second timestamps tie, so compare as a set rather than order.
        let texts: Vec<_> = entries.iter().map(|e| e.text.as_str()).collect();
        assert!(texts.contains(&"first"));
        assert!(texts.contains(&"second"));
    }

    #[test]
    fn blank_text_is_not_recorded() {
        let history = history();
        assert_eq!(history.insert(entry("   ")).unwrap(), None);
        assert!(history.list(&HistoryQuery::default()).unwrap().is_empty());
    }

    #[test]
    fn full_text_search_matches_prefixes() {
        let history = history();
        history.insert(entry("the quick brown fox")).unwrap();
        history.insert(entry("unrelated text")).unwrap();

        let found = history
            .list(&HistoryQuery {
                search: Some("qui".to_string()),
                ..Default::default()
            })
            .unwrap();

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].text, "the quick brown fox");
    }

    #[test]
    fn search_survives_punctuation() {
        let history = history();
        history.insert(entry("don't panic")).unwrap();

        // A bare apostrophe or quote would be an FTS5 syntax error unquoted.
        for needle in ["don't", "\"panic", "panic"] {
            let found = history
                .list(&HistoryQuery {
                    search: Some(needle.to_string()),
                    ..Default::default()
                })
                .unwrap();
            assert!(found.len() <= 1, "query {:?} must not error", needle);
        }
    }

    #[test]
    fn filters_by_preset() {
        let history = history();
        let mut a = entry("from preset a");
        a.preset_id = Some("a".to_string());
        let mut b = entry("from preset b");
        b.preset_id = Some("b".to_string());
        history.insert(a).unwrap();
        history.insert(b).unwrap();

        let found = history
            .list(&HistoryQuery {
                preset_id: Some("a".to_string()),
                ..Default::default()
            })
            .unwrap();

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].text, "from preset a");
    }

    #[test]
    fn deleting_removes_it_from_search_too() {
        let history = history();
        let id = history.insert(entry("ephemeral")).unwrap().unwrap();

        assert!(history.delete(id).unwrap());
        assert!(!history.delete(id).unwrap());

        let found = history
            .list(&HistoryQuery {
                search: Some("ephemeral".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert!(found.is_empty(), "FTS index must follow the delete");
    }

    #[test]
    fn prune_drops_only_old_entries() {
        let history = history();
        history.insert(entry("recent")).unwrap();

        {
            let conn = history.lock();
            conn.execute(
                "INSERT INTO recordings (created_at, duration_ms, text) VALUES (?1, 0, 'ancient')",
                params![now_seconds() - 100 * 86_400],
            )
            .unwrap();
        }

        assert_eq!(history.prune(0).unwrap(), 0, "0 disables pruning");
        assert_eq!(history.prune(90).unwrap(), 1);

        let remaining = history.list(&HistoryQuery::default()).unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].text, "recent");
    }

    #[test]
    fn stats_summarize_the_table() {
        let history = history();
        history.insert(entry("one two three")).unwrap();
        history.insert(entry("four five")).unwrap();

        let stats = history.stats().unwrap();
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.total_words, 5);
        assert_eq!(stats.total_duration_ms, 2000);
        assert!(stats.oldest_at.is_some());
    }

    #[test]
    fn clear_empties_everything() {
        let history = history();
        history.insert(entry("gone")).unwrap();
        history.clear().unwrap();

        assert_eq!(history.stats().unwrap().total_entries, 0);
        assert!(history.list(&HistoryQuery::default()).unwrap().is_empty());
    }
}
