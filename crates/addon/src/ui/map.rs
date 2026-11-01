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
use gw2lurker_core::project::{tile_box, tiles_of, TILE_ZOOM};
use gw2lurker_core::{project, FollowList, Frame};
use nexus::imgui::{ComboBox, Selectable, StyleColor, Ui};

use crate::i18n;

mod route;

use route::{draw_route, playback_control, range_choice, RouteCanvas};

/// La ruta inferida en texto, que se pinta junto al mapa (RF-33, RF-35).
pub(crate) use route::summary;

/// Translation key (and stable id) of this tab.
pub const KEY: &str = "Map";

/// Height of the drawing area when the map has no usable proportions.
const MAP_HEIGHT: f32 = 280.0;
/// Smallest and biggest height the map may take when it keeps its real proportions.
const MIN_MAP_HEIGHT: f32 = 160.0;
const MAX_MAP_HEIGHT: f32 = 620.0;

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
    /// Boxes of the four maps (RF-30).
    pub maps: &'a [WvwMap],
    /// Guilds the player follows: their objectives get a ring (RF-31).
    pub follow: &'a FollowList,
    /// Guild id → `[TAG] Nombre`, to label the objectives of the followed guilds (RF-31).
    pub guild_names: &'a BTreeMap<String, String>,
    /// Opacity of the window, which the real map inherits (T-29).
    pub opacity: f32,
    /// Draw the real tiles instead of the schematic board (T-29).
    pub real_map: bool,
    /// What the guild chosen in the *Guilds* tab claimed this session, for the route (T-16).
    pub history: &'a [ClaimRow],
    /// Clan cuya ruta se está dibujando, si hay uno elegido.
    pub guild: Option<&'a str>,
    /// Clanes que han reclamado algo en la sesión, **el más reciente primero**
    /// (`Dashboard::session_guilds`). Alimentan el desplegable de clan de esta pestaña.
    pub session_guilds: &'a [(String, usize, Option<chrono::DateTime<chrono::Utc>>)],
}

