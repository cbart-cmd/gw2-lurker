//! The static catalogues of the API: objectives, maps and guild names.
//!
//! All three are fetched at most once per session (objectives once per language) and kept in
//! process memory: they never change while the game runs, and asking for them on every poll
//! would spend the request budget on data that is already known.
//!
//! Split out of [`crate::worker`] when that file crossed 800 lines (AGENTS §2.8).

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use gw2lurker_api::{Api, ApiError, Backoff};
use gw2lurker_core::matchup::GuildSummary;
use gw2lurker_core::model::ObjectiveInfo;

use crate::net::WinHttpTransport;
/// How many guild names are resolved per poll.
///
/// One request each, and a matchup has around forty guilds, so with this budget every name is known
/// within a few polls. It was ten, which left alerts without a guild name for longer than the player
/// was willing to wait (report of 2026-10-31). The API limit is a bucket of 300 requests that
/// refills at 5 per second per IP (wiki *API:Best practices*, specs/07-spikes.md): twenty lookups
/// every 5 s is 4 per second, and only until the names of the matchup are known. The worker backs
/// off when the API returns 429.
const GUILD_LOOKUPS_PER_POLL: usize = 20;
const STATIC_DATA_RETRY: Duration = Duration::from_secs(60);

#[derive(Default)]
struct GuildLookupFailures {
    permanent: HashSet<String>,
    retry_after: HashMap<String, Instant>,
    backoff: HashMap<String, Backoff>,
}

impl GuildLookupFailures {
    fn should_skip(&self, id: &str, now: Instant) -> bool {
        self.permanent.contains(id)
            || self
                .retry_after
                .get(id)
                .is_some_and(|retry_after| now < *retry_after)
    }

    fn record(&mut self, id: &str, error: &ApiError, now: Instant) {
        if Backoff::should_retry(error) {
            self.permanent.remove(id);
            let delay = self
                .backoff
                .entry(id.to_owned())
                .or_default()
                .next_delay_ms();
            self.retry_after
                .insert(id.to_owned(), now + Duration::from_millis(delay));
        } else {
            self.permanent.insert(id.to_owned());
            self.retry_after.remove(id);
            self.backoff.remove(id);
        }
    }

    fn resolved(&mut self, id: &str) {
        self.permanent.remove(id);
        self.retry_after.remove(id);
        self.backoff.remove(id);
    }
}

fn guild_lookup_failures() -> &'static Mutex<GuildLookupFailures> {
    static FAILED: OnceLock<Mutex<GuildLookupFailures>> = OnceLock::new();
    FAILED.get_or_init(|| Mutex::new(GuildLookupFailures::default()))
}

fn static_data_retry_due(last_attempt: Option<Instant>, now: Instant) -> bool {
    last_attempt.is_none_or(|at| now.saturating_duration_since(at) >= STATIC_DATA_RETRY)
}

/// `[TAG] Nombre`, the way the game shows a guild.
pub fn guild_label(info: &gw2lurker_api::GuildInfo) -> String {
    match (info.tag.trim(), info.name.trim()) {
        ("", name) => name.to_owned(),
        (tag, "") => format!("[{tag}]"),
        (tag, name) => format!("[{tag}] {name}"),
    }
}

