//! Simplified map of one WvW map (T-14, RF-30).
//!
//! Objectives as the circles of the game: team colour outside and the drawing of a camp, a tower,
//! a keep or the castle inside ([`super::icons`]), over the official tiles or a plain background.
//! The projection comes from `continent_rect` of `/v2/maps` (ADR-011) and lives in
//! `gw2lurker_core::project`, which is pure and tested against the real fixtures.
//!
//! Only draws: coordinates, owners and guilds are resolved by the worker.

use std::collections::BTreeMap;

use gw2lurker_core::matchup::{ClaimRow, ObjectiveState};
use gw2lurker_core::model::{MapRole, ObjectiveInfo, ObjectiveType, TeamColor, WvwMap};
use gw2lurker_core::{project, FollowList, Frame};
use nexus::imgui::{ComboBox, Selectable, StyleColor, Ui};

use crate::i18n;

use super::navigation;

mod route;
mod scene;
mod tiles;
mod timeline;

#[cfg(test)]
mod trail_tests;

use route::{draw_route, RouteCanvas};
use tiles::{draw_background, draw_frame, draw_tiles, map_size};

pub(super) use timeline::pause_unless_drawn;

/// La ruta inferida en texto, que se pinta junto al mapa (RF-33, RF-35).
pub(crate) use route::summary;

/// Translation key (and stable id) of this tab.
pub const KEY: &str = "Map";

/// Grey of an objective nobody holds.
const UNCLAIMED: [f32; 4] = [0.55, 0.57, 0.6, 1.0];

/// Ring drawn around the objectives of a followed guild (RF-31).
const FOLLOWED_RING: [f32; 4] = [1.0, 0.86, 0.4, 1.0];

/// Everything the map tab draws, gathered by the caller (`ui::draw_tabs`).
///
/// A struct and not eight parameters: the signature was already at the limit clippy allows.
pub struct View<'a> {
    /// Every objective of the matchup with its owner and, if claimed, its guild (RF-30b).
    ///
    /// Not the claim rows: an objective captured without a claim has no row there and used to be
    /// drawn as unclaimed.
    pub states: &'a [ObjectiveState],
    /// Objectives of the player's language, with the `coord` the drawing needs.
    pub names: &'a [ObjectiveInfo],
    /// Boxes of known WvW maps, including alternate borderland geometry (RF-30).
    pub maps: &'a [WvwMap],
    /// Mumble map id, used because beta borderland instances share id 1652.
    pub current_map_id: u32,
    /// Guilds the player follows: their objectives get a ring (RF-31).
    pub follow: &'a FollowList,
    /// Guild id → `[TAG] Nombre`, to label the objectives of the followed guilds (RF-31).
    pub guild_names: &'a BTreeMap<String, String>,
    /// Opacity of the window, which the real map inherits (T-29).
    pub opacity: f32,
    /// Draw the real tiles instead of the schematic board (T-29).
    pub real_map: bool,
    /// Clan cuya ruta se está dibujando, si hay uno elegido.
    pub guild: Option<&'a str>,
    /// Clanes que han reclamado algo en la sesión, **el más reciente primero**
    /// (`Dashboard::session_guilds`). Alimentan el desplegable de clan de esta pestaña.
    pub session_guilds: &'a [(String, usize, Option<chrono::DateTime<chrono::Utc>>)],
    /// Worker revision and tracking-cycle start; invalidate the route cache.
    pub revision: (
        Option<chrono::DateTime<chrono::Utc>>,
        Option<chrono::DateTime<chrono::Utc>>,
    ),
    /// Temporal model prepared on the worker, shared without cloning claims every frame.
    pub prepared: Option<&'a std::sync::Arc<gw2lurker_core::route::trail::Trail<'static>>>,
}

