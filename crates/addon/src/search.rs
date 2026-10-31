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
pub fn start(region: Region, team: u32, text: String) -> bool {
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

fn run(region: Region, team: u32, text: &str) -> Result<Vec<Hit>, String> {
    let api = Api::new(WinHttpTransport::default());
    let ids = api.guild_search(text).map_err(|error| error.to_string())?;
    let teams = teams_of(region, &api)?;

    let mut hits = Vec::new();
    for id in ids.into_iter().take(MAX_LOOKUPS) {
        // Only the team asked for: that is the whole point of the team selector.
        if teams.get(&id) != Some(&team) {
            continue;
        }
        match api.guild(&id) {
            Ok(info) => hits.push(Hit {
                id,
                label: crate::catalog::guild_label(&info),
            }),
            Err(error) => log::debug!("search: sin nombre de {id} ({error})"),
        }
        if hits.len() >= MAX_HITS {
            break;
        }
    }
    log::info!(
        "search: {} resultados para «{text}» en el equipo {team}",
        hits.len()
    );
    Ok(hits)
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

        assert!(!start(Region::Eu, 12014, "lord".to_owned()));
        assert_eq!(snapshot(), State::Searching, "el estado no se pisa");

        *slot().lock().unwrap() = State::Idle;
    }
}
