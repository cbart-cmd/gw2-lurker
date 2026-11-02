//! Raidcore Nexus addon (cdylib).
//!
//! This is the only crate allowed to use `unsafe`, and only inside [`ffi`].
//!
//! Responsibilities kept out of the render callback: no network, no file system. The
//! configuration is read once on load and the UI only formats what it already has.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

mod alerts;
mod arc_team;
mod assignment;
mod background;
mod catalog;
mod clock;
mod config;
mod context;
mod detect;
mod diagnostics;
mod ffi;
mod guild_index;
mod i18n;
mod keybinds;
mod lang;
mod md5;
mod names;
mod net;
mod search;
mod session_names;
mod team;
mod ui;
mod worker;
mod writer;

use std::sync::{Mutex, MutexGuard, OnceLock};

nexus::export! {
    name: "WvW Lurker",
    signature: -0x4c75_726b,
    load: crate::load,
    unload: crate::unload,
    flags: nexus::AddonFlags::DisableHotloading,
    provider: nexus::UpdateProvider::Direct,
    update_link: "https://raw.githubusercontent.com/cbart-cmd/gw2addonsreleases/main/gw2_lurker.dll"
}

/// Directory name used by Nexus for this addon (`addons/gw2_lurker/`).
pub const ADDON_DIR: &str = "gw2_lurker";

/// Configuration, loaded once on load. If it cannot be read the addon runs with
/// defaults; T-26 will surface that as the safe-mode notice (RF-51).
static CONFIG: OnceLock<config::Config> = OnceLock::new();

/// Editable copy the UI mutates (window positions, opacity, lock…). Written by
/// [`writer::Writer`], so the render callback never touches the disk (rule 6).
static RUNTIME: OnceLock<Mutex<config::Config>> = OnceLock::new();

/// Background writer for `settings.json`. `Option` so `unload` can take it and join.
static WRITER: OnceLock<Mutex<Option<writer::Writer>>> = OnceLock::new();

/// Why the configuration could not be read, if it could not (RF-51).
///
/// Kept for the whole session: the notice has to stay visible until the player closes the game,
/// because that is when the file gets rewritten and the problem stops being obvious.
static SAFE_MODE: OnceLock<Option<String>> = OnceLock::new();

/// The load report, `Some` when the addon is running on defaults after a broken file (RF-51).
pub fn safe_mode() -> Option<String> {
    SAFE_MODE.get().cloned().flatten()
}

/// Editable configuration used by the UI.
///
/// Copies the loaded config on first use, so the render callback never blocks on file I/O
/// and never sees a half-loaded file. A poisoned lock is recovered: a panic elsewhere must
/// not leave the windows permanently frozen.
pub fn runtime() -> MutexGuard<'static, config::Config> {
    let runtime = RUNTIME.get_or_init(|| Mutex::new(config().clone()));
    runtime.lock().unwrap_or_else(|error| error.into_inner())
}

/// Asks the writer to persist the configuration. Cheap enough for the render callback.
pub fn request_save(config: &config::Config) {
    if let Some(slot) = WRITER.get() {
        let guard = slot.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(writer) = guard.as_ref() {
            if !writer.request(config) {
                log::error!("settings writer is unavailable; changes may be lost until shutdown");
            }
        }
    }
}

/// Persists the configuration when it changed, at most twice a second.
///
/// Dragging a window changes the position every frame; without the throttle that would be
/// sixty writes per second for nothing.
pub fn persist_if_changed() {
    use std::time::{Duration, Instant};

    static GATE: OnceLock<Mutex<(Option<config::Config>, Option<Instant>)>> = OnceLock::new();
    const MIN_INTERVAL: Duration = Duration::from_millis(500);

    let current = runtime();
    let mut gate = GATE
        .get_or_init(|| Mutex::new((None, None)))
        .lock()
        .unwrap_or_else(|error| error.into_inner());

    if gate.0.as_ref() == Some(&*current) {
        return;
    }
    let due = match gate.1 {
        None => true,
        Some(at) => at.elapsed() >= MIN_INTERVAL,
    };
    if !due {
        return;
    }

    request_save(&current);
    gate.0 = Some(current.clone());
    gate.1 = Some(Instant::now());
}

/// Version loaded in this session, from the crate metadata.
pub fn loaded_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Size and modification time of our own DLL, taken at load.
static LOADED_DLL: OnceLock<Option<(u64, std::time::SystemTime)>> = OnceLock::new();
const DLL_CHECK_INTERVAL: std::time::Duration = std::time::Duration::from_secs(10);

#[derive(Debug, Default)]
struct DllWatch {
    last_check: Option<std::time::Instant>,
    stale_path: Option<String>,
}

impl DllWatch {
    fn should_check(&mut self, now: std::time::Instant) -> bool {
        if self.stale_path.is_some()
            || self
                .last_check
                .is_some_and(|last| now.duration_since(last) < DLL_CHECK_INTERVAL)
        {
            return false;
        }
        self.last_check = Some(now);
        true
    }

    fn observe(
        &mut self,
        loaded: Option<(u64, std::time::SystemTime)>,
        current: Option<(u64, std::time::SystemTime)>,
        path: String,
    ) -> bool {
        if self.stale_path.is_some() || loaded.is_none() || current == loaded {
            return false;
        }
        self.stale_path = Some(path);
        true
    }
}

static DLL_WATCH: OnceLock<Mutex<DllWatch>> = OnceLock::new();

