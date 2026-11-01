//! Translation table (RF-44).
//!
//! Every user-visible string is indexed by its **English original**, which doubles as the
//! stable imgui id. The four first-class languages are Spanish, English, French and German:
//! there are real users of each, so **no column may be empty** and a test enforces it.
//!
//! This lives in `core` on purpose: the table and the fallback rules are pure, so the
//! completeness check runs on the Linux CI job, which cannot build the addon.

/// Columns per entry, in this order.
pub const COLUMNS: usize = 4;

/// Position of each language inside an entry.
const ES: usize = 0;
const EN: usize = 1;
const FR: usize = 2;
const DE: usize = 3;

/// Language of the interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Language {
    Es,
    #[default]
    En,
    Fr,
    De,
}

impl Language {
    /// All languages, in the table order.
    pub const ALL: [Language; COLUMNS] = [Language::Es, Language::En, Language::Fr, Language::De];

    /// Index of this language inside a table entry.
    pub fn index(self) -> usize {
        match self {
            Language::Es => ES,
            Language::En => EN,
            Language::Fr => FR,
            Language::De => DE,
        }
    }

    /// Two-letter code used by the API (`?lang=`), the config and Nexus.
    pub fn code(self) -> &'static str {
        match self {
            Language::Es => "es",
            Language::En => "en",
            Language::Fr => "fr",
            Language::De => "de",
        }
    }

    /// Parses a two-letter code, as stored in the configuration.
    pub fn from_code(code: &str) -> Option<Self> {
        Language::ALL
            .into_iter()
            .find(|language| language.code() == code.to_ascii_lowercase())
    }

    /// Maps Nexus RTAPI `GameData::language` (`GameLanguage` in `nexus-rs`).
    ///
    /// Verified against `nexus/src/api/rtapi/game.rs` in tag 0.12.0:
    /// `English = 0`, `Korean = 1`, `French = 2`, `German = 3`, `Spanish = 4`, `Chinese = 5`.
    /// Korean and Chinese have no column here, so they fall back to English, which is what
    /// `t` does with any language it has no string for.
    pub fn from_game_code(code: u32) -> Option<Self> {
        match code {
            0 => Some(Language::En),
            2 => Some(Language::Fr),
            3 => Some(Language::De),
            4 => Some(Language::Es),
            _ => None,
        }
    }

    /// Maps a Windows primary language id (low 10 bits of `LANGID`).
    ///
    /// Used only when the RealTime API is not installed, which is common: without it the
    /// game language is unreadable, and guessing from the OS language beats showing English
    /// to someone whose client is Spanish, French or German.
    pub fn from_windows_language(primary: u16) -> Option<Self> {
        match primary {
            0x09 => Some(Language::En),
            0x0A => Some(Language::Es),
            0x0C => Some(Language::Fr),
            0x07 => Some(Language::De),
            _ => None,
        }
    }
}

mod table;
pub use table::TEXT;

/// All the entries of the table, in two files so neither grows past the line limit.
fn entries() -> impl Iterator<Item = (&'static str, [&'static str; COLUMNS])> {
    TEXT.iter().chain(table::more::TEXT.iter()).copied()
}

/// Looks a key up in an arbitrary table. Kept separate so the English fallback can be tested
/// with a table that does have an empty cell, which the real table may never have.
#[cfg(test)]
fn lookup(
    table: &'static [(&'static str, [&'static str; COLUMNS])],
    language: Language,
    key: &'static str,
) -> &'static str {
    let Some((_, cells)) = table.iter().find(|(english, _)| *english == key) else {
        return key;
    };
    let cell = cells[language.index()];
    if cell.is_empty() {
        cells[EN]
    } else {
        cell
    }
}

