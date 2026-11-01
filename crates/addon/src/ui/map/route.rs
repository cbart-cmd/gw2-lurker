//! Ruta inferida de un clan y su reproducción (T-16, RF-32, RF-36).
//!
//! Aparte de `map.rs` para que ese fichero quepa en el límite de 800 líneas (AGENTS.md §2.7):
//! el mapa dibuja el tablero y los objetivos; esto, el camino que se deduce de sus reclamos.

use gw2lurker_core::i18n::t;

use super::*;

/// Time window of the route, in minutes; `None` is the whole session (T-16).
///
/// Session state: it is a way of looking at the session, not a setting.
pub(super) fn range_choice(ui: &Ui) -> Option<i64> {
    static CHOICE: std::sync::OnceLock<std::sync::Mutex<u8>> = std::sync::OnceLock::new();
    const OPTIONS: [(u8, Option<i64>, &str); 3] = [
        (1, Some(60), "Last hour"),
        (2, Some(120), "Last 2 hours"),
        (0, None, "All session"),
    ];

    let slot = CHOICE.get_or_init(|| std::sync::Mutex::new(0));
    let mut chosen = *slot.lock().unwrap_or_else(|error| error.into_inner());
    let current = OPTIONS
        .iter()
        .find(|(value, ..)| *value == chosen)
        .map(|(_, _, label)| i18n::text(label))
        .unwrap_or_else(|| i18n::text("All session"));

    ComboBox::new(i18n::widget("Range"))
        .preview_value(current)
        .build(ui, || {
            for (value, _, label) in OPTIONS {
                if Selectable::new(i18n::text(label))
                    .selected(chosen == value)
                    .build(ui)
                {
                    chosen = value;
                }
            }
        });
    *slot.lock().unwrap_or_else(|error| error.into_inner()) = chosen;

    OPTIONS
        .iter()
        .find(|(value, ..)| *value == chosen)
        .and_then(|(_, minutes, _)| *minutes)
}

/// Numbered, directed path of the chosen guild on this map (T-16, RF-32).
///
/// The route is **inferred** from the claims (ADR-005): the caption that says so is drawn by the
/// guilds tab, next to the same data.
/// Where the route is drawn: the projection box plus the rectangle on screen.
pub(super) struct RouteCanvas<'a> {
    pub(super) frame: &'a Frame,
    pub(super) origin: [f32; 2],
    pub(super) size: [f32; 2],
}

#[derive(Debug)]
pub(super) struct RouteNode {
    pub(super) point: [f32; 2],
    pub(super) step: usize,
    pub(super) origin: bool,
    pub(super) at: Option<chrono::DateTime<chrono::Utc>>,
}

impl RouteNode {
    pub(super) fn label(&self) -> String {
        if self.origin && self.step > 0 {
            format!("0/{}", self.step)
        } else {
            self.step.to_string()
        }
    }
}

pub(super) fn route_nodes(
    route: &gw2lurker_core::Route,
    names: &[ObjectiveInfo],
    canvas: &RouteCanvas<'_>,
    played: Option<usize>,
) -> Vec<RouteNode> {
    let coordinates: std::collections::HashMap<_, _> = names
        .iter()
        .filter_map(|name| {
            name.coord
                .map(|coord| (name.id.as_str(), [coord[0], coord[1]]))
        })
        .collect();
    let mut nodes: Vec<_> = route
        .steps
        .iter()
        .enumerate()
        .take(played.unwrap_or(route.steps.len()))
        .filter_map(|(index, step)| {
            let coord = *coordinates.get(step.objective_id.as_ref())?;
            let point = project(canvas.frame, coord, canvas.origin, canvas.size)?;
            Some(RouteNode {
                point,
                step: index + 1,
                origin: false,
                at: step.at,
            })
        })
        .collect();
    let Some(first) = nodes.first() else {
        return nodes;
    };
    let Some(origin) = route.origin_point.as_ref() else {
        return nodes;
    };
    let Some(point) = project(canvas.frame, origin.position, canvas.origin, canvas.size) else {
        return nodes;
    };
    if route.steps[first.step - 1].objective_id.as_ref() == origin.objective_id.as_str() {
        nodes[0].origin = true;
    } else {
        nodes.insert(
            0,
            RouteNode {
                point,
                step: 0,
                origin: true,
                at: None,
            },
        );
    }
    nodes
}

