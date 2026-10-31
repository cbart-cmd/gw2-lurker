//! The second half of the translation table (RF-44).
//!
//! Apart from `table.rs` so neither file goes past the 800-line limit (AGENTS.md §2.7).

use super::COLUMNS;

/// The rest of the table: `(english key, [es, en, fr, de])`.
pub const TEXT: &[(&str, [&str; COLUMNS])] = &[
    (
        "Yes",
        ["Sí", "Yes", "Oui", "Ja"],
    ),
    (
        "No",
        ["No", "No", "Non", "Nein"],
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
    (
        "Match in {0}",
        [
            "Partida en {0}",
            "Match in {0}",
            "Partie en {0}",
            "Gefecht in {0}",
        ],
    ),
    (
        "Unknown region",
        [
            "región desconocida",
            "unknown region",
            "région inconnue",
            "unbekannte Region",
        ],
    ),
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
        "Newest claim {0} min ago",
        [
            "Último reclamo: hace {0} min",
            "Newest claim {0} min ago",
            "Dernière revendication : il y a {0} min",
            "Letzte Beanspruchung: vor {0} Min.",
        ],
    ),
    (
        "You are in {0}",
        [
            "Estás en: {0}",
            "You are in {0}",
            "Vous êtes sur : {0}",
            "Du bist in: {0}",
        ],
    ),
    (
        "Outside WvW (map {0})",
        [
            "Fuera de WvW (mapa {0})",
            "Outside WvW (map {0})",
            "Hors MvM (carte {0})",
            "Außerhalb von WvW (Karte {0})",
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
        "How long each pop-up stays. They come one at a time; the list keeps them all.",
        [
            "Cuánto dura cada aviso emergente. Salen de uno en uno; la lista los guarda todos.",
            "How long each pop-up stays. They come one at a time; the list keeps them all.",
            "Combien de temps reste chaque notification. Elles arrivent une par une ; la liste les garde toutes.",
            "Wie lange jeder Hinweis bleibt. Sie kommen einzeln; die Liste behält alle.",
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
    (
        "{0} · you already follow it",
        [
            "{0} · ya lo sigues",
            "{0} · you already follow it",
            "{0} · vous la suivez déjà",
            "{0} · du folgst ihr schon",
        ],
    ),
    (
        "The API has nothing else by that name; the guild you already follow is above.",
        [
            "El API no tiene nada más con ese nombre; el clan que ya sigues está arriba.",
            "The API has nothing else by that name; the guild you already follow is above.",
            "L'API n'a rien d'autre sous ce nom ; la guilde que vous suivez déjà est au-dessus.",
            "Die API hat nichts weiter unter diesem Namen; die bereits gefolgte Gilde steht oben.",
        ],
    ),
];