/// Body of the *Map* tab.
///
/// The route is always captioned as inferred (ADR-005); the caption lives next to the drawing.
pub fn body(ui: &Ui, view: &View<'_>, role: &mut MapRole, here: Option<MapRole>) -> Option<String> {
    let View {
        states,
        names,
        maps,
        current_map_id,
        follow,
        guild_names,
        opacity,
        real_map,
        guild,
        session_guilds,
        ..
    } = *view;

    // Once per session: if the tab ever takes the game down, this is the line that says with
    // what it was working (reported 2026-10-31). Cheap and only on the first draw.
    static LOGGED: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    if LOGGED.set(()).is_ok() {
        let with_coord = names.iter().filter(|o| o.coord.is_some()).count();
        log::info!(
            "mapa: {} mapas, {} objetivos ({} con coord), rol {:?}, en {:?}",
            maps.len(),
            names.len(),
            with_coord,
            role,
            here
        );
    }

    draw_selector(ui, role, here, current_map_id, states, maps);
    // Desplegable de clan justo al lado del de mapa: los que sigues primero y después los últimos
    // que han reclamado algo, para no tener que ir a la pestaña *Clanes* a buscarlos (petción del
    // jugador, 2026-11-01). Elegir uno dibuja su ruta, igual que clicarlo en esa pestaña.
    let pick = draw_guild_choice(ui, follow, guild_names, guild, session_guilds);
    // El desplegable manda sobre su propio estado: «No guild» lo vacía y el resto elige la ruta.
    if let Some(pick) = pick.as_deref() {
        let next = navigation::guild_from_pick(pick);
        navigation::set_guild_choice(next.as_deref());
    }
    let chosen = pick.and_then(|pick| navigation::guild_from_pick(&pick));
    let mut route_state = timeline::prepare(ui, view);

    let Some(map) = map_for_role(maps, states, *role, here, current_map_id) else {
        ui.text_disabled(i18n::text("No map data yet"));
        return chosen;
    };
    let frame = Frame::of(map);
    if !frame.is_usable() {
        ui.text_disabled(i18n::text("No map data yet"));
        return chosen;
    }
    if !has_live_state_for_map(states, map.id) {
        ui.text_disabled(i18n::text(
            "Live objective status is unavailable for this map yet.",
        ));
    }

    let width = ui.content_region_avail()[0].max(140.0);
    let size = map_size(&frame, [width, ui.content_region_avail()[1]]);
    // Centrado: si la proporción real obliga a estrechar el mapa, que no quede pegado a la izquierda.
    let indent = ((width - size[0]) / 2.0).max(0.0);
    ui.set_cursor_pos([ui.cursor_pos()[0] + indent, ui.cursor_pos()[1]]);
    let origin = ui.cursor_screen_pos();
    let canvas = RouteCanvas {
        frame: &frame,
        origin,
        size,
    };
    route_state.prepare_scene(view, map.id, &canvas);
    // The area is reserved with a dummy so the window sizes itself; the drawing itself goes
    // straight to the draw list, which is what imgui wants for shapes.
    ui.dummy(size);

    // **Once per frame**: `get_window_draw_list` panics if an instance of the window draw list is
    // already alive ("You can only load one instance of it!", `arcdps-imgui` draw_list.rs:103).
    // Asking for it inside the loop took the game down the first time the tab was opened
    // (reported and confirmed in `Nexus.log`, 2026-10-31).
    let list = ui.get_window_draw_list();
    // One pass to know where every objective lands: the drawing and the hover check need the
    // same points, and asking `get_window_draw_list` twice per frame is a panic (see above).
    let mut drawn: Vec<([f32; 2], &ObjectiveInfo, Option<&ObjectiveState>)> = Vec::new();
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
        let state = states
            .iter()
            .find(|state| state.objective_id == objective.id);
        drawn.push((point, objective, state));
    }

    // Hover: the cursor position is enough, no imgui item needed for a drawing made by hand.
    let mouse = ui.io().mouse_pos;
    let spots: Vec<[f32; 2]> = drawn.iter().map(|(point, ..)| *point).collect();
    let hovered = if inside(mouse, origin, size) {
        nearest(&spots, mouse, 12.0)
    } else {
        None
    };

    // El detalle del objetivo se pinta **encima del ratón**: la línea de abajo obligaba a hacer
    // scroll para leerla (informe de 2026-10-31). Se calcula aquí porque necesita `ui`, y se
    // dibuja dentro del recorte del mapa, que es donde está el ratón.
    let hover_text = hovered
        .and_then(|index| drawn.get(index))
        .map(|(_, _, state)| detail_line(*state, guild_names, follow, i18n::active()));

    // Todo el dibujo va recortado al rectángulo del mapa. En imgui la lista de dibujo de una
    // ventana hija es la del **padre**, así que sin recorte propio las teselas y los anillos se
    // pintaban encima de las pestañas y fuera de la ventana (informe de 2026-10-31, con captura).
    // Con `with_clip_rect_intersect` el recorte del mapa se cruza con el de la ventana: nunca se
    // sale de la ventana aunque el mapa sea más grande que ella.
    list.with_clip_rect_intersect(origin, [origin[0] + size[0], origin[1] + size[1]], || {
        if real_map {
            draw_tiles(&list, &frame, origin, size, opacity);
            draw_frame(&list, origin, size, &map_label_for_id(map.id, *role));
        } else {
            draw_background(
                &list,
                origin,
                size,
                &frame,
                &map_label_for_id(map.id, *role),
            );
        }

        for (index, (point, objective, state)) in drawn.iter().enumerate() {
            let guild = state.and_then(|state| state.guild.as_deref());
            let followed = guild.is_some_and(|guild| follow.contains(guild));
            draw_objective(
                &list,
                *point,
                objective.kind,
                state.and_then(|state| state.owner),
                has_high_value_ring(*state),
                followed,
            );
            if let (true, Some(guild)) = (followed, guild) {
                // El tag del clan, junto al anillo: el color no puede ser la única información (RF-31).
                if let Some(tag) = guild_names.get(guild).and_then(|label| tag_of(label)) {
                    list.add_text([point[0] + 10.0, point[1] - 6.0], FOLLOWED_RING, tag);
                }
            }
            if hovered == Some(index) {
                list.add_circle(*point, 13.0, [1.0, 1.0, 1.0, 0.9])
                    .thickness(1.5)
                    .build();
            }
        }

        if guild.is_some() {
            draw_route(&list, &route_state.scene);
        }

        if let Some(text) = &hover_text {
            let [text_w, text_h] = ui.calc_text_size(text);
            let pad = 4.0;
            // Pegado al ratón, pero sin salirse del mapa: el recorte se lo comería.
            let x = (mouse[0] + 14.0)
                .min(origin[0] + size[0] - text_w - pad)
                .max(origin[0] + pad);
            let y = (mouse[1] + 12.0)
                .min(origin[1] + size[1] - text_h - pad)
                .max(origin[1] + pad);
            let at = [x, y];
            list.add_rect(
                [at[0] - pad, at[1] - pad],
                [at[0] + text_w + pad, at[1] + text_h + pad],
                [0.02, 0.02, 0.03, 0.85],
            )
            .filled(true)
            .build();
            list.add_text(at, [1.0, 1.0, 1.0, 1.0], text);
        }
    });

    // Con el ratón sobre un objetivo el detalle ya va encima de él; abajo solo queda la pista,
    // para que no se lea dos veces y para no empujar la ventana hacia abajo.
    if hover_text.is_none() {
        ui.text_disabled(i18n::text("Point at an objective to see who holds it"));
    }
    drop(list);
    if guild.is_some() {
        route::hover(ui, &route_state, &canvas);
        route::caption(ui, &route_state, names);
    }
    chosen
}

