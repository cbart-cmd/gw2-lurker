//! Raidcore Nexus addon (cdylib).
//!
//! This is the only crate allowed to use `unsafe`, and only inside [`ffi`].
//!
//! Responsibilities kept out of the render callback: no network, no file system. The
//! configuration is read once on load and the UI only formats what it already has.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

mod arc_team;
mod config;
mod context;
mod ffi;
mod i18n;
mod lang;
mod net;
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
    update_link: "https://raw.githubusercontent.com/cbart-cmd/gw2addonsreleases/main/gw2_lurker.dll",
    log_filter: "info"
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
            writer.request(config);
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

fn load() {
    let (config, report) = config::load();
    if let Some(error) = report {
        log::warn!("config: {error}");
    }
    let _ = CONFIG.set(config);

    match config::config_path() {
        Ok(path) => log::info!("WvW Lurker: ajustes en {}", path.display()),
        Err(error) => log::warn!("WvW Lurker: sin carpeta de addon escribible ({error})"),
    }
    let (map_id, map_type) = ffi::mumble_map().unwrap_or((0, 0));
    log::info!(
        "WvW Lurker cargado: mumble map {map_id} tipo {map_type}, gameplay {:?}, idioma {} (extras {:?}, nexus {:?})",
        ffi::is_gameplay(),
        i18n::effective().code(),
        lang::from_extras().map(gw2lurker_core::i18n::Language::code),
        lang::from_nexus_settings().map(gw2lurker_core::i18n::Language::code)
    );

    log::info!("WvW Lurker: {}", arc_team::describe());
    lang::subscribe();
    arc_team::subscribe();
    let _ = WRITER.set(Mutex::new(Some(writer::Writer::start())));
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
