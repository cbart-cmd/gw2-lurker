//! What the worker writes to the log, apart from `worker` itself so that file stays within its
//! size limit.
//!
//! The lines exist for one reason: «no me avisa» has three very different causes —the worker is
//! watching another matchup, no event was detected, or the event arrived and the filter ate it— and
//! without these lines they all look the same in `Nexus.log` (report of 2026-10-31).

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use chrono::{DateTime, Utc};

use gw2lurker_core::diff::Event;
use gw2lurker_core::model::Match;
use gw2lurker_core::select::Selection;
/// Says which matchup is being watched, and what came out of the poll.
///
/// This is the line that answers "why did nothing alert me": if the id starts with `1-` the
/// addon is watching North America and the player is probably in Europe (reported 2026-10-31,
/// the team picked was `11011`, which the API plays in `1-1`). Only logged when it changes, plus
/// every detected event, because both are rare.
pub fn log_watched_matchup(
    selection: &Selection,
    match_id: Option<&str>,
    live: &[Match],
    events: &[Event],
) {
    static LAST: Mutex<Option<String>> = Mutex::new(None);
    let region = match_id
        .and_then(|id| id.split_once('-'))
        .map(|(region, _)| region);
    let region = match region {
        Some("1") => "NA",
        Some("2") => "EU",
        _ => "?",
    };
    let line = format!(
        "{} ({region}) · {:?} · {} emparejamientos",
        match_id.unwrap_or("-"),
        selection,
        live.len()
    );
    let mut last = LAST.lock().unwrap_or_else(|error| error.into_inner());
    if last.as_deref() != Some(line.as_str()) {
        log::info!("worker: vigilando {line}");
        *last = Some(line);
    }
    drop(last);

    for event in events {
        match event {
            Event::Claim {
                objective,
                guild,
                late,
                ..
            } => log::info!("evento: Claim {objective} de {guild} (late: {late})"),
            Event::Flip {
                objective,
                to: from,
                ..
            } => log::info!("evento: Flip {objective} -> {from:?}"),
            Event::Unclaim {
                objective, guild, ..
            } => {
                log::info!("evento: Unclaim {objective} de {guild}")
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

/// A line every few polls with what the last one saw.
///
/// «No me avisa» has three very different causes and only the log can tell them apart: the poll may
/// not be running, the differ may see no changes, or the events may arrive and the filter eat them.
/// One line every ten polls (five minutes at the default) is cheap and answers the first two
/// (report of 2026-10-31). Since 2026-10-31 it also says **when the game** last saw a claim in
/// this matchup: if that timestamp is old, nothing is broken, there is simply nothing happening.
pub fn heartbeat(
    events: usize,
    claims: usize,
    objectives: usize,
    match_id: Option<&str>,
    last_claim: Option<DateTime<Utc>>,
) {
    static POLLS: AtomicU64 = AtomicU64::new(0);
    let count = POLLS.fetch_add(1, Ordering::Relaxed) + 1;
    if count.is_multiple_of(10) {
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
