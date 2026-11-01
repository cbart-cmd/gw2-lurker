//! Polling worker (T-11c, RF-20, RNF-04, RNF-05).
//!
//! One background thread owns every network call and every write to `session-*.jsonl`. The
//! render callback only reads a small derived [`Dashboard`] behind a mutex, so rule 6 (no
//! network, no file system in the render thread) holds and the frame never waits on HTTP.
//!
//! Poll size matters: while the match is unknown the worker asks for `ids=all` (~400 KB), at most
//! once per [`ALL_MATCHUPS_TTL`], and once the team is confirmed it switches to
//! `/wvw/matches/{id}` (~44 KB). Only that request follows the configured interval (5 s by
//! default, RNF-02): 0,2 requests/s against an API bucket of 300 that refills at 5 per second
//! per IP (wiki *API:Best practices*). HTTP 429 responses still trigger exponential backoff.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use gw2lurker_api::{Api, ApiError, Backoff};
use gw2lurker_core::diff::{Differ, Event};
use gw2lurker_core::matchup::{self, ClaimRow, GuildSummary, ObjectiveState};
use gw2lurker_core::model::{MapRole, Match, ObjectiveInfo};
use gw2lurker_core::select::{self, Selection};

pub use crate::alerts::{AlertKind, AlertLine, Received};
use crate::catalog::{maps_for, objectives_for, resolve_guild_names};
use crate::net::WinHttpTransport;

/// Claims kept in memory for the current tracking cycle (RF-72).
pub(crate) const MAX_HISTORY: usize = 2000;

/// Cap used by both the shared exponential backoff and the configured polling wait.
const MAX_BACKOFF: Duration = Duration::from_secs(300);

/// Whether the worker may use the network (set from the render callback's gate).
static MAY_POLL: AtomicBool = AtomicBool::new(false);
/// When the UI last asked for the list of live teams (the picker or the guild search).
///
/// A timestamp and not a flag: the render and the options pane are two callbacks whose order in
/// the frame is not guaranteed, so «on» has to survive a moment after the UI says so.
static NEEDS_TEAMS: Mutex<Option<Instant>> = Mutex::new(None);
/// How long an ask for the team list is honoured, in seconds.
const TEAMS_WANTED_FOR: Duration = Duration::from_secs(3);
/// How often the nine matchups are fetched while the picker is on screen (~400 KB each time).
///
/// The list of teams changes at reset, not every poll: with a 5 s interval, asking for all nine
/// every time would be 4,8 MB per minute for nothing.
const ALL_MATCHUPS_TTL: Duration = Duration::from_secs(60);
/// When the nine matchups were fetched last.
static LAST_ALL: Mutex<Option<Instant>> = Mutex::new(None);
/// Set on unload so the thread leaves its loop.
static STOP: AtomicBool = AtomicBool::new(false);

/// Health of the polling, shown in the UI (RF-26).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Health {
    /// No poll has finished yet.
    #[default]
    Idle,
    /// Last poll worked.
    Ok { at: DateTime<Utc> },
    /// Last poll failed; the worker keeps trying with backoff.
    Degraded {
        since: DateTime<Utc>,
        error: ApiError,
    },
}

