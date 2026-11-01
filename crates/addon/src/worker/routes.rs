//! Prepares immutable Estela models on the polling thread, never in the render callback.

use super::*;
use gw2lurker_core::route::trail::Trail;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// Atomically published history, annotations and derived per-guild models.
#[derive(Debug)]
pub(super) struct Prepared {
    pub history: Vec<ClaimRow>,
    pub events: Arc<Vec<Event>>,
    pub models: BTreeMap<String, Arc<Trail<'static>>>,
}

/// Merges a successful poll with bounded history and includes still-current baseline claims.
pub(super) fn prepare(
    mut history: Vec<ClaimRow>,
    mut events: Arc<Vec<Event>>,
    incoming: Vec<ClaimRow>,
    observed: &[Event],
    current: &[ClaimRow],
    names: &[ObjectiveInfo],
) -> Prepared {
    history.extend(incoming);
    if history.len() > MAX_HISTORY {
        history.drain(..history.len() - MAX_HISTORY);
    }
    for event in observed {
        crate::alerts::retain_route_event(&mut events, event);
    }
    let guilds: BTreeSet<_> = history
        .iter()
        .chain(current)
        .map(|r| r.guild.as_str())
        .collect();
    let models = guilds
        .into_iter()
        .map(|guild| {
            let rows = matchup::guild_timeline_with_current(&history, current, guild);
            let trail = Trail::new(&rows, guild, names, &events).into_owned();
            (guild.to_owned(), Arc::new(trail))
        })
        .collect();
    Prepared {
        history,
        events,
        models,
    }
}
