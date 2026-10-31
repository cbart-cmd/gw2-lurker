//! Which team the current character plays for (RF-03a).
//!
//! Two sources, in this order:
//! 1. the **manual override** in the settings window, which wins for every character — it is
//!    the escape hatch for the odd case where the old and the new team share a matchup;
//! 2. the **per-character roster**, keyed by the Mumble character name, so a second account
//!    in the other region gets its own picker instead of inheriting the main's team.

use gw2lurker_core::Roster;

/// Team of the current character, if one has been chosen.
pub fn selected() -> Option<u32> {
    let config = crate::runtime();
    if let Some(team) = config.team_id {
        return Some(team);
    }
    let character = crate::ffi::character_name()?;
    config.roster.get(&character)
}

/// Remembers the team for the current character and clears the manual override, so the picker
/// keeps following the character instead of pinning everyone to one team.
pub fn remember(team_id: u32) {
    let character = crate::ffi::character_name();
    let mut config = crate::runtime();
    config.team_id = None;
    if let Some(character) = character {
        config.roster.set(&character, team_id);
    }
}

/// Forgets the team of the current character.
pub fn forget() {
    let character = crate::ffi::character_name();
    let mut config = crate::runtime();
    config.team_id = None;
    if let Some(character) = character {
        config.roster.forget(&character);
    }
}

/// Roster as stored, for the picker (T-30) and tests.
#[allow(dead_code)]
pub fn roster() -> Roster {
    crate::runtime().roster.clone()
}
