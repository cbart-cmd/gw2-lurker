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
    /// Locked: cannot be moved or resized. The window keeps taking clicks.
    pub locked: bool,
    /// Click-through: the window keeps drawing but every click goes to the game (RF-18).
    ///
    /// It is a different thing from [`Self::locked`] on purpose: blocking the position is what
    /// you want while fighting, and giving up the mouse is not (bug of 2026-10-31).
    pub click_through: bool,
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
            click_through: false,
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

/// Fastest poll the addon allows, in seconds.
///
/// The player asked for the cadence of gw2mists (2026-10-31): **5 s**, which is affordable
/// because the poll asks for one match by id (~44 KB, the four maps) and not for the nine
/// (`~400 KB`). The worker still backs off on failures, and a poll every second would be the only
/// thing that could exceed the budget (`X-Rate-Limit-Limit: 600`; the window is not in the header
/// — see RNF-02).
pub const POLL_SECS_MIN: u32 = 5;

/// Interval used when the player has not chosen one.
pub const POLL_SECS_DEFAULT: u32 = 5;

/// Longest a pop-up may stay on screen, in seconds (RF-19a).
///
/// It is the same limit the settings slider offers: the sanitiser used to clamp to **5 s** while
/// the slider went to 20, so any duration above five seconds was written back as five on the next
/// save and the player saw it «lost» after restarting (report of 2026-11-01).
pub const POPUP_SECS_MAX: f32 = 20.0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Configured team id (`11xxx`/`12xxx`), `None` until the user picks one (RF-03a).
    pub team_id: Option<u32>,
    /// Name the user wrote for each team, per language (RF-03f).
    ///
    /// Indexed by **team id** and then by language: a translation belongs to one team, so
    /// switching team must not carry it over and coming back must bring it back (bug of
    /// 2026-10-31).
    pub team_translations: BTreeMap<String, BTreeMap<String, String>>,
    /// Legacy 0.6–0.10: translation per language only, applied to whichever team was
    /// configured. Read as a fallback and cleared on the first save.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub team_names: BTreeMap<String, String>,
    /// Single alias of 0.5.x, also migrated on first save.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team_alias: Option<String>,
    /// Polling interval in seconds; the measured default is 30 (RNF-02).
    pub poll_secs: u32,
    /// Where alerts apply.
    pub alert_scope: AlertScope,
    /// Which maps and guilds deserve an alert (RF-19a).
    pub alerts: gw2lurker_core::AlertFilter,
    /// How long the pop-up stays visible, in seconds; `0` disables it (RF-19a).
    pub popup_secs: f32,
    /// Whether the pop-up alerts are shown at all (T-42). Separate from `popup_secs` so turning
    /// them off and on again keeps the duration the player chose.
    pub popup_enabled: bool,
    /// Draw the real map tiles behind the objectives (T-29). On by default since the player asked
    /// for it (2026-10-31); RF-37 said off, and the switch is still there.
    pub map_tiles: bool,
    /// Text the player wrote for a guild, per guild id (T-43).
    ///
    /// Guild names are not translated by the game, so the only translations that can exist are
    /// the player's own: they win over the API name and are never overwritten by it.
    pub guild_labels: BTreeMap<String, String>,
    /// `[TAG] Nombre` de **los clanes que sigue el jugador**, por id de clan (RF-20b).
    ///
    /// Solo los seguidos, a propósito (petición del jugador, 2026-11-01): son diez como mucho y
    /// tienen que verse también fuera de WvW, donde no hay sondeo. Los nombres de los clanes que
    /// han reclamado algo en la sesión viven en **memoria** y se van al reiniciar el juego; el
    /// índice grande (`guild-names.json`) es solo la caché de la búsqueda por texto y no se copia
    /// aquí. `Config::sanitize` limpia cualquier nombre que no sea de un clan seguido.
    pub guild_names: BTreeMap<String, String>,
    /// Dónde se descarga el índice de nombres de clan (RF-20). Vacío = la URL publicada.
    pub guild_index_url: String,
    /// Where the player dragged the pop-up, `None` until it is moved (RF-19a).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub popup_pos: Option<[f32; 2]>,
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
    /// Ventanas con el estilo de Nexus o con el tema propio del addon (RF-60, RF-61).
    pub theme: Theme,
    /// Fondo del widget: semitransparente oscuro o transparente del todo (RF-63).
    pub widget_background: WidgetBackground,
    /// Escala del texto de las ventanas del addon, 0.8–1.6 (RF-19c).
    pub font_scale: f32,
    /// Cuántas líneas enseña el widget de avisos (RF-19c). El registro tiene su propio tope.
    pub max_lines: usize,
    /// Qué tipos de aviso se enseñan en el widget y en el pop-up (RF-19c).
    pub alert_kinds: AlertKinds,
}

