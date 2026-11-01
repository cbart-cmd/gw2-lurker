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
    // «Camps» y «Towers» ya existen más abajo (el filtro del mapa): se reutilizan aquí.
    (
        "Keeps and garrisons",
        ["Fortalezas y garitas", "Keeps and garrisons", "Fortresses et garnisons", "Festen und Garnisonen"],
    ),
    ("Castles", ["Castillos", "Castles", "Châteaux", "Burgen"]),
    (
        "Only structures a guild can claim raise an alert: ruins and mercenary camps never do.",
        [
            "Solo avisan las estructuras que un clan puede reclamar: las ruinas y los campamentos mercenarios nunca lo hacen.",
            "Only structures a guild can claim raise an alert: ruins and mercenary camps never do.",
            "Seules les structures qu'une guilde peut revendiquer déclenchent une alerte : les ruines et les camps mercenaires jamais.",
            "Nur Strukturen, die eine Gilde beanspruchen kann, lösen einen Hinweis aus: Ruinen und Söldnerlager nie.",
        ],
    ),
    (
        "Guild names index",
        ["Índice de nombres de clan", "Guild names index", "Index des noms de guilde", "Gildennamen-Index"],
    ),
    (
        "{0} guild names ready to search",
        [
            "{0} nombres de clan listos para buscar",
            "{0} guild names ready to search",
            "{0} noms de guilde prêts à chercher",
            "{0} Gildennamen zum Suchen bereit",
        ],
    ),
    (
        "Not downloaded yet: the search only asks the API, which returns one guild at most.",
        [
            "Sin descargar: el buscador solo pregunta al API, que devuelve un clan como mucho.",
            "Not downloaded yet: the search only asks the API, which returns one guild at most.",
            "Pas encore téléchargé : la recherche n'interroge que l'API, qui renvoie une guilde au plus.",
            "Noch nicht geladen: die Suche fragt nur die API, die höchstens eine Gilde liefert.",
        ],
    ),
    (
        "Downloading the index...",
        [
            "Descargando el índice...",
            "Downloading the index...",
            "Téléchargement de l'index...",
            "Index wird geladen...",
        ],
    ),
    (
        "Index download failed: {0}",
        [
            "No se pudo descargar el índice: {0}",
            "Index download failed: {0}",
            "Échec du téléchargement de l'index : {0}",
            "Index-Download fehlgeschlagen: {0}",
        ],
    ),
    ("Index URL", ["URL del índice", "Index URL", "URL de l'index", "Index-URL"]),
    (
        "Where the ZIP with the guild names is published, next to its .md5.",
        [
            "Dónde está publicado el ZIP con los nombres de clan, junto a su .md5.",
            "Where the ZIP with the guild names is published, next to its .md5.",
            "Où le ZIP des noms de guilde est publié, à côté de son .md5.",
            "Wo die ZIP mit den Gildennamen veröffentlicht ist, neben ihrer .md5.",
        ],
    ),
    (
        "Download the index",
        ["Descargar el índice", "Download the index", "Télécharger l'index", "Index herunterladen"],
    ),
    (
        "Reads the published ZIP, checks its MD5 and replaces the index on disk.",
        [
            "Lee el ZIP publicado, comprueba su MD5 y sustituye el índice del disco.",
            "Reads the published ZIP, checks its MD5 and replaces the index on disk.",
            "Lit le ZIP publié, vérifie son MD5 et remplace l'index sur le disque.",
            "Liest die veröffentlichte ZIP, prüft ihre MD5 und ersetzt den Index auf der Festplatte.",
        ],
    ),
    (
        "There is no index of guild names yet: only the API is searched, and it returns one guild at most. Download the index from the settings.",
        [
            "Todavía no hay índice de nombres de clan: solo se busca en el API, que devuelve un clan como mucho. Descarga el índice desde los ajustes.",
            "There is no index of guild names yet: only the API is searched, and it returns one guild at most. Download the index from the settings.",
            "Il n'y a pas encore d'index des noms de guilde : seule l'API est interrogée, qui renvoie une guilde au plus. Télécharge l'index depuis les réglages.",
            "Es gibt noch keinen Gildennamen-Index: es wird nur die API gefragt, die höchstens eine Gilde liefert. Lade den Index in den Einstellungen.",
        ],
    ),
    (
        "Nothing matches in the index ({0} guilds). The API returns one guild at most, and only when the text starts its name.",
        [
            "No hay coincidencias en el índice ({0} clanes). El API devuelve un clan como mucho, y solo cuando el texto empieza su nombre.",
            "Nothing matches in the index ({0} guilds). The API returns one guild at most, and only when the text starts its name.",
            "Aucune correspondance dans l'index ({0} guildes). L'API renvoie une guilde au plus, et seulement quand le texte commence son nom.",
            "Keine Treffer im Index ({0} Gilden). Die API liefert höchstens eine Gilde, und nur wenn der Text ihren Namen beginnt.",
        ],
    ),
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
    ("{0}: {1} -> {2}", ["{0}: {1} -> {2}", "{0}: {1} -> {2}", "{0} : {1} -> {2}", "{0}: {1} -> {2}"]),
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
    (
        "Holding something now",
        [
            "Con algo reclamado ahora",
            "Holding something now",
            "Tient quelque chose maintenant",
            "Hält gerade etwas",
        ],
    ),
    (
        "Only the guilds I follow",
        [
            "Solo los clanes que sigo",
            "Only the guilds I follow",
            "Seulement les guildes que je suis",
            "Nur die Gilden, denen ich folge",
        ],
    ),
    (
        "One switch for «only mine»: the map boxes above stop counting.",
        [
            "Un solo interruptor para «solo lo mío»: las casillas de mapa de arriba dejan de contar.",
            "One switch for «only mine»: the map boxes above stop counting.",
            "Un seul interrupteur pour « seulement les miennes » : les cases de carte ci-dessus ne comptent plus.",
            "Ein Schalter für «nur meine»: die Kartenfelder oben zählen nicht mehr.",
        ],
    ),
    (
        "Click-through is on: right-click here to use the mouse again (Nexus has {0} for the same).",
        [
            "Click-through activo: clic derecho aquí para volver a usar el ratón (en Nexus está {0} para lo mismo).",
            "Click-through is on: right-click here to use the mouse again (Nexus has {0} for the same).",
            "Clics traversants actifs : clic droit ici pour réutiliser la souris (Nexus a {0} pour la même chose).",
            "Klickdurchlässig: Rechtsklick hier, um die Maus wieder zu nutzen (Nexus hat {0} dafür).",
        ],
    ),
    (
        "Click-through is turned on from the right-click menu, on the widget itself.",
        [
            "El click-through se activa con el clic derecho sobre el widget.",
            "Click-through is turned on from the right-click menu, on the widget itself.",
            "Les clics traversants s'activent par clic droit sur le widget.",
            "Klickdurchlässig wird per Rechtsklick auf dem Widget aktiviert.",
        ],
    ),
    (
        "Click-through keeps the mouse for the game. Right-click on the widget to use the mouse again; the Nexus shortcut for {0} does the same.",
        [
            "El click-through deja el ratón al juego. Clic derecho sobre el widget para recuperarlo; el atajo de Nexus para {0} hace lo mismo.",
            "Click-through keeps the mouse for the game. Right-click on the widget to use the mouse again; the Nexus shortcut for {0} does the same.",
            "Les clics traversants laissent la souris au jeu. Clic droit sur le widget pour la récupérer ; le raccourci Nexus pour {0} fait pareil.",
            "Klickdurchlässig lässt die Maus im Spiel. Rechtsklick aufs Widget holt sie zurück; der Nexus-Shortcut für {0} tut dasselbe.",
        ],
    ),
    (
        "Part of the name",
        ["Parte del nombre", "Part of the name", "Une partie du nom", "Ein Teil des Namens"],
    ),
    (
        "Searches the guilds the addon already knows, plus one hit from the API.",
        [
            "Busca entre los clanes que el addon ya conoce, y añade una coincidencia del API.",
            "Searches the guilds the addon already knows, plus one hit from the API.",
            "Cherche parmi les guildes déjà connues, plus un résultat de l'API.",
            "Sucht unter den bereits bekannten Gilden, plus ein Treffer der API.",
        ],
    ),
    (
        "{0} guilds in the cache",
        [
            "{0} clanes en la caché",
            "{0} guilds in the cache",
            "{0} guildes en cache",
            "{0} Gilden im Cache",
        ],
    ),
    (
        "Type part of the name; the list updates as you type.",
        [
            "Escribe parte del nombre; la lista sale mientras escribes.",
            "Type part of the name; the list updates as you type.",
            "Tapez une partie du nom ; la liste se met à jour en direct.",
            "Tippe einen Teil des Namens; die Liste erscheint sofort.",
        ],
    ),
    (
        "Nothing matches in the cache ({0} guilds). The API returns one guild at most, and only when the text starts its name.",
        [
            "Nada coincide en la caché ({0} clanes). El API devuelve como mucho un clan, y solo si el texto empieza su nombre.",
            "Nothing matches in the cache ({0} guilds). The API returns one guild at most, and only when the text starts its name.",
            "Rien ne correspond en cache ({0} guildes). L'API renvoie une guilde au plus, et seulement si le texte commence son nom.",
            "Nichts im Cache ({0} Gilden). Die API liefert höchstens eine Gilde, und nur wenn der Text ihren Namen beginnt.",
        ],
    ),
    (
        "ArenaNet's API has not published any change to this match for {0} min; the addon keeps polling every {1} s.",
        [
            "La API de ArenaNet no publica ningún cambio de esta partida desde hace {0} min; el addon sigue sondeando cada {1} s.",
            "ArenaNet's API has not published any change to this match for {0} min; the addon keeps polling every {1} s.",
            "L'API d'ArenaNet ne publie aucun changement de cette partie depuis {0} min ; l'addon interroge toujours toutes les {1} s.",
            "Die API von ArenaNet meldet seit {0} Min. keine Änderung dieses Matches; das Addon fragt weiter alle {1} s ab.",
        ],
    ),
];
