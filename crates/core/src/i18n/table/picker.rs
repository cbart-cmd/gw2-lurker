//! Team picker window (RF-03a, RF-03j).

use super::COLUMNS;

/// The picker window and why it opens by itself, with their complete translations.
pub const TEXT: &[(&str, [&str; COLUMNS])] = &[
    (
        "Choose your team",
        [
            "Elige tu equipo",
            "Choose your team",
            "Choisissez votre équipe",
            "Wähle dein Team",
        ],
    ),
    (
        "Change team",
        [
            "Cambiar de equipo",
            "Change team",
            "Changer d'équipe",
            "Team wechseln",
        ],
    ),
    (
        "Confirm team",
        [
            "Confirmar equipo",
            "Confirm team",
            "Confirmer l'équipe",
            "Team bestätigen",
        ],
    ),
    (
        "Your matchup",
        [
            "Tu emparejamiento",
            "Your matchup",
            "Votre affrontement",
            "Deine Begegnung",
        ],
    ),
    (
        "Pick your team to see what happens in your matchup.",
        [
            "Elige tu equipo para ver lo que pasa en tu emparejamiento.",
            "Pick your team to see what happens in your matchup.",
            "Choisissez votre équipe pour voir ce qui se passe dans votre affrontement.",
            "Wähle dein Team, um zu sehen, was in deiner Begegnung passiert.",
        ],
    ),
    (
        "Your saved team is not playing right now: pick it again.",
        [
            "Tu equipo guardado no está jugando ahora: vuelve a elegirlo.",
            "Your saved team is not playing right now: pick it again.",
            "Votre équipe enregistrée ne joue pas en ce moment : choisissez-la à nouveau.",
            "Dein gespeichertes Team spielt gerade nicht: wähle es erneut.",
        ],
    ),
    (
        "The matchup has changed: confirm your team or pick another.",
        [
            "El emparejamiento ha cambiado: confirma tu equipo o elige otro.",
            "The matchup has changed: confirm your team or pick another.",
            "L'affrontement a changé : confirmez votre équipe ou choisissez-en une autre.",
            "Die Begegnung hat sich geändert: bestätige dein Team oder wähle ein anderes.",
        ],
    ),
];
