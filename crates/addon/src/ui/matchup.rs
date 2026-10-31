//! Matchup claims view (T-36, RF-70): what is claimed right now, grouped by map.
//!
//! Four maps of the matchup, each with its claimed objectives, the guild holding each one and
//! when it was claimed. Filters by objective kind and "only high value" keep the list short
//! enough to read during a fight.
//!
//! It only draws. The worker resolves the claims (`worker::Dashboard::claims`) with names in the
//! player's language, so nothing here touches the network or the objective table.

use gw2lurker_core::matchup::ClaimRow;
use gw2lurker_core::model::{MapRole, ObjectiveType};
use nexus::imgui::{StyleColor, Ui};

use crate::i18n;

/// Translation key (and stable id) of this window.
pub const KEY: &str = "Matchup claims";

/// Filters the player can toggle; session state, like the team narrowing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Filters {
    /// Only keeps, castles and fully upgraded objectives.
    pub high_value_only: bool,
    pub camps: bool,
    pub towers: bool,
    pub keeps: bool,
    /// Only the guilds the player follows, whatever the map (RF-73).
    pub followed_only: bool,
}

impl Filters {
    /// All kinds visible at the start; the boxes exist to *remove* noise, not to hide by default.
    pub fn all_kinds() -> Self {
        Self {
            high_value_only: false,
            camps: true,
            towers: true,
            keeps: true,
            followed_only: false,
        }
    }

    /// Whether a row survives the filters. `follow` is only consulted when the box is ticked.
    pub fn accepts(&self, row: &ClaimRow, follow: &gw2lurker_core::FollowList) -> bool {
        if self.followed_only && !follow.contains(&row.guild) {
            return false;
        }
        if self.high_value_only && !row.is_high_value() {
            return false;
        }
        match row.kind {
            ObjectiveType::Camp => self.camps,
            ObjectiveType::Tower => self.towers,
            ObjectiveType::Keep | ObjectiveType::Castle => self.keeps,
            _ => true,
        }
    }
}

/// Rows of one map, in the order the API gave them.
pub fn rows_of<'a>(
    rows: &'a [ClaimRow],
    role: MapRole,
    filters: &Filters,
    follow: &gw2lurker_core::FollowList,
) -> Vec<&'a ClaimRow> {
    rows.iter()
        .filter(|row| row.map_role == role && filters.accepts(row, follow))
        .collect()
}

/// Body of the *Map* tab. `state` is gone with the window: it lives in the widget now (T-42).
pub fn body(
    ui: &Ui,
    filters: &mut Filters,
    rows: &[ClaimRow],
    current: Option<MapRole>,
    follow: &gw2lurker_core::FollowList,
    names: &std::collections::BTreeMap<String, String>,
) {
    draw_filters(ui, filters);
    ui.separator();

    if rows.is_empty() {
        ui.text_disabled(i18n::text("No claims yet"));
        return;
    }

    for role in [
        MapRole::Center,
        MapRole::BlueHome,
        MapRole::GreenHome,
        MapRole::RedHome,
    ] {
        let of_map = rows_of(rows, role, filters, follow);
        if of_map.is_empty() {
            continue;
        }
        let here = if current == Some(role) {
            format!(" ({})", i18n::text("you are here"))
        } else {
            String::new()
        };
        let header = format!(
            "{} · {}{here}###map{:?}",
            map_label(role),
            of_map.len(),
            role
        );
        if nexus::imgui::CollapsingHeader::new(header)
            .default_open(true)
            .build(ui)
        {
            for row in of_map {
                draw_row(ui, row, names);
            }
        }
    }
    let shown = rows.iter().any(|row| filters.accepts(row, follow));
    if !shown {
        ui.text_disabled(i18n::text("Nothing matches the filters"));
    }
}

fn draw_filters(ui: &Ui, filters: &mut Filters) {
    ui.checkbox(
        i18n::widget("High value only"),
        &mut filters.high_value_only,
    );
    ui.same_line();
    ui.checkbox(i18n::widget("Camps"), &mut filters.camps);
    ui.same_line();
    ui.checkbox(i18n::widget("Towers"), &mut filters.towers);
    ui.same_line();
    ui.checkbox(i18n::widget("Keeps"), &mut filters.keeps);
    ui.same_line();
    ui.checkbox(i18n::widget("Followed guilds"), &mut filters.followed_only);
}

