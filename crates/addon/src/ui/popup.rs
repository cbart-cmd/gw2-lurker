//! El aviso emergente (RF-19a): encima de la pantalla, **de uno en uno**.
//!
//! Vive aparte porque es una superficie propia (y porque `mod.rs` estaba en el límite de 800
//! líneas, AGENTS.md §2.7).

use gw2lurker_core::model::MapRole;
use std::sync::atomic::{AtomicI64, Ordering};

use super::alerts;
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
    let (enabled, secs, opacity, mut position) = {
        let config = crate::config();
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
        )
    };
    if !enabled {
        return;
    }

    // RF-19c: el pop-up enseña los mismos tipos que el widget.
    let candidates: Vec<crate::worker::AlertLine> = alerts
        .iter()
        .filter(|line| crate::config().alert_kinds.shows(line.kind))
        .cloned()
        .collect();
    // De uno en uno: dos reclamaciones seguidas se leían como una sola y el pop-up se iba de
    // golpe (informe de 2026-10-31).
    let mut popup = queue_slot();
    let shown = popup.tick(&candidates, now, secs);
    drop(popup);
    let (mut showing, fade) = match shown {
        Some((line, fade)) => (vec![line], fade),
        None => return,
    };
    if let Some(test) = test_popup_line(now) {
        showing.insert(0, test);
    }

    alerts::draw_popup(ui, &showing, opacity, &mut position, fade);

    if position != crate::config().popup_pos {
        crate::runtime().popup_pos = position;
    }
}

/// The one pop-up at a time (RF-19a): session state, not a setting.
fn queue_slot() -> std::sync::MutexGuard<'static, alerts::PopupQueue> {
    static QUEUE: std::sync::OnceLock<std::sync::Mutex<alerts::PopupQueue>> =
        std::sync::OnceLock::new();
    QUEUE
        .get_or_init(|| std::sync::Mutex::new(alerts::PopupQueue::default()))
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
