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

/// Name the team has in the bundled tables for a language (no override).
pub fn automatic_name(language: &str, team_id: u32) -> String {
    gw2lurker_core::bundled()
        .name_in(language, team_id)
        .map(str::to_owned)
        .unwrap_or_else(|| format!("({team_id})"))
}

/// Name shown to the player: their own translation wins over the bundled one (RF-03f).
pub fn display_name(config: &Config, language: &str, team_id: u32) -> String {
    config
        .custom_name(team_id, language)
        .map(str::to_owned)
        .unwrap_or_else(|| automatic_name(language, team_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rf03f_a_translation_belongs_to_one_team() {
        // The bug of 2026-10-31: the name written for one team showed up on the next one.
        let mut config = Config::default();
        config.set_custom_name(
            11011,
            "es",
            "Invernadero de Lutgardis",
            "Lutgardis Conservatory",
        );
        config.set_custom_name(12014, "es", "Bosquemugoso", "Pajar de la Gran Casa");

        assert_eq!(
            config.custom_name(11011, "es"),
            Some("Invernadero de Lutgardis")
        );
        assert_eq!(config.custom_name(12014, "es"), Some("Bosquemugoso"));
        assert_eq!(
            config.custom_name(12015, "es"),
            None,
            "un equipo sin traducir no hereda la de otro"
        );
        assert_eq!(
            config.custom_name(11011, "fr"),
            None,
            "ni otro idioma del mismo equipo"
        );
    }

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

    #[test]
    fn rf03f_the_players_translation_wins_over_the_bundled_one() {
        // El nombre automático no se fija aquí a propósito: sale de `data/team-names.json`, y el
        // jugador puede traducirlo cuando quiera (petición de 2026-10-31). Lo que se prueba es el
        // mecanismo: sin nombre propio manda la tabla, con nombre propio manda el suyo, y vaciarlo
        // vuelve a la tabla.
        let mut config = Config::default();
        let automatic = automatic_name("es", 12014);
        assert!(!automatic.trim().is_empty(), "12014 sin nombre en español");
        assert_eq!(
            display_name(&config, "es", 12014),
            automatic,
            "sin traducción propia se usa la de la tabla"
        );

        config.set_custom_name(12014, "es", "Bosquemugoso", &automatic);
        assert_eq!(display_name(&config, "es", 12014), "Bosquemugoso");
        assert_eq!(
            display_name(&config, "en", 12014),
            "Great House Aviary",
            "la traducción es por idioma: en inglés no se cuela el nombre español"
        );

        config.set_custom_name(12014, "es", "  ", &automatic);
        assert_eq!(
            display_name(&config, "es", 12014),
            automatic,
            "vaciar el campo vuelve al automático"
        );
    }

    #[test]
    fn rf03f_writing_the_automatic_name_is_not_an_override() {
        let mut config = Config::default();
        config.set_custom_name(
            12014,
            "es",
            "Pajar de la Gran Casa",
            "Pajar de la Gran Casa",
        );
        assert!(
            config.team_translations.is_empty(),
            "dejarlo como estaba no crea una entrada"
        );

        config.set_custom_name(12014, "fr", "Bosquemugoso", "Volière de la Grande Maison");
        assert_eq!(
            config
                .team_translations
                .get("12014")
                .and_then(|by_language| by_language.get("fr"))
                .map(String::as_str),
            Some("Bosquemugoso")
        );
    }

    #[test]
    fn rf03f_the_old_single_alias_is_still_honoured() {
        let mut config = Config {
            team_alias: Some("Conservatorio de lutgardis".to_owned()),
            ..Config::default()
        };
        assert_eq!(
            display_name(&config, "es", 11011),
            "Conservatorio de lutgardis",
            "el alias de 0.5.x se sigue usando hasta que se guarde la traducción"
        );

        config.set_custom_name(
            11011,
            "es",
            "Invernadero de Lutgardis",
            "Lutgardis Conservatory",
        );
        assert_eq!(config.team_alias, None, "al guardar se migra y se limpia");
        assert_eq!(
            display_name(&config, "es", 11011),
            "Invernadero de Lutgardis"
        );
    }
}