/// Body of the *Map* tab.
///
/// The route is always captioned as inferred (ADR-005); the caption lives next to the drawing.
pub fn body(ui: &Ui, view: &View<'_>, role: &mut MapRole, here: Option<MapRole>) -> Option<String> {
    let View {
        states,
        names,
        maps,
        follow,
        guild_names,
        opacity,
        real_map,
        history,
        guild,
        session_guilds,
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

    draw_selector(ui, role, here, maps);
    // Desplegable de clan justo al lado del de mapa: los que sigues primero y después los últimos
    // que han reclamado algo, para no tener que ir a la pestaña *Clanes* a buscarlos (petción del
    // jugador, 2026-11-01). Elegir uno dibuja su ruta, igual que clicarlo en esa pestaña.
    let chosen = draw_guild_choice(ui, follow, guild_names, guild, session_guilds);

    let Some(map) = maps
        .iter()
        .find(|map| MapRole::from_map_id(map.id) == *role)
    else {
        ui.text_disabled(i18n::text("No map data yet"));
        return chosen;
    };
    let frame = Frame::of(map);
    if !frame.is_usable() {
        ui.text_disabled(i18n::text("No map data yet"));
        return chosen;
    }

    let width = ui.content_region_avail()[0].max(140.0);
    let size = map_size(&frame, [width, ui.content_region_avail()[1]]);
    // Centrado: si la proporción real obliga a estrechar el mapa, que no quede pegado a la izquierda.
    let indent = ((width - size[0]) / 2.0).max(0.0);
    ui.set_cursor_pos([ui.cursor_pos()[0] + indent, ui.cursor_pos()[1]]);
    let origin = ui.cursor_screen_pos();
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

    let since =
        range_choice(ui).map(|minutes| chrono::Utc::now() - chrono::Duration::minutes(minutes));
    // Reproducción animada de la ruta (RF-36): play/pausa y una barra de progreso. Sin reproducir
    // se dibuja la ruta entera, que es lo que se quiere casi siempre.
    let played = playback_control(
        ui,
        history.iter().filter(|row| row.map_id == map.id).count(),
    );

    // Todo el dibujo va recortado al rectángulo del mapa. En imgui la lista de dibujo de una
    // ventana hija es la del **padre**, así que sin recorte propio las teselas y los anillos se
    // pintaban encima de las pestañas y fuera de la ventana (informe de 2026-10-31, con captura).
    // Con `with_clip_rect_intersect` el recorte del mapa se cruza con el de la ventana: nunca se
    // sale de la ventana aunque el mapa sea más grande que ella.
    list.with_clip_rect_intersect(origin, [origin[0] + size[0], origin[1] + size[1]], || {
        if real_map {
            draw_tiles(&list, &frame, origin, size, opacity);
            draw_frame(&list, origin, size, map_label(*role));
        } else {
            draw_background(&list, origin, size, &frame, map_label(*role));
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

        draw_route(
            &list,
            &RouteCanvas {
                frame: &frame,
                origin,
                size,
            },
            map.id,
            names,
            history,
            since,
            played,
        );

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
    chosen
}

/// Tamaño en pantalla del mapa, con la **proporción real** de `continent_rect`.
///
/// Se dibujaba `ancho × 280` y los mapas salían chafados: la frontera alpina es 2560×3584
/// (1,4 veces más alta que ancha) y Eternos es cuadrada (medido en `/v2/maps`). Con la proporción
/// de verdad el mapa se parece al de gw2mists (informe de 2026-10-31).
fn map_size(frame: &Frame, avail: [f32; 2]) -> [f32; 2] {
    let width = avail[0].max(140.0);
    let (wide, tall) = (frame.x[1] - frame.x[0], frame.y[1] - frame.y[0]);
    let aspect = (tall / wide) as f32;
    if !aspect.is_finite() || !(0.25..=4.0).contains(&aspect) {
        return [width, MAP_HEIGHT];
    }
    // Que quepa: si la proporción pide más alto de lo que hay, se encoge el ancho y se mantiene.
    let cap = avail[1].clamp(MIN_MAP_HEIGHT, MAX_MAP_HEIGHT);
    let height = width * aspect;
    if height > cap {
        [((cap / aspect).max(140.0)), height.min(cap)]
    } else {
        [width, height]
    }
}

/// The real map: the official tiles, drawn one by one with the opacity of the window (T-29).
///
/// A tile that is not on disk yet is simply skipped: the download runs in the worker thread and
/// the next frame will have it. Nothing blocks the render callback.
fn draw_tiles(
    list: &nexus::imgui::DrawListMut<'_>,
    frame: &Frame,
    origin: [f32; 2],
    size: [f32; 2],
    opacity: f32,
) {
    // El presupuesto de texturas es **por frame**: sin reiniciarlo se gastaba una vez por sesión
    // y el mapa se quedaba con las cuatro primeras teselas y nada más (informe de 2026-10-31,
    // «el mapa no es real»).
    TILE_BUDGET.store(TILES_PER_FRAME, std::sync::atomic::Ordering::Relaxed);
    for (x, y) in tiles_of(frame, TILE_ZOOM) {
        let Some(texture) = tile_texture(x, y) else {
            continue;
        };
        let [x0, y0, x1, y1] = tile_box(x, y, TILE_ZOOM);
        let (Some(a), Some(b)) = (
            project(frame, [x0, y0], origin, size),
            project(frame, [x1, y1], origin, size),
        ) else {
            continue;
        };
        // Las dos esquinas se proyectan igual: continente e imgui crecen hacia el sur, así que no
        // hay que invertir nada y la imagen de la tesela se coloca con su norte arriba.
        let p_min = [a[0].min(b[0]), a[1].min(b[1])];
        let p_max = [a[0].max(b[0]), a[1].max(b[1])];
        list.add_image(texture, p_min, p_max)
            .col([1.0, 1.0, 1.0, opacity])
            .build();
    }
}

/// Loaded tiles, by coordinates. They never change, so the session keeps them.
static TILE_TEXTURES: std::sync::OnceLock<
    std::sync::Mutex<BTreeMap<(u32, u32), nexus::imgui::TextureId>>,
> = std::sync::OnceLock::new();

/// How many tiles may be created in one frame. The first time the tab is opened the map fills in
/// over a few frames instead of stalling the game.
const TILES_PER_FRAME: u8 = 4;

/// What is left of that allowance in the current frame. Reset by [`draw_tiles`].
static TILE_BUDGET: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(TILES_PER_FRAME);

/// Texture of one tile, loading it from the disk cache the first time it is needed.
///
/// A tile that fails is remembered for [`TILE_RETRY`] so the per-frame budget reaches the rest
/// instead of burning itself on the same four files: with the download writing straight to the
/// final name, the render could read a half-written JPEG, and those four failures starved every
/// other tile of the map (log of 2026-10-31: «no se pudo cargar la textura» in a loop).
fn tile_texture(x: u32, y: u32) -> Option<nexus::imgui::TextureId> {
    let cache = TILE_TEXTURES.get_or_init(|| std::sync::Mutex::new(BTreeMap::new()));
    if let Some(id) = cache
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .get(&(x, y))
    {
        return Some(*id);
    }
    if TILE_BUDGET.load(std::sync::atomic::Ordering::Relaxed) == 0 {
        return None;
    }
    if failed_recently(x, y) {
        return None;
    }
    let path = crate::catalog::tile_path(x, y)?;
    // Un fichero de cero bytes es una descarga a medias: no es que falte, es que no se puede leer.
    let meta = match std::fs::metadata(&path) {
        Ok(meta) => meta,
        Err(_) => {
            remember_failure(x, y);
            return None;
        }
    };
    if meta.len() < MIN_TILE_BYTES {
        remember_failure(x, y);
        return None;
    }
    TILE_BUDGET.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    let identifier = format!("gw2lurker-tile-{x}-{y}");
    let Some(texture) = nexus::texture::get_texture_or_create_from_file(&identifier, &path) else {
        // Sin esta línea, «el mapa no es real» no se puede distinguir de «las teselas no están».
        // Una vez por tesela y reintento: no una línea por frame.
        if !remember_failure(x, y) {
            log::warn!("tiles: no se pudo cargar la textura de {path:?}");
        }
        return None;
    };
    let id = texture.id();
    cache
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .insert((x, y), id);
    Some(id)
}

/// Smallest file that can be a tile: anything less is a download in progress.
const MIN_TILE_BYTES: u64 = 1024;

/// How long a tile that failed is left alone before trying again.
const TILE_RETRY: std::time::Duration = std::time::Duration::from_secs(5);

/// When each tile failed last, so neither the budget nor the log repeats it every frame.
fn failures() -> &'static std::sync::Mutex<BTreeMap<(u32, u32), std::time::Instant>> {
    static FAILURES: std::sync::OnceLock<
        std::sync::Mutex<BTreeMap<(u32, u32), std::time::Instant>>,
    > = std::sync::OnceLock::new();
    FAILURES.get_or_init(|| std::sync::Mutex::new(BTreeMap::new()))
}

fn failed_recently(x: u32, y: u32) -> bool {
    failures()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .get(&(x, y))
        .is_some_and(|at| at.elapsed() < TILE_RETRY)
}

/// Marks the tile as failed. Returns `true` when it was already failing (so the log says it once).
fn remember_failure(x: u32, y: u32) -> bool {
    let mut map = failures().lock().unwrap_or_else(|error| error.into_inner());
    map.insert((x, y), std::time::Instant::now()).is_some()
}

/// Border and name over whatever the background is.
fn draw_frame(list: &nexus::imgui::DrawListMut<'_>, origin: [f32; 2], size: [f32; 2], name: &str) {
    let corner = [origin[0] + size[0], origin[1] + size[1]];
    list.add_rect(origin, corner, [0.0, 0.0, 0.0, 0.75])
        .thickness(1.0)
        .build();
    list.add_text(
        [origin[0] + 6.0, origin[1] + 4.0],
        [1.0, 1.0, 1.0, 0.9],
        name,
    );
}

/// Dark board with a grid and the map name: no textures (ADR-002), but the objectives are not
/// floating on nothing either (report of 2026-10-31). The real map is [`draw_tiles`].
fn draw_background(
    list: &nexus::imgui::DrawListMut<'_>,
    origin: [f32; 2],
    size: [f32; 2],
    frame: &Frame,
    name: &str,
) {
    let corner = [origin[0] + size[0], origin[1] + size[1]];
    list.add_rect(origin, corner, [0.10, 0.11, 0.13, 0.9])
        .filled(true)
        .build();

    // A line every 512 continent units, which is a quarter of a borderland.
    let step_x = (512.0 / (frame.x[1] - frame.x[0])) as f32 * size[0];
    let step_y = (512.0 / (frame.y[1] - frame.y[0])) as f32 * size[1];
    if step_x > 6.0 && step_y > 6.0 {
        let colour = [1.0, 1.0, 1.0, 0.06];
        let mut x = origin[0] + step_x;
        while x < corner[0] {
            list.add_line([x, origin[1]], [x, corner[1]], colour)
                .build();
            x += step_x;
        }
        let mut y = origin[1] + step_y;
        while y < corner[1] {
            list.add_line([origin[0], y], [corner[0], y], colour)
                .build();
            y += step_y;
        }
    }

    list.add_rect(origin, corner, [0.32, 0.34, 0.38, 1.0])
        .thickness(1.0)
        .build();
    list.add_text(
        [origin[0] + 6.0, origin[1] + 4.0],
        [0.75, 0.78, 0.82, 1.0],
        name,
    );
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
const MAP_COMBO_WIDTH: f32 = 120.0;

/// Ancho del desplegable de clan: `[TAG] Nombre` no cabe en menos.
const GUILD_COMBO_WIDTH: f32 = 210.0;

fn draw_selector(ui: &Ui, role: &mut MapRole, here: Option<MapRole>, maps: &[WvwMap]) {
    *role = resolved_map_role(*role, here);
    let current = map_label(*role);
    let _width = ui.push_item_width(MAP_COMBO_WIDTH);
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