/// `[TAG] Nombre` → `TAG`, which is what fits next to a ring.
fn tag_of(label: &str) -> Option<&str> {
    let start = label.find('[')? + 1;
    let end = label[start..].find(']')? + start;
    Some(&label[start..end])
}

/// Inside the drawing area?
fn inside(point: [f32; 2], origin: [f32; 2], size: [f32; 2]) -> bool {
    point[0] >= origin[0]
        && point[0] <= origin[0] + size[0]
        && point[1] >= origin[1]
        && point[1] <= origin[1] + size[1]
}

/// Index of the spot closest to `mouse`, if there is one within `max_distance` pixels.
///
/// Pure so the rule can be tested without a game: the closest wins, not the first, because two
/// objectives can be a few pixels apart on a borderland.
fn nearest(spots: &[[f32; 2]], mouse: [f32; 2], max_distance: f32) -> Option<usize> {
    let mut best: Option<(usize, f32)> = None;
    for (index, spot) in spots.iter().enumerate() {
        let dx = spot[0] - mouse[0];
        let dy = spot[1] - mouse[1];
        let distance = (dx * dx + dy * dy).sqrt();
        if distance <= max_distance && best.is_none_or(|(_, best)| distance < best) {
            best = Some((index, distance));
        }
    }
    best.map(|(index, _)| index)
}

