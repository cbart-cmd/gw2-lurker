//! Polling worker (T-11c, RF-20, RNF-04, RNF-05).
//!
//! One background thread owns every network call and every write to `session-*.jsonl`. The
//! render callback only reads a small derived [`Dashboard`] behind a mutex, so rule 6 (no
//! network, no file system in the render thread) holds and the frame never waits on HTTP.
//!
//! Poll size matters: while the match is unknown the worker asks for `ids=all` (~400 KB), and
//! once the team is selected it switches to `/wvw/matches/{id}` (~44 KB), which at one poll
//! every 30 s keeps us well inside the 600 requests/hour budget.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use gw2lurker_api::Api;
use gw2lurker_core::diff::{Differ, Event};
use gw2lurker_core::matchup::{self, ClaimRow, GuildSummary};
use gw2lurker_core::model::{MapRole, Match, ObjectiveInfo, TeamColor};
use gw2lurker_core::select::{self, Selection};
use gw2lurker_store::{Record, SessionStore};

use crate::catalog::{maps_for, objectives_for, resolve_guild_names};
use crate::net::WinHttpTransport;

/// Rounds of alerts kept for the widget (RF-19); the store keeps everything else.
const MAX_ALERTS: usize = 50;
/// Claims kept in memory for the guild history of the session (RF-72); the store has all.
const MAX_HISTORY: usize = 2000;

/// How often a failing API is retried at most, on top of the configured interval.
const MAX_BACKOFF: Duration = Duration::from_secs(300);

/// Whether the worker may use the network (set from the render callback's gate).
static MAY_POLL: AtomicBool = AtomicBool::new(false);
/// Set while the team picker is on screen: the list of live teams is needed even when polling
/// is not allowed (outside WvW), so the drop-down is never empty (bug of 2026-10-31).
static NEEDS_TEAMS: AtomicBool = AtomicBool::new(false);
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
    Degraded { since: DateTime<Utc>, error: String },
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
    /// Guilds ranked by most recent claim (RF-71).
    pub guilds: Vec<GuildSummary>,
    /// Objectives of the current language, for naming alerts.
    pub names: Vec<ObjectiveInfo>,
    /// Guild id → `[TAG] Nombre`, from `/v2/guild` (RF-20). Missing ones are shown as the id.
    pub guild_names: std::collections::BTreeMap<String, String>,
    /// The four WvW maps with their boxes, for the projection (RF-30).
    pub maps: Vec<gw2lurker_core::WvwMap>,
    /// Every team playing right now, with its matchup and colour (RF-03a).
    pub candidates: Vec<gw2lurker_core::Candidate>,
    /// Map id → matchup role, so an alert knows which map it happened on.
    pub map_roles: std::collections::BTreeMap<u32, MapRole>,
    /// Claims seen during this session, oldest first: the guild history of ADR-012.
    pub history: Vec<ClaimRow>,
    /// Most recent events, newest last (RF-19), each with **when we saw it**.
    ///
    /// The instant is stored here and not rebuilt when the line is formatted: recomputing it
    /// made every alert look brand new on every frame, so the pop-up never expired (bug of
    /// 2026-10-31).
    pub alerts: Vec<Received>,
    /// When the dashboard was last rebuilt.
    pub updated_at: Option<DateTime<Utc>>,
}

impl Dashboard {
    fn push_alert(&mut self, event: Event) {
        if !matches!(event, Event::Baseline { .. } | Event::Gap { .. }) {
            self.alerts.push(Received {
                event,
                received: Utc::now(),
            });
            if self.alerts.len() > MAX_ALERTS {
                let excess = self.alerts.len() - MAX_ALERTS;
                self.alerts.drain(..excess);
            }
        }
    }

