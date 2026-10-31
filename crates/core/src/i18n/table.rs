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
        "Nothing matches the filters",        [
            "Nada cumple los filtros",
            "Nothing matches the filters",
            "Rien ne correspond aux filtres",
            "Nichts entspricht den Filtern",
        ],
    ),
    (
        "Last {0} minutes; the log window has the rest.",
        [
            "Últimos {0} minutos; el registro tiene el resto.",
            "Last {0} minutes; the log window has the rest.",
            "Les {0} dernières minutes ; le journal a le reste.",
            "Die letzten {0} Minuten; das Protokoll hat den Rest.",
        ],
    ),
    ("History", ["Historial", "History", "Historique", "Verlauf"]),
    ("Close", ["Cerrar", "Close", "Fermer", "Schließen"]),
    (
        "Click-through",
        ["Click-through", "Click-through", "Clics traversants", "Klickdurchlässig"],
    ),
    (
        "Click-through (no mouse)",
        [
            "Click-through (sin ratón)",
            "Click-through (no mouse)",
            "Clics traversants (sans souris)",
            "Klickdurchlässig (keine Maus)",
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
    ("Save", ["Grabar", "Save", "Enregistrer", "Speichern"]),
    ("Windows", ["Ventanas", "Windows", "Fenêtres", "Fenster"]),
    (
        "Click-through keeps the mouse for the game. The shortcut {0} brings the window back, and it can be rebound in Nexus.",
        [
            "Click-through deja el ratón para el juego. El atajo {0} devuelve la ventana, y se puede reasignar en Nexus.",
            "Click-through keeps the mouse for the game. The shortcut {0} brings the window back, and it can be rebound in Nexus.",
            "Les clics traversants laissent la souris au jeu. Le raccourci {0} ramène la fenêtre, et il se modifie dans Nexus.",
            "Klickdurchlässig lässt die Maus beim Spiel. Das Kürzel {0} holt das Fenster zurück, änderbar in Nexus.",
        ],
    ),
    ("from their keep", ["desde su fortaleza", "from their keep", "depuis leur forteresse", "aus ihrer Festung"]),
    ("from their side keep", ["desde su fortaleza lateral", "from their side keep", "depuis leur forteresse latérale", "aus ihrer Seitenfestung"]),
    ("from the spawn point", ["desde el punto de reaparición", "from the spawn point", "depuis le point de réapparition", "vom Wiederbelebungspunkt"]),
    (
        "The changes are applied as you edit; this writes them now.",
        [
            "Los cambios se aplican al editarlos; esto los escribe ahora.",
            "The changes are applied as you edit; this writes them now.",
            "Les changements s'appliquent pendant la saisie ; ceci les enregistre maintenant.",
            "Änderungen gelten sofort; dies schreibt sie jetzt.",
        ],
    ),
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
        "Ordered by most recent claim",
        [
            "Ordenados por la reclamación más reciente",
            "Ordered by most recent claim",
            "Triés par revendication la plus récente",
            "Nach der neuesten Beanspruchung sortiert",
        ],
    ),
    (
        "Pick a guild to see its history",
        [
            "Elige un clan para ver su historial",
            "Pick a guild to see its history",
            "Choisissez une guilde pour voir son historique",
            "Wähle eine Gilde, um ihren Verlauf zu sehen",
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
    ("Alert on", ["Avisar de", "Alert on", "Alerter sur", "Hinweis bei"]),
    ("Eternals", ["Eternos", "Eternals", "Éternel", "Ewige Schlachtfelder"]),
    ("Blue alpine", ["Alpino azul", "Blue alpine", "Alpin bleu", "Blaues Alpin"]),
    ("Green alpine", ["Alpino verde", "Green alpine", "Alpin vert", "Grünes Alpin"]),
    ("Red desert", ["Desierto rojo", "Red desert", "Désert rouge", "Rote Wüste"]),
    ("The map I am on", ["El mapa en el que estoy", "The map I am on", "La carte où je suis", "Die Karte, auf der ich bin"]),
    ("Guilds I follow", ["Los clanes que sigo", "Guilds I follow", "Les guildes que je suis", "Gilden, denen ich folge"]),
    ("Pop-up seconds", ["Segundos del aviso emergente", "Pop-up seconds", "Secondes de la notification", "Sekunden des Hinweises"]),
    (
        "The pop-up disappears after this. The alert stays in the list.",
        [
            "El aviso emergente desaparece pasado este tiempo. El aviso sigue en la lista.",
            "The pop-up disappears after this. The alert stays in the list.",
            "La notification disparaît après ce délai. L'alerte reste dans la liste.",
            "Der Hinweis verschwindet danach. Der Eintrag bleibt in der Liste.",
        ],
    ),
    ("Followed guilds", ["Clanes seguidos", "Followed guilds", "Guildes suivies", "Gefolgte Gilden"]),
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
    ("Unknown map", ["Mapa desconocido", "Unknown map", "Carte inconnue", "Unbekannte Karte"]),
    (
        "The list of teams arrives with the first poll; wait a few seconds.",
        [
            "La lista de equipos llega con el primer sondeo; espera unos segundos.",
            "The list of teams arrives with the first poll; wait a few seconds.",
            "La liste des équipes arrive au premier sondage ; patientez quelques secondes.",
            "Die Teamliste kommt mit der ersten Abfrage; warte ein paar Sekunden.",
        ],
    ),
    ("you are here", ["estás aquí", "you are here", "vous êtes ici", "du bist hier"]),
    ("High value only", ["Solo alto valor", "High value only", "Haute valeur seulement", "Nur hoher Wert"]),
    ("Camps", ["Campamentos", "Camps", "Camps", "Lager"]),
    ("Towers", ["Torres", "Towers", "Tours", "Türme"]),
    ("Keeps", ["Fortalezas", "Keeps", "Forteresses", "Festungen"]),
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