/// Estilo de las ventanas del addon (RF-60, RF-61).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    /// Hereda el estilo que aplica Nexus: es lo que el jugador ya ve en el resto del juego.
    #[default]
    Inherited,
    /// Tema oscuro propio, aplicado **solo** a las ventanas del addon (RF-61).
    Own,
}

/// Fondo del widget (RF-63).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetBackground {
    /// Semitransparente oscuro: se lee sobre cualquier cosa.
    #[default]
    Dark,
    /// Transparente: solo el texto. La opacidad sigue mandando.
    Transparent,
}

/// Qué se enseña en el widget y en el pop-up (RF-19c).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AlertKinds {
    /// Capturas y cambios de dueño.
    pub flips: bool,
    /// Reclamaciones de clanes.
    pub claims: bool,
    /// Objetivos que se quedan sin clan.
    pub unclaims: bool,
}

impl Default for AlertKinds {
    fn default() -> Self {
        Self {
            flips: true,
            claims: true,
            unclaims: true,
        }
    }
}

impl AlertKinds {
    /// Whether a line of this kind is shown (RF-19c).
    pub fn shows(&self, kind: crate::worker::AlertKind) -> bool {
        match kind {
            crate::worker::AlertKind::Flip => self.flips,
            crate::worker::AlertKind::Claim => self.claims,
            crate::worker::AlertKind::Unclaim => self.unclaims,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            team_id: None,
            team_alias: None,
            team_names: BTreeMap::new(),
            team_translations: BTreeMap::new(),
            poll_secs: POLL_SECS_DEFAULT,
            alert_scope: AlertScope::default(),
            alerts: gw2lurker_core::AlertFilter::default(),
            popup_secs: 2.0,
            popup_enabled: true,
            map_tiles: true,
            guild_labels: BTreeMap::new(),
            guild_names: BTreeMap::new(),
            guild_index_url: crate::guild_index::DEFAULT_URL.to_owned(),
            popup_pos: None,
            language: LanguagePreference::default(),
            followed: FollowList::default(),
            widget_opacity: 0.85,
            widget_locked: true,
            windows: BTreeMap::new(),
            roster: gw2lurker_core::Roster::default(),
            debug_show_everywhere: false,
            theme: Theme::default(),
            widget_background: WidgetBackground::default(),
            font_scale: 1.0,
            max_lines: 5,
            alert_kinds: AlertKinds::default(),
        }
    }
}

impl Config {
    /// State of one window, created with defaults the first time it is asked for.
    pub fn window(&mut self, key: &str) -> &mut WindowState {
        self.windows.entry(key.to_owned()).or_default()
    }

    /// Name the user gave `team_id` in this language, if any.
    ///
    /// Falls back to the legacy shapes so nobody loses what they wrote: `team_names` (one name
    /// per language, from 0.6–0.10) and `team_alias` (a single name, from 0.5.x).
    pub fn custom_name(&self, team_id: u32, language: &str) -> Option<&str> {
        let name = self
            .team_translations
            .get(&team_id.to_string())
            .and_then(|by_language| by_language.get(language))
            .map(String::as_str)
            .or_else(|| self.team_names.get(language).map(String::as_str))
            .or(self.team_alias.as_deref())?;
        let name = name.trim();
        (!name.is_empty()).then_some(name)
    }

    /// Stores (or clears, when equal to the automatic name) the translation of one team.
    ///
    /// Writing the automatic name back is not an override: it removes the entry, so the addon
    /// keeps following the bundled translations. The legacy fields are always cleared, because
    /// from now on a translation belongs to a team.
    pub fn set_custom_name(&mut self, team_id: u32, language: &str, name: &str, automatic: &str) {
        let name = name.trim();
        let key = team_id.to_string();
        if name.is_empty() || name == automatic.trim() {
            if let Some(by_language) = self.team_translations.get_mut(&key) {
                by_language.remove(language);
                if by_language.is_empty() {
                    self.team_translations.remove(&key);
                }
            }
        } else {
            self.team_translations
                .entry(key)
                .or_default()
                .insert(language.to_owned(), name.to_owned());
        }
        self.team_names.clear();
        self.team_alias = None;
    }

