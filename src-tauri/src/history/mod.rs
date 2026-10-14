//! Transcription history.
//!
//! Every completed transcription is recorded here: the text, when it happened,
//! how long the recording was, which preset produced it, and which app it was
//! pasted into. Audio is deliberately **not** stored — only text.
//!
//! Backed by a single SQLite file with an FTS5 index over the text, so search
//! stays fast as the table grows.

mod store;

pub use store::{History, HistoryEntry, HistoryQuery, HistoryStats, NewEntry};