/// Una línea con lo que dice el hover, **sin el nombre del sitio**: el jugador ya sabe dónde está y
/// el nombre alargaba la caja hasta tapar los objetivos vecinos (petición del jugador, 2026-11-01).
///
/// Orden: dueño, clan, hora y, **al final**, «Alto valor» — se mira primero quién lo tiene y cuándo,
/// y el alto valor es un matiz (petición del jugador, 2026-11-01: «sería azul · * yak… · 12:31 ·
/// alto valor»).
///
/// El idioma entra como parámetro: leer el activo pasa por Nexus (API RealTime), que revienta fuera
/// del juego y haría esto imposible de probar — el mismo motivo que en `alerts::spans`.
fn detail_line(
    state: Option<&ObjectiveState>,
    guild_names: &BTreeMap<String, String>,
    follow: &FollowList,
    language: gw2lurker_core::i18n::Language,
) -> String {
    use gw2lurker_core::i18n::t;
    let mut parts: Vec<String> = Vec::new();
    if let Some(state) = state {
        // El dueño sale del objetivo, **no** del reclamo: un objetivo capturado sin reclamar
        // también tiene color (informe de 2026-11-01, «Puesto avanzado abrasado»).
        if let Some(owner) = state.owner {
            parts.push(
                t(
                    language,
                    match owner {
                        TeamColor::Red => "Red",
                        TeamColor::Green => "Green",
                        TeamColor::Blue => "Blue",
                    },
                )
                .to_owned(),
            );
        }
        match state.guild.as_deref() {
            // `[TAG] Nombre`, no el id: el hover decía el id y no había forma de reconocer el clan
            // (informe de 2026-10-31).
            Some(guild) => {
                let name = guild_names
                    .get(guild)
                    .cloned()
                    .unwrap_or_else(|| guild.to_owned());
                parts.push(if follow.contains(guild) {
                    format!("* {name}")
                } else {
                    name
                });
                if let Some(at) = state.claimed_at {
                    parts.push(crate::clock::clock(at));
                }
            }
            None => parts.push(t(language, "Unclaimed").to_owned()),
        }
        if state.is_high_value() {
            parts.push(t(language, "High value").to_owned());
        }
    } else {
        parts.push(t(language, "Unclaimed").to_owned());
    }
    parts.join("  ·  ")
}

/// Ancho del desplegable de mapa. Antes ocupaba todo el ancho de la ventana para enseñar cuatro
/// palabras («Eternos», «Alpino azul»…) y dejaba sin sitio al de clan (petición del jugador,
/// 2026-11-01).
const MAP_COMBO_WIDTH: f32 = 200.0;

/// Ancho del desplegable de clan: `[TAG] Nombre` no cabe en menos.
const GUILD_COMBO_WIDTH: f32 = 210.0;

