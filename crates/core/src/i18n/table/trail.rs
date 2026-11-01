//! B - Estela messages (RF-32, RF-33, RF-36, RF-44).

use super::COLUMNS;

/// Route controls and their complete translations.
pub const TEXT: &[(&str, [&str; COLUMNS])] = &[
    ("No claims recorded for this guild yet.", [
        "Todavía no hay claims registrados de este clan.",
        "No claims recorded for this guild yet.",
        "Aucune revendication enregistrée pour cette guilde.",
        "Noch keine Ansprüche dieser Gilde erfasst.",
    ]),
    ("Trail window", ["Ventana de la estela", "Trail window", "Période de la trace", "Zeitraum der Spur"]),
    ("Route step", ["Paso de la ruta", "Route step", "Étape du parcours", "Routenschritt"]),
    ("All available history", ["Todo el historial disponible", "All available history", "Tout l'historique disponible", "Gesamter verfügbarer Verlauf"]),
    ("Last 15 minutes", ["Últimos 15 minutos", "Last 15 minutes", "15 dernières minutes", "Letzte 15 Minuten"]),
    ("Unknown time", ["Hora desconocida", "Unknown time", "Heure inconnue", "Unbekannte Zeit"]),
    ("Step {0} of {1}: {2} - {3}", [
        "Paso {0} de {1}: {2} - {3}", "Step {0} of {1}: {2} - {3}",
        "Étape {0} sur {1} : {2} - {3}", "Schritt {0} von {1}: {2} - {3}",
    ]),
    ("{0} since the previous claim", [
        "{0} desde el claim anterior", "{0} since the previous claim",
        "{0} depuis la revendication précédente", "{0} seit dem vorherigen Anspruch",
    ]),
    ("{0} until the next claim", [
        "{0} hasta el siguiente claim", "{0} until the next claim",
        "{0} jusqu'à la prochaine revendication", "{0} bis zum nächsten Anspruch",
    ]),
    ("Polling gap between these claims", [
        "Hueco de sondeo entre estos claims", "Polling gap between these claims",
        "Interruption des relevés entre ces revendications", "Abfragelücke zwischen diesen Ansprüchen",
    ]),
    ("No claims recorded for {0}", [
        "Sin claims registrados durante {0}", "No claims recorded for {0}",
        "Aucune revendication enregistrée pendant {0}", "Keine Ansprüche erfasst seit {0}",
    ]),
    ("Lower lane: unknown claim time", [
        "Carril inferior: hora del claim desconocida", "Lower lane: unknown claim time",
        "Ligne inférieure : heure de revendication inconnue", "Untere Spur: Anspruchszeit unbekannt",
    ]),
    ("Recent: thicker trail. Thin rings: time until the next claim, not distance.", [
        "Más reciente: estela más gruesa. Anillos finos: tiempo hasta el siguiente claim, no distancia.",
        "Recent: thicker trail. Thin rings: time until the next claim, not distance.",
        "Plus récent : trace plus épaisse. Anneaux fins : délai avant la prochaine revendication, pas la distance.",
        "Aktuell: dickere Spur. Dünne Ringe: Zeit bis zum nächsten Anspruch, keine Entfernung.",
    ]),
    ("Route steps: {0}", [
        "Pasos de la ruta: {0}", "Route steps: {0}", "Étapes du parcours : {0}", "Routenschritte: {0}",
    ]),
];
