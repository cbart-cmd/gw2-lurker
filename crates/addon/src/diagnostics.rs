//! What the worker writes to the log, apart from `worker` itself so that file stays within its
//! size limit.
//!
//! The lines exist for one reason: «no me avisa» has three very different causes —the worker is
//! watching another matchup, no event was detected, or the event arrived and the filter ate it— and
//! without these lines they all look the same in `Nexus.log` (report of 2026-10-31).

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};

use gw2lurker_core::diff::Event;
use gw2lurker_core::model::Match;
use gw2lurker_core::select::Selection;

/// How often the heartbeat is written, whatever the poll interval is.
const HEARTBEAT_EVERY: Duration = Duration::from_secs(300);

/// Whether `line` is different from the last one written in this slot, remembering it if so.
///
/// Every line that describes a **state** goes through here: the same sentence repeated at each
/// poll buried the lines that matter in `Nexus.log` (90 lines in four minutes, report of
/// 2026-11-01).
pub fn changed(last: &mut Option<String>, line: &str) -> bool {
    if last.as_deref() == Some(line) {
        return false;
    }
    *last = Some(line.to_owned());
    true
}

/// What identifies the watched matchup: id, region and selection, and nothing that moves.
///
/// It used to carry how many matchups the poll had returned, which is nine once a minute (`ids=all`)
/// and one the rest of the time, so the line was rewritten at every other poll.
fn watched_key(selection: &Selection, match_id: Option<&str>) -> String {
    let region = match_id
        .and_then(|id| id.split_once('-'))
        .map(|(region, _)| region);
    let region = match region {
        Some("1") => "NA",
        Some("2") => "EU",
        _ => "?",
    };
    format!("{} ({region}) · {:?}", match_id.unwrap_or("-"), selection)
}

/// Says which matchup is being watched, and what came out of the poll.
///
/// This is the line that answers "why did nothing alert me": if the id starts with `1-` the
/// addon is watching North America and the player is probably in Europe (reported 2026-10-31,
/// the team picked was `11011`, which the API plays in `1-1`). Only logged when it changes. The
/// events that are rare (`Baseline`, `Gap`, `MatchChanged`) are written too; the ones that are not
/// (`Claim`, `Flip`, `Unclaim`: at the start of a session every claim already standing arrives as
/// one) go to `debug`, which the addon's log filter leaves out.
pub fn log_watched_matchup(
    selection: &Selection,
    match_id: Option<&str>,
    live: &[Match],
    events: &[Event],
) {
    static LAST: Mutex<Option<String>> = Mutex::new(None);
    let key = watched_key(selection, match_id);
    let mut last = LAST.lock().unwrap_or_else(|error| error.into_inner());
    if changed(&mut last, &key) {
        log::info!("worker: vigilando {key} · {} emparejamientos", live.len());
    }
    drop(last);

    for event in events {
        match event {
            Event::Claim {
                objective,
                guild,
                late,
                ..
            } => log::debug!("evento: Claim {objective} de {guild} (late: {late})"),
            Event::Flip {
                objective,
                to: from,
                ..
            } => log::debug!("evento: Flip {objective} -> {from:?}"),
            Event::Unclaim {
                objective, guild, ..
            } => {
                log::debug!("evento: Unclaim {objective} de {guild}")
            }
            Event::Baseline { reason } => log::info!("evento: Baseline ({reason:?})"),
            Event::Gap { .. } => log::info!("evento: Gap (nos hemos perdido un tramo)"),
            Event::MatchChanged { old, new } => {
                log::info!("evento: MatchChanged {old} -> {new}")
            }
        }
    }
}

/// `claimed_at` comes from the API and is the game's clock, not ours (measured: the newest claim
/// of `1-1` was 21:02 UTC while other matchups were at 21:10, which is the difference between
/// «the addon is stuck» and «nobody claimed anything here», report of 2026-10-31).
pub fn last_claim_of(live: &[Match], match_id: Option<&str>) -> Option<DateTime<Utc>> {
    let entry = live
        .iter()
        .find(|entry| Some(entry.id.as_str()) == match_id)?;
    entry
        .maps
        .iter()
        .flat_map(|map| map.objectives.iter())
        .filter_map(|objective| objective.claimed_at)
        .max()
}

/// Whether the heartbeat is due: the first poll writes it (it proves the worker is up) and then
/// one every [`HEARTBEAT_EVERY`].
///
/// By time and not by number of polls: «every ten polls» was five minutes at the old 30 s default
/// and one line a minute at the current 5 s (report of 2026-11-01).
fn heartbeat_due(last: &mut Option<Instant>, now: Instant) -> bool {
    let due = last.is_none_or(|at| now.saturating_duration_since(at) >= HEARTBEAT_EVERY);
    if due {
        *last = Some(now);
    }
    due
}

