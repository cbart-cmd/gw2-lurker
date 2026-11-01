//! Guild search for the follow selector (T-44).
//!
//! The search cannot run in the render callback (rule 6: no network there), so it goes on its
//! own thread and the window only reads the state. The big index of names (`guild-names.json`,
//! thousands of entries) is read **here** and dropped as soon as it is filtered, so the render
//! never walks it (request of 2026-11-01).
//!
//! Two modes: [`Mode::Local`] only reads the index (instantaneous, no network) and is what a
//! keystroke triggers; [`Mode::Full`] adds the API calls:
//!
//! - `/v2/guild/search?name=` matches **a single word, by prefix and case-insensitive**: `Raven`
//!   finds *Legends Of Ravenous…* and `avenous` does not. Multi-word text returns nothing, so the
//!   window says "one word" instead of pretending otherwise (verified 2026-10-31).
//! - `/v2/guild/:id` is one call per id: it gives the name and the tag.
//! - `/v2/wvw/guilds/:region` maps guild → team and is ~280 KB, so it is cached for a while.

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use gw2lurker_api::{Api, ApiError};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Failure {
    Api(ApiError),
    Internal(String),
}

impl From<ApiError> for Failure {
    fn from(error: ApiError) -> Self {
        Self::Api(error)
    }
}

/// What the window draws while the thread works.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum State {
    #[default]
    Idle,
    /// Búsqueda en marcha. **Guarda los resultados anteriores**: si se vaciara la lista, el clic
    /// que el jugador acaba de hacer caería en el aire, porque la fila desaparecería en el mismo
    /// fotograma en el que se pulsa (informe del jugador, 2026-11-01: «el primer clic no añade el
    /// clan, tengo que darle otra vez»).
    Searching(Vec<Hit>),
    Done(Vec<Hit>),
    Failed(Failure),
}

impl State {
    /// Hits on screen right now, whatever the state: lo que se puede clicar mientras se busca.
    pub fn hits(&self) -> &[Hit] {
        match self {
            State::Searching(hits) | State::Done(hits) => hits,
            State::Idle | State::Failed(_) => &[],
        }
    }
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
pub fn start(region: Region, team: Option<u32>, text: String, mode: Mode) -> bool {
    {
        let mut state = slot().lock().unwrap_or_else(|error| error.into_inner());
        if matches!(*state, State::Searching(_)) {
            return false;
        }
        // Los resultados que hay en pantalla se quedan: mientras llega la nueva búsqueda la lista
        // sigue siendo clicable (ver [`State::Searching`]).
        *state = State::Searching(state.hits().to_vec());
    }

    let spawned = crate::background::spawn("gw2lurker-search", move || {
        let outcome = run(region, team, &text, mode);
        if crate::background::is_stopping() {
            return;
        }
        let mut state = slot().lock().unwrap_or_else(|error| error.into_inner());
        *state = match outcome {
            Ok(hits) => State::Done(hits),
            Err(error) => State::Failed(error),
        };
    });

    if let Err(error) = spawned {
        *slot().lock().unwrap_or_else(|error| error.into_inner()) =
            State::Failed(Failure::Internal(error.to_string()));
        return false;
    }
    true
}

/// What a search is allowed to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Solo el índice del disco: instantáneo y sin red (lo que se lanza al escribir).
    Local,
    /// El índice **y** el API (lo que se lanza con Enter o el botón).
    Full,
}

fn run(region: Region, team: Option<u32>, text: &str, mode: Mode) -> Result<Vec<Hit>, Failure> {
    // El índice se lee aquí, en este hilo, y se suelta en cuanto se filtra: son miles de entradas
    // y el hilo de render no las toca nunca (petición del jugador, 2026-11-01).
    let index = crate::names::read();
    let local = crate::names::search_in(&index, text, MAX_HITS);
    drop(index);
    log::debug!("search: {} en el índice local para «{text}»", local.len());

    if mode == Mode::Local {
        return Ok(local
            .into_iter()
            .map(|(id, label)| Hit {
                id,
                label,
                team: None,
                matches_team: true,
            })
            .collect());
    }

    let api = Api::new(WinHttpTransport::default());
    let ctx = format!("«{text}» en {region:?}");
    let teams = teams_of(region, &api)?;

    let mut hits: Vec<Hit> = local
        .into_iter()
        .map(|(id, label)| {
            let of_team = teams.get(&id).copied();
            Hit {
                matches_team: team.is_none_or(|wanted| of_team == Some(wanted)),
                id,
                label,
                team: of_team,
            }
        })
        .collect();

    let ids = api.guild_search(text)?;
    for raw_id in ids.into_iter().take(MAX_LOOKUPS) {
        // Un id de clan es un UUID. Cualquier otra cosa que devuelva el índice se descarta aquí:
        // pedirla a `/v2/guild/:id` solo produce un error opaco de la API (informe de 2026-10-31,
        // «invalid type string …»).
        let Some(id) = gw2lurker_core::normalize_guild_id(&raw_id) else {
            log::warn!("search: la API devolvió {raw_id}, que no es un id de clan; se ignora");
            continue;
        };
        if hits.iter().any(|hit| hit.id == id) {
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

/// Guild → team of a region, with the moment it was fetched.
type TeamCache = Option<(Region, Instant, BTreeMap<String, u32>)>;

/// Guild → team of a region, cached because the answer is ~280 KB and only changes at reset.
fn teams_of(
    region: Region,
    api: &Api<WinHttpTransport>,
) -> Result<BTreeMap<String, u32>, ApiError> {
    static CACHE: OnceLock<Mutex<TeamCache>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(None));
    let mut guard = cache.lock().unwrap_or_else(|error| error.into_inner());

    if let Some((cached, at, teams)) = guard.as_ref() {
        if *cached == region && at.elapsed() < TEAMS_TTL {
            return Ok(teams.clone());
        }
    }

    let teams = api.wvw_guilds(region)?;
    *guard = Some((region, Instant::now(), teams.clone()));
    Ok(teams)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gw2lurker_api::looks_like_guild_id;

    #[test]
    fn t44_a_search_is_not_started_on_top_of_another() {
        *slot().lock().unwrap() = State::Searching(Vec::new());

        assert!(!start(
            Region::Eu,
            Some(12014),
            "lord".to_owned(),
            Mode::Local
        ));
        assert_eq!(
            snapshot(),
            State::Searching(Vec::new()),
            "el estado no se pisa"
        );

        *slot().lock().unwrap() = State::Idle;
    }

    #[test]
    fn t44_the_results_on_screen_survive_a_new_search() {
        // El clic del jugador caía en el aire: al lanzar otra búsqueda la lista se vaciaba en el
        // mismo fotograma del clic, así que el primer clic no añadía el clan y hacía falta un
        // segundo (informe de 2026-11-01).
        let hit = Hit {
            id: "114D20B6-6E89-4432-982C-C77B58F57460".to_owned(),
            label: "[YaK] Dolyak Sanctuary".to_owned(),
            team: None,
            matches_team: true,
        };
        *slot().lock().unwrap() = State::Done(vec![hit.clone()]);

        {
            // Lo mismo que hace `start` antes de lanzar el hilo, sin lanzarlo: un test no toca
            // disco ni red.
            let mut state = slot().lock().unwrap();
            let previous = state.hits().to_vec();
            *state = State::Searching(previous);
        }

        assert_eq!(
            snapshot().hits(),
            &[hit],
            "los resultados siguen en pantalla (y clicables) mientras se busca"
        );

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
