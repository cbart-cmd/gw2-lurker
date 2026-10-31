//! JSON-lines event storage (ADR-003) scoped to one game session (ADR-012).
//!
//! - One file per session: `history/<sesión>.jsonl`, append-only.
//! - Opening a session **discards the previous one**: the history only covers what was
//!   observed while the addon was running (ADR-012). This is deliberate, not a bug.
//! - A corrupt line never aborts a read: it is skipped and counted, because losing the
//!   whole history over one bad byte would be worse than losing one event.
//! - Files rotate at `MAX_BYTES` so a long session cannot grow without bound (RNF-03).

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use gw2lurker_core::diff::Event;

/// Current storage format version. Bump only with a migration in mind.
pub const FORMAT_VERSION: u8 = 1;

/// Rotation size for one session file (RNF-03: 2 MB).
pub const MAX_BYTES: u64 = 2 * 1024 * 1024;

/// One stored observation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
    /// Format version, so a future reader can reject what it does not understand.
    pub v: u8,
    /// When the event was observed.
    pub at: DateTime<Utc>,
    /// Match the event belongs to.
    pub match_id: String,
    /// The event itself, as produced by `core::diff`.
    pub event: Event,
}

impl Record {
    /// Builds a record with the current format version.
    pub fn new(at: DateTime<Utc>, match_id: impl Into<String>, event: Event) -> Self {
        Self {
            v: FORMAT_VERSION,
            at,
            match_id: match_id.into(),
            event,
        }
    }
}

/// Append-only writer for one session.
#[derive(Debug)]
pub struct SessionStore {
    dir: PathBuf,
    session: String,
    seq: u32,
    file: PathBuf,
    written: u64,
    max_bytes: u64,
    corrupt: usize,
}

impl SessionStore {
    /// Opens (and creates) the history directory for a new session.
    ///
    /// Any `*.jsonl` left by a previous session is removed (ADR-012): history is
    /// session-scoped, and keeping stale files around would only mislead the UI.
    pub fn open(dir: impl AsRef<Path>, session: &str) -> io::Result<Self> {
        let dir = dir.as_ref().to_path_buf();
        fs::create_dir_all(&dir)?;

        for entry in fs::read_dir(&dir)? {
            let path = entry?.path();
            if path.extension().is_some_and(|e| e == "jsonl") {
                fs::remove_file(&path)?;
            }
        }

        let file = dir.join(format!("{session}-{:03}.jsonl", 0));
        File::create(&file)?;

        Ok(Self {
            dir,
            session: session.to_string(),
            seq: 0,
            file,
            written: 0,
            max_bytes: MAX_BYTES,
            corrupt: 0,
        })
    }

    /// Overrides the rotation size (tests and configuration).
    pub fn with_max_bytes(mut self, max_bytes: u64) -> Self {
        self.max_bytes = max_bytes;
        self
    }

    /// Path of the file currently being appended to.
    pub fn current_file(&self) -> &Path {
        &self.file
    }

    /// Lines skipped as unreadable during the last read.
    pub fn corrupt_lines(&self) -> usize {
        self.corrupt
    }

    /// Appends one record, rotating first if the current file is full.
    pub fn append(&mut self, record: &Record) -> io::Result<()> {
        if self.written >= self.max_bytes {
            self.rotate()?;
        }

        let mut line = serde_json::to_string(record)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        line.push('\n');

        let mut file = OpenOptions::new().append(true).open(&self.file)?;
        file.write_all(line.as_bytes())?;
        self.written += line.len() as u64;
        Ok(())
    }

    fn rotate(&mut self) -> io::Result<()> {
        self.seq += 1;
        let next = self
            .dir
            .join(format!("{}-{:03}.jsonl", self.session, self.seq));
        self.file = next;
        File::create(&self.file)?;
        self.written = 0;
        Ok(())
    }