/// What the UI reads. Small on purpose: the render callback must never touch the raw match.
#[derive(Debug, Clone, Default)]
pub struct Dashboard {
    /// Polling health.
    pub health: Health,
    /// Match being followed.
    pub match_id: Option<String>,
    /// Selection state of the configured team (RF-03a).
    pub selection: Selection,
    /// Claims across the matchup (RF-70).
    pub claims: Vec<ClaimRow>,
    /// Every objective of the matchup with its owner, claimed or not (RF-30b).
    ///
    /// The map cannot colour objectives from `claims`: an objective captured without a claim has
    /// no row there and would be drawn as unclaimed.
    pub states: Vec<ObjectiveState>,
    /// Guilds ranked by most recent claim (RF-71).
    pub guilds: Vec<GuildSummary>,
    /// Objectives of the current language, for naming alerts.
    pub names: Vec<ObjectiveInfo>,
    /// Guild id → `[TAG] Nombre`, from `/v2/guild` (RF-20). Missing ones are shown as the id.
    pub guild_names: std::collections::BTreeMap<String, String>,
    /// Legacy world id → name, for the team picker (RF-03a).
    pub world_names: std::collections::BTreeMap<u32, String>,
    /// The four WvW maps with their boxes, for the projection (RF-30).
    pub maps: Vec<gw2lurker_core::WvwMap>,
    /// Every team playing right now, with its matchup and colour (RF-03a).
    pub candidates: Vec<gw2lurker_core::Candidate>,
    /// Map id → matchup role, so an alert knows which map it happened on.
    pub map_roles: std::collections::BTreeMap<u32, MapRole>,
    /// Claims seen during this tracking cycle, oldest first: the guild history of ADR-012.
    pub history: Vec<ClaimRow>,
    /// Start of the current tracking cycle: addon load or confirmed team reassignment.
    pub history_since: Option<DateTime<Utc>>,
    /// Most recent events, newest last (RF-19), each with **when we saw it**.
    ///
    /// The instant is stored here and not rebuilt when the line is formatted: recomputing it
    /// made every alert look brand new on every frame, so the pop-up never expired (bug of
    /// 2026-10-31).
    pub alerts: Vec<Received>,
    /// When the dashboard was last rebuilt.
    pub updated_at: Option<DateTime<Utc>>,
    /// Team the **game** says the player is on, from Mumble, when it can be told apart (T-19).
    ///
    /// If this differs from the configured team, the addon is watching a matchup the player is
    /// not in, which is what has made every alert look broken (report of 2026-10-31).
    pub played_team: Option<u32>,
    /// Whether the watched matchup comes from the game and not from the saved team.
    pub auto_detected: bool,
    /// Huella de la última copia de la partida que se ha visto, y desde cuándo no cambia.
    ///
    /// Si la API devuelve la misma copia durante minutos, el sondeo funciona y lo que está parado
    /// es ArenaNet: el widget lo dice con esto (informe del jugador, 2026-11-01).
    pub payload_digest: Option<u64>,
    /// Desde cuándo la huella no cambia.
    pub payload_since: Option<DateTime<Utc>>,
    /// When the game last saw a claim in this matchup, as the API tells it.
    ///
    /// `updated_at` is *our* poll; this is the game's own timestamp. Without it, «no recoge nada»
    /// cannot be told apart from «en esta partida no ha reclamado nadie» (report of 2026-10-31:
    /// the API had nothing new for nine minutes while other matchups were moving).
    pub last_claim_at: Option<DateTime<Utc>>,
}

impl Dashboard {
    pub(crate) fn reset_tracking_cycle(&mut self, at: DateTime<Utc>) {
        self.match_id = None;
        self.selection = Selection::Missing;
        self.claims.clear();
        self.states.clear();
        self.guilds.clear();
        self.candidates.clear();
        self.map_roles.clear();
        self.history.clear();
        self.history_since = Some(at);
        self.alerts.clear();
        self.updated_at = None;
        self.played_team = None;
        self.auto_detected = false;
        self.payload_digest = None;
        self.payload_since = None;
        self.last_claim_at = None;
    }

    /// Claims of one guild during the current tracking cycle, in the order they happened (RF-72).
    pub fn guild_history(&self, guild: &str) -> Vec<ClaimRow> {
        matchup::guild_timeline_with_current(&self.history, &self.claims, guild)
    }

    /// Latest current claim's map, or the last historical one when nothing is held (RF-71).
    pub fn last_map_of(&self, guild: &str) -> Option<MapRole> {
        matchup::last_claim_map(&self.claims, &self.history, guild)
    }

    /// Guilds that hold something right now: the ones that claimed during this tracking cycle plus
    /// the ones already holding an objective at its baseline (ADR-012).
    ///
    /// It is what the *Add guild* panel offers first: those are the guilds the player has just
    /// seen on screen, so following one is a click and no network (T-44).
    pub fn session_guilds(&self) -> Vec<(String, usize, Option<DateTime<Utc>>)> {
        let mut acc: std::collections::BTreeMap<&str, (usize, Option<DateTime<Utc>>)> =
            std::collections::BTreeMap::new();
        for row in &self.history {
            let entry = acc.entry(row.guild.as_str()).or_insert((0, None));
            entry.0 += 1;
            if row.claimed_at > entry.1 {
                entry.1 = row.claimed_at;
            }
        }
        let mut rows: Vec<(String, usize, Option<DateTime<Utc>>)> = acc
            .into_iter()
            .map(|(guild, (claims, last))| (guild.to_owned(), claims, last))
            .collect();
        // Lo más reciente arriba: es lo que se busca al añadir.
        rows.sort_by_key(|row| std::cmp::Reverse(row.2));
        rows
    }