    /// Recent events as ready-to-draw lines, **newest first** (RF-19), already filtered.
    ///
    /// The objective ids are resolved here so the render callback never searches the table,
    /// and the result is owned so the dashboard lock is not held while drawing. `filter`,
    /// `follow` and `current` decide which ones the player asked for (RF-19a).
    pub fn recent_alerts(
        &self,
        filter: &gw2lurker_core::AlertFilter,
        follow: &gw2lurker_core::FollowList,
        current: Option<MapRole>,
        limit: usize,
    ) -> Vec<AlertLine> {
        self.alerts
            .iter()
            .rev()
            .filter_map(|received| self.alert_line(&received.event, received.received))
            // What was already claimed when the addon started is not news: showing it made the
            // widget look like it never forgot anything (bug of 2026-10-31). The exception is a
            // followed guild, which is followed precisely to hear about it (RF-21, RF-25).
            .filter(|line| {
                !line.late
                    || line
                        .guild
                        .as_deref()
                        .is_some_and(|guild| follow.contains(guild))
            })
            .filter(|line| filter.notify(line.map_role, line.guild.as_deref(), follow, current))
            .take(limit)
            .collect()
    }

    fn alert_line(&self, event: &Event, received: DateTime<Utc>) -> Option<AlertLine> {
        let name = |id: &str| {
            self.objective_name(id)
                .map(str::to_owned)
                .unwrap_or_else(|| id.to_owned())
        };
        // Objective ids are `"<map id>-<n>"`, and `map_roles` maps that id to the matchup role
        // the alert filter talks about.
        let role = |id: &str| {
            id.split_once('-')
                .and_then(|(map, _)| map.parse::<u32>().ok())
                .map(|map_id| {
                    self.map_roles
                        .get(&map_id)
                        .copied()
                        .unwrap_or_else(|| MapRole::from_map_id(map_id))
                })
                .unwrap_or(MapRole::Unknown)
        };
        match event {
            Event::Flip {
                objective,
                from,
                to,
                at,
            } => Some(AlertLine {
                at: *at,
                received,
                map_role: role(objective),
                kind: AlertKind::Flip,
                objective: name(objective),
                guild: None,
                guild_name: None,
                from: *from,
                to: *to,
                late: false,
            }),
            Event::Claim {
                objective,
                guild,
                at,
                late,
            } => Some(AlertLine {
                at: *at,
                received,
                map_role: role(objective),
                kind: AlertKind::Claim,
                objective: name(objective),
                guild: Some(guild.clone()),
                guild_name: self.guild_names.get(guild).cloned(),
                from: None,
                to: None,
                late: *late,
            }),
            Event::Unclaim {
                objective,
                guild,
                at,
            } => Some(AlertLine {
                at: *at,
                received,
                map_role: role(objective),
                kind: AlertKind::Unclaim,
                objective: name(objective),
                guild: Some(guild.clone()),
                guild_name: self.guild_names.get(guild).cloned(),
                from: None,
                to: None,
                late: false,
            }),
            Event::Baseline { .. } | Event::Gap { .. } | Event::MatchChanged { .. } => None,
        }
    }

    /// Claims of one guild during the session, in the order they happened (RF-72).
    pub fn guild_history(&self, guild: &str) -> Vec<ClaimRow> {
        matchup::guild_timeline(&self.history, guild)
    }

    /// Guilds that claimed something this session, with their number of claims and the last one.
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
}

/// What happened, for the alert widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertKind {
    Flip,
    Claim,
    Unclaim,
}

/// One line of the alert widget.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlertLine {
    /// When the game says it happened.
    pub at: DateTime<Utc>,
    /// When this addon learnt about it; the pop-up timer runs from here (RF-19a).
    pub received: DateTime<Utc>,
    /// Map of the matchup the objective belongs to, for the alert filter.
    pub map_role: MapRole,
    pub kind: AlertKind,
    pub objective: String,
    pub guild: Option<String>,
    /// `[TAG] Nombre` of [`Self::guild`] when it is known (RF-20). `guild` stays the raw id
    /// because that is what following a guild compares against.
    pub guild_name: Option<String>,
    pub from: Option<TeamColor>,
    pub to: Option<TeamColor>,
    /// Discovered at a baseline instead of seen live (RF-25).
    pub late: bool,
}

