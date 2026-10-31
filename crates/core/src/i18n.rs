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
}

/// The table: `(english key, [es, en, fr, de])`.
///
/// Objective-type terms (`Camp`, `Tower`, `Keep`, `Castle`) are the ones T-41 has to confirm
/// with native speakers; they follow the official game terminology as closely as we can
/// without a client in that language.
pub const TEXT: &[(&str, [&str; COLUMNS])] = &[
    (
        "WvW Lurker",
        ["WvW Lurker", "WvW Lurker", "WvW Lurker", "WvW Lurker"],
    ),
    (
        "Loading…",
        ["Cargando…", "Loading…", "Chargement…", "Wird geladen…"],
    ),
    (
        "Map {0} (type {1})",
        [
            "Mapa {0} (tipo {1})",
            "Map {0} (type {1})",
            "Carte {0} (type {1})",
            "Karte {0} (Typ {1})",
        ],
    ),
    (
        "Team {0}",
        ["Equipo {0}", "Team {0}", "Équipe {0}", "Team {0}"],
    ),
    (
        "No team configured yet",
        [
            "Sin equipo configurado todavía",
            "No team configured yet",
            "Aucune équipe configurée pour l'instant",
            "Noch kein Team eingestellt",
        ],
    ),
    (
        "Polling every {0} s",
        [
            "Sondeo cada {0} s",
            "Polling every {0} s",
            "Sondage toutes les {0} s",
            "Abfrage alle {0} s",
        ],
    ),
    (
        "Settings",
        ["Ajustes", "Settings", "Paramètres", "Einstellungen"],
    ),
    ("Alerts", ["Avisos", "Alerts", "Alertes", "Hinweise"]),
    ("Map", ["Mapa", "Map", "Carte", "Karte"]),
    ("Guilds", ["Clanes", "Guilds", "Guildes", "Gilden"]),
    ("History", ["Historial", "History", "Historique", "Verlauf"]),
    ("Close", ["Cerrar", "Close", "Fermer", "Schließen"]),
    (
        "Lock position",
        [
            "Bloquear posición",
            "Lock position",
            "Verrouiller la position",
            "Position sperren",
        ],
    ),
    ("Opacity", ["Opacidad", "Opacity", "Opacité", "Deckkraft"]),
    (
        "Reset position and size",
        [
            "Restablecer posición y tamaño",
            "Reset position and size",
            "Réinitialiser la position et la taille",
            "Position und Größe zurücksetzen",
        ],
    ),
    ("Language", ["Idioma", "Language", "Langue", "Sprache"]),
    ("Team", ["Equipo", "Team", "Équipe", "Team"]),
    ("Alias", ["Alias", "Alias", "Alias", "Alias"]),
    (
        "Poll interval",
        [
            "Intervalo de sondeo",
            "Poll interval",
            "Intervalle de sondage",
            "Abfrageintervall",
        ],
    ),
    (
        "Alert scope",
        [
            "Alcance de los avisos",
            "Alert scope",
            "Portée des alertes",
            "Umfang der Hinweise",
        ],
    ),
    (
        "Current map only",
        [
            "Solo el mapa actual",
            "Current map only",
            "Carte actuelle uniquement",
            "Nur aktuelle Karte",
        ],
    ),
    (
        "All maps of the matchup",
        [
            "Todos los mapas del emparejamiento",
            "All maps of the matchup",
            "Toutes les cartes de la confrontation",
            "Alle Karten des Matchups",
        ],
    ),
    (
        "Auto (game language)",
        [
            "Auto (idioma del juego)",
            "Auto (game language)",
            "Auto (langue du jeu)",
            "Auto (Spielsprache)",
        ],
    ),
    ("Spanish", ["Español", "Spanish", "Espagnol", "Spanisch"]),
    ("English", ["Inglés", "English", "Anglais", "Englisch"]),
    ("French", ["Francés", "French", "Français", "Französisch"]),
    ("German", ["Alemán", "German", "Allemand", "Deutsch"]),
    ("Follow", ["Seguir", "Follow", "Suivre", "Folgen"]),
    ("Following", ["Siguiendo", "Following", "Suivi", "Gefolgt"]),
    (
        "Claimed",
        ["Reclamado", "Claimed", "Revendiqué", "Beansprucht"],
    ),
    (
        "Unclaimed",
        [
            "Sin reclamar",
            "Unclaimed",
            "Non revendiqué",
            "Nicht beansprucht",
        ],
    ),
    ("Camp", ["Campamento", "Camp", "Camp", "Lager"]),
    ("Tower", ["Torre", "Tower", "Tour", "Turm"]),
    ("Keep", ["Fortaleza", "Keep", "Forteresse", "Festung"]),
    ("Castle", ["Castillo", "Castle", "Château", "Schloss"]),
    ("Ruins", ["Ruinas", "Ruins", "Ruines", "Ruinen"]),
    (
        "Mercenary",
        ["Mercenarios", "Mercenary", "Mercenaires", "Söldner"],
    ),
    (
        "High value",
        ["Alto valor", "High value", "Haute valeur", "Hoher Wert"],
    ),
    (
        "Current map",
        [
            "Mapa actual",
            "Current map",
            "Carte actuelle",
            "Aktuelle Karte",
        ],
    ),
    (
        "Other maps",
        [
            "Otros mapas",
            "Other maps",
            "Autres cartes",
            "Andere Karten",
        ],
    ),
    (
        "No claims yet",
        [
            "Sin reclamaciones todavía",
            "No claims yet",
            "Aucune revendication pour l'instant",
            "Noch keine Beanspruchungen",
        ],
    ),
    (
        "Guild unknown",
        [
            "Clan desconocido",
            "Guild unknown",
            "Guilde inconnue",
            "Gilde unbekannt",
        ],
    ),
    (
        "Route inferred from claims; it is not the real path.",
        [
            "Ruta inferida a partir de las reclamaciones; no es el recorrido real.",
            "Route inferred from claims; it is not the real path.",
            "Trajet déduit des revendications ; ce n'est pas le parcours réel.",
            "Weg aus Beanspruchungen abgeleitet; nicht der tatsächliche Verlauf.",
        ],
    ),
];

/// Looks a key up in an arbitrary table. Kept separate so the English fallback can be tested
/// with a table that does have an empty cell, which [`TEXT`] may never have.
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
    lookup(TEXT, language, key)
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
        for (key, cells) in TEXT {
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
    fn rf44_keys_are_unique() {
        for (index, (key, _)) in TEXT.iter().enumerate() {
            let repeated = TEXT[index + 1..].iter().find(|(other, _)| other == key);
            assert!(repeated.is_none(), "clave duplicada: «{key}»");
        }
    }

    #[test]
    fn rf44_unknown_key_is_returned_as_is() {
        assert_eq!(t(Language::De, "not in the table"), "not in the table");
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
