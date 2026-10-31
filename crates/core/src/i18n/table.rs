//! The translation table itself (RF-44).
//!
//! Kept apart from the lookup logic so the file that grows with every new string does not
//! push the module past the 500-line advice in AGENTS.md §2.
//!
//! Half of the entries: the rest are in [`super::more`] (the file was over 800 lines, 2026-10-31).

use super::COLUMNS;

pub mod more;

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
    (
        "Matchup claims",
        [
            "Objetivos del matchup",
            "Matchup claims",
            "Prises du match",
            "Eroberungen des Matchups",
        ],
    ),
    ("Alert log", ["Registro de avisos", "Alert log", "Journal des alertes", "Hinweisprotokoll"]),
    ("Show widget", ["Ver la ventana", "Show widget", "Voir la fenêtre", "Fenster zeigen"]),
    ("Test pop-up", ["Probar pop-up", "Test pop-up", "Tester le pop-up", "Pop-up testen"]),
    (
        "Your name for this guild",
        [
            "Tu nombre para este clan",
            "Your name for this guild",
            "Votre nom pour cette guilde",
            "Dein Name für diese Gilde",
        ],
    ),
    ("None followed yet", ["Ninguno seguido todavía", "None followed yet", "Aucune suivie pour l'instant", "Noch keine verfolgt"]),
    ("Add guild", ["Añadir clan", "Add guild", "Ajouter une guilde", "Gilde hinzufügen"]),
    (
        "Claimed since the addon started",
        [
            "Han reclamado desde que arrancó",
            "Claimed since the addon started",
            "Ont revendiqué depuis le démarrage",
            "Haben seit dem Start erobert",
        ],
    ),
    ("None yet", ["Ninguno todavía", "None yet", "Aucun pour l'instant", "Noch keiner"]),
    (
        "Search a guild",
        [
            "Buscar un clan",
            "Search a guild",
            "Chercher une guilde",
            "Gilde suchen",
        ],
    ),
    (
        "The saved team ({0}) is not playing right now: pick it again in Settings.",
        [
            "El equipo guardado ({0}) no está jugando ahora mismo: vuelve a elegirlo en Ajustes.",
            "The saved team ({0}) is not playing right now: pick it again in Settings.",
            "L'équipe enregistrée ({0}) ne joue pas en ce moment : choisissez-la à nouveau dans les réglages.",
            "Das gespeicherte Team ({0}) spielt gerade nicht: wähle es in den Einstellungen neu.",
        ],
    ),
    (
        "Pick your team first: the search needs to know your region.",
        [
            "Elige tu equipo primero: la búsqueda necesita saber tu región.",
            "Pick your team first: the search needs to know your region.",
            "Choisissez d'abord votre équipe : la recherche a besoin de votre région.",
            "Wähle zuerst dein Team: die Suche braucht deine Region.",
        ],
    ),
    (
        "The list of teams arrives with the next poll.",
        [
            "La lista de equipos llega con el próximo sondeo.",
            "The list of teams arrives with the next poll.",
            "La liste des équipes arrive à la prochaine interrogation.",
            "Die Teamliste kommt mit der nächsten Abfrage.",
        ],
    ),
    ("Team (any)", ["Equipo (cualquiera)", "Team (any)", "Équipe (toutes)", "Team (beliebig)"]),
    (
        "Click to see this guild's route",
        [
            "Clic para ver la ruta de este clan",
            "Click to see this guild's route",
            "Cliquez pour voir la route de cette guilde",
            "Klicken, um die Route dieser Gilde zu sehen",
        ],
    ),
    (
        "Updated to a newer version (running {0}): restart the game to apply it.",
        [
            "Hay una versión más nueva en disco (estás con la {0}): reinicia el juego para aplicarla.",
            "Updated to a newer version (running {0}): restart the game to apply it.",
            "Une version plus récente est sur le disque ({0} en cours) : relancez le jeu.",
            "Eine neuere Version liegt auf der Platte (läuft: {0}): Spiel neu starten.",
        ],
    ),
    (
        "Window style",
        [
            "Estilo de ventanas",
            "Window style",
            "Style des fenêtres",
            "Fensterstil",
        ],
    ),
    (
        "Inherited from Nexus",
        [
            "Heredado de Nexus",
            "Inherited from Nexus",
            "Hérité de Nexus",
            "Von Nexus übernommen",
        ],
    ),
    (
        "Addon's own dark theme",
        [
            "Tema oscuro del addon",
            "Addon's own dark theme",
            "Thème sombre de l'addon",
            "Eigenes dunkles Thema",
        ],
    ),
    ("Background", ["Fondo", "Background", "Fond", "Hintergrund"]),
    (
        "Dark, semi-transparent",
        [
            "Oscuro, semitransparente",
            "Dark, semi-transparent",
            "Sombre, semi-transparent",
            "Dunkel, halbtransparent",
        ],
    ),
    (
        "Transparent",
        ["Transparente", "Transparent", "Transparent", "Transparent"],
    ),
    (
        "Text size",
        ["Tamaño del texto", "Text size", "Taille du texte", "Textgröße"],
    ),
    (
        "Lines in the widget",
        [
            "Líneas en el widget",
            "Lines in the widget",
            "Lignes dans le widget",
            "Zeilen im Widget",
        ],
    ),
    (
        "Show captures",
        [
            "Mostrar capturas",
            "Show captures",
            "Afficher les captures",
            "Eroberungen zeigen",
        ],
    ),
    (
        "Show claims",
        [
            "Mostrar reclamaciones",
            "Show claims",
            "Afficher les revendications",
            "Beanspruchungen zeigen",
        ],
    ),
    (
        "Show releases",
        [
            "Mostrar liberaciones",
            "Show releases",
            "Afficher les libérations",
            "Freigaben zeigen",
        ],
    ),
    ("Play route", ["Reproducir ruta", "Play route", "Lire la route", "Route abspielen"]),
    ("Pause route", ["Pausar ruta", "Pause route", "Mettre en pause", "Route pausieren"]),
    ("Whole route", ["Ruta completa", "Whole route", "Route entière", "Ganze Route"]),
    (
        "Nexus pop-up for everything I follow",
        [
            "Aviso emergente (Nexus) de todo lo que sigo",
            "Nexus pop-up for everything I follow",
            "Notification (Nexus) de tout ce que je suis",
            "Hinweis (Nexus) für alles, dem ich folge",
        ],
    ),
    (
        "The one Nexus draws, on top of the game, even with the widget closed.",
        [
            "El que dibuja Nexus, encima del juego, aunque el widget esté cerrado.",
            "The one Nexus draws, on top of the game, even with the widget closed.",
            "Celui que dessine Nexus, par-dessus le jeu, même widget fermé.",
            "Der, den Nexus über dem Spiel zeichnet, auch bei geschlossenem Widget.",
        ],
    ),
    (
        "jump to {0}",
        [
            "salto a {0}",
            "jump to {0}",
            "saut vers {0}",
            "Sprung nach {0}",
        ],
    ),
    (
        "Following guilds lives in the Guilds tab, next to the match you are looking at.",
        [
            "Seguir clanes está en la pestaña Clanes, al lado de la partida que estás mirando.",
            "Following guilds lives in the Guilds tab, next to the match you are looking at.",
            "Suivre des guildes se fait dans l'onglet Guildes, à côté du match que vous regardez.",
            "Gilden folgen geht im Reiter Gilden, direkt neben dem Match, das du siehst.",
        ],
    ),
    ("Search", ["Buscar", "Search", "Chercher", "Suchen"]),
    ("Range", ["Rango", "Range", "Plage", "Zeitraum"]),
    ("Last hour", ["Última hora", "Last hour", "Dernière heure", "Letzte Stunde"]),
    ("Last 2 hours", ["Últimas 2 horas", "Last 2 hours", "2 dernières heures", "Letzte 2 Stunden"]),
    ("All session", ["Toda la sesión", "All session", "Toute la session", "Gesamte Sitzung"]),
    ("Real map", ["Mapa real", "Real map", "Carte réelle", "Echte Karte"]),
    (
        "Part of the name, one word",
        [
            "Parte del nombre, una palabra",
            "Part of the name, one word",
            "Une partie du nom, un mot",
            "Teil des Namens, ein Wort",
        ],
    ),
    (
        "The API searches one word by prefix: «Raven», not «avenous».",
        [
            "La API busca una palabra por prefijo: «Raven», no «avenous».",
            "The API searches one word by prefix: «Raven», not «avenous».",
            "L'API cherche un mot par préfixe : « Raven », pas « avenous ».",
            "Die API sucht ein Wort als Präfix: „Raven“, nicht „avenous“.",
        ],
    ),
    (
        "Type part of the name and search.",
        [
            "Escribe parte del nombre y busca.",
            "Type part of the name and search.",
            "Saisissez une partie du nom et cherchez.",
            "Teil des Namens eingeben und suchen.",
        ],
    ),
    ("Searching…", ["Buscando…", "Searching…", "Recherche…", "Suche läuft…"]),
    (
        "Search failed: {0}",
        ["La búsqueda falló: {0}", "Search failed: {0}", "Échec de la recherche : {0}", "Suche fehlgeschlagen: {0}"],
    ),
    (
        "No guild matches.",
        [
            "Ningún clan coincide.",
            "No guild matches.",
            "Aucune guilde ne correspond.",
            "Keine Gilde passt.",
        ],
    ),    (
        "Point at an objective to see who holds it",
        [
            "Pasa el ratón por un objetivo para ver quién lo tiene",
            "Point at an objective to see who holds it",
            "Survolez un objectif pour voir qui le détient",
            "Mit der Maus auf ein Ziel zeigen, um den Besitzer zu sehen",
        ],
    ),
    ("Sample objective", ["Objetivo de ejemplo", "Sample objective", "Objectif d'exemple", "Beispielziel"]),
    ("Test guild", ["[TEST] Clan de prueba", "[TEST] Test guild", "[TEST] Guilde de test", "[TEST] Testgilde"]),
    (
        "Click-through is on: right-click here or press {0} to use the mouse again.",
        [
            "Click-through activo: haz clic derecho aquí o pulsa {0} para volver a usar el ratón.",
            "Click-through is on: right-click here or press {0} to use the mouse again.",
            "Clics traversants actifs : clic droit ici ou {0} pour réutiliser la souris.",
            "Klickdurchlässig aktiv: Rechtsklick hier oder {0}, um die Maus zu nutzen.",
        ],
    ),
    (
        "Nothing to watch on this map, but the settings work here.",
        [
            "En este mapa no hay nada que vigilar, pero los ajustes funcionan.",
            "Nothing to watch on this map, but the settings work here.",
            "Rien à surveiller sur cette carte, mais les réglages fonctionnent.",
            "Auf dieser Karte gibt es nichts zu beobachten, die Einstellungen gehen aber.",
        ],
    ),
    (
        "Pop-up alerts",
        [
            "Avisos emergentes",
            "Pop-up alerts",
            "Alertes surgissantes",
            "Aufpoppende Hinweise",
        ],
    ),
    (
        "Settings could not be read ({0}); running with defaults.",
        [
            "No se pudo leer la configuración ({0}); se usan los valores por defecto.",
            "Settings could not be read ({0}); running with defaults.",
            "Impossible de lire la configuration ({0}) ; valeurs par défaut utilisées.",
            "Die Einstellungen konnten nicht gelesen werden ({0}); es gelten die Vorgaben.",
        ],
    ),
    (
        "Followed guilds ({0})",
        [
            "Clanes seguidos ({0})",
            "Followed guilds ({0})",
            "Guildes suivies ({0})",
            "Verfolgte Gilden ({0})",
        ],
    ),
    (
        "The list is full: stop following one to add another.",
        [
            "La lista está llena: deja de seguir uno para añadir otro.",
            "The list is full: stop following one to add another.",
            "La liste est pleine : arrêtez de suivre une guilde pour en ajouter une autre.",
            "Die Liste ist voll: eine Gilde entfolgen, um eine andere hinzuzufügen.",
        ],
    ),
    (
        "Stop following {0}?",
        [
            "¿Quitar {0} de los que sigues?",
            "Stop following {0}?",
            "Ne plus suivre {0} ?",
            "{0} nicht mehr folgen?",
        ],
    ),
];
