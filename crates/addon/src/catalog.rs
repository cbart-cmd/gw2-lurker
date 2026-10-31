//! The static catalogues of the API: objectives, maps and guild names.
//!
//! All three are fetched at most once per session (objectives once per language) and kept in
//! process memory: they never change while the game runs, and asking for them on every poll
//! would spend the request budget on data that is already known.
//!
//! Split out of [`crate::worker`] when that file crossed 800 lines (AGENTS §2.8).

use std::sync::{Mutex, OnceLock};

use gw2lurker_api::Api;
use gw2lurker_core::matchup::GuildSummary;
use gw2lurker_core::model::ObjectiveInfo;

use crate::net::WinHttpTransport;
/// How many guild names are looked up per poll.
///
/// `/v2/guild` answers one guild per call, so asking for all of them at once would be a burst.
/// With this budget a matchup with twenty guilds is fully named in about a minute, and the
/// answers stay for the session.
const GUILD_LOOKUPS_PER_POLL: usize = 10;

#[derive(Default)]
struct GuildNameCache {
    known: std::collections::BTreeMap<String, String>,
    /// Guilds the API refused once. A disbanded guild answers 404 forever, so retrying it on
    /// every poll would burn the request budget for nothing.
    failed: std::collections::HashSet<String>,
}

/// `[TAG] Nombre`, the way the game shows a guild.
pub fn guild_label(info: &gw2lurker_api::GuildInfo) -> String {
    match (info.tag.trim(), info.name.trim()) {
        ("", name) => name.to_owned(),
        (tag, "") => format!("[{tag}]"),
        (tag, name) => format!("[{tag}] {name}"),
    }
}

/// Names of the guilds in the matchup: from the session cache, then from the API.
///
/// Returns the whole map so the caller publishes it with the dashboard and the UI never has to
/// take a second lock to label a row.
pub fn resolve_guild_names(
    api: &Api<WinHttpTransport>,
    guilds: &[GuildSummary],
) -> std::collections::BTreeMap<String, String> {
    static CACHE: OnceLock<Mutex<GuildNameCache>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(GuildNameCache::default()));
    let mut cache = cache.lock().unwrap_or_else(|error| error.into_inner());

    let mut budget = GUILD_LOOKUPS_PER_POLL;
    for summary in guilds {
        if budget == 0 {
            break;
        }
        let id = &summary.guild;
        if cache.known.contains_key(id) || cache.failed.contains(id) {
            continue;
        }
        budget -= 1;
        match api.guild(id) {
            Ok(info) => {
                cache.known.insert(id.clone(), guild_label(&info));
            }
            Err(error) => {
                log::debug!("worker: sin nombre de clan {id} ({error})");
                cache.failed.insert(id.clone());
            }
        }
    }
    cache.known.clone()
}
/// The four WvW maps, fetched once per session (they never change).
///
/// `None` means the API has not answered yet: the map tab says so instead of drawing an empty
/// box, and the next poll tries again.
pub fn maps_for(api: &Api<WinHttpTransport>) -> Vec<gw2lurker_core::WvwMap> {
    static MAPS: OnceLock<Mutex<Option<Vec<gw2lurker_core::WvwMap>>>> = OnceLock::new();
    let cache = MAPS.get_or_init(|| Mutex::new(None));
    let mut guard = cache.lock().unwrap_or_else(|error| error.into_inner());
    if guard.is_none() {
        match api.maps() {
            Ok(maps) => *guard = Some(maps),
            Err(error) => {
                log::warn!("worker: sin cajas de mapa ({error})");
                return Vec::new();
            }
        }
    }
    guard.clone().unwrap_or_default()
}
pub fn objectives_for(api: &Api<WinHttpTransport>) -> Result<Vec<ObjectiveInfo>, String> {
    static NAMES: OnceLock<Mutex<(String, Vec<ObjectiveInfo>)>> = OnceLock::new();
    let language = crate::i18n::effective().code();
    let cache = NAMES.get_or_init(|| Mutex::new((String::new(), Vec::new())));
    let mut guard = cache.lock().unwrap_or_else(|error| error.into_inner());

    // Static data: fetched once per session and per language. Changing the language refetches
    // once, which is what RF-44 asks for (no network while holding right Ctrl).
    if guard.0 != language || guard.1.is_empty() {
        let fetched = api
            .objectives(language)
            .map_err(|error| error.to_string())?;
        *guard = (language.to_owned(), fetched);
    }
    Ok(guard.1.clone())
}

