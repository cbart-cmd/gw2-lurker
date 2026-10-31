//! Addon configuration, stored as JSON in the addon directory (RF-41).
//!
//! Read once on load and never inside the render callback. A broken file is renamed to
//! `config.json.corrupt-<fecha>` and the addon continues with defaults (the visible
//! notice is T-26 / RF-51).

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use chrono::Utc;
use gw2lurker_core::model::TeamColor;
use gw2lurker_core::FollowList;
use serde::{Deserialize, Serialize};

/// Where the alerts apply (RF-12).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertScope {
    /// Only the map the player is on.
    #[default]
    CurrentMap,
    /// Every map of the matchup.
    AllMaps,
}

/// Language preference (RF-44). `Auto` follows the game language.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LanguagePreference {
    #[default]
    Auto,
    Es,
    En,
    Fr,
    De,
}

/// Per-window UI state (RF-46).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WindowState {
    /// Whether the window is shown. Closing it hides it, it does not destroy it.
    pub visible: bool,
    /// Collapsed to its title bar.
    pub collapsed: bool,
    /// Locked: cannot be moved or resized.
    pub locked: bool,
    /// Background opacity, 0.0–1.0.
    pub opacity: f32,
    /// Last position, `None` until the window has been drawn once.
    pub position: Option<[f32; 2]>,
    /// Last size, `None` until the window has been drawn once.
    pub size: Option<[f32; 2]>,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            visible: true,
            collapsed: false,
            locked: false,
            opacity: 0.9,
            position: None,
            size: None,
        }
    }
}

impl WindowState {
    fn sanitize(&mut self) {
        self.opacity = if self.opacity.is_finite() {
            self.opacity.clamp(0.0, 1.0)
        } else {
            WindowState::default().opacity
        };
        if let Some(position) = self.position {
            if !position[0].is_finite() || !position[1].is_finite() {
                self.position = None;
            }
        }
        if let Some(size) = self.size {
            if !size[0].is_finite() || !size[1].is_finite() {
                self.size = None;
            } else {
                self.size = Some([size[0].max(120.0), size[1].max(60.0)]);
            }
        }
    }
}

/// Everything the addon remembers between runs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Configured team id (`11xxx`/`12xxx`), `None` until the user picks one (RF-03a).
    pub team_id: Option<u32>,
    /// Name the user wrote for their team, per language code (RF-03f).
    ///
    /// Prefilled with the name in the game language, so a wrong or missing translation can be
    /// fixed without waiting for a release. Stored per language on purpose: switching the game
    /// language must not show the Spanish name over the French one.
    pub team_names: BTreeMap<String, String>,
    /// Single alias of 0.5.x, migrated into `team_names` on first save.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team_alias: Option<String>,
    /// Polling interval in seconds; the measured default is 30 (RNF-02).
    pub poll_secs: u32,
    /// Where alerts apply.
    pub alert_scope: AlertScope,
    /// Language preference.
    pub language: LanguagePreference,
    /// Guilds marked as favourites (RF-73).
    pub followed: FollowList,
    /// Widget opacity, 0.0–1.0 (RF-46).
    pub widget_opacity: f32,
    /// Whether the widget is locked (RF-18).
    pub widget_locked: bool,
    /// Per-window UI state, keyed by the window's English title (RF-46).
    pub windows: BTreeMap<String, WindowState>,
    /// One team per character (RF-03a); the manual `team_id` wins over it.
    pub roster: gw2lurker_core::Roster,
    /// Draws the overlay outside WvW too. Diagnostic only: `false` in normal use (RF-01).
    pub debug_show_everywhere: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            team_id: None,
            team_alias: None,
            team_names: BTreeMap::new(),
            poll_secs: 30,
            alert_scope: AlertScope::default(),
            language: LanguagePreference::default(),
            followed: FollowList::default(),
            widget_opacity: 0.85,
            widget_locked: true,
            windows: BTreeMap::new(),
            roster: gw2lurker_core::Roster::default(),
            debug_show_everywhere: false,
        }
    }
}

