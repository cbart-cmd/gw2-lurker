//! Raidcore Nexus addon (cdylib).
//!
//! This is the only crate allowed to use `unsafe`, and only inside [`ffi`].
//!
//! Responsibilities kept out of the render callback: no network, no file system. The
//! configuration is read once on load and the UI only formats what it already has.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

mod config;
mod context;
mod ffi;
mod i18n;
mod ui;

use std::sync::OnceLock;

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

fn load() {
    let (config, report) = config::load();
    if let Some(error) = report {
        log::warn!("config: {error}");
    }
    let _ = CONFIG.set(config);
    i18n::register_translations();
    ui::register();
    log::info!("WvW Lurker cargado (idioma {})", i18n::effective().code());
}

fn unload() {
    log::info!("WvW Lurker descargado");
}

/// Current configuration. Falls back to defaults if load never ran, which keeps the
/// render callback total and free of panics.
pub fn config() -> &'static config::Config {
    CONFIG.get_or_init(config::Config::default)
}