/// Names of the guilds in the matchup: from **this session's file** (`temp.json`), from the
/// settings, then from the API.
///
/// Three places, on purpose (peticiones del jugador, 2026-11-01):
///
/// - **`settings.json`** guarda **solo los clanes que sigues** (`Config::guild_names`). Son diez
///   como mucho y tienen que verse también fuera de WvW, cuando no hay sondeo.
/// - **`temp.json`** ([`crate::session_names`]) guarda los de los clanes que han reclamado algo
///   **en esta sesión**. Se **regenera en cada arranque**, así que no dura más que el propio juego.
/// - El índice grande `guild-names.json` **no** se lee aquí: es solo la caché de la búsqueda.
///
/// Returns the map so the caller publishes it with the dashboard.
pub fn resolve_guild_names(
    api: &Api<WinHttpTransport>,
    guilds: &[GuildSummary],
    followed: &gw2lurker_core::FollowList,
) -> BTreeMap<String, String> {
    let saved = crate::runtime().guild_names.clone();
    let mut known = saved.clone();
    known.extend(crate::session_names::all());

    // The followed guilds first: they are the ones the player sees by name in the settings and
    // in the alerts, and they were being left as raw ids because only the guilds that had
    // claimed something in the matchup were looked up (bug of 2026-10-31).
    let ids = followed
        .iter()
        .chain(guilds.iter().map(|summary| summary.guild.as_str()));

    let mut budget = GUILD_LOOKUPS_PER_POLL;
    for id in ids {
        if crate::background::is_stopping() {
            break;
        }
        if budget == 0 {
            break;
        }
        if known.contains_key(id)
            || guild_lookup_failures()
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .should_skip(id, Instant::now())
        {
            continue;
        }
        budget -= 1;
        match api.guild(id) {
            Ok(info) => {
                known.insert(id.to_owned(), guild_label(&info));
                guild_lookup_failures()
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .resolved(id);
            }
            Err(error) => {
                log::debug!("worker: sin nombre de clan {id} ({error})");
                guild_lookup_failures()
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .record(id, &error, Instant::now());
            }
        }
    }

    // Todo lo resuelto se queda en `temp.json`, el fichero de la sesión: se borra al arrancar el
    // addon, así que no dura más que el propio juego.
    if crate::session_names::merge(&known) {
        if let Err(error) = crate::session_names::save() {
            log::warn!("no se pudo guardar temp.json ({error})");
        }
    }

    // Y a los ajustes solo van los clanes que se siguen: la lista de seguidos manda, y lo que ya
    // no se sigue se quita (así el fichero no crece nunca más allá de `MAX_FOLLOWED`).
    {
        let mut config = crate::runtime();
        let followed_now: std::collections::BTreeSet<String> =
            config.followed.iter().map(str::to_owned).collect();
        let mut wanted: BTreeMap<String, String> = config
            .guild_names
            .iter()
            .filter(|(id, _)| followed_now.contains(*id))
            .map(|(id, label)| (id.clone(), label.clone()))
            .collect();
        for id in &followed_now {
            if let Some(label) = known.get(id) {
                wanted.insert(id.clone(), label.clone());
            }
        }
        if wanted != config.guild_names {
            config.guild_names = wanted;
            crate::request_save(&config);
        }
    }
    known
}
/// Names of the old servers, fetched once per session.
///
/// They are shown next to each team in the picker: the team names are new, the world names are
/// what the player has known for years (report of 2026-10-31).
pub fn world_names(api: &Api<WinHttpTransport>) -> BTreeMap<u32, String> {
    struct WorldCache {
        names: BTreeMap<u32, String>,
        last_attempt: Option<Instant>,
    }
    static WORLDS: OnceLock<Mutex<WorldCache>> = OnceLock::new();
    let cache = WORLDS.get_or_init(|| {
        Mutex::new(WorldCache {
            names: BTreeMap::new(),
            last_attempt: None,
        })
    });
    let mut guard = cache.lock().unwrap_or_else(|error| error.into_inner());
    if guard.names.is_empty() && static_data_retry_due(guard.last_attempt, Instant::now()) {
        guard.last_attempt = Some(Instant::now());
        match api.worlds() {
            Ok(worlds) => guard.names = worlds,
            Err(error) => log::warn!("worker: sin nombres de mundo ({error})"),
        }
    }
    guard.names.clone()
}

