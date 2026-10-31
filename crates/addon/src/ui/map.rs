//! Simplified map of one WvW map (T-14, RF-30).
//!
//! Objectives as coloured shapes on a plain background: circle for a camp, square for a tower,
//! diamond for a keep and triangle for the castle. No textures and no tiles (ADR-002); the
//! projection comes from `continent_rect` of `/v2/maps` (ADR-011) and lives in
//! `gw2lurker_core::project`, which is pure and tested against the real fixtures.
//!
//! Only draws: coordinates, owners and guilds are resolved by the worker.

use gw2lurker_core::matchup::ClaimRow;
use gw2lurker_core::model::{MapRole, ObjectiveInfo, ObjectiveType, TeamColor, WvwMap};
use gw2lurker_core::{project, FollowList, Frame};
use nexus::imgui::{ComboBox, Selectable, StyleColor, Ui};

use crate::i18n;

/// Translation key (and stable id) of this tab.
pub const KEY: &str = "Map";

/// Height of the drawing area.
const MAP_HEIGHT: f32 = 280.0;

/// Grey of an objective nobody holds.
const UNCLAIMED: [f32; 4] = [0.55, 0.57, 0.6, 1.0];

/// Ring drawn around the objectives of a followed guild (RF-31).
const FOLLOWED_RING: [f32; 4] = [1.0, 0.86, 0.4, 1.0];

/// Body of the *Map* tab.
pub fn body(
    ui: &Ui,
    rows: &[ClaimRow],
    names: &[ObjectiveInfo],
    maps: &[WvwMap],
    role: &mut MapRole,
    here: Option<MapRole>,
    follow: &FollowList,
) {
    draw_selector(ui, role, here, maps);

    let Some(map) = maps
        .iter()
        .find(|map| MapRole::from_map_id(map.id) == *role)
    else {
        ui.text_disabled(i18n::text("No map data yet"));
        return;
    };
    let frame = Frame::of(map);
    if !frame.is_usable() {
        ui.text_disabled(i18n::text("No map data yet"));
        return;
    }

    let width = ui.content_region_avail()[0].max(140.0);
    let origin = ui.cursor_screen_pos();
    // The area is reserved with a dummy so the window sizes itself; the drawing itself goes
    // straight to the draw list, which is what imgui wants for shapes.
    ui.dummy([width, MAP_HEIGHT]);

    let list = ui.get_window_draw_list();
    let corner = [origin[0] + width, origin[1] + MAP_HEIGHT];
    list.add_rect(origin, corner, [0.10, 0.11, 0.13, 0.85])
        .filled(true)
        .build();
    list.add_rect(origin, corner, [0.32, 0.34, 0.38, 1.0])
        .thickness(1.0)
        .build();

    let size = [width, MAP_HEIGHT];
    for objective in names.iter().filter(|objective| objective.map_id == map.id) {
        let Some(coord) = objective.coord else {
            continue; // `coord: null` se omite, no se dibuja en (0,0) (RF-30)
        };
        if !is_drawable(objective.kind) {
            continue;
        }
        let Some(point) = project(&frame, [coord[0], coord[1]], origin, size) else {
            continue;
        };
        let row = rows.iter().find(|row| row.objective_id == objective.id);
        draw_objective(
            ui,
            point,
            objective.kind,
            row.and_then(|row| row.owner),
            row.is_none_or(|row| row.is_high_value()),
            row.is_some_and(|row| follow.contains(&row.guild)),
        );
    }
}