    /// Human name of an objective, for the alert lines (T-13).
    pub fn objective_name(&self, id: &str) -> Option<&str> {
        self.names
            .iter()
            .find(|objective| objective.id == id)
            .map(|objective| objective.name.as_str())
    }

    /// Kind of an objective, for the drawing of the alert lines (T-47).
    pub(crate) fn objective_kind(&self, id: &str) -> gw2lurker_core::ObjectiveType {
        self.names
            .iter()
            .find(|objective| objective.id == id)
            .map_or(gw2lurker_core::ObjectiveType::Unknown, |known| known.kind)
    }
}

static DASHBOARD: OnceLock<Mutex<Dashboard>> = OnceLock::new();

/// Reads the dashboard. Hold the guard only while formatting what is needed.
pub fn dashboard() -> MutexGuard<'static, Dashboard> {
    DASHBOARD
        .get_or_init(|| Mutex::new(Dashboard::default()))
        .lock()
        .unwrap_or_else(|error| error.into_inner())
}

/// Tells the worker whether it may poll this frame. Called from the render callback.
pub fn set_may_poll(allowed: bool) {
    MAY_POLL.store(allowed, Ordering::Relaxed);
}

/// Tells the worker the team picker is on screen, so it fetches the list of live teams even
/// when polling is not allowed (the settings window also opens outside WvW).
pub fn set_teams_needed(needed: bool) {
    if needed {
        *NEEDS_TEAMS
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = Some(Instant::now());
    }
}

/// Whether the UI asked for the team list a moment ago.
fn teams_needed() -> bool {
    NEEDS_TEAMS
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .is_some_and(|at| at.elapsed() < TEAMS_WANTED_FOR)
}

/// Starts the worker. Idempotent: a second call does nothing.
pub fn start() {
    static STARTED: OnceLock<()> = OnceLock::new();
    STARTED.get_or_init(|| {
        // Los nombres ya resueltos están en `settings.json` (los clanes seguidos) y en `temp.json`
        // (los de la sesión): que la ventana los tenga desde el primer fotograma, también en el
        // Bastión del Armisticio, donde no se sondea nada (informe del jugador, 2026-11-01).
        let mut names = crate::config().guild_names.clone();
        names.extend(crate::session_names::all());
        dashboard().guild_names = names;
        if let Err(error) = crate::background::spawn("gw2lurker-worker", run) {
            log::warn!("worker: no se pudo arrancar el hilo: {error}");
        }
    });
}

/// Asks the worker to stop and waits for it to leave.
pub fn stop() {
    STOP.store(true, Ordering::Relaxed);
    crate::background::stop_all();
}

fn run() {
    let api = Api::new(WinHttpTransport::default());
    let mut differ = Differ::new(gw2lurker_core::DiffConfig::default());
    let mut assignment = crate::assignment::AssignmentMonitor::default();
    let mut teams_at: Option<Instant> = None;
    let mut names_at: Option<Instant> = None;
    let mut backoff = Backoff::default();

    while !STOP.load(Ordering::Relaxed) {
        crate::check_dll_update();
        if !MAY_POLL.load(Ordering::Relaxed) {
            // Outside WvW, only the team list is worth fetching, and rarely: it changes at
            // reset, not every half minute.
            if teams_needed() && teams_stale(&mut teams_at) {
                match api.matches_all() {
                    Ok(live) => publish_teams(&live, crate::team::selected()),
                    Err(error) => log::warn!("worker: sin lista de equipos ({error})"),
                }
                teams_at = Some(Instant::now());
            }
            // Y los nombres de los clanes que sigue, por si alguno se añadió aquí: la lista de
            // seguidos tiene que enseñar el nombre, no el id (informe del jugador, 2026-11-01).
            if names_stale(&mut names_at) {
                let followed = crate::runtime().followed.clone();
                let names = resolve_guild_names(&api, &[], &followed);
                dashboard().guild_names = names;
                names_at = Some(Instant::now());
            }
            sleep(Duration::from_millis(500));
            continue;
        }

        let backoff_delay = match poll_once(&api, &mut differ, &mut assignment) {
            Ok(()) => {
                backoff.reset();
                Duration::ZERO
            }
            Err(error) => {
                if STOP.load(Ordering::Relaxed) {
                    break;
                }
                let delay = retry_delay(&mut backoff, &error);
                log::warn!(
                    "worker: {error}; siguiente sondeo en {} ms",
                    delay.as_millis()
                );
                let mut board = dashboard();
                board.health = Health::Degraded {
                    since: started_degraded(&board.health),
                    error,
                };
                delay
            }
        };

        let interval = Duration::from_secs(u64::from(crate::runtime().poll_secs));
        let wait = interval.max(backoff_delay).min(MAX_BACKOFF);
        sleep(wait);
    }

    log::info!("worker: detenido");
}