/// WvW map geometries, fetched once per session (they never change).
/// `None` means the API has not answered yet: the map tab says so instead of drawing an empty
/// box, and the next attempt follows a short retry cooldown.
pub fn maps_for(api: &Api<WinHttpTransport>) -> Vec<gw2lurker_core::WvwMap> {
    struct MapCache {
        maps: Option<Vec<gw2lurker_core::WvwMap>>,
        last_attempt: Option<Instant>,
    }
    static MAPS: OnceLock<Mutex<MapCache>> = OnceLock::new();
    let cache = MAPS.get_or_init(|| {
        Mutex::new(MapCache {
            maps: None,
            last_attempt: None,
        })
    });
    let mut guard = cache.lock().unwrap_or_else(|error| error.into_inner());
    if guard.maps.is_none() && static_data_retry_due(guard.last_attempt, Instant::now()) {
        guard.last_attempt = Some(Instant::now());
        match api.maps() {
            Ok(maps) => guard.maps = Some(maps),
            Err(error) => {
                log::warn!("worker: sin cajas de mapa ({error})");
                return Vec::new();
            }
        }
    }
    guard.maps.clone().unwrap_or_default()
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

/// Continent and floor of the WvW maps on the tile server.
///
/// The borderlands and Eternal Battlegrounds live in **continent 2, floor 1**; continent 1
/// answers an ocean tile for any coordinate, which fails silently (measured 2026-10-31).
const TILE_CONTINENT: &str = "2";
const TILE_FLOOR: &str = "1";
const TILE_CACHE_TTL: Duration = Duration::from_secs(24 * 60 * 60);
pub(crate) const MIN_TILE_BYTES: u64 = 1024;
static TILE_REVISION: AtomicU64 = AtomicU64::new(0);

#[derive(Default)]
struct TileDataCache(HashMap<(u32, u32), std::sync::Arc<[u8]>>);

impl TileDataCache {
    fn get(&self, x: u32, y: u32) -> Option<std::sync::Arc<[u8]>> {
        self.0.get(&(x, y)).cloned()
    }

    fn insert(&mut self, x: u32, y: u32, bytes: Vec<u8>) -> bool {
        if (bytes.len() as u64) < MIN_TILE_BYTES {
            return false;
        }
        self.0.insert((x, y), std::sync::Arc::from(bytes));
        true
    }
}

static TILE_DATA: OnceLock<Mutex<TileDataCache>> = OnceLock::new();

fn tile_data() -> &'static Mutex<TileDataCache> {
    TILE_DATA.get_or_init(|| Mutex::new(TileDataCache::default()))
}

pub(crate) fn tile_bytes(x: u32, y: u32) -> Option<std::sync::Arc<[u8]>> {
    tile_data()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .get(x, y)
}

pub(crate) fn tile_revision() -> u64 {
    TILE_REVISION.load(Ordering::Relaxed)
}

/// File of one tile. The name has every coordinate so a cache can be inspected by hand, and the
/// continent is in it too: a tile of another continent is not the same tile.
pub fn tile_path(x: u32, y: u32) -> Option<std::path::PathBuf> {
    Some(tiles_dir()?.join(format!(
        "{TILE_CONTINENT}-{TILE_FLOOR}-{}-{x}-{y}.jpg",
        gw2lurker_core::project::TILE_ZOOM
    )))
}

/// URL of one tile on the official tile server.
pub fn tile_url(x: u32, y: u32) -> String {
    format!(
        "https://tiles.guildwars2.com/{TILE_CONTINENT}/{TILE_FLOOR}/{}/{x}/{y}.jpg",
        gw2lurker_core::project::TILE_ZOOM
    )
}

/// Deletes tiles saved by older versions, whose names did not carry the continent.
///
/// They were the ocean tiles of continent 1: leaving them there wastes a megabyte and, worse,
/// any name collision would draw the wrong sea again.
fn forget_stale_tiles(keep_prefix: &str) {
    let Some(dir) = tiles_dir() else {
        return;
    };
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return;
    };
    let mut removed = 0;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if name.ends_with(".jpg")
            && !name.starts_with(keep_prefix)
            && std::fs::remove_file(entry.path()).is_ok()
        {
            removed += 1;
        }
    }
    if removed > 0 {
        log::info!("tiles: {removed} teselas de una version anterior borradas");
    }
}