    /// Reads every record of the session, in file order.
    ///
    /// Corrupt lines and records from a newer format version are skipped and counted.
    pub fn read_all(&mut self) -> io::Result<Vec<Record>> {
        self.corrupt = 0;
        let mut out = Vec::new();

        let mut files: Vec<PathBuf> = fs::read_dir(&self.dir)?
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "jsonl"))
            .collect();
        files.sort();

        for path in files {
            let reader = BufReader::new(File::open(&path)?);
            for line in reader.lines() {
                let line = line?;
                if line.trim().is_empty() {
                    continue;
                }
                match serde_json::from_str::<Record>(&line) {
                    Ok(record) if record.v <= FORMAT_VERSION => out.push(record),
                    _ => self.corrupt += 1,
                }
            }
        }

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gw2lurker_core::diff::BaselineReason;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(tag: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("gw2lurker-{tag}-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn at(secs: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(1_800_000_000 + secs, 0).unwrap()
    }

    fn record(secs: i64, guild: &str) -> Record {
        Record::new(
            at(secs),
            "2-2",
            Event::Claim {
                objective: "38-20".to_string(),
                guild: guild.to_string(),
                at: at(secs),
            },
        )
    }

    #[test]
    fn t10_append_and_read_roundtrip() {
        let dir = temp_dir("roundtrip");
        let mut store = SessionStore::open(&dir, "sesion").unwrap();

        store.append(&record(1, "AAA")).unwrap();
        store.append(&record(2, "BBB")).unwrap();

        let back = store.read_all().unwrap();
        assert_eq!(back.len(), 2);
        assert_eq!(back[0], record(1, "AAA"));
        assert_eq!(back[1], record(2, "BBB"));
        assert_eq!(store.corrupt_lines(), 0);

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn adr003_corrupt_line_is_skipped_and_counted() {
        let dir = temp_dir("corrupt");
        let mut store = SessionStore::open(&dir, "sesion").unwrap();
        store.append(&record(1, "AAA")).unwrap();

        let mut file = OpenOptions::new()
            .append(true)
            .open(store.current_file())
            .unwrap();
        writeln!(file, "{{ esto no es json").unwrap();
        drop(file);

        let back = store.read_all().unwrap();
        assert_eq!(back.len(), 1, "el registro bueno sobrevive");
        assert_eq!(store.corrupt_lines(), 1);

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn rnf03_long_session_rotates_instead_of_growing() {
        let dir = temp_dir("rotate");
        let mut store = SessionStore::open(&dir, "sesion")
            .unwrap()
            .with_max_bytes(120);

        for i in 0..8 {
            store.append(&record(i, "AAA")).unwrap();
        }

        let files = fs::read_dir(&dir)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().is_some_and(|x| x == "jsonl"))
            .count();
        assert!(files > 1, "debe haber rotado, hay {files} fichero(s)");

        let back = store.read_all().unwrap();
        assert_eq!(back.len(), 8, "rotar no pierde registros");

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn adr012_opening_a_session_discards_the_previous_one() {
        let dir = temp_dir("newsession");
        let mut first = SessionStore::open(&dir, "ayer").unwrap();
        first.append(&record(1, "VIEJO")).unwrap();
        drop(first);

        let mut second = SessionStore::open(&dir, "hoy").unwrap();
        second.append(&record(2, "NUEVO")).unwrap();

        let back = second.read_all().unwrap();
        assert_eq!(back.len(), 1, "el historial anterior se descarta");
        assert_eq!(back[0], record(2, "NUEVO"));

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn records_from_a_future_format_are_ignored() {
        let dir = temp_dir("future");
        let mut store = SessionStore::open(&dir, "sesion").unwrap();

        let mut file = OpenOptions::new()
            .append(true)
            .open(store.current_file())
            .unwrap();
        writeln!(
            file,
            r#"{{"v":99,"at":"2026-10-31T12:00:00Z","match_id":"2-2","event":{{"Baseline":{{"reason":"SessionStart"}}}}}}"#
        )
        .unwrap();
        writeln!(
            file,
            r#"{{"v":1,"at":"2026-10-31T12:00:00Z","match_id":"2-2","event":{{"Baseline":{{"reason":"SessionStart"}}}}}}"#
        )
        .unwrap();
        drop(file);

        let back = store.read_all().unwrap();
        assert_eq!(back.len(), 1);
        assert_eq!(
            back[0].v, FORMAT_VERSION,
            "solo sobrevive la version conocida"
        );
        assert_eq!(store.corrupt_lines(), 1);

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn events_keep_their_shape_through_storage() {
        let dir = temp_dir("shape");
        let mut store = SessionStore::open(&dir, "sesion").unwrap();

        let events = [
            Event::Baseline {
                reason: BaselineReason::SessionStart,
            },
            Event::MatchChanged {
                old: "2-2".to_string(),
                new: "2-3".to_string(),
            },
            Event::Gap {
                from: at(0),
                to: at(120),
            },
        ];

        for (i, event) in events.iter().enumerate() {
            store
                .append(&Record::new(at(i as i64), "2-2", event.clone()))
                .unwrap();
        }

        let back = store.read_all().unwrap();
        let stored: Vec<Event> = back.into_iter().map(|r| r.event).collect();
        assert_eq!(stored, events);

        fs::remove_dir_all(&dir).ok();
    }
}