/// How long the team list fetched outside WvW stays valid. Teams only change at reset.
const TEAMS_TTL: Duration = Duration::from_secs(600);

fn teams_stale(last: &mut Option<Instant>) -> bool {
    match *last {
        Some(at) => at.elapsed() >= TEAMS_TTL,
        None => true,
    }
}

/// How long the guild names resolved outside WvW stay fresh (names do not change).
const NAMES_TTL: Duration = Duration::from_secs(600);

fn names_stale(last: &mut Option<Instant>) -> bool {
    match *last {
        Some(at) => at.elapsed() >= NAMES_TTL,
        None => true,
    }
}

/// Fills the dashboard with the teams of the live matches, and nothing else.
fn publish_teams(live: &[Match], selected: Option<u32>) {
    let mut board = dashboard();
    board.candidates = select::candidates(live);
    board.selection = select::evaluate(selected, live);
    if let Some(match_id) = selected_match_id(&board.selection) {
        board.match_id = Some(match_id.to_owned());
    }
}

fn started_degraded(health: &Health) -> DateTime<Utc> {
    match health {
        Health::Degraded { since, .. } => *since,
        _ => Utc::now(),
    }
}

fn retry_delay(backoff: &mut Backoff, error: &ApiError) -> Duration {
    if Backoff::should_retry(error) {
        Duration::from_millis(backoff.next_delay_ms())
    } else {
        backoff.reset();
        Duration::ZERO
    }
}

/// Sleeps in slices so `stop()` and a gate change are noticed quickly.
fn sleep(total: Duration) {
    let step = Duration::from_millis(250);
    let mut left = total;
    while left > Duration::ZERO && !STOP.load(Ordering::Relaxed) {
        crate::check_dll_update();
        crate::keybinds::refresh();
        let chunk = left.min(step);
        std::thread::sleep(chunk);
        left -= chunk;
    }
}

