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
use std::time::Duration;

use chrono::{DateTime, Utc};
use gw2lurker_api::Api;
use gw2lurker_core::diff::{Differ, Event};
use gw2lurker_core::matchup::{self, ClaimRow, GuildSummary};
use gw2lurker_core::model::{Match, ObjectiveInfo, TeamColor};
use gw2lurker_core::select::{self, Selection};
use gw2lurker_store::{Record, SessionStore};

use crate::net::WinHttpTransport;

/// Rounds of alerts kept for the widget (RF-19); the store keeps everything else.
const MAX_ALERTS: usize = 50;

/// How often a failing API is retried at most, on top of the configured interval.
const MAX_BACKOFF: Duration = Duration::from_secs(300);

/// Whether the worker may use the network (set from the render callback's gate).
static MAY_POLL: AtomicBool = AtomicBool::new(false);
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
    /// Every team playing right now, with its matchup and colour (RF-03a).
    pub candidates: Vec<gw2lurker_core::Candidate>,
    /// Most recent events, newest last (RF-19).
    pub alerts: Vec<Event>,
    /// When the dashboard was last rebuilt.
    pub updated_at: Option<DateTime<Utc>>,
}

impl Dashboard {
    fn push_alert(&mut self, event: Event) {
        if !matches!(event, Event::Baseline { .. } | Event::Gap { .. }) {
            self.alerts.push(event);
            if self.alerts.len() > MAX_ALERTS {
                let excess = self.alerts.len() - MAX_ALERTS;
                self.alerts.drain(..excess);
            }
        }
    }

    /// Recent events as ready-to-draw lines, **newest first** (RF-19).
    ///
    /// The objective ids are resolved here so the render callback never searches the table,
    /// and the result is owned so the dashboard lock is not held while drawing.
    pub fn recent_alerts(&self, limit: usize) -> Vec<AlertLine> {
        self.alerts
            .iter()
            .rev()
            .filter_map(|event| self.alert_line(event))
            .take(limit)
            .collect()
    }

    fn alert_line(&self, event: &Event) -> Option<AlertLine> {
        let name = |id: &str| {
            self.objective_name(id)
                .map(str::to_owned)
                .unwrap_or_else(|| id.to_owned())
        };
        match event {
            Event::Flip {
                objective,
                from,
                to,
                at,
            } => Some(AlertLine {
                at: *at,
                kind: AlertKind::Flip,
                objective: name(objective),
                guild: None,
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
                kind: AlertKind::Claim,
                objective: name(objective),
                guild: Some(guild.clone()),
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
                kind: AlertKind::Unclaim,
                objective: name(objective),
                guild: Some(guild.clone()),
                from: None,
                to: None,
                late: false,
            }),
            Event::Baseline { .. } | Event::Gap { .. } | Event::MatchChanged { .. } => None,
        }
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
    pub at: DateTime<Utc>,
    pub kind: AlertKind,
    pub objective: String,
    pub guild: Option<String>,
    pub from: Option<TeamColor>,
    pub to: Option<TeamColor>,
    /// Discovered at a baseline instead of seen live (RF-25).
    pub late: bool,
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
    let mut store = open_store();
    let mut backoff_ms: u64 = 0;

    while !STOP.load(Ordering::Relaxed) {
        if !MAY_POLL.load(Ordering::Relaxed) {
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
    let live = match team_id {
        // Known team: ask only for its match (~44 KB instead of ~400 KB).
        Some(team) => match api.match_for_team(team) {
            Ok(found) => vec![found],
            // The team no longer plays: the ladder reshuffled, so refresh the whole list to
            // let the picker offer the live teams again.
            Err(_) => api.matches_all().map_err(|error| error.to_string())?,
        },
        None => api.matches_all().map_err(|error| error.to_string())?,
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
    if let Some(entry) = current_match(&live, match_id.as_deref()) {
        events = differ.push(gw2lurker_core::diff::Snapshot::from_match(
            entry,
            Utc::now(),
        ));
        claims = matchup::claims(entry, &names);
        guilds = matchup::rank_guilds(&claims);
        if let Some(store) = store {
            for event in &events {
                let record = Record::new(Utc::now(), entry.id.clone(), event.clone());
                if let Err(error) = store.append(&record) {
                    log::warn!("worker: no se pudo guardar el evento ({error})");
                }
            }
        }
    }

    let now = Utc::now();
    let candidates = select::candidates(&live);
    let mut board = dashboard();
    board.health = Health::Ok { at: now };
    board.selection = selection;
    board.match_id = match_id;
    board.names = names;
    board.candidates = candidates;
    board.claims = claims;
    board.guilds = guilds;
    for event in events {
        board.push_alert(event);
    }
    board.updated_at = Some(now);
    Ok(())
}

fn current_match<'a>(live: &'a [Match], id: Option<&str>) -> Option<&'a Match> {
    match id {
        Some(id) => live.iter().find(|entry| entry.id == id),
        None => live.first(),
    }
}

fn objectives_for(api: &Api<WinHttpTransport>) -> Result<Vec<ObjectiveInfo>, String> {
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