/// Directory of the map tiles inside the addon's folder.
pub fn tiles_dir() -> Option<std::path::PathBuf> {
    Some(crate::config::addon_dir().ok()?.join("tiles"))
}

/// File of one tile. The name has every coordinate so a cache can be inspected by hand.
pub fn tile_path(x: u32, y: u32) -> Option<std::path::PathBuf> {
    Some(tiles_dir()?.join(format!(
        "1-1-{}-{x}-{y}.jpg",
        gw2lurker_core::project::TILE_ZOOM
    )))
}

/// URL of one tile on the official tile server.
pub fn tile_url(x: u32, y: u32) -> String {
    format!(
        "https://tiles.guildwars2.com/1/1/{}/{x}/{y}.jpg",
        gw2lurker_core::project::TILE_ZOOM
    )
}

/// Downloads the tiles of the four maps that are not in the cache yet, on its own thread.
///
/// Once per session. The render callback must never touch the network (rule 6), so the window
/// only reads the files afterwards; whatever is still missing simply is not drawn yet.
pub fn ensure_tiles(maps: &[gw2lurker_core::WvwMap]) {
    static STARTED: OnceLock<()> = OnceLock::new();
    let mut jobs = Vec::new();
    for map in maps {
        let frame = gw2lurker_core::Frame::of(map);
        for (x, y) in gw2lurker_core::project::tiles_of(&frame, gw2lurker_core::project::TILE_ZOOM)
        {
            let Some(path) = tile_path(x, y) else {
                return;
            };
            if !path.exists() {
                jobs.push((path, tile_url(x, y)));
            }
        }
    }
    if jobs.is_empty() || STARTED.set(()).is_err() {
        return;
    }

    let spawned = std::thread::Builder::new()
        .name("gw2lurker-tiles".to_owned())
        .spawn(move || {
            let transport = WinHttpTransport::default();
            let mut saved = 0;
            for (path, url) in jobs {
                match gw2lurker_api::Transport::get(&transport, &url) {
                    Ok(bytes) => {
                        if let Some(dir) = path.parent() {
                            let _ = std::fs::create_dir_all(dir);
                        }
                        match std::fs::write(&path, &bytes) {
                            Ok(()) => saved += 1,
                            Err(error) => {
                                log::warn!("tiles: no se pudo guardar {path:?} ({error})")
                            }
                        }
                    }
                    Err(error) => log::warn!("tiles: {url} ({error})"),
                }
            }
            log::info!("tiles: {saved} teselas del mapa en caché");
        });
    if spawned.is_err() {
        log::warn!("tiles: no se pudo lanzar la descarga");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rf20_a_guild_is_named_with_its_tag() {
        let info = gw2lurker_api::GuildInfo {
            id: "1DB147F0".to_owned(),
            name: "Legends Of Ravenous Destruction".to_owned(),
            tag: "Lord".to_owned(),
        };
        assert_eq!(guild_label(&info), "[Lord] Legends Of Ravenous Destruction");

        let untagged = gw2lurker_api::GuildInfo {
            tag: String::new(),
            ..info.clone()
        };
        assert_eq!(guild_label(&untagged), "Legends Of Ravenous Destruction");

        let anon = gw2lurker_api::GuildInfo {
            name: String::new(),
            ..info
        };
        assert_eq!(guild_label(&anon), "[Lord]", "sin nombre queda la etiqueta");
    }
}
