//! Glue between Nexus/Mumble and the pure gate in `core`.

use gw2lurker_core::gate::{self, GameState, Gate};

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
