//! The translation table itself (RF-44).
//!
//! Kept apart from the lookup logic so the file that grows with every new string does not
//! push the module past the 500-line advice in AGENTS.md §2.

use super::COLUMNS;

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
    ("Translation", ["Traducción", "Translation", "Traduction", "Übersetzung"]),
    (
        "Translation (only for you)",
        [
            "Traducción (solo para tu instalación)",
            "Translation (only for your install)",
            "Traduction (pour votre installation seulement)",
            "Übersetzung (nur für deine Installation)",
        ],
    ),
    (
        "Prefilled with the name in the game language. Clear it to use the automatic one.",
        [
            "Relleno con el nombre en el idioma del juego. Vacíalo para volver al automático.",
            "Prefilled with the name in the game language. Clear it to use the automatic one.",
            "Pré-rempli avec le nom dans la langue du jeu. Videz-le pour revenir à l'automatique.",
            "Vorausgefüllt mit dem Namen in der Spielsprache. Leeren, um den automatischen zu nutzen.",
        ],
    ),
    ("Region", ["Región", "Region", "Région", "Region"]),
    ("Colour", ["Color", "Colour", "Couleur", "Farbe"]),
    ("Europe", ["Europa", "Europe", "Europe", "Europa"]),
    (
        "North America",
        [
            "Norteamérica",
            "North America",
            "Amérique du Nord",
            "Nordamerika",
        ],
    ),
    ("Red", ["Rojo", "Red", "Rouge", "Rot"]),
    ("Green", ["Verde", "Green", "Vert", "Grün"]),
    ("Blue", ["Azul", "Blue", "Bleu", "Blau"]),
    (
        "No live teams for that region and colour",
        [
            "No hay equipos en juego con esa región y color",
            "No live teams for that region and colour",
            "Aucune équipe en jeu pour cette région et cette couleur",
            "Keine laufenden Teams für diese Region und Farbe",
        ],
    ),
    (
        "Pick region and colour, then your team",
        [
            "Elige región y color, luego tu equipo",
            "Pick region and colour, then your team",
            "Choisissez la région et la couleur, puis votre équipe",
            "Wähle Region und Farbe, dann dein Team",
        ],
    ),
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
        "No data yet",
        [
            "Sin datos todavía",
            "No data yet",
            "Pas encore de données",
            "Noch keine Daten",
        ],
    ),
    (
        "Matchup {0}",
        [
            "Emparejamiento {0}",
            "Matchup {0}",
            "Confrontation {0}",
            "Matchup {0}",
        ],
    ),
    (
        "Last update {0}",
        [
            "Última actualización {0}",
            "Last update {0}",
            "Dernière mise à jour {0}",
            "Letzte Aktualisierung {0}",
        ],
    ),
    (
        "Error: {0}",
        ["Error: {0}", "Error: {0}", "Erreur : {0}", "Fehler: {0}"],
    ),
    (
        "{0} claimed objectives",
        [
            "{0} objetivos reclamados",
            "{0} claimed objectives",
            "{0} objectifs revendiqués",
            "{0} beanspruchte Ziele",
        ],
    ),
    (
        "{0} guilds",
        ["{0} clanes", "{0} guilds", "{0} guildes", "{0} Gilden"],
    ),
    ("No alerts yet", ["Sin avisos todavía", "No alerts yet", "Pas encore d'alertes", "Noch keine Hinweise"]),
    (
        "Already claimed when the addon started",
        [
            "Ya estaba reclamado cuando arrancó el addon",
            "Already claimed when the addon started",
            "Déjà revendiqué au démarrage de l'addon",
            "Beim Start des Addons bereits beansprucht",
        ],
    ),
    ("{0}: {1} → {2}", ["{0}: {1} → {2}", "{0}: {1} → {2}", "{0} : {1} → {2}", "{0}: {1} → {2}"]),
    (
        "{0} claimed by {1}",
        [
            "{0} reclamado por {1}",
            "{0} claimed by {1}",
            "{0} revendiqué par {1}",
            "{0} beansprucht von {1}",
        ],
    ),
    (
        "{0} released by {1}",
        [
            "{0} liberado por {1}",
            "{0} released by {1}",
            "{0} libéré par {1}",
            "{0} freigegeben von {1}",
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