/// Loads cached tiles into memory and downloads stale ones on a background thread.
///
/// Once per session. The render callback reads the in-memory snapshots only; it does no file I/O.
pub fn ensure_tiles(maps: &[gw2lurker_core::WvwMap]) {
    static STARTED: OnceLock<()> = OnceLock::new();
    // Sin cajas de mapa no hay nada que pedir, y marcarlo como hecho dejaría el mapa real vacío
    // para toda la sesión: el primer sondeo puede llegar antes que `/v2/maps` (bug encontrado
    // revisando el de las teselas, 2026-10-31).
    if maps.is_empty() || STARTED.get().is_some() {
        return;
    }
    forget_stale_tiles(&format!("{TILE_CONTINENT}-{TILE_FLOOR}-"));

    let now = std::time::SystemTime::now();
    let mut queued: BTreeMap<(u32, u32), bool> = BTreeMap::new();
    for map in maps {
        let frame = gw2lurker_core::Frame::of(map);
        let refresh = map.is_seared_borderlands();
        for (x, y) in gw2lurker_core::project::tiles_of(&frame, gw2lurker_core::project::TILE_ZOOM)
        {
            queued
                .entry((x, y))
                .and_modify(|force| *force |= refresh)
                .or_insert(refresh);
        }
    }
    let mut jobs = Vec::new();
    for ((x, y), force) in queued {
        let Some(path) = tile_path(x, y) else {
            return;
        };
        let metadata = std::fs::metadata(&path).ok();
        let is_stale = metadata.as_ref().is_none_or(|metadata| {
            if metadata.len() < MIN_TILE_BYTES {
                return true;
            }
            metadata
                .modified()
                .map_or(true, |modified| tile_needs_refresh(modified, now, force))
        });
        if is_stale || tile_bytes(x, y).is_none() {
            jobs.push((x, y, path, is_stale));
        }
    }
    if jobs.is_empty() {
        log::info!("tiles: las teselas ya estaban listas en memoria");
        let _ = STARTED.set(());
        return;
    }
    log::info!(
        "tiles: {} teselas del mapa por preparar desde {TILE_CONTINENT}/{TILE_FLOOR}",
        jobs.len()
    );

    let spawned = crate::background::spawn("gw2lurker-tiles", move || {
        let transport = WinHttpTransport::default();
        let mut prepared = 0;
        let mut saved = 0;
        for (x, y, path, refresh) in jobs {
            if crate::background::is_stopping() {
                break;
            }
            if let Ok(bytes) = std::fs::read(&path) {
                if tile_data()
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .insert(x, y, bytes)
                {
                    prepared += 1;
                }
            }
            if !refresh && tile_bytes(x, y).is_some() {
                continue;
            }

            let url = tile_url(x, y);
            match gw2lurker_api::Transport::get(&transport, &url) {
                Ok(bytes) => {
                    if crate::background::is_stopping() {
                        break;
                    }
                    let inserted = tile_data()
                        .lock()
                        .unwrap_or_else(|error| error.into_inner())
                        .insert(x, y, bytes.clone());
                    if !inserted {
                        log::warn!("tiles: respuesta demasiado pequeña de {url}");
                        continue;
                    }
                    prepared += 1;
                    if let Some(dir) = path.parent() {
                        if let Err(error) = std::fs::create_dir_all(dir) {
                            log::warn!("tiles: no se pudo crear {dir:?} ({error})");
                            continue;
                        }
                    }
                    // A un nombre temporal y luego renombrar: si se escribe directo, el
                    // addon puede ver el fichero a medias y guardar una tesela dañada.
                    let part = path.with_extension("part");
                    match std::fs::write(&part, &bytes) {
                        Ok(()) => match std::fs::rename(&part, &path) {
                            Ok(()) => saved += 1,
                            Err(error) => {
                                let _ = std::fs::remove_file(&part);
                                log::warn!("tiles: no se pudo guardar {path:?} ({error})")
                            }
                        },
                        Err(error) => {
                            log::warn!("tiles: no se pudo guardar {path:?} ({error})")
                        }
                    }
                }
                Err(error) => log::warn!("tiles: {url} ({error})"),
            }
        }
        if prepared > 0 {
            TILE_REVISION.fetch_add(1, Ordering::Relaxed);
        }
        log::info!("tiles: {prepared} teselas listas en memoria, {saved} guardadas en caché");
    });
    match spawned {
        Ok(()) => {
            let _ = STARTED.set(());
        }
        Err(error) => {
            log::warn!("tiles: no se pudo lanzar la descarga ({error})");
        }
    }
}

