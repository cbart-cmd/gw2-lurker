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
/// (`~400 KB`). That is the **only** request made at this cadence: 0,2 per second against a
/// bucket of 300 that refills at 5 per second per IP (wiki *API:Best practices*, RNF-02).
pub const POLL_SECS_MIN: u32 = 5;

/// Interval used when the player has not chosen one.
pub const POLL_SECS_DEFAULT: u32 = 5;

/// Longest a pop-up may stay on screen, in seconds (RF-19a).
///
/// It is the same limit the settings slider offers: the sanitiser used to clamp to **5 s** while
/// the slider went to 20, so any duration above five seconds was written back as five on the next
/// save and the player saw it «lost» after restarting (report of 2026-11-01).
pub const POPUP_SECS_MAX: f32 = 20.0;

/// Cuánto dura el aviso emergente cuando el jugador no ha tocado nada.
///
/// Quince segundos: con 2 se iba antes de que diera tiempo a leerlo en mitad de un fight
/// (petición del jugador, 2026-11-01).
pub const POPUP_SECS_DEFAULT: f32 = 15.0;

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
    /// One team per account (RF-03a), saved as `roster.by_account` and keyed by the account name
    /// arcdps reports; the character name is the key only while arcdps has not answered (or is
    /// not installed). The `by_character` map of versions up to 0.26.5 is not read. The manual
    /// `team_id` wins over it.
    pub roster: gw2lurker_core::Roster,
    /// Último emparejamiento confirmado por equipo (id de equipo → `prompt::matchup_key`), para
    /// abrir solo el selector cuando cambian los rivales (RF-03j).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub confirmed_matchups: BTreeMap<String, String>,
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
            popup_secs: POPUP_SECS_DEFAULT,
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
            confirmed_matchups: BTreeMap::new(),
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
        let mut canonical_names = BTreeMap::new();
        for (id, name) in std::mem::take(&mut self.guild_names) {
            let id = gw2lurker_core::normalize_guild_id(&id).unwrap_or(id);
            canonical_names.entry(id).or_insert(name);
        }
        self.guild_names = canonical_names;
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
mod tests;