impl Config {
    /// State of one window, created with defaults the first time it is asked for.
    pub fn window(&mut self, key: &str) -> &mut WindowState {
        self.windows.entry(key.to_owned()).or_default()
    }

    /// Name the user gave the team in this language, if any.
    ///
    /// Falls back to the single alias of 0.5.x so nobody loses what they had written.
    pub fn custom_name(&self, language: &str) -> Option<&str> {
        if let Some(name) = self.team_names.get(language) {
            return (!name.trim().is_empty()).then_some(name.trim());
        }
        self.team_alias
            .as_deref()
            .map(str::trim)
            .filter(|alias| !alias.is_empty())
    }

    /// Stores (or clears, when equal to the automatic name) the translation of a language.
    ///
    /// Writing the automatic name back is not an override: it removes the entry, so the addon
    /// keeps following the bundled translations.
    pub fn set_custom_name(&mut self, language: &str, name: &str, automatic: &str) {
        let name = name.trim();
        if name.is_empty() || name == automatic.trim() {
            self.team_names.remove(language);
        } else {
            self.team_names.insert(language.to_owned(), name.to_owned());
        }
        // Anything stored per language supersedes the legacy single alias.
        self.team_alias = None;
    }

    /// Colour of the configured team inside the given match, if it can be resolved.
    pub fn team_color(&self, m: &gw2lurker_core::Match) -> Option<TeamColor> {
        let id = self.team_id?;
        m.color_of(id)
    }

    /// Clamps values that could make the UI unusable if edited by hand.
    fn sanitize(&mut self) {
        self.poll_secs = self.poll_secs.clamp(15, 600);
        self.widget_opacity = if self.widget_opacity.is_finite() {
            self.widget_opacity.clamp(0.0, 1.0)
        } else {
            Config::default().widget_opacity
        };
        for state in self.windows.values_mut() {
            state.sanitize();
        }
    }
}

