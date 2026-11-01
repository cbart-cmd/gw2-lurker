//! Dónde está jugando el jugador, según el juego (RF-03a).
//!
//! Vive aparte de `worker` para que ese fichero quepa en el límite de 800 líneas (AGENTS.md
//! §2.7) y porque es una responsabilidad propia: preguntar a Mumble y buscar la partida.

use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use gw2lurker_core::model::Match;

pub use gw2lurker_core::detect::Detected;

/// When the last detection ran and what came out of it.
type Detection = Option<(Instant, Option<Detected>)>;
const RETRY_AFTER_MISSING_DATA: Duration = Duration::from_secs(60);

fn slot() -> &'static Mutex<Detection> {
    static SLOT: OnceLock<Mutex<Detection>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

/// Whether the nine matchups have to be asked for: **once per load**, not every quarter of an hour.
///
/// `shard_id` does not identify the matchup (measured 2026-10-31: it is `0x80000001` and appears
/// in none of the nine), so asking again and again buys nothing: what this first sweep is for is
/// **validating the saved team** and filling the picker. From then on the poll asks for the match
/// the player chose and nothing else (petición del jugador, 2026-10-31).
pub fn due() -> bool {
    let cached = slot();
    let detection = cached.lock().unwrap_or_else(|error| error.into_inner());
    match detection.as_ref() {
        None => true,
        Some((at, found)) => found.is_none() && retry_due(*at, Instant::now()),
    }
}

fn retry_due(at: Instant, now: Instant) -> bool {
    now.saturating_duration_since(at) >= RETRY_AFTER_MISSING_DATA
}

/// Forgets the last sweep, so the next poll asks for the nine matchups again.
///
/// Used when the character changes: the roster is per character, so another alt can play with a
/// different team (RF-03a).
pub fn forget() {
    *slot().lock().unwrap_or_else(|error| error.into_inner()) = None;
}

pub fn cached() -> Option<Detected> {
    slot()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .as_ref()
        .and_then(|(_, found)| found.clone())
}

/// Asks Mumble which shard the player is on and looks it up in every matchup.
///
/// `saved` is only used for the log line: a saved team that does not match the game is exactly
/// what used to make the addon watch the wrong match (report of 2026-10-31).
pub fn detect_in(live: &[Match], saved: Option<u32>) -> Option<Detected> {
    let found = if let Some((shard, colour)) = crate::ffi::mumble_where() {
        let found = gw2lurker_core::detect::of_shard(live, shard, colour);
        match &found {
            Some(detected) => {
                if Some(detected.team) != saved {
                    log::info!(
                        "deteccion: el juego dice {} (shard {shard}, color {:?}, confirmado {}); el equipo guardado era {saved:?}. Se vigila el del juego",
                        detected.match_id,
                        detected.colour,
                        detected.colour_confirmed
                    );
                }
            }
            None => log::info!(
                "deteccion: el shard {shard} (color {colour:?}) no está en ninguna partida; se usa el equipo guardado {saved:?}"
            ),
        }
        found
    } else {
        None
    };
    *slot().lock().unwrap_or_else(|error| error.into_inner()) =
        Some((Instant::now(), found.clone()));
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rf03a_missing_mumble_data_is_retried_after_a_cooldown() {
        let now = Instant::now();
        assert!(!retry_due(now, now));
        assert!(retry_due(now - RETRY_AFTER_MISSING_DATA, now));
    }
}
