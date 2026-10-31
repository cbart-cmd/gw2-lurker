//! On-disk cache for data that changes rarely (RF-20, RF-44).
//!
//! Objectives are cached **per language** so switching language (or holding right Ctrl,
//! RF-45) never needs the network, and guild names are cached so the follow list does not
//! hammer `/v2/guild/:id` every session.
//!
//! A corrupt or missing entry is simply a cache miss: it never aborts the caller.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use gw2lurker_core::model::ObjectiveInfo;

use crate::GuildInfo;

/// File-backed cache inside the addon directory.
#[derive(Debug, Clone)]
pub struct Cache {
    dir: PathBuf,
}

impl Cache {
    /// Creates the cache directory if needed.
    pub fn new(dir: impl AsRef<Path>) -> io::Result<Self> {
        let dir = dir.as_ref().to_path_buf();
        fs::create_dir_all(&dir)?;
        Ok(Self { dir })
    }

    /// Directory in use.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    fn objectives_path(&self, lang: &str) -> PathBuf {
        self.dir.join(format!("objectives-{lang}.json"))
    }

    fn guild_path(&self, id: &str) -> PathBuf {
        self.dir.join(format!("guild-{id}.json"))
    }

    /// Cached objectives for a language.
    pub fn objectives(&self, lang: &str) -> Option<Vec<ObjectiveInfo>> {
        read(&self.objectives_path(lang))
    }

    /// Stores objectives for a language.
    pub fn store_objectives(&self, lang: &str, values: &[ObjectiveInfo]) -> io::Result<()> {
        write(&self.objectives_path(lang), values)
    }

    /// Cached guild.
    pub fn guild(&self, id: &str) -> Option<GuildInfo> {
        read(&self.guild_path(id))
    }

    /// Stores a guild.
    pub fn store_guild(&self, guild: &GuildInfo) -> io::Result<()> {
        write(&self.guild_path(&guild.id), guild)
    }

    /// Removes every cached file (used when the match changes).
    pub fn clear(&self) -> io::Result<()> {
        for entry in fs::read_dir(&self.dir)? {
            let path = entry?.path();
            if path.extension().is_some_and(|e| e == "json") {
                fs::remove_file(path)?;
            }
        }
        Ok(())
    }
}

fn read<T: for<'de> serde::Deserialize<'de>>(path: &Path) -> Option<T> {
    let raw = fs::read(path).ok()?;
    serde_json::from_slice(&raw).ok()
}

fn write<T: serde::Serialize + ?Sized>(path: &Path, value: &T) -> io::Result<()> {
    let json = serde_json::to_vec(value).map_err(io::Error::other)?;
    fs::write(path, json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    const OBJECTIVES: &[u8] = include_bytes!("../tests/fixtures/objectives_all.json");
    const GUILD: &[u8] = include_bytes!("../tests/fixtures/guild.json");

    fn temp_dir(tag: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("gw2lurker-cache-{tag}-{nanos}"))
    }

    #[test]
    fn rf44_objectives_round_trip_per_language() {
        let dir = temp_dir("objectives");
        let cache = Cache::new(&dir).unwrap();
        let values: Vec<ObjectiveInfo> = serde_json::from_slice(OBJECTIVES).unwrap();

        cache.store_objectives("es", &values).unwrap();
        let back = cache.objectives("es").unwrap();
        assert_eq!(back.len(), values.len());
        assert_eq!(back[0], values[0]);

        assert!(
            cache.objectives("de").is_none(),
            "cada idioma tiene su propio fichero"
        );

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn rf20_guild_round_trip() {
        let dir = temp_dir("guild");
        let cache = Cache::new(&dir).unwrap();
        let guild: GuildInfo = serde_json::from_slice(GUILD).unwrap();

        cache.store_guild(&guild).unwrap();
        assert_eq!(cache.guild(&guild.id), Some(guild));

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn corrupt_cache_is_a_miss_not_an_error() {
        let dir = temp_dir("corrupt");
        let cache = Cache::new(&dir).unwrap();
        fs::write(dir.join("guild-X.json"), b"{ roto").unwrap();

        assert_eq!(cache.guild("X"), None);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn clear_removes_json_only() {
        let dir = temp_dir("clear");
        let cache = Cache::new(&dir).unwrap();
        let guild: GuildInfo = serde_json::from_slice(GUILD).unwrap();
        cache.store_guild(&guild).unwrap();
        fs::write(dir.join("notas.txt"), b"otra cosa").unwrap();

        cache.clear().unwrap();
        assert!(cache.guild(&guild.id).is_none());
        assert!(dir.join("notas.txt").exists());

        fs::remove_dir_all(&dir).ok();
    }
}
