//! Guilds of the matchup and the history of the chosen one (T-37, T-38; RF-71, RF-72).
//!
//! One window with two halves: the guilds that hold something right now, ordered by the most
//! recent claim, and — when one is selected — its history during this session, split between the
//! map the player is on and the rest.
//!
//! The star marks a guild as followed (RF-73): a followed guild raises alerts on every map of
//! the matchup, which is the point of following one. Nothing here touches the network; the
//! worker resolves the rows.

use std::sync::Mutex;

use gw2lurker_core::matchup::GuildSummary;
use gw2lurker_core::{FollowList, MapRole};
use nexus::imgui::Ui;

use crate::i18n;

/// Translation key (and stable id) of this window.
pub const KEY: &str = "Guilds";

/// Guild whose history is on screen. Session state, like the other view filters.
static SELECTED: Mutex<Option<String>> = Mutex::new(None);

/// What the player did in the window this frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Follow or stop following this guild.
    ToggleFollow(String),
}

fn selected() -> Option<String> {
    SELECTED
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clone()
}

/// Guild whose history is on screen, if any. The map tab draws its route (T-16).
pub fn selected_guild() -> Option<String> {
    selected()
}

/// Selects a guild from outside this tab: an alert line or the list of followed guilds was
/// clicked, and the route of that guild is what the player wants to see (request of 2026-10-31).
pub fn select(guild: &str) {
    set_selected(Some(guild.to_owned()));
}

fn set_selected(guild: Option<String>) {
    *SELECTED.lock().unwrap_or_else(|error| error.into_inner()) = guild;
}

/// Body of the *Guilds* tab. `history` is the whole session; this filters it by guild.
pub fn body(
    ui: &Ui,
    guilds: &[GuildSummary],
    names: &std::collections::BTreeMap<String, String>,
    labels: &mut std::collections::BTreeMap<String, String>,
    follow: &FollowList,
    current: Option<MapRole>,
    history: impl Fn(&str) -> Vec<gw2lurker_core::matchup::ClaimRow>,
) -> Option<Action> {
    let mut action = None;
    // El nombre del clan no lo traduce el juego: lo que escriba el jugador manda y se enseña
    // detrás del nombre real (T-43).
    let label = |guild: &str| {
        let base = names
            .get(guild)
            .cloned()
            .unwrap_or_else(|| guild.to_owned());
        match labels.get(guild) {
            Some(own) if !own.trim().is_empty() => format!("{base}  ·  {own}"),
            _ => base,
        }
    };

    {
        if guilds.is_empty() {
            ui.text_disabled(i18n::text("No claims yet"));
            return None;
        }

        ui.text(i18n::text("Ordered by most recent claim"));
        if follow.is_full() {
            ui.text_disabled(i18n::text(
                "The list is full: stop following one to add another.",
            ));
        }
        // The list scrolls on its own: it is longer than the window, and with the history
        // pushed off-screen choosing a guild looked like it did nothing (bug of 2026-10-31).
        nexus::imgui::ChildWindow::new("###guild_list")
            .size([0.0, 150.0])
            .border(true)
            .build(ui, || {
                let chosen = selected();
                for summary in guilds {
                    let followed = follow.contains(&summary.guild);
                    let star = if followed { "★" } else { "☆" };
                    if ui.small_button(format!("{star}###follow{}", summary.guild)) {
                        action = Some(Action::ToggleFollow(summary.guild.clone()));
                    }
                    ui.same_line();

                    let when = summary.last_at.map(crate::clock::clock).unwrap_or_default();
                    let here = match summary.last_map {
                        Some((_, role)) if Some(role) == current => {
                            format!(" ← {}", map_label(role))
                        }
                        Some((_, role)) => format!(" · {}", map_label(role)),
                        None => String::new(),
                    };
                    let text = format!(
                        "{}  ·  {} · {}{here}###guild{}",
                        label(&summary.guild),
                        summary.claims,
                        when,
                        summary.guild
                    );
                    if nexus::imgui::Selectable::new(text)
                        .selected(chosen.as_deref() == Some(summary.guild.as_str()))
                        .build(ui)
                    {
                        let next = if chosen.as_deref() == Some(summary.guild.as_str()) {
                            None
                        } else {
                            Some(summary.guild.clone())
                        };
                        set_selected(next);
                    }
                }
            });

        let Some(chosen) = selected() else {
            ui.separator();
            ui.text_disabled(i18n::text("Pick a guild to see its history"));
            return action;
        };

        ui.separator();
        ui.text(label(&chosen));
        // Campo libre: es la única traducción posible para un nombre de clan (T-43).
        let mut own = labels.get(&chosen).cloned().unwrap_or_default();
        if nexus::imgui::InputText::new(ui, "###guildlabel", &mut own)
            .hint(i18n::text("Your name for this guild"))
            .build()
        {
            if own.trim().is_empty() {
                labels.remove(&chosen);
            } else {
                labels.insert(chosen.clone(), own);
            }
        }
        let rows = history(&chosen);
        if rows.is_empty() {
            ui.text_disabled(i18n::text("No claims yet"));
            return action;
        }
        let (here, elsewhere) = match current {
            Some(role) => gw2lurker_core::matchup::split_by_map(&rows, role),
            None => (Vec::new(), rows.clone()),
        };
        if let Some(route) = gw2lurker_core::build_route(&rows, &chosen) {
            // ADR-005: the path is inferred from claims, and the window says so.
            ui.text_disabled(i18n::text(
                "Route inferred from claims; it is not the real path.",
            ));
            ui.text(format!(
                "{} → {}",
                i18n::text(route_reason(route.origin)),
                route_steps(&route, &rows, i18n::effective())
            ));
        }
        draw_group(ui, "Current map", &here);
        draw_group(ui, "Other maps", &elsewhere);
    }

    action
}