/// Directory where the addon may write, created if needed.
///
/// `<Documentos>\Guild Wars 2\addons\gw2_lurker`, like the neighbouring addons. If Documents
/// cannot be resolved we fall back to the folder Nexus gives us next to the DLL, because
/// losing the settings is worse than writing them in the wrong place.
pub fn addon_dir() -> io::Result<PathBuf> {
    if let Some(documents) = crate::ffi::paths::documents() {
        let dir = addon_dir_in(&documents);
        fs::create_dir_all(&dir)?;
        return Ok(dir);
    }
    let dir = nexus::paths::get_addon_dir(crate::ADDON_DIR)?;
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// `<Documentos>\Guild Wars 2\addons\<addon>`, split out so it can be tested without the API.
pub fn addon_dir_in(documents: &Path) -> PathBuf {
    documents
        .join("Guild Wars 2")
        .join("addons")
        .join(crate::ADDON_DIR)
}

/// Folder Nexus used to hand us, where 0.1–0.4.2 kept the settings.
fn legacy_addon_dir() -> Option<PathBuf> {
    nexus::paths::get_addon_dir(crate::ADDON_DIR).ok()
}

/// Copies an existing settings file from the old location the first time.
///
/// Returns a note for the log when something was migrated. The old file is left alone: it is
/// inside `Program Files` and may not even be deletable without elevation.
fn migrate_legacy(dir: &Path) -> Option<String> {
    let legacy = legacy_addon_dir()?;
    if legacy == dir {
        return None;
    }
    for name in [SETTINGS_FILE, LEGACY_FILE] {
        let target = dir.join(SETTINGS_FILE);
        if target.exists() {
            return None;
        }
        let source = legacy.join(name);
        if source.exists() {
            return match fs::copy(&source, &target) {
                Ok(bytes) => Some(format!(
                    "ajustes migrados de {} a {} ({bytes} bytes)",
                    source.display(),
                    target.display()
                )),
                Err(error) => Some(format!("no se pudo migrar {} ({error})", source.display())),
            };
        }
    }
    None
}

/// Name of the settings file, like `settings.json` in the neighbouring addons.
pub const SETTINGS_FILE: &str = "settings.json";
/// Previous good copy, also like the neighbours.
pub const SETTINGS_BACKUP: &str = "settings.json.bak";
/// Name used by 0.1–0.3, still read so nobody loses what they configured.
pub const LEGACY_FILE: &str = "config.json";

/// Path of the settings file.
pub fn config_path() -> io::Result<PathBuf> {
    Ok(addon_dir()?.join(SETTINGS_FILE))
}

/// Path of the previous good copy.
pub fn backup_path() -> io::Result<PathBuf> {
    Ok(addon_dir()?.join(SETTINGS_BACKUP))
}

/// Loads the settings.
///
/// The second element explains what happened when the file had to be discarded, so the
/// caller can log it and, later, show the safe-mode notice (T-26 / RF-51).
pub fn load() -> (Config, Option<String>) {
    let dir = match addon_dir() {
        Ok(dir) => dir,
        Err(e) => {
            return (
                Config::default(),
                Some(format!("sin directorio de addon: {e}")),
            )
        }
    };

    let migrated = migrate_legacy(&dir);

    let primary = dir.join(SETTINGS_FILE);
    let legacy = dir.join(LEGACY_FILE);
    let (path, was_legacy) = if primary.exists() {
        (primary, false)
    } else if legacy.exists() {
        (legacy, true)
    } else {
        return (Config::default(), migrated);
    };

    let raw = match fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(e) => return (Config::default(), Some(format!("no se pudo leer: {e}"))),
    };

    match serde_json::from_str::<Config>(&raw) {
        Ok(mut config) => {
            config.sanitize();
            let note = was_legacy
                .then(|| {
                    format!(
                        "leído {LEGACY_FILE} de una versión anterior; se guardará como {SETTINGS_FILE}"
                    )
                })
                .or(migrated);
            (config, note)
        }
        Err(e) => {
            let stem = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(SETTINGS_FILE);
            let backup = dir.join(format!(
                "{stem}.corrupt-{}",
                Utc::now().format("%Y%m%d-%H%M%S")
            ));
            let renamed = fs::rename(&path, &backup).is_ok();
            let detail = if renamed {
                format!("{stem} dañado ({e}); copia en {}", backup.display())
            } else {
                format!("{stem} dañado ({e}) y no se pudo conservar copia")
            };
            (Config::default(), Some(detail))
        }
    }
}

