//! Which team the current **account** plays for (RF-03a).
//!
//! Two sources, in this order:
//! 1. the **manual override** in the settings window, which wins for every account — it is
//!    the escape hatch for the odd case where the old and the new team share a matchup;
//! 2. the **per-account memory**, keyed by the account name arcdps reports
//!    (`:Jhoin.1234`), so every character of that account shares the team and a second account
//!    in the other region gets its own picker instead of inheriting the main's team.
//!
//! The character name is only the key while arcdps has not said which account we are in (or is
//! not installed). The key a team is saved under is always the key it is read from, and once the
//! account is known an entry saved under a character is never read: the per-character memory of
//! versions up to 0.26.5 is **not migrated** (decision of the player, 2026-11-01).
//!
//! **Every entry point has an `_in` variant that takes the configuration by reference.** The
//! locking wrappers must never run while the caller already holds the configuration:
//! `std::sync::Mutex` is not reentrant, and calling `selected()` from inside the settings
//! window froze the game on 2026-10-31, the first time it was drawn after entering a map.

use gw2lurker_core::Roster;

use crate::config::Config;

/// Key the team is saved under: the **account** wins; the character is the fallback while arcdps
/// has not said which account we are in, or is not installed.
pub fn key(account: Option<&str>, character: Option<&str>) -> Option<String> {
    account
        .map(str::trim)
        .filter(|account| !account.is_empty())
        .or_else(|| {
            character
                .map(str::trim)
                .filter(|character| !character.is_empty())
        })
        .map(str::to_owned)
}

/// Key of the player playing right now, or `None` if neither source has an answer yet.
pub fn current_key() -> Option<String> {
    key(
        crate::arc_team::self_account().as_deref(),
        crate::ffi::character_name().as_deref(),
    )
}

/// Team of the current account, reading an already-locked configuration.
pub fn selected_in(config: &Config) -> Option<u32> {
    resolve(
        config.team_id,
        &config.roster,
        crate::arc_team::self_account().as_deref(),
        crate::ffi::character_name().as_deref(),
    )
}

/// The actual rule, pure so it can be tested without Nexus: the manual override wins, then the
/// entry under the current key, which is the account (or the character only while arcdps has not
/// said which account we are in, or is not installed). Once the account is known an entry saved
/// under a character is never read.
pub fn resolve(
    manual: Option<u32>,
    roster: &Roster,
    account: Option<&str>,
    character: Option<&str>,
) -> Option<u32> {
    manual.or_else(|| key(account, character).and_then(|key| roster.get(&key)))
}

/// Remembers the team for the current account and clears the manual override.
pub fn remember_in(config: &mut Config, team_id: u32) {
    remember_under(config, current_key().as_deref(), team_id);
}

/// Forgets the team of the current account.
pub fn forget_in(config: &mut Config) {
    forget_under(config, current_key().as_deref());
}

/// The rule behind [`remember_in`], pure so it can be tested: the pick is saved under the same
/// key [`resolve`] reads, and it replaces a manual override, which would hide it otherwise.
fn remember_under(config: &mut Config, key: Option<&str>, team_id: u32) {
    config.team_id = None;
    if let Some(key) = key {
        config.roster.set(key, team_id);
    }
}

/// The rule behind [`forget_in`]: it clears the key [`resolve`] reads and no other.
fn forget_under(config: &mut Config, key: Option<&str>) {
    config.team_id = None;
    if let Some(key) = key {
        config.roster.forget(key);
    }
}

/// Team of the current account. Locks the configuration: never call it with a guard alive.
pub fn selected() -> Option<u32> {
    selected_in(&crate::runtime())
}

/// Remembers the team for the current account. Locks the configuration.
pub fn remember(team_id: u32) {
    remember_in(&mut crate::runtime(), team_id);
}

