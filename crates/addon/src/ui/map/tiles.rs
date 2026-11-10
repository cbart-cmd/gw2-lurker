use std::collections::BTreeMap;

use gw2lurker_core::project::{tile_box, tiles_of, TILE_ZOOM};
use gw2lurker_core::{project, Frame};

/// Height of the drawing area when the map has no usable proportions.
const MAP_HEIGHT: f32 = 280.0;
/// Smallest and biggest height the map may take when it keeps its real proportions.
const MIN_MAP_HEIGHT: f32 = 160.0;
const MAX_MAP_HEIGHT: f32 = 620.0;

/// Tamaño en pantalla del mapa, con la **proporción real** de `continent_rect`.
///
/// Se dibujaba `ancho × 280` y los mapas salían chafados: la frontera alpina es 2560×3584
/// (1,4 veces más alta que ancha) y Eternos es cuadrada (medido en `/v2/maps`). Con la proporción
/// de verdad el mapa se parece al de gw2mists (informe de 2026-10-31).
pub(super) fn map_size(frame: &Frame, avail: [f32; 2]) -> [f32; 2] {
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
pub(super) fn draw_tiles(
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

/// Loaded tile texture and the file version it represents.
type TileTextureCache = BTreeMap<(u32, u32), (u64, nexus::imgui::TextureId)>;

static TILE_TEXTURES: std::sync::OnceLock<std::sync::Mutex<TileTextureCache>> =
    std::sync::OnceLock::new();

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
    let revision = crate::catalog::tile_revision();
    let cached = cache
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .get(&(x, y))
        .copied();
    if let Some((cached_revision, id)) = cached {
        if cached_revision == revision {
            return Some(id);
        }
    }
    let path = crate::catalog::tile_path(x, y)?;
    let meta = match std::fs::metadata(&path) {
        Ok(meta) => meta,
        Err(_) => {
            remember_failure(x, y);
            return None;
        }
    };
    if meta.len() < crate::catalog::MIN_TILE_BYTES {
        remember_failure(x, y);
        return None;
    }
    if TILE_BUDGET.load(std::sync::atomic::Ordering::Relaxed) == 0 || failed_recently(x, y) {
        return cached.map(|(_, id)| id);
    }
    TILE_BUDGET.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    let identifier = format!("gw2lurker-tile-{x}-{y}-{revision}");
    let Some(texture) = nexus::texture::get_texture_or_create_from_file(&identifier, &path) else {
        // Sin esta línea, «el mapa no es real» no se puede distinguir de «las teselas no están».
        // Una vez por tesela y reintento: no una línea por frame.
        if !remember_failure(x, y) {
            log::warn!("tiles: no se pudo cargar la textura de {path:?}");
        }
        return cached.map(|(_, id)| id);
    };
    let id = texture.id();
    cache
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .insert((x, y), (revision, id));
    Some(id)
}

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
pub(super) fn draw_frame(
    list: &nexus::imgui::DrawListMut<'_>,
    origin: [f32; 2],
    size: [f32; 2],
    name: &str,
) {
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
pub(super) fn draw_background(
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