/// Writes the settings atomically and keeps the previous file as `settings.json.bak`.
///
/// Called from the writer thread, never from the render callback (rule 6).
pub fn save(config: &Config) -> io::Result<()> {
    let path = config_path()?;
    let mut config = config.clone();
    config.sanitize();
    let json = serde_json::to_string_pretty(&config).map_err(io::Error::other)?;

    let temp = path.with_extension("json.tmp");
    fs::write(&temp, json)?;
    if path.exists() {
        let _ = fs::copy(&path, backup_path()?);
    }
    fs::rename(&temp, &path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rf41_settings_live_under_documents_like_the_neighbours() {
        let dir = addon_dir_in(Path::new(r"C:\Users\X\Documentos"));
        assert_eq!(
            dir,
            Path::new(r"C:\Users\X\Documentos\Guild Wars 2\addons\gw2_lurker")
        );
        assert_eq!(
            dir.file_name().and_then(|name| name.to_str()),
            Some(crate::ADDON_DIR),
            "la carpeta lleva el nombre del addon, como timers_wvw o gw2_builds_wvw"
        );
    }

    #[test]
    fn rf41_defaults_are_usable() {
        let c = Config::default();
        assert_eq!(c.team_id, None);
        assert_eq!(c.poll_secs, 30, "RNF-02: 30 s medido");
        assert!(c.widget_locked);
        assert_eq!(c.alert_scope, AlertScope::CurrentMap);
        assert_eq!(c.language, LanguagePreference::Auto);
    }

    #[test]
    fn rf41_old_files_without_fields_still_load() {
        let c: Config = serde_json::from_str(r#"{"team_id": 12014}"#).unwrap();
        assert_eq!(c.team_id, Some(12014));
        assert_eq!(
            c.poll_secs, 30,
            "los campos nuevos toman su valor por defecto"
        );
    }

    #[test]
    fn rf41_out_of_range_values_are_clamped() {
        let mut c = Config {
            poll_secs: 1,
            widget_opacity: 5.0,
            ..Config::default()
        };
        c.sanitize();
        assert_eq!(c.poll_secs, 15);
        assert_eq!(c.widget_opacity, 1.0);

        let mut nan = Config {
            widget_opacity: f32::NAN,
            ..Config::default()
        };
        nan.sanitize();
        assert_eq!(nan.widget_opacity, 0.85, "NaN no puede llegar a imgui");
    }

    #[test]
    fn rf03b_configured_team_resolves_its_colour() {
        const MATCH: &str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../api/tests/fixtures/match_2-2.json"
        ));
        let m: gw2lurker_core::Match = serde_json::from_str(MATCH).unwrap();

        let with_team = Config {
            team_id: Some(12014),
            ..Config::default()
        };
        assert_eq!(with_team.team_color(&m), Some(TeamColor::Blue));

        assert_eq!(
            Config::default().team_color(&m),
            None,
            "sin equipo no hay color"
        );
    }

    #[test]
    fn rf73_followed_guilds_round_trip() {
        let mut c = Config::default();
        c.followed.add("1DB147F0");

        let json = serde_json::to_string(&c).unwrap();
        let back: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(back, c);
        assert!(back.followed.contains("1DB147F0"));
    }

    #[test]
    fn rf46_window_state_defaults_are_visible_and_movable() {
        let mut c = Config::default();
        let state = c.window("Settings");
        assert!(state.visible, "una ventana nueva se ve");
        assert!(!state.locked, "y se puede mover");
        assert!(!state.collapsed);
        assert_eq!(
            state.position, None,
            "la posición la decide el primer frame"
        );
        assert!(state.opacity > 0.5 && state.opacity <= 1.0);
    }

    #[test]
    fn rf46_window_state_survives_a_restart() {
        let mut c = Config::default();
        {
            let state = c.window("Settings");
            state.position = Some([120.0, 340.0]);
            state.size = Some([420.0, 260.0]);
            state.opacity = 0.6;
            state.locked = true;
        }

        let json = serde_json::to_string(&c).unwrap();
        let mut back: Config = serde_json::from_str(&json).unwrap();
        back.sanitize();
        let state = back.window("Settings");
        assert_eq!(state.position, Some([120.0, 340.0]));
        assert_eq!(state.size, Some([420.0, 260.0]));
        assert_eq!(state.opacity, 0.6);
        assert!(state.locked, "bloqueada sigue bloqueada tras reiniciar");
    }

    #[test]
    fn rf46_impossible_window_values_are_discarded() {
        let mut state = WindowState {
            opacity: f32::NAN,
            position: Some([f32::INFINITY, 10.0]),
            size: Some([4.0, 2.0]),
            ..WindowState::default()
        };
        state.sanitize();

        assert_eq!(
            state.opacity,
            WindowState::default().opacity,
            "NaN no llega a imgui"
        );
        assert_eq!(state.position, None, "una posición infinita se descarta");
        assert_eq!(
            state.size,
            Some([120.0, 60.0]),
            "el tamaño mínimo se respeta"
        );
    }

    #[test]
    fn rf46_unknown_window_key_does_not_panic() {
        let mut c: Config = serde_json::from_str(r#"{"windows":{}}"#).unwrap();
        c.window("Settings").opacity = 0.3;
        assert_eq!(c.windows.len(), 1);
    }
}