fn draw_selector(
    ui: &Ui,
    role: &mut MapRole,
    here: Option<MapRole>,
    current_map_id: u32,
    states: &[ObjectiveState],
    maps: &[WvwMap],
) {
    *role = resolved_map_role(*role, here);
    let current = map_for_role(maps, states, *role, here, current_map_id)
        .map(|map| map_label_for_id(map.id, *role))
        .unwrap_or_else(|| map_label(*role).to_owned());
    let _width = ui.push_item_width(MAP_COMBO_WIDTH);
    ComboBox::new(i18n::widget("Map"))
        .preview_value(current.as_str())
        .build(ui, || {
            for candidate in [
                MapRole::Center,
                MapRole::BlueHome,
                MapRole::GreenHome,
                MapRole::RedHome,
            ] {
                let Some(map) = map_for_role(maps, states, candidate, here, current_map_id) else {
                    continue;
                };
                let label = map_label_for_id(map.id, candidate);
                let _tint = if here == Some(candidate) {
                    Some(ui.push_style_color(StyleColor::Text, [0.6, 0.9, 1.0, 1.0]))
                } else {
                    None
                };
                if Selectable::new(label.as_str())
                    .selected(*role == candidate)
                    .build(ui)
                {
                    *role = candidate;
                }
            }
        });
}

/// Los clanes del desplegable: primero los que sigue el jugador (en el orden en que los añadió) y
/// después los que han reclamado algo **en la sesión**, el más reciente primero, sin repetir los
/// que ya salen arriba.
fn guild_choices(
    follow: &FollowList,
    session: &[(String, usize, Option<chrono::DateTime<chrono::Utc>>)],
    limit: usize,
) -> Vec<String> {
    let mut out: Vec<String> = follow.iter().map(str::to_owned).collect();
    for (guild, ..) in session {
        if out.len() >= follow.len() + limit {
            break;
        }
        if !out.iter().any(|known| known == guild) {
            out.push(guild.clone());
        }
    }
    out
}

/// Desplegable de clan, al lado del de mapa. Devuelve el que el jugador acaba de elegir, si hay uno.
///
/// La ventana de la pestaña es estrecha: si no caben los dos, el de clan baja a la línea siguiente
/// en vez de salirse por la derecha.
fn draw_guild_choice(
    ui: &Ui,
    follow: &FollowList,
    names: &BTreeMap<String, String>,
    current: Option<&str>,
    session: &[(String, usize, Option<chrono::DateTime<chrono::Utc>>)],
) -> Option<String> {
    let choices = guild_choices(follow, session, GUILDS_IN_MENU);
    let preview = current
        .map(|guild| label_of(names, guild))
        .unwrap_or_else(|| i18n::text("No guild").to_owned());

    // Al lado del de mapa si cabe; si la ventana está estrecha, se baja de línea en vez de salirse.
    let remaining = ui.content_region_avail()[0];
    if remaining >= GUILD_COMBO_WIDTH.min(160.0) {
        ui.same_line();
    }
    let mut chosen = None;
    let _width = ui.push_item_width(GUILD_COMBO_WIDTH);
    ComboBox::new(i18n::widget("Guild"))
        .preview_value(preview)
        .build(ui, || {
            // Primera entrada: volver a no elegir ninguno. Sin ella el desplegable no se vaciaba
            // y la ruta se quedaba pegada al mapa (petición del jugador, 2026-11-01).
            if Selectable::new(format!("{}###noguild", i18n::text("No guild")))
                .selected(current.is_none())
                .build(ui)
            {
                chosen = Some(String::new());
            }
            if choices.is_empty() {
                ui.text_disabled(i18n::text("None yet"));
                return;
            }
            for guild in &choices {
                let followed = follow.contains(guild);
                let star = if followed { "* " } else { "" };
                let text = format!("{star}{}###guild{guild}", label_of(names, guild));
                if Selectable::new(text)
                    .selected(current == Some(guild.as_str()))
                    .build(ui)
                {
                    chosen = Some(guild.clone());
                }
            }
        });
    chosen
}

