//! Which team the player is on, as far as arcdps can tell (RF-03a).
//!
//! The game itself does not tell addons the WvW team: Mumble only carries `map_id`,
//! `map_type` and a socket address. arcdps, however, assigns every agent a `team` field and
//! marks the local player with `is_self`, and Nexus bridges that as
//! `EV_ARCDPS_SELF_JOIN` on every map load. That is exactly what `../fight_analysis_danger`
//! uses, and the reason it labels the live colour `Own`: the numeric `team` value does not
//! have a verified mapping to red/green/blue.
//!
//! So we keep it as an **opaque tag**, which is what we actually need: a stable way to
//! notice that the player is on a different team than the one selected. Once the player
//! picks their team, the colour from the API is memorised together with the tag, and the
//! calibration is used to detect a change (another account, or the monthly re-link).

use std::sync::atomic::{AtomicI32, Ordering};

use gw2lurker_core::TeamColor;
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
pub fn self_team_tag() -> Option<u16> {
    let tag = SELF_TEAM.load(Ordering::Relaxed);
    u16::try_from(tag).ok()
}

/// Whether the tag is worth using as a change detector.
///
/// arcdps reports `0` for agents without a team (PvE, neutral), which says nothing.
pub fn has_team() -> bool {
    self_team_tag().is_some_and(|tag| tag != 0)
}

/// Learned relation between the opaque tag and the colour the API gives the team.
///
/// Kept per session: the tag is only meaningful for the current client session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Calibration {
    tag: Option<u16>,
    color: Option<TeamColor>,
}

impl Calibration {
    /// Memorises the pair after the player picks a team.
    pub fn learn(&mut self, tag: Option<u16>, color: TeamColor) {
        self.tag = tag.filter(|tag| *tag != 0);
        self.color = Some(color);
    }

    /// Colour learned for the current tag, if the tag still matches.
    pub fn color_now(&self) -> Option<TeamColor> {
        match (self.tag, self.color) {
            (Some(learned), Some(color)) if Some(learned) == self_team_tag() => Some(color),
            _ => None,
        }
    }

    /// Whether the player is demonstrably on a different team than the one selected: the tag
    /// changed and arcdps is reporting one.
    pub fn changed(&self) -> bool {
        match (self.tag, self_team_tag()) {
            (Some(learned), Some(current)) => learned != current && current != 0,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rf03a_without_arcdps_nothing_is_claimed() {
        // No `SELF_JOIN` has been received in the test process.
        assert_eq!(self_team_tag(), None);
        assert!(!has_team());
        assert_eq!(Calibration::default().color_now(), None);
        assert!(
            !Calibration::default().changed(),
            "sin datos no se pregunta"
        );
    }

    #[test]
    fn rf03a_calibration_forgets_a_different_tag() {
        let mut calibration = Calibration::default();
        calibration.learn(Some(2), TeamColor::Blue);
        assert_eq!(
            calibration.color_now(),
            None,
            "el tag aprendido no es el actual, así que no se afirma nada"
        );
    }

    #[test]
    fn rf03a_zero_tag_is_not_learned() {
        let mut calibration = Calibration::default();
        calibration.learn(Some(0), TeamColor::Red);
        assert_eq!(calibration.color_now(), None);
    }
}
