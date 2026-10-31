//! Glue between Nexus/Mumble and the pure gate in `core`.

use std::sync::Mutex;

use gw2lurker_core::gate::{self, GameState, Gate, Hidden};

/// Current game state as the core understands it.
pub fn snapshot() -> GameState {
    let (map_id, map_type) = crate::ffi::mumble_map().unwrap_or((0, 0));
    GameState {
        is_gameplay: crate::ffi::is_gameplay(),
        map_id,
        map_type,
    }
}

/// Whether the UI may draw and the worker may poll (RF-01, RF-02, RF-02a).
pub fn gate() -> Gate {
    gate::evaluate(snapshot())
}

/// Map role the player is standing on, for the "only my map" alert option (RF-19a).
pub fn current_map_role() -> Option<gw2lurker_core::MapRole> {
    match gw2lurker_core::MapRole::from_map_id(snapshot().map_id) {
        gw2lurker_core::MapRole::Unknown => None,
        role => Some(role),
    }
}

/// Logs the gate decision, but only when it changes.
///
/// Without this, "the addon loaded and shows nothing" is indistinguishable from "the render
/// callback never ran". The line carries `map_id`, `map_type` and `is_gameplay`, which is
/// exactly what is needed to tell a wrong `map_type` from a hidden-by-design state.
pub fn log_gate(gate: Gate) {
    static LAST: Mutex<Option<(GameState, Option<Hidden>)>> = Mutex::new(None);

    let state = snapshot();
    let key = (state, gate.hidden);
    let mut last = LAST.lock().unwrap_or_else(|error| error.into_inner());
    if last.as_ref() != Some(&key) {
        log::info!(
            "gate: map {} tipo {} gameplay {:?} -> show_ui {} ({:?})",
            state.map_id,
            state.map_type,
            state.is_gameplay,
            gate.show_ui,
            gate.hidden
        );
        *last = Some(key);
    }
}