fn draw_row(ui: &Ui, row: &ClaimRow, names: &std::collections::BTreeMap<String, String>) {
    let name = row.name.clone().unwrap_or_else(|| row.objective_id.clone());
    let guild = names
        .get(&row.guild)
        .cloned()
        .unwrap_or_else(|| row.guild.clone());
    let when = row
        .claimed_at
        .map(|at| at.format("%H:%M").to_string())
        .unwrap_or_default();

    if let Some(colour) = row.owner {
        let _tint = ui.push_style_color(StyleColor::Text, colour_rgba(colour));
        ui.text(format!("{when}  {name}  ·  {guild}"));
    } else {
        ui.text(format!("{when}  {name}  ·  {guild}"));
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

fn colour_rgba(colour: gw2lurker_core::TeamColor) -> [f32; 4] {
    match colour {
        gw2lurker_core::TeamColor::Red => [1.0, 0.45, 0.45, 1.0],
        gw2lurker_core::TeamColor::Green => [0.6, 0.95, 0.6, 1.0],
        gw2lurker_core::TeamColor::Blue => [0.52, 0.75, 1.0, 1.0],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use gw2lurker_core::model::TeamColor;

    fn row(kind: ObjectiveType, role: MapRole) -> ClaimRow {
        ClaimRow {
            map_id: 38,
            map_role: role,
            objective_id: "38-20".to_owned(),
            name: Some("Zanja de Mendon".to_owned()),
            kind,
            owner: Some(TeamColor::Blue),
            guild: "1DB147F0".to_owned(),
            claimed_at: Some(Utc.with_ymd_and_hms(2026, 10, 31, 12, 0, 0).unwrap()),
        }
    }

    /// Nobody followed: the box that filters by followed guilds is off in these tests.
    fn followed() -> gw2lurker_core::FollowList {
        gw2lurker_core::FollowList::new()
    }

    /// Somebody followed: the only guild in these fixtures.
    fn following() -> gw2lurker_core::FollowList {
        let mut list = gw2lurker_core::FollowList::new();
        list.add("1DB147F0");
        list
    }

    #[test]
    fn rf73_the_claims_view_can_show_only_followed_guilds() {
        let rows = vec![
            row(ObjectiveType::Camp, MapRole::Center),
            ClaimRow {
                guild: "SOMEONE-ELSE".to_owned(),
                ..row(ObjectiveType::Tower, MapRole::Center)
            },
        ];
        let filters = Filters {
            followed_only: true,
            ..Filters::all_kinds()
        };

        let shown = rows_of(&rows, MapRole::Center, &filters, &following());
        assert_eq!(shown.len(), 1, "solo el clan seguido");
        assert_eq!(shown[0].guild, "1DB147F0");
        assert!(
            rows_of(&rows, MapRole::Center, &filters, &followed()).is_empty(),
            "sin clanes seguidos no hay nada que enseñar"
        );
    }

    #[test]
    fn rf70_all_kinds_are_visible_by_default() {
        let filters = Filters::all_kinds();
        for kind in [
            ObjectiveType::Camp,
            ObjectiveType::Tower,
            ObjectiveType::Keep,
            ObjectiveType::Castle,
        ] {
            assert!(
                filters.accepts(&row(kind, MapRole::Center), &followed()),
                "{kind:?}"
            );
        }
    }

    #[test]
    fn rf70_kind_filters_remove_rows() {
        let filters = Filters {
            camps: false,
            ..Filters::all_kinds()
        };
        assert!(!filters.accepts(&row(ObjectiveType::Camp, MapRole::Center), &followed()));
        assert!(filters.accepts(&row(ObjectiveType::Tower, MapRole::Center), &followed()));

        let filters = Filters {
            towers: false,
            keeps: false,
            ..Filters::all_kinds()
        };
        assert!(!filters.accepts(&row(ObjectiveType::Tower, MapRole::Center), &followed()));
        assert!(!filters.accepts(&row(ObjectiveType::Castle, MapRole::Center), &followed()));
        assert!(
            filters.accepts(&row(ObjectiveType::Ruins, MapRole::Center), &followed()),
            "ruinas no se filtran: no tienen casilla"
        );
    }

    #[test]
    fn rf70_high_value_hides_camps_but_keeps_castles() {
        let filters = Filters {
            high_value_only: true,
            ..Filters::all_kinds()
        };
        assert!(!filters.accepts(&row(ObjectiveType::Camp, MapRole::Center), &followed()));
        assert!(filters.accepts(&row(ObjectiveType::Keep, MapRole::Center), &followed()));
        assert!(filters.accepts(&row(ObjectiveType::Castle, MapRole::Center), &followed()));
    }

    #[test]
    fn rf70_rows_are_grouped_by_map() {
        let rows = vec![
            row(ObjectiveType::Camp, MapRole::Center),
            row(ObjectiveType::Tower, MapRole::GreenHome),
            row(ObjectiveType::Keep, MapRole::Center),
        ];
        let filters = Filters::all_kinds();
        assert_eq!(
            rows_of(&rows, MapRole::Center, &filters, &followed()).len(),
            2
        );
        assert_eq!(
            rows_of(&rows, MapRole::GreenHome, &filters, &followed()).len(),
            1
        );
        assert!(rows_of(&rows, MapRole::RedHome, &filters, &followed()).is_empty());
    }
}
