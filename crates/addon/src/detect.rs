//! Dónde está jugando el jugador, según el juego (RF-03a).
//!
//! Vive aparte de `worker` para que ese fichero quepa en el límite de 800 líneas (AGENTS.md
//! §2.7) y porque es una responsabilidad propia: preguntar a Mumble y buscar la partida.

use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use gw2lurker_core::model::Match;

pub use gw2lurker_core::detect::Detected;

/// How often the worker re-asks the game where the player is (the shard only changes at reset).
const DETECT_EVERY: Duration = Duration::from_secs(15 * 60);

/// When the last detection ran and what came out of it.
type Detection = Option<(Instant, Option<Detected>)>;

fn slot() -> &'static Mutex<Detection> {
    static SLOT: OnceLock<Mutex<Detection>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

/// Whether it is time to ask all nine matchups where the player is.
pub fn due() -> bool {
    let slot = slot().lock().unwrap_or_else(|error| error.into_inner());
    match slot.as_ref() {
        Some((at, _)) => at.elapsed() >= DETECT_EVERY,
        None => true,
    }
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
    let (shard, colour) = crate::ffi::mumble_where()?;
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
    *slot().lock().unwrap_or_else(|error| error.into_inner()) =
        Some((Instant::now(), found.clone()));
    found
}