    /// Colour of the configured team inside the given match, if it can be resolved.
    pub fn team_color(&self, m: &gw2lurker_core::Match) -> Option<TeamColor> {
        let id = self.team_id?;
        m.color_of(id)
    }

    /// Extra widgets: the pop-up time is a duration, not a unitless number.
    fn sanitize(&mut self) {
        self.poll_secs = self.poll_secs.clamp(POLL_SECS_MIN, 600);
        self.popup_secs = if self.popup_secs.is_finite() {
            self.popup_secs.clamp(0.0, POPUP_SECS_MAX)
        } else {
            Config::default().popup_secs
        };
        self.widget_opacity = if self.widget_opacity.is_finite() {
            self.widget_opacity.clamp(0.0, 1.0)
        } else {
            Config::default().widget_opacity
        };
        for state in self.windows.values_mut() {
            state.sanitize();
        }
        // Solo los clanes que se siguen tienen nombre guardado (petición del jugador, 2026-11-01):
        // los de la sesión viven en memoria y el índice de búsqueda no se copia aquí. Así el
        // fichero no crece sin freno, y de paso limpia lo que versiones anteriores dejaron.
        let followed: std::collections::BTreeSet<String> =
            self.followed.iter().map(str::to_owned).collect();
        let before = self.guild_names.len();
        self.guild_names.retain(|id, _| followed.contains(id));
        if self.guild_names.len() != before {
            log::info!(
                "ajustes: {} nombres de clan de la sesión descartados (solo se guardan los seguidos)",
                before - self.guild_names.len()
            );
        }
        // Una URL vacía o sin https no se acepta: la descarga del índice la rechazaría y el
        // jugador vería un error que no puede arreglar desde la ventana.
        let url = self.guild_index_url.trim();
        self.guild_index_url = if url.starts_with("https://") {
            url.to_owned()
        } else {
            crate::guild_index::DEFAULT_URL.to_owned()
        };
        // Errata de mi URL por defecto en la 0.25.0: el repositorio es `gw2addonsreleases`, con dos
        // «s». Quien tenga la mala guardada se queda sin descarga (404), así que se corrige sola.
        self.guild_index_url = self
            .guild_index_url
            .replace("gw2addonreleases", "gw2addonsreleases");
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
    fn rf19a_popup_time_defaults_to_two_seconds_and_clamps() {
        assert_eq!(Config::default().popup_secs, 2.0);
        let mut c = Config {
            popup_secs: 30.0,
            ..Config::default()
        };
        c.sanitize();
        // El tope tiene que ser el mismo que el del control de Ajustes: con 5 s aquí y 20 en la
        // interfaz, cualquier valor por encima de cinco se guardaba como cinco y el jugador lo
        // veía «perdido» tras reiniciar (informe del 2026-11-01).
        assert_eq!(
            c.popup_secs, POPUP_SECS_MAX,
            "como mucho lo que ofrece el control"
        );
        let mut c = Config {
            popup_secs: 12.0,
            ..Config::default()
        };
        c.sanitize();
        assert_eq!(
            c.popup_secs, 12.0,
            "12 s se guarda tal cual: no se pierde al reiniciar"
        );
        let mut c = Config {
            popup_secs: -1.0,
            ..Config::default()
        };
        c.sanitize();
        assert_eq!(c.popup_secs, 0.0, "0 s es válido: sin pop-up");
        let mut c = Config {
            popup_secs: f32::NAN,
            ..Config::default()
        };
        c.sanitize();
        assert_eq!(c.popup_secs, 2.0);
    }

    #[test]
    fn rf19a_every_alert_source_is_enabled_by_default() {
        let filter = Config::default().alerts;
        assert!(filter.ebg && filter.blue && filter.green && filter.red);
        assert!(filter.current_map && filter.followed);
    }

    #[test]
    fn rf41_defaults_are_usable() {
        let c = Config::default();
        assert_eq!(c.team_id, None);
        assert_eq!(
            c.poll_secs, POLL_SECS_DEFAULT,
            "RNF-02: 5 s por petición del jugador"
        );
        assert!(c.widget_locked);
        assert_eq!(c.alert_scope, AlertScope::CurrentMap);
        assert_eq!(c.language, LanguagePreference::Auto);
    }

    #[test]
    fn rf41_old_files_without_fields_still_load() {
        let c: Config = serde_json::from_str(r#"{"team_id": 12014}"#).unwrap();
        assert_eq!(c.team_id, Some(12014));
        assert_eq!(
            c.poll_secs, POLL_SECS_DEFAULT,
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
        assert_eq!(c.poll_secs, POLL_SECS_MIN);
        assert_eq!(c.widget_opacity, 1.0);

        let mut nan = Config {
            widget_opacity: f32::NAN,
            ..Config::default()
        };
        nan.sanitize();
        assert_eq!(nan.widget_opacity, 0.85, "NaN no puede llegar a imgui");
    }

    #[test]
    fn rf20c_an_index_url_without_https_falls_back_to_the_published_one() {
        // La descarga del índice exige https; una URL escrita a mano sin él dejaría al jugador con
        // un error que no puede arreglar desde la ventana.
        for bad in ["", "   ", "http://ejemplo/x.zip", "ejemplo/x.zip"] {
            let mut c = Config {
                guild_index_url: bad.to_owned(),
                ..Config::default()
            };
            c.sanitize();
            assert_eq!(
                c.guild_index_url,
                crate::guild_index::DEFAULT_URL,
                "{bad:?}"
            );
        }
        let mut good = Config {
            guild_index_url: " https://ejemplo/x.zip ".to_owned(),
            ..Config::default()
        };
        good.sanitize();
        assert_eq!(good.guild_index_url, "https://ejemplo/x.zip");
    }

    #[test]
    fn rf20c_the_broken_default_url_repairs_itself() {
        // La 0.25.0 guardó `gw2addonreleases` (una «s» menos) en el fichero del jugador y con ella
        // GitHub responde 404: «no se pudo descargar… no existe» (informe de 2026-11-01).
        let mut c = Config {
            guild_index_url: "https://raw.githubusercontent.com/cbart-cmd/gw2addonreleases/main/gw2_lurker_guilds.zip".to_owned(),
            ..Config::default()
        };
        c.sanitize();
        assert_eq!(c.guild_index_url, crate::guild_index::DEFAULT_URL);
        assert!(c.guild_index_url.contains("gw2addonsreleases"));
    }

    #[test]
    fn rf20b_guild_names_are_stored_in_the_settings_file() {
        // Los nombres resueltos viven en `settings.json`, no en el índice de búsqueda: es lo que
        // evita clonar miles de entradas en cada sondeo (petición del jugador, 2026-11-01).
        let mut c = Config::default();
        c.guild_names.insert(
            "6C45F3C8-F21C-EF11-81AB-8B4A188BDE13".to_owned(),
            "[YaK] Dolyak Sanctuary".to_owned(),
        );
        let json = serde_json::to_string(&c).unwrap();
        let back: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(
            back.guild_names
                .get("6C45F3C8-F21C-EF11-81AB-8B4A188BDE13")
                .map(String::as_str),
            Some("[YaK] Dolyak Sanctuary")
        );
    }

    #[test]
    fn rf20b_only_the_followed_guilds_keep_their_name_in_the_settings() {
        // Petición del jugador (2026-11-01): en `settings.json` solo los nombres de los clanes que
        // sigue; los de la sesión, en memoria, para que el fichero no crezca sin freno.
        let mut c = Config::default();
        c.followed.add("SEGUIDO");
        c.guild_names
            .insert("SEGUIDO".to_owned(), "[YaK] Dolyak Sanctuary".to_owned());
        c.guild_names
            .insert("DE_PASO".to_owned(), "[DB] Die Barbaren".to_owned());
        c.guild_names
            .insert("OTRO".to_owned(), "[DG] Die Garde".to_owned());
        c.sanitize();
        assert_eq!(c.guild_names.len(), 1, "{:?}", c.guild_names);
        assert_eq!(
            c.guild_names.get("SEGUIDO").map(String::as_str),
            Some("[YaK] Dolyak Sanctuary")
        );
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
