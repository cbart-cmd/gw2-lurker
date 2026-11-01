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
/// Used when the account changes: the team is remembered per account, so another account can
/// play with a different team (RF-03a).
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

/// What a detection has to say: a saved team that disagrees with the game, or a shard that is in
/// no matchup. When the game and the saved team agree there is nothing to report.
fn detection_line(
    found: Option<&Detected>,
    shard: u32,
    colour: Option<gw2lurker_core::TeamColor>,
    saved: Option<u32>,
) -> Option<String> {
    match found {
        Some(detected) if Some(detected.team) == saved => None,
        Some(detected) => Some(format!(
            "deteccion: el juego dice {} (shard {shard}, color {:?}, confirmado {}); el equipo guardado era {saved:?}. Se vigila el del juego",
            detected.match_id, detected.colour, detected.colour_confirmed
        )),
        None => Some(format!(
            "deteccion: el shard {shard} (color {colour:?}) no está en ninguna partida; se usa el equipo guardado {saved:?}"
        )),
    }
}

/// Asks Mumble which shard the player is on and looks it up in every matchup.
///
/// `saved` is only used for the log line: a saved team that does not match the game is exactly
/// what used to make the addon watch the wrong match (report of 2026-10-31). The line is written
/// once per outcome: with no matchup found the sweep is retried every minute and said the same
/// thing each time (report of 2026-11-01).
pub fn detect_in(live: &[Match], saved: Option<u32>) -> Option<Detected> {
    static LAST_LINE: Mutex<Option<String>> = Mutex::new(None);
    let found = if let Some((shard, colour)) = crate::ffi::mumble_where() {
        let found = gw2lurker_core::detect::of_shard(live, shard, colour);
        let mut last = LAST_LINE.lock().unwrap_or_else(|error| error.into_inner());
        match detection_line(found.as_ref(), shard, colour, saved) {
            Some(line) if crate::diagnostics::changed(&mut last, &line) => log::info!("{line}"),
            Some(_) => {}
            None => *last = None,
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
    use gw2lurker_core::TeamColor;

    fn detected(team: u32) -> Detected {
        Detected {
            match_id: "2-2".to_owned(),
            team,
            colour: TeamColor::Blue,
            colour_confirmed: true,
        }
    }

    #[test]
    fn rf03a_missing_mumble_data_is_retried_after_a_cooldown() {
        let now = Instant::now();
        assert!(!retry_due(now, now));
        assert!(retry_due(now - RETRY_AFTER_MISSING_DATA, now));
    }

    #[test]
    fn rf03a_a_detection_that_agrees_with_the_saved_team_says_nothing() {
        assert_eq!(
            detection_line(Some(&detected(12014)), 7, None, Some(12014)),
            None
        );
    }

    #[test]
    fn rf03a_a_detection_that_disagrees_with_the_saved_team_says_both() {
        let line = detection_line(Some(&detected(12014)), 7, None, Some(11011))
            .expect("a disagreement is reported");
        assert!(line.contains("2-2") && line.contains("11011"), "{line}");
    }

    #[test]
    fn rf03a_a_shard_in_no_matchup_is_reported_with_the_saved_team() {
        let line = detection_line(None, 7, Some(TeamColor::Red), Some(12014))
            .expect("not finding the matchup is reported");
        assert!(line.contains("12014") && line.contains("shard 7"), "{line}");
    }
}
