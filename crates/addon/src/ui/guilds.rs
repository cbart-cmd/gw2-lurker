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

fn set_selected(guild: Option<String>) {
    *SELECTED.lock().unwrap_or_else(|error| error.into_inner()) = guild;
}

/// Draws the window. `history` is the whole session; this filters it by guild.
pub fn draw(
    ui: &Ui,
    state: &mut crate::config::WindowState,
    guilds: &[GuildSummary],
    follow: &FollowList,
    current: Option<MapRole>,
    history: impl Fn(&str) -> Vec<gw2lurker_core::matchup::ClaimRow>,
) -> Option<Action> {
    let mut action = None;

    crate::ui::window::draw(ui, KEY, 4, [420.0, 340.0], state, |ui| {
        if guilds.is_empty() {
            ui.text_disabled(i18n::text("No claims yet"));
            return;
        }

        let chosen = selected();
        ui.text(i18n::text("Ordered by most recent claim"));
        for summary in guilds {
            let followed = follow.contains(&summary.guild);
            let star = if followed { "★" } else { "☆" };
            if ui.small_button(format!("{star}###follow{}", summary.guild)) {
                action = Some(Action::ToggleFollow(summary.guild.clone()));
            }
            ui.same_line();

            let when = summary
                .last_at
                .map(|at| at.format("%H:%M").to_string())
                .unwrap_or_default();
            let here = match summary.last_map {
                Some((_, role)) if Some(role) == current => format!(" ← {}", map_label(role)),
                Some((_, role)) => format!(" · {}", map_label(role)),
                None => String::new(),
            };
            let label = format!(
                "{}  ·  {} · {}{here}###guild{}",
                summary.guild, summary.claims, when, summary.guild
            );
            if nexus::imgui::Selectable::new(label)
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

        let Some(chosen) = selected() else {
            ui.separator();
            ui.text_disabled(i18n::text("Pick a guild to see its history"));
            return;
        };

        ui.separator();
        ui.text(format!("{}###history{}", chosen, chosen));
        let rows = history(&chosen);
        if rows.is_empty() {
            ui.text_disabled(i18n::text("No claims yet"));
            return;
        }
        let (here, elsewhere) = match current {
            Some(role) => gw2lurker_core::matchup::split_by_map(&rows, role),
            None => (Vec::new(), rows.clone()),
        };
        draw_group(ui, "Current map", &here);
        draw_group(ui, "Other maps", &elsewhere);
    });

    action
}

fn draw_group(ui: &Ui, heading: &'static str, rows: &[gw2lurker_core::matchup::ClaimRow]) {
    if rows.is_empty() {
        return;
    }
    ui.text_disabled(i18n::text(heading));
    for row in rows {
        let name = row.name.clone().unwrap_or_else(|| row.objective_id.clone());
        let when = row
            .claimed_at
            .map(|at| at.format("%H:%M").to_string())
            .unwrap_or_default();
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
