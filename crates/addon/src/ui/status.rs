//! La línea de estado de la ventana: salud del sondeo, partida, equipo y origen del empate.
//!
//! Sacado de `ui/mod.rs` porque ese fichero llegó a las 803 líneas con el arreglo de los nombres de
//! la sesión (límite duro: 800, AGENTS §2.8).

use gw2lurker_core::model::{MapRole, TeamColor};

use crate::i18n;

use super::{map, Tabbed, WIDGET};

/// Minutos con la copia de la partida idéntica (puntos incluidos, `matchup::digest`) a partir de
/// los cuales se avisa (RF-43). En MvM el marcador cambia cada pocos segundos; 5 min sin ningún
/// cambio es la API de ArenaNet parada (petición del jugador, 2026-11-02).
const FROZEN_AFTER_MIN: i64 = 5;

pub(super) fn draw_health(
    ui: &nexus::imgui::Ui,
    config: &mut crate::config::Config,
    data: &Tabbed,
    team_id: Option<u32>,
) {
    let state = crate::context::snapshot();
    let (
        health,
        match_id,
        updated,
        selection,
        last_claim_at,
        candidates,
        played_team,
        auto_detected,
        payload_since,
    ) = {
        let board = crate::worker::dashboard();
        (
            board.health.clone(),
            board.match_id.clone(),
            board.updated_at,
            board.selection.clone(),
            board.last_claim_at,
            board.candidates.clone(),
            board.played_team,
            board.auto_detected,
            board.payload_since,
        )
    };
    let language = i18n::effective().code();

    // El jugador reconoce el mapa por su nombre, no por el id de Mumble: el id solo se enseña
    // cuando no hay rol, que es cuando de verdad hace falta para diagnosticar (informe de
    // 2026-10-31, «no sé qué es mapa 95»).
    let role = MapRole::from_mumble(state.map_id, state.map_type);
    if role == MapRole::Unknown {
        let map_id = state.map_id.to_string();
        let label = if gw2lurker_core::gate::is_wvw_map_type(state.map_type) {
            "WvW map outside the matchup (map {0})"
        } else {
            "Outside WvW (map {0})"
        };
        ui.text_disabled(i18n::format(label, &[&map_id]));
    } else {
        let map_label = data
            .maps
            .iter()
            .find(|map| map.id == state.map_id)
            .map(|map| map::map_label_for_map(map, role))
            .unwrap_or_else(|| map::map_label(role).to_owned());
        ui.text_disabled(i18n::format("You are in {0}", &[&map_label]));
    }
    match team_id.map(|id| crate::team::display_name(config, language, id)) {
        Some(label) => {
            if auto_detected {
                // Lo dice el juego, no los ajustes: así se ve de un vistazo que el addon está
                // mirando la partida en la que estás (informe de 2026-10-31).
                ui.text(i18n::format("Team {0} (from the game)", &[&label]));
            } else {
                ui.text(i18n::format("Team {0}", &[&label]));
            }
        }
        None => ui.text(i18n::text("No team configured yet")),
    }
    // Nada de «1-1»: eso es un código interno del API. Lo que el jugador reconoce es **quién
    // juega** en su partida, con sus colores. El id sigue en el log, que es donde sirve
    // (insistencia del jugador, 2026-10-31: «soy humano, nadie dice 1-1»).
    if team_id.is_some() {
        if let Some(id) = &match_id {
            let region = match region_of_matchup(id) {
                "NA" => i18n::text("North America"),
                "EU" => i18n::text("Europe"),
                _ => i18n::text("Unknown region"),
            };
            ui.text_disabled(i18n::format("Match in {0}", &[region]));
            let teams: Vec<String> = [TeamColor::Green, TeamColor::Blue, TeamColor::Red]
                .into_iter()
                .filter_map(|colour| {
                    let candidate = candidates
                        .iter()
                        .find(|c| c.match_id == *id && c.color == colour)?;
                    let name = crate::team::display_name(config, language, candidate.team_id);
                    Some(format!("{} · {name}", colour_label(colour)))
                })
                .collect();
            if !teams.is_empty() {
                ui.text_wrapped(teams.join("   "));
            }
        }
        // Si el juego dice que juegas con otro equipo, decirlo: es la causa de todos los «no salen
        // avisos» hasta ahora (informe de 2026-10-31).
        if let (Some(played), Some(configured)) = (played_team, team_id) {
            if played != configured {
                let played_name = crate::team::display_name(config, language, played);
                let configured_name = crate::team::display_name(config, language, configured);
                let _tint =
                    ui.push_style_color(nexus::imgui::StyleColor::Text, [1.0, 0.6, 0.2, 1.0]);
                ui.text_wrapped(i18n::format(
                "The game says you play with {0}, but the addon watches {1}: change the team in Settings.",
                &[&played_name, &configured_name],
            ));
            }
        }
        // A saved team that no longer plays leaves the addon watching a matchup that is not the
        // player's: saying it out loud is the difference between "no funciona" and "elige equipo".
        if let gw2lurker_core::select::Selection::Stale { previous } = selection {
            let _tint = ui.push_style_color(nexus::imgui::StyleColor::Text, [1.0, 0.75, 0.3, 1.0]);
            ui.text_wrapped(i18n::format(
                "The saved team ({0}) is not playing right now: pick it again in Settings.",
                &[&previous.to_string()],
            ));
        }
        let summary = match &health {
            crate::worker::Health::Idle => i18n::text("No data yet").to_owned(),
            crate::worker::Health::Ok { .. } => {
                i18n::format("{0} claimed objectives", &[&data.claims.len().to_string()])
            }
            crate::worker::Health::Degraded { error, .. } => {
                i18n::format("Error: {0}", &[&i18n::format_api_error(error)])
            }
        };
        let when = updated.map(crate::clock::clock_seconds).unwrap_or_default();
        if when.is_empty() {
            ui.text_disabled(summary);
        } else {
            ui.text_disabled(format!(
                "{summary} · {}",
                i18n::format("Last update {0}", &[&when])
            ));
        }
        // Cuándo vio el juego el último reclamo de esta partida: si eso es viejo, el addon no está
        // parado, es que aquí no ha reclamado nadie (informe de 2026-10-31).
        if let Some(at) = last_claim_at {
            let minutes = chrono::Utc::now()
                .signed_duration_since(at)
                .num_minutes()
                .max(0);
            ui.text_disabled(i18n::format(
                "Newest claim {0} min ago",
                &[&minutes.to_string()],
            ));
        }
        // La partida puede quedarse congelada en la API de ArenaNet mientras el sondeo sigue vivo:
        // el 2026-11-01 la copia de `1-1` no cambió en 20 minutos (puntos incluidos) y el jugador lo
        // leyó como «ha dejado de funcionar». Se dice, en naranja, con los minutos y el intervalo.
        let polling = crate::context::gate();
        if polling.show_ui && polling.poll {
            if let Some(since) = payload_since {
                let minutes = chrono::Utc::now()
                    .signed_duration_since(since)
                    .num_minutes()
                    .max(0);
                if minutes >= FROZEN_AFTER_MIN {
                    // Del `config` que ya tiene el candado, **nunca** de `crate::runtime()`: pedirlo
                    // aquí volvía a bloquear el mismo mutex no reentrante y congelaba el juego
                    // (crash del jugador, 2026-11-01 00:40; es el mismo fallo del 2026-10-31).
                    let secs = config.poll_secs.to_string();
                    let _tint =
                        ui.push_style_color(nexus::imgui::StyleColor::Text, [1.0, 0.75, 0.3, 1.0]);
                    ui.text_wrapped(i18n::format(
                    "ArenaNet's API has not published any change to this match for {0} min; the addon keeps polling every {1} s.",
                    &[&minutes.to_string(), &secs],
                ));
                }
            }
        }
    }

    // Click-through has no visual trace of its own: without this line the only symptom is that
    // nothing answers the mouse, which is indistinguishable from a frozen addon (bug of
    // 2026-10-31).
    if config.window(WIDGET).click_through {
        let _tint = ui.push_style_color(nexus::imgui::StyleColor::Text, [1.0, 0.75, 0.3, 1.0]);
        ui.text_wrapped(i18n::format(
            "Click-through is on: right-click here to use the mouse again (Nexus has {0} for the same).",
            &[crate::keybinds::shortcut().as_str()],
        ));
    }

    // RF-50: Nexus deja la DLL nueva en disco y pide reiniciar; mientras tanto hay que decirlo, o
    // el jugador sigue viendo la versión vieja sin saber por qué.
    if let Some(path) = crate::stale_dll() {
        let _tint = ui.push_style_color(nexus::imgui::StyleColor::Text, [1.0, 0.75, 0.3, 1.0]);
        ui.text_wrapped(i18n::format(
            "Updated to a newer version (running {0}): restart the game to apply it.",
            &[crate::loaded_version()],
        ));
        if ui.is_item_hovered() {
            ui.tooltip_text(path);
        }
    }
}

/// Nombre del color del equipo, el mismo que usa el panel de WvW del juego.
pub(super) fn colour_label(colour: TeamColor) -> &'static str {
    match colour {
        TeamColor::Red => i18n::text("Red"),
        TeamColor::Green => i18n::text("Green"),
        TeamColor::Blue => i18n::text("Blue"),
    }
}

/// Region of a matchup id: `1-x` is North America and `2-x` is Europe.
///
/// It is in the widget on purpose: a team of the wrong region makes the addon watch a matchup
/// that has nothing to do with the player, and then no alert ever fires (bug of 2026-10-31).
/// The **id itself is never shown**: `1-1` means nothing to a player, so only the region is.
pub(super) fn region_of(match_id: &str) -> Option<gw2lurker_core::model::Region> {
    use gw2lurker_core::model::Region;
    match match_id.split_once('-').map(|(region, _)| region) {
        Some("1") => Some(Region::Na),
        Some("2") => Some(Region::Eu),
        _ => None,
    }
}

fn region_of_matchup(match_id: &str) -> &'static str {
    match region_of(match_id) {
        Some(gw2lurker_core::model::Region::Na) => "NA",
        Some(gw2lurker_core::model::Region::Eu) => "EU",
        None => "?",
    }
}
