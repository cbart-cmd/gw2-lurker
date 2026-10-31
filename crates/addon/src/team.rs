//! Which team the current character plays for (RF-03a).
//!
//! Two sources, in this order:
//! 1. the **manual override** in the settings window, which wins for every character — it is
//!    the escape hatch for the odd case where the old and the new team share a matchup;
//! 2. the **per-character roster**, keyed by the Mumble character name, so a second account
//!    in the other region gets its own picker instead of inheriting the main's team.
//!
//! **Every entry point has an `_in` variant that takes the configuration by reference.** The
//! locking wrappers must never run while the caller already holds the configuration:
//! `std::sync::Mutex` is not reentrant, and calling `selected()` from inside the settings
//! window froze the game on 2026-10-31, the first time it was drawn after entering a map.

use gw2lurker_core::Roster;

use crate::config::Config;

/// Team of the current character, reading an already-locked configuration.
pub fn selected_in(config: &Config) -> Option<u32> {
    resolve(
        config.team_id,
        &config.roster,
        crate::ffi::character_name().as_deref(),
    )
}

/// The actual rule, pure so it can be tested without Nexus: the manual override wins, then
/// the roster entry for that character.
pub fn resolve(manual: Option<u32>, roster: &Roster, character: Option<&str>) -> Option<u32> {
    match (manual, character) {
        (Some(team), _) => Some(team),
        (None, Some(character)) => roster.get(character),
        (None, None) => None,
    }
}

/// Remembers the team for the current character and clears the manual override.
pub fn remember_in(config: &mut Config, team_id: u32) {
    let character = crate::ffi::character_name();
    config.team_id = None;
    if let Some(character) = character {
        config.roster.set(&character, team_id);
    }
}

/// Forgets the team of the current character.
pub fn forget_in(config: &mut Config) {
    let character = crate::ffi::character_name();
    config.team_id = None;
    if let Some(character) = character {
        config.roster.forget(&character);
    }
}

/// Team of the current character. Locks the configuration: never call it with a guard alive.
pub fn selected() -> Option<u32> {
    selected_in(&crate::runtime())
}

/// Remembers the team for the current character. Locks the configuration.
pub fn remember(team_id: u32) {
    remember_in(&mut crate::runtime(), team_id);
}

/// Forgets the team of the current character. Locks the configuration.
pub fn forget() {
    forget_in(&mut crate::runtime());
}

/// Roster as stored, for the picker (T-30) and tests.
#[allow(dead_code)]
pub fn roster() -> Roster {
    crate::runtime().roster.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rf03a_override_wins_over_the_roster() {
        let mut roster = Roster::default();
        roster.set("Personaje", 12014);

        assert_eq!(
            resolve(Some(12015), &roster, Some("Personaje")),
            Some(12015)
        );
        assert_eq!(resolve(None, &roster, Some("Personaje")), Some(12014));
        assert_eq!(
            resolve(None, &roster, Some("Otro")),
            None,
            "un personaje sin equipo no hereda el de otro"
        );
        assert_eq!(
            resolve(None, &roster, None),
            None,
            "sin Mumble no se inventa nada"
        );
    }
}