/// Translation of `key`, falling back to the key itself when it is not in the table.
pub fn t(language: Language, key: &'static str) -> &'static str {
    let Some((_, cells)) = entries().find(|(english, _)| *english == key) else {
        return key;
    };
    let cell = cells[language.index()];
    if cell.is_empty() {
        cells[EN]
    } else {
        cell
    }
}

/// Translation with a stable imgui id appended.
///
/// imgui identifies widgets by label, so a label that changes with the language would lose
/// its state (collapsed, position of a popup…). The `###` suffix keeps the id constant.
pub fn w(language: Language, key: &'static str) -> String {
    format!("{}###{key}", t(language, key))
}

/// Translation with `{0}`, `{1}`… replaced. A placeholder with no argument is left as is.
pub fn f(language: Language, key: &'static str, args: &[&str]) -> String {
    let mut out = t(language, key).to_owned();
    for (index, value) in args.iter().enumerate() {
        out = out.replace(&format!("{{{index}}}"), value);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rf44_every_column_is_filled() {
        for (key, cells) in entries() {
            for language in Language::ALL {
                assert!(
                    !cells[language.index()].is_empty(),
                    "falta {} para «{key}»",
                    language.code()
                );
            }
        }
    }

    #[test]
    fn rf44_the_table_stays_inside_latin_1() {
        // El atlas de fuentes de Nexus trae latín básico y latino-1, y nada más: una flecha
        // (`→`), un punto suspensivo (`…`) o una estrella (`★`) se dibujan como «?». El jugador
        // lo vio en la ventana de avisos el 2026-11-01 («a veces pone un ? que creía que era una
        // flecha»), así que la tabla se queda en ASCII + latino-1 y esto lo impide volver.
        let latin1 = |text: &str| text.chars().all(|character| character <= '\u{00ff}');
        for (key, cells) in entries() {
            assert!(
                latin1(key),
                "la clave «{key}» tiene un carácter fuera de latino-1"
            );
            for language in Language::ALL {
                let cell = cells[language.index()];
                assert!(
                    latin1(cell),
                    "«{cell}» ({}) tiene un carácter fuera de latino-1",
                    language.code()
                );
            }
        }
    }

    #[test]
    fn rf44_keys_are_unique() {
        let all: Vec<(&str, [&str; COLUMNS])> = entries().collect();
        for index in 0..all.len() {
            let key = all[index].0;
            let repeated = all[index + 1..].iter().any(|(other, _)| *other == key);
            assert!(!repeated, "clave duplicada: «{key}»");
        }
    }

    #[test]
    fn rf44_unknown_key_is_returned_as_is() {
        assert_eq!(t(Language::De, "not in the table"), "not in the table");
    }

    #[test]
    fn rf44_both_halves_of_the_table_are_looked_up() {
        // La tabla vive en dos ficheros: una clave de cada mitad tiene que traducirse, o la
        // segunda mitad se quedaría sin usar en silencio.
        assert_eq!(t(Language::Es, "Region"), "Región");
        assert_eq!(t(Language::Es, "Red desert"), "Desierto rojo");
    }

    #[test]
    fn rf44_empty_cell_falls_back_to_english() {
        static PARTIAL: &[(&str, [&str; COLUMNS])] =
            &[("Only English", ["", "Only English", "", ""])];
        assert_eq!(
            lookup(PARTIAL, Language::Fr, "Only English"),
            "Only English"
        );
        assert_eq!(
            lookup(PARTIAL, Language::En, "Only English"),
            "Only English"
        );
    }

    #[test]
    fn rf44_format_substitutes_every_placeholder() {
        assert_eq!(f(Language::Es, "Team {0}", &["12014"]), "Equipo 12014");
        assert_eq!(
            f(Language::De, "Map {0} (type {1})", &["38", "9"]),
            "Karte 38 (Typ 9)"
        );
        assert_eq!(
            f(Language::En, "Map {0} (type {1})", &["38"]),
            "Map 38 (type {1})",
            "un hueco sin argumento se queda visible, nunca revienta"
        );
    }

    #[test]
    fn rf44_widget_id_does_not_change_with_language() {
        for language in Language::ALL {
            assert!(w(language, "Settings").ends_with("###Settings"));
        }
        assert_eq!(w(Language::Fr, "Settings"), "Paramètres###Settings");
    }

    #[test]
    fn rf44_game_language_codes_match_nexus() {
        assert_eq!(Language::from_game_code(0), Some(Language::En));
        assert_eq!(Language::from_game_code(2), Some(Language::Fr));
        assert_eq!(Language::from_game_code(3), Some(Language::De));
        assert_eq!(Language::from_game_code(4), Some(Language::Es));
        assert_eq!(
            Language::from_game_code(1),
            None,
            "coreano no tiene columna"
        );
        assert_eq!(Language::from_game_code(5), None, "chino no tiene columna");
        assert_eq!(
            Language::from_game_code(99),
            None,
            "un valor futuro cae a inglés"
        );
    }

    #[test]
    fn rf44_windows_language_is_only_a_fallback() {
        assert_eq!(Language::from_windows_language(0x0A), Some(Language::Es));
        assert_eq!(Language::from_windows_language(0x0C), Some(Language::Fr));
        assert_eq!(Language::from_windows_language(0x07), Some(Language::De));
        assert_eq!(Language::from_windows_language(0x09), Some(Language::En));
        assert_eq!(
            Language::from_windows_language(0x11),
            None,
            "japonés: sin columna"
        );
        assert_eq!(
            Language::from_windows_language(0x040A & 0x3FF),
            Some(Language::Es),
            "con subidioma también funciona"
        );
    }

    #[test]
    fn rf44_language_codes_round_trip() {
        for language in Language::ALL {
            assert_eq!(Language::from_code(language.code()), Some(language));
        }
        assert_eq!(
            Language::from_code("FR"),
            Some(Language::Fr),
            "mayúsculas también"
        );
        assert_eq!(Language::from_code("it"), None);
    }

    #[test]
    fn rf44_spanish_column_is_actually_spanish() {
        assert_eq!(t(Language::Es, "Guilds"), "Clanes");
        assert_eq!(t(Language::Fr, "Guilds"), "Guildes");
        assert_eq!(t(Language::De, "Guilds"), "Gilden");
        assert_eq!(t(Language::En, "Guilds"), "Guilds");
    }
}