pub(super) fn draw_route(
    list: &nexus::imgui::DrawListMut<'_>,
    canvas: &RouteCanvas<'_>,
    route: &gw2lurker_core::Route,
    names: &[ObjectiveInfo],
    played: Option<usize>,
) {
    let nodes = route_nodes(route, names, canvas, played);
    for pair in nodes.windows(2) {
        if pair[0].step == 0 {
            draw_origin_connector(list, pair[0].point, pair[1].point);
        } else {
            list.add_line(pair[0].point, pair[1].point, [1.0, 0.75, 0.3, 0.8])
                .thickness(2.0)
                .build();
        }
    }
    for node in &nodes {
        let point = node.point;
        let colour = if node.at.is_none() && !node.origin {
            [0.7, 0.7, 0.7, 1.0]
        } else {
            [1.0, 0.75, 0.3, 1.0]
        };
        if !node.origin {
            list.add_circle(point, 8.0, [0.05, 0.05, 0.07, 0.9])
                .filled(true)
                .build();
        }
        list.add_circle(point, 8.0, colour).thickness(1.5).build();
        if node.origin {
            list.add_circle(point, 10.0, colour).thickness(1.0).build();
        }
        let number = node.label();
        list.add_text(
            [point[0] - number.len() as f32 * 3.0, point[1] - 7.0],
            [1.0, 1.0, 1.0, 1.0],
            &number,
        );
    }
    // Hacia dónde va: punta de flecha en el último tramo.
    if let [.., before, last] = nodes.as_slice() {
        draw_heading(list, before.point, last.point);
    }
    // The most recent claim is the one the player wants to see first (RF-32): a bright ring.
    if let Some(last) = nodes.last() {
        list.add_circle(last.point, 12.0, [1.0, 0.4, 0.2, 1.0])
            .thickness(2.0)
            .build();
    }
}

fn draw_origin_connector(list: &nexus::imgui::DrawListMut<'_>, from: [f32; 2], to: [f32; 2]) {
    let dx = to[0] - from[0];
    let dy = to[1] - from[1];
    let length = dx.hypot(dy);
    if !length.is_finite() {
        log::warn!("route origin connector has non-finite coordinates");
        return;
    }
    if length < 1.0 {
        return;
    }
    // Bound drawing work even if coordinates fall far outside the map.
    let count = (length / 10.0).ceil().clamp(1.0, 128.0) as usize;
    let stride = length / count as f32;
    for index in 0..count {
        let at = index as f32 * stride;
        let end = (at + stride * 0.6).min(length);
        list.add_line(
            [from[0] + dx * at / length, from[1] + dy * at / length],
            [from[0] + dx * end / length, from[1] + dy * end / length],
            [1.0, 0.75, 0.3, 0.8],
        )
        .thickness(1.5)
        .build();
    }
}

/// Cuánto dura una reproducción completa de la ruta, en segundos (RF-36).
const PLAYBACK_SECS: f32 = 12.0;

/// Estado de la reproducción: desde cuándo corre, o `None` si está parada (RF-36).
static PLAYBACK: std::sync::OnceLock<std::sync::Mutex<Option<std::time::Instant>>> =
    std::sync::OnceLock::new();