/// A line every five minutes with what the last poll saw.
///
/// «No me avisa» has three very different causes and only the log can tell them apart: the poll may
/// not be running, the differ may see no changes, or the events may arrive and the filter eat them.
/// A line every few minutes is cheap and answers the first two (report of 2026-10-31). Since
/// 2026-10-31 it also says **when the game** last saw a claim in this matchup: if that timestamp is
/// old, nothing is broken, there is simply nothing happening.
pub fn heartbeat(
    events: usize,
    claims: usize,
    objectives: usize,
    match_id: Option<&str>,
    last_claim: Option<DateTime<Utc>>,
) {
    static POLLS: AtomicU64 = AtomicU64::new(0);
    static LAST: Mutex<Option<Instant>> = Mutex::new(None);
    let count = POLLS.fetch_add(1, Ordering::Relaxed) + 1;
    let due = heartbeat_due(
        &mut LAST.lock().unwrap_or_else(|error| error.into_inner()),
        Instant::now(),
    );
    if due {
        let newest = match last_claim {
            Some(at) => {
                let minutes = Utc::now().signed_duration_since(at).num_minutes();
                format!(
                    "último reclamo hace {minutes} min ({})",
                    crate::clock::clock(at)
                )
            }
            None => "sin reclamos en esta partida".to_owned(),
        };
        log::info!(
            "sondeo {count}: partida {match_id:?}, {claims} reclamados de {objectives} objetivos, {events} eventos nuevos, {newest}"
        );
    }
}

/// Says, once per change, where the game says the player is and which team the addon watches.
///
/// The `shard_id` is logged raw on purpose: its meaning in WvW is still unverified (T-19), and
/// this line is what settles it. The colour is Mumble's `team_color_id`, which is trustworthy.
pub fn log_where(configured: Option<u32>, match_id: Option<&str>, live: &[Match]) {
    /// (equipo configurado, shard de Mumble, color de Mumble)
    type Key = (Option<u32>, Option<u32>, Option<u32>);
    static LAST: Mutex<Option<Key>> = Mutex::new(None);

    let Some((shard, colour)) = crate::ffi::mumble_where() else {
        return;
    };
    let played = live
        .iter()
        .find(|entry| Some(entry.id.as_str()) == match_id)
        .and_then(|entry| entry.team_of_shard(shard));
    let key = (configured, Some(shard), colour.map(|c| c as u32));
    {
        let mut last = LAST.lock().unwrap_or_else(|error| error.into_inner());
        if last.as_ref() == Some(&key) {
            return;
        }
        *last = Some(key);
    }
    let watched = configured
        .and_then(|team| {
            live.iter()
                .find(|entry| Some(entry.id.as_str()) == match_id)
                .map(|entry| (team, entry.color_of(team)))
        })
        .map(|(team, colour)| format!("{team} ({colour:?})"))
        .unwrap_or_else(|| "ninguno".to_owned());
    log::info!(
        "donde: mumble shard {shard}, color {colour:?}; equipo configurado {watched}; mumble dice {played:?}"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t35_a_state_line_is_written_once_until_it_changes() {
        let confirmed = Selection::Confirmed {
            team_id: 12014,
            match_id: "2-2".to_owned(),
        };
        let mut last = None;
        let line = watched_key(&confirmed, Some("2-2"));
        assert!(
            changed(&mut last, &line),
            "the first line is always written"
        );
        assert!(
            !changed(&mut last, &watched_key(&confirmed, Some("2-2"))),
            "the same state at the next poll is not written again"
        );
        assert!(
            changed(&mut last, &watched_key(&Selection::Missing, None)),
            "a different state is written"
        );
        assert!(
            changed(&mut last, &line),
            "and going back to the first is a change too"
        );
    }

    #[test]
    fn t35_the_watched_line_still_says_which_region_is_being_watched() {
        let missing = Selection::Missing;
        assert!(watched_key(&missing, Some("2-2")).contains("(EU)"));
        assert!(watched_key(&missing, Some("1-1")).contains("(NA)"));
        assert!(watched_key(&missing, None).contains("(?)"));
    }

    #[test]
    fn t35_the_heartbeat_is_one_line_per_interval_whatever_the_poll_interval() {
        let start = Instant::now();
        let mut last = None;
        assert!(
            heartbeat_due(&mut last, start),
            "the first poll proves the worker is up"
        );
        for poll in [
            Duration::from_secs(5),
            Duration::from_secs(30),
            Duration::from_secs(120),
        ] {
            let mut last = Some(start);
            let mut at = start + poll;
            while at < start + HEARTBEAT_EVERY {
                assert!(
                    !heartbeat_due(&mut last, at),
                    "no line before the interval ({poll:?})"
                );
                at += poll;
            }
            assert!(
                heartbeat_due(&mut last, at),
                "the interval is over, so the line is written ({poll:?})"
            );
            assert!(
                !heartbeat_due(&mut last, at + poll),
                "and the next poll does not write it again ({poll:?})"
            );
        }
    }
}