fn tile_needs_refresh(
    modified: std::time::SystemTime,
    now: std::time::SystemTime,
    force: bool,
) -> bool {
    force
        || now
            .duration_since(modified)
            .map_or(true, |age| age >= TILE_CACHE_TTL)
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

    #[test]
    fn rf20_transient_guild_lookup_errors_are_retried_after_a_cooldown() {
        let now = Instant::now();
        let mut failures = GuildLookupFailures::default();
        failures.record("guild", &ApiError::Transport("timeout".to_owned()), now);
        assert!(failures.should_skip("guild", now + Duration::from_millis(1_999)));
        assert!(!failures.should_skip("guild", now + Duration::from_secs(2)));
        failures.record(
            "guild",
            &ApiError::Status(503),
            now + Duration::from_secs(2),
        );
        assert!(failures.should_skip("guild", now + Duration::from_secs(5)));
        assert!(!failures.should_skip("guild", now + Duration::from_secs(6)));
        failures.record("missing", &ApiError::NotFound, now);
        assert!(failures.should_skip("missing", now + Duration::from_secs(600)));
        failures.resolved("missing");
        assert!(!failures.should_skip("missing", now));
    }

    #[test]
    fn rf20_non_retryable_guild_errors_are_not_requested_again() {
        let now = Instant::now();
        let mut failures = GuildLookupFailures::default();
        for error in [ApiError::Status(400), ApiError::Parse("invalid".to_owned())] {
            failures.record("guild", &error, now);
            assert!(failures.should_skip("guild", now + Duration::from_secs(600)));
        }
    }

    #[test]
    fn rf20_static_catalogues_retry_after_a_cooldown_not_every_poll() {
        let now = Instant::now();
        assert!(static_data_retry_due(None, now));
        assert!(!static_data_retry_due(Some(now), now));
        assert!(static_data_retry_due(Some(now - STATIC_DATA_RETRY), now));
    }

    #[test]
    fn p05_seared_map_tiles_are_always_refreshed() {
        let now = std::time::UNIX_EPOCH + Duration::from_secs(10 * 24 * 60 * 60);
        assert!(tile_needs_refresh(now - Duration::from_secs(60), now, true));
    }

    #[test]
    fn p06_cached_tiles_expire_daily_but_recent_tiles_are_reused() {
        let now = std::time::UNIX_EPOCH + Duration::from_secs(10 * 24 * 60 * 60);
        assert!(tile_needs_refresh(
            now - Duration::from_secs(25 * 60 * 60),
            now,
            false
        ));
        assert!(!tile_needs_refresh(
            now - Duration::from_secs(23 * 60 * 60),
            now,
            false
        ));
    }

    #[test]
    fn rf37_renderer_keeps_a_safe_snapshot_while_worker_replaces_tile_bytes() {
        let mut cache = TileDataCache::default();
        let original = vec![1; MIN_TILE_BYTES as usize];
        let refreshed = vec![2; MIN_TILE_BYTES as usize];

        assert!(cache.insert(5, 11, original.clone()));
        let render_snapshot = cache.get(5, 11).expect("worker inserted tile bytes");
        assert!(cache.insert(5, 11, refreshed.clone()));

        assert_eq!(render_snapshot.as_ref(), original);
        assert_eq!(cache.get(5, 11).as_deref(), Some(refreshed.as_slice()));
    }
}