/// Cuántos clanes **no seguidos** ofrece el desplegable del mapa.
///
/// Cinco: es un atajo para el clan que acaba de reclamar, no un listado (petición del jugador,
/// 2026-11-01: «y justo después los últimos 5 que han reclamado algo»).
const GUILDS_IN_MENU: usize = 5;

/// `[TAG] Nombre` cuando se conoce, el id si no.
fn label_of(names: &BTreeMap<String, String>, guild: &str) -> String {
    names
        .get(guild)
        .cloned()
        .unwrap_or_else(|| guild.to_owned())
}

/// Objectives worth drawing: the ones that score (RF-30a hides ruins, mercenaries and spawns).
fn is_drawable(kind: ObjectiveType) -> bool {
    matches!(
        kind,
        ObjectiveType::Camp | ObjectiveType::Tower | ObjectiveType::Keep | ObjectiveType::Castle
    )
}

fn has_high_value_ring(state: Option<&ObjectiveState>) -> bool {
    state.is_some_and(ObjectiveState::is_high_value)
}

fn resolved_map_role(role: MapRole, here: Option<MapRole>) -> MapRole {
    if role == MapRole::Unknown {
        here.unwrap_or(MapRole::Center)
    } else {
        role
    }
}

fn map_for_role<'a>(
    maps: &'a [WvwMap],
    states: &[ObjectiveState],
    role: MapRole,
    here: Option<MapRole>,
    current_map_id: u32,
) -> Option<&'a WvwMap> {
    if here == Some(role) {
        if let Some(map) = maps.iter().find(|map| map.id == current_map_id) {
            return Some(map);
        }
    }

    states
        .iter()
        .find(|state| state.map_role == role)
        .and_then(|state| maps.iter().find(|map| map.id == state.map_id))
        .or_else(|| maps.iter().find(|map| MapRole::from_map_id(map.id) == role))
}

fn has_live_state_for_map(states: &[ObjectiveState], map_id: u32) -> bool {
    states.iter().any(|state| state.map_id == map_id)
}

pub(super) fn map_label_for_id(map_id: u32, role: MapRole) -> String {
    if matches!(map_id, 1652 | 1654) {
        let color = match role {
            MapRole::BlueHome => i18n::text("Blue"),
            MapRole::GreenHome => i18n::text("Green"),
            MapRole::RedHome => i18n::text("Red"),
            _ => i18n::text("Unknown map"),
        };
        format!("{} ({color})", i18n::text("Seared Borderlands"))
    } else {
        map_label(role).to_owned()
    }
}

fn draw_objective(
    list: &nexus::imgui::DrawListMut<'_>,
    point: [f32; 2],
    kind: ObjectiveType,
    owner: Option<TeamColor>,
    high_value: bool,
    followed: bool,
) {
    let colour = owner.map_or(UNCLAIMED, colour_rgba);
    // Como en el juego: un círculo del color del equipo con el dibujo dentro (T-46). El tamaño
    // sigue al tipo, así que la forma se lee aunque el color no se distinga.
    let radius = match kind {
        ObjectiveType::Camp => 5.5,
        ObjectiveType::Tower => 6.5,
        ObjectiveType::Keep => 8.0,
        _ => 9.5,
    };
    super::icons::badge(list, kind, point, radius, colour);

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

/// Los colores de equipo salen del mismo sitio en todo el addon (RF-62).
fn colour_rgba(colour: TeamColor) -> [f32; 4] {
    super::theme::team_colour(colour)
}

/// El nombre del mapa en el idioma activo: lo comparten la pestaña y la cabecera del widget.
pub(super) fn map_label(role: MapRole) -> &'static str {
    match role {
        MapRole::Center => i18n::text("Eternals"),
        MapRole::BlueHome => i18n::text("Blue alpine"),
        MapRole::GreenHome => i18n::text("Green alpine"),
        MapRole::RedHome => i18n::text("Red desert"),
        MapRole::Unknown => i18n::text("Unknown map"),
    }
}

#[cfg(test)]
mod tests;
