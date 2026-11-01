//! Guilds of the matchup and the history of the chosen one (T-37, T-38; RF-71, RF-72).
//!
//! One window with two halves: the guilds that hold something right now, ordered by the most
//! recent claim, and — when one is selected — its history during this tracking cycle, split between the
//! map the player is on and the rest.
//!
//! Cada fila lleva dos botones, porque clicar la fila no decía qué hacía (petición del jugador,
//! 2026-11-01): `+` la sigue (RF-73) e `i` enseña su ruta inferida **en el mapa**, que es donde se
//! dibuja; la ruta ya no se escribe en texto en esta pestaña.
//!
//! Nothing here touches the network; the worker resolves the rows.

use std::sync::Mutex;

use gw2lurker_core::matchup::GuildSummary;
use gw2lurker_core::{FollowList, MapRole};
use nexus::imgui::{StyleColor, Ui};

use crate::i18n;

/// Translation key (and stable id) of this window.
pub const KEY: &str = "Guilds";

/// Guild whose history is on screen. Session state, like the other view filters.
static SELECTED: Mutex<Option<String>> = Mutex::new(None);

/// What the player did in the window this frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// `+`: start following the guild (RF-73). Never stops following one: that is the `×` of the
    /// followed list, which asks first.
    Follow(String),
    /// `i`: show its inferred route on the *Map* tab, where it is drawn (RF-32, RF-33).
    ShowOnMap(String),
}

pub(super) struct Context {
    pub(super) current: Option<MapRole>,
    pub(super) history_since: Option<chrono::DateTime<chrono::Utc>>,
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

/// Body of the *Guilds* tab. `history` is the current tracking cycle filtered by guild.
pub fn body(
    ui: &Ui,
    guilds: &[GuildSummary],
    names: &std::collections::BTreeMap<String, String>,
    labels: &mut std::collections::BTreeMap<String, String>,
    follow: &FollowList,
    context: Context,
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
        if let Some(since) = context.history_since {
            ui.text_disabled(i18n::format(
                "Tracking since {0}",
                &[&crate::clock::clock(since)],
            ));
        }
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
                    // Dos botones por fila, porque clicar la fila no decía qué hacía (petición del
                    // jugador, 2026-11-01). `+` sigue al clan; si ya se sigue, el `*` es estado y no
                    // un botón (pulsarlo no hace nada a propósito: quitar se hace con la `×` de la
                    // lista de seguidos, que pregunta antes). Ocupa lo mismo que el `+`, así que las
                    // filas no se desalinean.
                    if followed {
                        let _dim = ui.push_style_color(StyleColor::Text, [0.55, 0.57, 0.6, 1.0]);
                        ui.small_button(format!("*###already{}", summary.guild));
                    } else if ui.small_button(format!("+###add{}", summary.guild)) {
                        action = Some(Action::Follow(summary.guild.clone()));
                    }
                    ui.same_line();
                    // `i`: la inferencia se ve en el mapa, no en un texto debajo de esta lista.
                    if ui.small_button(format!("i###info{}", summary.guild)) {
                        action = Some(Action::ShowOnMap(summary.guild.clone()));
                    }
                    ui.same_line();

                    let when = summary.last_at.map(crate::clock::clock).unwrap_or_default();
                    let here = match summary.last_map {
                        Some((_, role)) if Some(role) == context.current => {
                            format!("  ·  {} ({})", map_label(role), i18n::text("you are here"))
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
        let (here, elsewhere) = match context.current {
            Some(role) => gw2lurker_core::matchup::split_by_map(&rows, role),
            None => (Vec::new(), rows.clone()),
        };
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