/// Forgets the team of the current account. Locks the configuration.
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
        roster.set("Jhoin.1234", 12014);

        assert_eq!(
            resolve(Some(12015), &roster, Some("Jhoin.1234"), Some("Jhoin")),
            Some(12015)
        );
        assert_eq!(
            resolve(None, &roster, Some("Jhoin.1234"), Some("Jhoin")),
            Some(12014)
        );
        assert_eq!(
            resolve(None, &roster, Some("Otra.5678"), Some("Otro")),
            None,
            "una cuenta sin equipo no hereda el de otra"
        );
        assert_eq!(
            resolve(None, &roster, None, None),
            None,
            "sin Mumble ni arcdps no se inventa nada"
        );
    }

    #[test]
    fn rf03a_the_team_is_remembered_per_account_and_shared_by_its_characters() {
        // Petición del jugador (2026-11-01): el equipo se guarda por cuenta, así que todos los
        // personajes de esa cuenta lo comparten y una segunda cuenta no lo hereda.
        let mut roster = Roster::default();
        let account = key(Some("Jhoin.1234"), Some("Jhoin"));
        assert_eq!(account.as_deref(), Some("Jhoin.1234"));
        roster.set(account.as_deref().expect("cuenta"), 12014);

        let other_character = key(Some("Jhoin.1234"), Some("Alt Del Main"));
        assert_eq!(other_character.as_deref(), Some("Jhoin.1234"));
        assert_eq!(
            resolve(
                None,
                &roster,
                other_character.as_deref(),
                Some("Alt Del Main")
            ),
            Some(12014),
            "otro personaje de la misma cuenta ya tiene equipo: no se pregunta"
        );

        let second_account = key(Some("Alt.5678"), Some("Bot"));
        assert_eq!(second_account.as_deref(), Some("Alt.5678"));
        assert_eq!(
            resolve(None, &roster, second_account.as_deref(), Some("Bot")),
            None,
            "una cuenta nueva empieza en blanco"
        );
        assert_eq!(
            key(None, Some("Jhoin")).as_deref(),
            Some("Jhoin"),
            "sin cuenta todavía, la clave es el personaje (lo que guardaban las versiones viejas)"
        );
        assert_eq!(key(Some("  "), Some(" Jhoin ")).as_deref(), Some("Jhoin"));
    }

    #[test]
    fn rf03a_a_character_entry_is_not_read_once_the_account_is_known() {
        // Decisión del jugador (2026-11-01): sin migración. Una entrada por personaje de
        // ≤ 0.26.4 no sirve cuando arcdps ya ha dado la cuenta; solo vale sin arcdps, donde el
        // personaje es la única clave que hay.
        let mut roster = Roster::default();
        roster.set("Jhoin", 11011);

        assert_eq!(
            resolve(None, &roster, Some(":Jhoin.1234"), Some("Jhoin")),
            None,
            "con la cuenta conocida solo cuenta la cuenta"
        );
        assert_eq!(
            resolve(None, &roster, None, Some("Jhoin")),
            Some(11011),
            "sin cuenta (sin arcdps) el personaje es la clave"
        );
    }

    #[test]
    fn rf03a_picking_a_team_saves_it_under_the_account_only() {
        // Con un equipo manual puesto, elegir otro lo sustituye: si no, la elección no se vería.
        let mut config = Config {
            team_id: Some(11011),
            ..Config::default()
        };

        let key = key(Some(":Jhoin.1234"), Some("Jhoin"));
        remember_under(&mut config, key.as_deref(), 12014);

        assert_eq!(config.team_id, None);
        assert_eq!(config.roster.len(), 1, "una sola entrada, la de la cuenta");
        assert_eq!(config.roster.get(":Jhoin.1234"), Some(12014));
        assert_eq!(config.roster.get("Jhoin"), None, "nada por personaje");
    }

    #[test]
    fn rf03a_forgetting_clears_the_key_that_was_read_and_nothing_else() {
        let mut config = Config::default();
        config.roster.set(":Jhoin.1234", 12014);
        config.roster.set(":Otra.5678", 11011);

        let key = key(Some(":Jhoin.1234"), Some("Jhoin"));
        forget_under(&mut config, key.as_deref());

        assert_eq!(
            resolve(None, &config.roster, Some(":Jhoin.1234"), Some("Jhoin")),
            None
        );
        assert_eq!(
            config.roster.get(":Otra.5678"),
            Some(11011),
            "las demás cuentas no se tocan"
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