fn draw_selector(ui: &Ui, role: &mut MapRole, here: Option<MapRole>, maps: &[WvwMap]) {
    let current = map_label(*role);
    if here.is_some() && *role == MapRole::Unknown {
        *role = here.unwrap_or(MapRole::Center);
    }
    ComboBox::new(i18n::widget("Map"))
        .preview_value(current)
        .build(ui, || {
            for candidate in [
                MapRole::Center,
                MapRole::BlueHome,
                MapRole::GreenHome,
                MapRole::RedHome,
            ] {
                let known = maps
                    .iter()
                    .any(|map| MapRole::from_map_id(map.id) == candidate);
                if !known {
                    continue;
                }
                let label = map_label(candidate);
                let _tint = if here == Some(candidate) {
                    Some(ui.push_style_color(StyleColor::Text, [0.6, 0.9, 1.0, 1.0]))
                } else {
                    None
                };
                if Selectable::new(label)
                    .selected(*role == candidate)
                    .build(ui)
                {
                    *role = candidate;
                }
            }
        });
}

/// Objectives worth drawing: the ones that score (RF-30a hides ruins, mercenaries and spawns).
fn is_drawable(kind: ObjectiveType) -> bool {
    matches!(
        kind,
        ObjectiveType::Camp | ObjectiveType::Tower | ObjectiveType::Keep | ObjectiveType::Castle
    )
}

fn draw_objective(
    ui: &Ui,
    point: [f32; 2],
    kind: ObjectiveType,
    owner: Option<TeamColor>,
    high_value: bool,
    followed: bool,
) {
    let list = ui.get_window_draw_list();
    let colour = owner.map_or(UNCLAIMED, colour_rgba);
    let radius = match kind {
        ObjectiveType::Camp => 4.0,
        ObjectiveType::Tower => 5.0,
        ObjectiveType::Keep => 6.0,
        _ => 7.5,
    };

    match kind {
        ObjectiveType::Camp => {
            list.add_circle(point, radius, colour).filled(true).build();
        }
        ObjectiveType::Tower => {
            let half = radius;
            list.add_rect(
                [point[0] - half, point[1] - half],
                [point[0] + half, point[1] + half],
                colour,
            )
            .filled(true)
            .build();
        }
        ObjectiveType::Keep => {
            // Diamond: two triangles, because that is what the draw list gives us.
            let up = [point[0], point[1] - radius];
            let down = [point[0], point[1] + radius];
            let left = [point[0] - radius, point[1]];
            let right = [point[0] + radius, point[1]];
            list.add_triangle(left, up, right, colour)
                .filled(true)
                .build();
            list.add_triangle(left, down, right, colour)
                .filled(true)
                .build();
        }
        _ => {
            let up = [point[0], point[1] - radius];
            let left = [point[0] - radius, point[1] + radius];
            let right = [point[0] + radius, point[1] + radius];
            list.add_triangle(up, left, right, colour)
                .filled(true)
                .build();
        }
    }

    if high_value {
        // Double outline: a second mark, never colour alone (RF-30b, accessibility).
        list.add_circle(point, radius + 3.0, [1.0, 1.0, 1.0, 0.75])
            .thickness(1.0)
            .build();
    }
    if followed {
        list.add_circle(point, radius + 5.5, FOLLOWED_RING)
            .thickness(1.5)
            .build();
    }
}

fn colour_rgba(colour: TeamColor) -> [f32; 4] {
    match colour {
        TeamColor::Red => [0.92, 0.35, 0.35, 1.0],
        TeamColor::Green => [0.42, 0.82, 0.45, 1.0],
        TeamColor::Blue => [0.42, 0.62, 0.95, 1.0],
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
    fn rf30a_scoring_objectives_are_the_only_ones_drawn() {
        for kind in [
            ObjectiveType::Camp,
            ObjectiveType::Tower,
            ObjectiveType::Keep,
            ObjectiveType::Castle,
        ] {
            assert!(is_drawable(kind), "{kind:?} puntúa");
        }
        for kind in [
            ObjectiveType::Ruins,
            ObjectiveType::Mercenary,
            ObjectiveType::Spawn,
            ObjectiveType::Resource,
            ObjectiveType::Generic,
            ObjectiveType::Unknown,
        ] {
            assert!(!is_drawable(kind), "{kind:?} no se dibuja por defecto");
        }
    }
}