/// Path of the DLL Nexus put on disk: `<juego>/addons/gw2_lurker.dll`.
///
/// It is the file Nexus replaces when an update is published, and the one named in its log line
/// `Update available for …` (RF-50).
fn own_dll_path() -> Option<std::path::PathBuf> {
    let addons = nexus::paths::get_game_dir().ok()?.join("addons");
    Some(addons.join("gw2_lurker.dll"))
}

fn dll_fingerprint(path: &std::path::Path) -> Option<(u64, std::time::SystemTime)> {
    let meta = std::fs::metadata(path).ok()?;
    Some((meta.len(), meta.modified().ok()?))
}

/// Remembers the DLL as it was at load, so a later change can be noticed (RF-50).
fn remember_loaded_dll() {
    let fingerprint = own_dll_path().and_then(|path| dll_fingerprint(&path));
    let _ = LOADED_DLL.set(fingerprint);
}

/// Checks for a replacement from the worker thread, at most once every ten seconds.
///
/// The UI must only read the cached result: filesystem metadata calls do not belong on the render
/// thread, and the notice must keep updating after the initial unchanged check (RF-50).
pub(crate) fn check_dll_update() {
    let should_check = {
        let mut watch = DLL_WATCH
            .get_or_init(|| Mutex::new(DllWatch::default()))
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        watch.should_check(std::time::Instant::now())
    };
    if !should_check {
        return;
    }

    let Some(loaded) = LOADED_DLL.get().copied().flatten() else {
        return;
    };
    let Some(path) = own_dll_path() else {
        return;
    };
    let current = dll_fingerprint(&path);
    let mut watch = DLL_WATCH
        .get_or_init(|| Mutex::new(DllWatch::default()))
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    if watch.observe(Some(loaded), current, path.display().to_string()) {
        log::info!(
            "actualizado en disco: la DLL ha cambiado ({} -> {} bytes); hace falta reiniciar",
            loaded.0,
            current.map_or(0, |fingerprint| fingerprint.0)
        );
    }
}

/// Whether Nexus has replaced the file on disk since this session started.
pub fn stale_dll() -> Option<String> {
    DLL_WATCH.get().and_then(|watch| {
        watch
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .stale_path
            .clone()
    })
}

fn load() {
    let (config, report) = config::load();
    if let Some(error) = report.clone() {
        log::warn!("config: {error}");
    }
    // RF-51: a broken file is not only logged, it is shown. A settings file that silently went
    // back to defaults looks like the addon forgot everything.
    let _ = SAFE_MODE.set(report);
    let language = config.language;
    let _ = CONFIG.set(config);
    i18n::set_preference(language);
    remember_loaded_dll();

    match config::config_path() {
        Ok(path) => log::info!("WvW Lurker: ajustes en {}", path.display()),
        Err(error) => log::warn!("WvW Lurker: sin carpeta de addon escribible ({error})"),
    }
    let (map_id, map_type) = ffi::mumble_map().unwrap_or((0, 0));
    log::info!(
        "WvW Lurker v{} cargado: mumble map {map_id} tipo {map_type}, gameplay {:?}, idioma {} (extras {:?}, nexus {:?})",
        env!("CARGO_PKG_VERSION"),
        ffi::is_gameplay(),
        i18n::effective().code(),
        lang::from_extras().map(gw2lurker_core::i18n::Language::code),
        lang::from_nexus_settings().map(gw2lurker_core::i18n::Language::code)
    );

    log::info!("WvW Lurker: {}", arc_team::describe());
    keybinds::remember_shortcut();
    session_names::reset();
    guild_index::ensure_present();
    lang::subscribe();
    arc_team::subscribe();
    match writer::Writer::start() {
        Ok(writer) => {
            if WRITER.set(Mutex::new(Some(writer))).is_err() {
                log::error!("settings writer was already initialized");
            }
        }
        Err(error) => {
            log::error!(
                "could not start settings writer; settings will be saved on shutdown: {error}"
            );
        }
    }
    ui::register();
    worker::start();
}

fn unload() {
    worker::stop();
    let runtime = runtime();
    let taken = WRITER.get().and_then(|slot| {
        slot.lock()
            .unwrap_or_else(|error| error.into_inner())
            .take()
    });
    match taken {
        Some(writer) => writer.finish(&runtime),
        None => {
            if let Err(error) = config::save(&runtime) {
                log::warn!("no se pudo guardar settings.json: {error}");
            }
        }
    }
    log::info!("WvW Lurker descargado");
}

/// Configuration as it was loaded: what the worker reads, never changes.
pub fn config() -> &'static config::Config {
    CONFIG.get_or_init(config::Config::default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rf50_dll_replacement_is_detected_after_an_unchanged_check() {
        let start = std::time::Instant::now();
        let loaded = (100, std::time::SystemTime::UNIX_EPOCH);
        let mut watch = DllWatch::default();

        assert!(watch.should_check(start));
        assert!(!watch.observe(Some(loaded), Some(loaded), "addon.dll".to_owned()));
        assert!(!watch.should_check(start + std::time::Duration::from_secs(5)));
        assert!(watch.should_check(start + DLL_CHECK_INTERVAL));
        assert!(watch.observe(
            Some(loaded),
            Some((200, std::time::SystemTime::UNIX_EPOCH)),
            "addon.dll".to_owned()
        ));
        assert!(!watch.should_check(start + DLL_CHECK_INTERVAL * 2));
        assert_eq!(watch.stale_path.as_deref(), Some("addon.dll"));
    }
}
