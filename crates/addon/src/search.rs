//! Guild search for the follow selector (T-44).
//!
//! The search cannot run in the render callback (rule 6: no network there), so it goes on its
//! own thread and the window only reads the state. Three API calls are involved and each one has
//! its catch:
//!
//! - `/v2/guild/search?name=` matches **a single word, by prefix and case-insensitive**: `Raven`
//!   finds *Legends Of Ravenous…* and `avenous` does not. Multi-word text returns nothing, so the
//!   window says "one word" instead of pretending otherwise (verified 2026-10-31).
//! - `/v2/guild/:id` is one call per id: it gives the name and the tag.
//! - `/v2/wvw/guilds/:region` maps guild → team and is ~280 KB, so it is cached for a while.

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use gw2lurker_api::Api;
use gw2lurker_core::Region;

use crate::net::WinHttpTransport;

/// Ids looked up per search. One call each, and the answer rarely needs more than a handful.
const MAX_LOOKUPS: usize = 20;

/// Hits kept per search.
const MAX_HITS: usize = 8;

/// How long the guild → team map is considered fresh (it only changes at reset).
const TEAMS_TTL: Duration = Duration::from_secs(30 * 60);

/// One guild found by the search.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    pub id: String,
    /// `[TAG] Nombre`.
    pub label: String,
    /// Team it plays for right now, when the map of the region knows it.
    pub team: Option<u32>,
    /// Whether it plays for the team the player picked (the rest are shown after these).
    pub matches_team: bool,
}

/// What the window draws while the thread works.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum State {
    #[default]
    Idle,
    Searching,
    Done(Vec<Hit>),
    Failed(String),
}

static STATE: OnceLock<Mutex<State>> = OnceLock::new();

fn slot() -> &'static Mutex<State> {
    STATE.get_or_init(|| Mutex::new(State::Idle))
}

/// Current state, copied so the window never holds the lock while drawing.
pub fn snapshot() -> State {
    slot()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clone()
}

/// Starts a search on its own thread. Returns `false` if one is already running.
///
/// `team` is **optional**: the region is what the search needs to look in the right half of the
/// world, and asking for a team as well left the button doing nothing whenever the list of live
/// teams had not arrived (bug of 2026-10-31).
pub fn start(region: Region, team: Option<u32>, text: String) -> bool {
    {
        let mut state = slot().lock().unwrap_or_else(|error| error.into_inner());
        if matches!(*state, State::Searching) {
            return false;
        }
        *state = State::Searching;
    }

    let spawned = std::thread::Builder::new()
        .name("gw2lurker-search".to_owned())
        .spawn(move || {
            let outcome = run(region, team, &text);
            let mut state = slot().lock().unwrap_or_else(|error| error.into_inner());
            *state = match outcome {
                Ok(hits) => State::Done(hits),
                Err(error) => State::Failed(error),
            };
        });

    if spawned.is_err() {
        *slot().lock().unwrap_or_else(|error| error.into_inner()) =
            State::Failed("no se pudo lanzar la búsqueda".to_owned());
        return false;
    }
    true
}

fn run(region: Region, team: Option<u32>, text: &str) -> Result<Vec<Hit>, String> {
    let api = Api::new(WinHttpTransport::default());
    let ctx = format!("«{text}» en {region:?}");
    let ids = api
        .guild_search(text)
        .map_err(|error| format!("{ctx} (búsqueda): {error}"))?;
    let teams = teams_of(region, &api).map_err(|error| format!("{ctx} (equipos): {error}"))?;

    let mut hits = Vec::new();
    for id in ids.into_iter().take(MAX_LOOKUPS) {
        // Un id de clan es un UUID. Cualquier otra cosa que devuelva el índice se descarta aquí:
        // pedirla a `/v2/guild/:id` solo produce un error opaco de la API (informe de 2026-10-31,
        // «invalid type string …»).
        if !looks_like_guild_id(&id) {
            log::warn!("search: la API devolvió {id}, que no es un id de clan; se ignora");
            continue;
        }
        let of_team = teams.get(&id).copied();
        // El equipo elegido **no descarta** resultados: solo ordena. Descartándolos, buscar un
        // clan propio con el equipo por defecto decía «no existe» (informe de 2026-10-31).
        let matches_team = team.is_none_or(|wanted| of_team == Some(wanted));
        match api.guild(&id) {
            Ok(info) => hits.push(Hit {
                id,
                label: crate::catalog::guild_label(&info),
                team: of_team,
                matches_team,
            }),
            Err(error) => log::warn!("search: sin nombre de {id} ({error})"),
        }
        if hits.len() >= MAX_HITS {
            break;
        }
    }
    // Los del equipo elegido primero: el jugador está buscando el suyo.
    hits.sort_by_key(|hit| !hit.matches_team);
    log::info!(
        "search: {} resultados para {ctx} con equipo {team:?}",
        hits.len()
    );
    Ok(hits)
}

/// Whether a string has the shape of a guild id: 32 hexadecimal digits, with or without dashes.
///
/// `/v2/guild/<id>` answers with an opaque error for anything else, so the shape is checked
/// before spending a request on it.
fn looks_like_guild_id(id: &str) -> bool {
    let mut digits = 0;
    for character in id.chars() {
        if character == '-' {
            continue;
        }
        if !character.is_ascii_hexdigit() {
            return false;
        }
        digits += 1;
    }
    digits == 32
}

/// Guild → team of a region, with the moment it was fetched.
type TeamCache = Option<(Region, Instant, BTreeMap<String, u32>)>;

/// Guild → team of a region, cached because the answer is ~280 KB and only changes at reset.
fn teams_of(region: Region, api: &Api<WinHttpTransport>) -> Result<BTreeMap<String, u32>, String> {
    static CACHE: OnceLock<Mutex<TeamCache>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(None));
    let mut guard = cache.lock().unwrap_or_else(|error| error.into_inner());

    if let Some((cached, at, teams)) = guard.as_ref() {
        if *cached == region && at.elapsed() < TEAMS_TTL {
            return Ok(teams.clone());
        }
    }

    let teams = api.wvw_guilds(region).map_err(|error| error.to_string())?;
    *guard = Some((region, Instant::now(), teams.clone()));
    Ok(teams)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t44_a_search_is_not_started_on_top_of_another() {
        *slot().lock().unwrap() = State::Searching;

        assert!(!start(Region::Eu, Some(12014), "lord".to_owned()));
        assert_eq!(snapshot(), State::Searching, "el estado no se pisa");

        *slot().lock().unwrap() = State::Idle;
    }

    #[test]
    fn t44_only_guild_ids_are_looked_up() {
        assert!(looks_like_guild_id("114D20B6-6E89-4432-982C-C77B58F57460"));
        assert!(looks_like_guild_id("114d20b66e894432982cc77b58f57460"));
        assert!(
            !looks_like_guild_id("11004"),
            "un id de equipo no es un id de clan: pedirlo da un error opaco"
        );
        assert!(!looks_like_guild_id(""), "ni la cadena vacía");
        assert!(
            !looks_like_guild_id("114D20B6-6E89-4432-982C"),
            "ni medio id"
        );
    }
}
