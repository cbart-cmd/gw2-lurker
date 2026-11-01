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
use std::sync::Mutex;

use nexus::event::{arc::SELF_JOIN, event_consume};

/// arcdps team of the local player; `-1` until arcdps reports it.
static SELF_TEAM: AtomicI32 = AtomicI32::new(-1);

/// Account name arcdps reports for the local player, `None` until the first self-join.
static ACCOUNT: Mutex<Option<String>> = Mutex::new(None);

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
    let Some(agent) = payload else {
        return;
    };
    SELF_TEAM.store(i32::from(agent.team), Ordering::Relaxed);
    // El equipo se recuerda por cuenta, no por personaje (RF-03a): arcdps da el nombre de
    // cuenta en el mismo evento que el equipo, así que no hay que preguntar nada.
    if let Some(account) = agent.account().map(str::trim).filter(|a| !a.is_empty()) {
        let mut slot = ACCOUNT.lock().unwrap_or_else(|error| error.into_inner());
        if slot.as_deref() != Some(account) {
            log::info!("arcdps: cuenta {account}");
            *slot = Some(account.to_owned());
        }
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

/// Account name arcdps reported for the local player (`Jhoin.1234`), which is the key the team
/// is saved under (RF-03a).
///
/// Copied out on purpose: the event arrives on arcdps' thread, so callers never hold the lock.
pub fn self_account() -> Option<String> {
    ACCOUNT
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clone()
}

/// Tag as text for the log, so a bug report can say which side arcdps thought we were on.
pub fn describe() -> String {
    match self_team_tag() {
        Some(tag) => format!("arcdps team {tag}"),
        None => "no arcdps team".to_owned(),
    }
}