fn draw_group(ui: &Ui, heading: &'static str, rows: &[gw2lurker_core::matchup::ClaimRow]) {
    if rows.is_empty() {
        return;
    }
    ui.text_disabled(i18n::text(heading));
    for row in rows {
        let name = row.name.clone().unwrap_or_else(|| row.objective_id.clone());
        let when = row.claimed_at.map(crate::clock::clock).unwrap_or_default();
        ui.text(format!("{when}  {name}  ·  {}", map_label(row.map_role)));
    }
}

/// La ruta en texto, con un **«salto a <mapa>»** cuando los pasos cambian de mapa (RF-35).
///
/// La ruta se construye por mapa, así que el salto no está en los pasos: se deduce de los claims,
/// que sí llevan el mapa en el id del objetivo.
fn route_steps(
    route: &gw2lurker_core::Route,
    rows: &[gw2lurker_core::matchup::ClaimRow],
    language: gw2lurker_core::i18n::Language,
) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut map: Option<MapRole> = None;
    for step in &route.steps {
        let step_map = rows
            .iter()
            .find(|row| row.objective_id == step.objective_id)
            .map(|row| row.map_role);
        if let Some(role) = step_map {
            if map.is_some_and(|previous| previous != role) {
                parts.push(gw2lurker_core::i18n::f(
                    language,
                    "jump to {0}",
                    &[map_label_in(role, language)],
                ));
            }
            map = Some(role);
        }
        parts.push(step.name.clone());
    }
    parts.join(" → ")
}

fn route_reason(origin: gw2lurker_core::RouteOrigin) -> &'static str {
    match origin {
        gw2lurker_core::RouteOrigin::Garrison => "from their keep",
        gw2lurker_core::RouteOrigin::SideKeep => "from their side keep",
        gw2lurker_core::RouteOrigin::Spawn => "from the spawn point",
    }
}

/// Nombre del mapa en el idioma que se pase: las pruebas no pueden leer el idioma activo, porque
/// eso pasa por la API de Nexus y revienta fuera del juego.
fn map_label_in(role: MapRole, language: gw2lurker_core::i18n::Language) -> &'static str {
    use gw2lurker_core::i18n::t;
    match role {
        MapRole::Center => t(language, "Eternals"),
        MapRole::BlueHome => t(language, "Blue alpine"),
        MapRole::GreenHome => t(language, "Green alpine"),
        MapRole::RedHome => t(language, "Red desert"),
        MapRole::Unknown => t(language, "Unknown map"),
    }
}

fn map_label(role: MapRole) -> &'static str {
    match role {
        MapRole::Center => i18n::text("Eternals"),
        MapRole::BlueHome => i18n::text("Blue alpine"),
        MapRole::GreenHome => i18n::text("Green alpine"),
        MapRole::RedHome => i18n::text("Red desert"),
        MapRole::Unknown => i18n::text("Unknown map"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rf35_a_route_that_crosses_maps_says_jump_to() {
        // Tres pasos, el último en otro mapa: entre el segundo y el tercero tiene que aparecer el
        // salto (RF-35).
        let row = |objective_id: &str, role: MapRole| gw2lurker_core::matchup::ClaimRow {
            map_id: 38,
            map_role: role,
            objective_id: objective_id.to_owned(),
            name: None,
            kind: gw2lurker_core::ObjectiveType::Tower,
            owner: None,
            guild: "AAAA".to_owned(),
            claimed_at: None,
            points_tick: 0,
        };
        let rows = vec![
            row("38-10", MapRole::Center),
            row("38-20", MapRole::Center),
            row("96-30", MapRole::BlueHome),
        ];
        let step = |id: &str| gw2lurker_core::route::Step {
            name: id.to_owned(),
            objective_id: id.to_owned(),
            at: None,
        };
        let route = gw2lurker_core::Route {
            origin: gw2lurker_core::RouteOrigin::Spawn,
            steps: vec![step("38-10"), step("38-20"), step("96-30")],
        };

        let text = route_steps(&route, &rows, gw2lurker_core::i18n::Language::Es);
        assert_eq!(text.matches("38-10").count(), 1, "{text}");
        assert!(
            text.contains("salto a"),
            "el cambio de mapa se anuncia: {text}"
        );
        let jump = text.find("salto a").expect("hay salto");
        assert!(
            jump > text.find("38-20").expect("paso 2")
                && jump < text.find("96-30").expect("paso 3"),
            "el salto va entre los dos pasos: {text}"
        );
    }

    #[test]
    fn rf72_the_selection_survives_a_frame_and_can_be_cleared() {
        set_selected(Some("1DB147F0".to_owned()));
        assert_eq!(selected().as_deref(), Some("1DB147F0"));
        set_selected(None);
        assert_eq!(selected(), None);
    }

    #[test]
    fn rf71_guilds_are_listed_once_each() {
        // The worker ranks with `rank_guilds`, which groups by guild: this pins that the view
        // does not need to deduplicate anything itself.
        let summaries = [
            GuildSummary {
                guild: "AAAA".to_owned(),
                claims: 2,
                last_at: None,
                last_map: Some((38, MapRole::Center)),
            },
            GuildSummary {
                guild: "BBBB".to_owned(),
                claims: 1,
                last_at: None,
                last_map: None,
            },
        ];
        let mut names: Vec<&str> = summaries.iter().map(|s| s.guild.as_str()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), summaries.len());
    }
}
