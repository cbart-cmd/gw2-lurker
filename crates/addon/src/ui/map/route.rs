//! Native drawing of the cached B - Estela scene (T-52).
//!
//! Curves connect observations visually; they do not represent real paths (ADR-005).

use gw2lurker_core::i18n::t;
use nexus::imgui::{DrawListMut, Ui};

use super::scene::{Scene, Stroke};
use super::timeline::Session;
use super::{i18n, ClaimRow, Frame, MapRole, ObjectiveInfo};

/// Projection box and screen rectangle used by both geometry tests and native drawing.
pub(super) struct RouteCanvas<'a> {
    pub frame: &'a Frame,
    pub origin: [f32; 2],
    pub size: [f32; 2],
}

/// Draws prepared geometry; all expensive grouping/projection happens outside the steady frame.
pub(super) fn draw_route(list: &DrawListMut<'_>, scene: &Scene) {
    let segments = (2048 / (scene.curves.len() + 1)).clamp(2, 16) as u32;
    for stroke in &scene.curves {
        draw_stroke(list, stroke, segments);
    }
    if let Some(stroke) = &scene.origin_link {
        draw_stroke(list, stroke, 16);
    }
    if let Some(point) = scene.origin {
        dashed_ring(list, point, 10.0, [1.0, 0.75, 0.3, 1.0]);
        if !scene.markers.iter().any(|m| m.label.starts_with("0/")) {
            list.add_text([point[0] - 3.0, point[1] - 7.0], [1.0; 4], "0");
        }
    }
    for marker in &scene.markers {
        let colour = if marker.future {
            [0.62, 0.70, 0.77, 0.8]
        } else if marker.outside {
            [0.4, 0.45, 0.5, 0.8]
        } else {
            [1.0, 0.75, 0.3, 1.0]
        };
        if let Some(radius) = marker.interval_radius {
            list.add_circle(marker.point, radius, [1.0, 0.75, 0.3, 0.4])
                .num_segments(24)
                .thickness(1.0)
                .build();
        }
        if !marker.future {
            let fill = if marker.current {
                [1.0, 0.4, 0.2, 1.0]
            } else if marker.latest {
                colour
            } else {
                [0.04, 0.05, 0.06, 0.95]
            };
            list.add_circle(marker.point, 8.5, fill)
                .num_segments(16)
                .filled(true)
                .build();
        }
        if marker.future || marker.late {
            dashed_ring(list, marker.point, 8.5, colour);
        } else {
            list.add_circle(marker.point, 8.5, colour)
                .num_segments(16)
                .thickness(1.5)
                .build();
        }
        if marker.current || marker.latest {
            let tint = if marker.current {
                [1.0, 0.4, 0.2, 1.0]
            } else {
                colour
            };
            list.add_circle(marker.point, 12.0, tint)
                .num_segments(20)
                .thickness(2.0)
                .build();
        }
        let at = [
            marker.point[0] - marker.label.len().min(9) as f32 * 3.0,
            marker.point[1] - 7.0,
        ];
        let text = if !marker.future && (marker.current || marker.latest) {
            [0.03, 0.03, 0.04, 1.0]
        } else {
            colour
        };
        list.add_text(at, text, &marker.label);
        if marker.current || marker.latest {
            list.add_text([at[0] + 0.5, at[1]], text, &marker.label);
        }
    }
}

/// Uses the verified arcdps-imgui 0.8 draw_list.rs Bezier builder; fixed tessellation bounds cost.
fn draw_stroke(list: &DrawListMut<'_>, stroke: &Stroke, segments: u32) {
    let c = &stroke.curve;
    if stroke.dashed {
        let divisions = segments * 2;
        for i in (0..divisions).step_by(2) {
            list.add_line(
                c.point(i as f32 / divisions as f32),
                c.point((i + 1) as f32 / divisions as f32),
                stroke.colour,
            )
            .thickness(stroke.width)
            .build();
        }
    } else {
        let (a, b) = c.cubic();
        list.add_bezier_curve(c.from, a, b, c.to, stroke.colour)
            .num_segments(segments)
            .thickness(stroke.width)
            .build();
    }
    let at = c.point(0.58);
    let before = c.point(0.53);
    let Some((ux, uy)) = heading(before, c.point(0.68)) else {
        return;
    };
    for angle in [0.52_f32, -0.52] {
        let (sin, cos) = angle.sin_cos();
        list.add_line(
            at,
            [
                at[0] - (ux * cos - uy * sin) * 7.0,
                at[1] - (ux * sin + uy * cos) * 7.0,
            ],
            stroke.colour,
        )
        .thickness(stroke.width.min(2.0))
        .build();
    }
}