/// Play/pausa de la ruta y cuántos pasos hay que dibujar, `None` para dibujarla entera.
pub(super) fn playback_control(ui: &Ui, steps: usize) -> Option<usize> {
    if steps < 2 {
        return None;
    }
    let slot = PLAYBACK.get_or_init(|| std::sync::Mutex::new(None));
    let mut started = *slot.lock().unwrap_or_else(|error| error.into_inner());
    let playing = started.is_some();
    let label = if playing {
        i18n::widget("Pause route")
    } else {
        i18n::widget("Play route")
    };
    if ui.button(label) {
        started = if playing {
            None
        } else {
            Some(std::time::Instant::now())
        };
        *slot.lock().unwrap_or_else(|error| error.into_inner()) = started;
    }
    ui.same_line();
    match started {
        None => {
            ui.text_disabled(i18n::text("Whole route"));
            None
        }
        Some(at) => {
            let fraction =
                gw2lurker_core::route::progress(at.elapsed().as_secs_f32(), PLAYBACK_SECS);
            ui.text_disabled(format!("{:.0} %", fraction * 100.0));
            Some(gw2lurker_core::route::visible_steps(steps, fraction))
        }
    }
}

/// Resumen en texto de la ruta inferida, para ponerlo **al lado del mapa** (RF-32, RF-33, RF-35).
///
/// Vivía debajo de la lista de clanes y el jugador no sabía qué hacía clicar una fila (petición del
/// 2026-11-01): ahora la ruta se dibuja en el mapa y aquí queda solo la línea que la explica,
/// incluido el «salto a <mapa>» cuando sigue en otro mapa.
pub(crate) fn summary(
    rows: &[ClaimRow],
    guild: &str,
    language: gw2lurker_core::i18n::Language,
) -> Option<String> {
    let route = gw2lurker_core::build_route(rows, guild)?;
    Some(route_steps(&route, rows, language))
}

/// La ruta en texto, con un **«salto a <mapa>»** cuando los pasos cambian de mapa (RF-35).
///
/// La ruta se construye por mapa, así que el salto no está en los pasos: se deduce de los claims,
/// que sí llevan el mapa en el id del objetivo.
pub(super) fn route_steps(
    route: &gw2lurker_core::Route,
    rows: &[ClaimRow],
    language: gw2lurker_core::i18n::Language,
) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut map: Option<MapRole> = None;
    for step in &route.steps {
        let step_map = rows
            .iter()
            .find(|row| row.objective_id.as_str() == step.objective_id.as_ref())
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
        parts.push(step.name.to_string());
    }
    parts.join(" -> ")
}

/// Nombre del mapa en el idioma que se pase: las pruebas no pueden leer el idioma activo, porque
/// eso pasa por la API de Nexus y revienta fuera del juego.
fn map_label_in(role: MapRole, language: gw2lurker_core::i18n::Language) -> &'static str {
    match role {
        MapRole::Center => t(language, "Eternals"),
        MapRole::BlueHome => t(language, "Blue alpine"),
        MapRole::GreenHome => t(language, "Green alpine"),
        MapRole::RedHome => t(language, "Red desert"),
        MapRole::Unknown => t(language, "Unknown map"),
    }
}

/// Unit vector of the last jump, `None` when it is too short to mean anything (the same
/// objective twice, or two pixels apart). Pure, so the rule can be tested.
pub(super) fn heading(from: [f32; 2], to: [f32; 2]) -> Option<(f32, f32)> {
    let dx = to[0] - from[0];
    let dy = to[1] - from[1];
    let length = (dx * dx + dy * dy).sqrt();
    (length >= 12.0).then(|| (dx / length, dy / length))
}

/// Two short strokes on the newest claim pointing away from the previous one.
///
/// Es la única pista honesta de «hacia dónde va» sin inventar el recorrido (ADR-005).
fn draw_heading(list: &nexus::imgui::DrawListMut<'_>, from: [f32; 2], to: [f32; 2]) {
    let Some((ux, uy)) = heading(from, to) else {
        return;
    };
    // Puntas a ±30° del sentido de la marcha, de 10 px.
    for angle in [0.52_f32, -0.52] {
        let (sin, cos) = angle.sin_cos();
        let (px, py) = (ux * cos - uy * sin, ux * sin + uy * cos);
        list.add_line(
            to,
            [to[0] - px * 10.0, to[1] - py * 10.0],
            [1.0, 0.55, 0.2, 0.9],
        )
        .thickness(2.0)
        .build();
    }
}