/// An event and the instant this addon learnt about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Received {
    pub event: Event,
    /// When the poll that found it finished; the pop-up timer runs from here (RF-19a).
    pub received: DateTime<Utc>,
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
    NEEDS_TEAMS.store(needed, Ordering::Relaxed);
}

/// Starts the worker. Idempotent: a second call does nothing.
pub fn start() {
    static STARTED: OnceLock<()> = OnceLock::new();
    STARTED.get_or_init(|| {
        let spawn = std::thread::Builder::new()
            .name("gw2lurker-worker".to_owned())
            .spawn(run);
        if let Err(error) = spawn {
            log::warn!("worker: no se pudo arrancar el hilo: {error}");
        }
    });
}

/// Asks the worker to stop and waits for it to leave.
pub fn stop() {
    STOP.store(true, Ordering::Relaxed);
}

fn run() {
    let api = Api::new(WinHttpTransport::default());
    let mut differ = Differ::new(gw2lurker_core::DiffConfig::default());
    let mut teams_at: Option<Instant> = None;
    let mut store = open_store();
    let mut backoff_ms: u64 = 0;

    while !STOP.load(Ordering::Relaxed) {
        if !MAY_POLL.load(Ordering::Relaxed) {
            // Outside WvW, only the team list is worth fetching, and rarely: it changes at
            // reset, not every half minute.
            if NEEDS_TEAMS.load(Ordering::Relaxed) && teams_stale(&mut teams_at) {
                match api.matches_all() {
                    Ok(live) => publish_teams(&live, crate::team::selected()),
                    Err(error) => log::warn!("worker: sin lista de equipos ({error})"),
                }
                teams_at = Some(Instant::now());
            }
            sleep(Duration::from_millis(500));
            continue;
        }

        match poll_once(&api, &mut differ, store.as_mut()) {
            Ok(()) => backoff_ms = 0,
            Err(error) => {
                backoff_ms = next_backoff(backoff_ms);
                log::warn!("worker: {error}; reintento en {backoff_ms} ms");
                let mut board = dashboard();
                board.health = Health::Degraded {
                    since: started_degraded(&board.health),
                    error,
                };
            }
        }

        let interval = Duration::from_secs(u64::from(crate::config().poll_secs));
        let wait = interval
            .max(Duration::from_millis(backoff_ms))
            .min(MAX_BACKOFF);
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

/// Fills the dashboard with the teams of the live matches, and nothing else.
fn publish_teams(live: &[Match], selected: Option<u32>) {
    let mut board = dashboard();
    board.candidates = select::candidates(live);
    board.selection = select::evaluate(selected, live);
    board.match_id = match &board.selection {
        Selection::Confirmed { match_id, .. } => Some(match_id.clone()),
        _ => None,
    };
}

fn started_degraded(health: &Health) -> DateTime<Utc> {
    match health {
        Health::Degraded { since, .. } => *since,
        _ => Utc::now(),
    }
}

fn next_backoff(current_ms: u64) -> u64 {
    let next = if current_ms == 0 {
        1_000
    } else {
        current_ms * 2
    };
    next.min(MAX_BACKOFF.as_millis() as u64)
}

/// Sleeps in slices so `stop()` and a gate change are noticed quickly.
fn sleep(total: Duration) {
    let step = Duration::from_millis(250);
    let mut left = total;
    while left > Duration::ZERO && !STOP.load(Ordering::Relaxed) {
        let chunk = left.min(step);
        std::thread::sleep(chunk);
        left -= chunk;
    }
}

fn open_store() -> Option<SessionStore> {
    let dir = crate::config::addon_dir().ok()?;
    match SessionStore::open(dir, "session") {
        Ok(store) => Some(store),
        Err(error) => {
            log::warn!("worker: sin histórico ({error})");
            None
        }
    }
}

fn poll_once(
    api: &Api<WinHttpTransport>,
    differ: &mut Differ,
    store: Option<&mut SessionStore>,
) -> Result<(), String> {
    let team_id = crate::team::selected();
    // The picker lists every team of the region and colour, so it needs all nine matchups.
    // Inside WvW the worker would otherwise ask for the player's match alone and the list
    // showed one team per colour (bug of 2026-10-31).
    let full = wants_full_list(team_id, NEEDS_TEAMS.load(Ordering::Relaxed));
    let live = match team_id {
        // Known team: ask only for its match (~44 KB instead of ~400 KB).
        Some(team) if !full => match api.match_for_team(team) {
            Ok(found) => vec![found],
            // The team no longer plays: the ladder reshuffled, so refresh the whole list to
            // let the picker offer the live teams again.
            Err(_) => api.matches_all().map_err(|error| error.to_string())?,
        },
        _ => api.matches_all().map_err(|error| error.to_string())?,
    };

    let selection = select::evaluate(team_id, &live);
    let names = objectives_for(api)?;
    let match_id = match &selection {
        Selection::Confirmed { match_id, .. } => Some(match_id.clone()),
        _ => live.first().map(|entry| entry.id.clone()),
    };

    // Diff and store first: writing to the history file must not happen with the dashboard
    // locked, or the render callback would wait on disk I/O.
    let mut events = Vec::new();
    let mut claims = Vec::new();
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
        guilds = matchup::rank_guilds(&claims);
        new_history = events
            .iter()
            .filter_map(|event| history_row(event, entry, &names, &roles_of_match))
            .collect();
        if let Some(store) = store {
            for event in &events {
                // On disk only the followed guilds: the rest of the claims are the state of the
                // matchup, which the API returns anyway (bug of 2026-10-31).
                if !keeps_in_the_store(event, &crate::config().followed) {
                    continue;
                }
                let record = Record::new(Utc::now(), entry.id.clone(), event.clone());
                if let Err(error) = store.append(&record) {
                    log::warn!("worker: no se pudo guardar el evento ({error})");
                }
            }
        }
    }

    let now = Utc::now();
    let candidates = select::candidates(&live);
    let guild_names = resolve_guild_names(api, &guilds);
    let maps = maps_for(api);
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
    board.selection = selection;
    board.match_id = match_id;
    board.names = names;
    board.candidates = candidates;
    board.guild_names = guild_names;
    board.maps = maps;
    board.map_roles = map_roles;
    board.claims = claims;
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
    Ok(())
}

/// Whether the next poll has to fetch every matchup instead of only the player's one.
///
/// The team picker offers the teams of the region and colour that are playing *now*, and those
/// live in all nine matchups, not just the player's: inside WvW the list showed one team per
/// colour (bug of 2026-10-31, RF-03a).
fn wants_full_list(team_id: Option<u32>, picker_on_screen: bool) -> bool {
    team_id.is_none() || picker_on_screen
}

/// Whether an event is worth writing to disk: only what a followed guild did.
///
/// The claims of the rest of the matchup are the current state, and the API returns them on
/// every poll; keeping them on disk made the history look like it never restarted.
fn keeps_in_the_store(event: &Event, follow: &gw2lurker_core::FollowList) -> bool {
    match event {
        Event::Claim { guild, late, .. } => !*late && follow.contains(guild),
        _ => false,
    }
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
        late,
    } = event
    else {
        return None;
    };
    // A claim that was already there when the addon started is not part of this session: the
    // history has to start empty after a restart (bug of 2026-10-31).
    if *late {
        return None;
    }
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
    match id {
        Some(id) => live.iter().find(|entry| entry.id == id),
        None => live.first(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Full data path against the live API: WinHTTP + parse + the core view models.
    /// Ignored by default because CI must not depend on the network.
    #[test]
    #[ignore = "requiere red"]
    fn live_pipeline_builds_the_dashboard() {
        let api = Api::new(WinHttpTransport::default());
        let live = api.matches_all().expect("matches");
        let names = api.objectives("es").expect("objectives");

        let team = 12014;
        let selection = select::evaluate(Some(team), &live);
        let Selection::Confirmed { match_id, .. } = selection else {
            panic!("12014 deberia estar jugando: {selection:?}");
        };

        let entry = api.match_by_id(&match_id).expect("match");
        let rows = matchup::claims(&entry, &names);
        let guilds = matchup::rank_guilds(&rows);

        assert!(!rows.is_empty(), "el matchup real tiene claims");
        assert!(
            rows.iter().any(|row| row.name.is_some()),
            "las filas llevan nombre de objetivo en espanol"
        );
        assert!(!guilds.is_empty());
        assert!(
            guilds
                .windows(2)
                .all(|pair| pair[0].last_at >= pair[1].last_at),
            "el ranking va de mas reciente a menos"
        );
    }

    #[test]
    fn rf03a_the_picker_asks_for_every_matchup() {
        assert!(
            wants_full_list(Some(12014), true),
            "con el selector en pantalla hacen falta los nueve matchups, no solo el mío"
        );
        assert!(
            !wants_full_list(Some(12014), false),
            "sin el selector basta el matchup del jugador (~44 KB en vez de ~400 KB)"
        );
        assert!(
            wants_full_list(None, false),
            "sin equipo configurado hay que pedirlo todo"
        );
    }

    #[test]
    fn rf25_a_restart_does_not_show_yesterday() {
        let mut board = Dashboard::default();
        board.push_alert(Event::Claim {
            objective: "38-20".to_owned(),
            guild: "1DB147F0".to_owned(),
            at: Utc::now(),
            late: true,
        });
        board.push_alert(Event::Claim {
            objective: "38-21".to_owned(),
            guild: "1DB147F0".to_owned(),
            at: Utc::now(),
            late: false,
        });

        let filter = gw2lurker_core::AlertFilter::default();
        let follow = gw2lurker_core::FollowList::new();
        let lines = board.recent_alerts(&filter, &follow, None, 50);

        assert_eq!(lines.len(), 1, "solo el claim visto en vivo");
        assert_eq!(lines[0].objective, "38-21");
        assert!(!lines[0].late);
    }

    #[test]
    fn rf21_a_followed_guild_is_told_even_if_the_claim_was_already_there() {
        let mut board = Dashboard::default();
        board.push_alert(Event::Claim {
            objective: "38-20".to_owned(),
            guild: "1DB147F0".to_owned(),
            at: Utc::now(),
            late: true,
        });

        let mut follow = gw2lurker_core::FollowList::new();
        follow.add("1DB147F0");
        let filter = gw2lurker_core::AlertFilter::default();

        let lines = board.recent_alerts(&filter, &follow, None, 50);
        assert_eq!(
            lines.len(),
            1,
            "un clan seguido avisa aunque el claim ya estuviera al arrancar"
        );
        assert!(lines[0].late);
    }

    #[test]
    fn rf73_only_the_followed_guilds_go_to_disk() {
        let mut follow = gw2lurker_core::FollowList::new();
        follow.add("1DB147F0");

        let claim = |guild: &str, late: bool| Event::Claim {
            objective: "38-20".to_owned(),
            guild: guild.to_owned(),
            at: Utc::now(),
            late,
        };

        assert!(keeps_in_the_store(&claim("1DB147F0", false), &follow));
        assert!(
            !keeps_in_the_store(&claim("OTRO", false), &follow),
            "los claims de clanes que no sigues no se guardan"
        );
        assert!(
            !keeps_in_the_store(&claim("1DB147F0", true), &follow),
            "lo que ya estaba al arrancar no es historial de esta sesión"
        );
        assert!(!keeps_in_the_store(
            &Event::Gap {
                from: Utc::now(),
                to: Utc::now(),
            },
            &follow
        ));
    }

    #[test]
    fn rf19_alerts_are_bounded_and_skip_baselines() {
        let mut board = Dashboard::default();
        board.push_alert(Event::Baseline {
            reason: gw2lurker_core::BaselineReason::SessionStart,
        });
        assert!(board.alerts.is_empty(), "un baseline no es un aviso");

        for _ in 0..(MAX_ALERTS + 10) {
            board.push_alert(Event::Unclaim {
                objective: "38-20".to_owned(),
                guild: "1DB147F0".to_owned(),
                at: Utc::now(),
            });
        }
        assert_eq!(board.alerts.len(), MAX_ALERTS);
    }
}
