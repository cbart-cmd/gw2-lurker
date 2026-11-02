//! The transient, stacked pop-up (RF-19d).
//!
//! Vive aparte porque es una superficie propia (y porque `mod.rs` estaba en el límite de 800
//! líneas, AGENTS.md §2.7).

use gw2lurker_core::model::MapRole;
use std::sync::atomic::{AtomicI64, Ordering};

use super::alerts;
use super::popup_stack::{PopupLine, PopupSnapshot, PopupStack};
use super::WIDGET;
use crate::i18n;

/// When the *Test pop-up* button of the settings wants a fake alert on screen (milliseconds).
pub static TEST_POPUP_UNTIL: AtomicI64 = AtomicI64::new(0);

/// Shows a fake pop-up for a few seconds, so it can be placed without waiting for a capture.
pub fn start_test_popup() {
    let until = chrono::Utc::now().timestamp_millis() + 5000;
    TEST_POPUP_UNTIL.store(until, Ordering::Relaxed);
}

/// Pop-up with what just happened, on top of the screen (RF-19a).
pub fn draw(
    ui: &nexus::imgui::Ui,
    alerts: &[crate::worker::AlertLine],
    now: chrono::DateTime<chrono::Utc>,
) {
    let (enabled, secs, opacity, mut position, alert_kinds, followed, team_id) = {
        let config = crate::runtime();
        let opacity = config
            .windows
            .get(WIDGET)
            .map(|state| state.opacity)
            .unwrap_or_default();
        (
            config.popup_enabled,
            config.popup_secs,
            opacity,
            config.popup_pos,
            config.alert_kinds,
            config.followed.clone(),
            crate::team::selected_in(&config),
        )
    };

    // RF-19c: el pop-up enseña los mismos tipos que el widget.
    let candidates: Vec<crate::worker::AlertLine> = alerts
        .iter()
        .filter(|line| alert_kinds.shows(line.kind))
        .cloned()
        .collect();

    let (states, player_team) = {
        let board = crate::worker::dashboard();
        let team_id = board.played_team.or(team_id);
        let player_team = team_id.and_then(|id| {
            board
                .candidates
                .iter()
                .find(|candidate| candidate.team_id == id)
                .map(|candidate| candidate.color)
        });
        (board.states.clone(), player_team)
    };

    let mut popup = queue_slot();
    let snapshot = popup.tick(
        &candidates,
        now,
        if enabled { secs } else { 0.0 },
        &followed,
        &states,
        player_team,
    );
    drop(popup);
    if !enabled {
        return;
    }

    let test = test_popup_line(now);
    let placing = test.is_some();
    let showing = popup_lines(snapshot, test);
    if showing.lines.is_empty() && showing.overflow == 0 {
        return;
    }

    alerts::draw_popup(
        ui,
        &showing.lines,
        showing.overflow,
        opacity,
        &mut position,
        placing,
        &alerts::LineContext {
            now,
            player_team,
            states: &states,
        },
    );

    let mut config = crate::runtime();
    if position != config.popup_pos {
        config.popup_pos = position;
    }
}

fn popup_lines(snapshot: PopupSnapshot, test: Option<crate::worker::AlertLine>) -> PopupSnapshot {
    if let Some(line) = test {
        PopupSnapshot {
            lines: vec![PopupLine { line, fade: 1.0 }],
            overflow: 0,
        }
    } else {
        snapshot
    }
}

/// The pop-up stack is session state, not a setting.
fn queue_slot() -> std::sync::MutexGuard<'static, PopupStack> {
    static QUEUE: std::sync::OnceLock<std::sync::Mutex<PopupStack>> = std::sync::OnceLock::new();
    QUEUE
        .get_or_init(|| std::sync::Mutex::new(PopupStack::default()))
        .lock()
        .unwrap_or_else(|error| error.into_inner())
}

/// A fake line while the settings ask for one, so the pop-up can be placed at will.
fn test_popup_line(now: chrono::DateTime<chrono::Utc>) -> Option<crate::worker::AlertLine> {
    if now.timestamp_millis() >= TEST_POPUP_UNTIL.load(Ordering::Relaxed) {
        return None;
    }
    Some(crate::worker::AlertLine {
        at: now,
        received: now,
        map_role: MapRole::Center,
        objective_id: "38-1".to_owned(),
        objective_kind: gw2lurker_core::ObjectiveType::Camp,
        kind: crate::worker::AlertKind::Claim,
        objective: i18n::text("Sample objective").to_owned(),
        guild: None,
        guild_name: Some(i18n::text("Test guild").to_owned()),
        from: None,
        to: None,
        late: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line() -> crate::worker::AlertLine {
        let now = chrono::Utc::now();
        crate::worker::AlertLine {
            at: now,
            received: now,
            map_role: MapRole::Center,
            objective_id: "38-1".to_owned(),
            objective_kind: gw2lurker_core::ObjectiveType::Camp,
            kind: crate::worker::AlertKind::Claim,
            objective: "Camp".to_owned(),
            guild: None,
            guild_name: Some("Test".to_owned()),
            from: None,
            to: None,
            late: false,
        }
    }

    #[test]
    fn rf19a_a_test_popup_is_shown_without_a_real_alert() {
        let showing = popup_lines(PopupSnapshot::default(), Some(line()));
        assert_eq!(showing.lines.len(), 1);
        assert_eq!(showing.lines[0].line.guild_name.as_deref(), Some("Test"));
        assert_eq!(showing.lines[0].fade, 1.0);
    }
}