fn poll_once(
    api: &Api<WinHttpTransport>,
    differ: &mut Differ,
    assignment: &mut crate::assignment::AssignmentMonitor,
) -> Result<(), ApiError> {
    let saved = crate::team::selected();
    // La clave es la cuenta (RF-03a): cambiar de personaje dentro de ella no cambia de equipo ni
    // obliga a revalidar; entrar con otra cuenta sí, una sola vez.
    if identity_changed() {
        crate::detect::forget();
    }
    // Con el selector en pantalla hacen falta los nueve emparejamientos, pero como mucho una vez
    // por minuto: la lista de equipos solo cambia al reiniciarse el emparejamiento y a 5 s serían
    // 4,8 MB por minuto (informe del jugador, 2026-10-31).
    let previous_match_id = watched_match_id();
    let assignment_due = if let Some(match_id) = previous_match_id.as_deref() {
        match assignment.rotation_confirmed(api, match_id, Utc::now(), Instant::now()) {
            Ok(due) => due,
            Err(error) => {
                log::warn!("worker: no se pudo comprobar la reasignación de equipos ({error})");
                false
            }
        }
    } else {
        false
    };
    let needs_all =
        needs_all_now(teams_needed(), crate::detect::due(), all_stale()) || assignment_due;
    // Lo que se pide es la partida del equipo que **se va a vigilar**, no la del guardado: con
    // la detección cacheada un cuarto de hora, pedir la del guardado y evaluar después el equipo
    // detectado dejaba al addon mirando otra partida 14 de cada 15 minutos (informe de
    // 2026-10-31).
    let polled = polled_team(saved, crate::detect::cached().map(|found| found.team));
    let known_match = previous_match_id;
    if should_skip_poll(needs_all, known_match.as_deref(), polled, all_stale()) {
        return Ok(());
    }
    let live = fetch_live(
        api,
        fetch_plan(needs_all, known_match.as_deref(), polled),
        polled,
    )?;
    if crate::background::is_stopping() {
        return Ok(());
    }

    // El juego manda: si Mumble dice en qué partida está el jugador, se vigila ésa y no la del
    // equipo guardado. Es la causa de todos los «no salen avisos» (informe de 2026-10-31).
    let detected = if needs_all {
        crate::detect::detect_in(&live, saved)
    } else {
        crate::detect::cached()
    };
    let team_id = detected.as_ref().map(|found| found.team).or(saved);

    let selection = select::evaluate(team_id, &live);
    let names = match objectives_for(api) {
        Ok(names) => names,
        Err(error) => {
            log::warn!("worker: sin nombres de objetivo ({error}); se usan los ids");
            Vec::new()
        }
    };
    if crate::background::is_stopping() {
        return Ok(());
    }
    let match_id = selected_match_id(&selection).map(str::to_owned);
    let assignment_reset_at = Utc::now();
    let assignment_reset = if assignment_due {
        let mut board = dashboard();
        crate::assignment::reset_tracking_for_assignment(
            &mut board,
            differ,
            assignment,
            known_match.as_deref(),
            match_id.as_deref(),
            assignment_reset_at,
        )
    } else {
        false
    };
    if assignment_reset {
        log::info!("worker: reasignación confirmada; historial y alertas reiniciados");
    }

    // Diff first: the dashboard lock must not be taken while building the history.
    let mut events = Vec::new();
    let mut claims = Vec::new();
    let mut states = Vec::new();
    let mut guilds = Vec::new();
    let mut new_history = Vec::new();
    let roles_of_match: std::collections::BTreeMap<u32, MapRole> = live
        .iter()
        .find(|entry| Some(entry.id.as_str()) == match_id.as_deref())
        .map(|entry| entry.maps.iter().map(|map| (map.id, map.role)).collect())
        .unwrap_or_default();

    if let Some(entry) = current_match(&live, match_id.as_deref()) {
        events = differ.push(gw2lurker_core::diff::Snapshot::from_match(
            entry,
            Utc::now(),
        ));
        claims = matchup::claims(entry, &names);
        states = matchup::states(entry);
        guilds = matchup::rank_guilds(&claims);
        new_history = events
            .iter()
            .filter_map(|event| history_row(event, entry, &names, &roles_of_match))
            .collect();
    }

    let now = Utc::now();
    let candidates = select::candidates(&live);
    let followed = crate::runtime().followed.clone();
    let guild_names = resolve_guild_names(api, &guilds, &followed);
    if crate::background::is_stopping() {
        return Ok(());
    }
    let world_names = crate::catalog::world_names(api);
    if crate::background::is_stopping() {
        return Ok(());
    }
    let maps = maps_for(api);
    if crate::background::is_stopping() {
        return Ok(());
    }
    crate::diagnostics::log_watched_matchup(&selection, match_id.as_deref(), &live, &events);
    crate::diagnostics::log_where(team_id, match_id.as_deref(), &live);
    crate::catalog::ensure_tiles(&maps);
    let map_roles = match_id
        .as_deref()
        .and_then(|id| live.iter().find(|entry| entry.id == id))
        .map(|entry| {
            entry
                .maps
                .iter()
                .map(|map| (map.id, map.role))
                .collect::<std::collections::BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let mut board = dashboard();
    board.health = Health::Ok { at: now };
    board.history_since.get_or_insert(now);
    // Huella de la partida: si la API repite la misma copia, se guarda desde cuándo.
    if let Some(entry) = current_match(&live, match_id.as_deref()) {
        let digest = matchup::digest(entry);
        if board.payload_digest != Some(digest) {
            board.payload_digest = Some(digest);
            board.payload_since = Some(now);
        } else if board.payload_since.is_none() {
            board.payload_since = Some(now);
        }
    }
    board.selection = selection;
    board.match_id = match_id.clone();
    board.names = names;
    board.candidates = candidates;
    board.guild_names = guild_names;
    board.world_names = world_names;
    board.maps = maps;
    board.map_roles = map_roles;
    let objective_count = board.names.len();
    board.claims = claims;
    let claim_count = board.claims.len();
    board.states = states;
    board.guilds = guilds;
    for event in &events {
        board.push_alert(event.clone());
    }
    board.history.extend(new_history);
    if board.history.len() > MAX_HISTORY {
        let excess = board.history.len() - MAX_HISTORY;
        board.history.drain(..excess);
    }
    board.updated_at = Some(now);
    board.last_claim_at = crate::diagnostics::last_claim_of(&live, match_id.as_deref());
    board.played_team = detected.as_ref().map(|found| found.team);
    board.auto_detected = detected
        .as_ref()
        .is_some_and(|found| Some(found.team) != saved);
    let board_last_claim = board.last_claim_at;
    drop(board);
    crate::diagnostics::heartbeat(
        events.len(),
        claim_count,
        objective_count,
        match_id.as_deref(),
        board_last_claim,
    );
    Ok(())
}

/// Whether this poll has to ask for the nine matchups (~400 KB) or just for one match (~44 KB).
///
/// All nine are needed for the initial detection attempt, when missing Mumble data is due for
/// retry, or when the picker is on screen **and** the list is stale. Asking for all nine on every
/// poll was 4,8 MB per minute at the five-second cadence.
fn needs_all_now(picker: bool, detect_due: bool, all_stale: bool) -> bool {
    detect_due || (picker && all_stale)
}

fn should_skip_poll(
    needs_all: bool,
    known_match: Option<&str>,
    polled: Option<u32>,
    all_stale: bool,
) -> bool {
    if needs_all || known_match.is_some() {
        return false;
    }
    // Sin id de partida, buscar el equipo es `ids=all` (~400 KB): la cadencia configurada es solo
    // para el estado de los mapas de la partida elegida, así que esto espera al enfriamiento.
    polled.is_none() || !all_stale
}

/// Whether the nine matchups were fetched longer than [`ALL_MATCHUPS_TTL`] ago.
fn all_stale() -> bool {
    LAST_ALL
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .is_none_or(|at| at.elapsed() >= ALL_MATCHUPS_TTL)
}

/// Whether the identity changed since the last poll, which re-arms the one sweep of the nine
/// matchups: the team is remembered per account (RF-03a), so changing character inside the same
/// account does not need a sweep.
fn identity_changed() -> bool {
    static LAST: Mutex<Option<String>> = Mutex::new(None);
    let Some(key) = crate::team::current_key() else {
        return false;
    };
    let mut last = LAST.lock().unwrap_or_else(|error| error.into_inner());
    if last.as_deref() == Some(key.as_str()) {
        return false;
    }
    let changed = last.is_some();
    *last = Some(key);
    changed
}

/// Remembers that this poll fetched the nine matchups.
fn remember_all() {
    *LAST_ALL.lock().unwrap_or_else(|error| error.into_inner()) = Some(Instant::now());
}

/// Team whose match the poll asks for: the one the game gave, or the saved one as before.
///
/// The detection is cached for a quarter of an hour, so after a successful detection the save
/// in the settings can be a different team; asking for the saved match and then evaluating the
/// detected team found it nowhere and fell back to that same wrong match.
fn polled_team(saved: Option<u32>, detected: Option<u32>) -> Option<u32> {
    detected.or(saved)
}

/// What one poll has to ask the API for.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Fetch {
    /// The nine matchups: the picker is on screen, or it is time to look for the player's match
    /// again (`~400 KB`).
    All,
    /// The match we already know, by id: `~44 KB` with its **four maps**, which is what makes a
    /// five-second poll affordable.
    ById(String),
    /// The match of a team whose id is not known yet.
    Team(u32),
}

/// Which request a poll needs. Pure, so the rule can be tested without the network.
fn fetch_plan(needs_all: bool, known: Option<&str>, polled: Option<u32>) -> Fetch {
    if needs_all {
        return Fetch::All;
    }
    match known {
        Some(id) => Fetch::ById(id.to_owned()),
        None => polled.map_or(Fetch::All, Fetch::Team),
    }
}

/// The match the poll is watching, from the last one published (cheap: one clone of an id).
fn watched_match_id() -> Option<String> {
    dashboard().match_id.clone()
}

/// Runs the plan. Every fallback ends in `ids=all`: it is the only request that can find a match
/// whose id or team we no longer know.
fn fetch_live(
    api: &Api<WinHttpTransport>,
    plan: Fetch,
    polled: Option<u32>,
) -> Result<Vec<Match>, ApiError> {
    match plan {
        Fetch::All => {
            let live = api.matches_all()?;
            remember_all();
            Ok(live)
        }
        Fetch::ById(id) => {
            let found = api.match_by_id(&id)?;
            // La partida guardada puede haber terminado o el jugador haber cambiado de equipo:
            // si el equipo que se vigila no está ahí, se busca otra vez.
            if polled.is_none_or(|team| found.color_of(team).is_some()) {
                return Ok(vec![found]);
            }
            fetch_by_team(api, polled)
        }
        Fetch::Team(_) => fetch_by_team(api, polled),
    }
}

/// The match of a team, or every matchup when that team no longer plays (the ladder reshuffled,
/// so the picker has to offer the live teams again).
fn fetch_by_team(api: &Api<WinHttpTransport>, polled: Option<u32>) -> Result<Vec<Match>, ApiError> {
    if crate::background::is_stopping() {
        return Ok(Vec::new());
    }
    let live = api.matches_all()?;
    remember_all();
    Ok(matching_team_or_all(live, polled))
}

fn matching_team_or_all(live: Vec<Match>, polled: Option<u32>) -> Vec<Match> {
    if let Some(team) = polled {
        if let Some(found) = live.iter().find(|entry| entry.color_of(team).is_some()) {
            return vec![found.clone()];
        }
    }
    live
}

/// Turns a claim event into a history row, resolving name, kind and owner from the snapshot.
///
/// Free function on purpose: the caller has the match snapshot and the names at hand, and this
/// keeps the dashboard lock out of the loop that also writes to disk.
fn history_row(
    event: &Event,
    entry: &Match,
    names: &[ObjectiveInfo],
    map_roles: &std::collections::BTreeMap<u32, MapRole>,
) -> Option<ClaimRow> {
    let Event::Claim {
        objective,
        guild,
        at,
        ..
    } = event
    else {
        return None;
    };
    // Los reclamos que ya existían al cargar **sí** entran en el historial, con su `claimed_at`
    // real: ADR-012 punto 2 pide que «la ruta empiece con el mapa tal y como está». Dejarlos
    // fuera hacía que un clan seguido que ya tenía un campa apareciera sin historial ni ruta
    // hasta que volviera a reclamar (informe de 2026-10-31). Lo que no entran es en los avisos:
    // de eso se encarga `AlertLine::late` en `recent_alerts`.
    let map_id = objective
        .split_once('-')
        .and_then(|(map, _)| map.parse::<u32>().ok())?;
    let role = map_roles
        .get(&map_id)
        .copied()
        .unwrap_or_else(|| MapRole::from_map_id(map_id));
    if role == MapRole::Unknown {
        return None;
    }
    let info = names.iter().find(|known| &known.id == objective);
    let found = entry
        .maps
        .iter()
        .flat_map(|map| map.objectives.iter())
        .find(|candidate| &candidate.id == objective);
    let owner = found.and_then(|candidate| candidate.owner);

    Some(ClaimRow {
        map_id,
        map_role: role,
        objective_id: objective.clone(),
        name: info.map(|known| known.name.clone()),
        kind: info.map_or(gw2lurker_core::ObjectiveType::Unknown, |known| known.kind),
        owner,
        guild: guild.clone(),
        claimed_at: Some(*at),
        points_tick: found.map_or(0, |candidate| candidate.points_tick),
    })
}

fn current_match<'a>(live: &'a [Match], id: Option<&str>) -> Option<&'a Match> {
    let id = id?;
    live.iter().find(|entry| entry.id == id)
}

fn selected_match_id(selection: &Selection) -> Option<&str> {
    match selection {
        Selection::Confirmed { match_id, .. } => Some(match_id),
        Selection::Missing | Selection::Stale { .. } => None,
    }
}

#[cfg(test)]
mod tests;