/// Dashed outline distinguishes future and late observations without relying on colour.
fn dashed_ring(list: &DrawListMut<'_>, point: [f32; 2], radius: f32, colour: [f32; 4]) {
    for i in (0..24).step_by(2) {
        let p = |n: usize| {
            let angle = std::f32::consts::TAU * n as f32 / 24.0;
            [
                point[0] + angle.cos() * radius,
                point[1] + angle.sin() * radius,
            ]
        };
        list.add_line(p(i), p(i + 1), colour).thickness(1.5).build();
    }
}

/// Origin explanation and legend remain beside the map, never presented as observed movement.
pub(super) fn caption(ui: &Ui, state: &Session, names: &[ObjectiveInfo]) {
    if let Some(route) = &state.origin {
        if let Some(point) = &route.origin_point {
            let name = names
                .iter()
                .find(|n| n.id == point.objective_id)
                .map(|n| n.name.as_str())
                .unwrap_or(&point.objective_id);
            let label = format!("{name} ({})", i18n::text(route.origin.reason()));
            ui.text_wrapped(i18n::format("Inferred origin (0): {0}", &[&label]));
            if point.approximate {
                ui.text_wrapped(i18n::text(
                    "Respawn position is approximate: it uses the official map label.",
                ));
            }
        } else {
            ui.text_wrapped(i18n::text("Origin unknown: missing team or coordinates."));
        }
    } else {
        ui.text_wrapped(i18n::text(
            "No claims on this map in the selected time range.",
        ));
    }
    ui.text_wrapped(i18n::text(
        "Recent: thicker trail. Thin rings: time until the next claim, not distance.",
    ));
}

/// Hover metadata of route nodes, retaining all revisit numbers without an oversized tooltip.
pub(super) fn hover(ui: &Ui, state: &Session, canvas: &RouteCanvas<'_>) {
    let mouse = ui.io().mouse_pos;
    if !super::inside(mouse, canvas.origin, canvas.size) {
        return;
    }
    let Some(marker) = state
        .scene
        .markers
        .iter()
        .filter(|m| (m.point[0] - mouse[0]).hypot(m.point[1] - mouse[1]) <= 12.0)
        .min_by(|a, b| {
            let d = |p: [f32; 2]| (p[0] - mouse[0]).hypot(p[1] - mouse[1]);
            d(a.point).total_cmp(&d(b.point))
        })
    else {
        return;
    };
    let mut lines = vec![i18n::format("Route steps: {0}", &[&marker.label])];
    for &i in marker.steps.iter().take(10) {
        let s = &state.trail.steps[i];
        let time = s
            .claim
            .claimed_at
            .map(crate::clock::clock_seconds)
            .unwrap_or_else(|| i18n::text("Unknown time").to_owned());
        let suffix = if s.late { i18n::text("late") } else { "" };
        lines.push(format!("{}: {time} {suffix}", i + 1));
        if let Some(dt) = s.until_next {
            lines.push(i18n::format(
                "{0} until the next claim",
                &[&super::timeline::duration(dt)],
            ));
        }
    }
    ui.tooltip_text(lines.join("\n"));
}

/// Text summary with map transitions, kept compatible with the Guilds navigation tests.
pub(crate) fn summary(
    rows: &[ClaimRow],
    guild: &str,
    language: gw2lurker_core::i18n::Language,
) -> Option<String> {
    let route = gw2lurker_core::build_route(rows, guild)?;
    Some(route_steps(&route, rows, language))
}

/// Summarizes the inferred sequence with an explicit jump between maps.
pub(super) fn route_steps(
    route: &gw2lurker_core::Route<'_>,
    rows: &[ClaimRow],
    language: gw2lurker_core::i18n::Language,
) -> String {
    let mut parts = Vec::new();
    let mut map = None;
    for step in &route.steps {
        let step_map = rows
            .iter()
            .find(|r| r.objective_id.as_str() == step.objective_id.as_ref())
            .map(|r| r.map_role);
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
        parts.push(step.name.to_string());
    }
    parts.join(" -> ")
}

/// Map name in a supplied language, so tests never query Nexus for the active language.
fn map_label_in(role: MapRole, language: gw2lurker_core::i18n::Language) -> &'static str {
    match role {
        MapRole::Center => t(language, "Eternals"),
        MapRole::BlueHome => t(language, "Blue alpine"),
        MapRole::GreenHome => t(language, "Green alpine"),
        MapRole::RedHome => t(language, "Red desert"),
        MapRole::Unknown => t(language, "Unknown map"),
    }
}

/// Direction of a non-degenerate connector; no inferred heading from coincident points.
pub(super) fn heading(from: [f32; 2], to: [f32; 2]) -> Option<(f32, f32)> {
    let dx = to[0] - from[0];
    let dy = to[1] - from[1];
    let length = dx.hypot(dy);
    (length >= 12.0).then(|| (dx / length, dy / length))
}
