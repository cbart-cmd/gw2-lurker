//! Which team the player is on, as arcdps sees it (RF-03a).
//!
//! The game does not tell addons the WvW team: Mumble carries `map_id`, `map_type` and a
//! socket address, and Unofficial Extras' squad info has no team field (both checked). arcdps
//! does assign every agent a `team`, marks the local player with `is_self`, and Nexus bridges
//! that as `EV_ARCDPS_SELF_JOIN` on every map load — the same source `../fight_analysis_danger`
//! uses, and the reason it labels the live colour `Own`: the numeric value has no verified
//! mapping to red/green/blue.
//!
//! So the tag is kept **opaque**. Today it is logged, which is what makes a report like "I was
//! on the green team" checkable; if the colour mapping is ever verified, the value is already
//! here and nothing else has to change.

use std::sync::atomic::{AtomicI32, Ordering};

use nexus::event::{arc::SELF_JOIN, event_consume};

/// arcdps team of the local player; `-1` until arcdps reports it.
static SELF_TEAM: AtomicI32 = AtomicI32::new(-1);

/// Subscribes to arcdps' self-join event. Must be called from `load`.
///
/// Nexus fires it on every map load and can replay it on demand, so the first frame after
/// entering a WvW map already has an answer.
pub fn subscribe() {
    SELF_JOIN
        .subscribe(event_consume!(<nexus::event::arc::AgentUpdate> on_self_join))
        .revert_on_unload();
}

fn on_self_join(payload: Option<&nexus::event::arc::AgentUpdate>) {
    if let Some(agent) = payload {
        SELF_TEAM.store(i32::from(agent.team), Ordering::Relaxed);
    }
}

/// Opaque arcdps team tag of the local player, if arcdps has reported one.
///
/// `0` means "no team" (PvE, neutral), which says nothing about WvW.
pub fn self_team_tag() -> Option<u16> {
    u16::try_from(SELF_TEAM.load(Ordering::Relaxed))
        .ok()
        .filter(|tag| *tag != 0)
}

/// Tag as text for the log, so a bug report can say which side arcdps thought we were on.
pub fn describe() -> String {
    match self_team_tag() {
        Some(tag) => format!("arcdps team {tag}"),
        None => "no arcdps team".to_owned(),
    }
}
