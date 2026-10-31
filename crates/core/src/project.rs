//! Projection of WvW coordinates onto the rectangle of a window (T-14, RF-30).
//!
//! `coord` in `/v2/wvw/objectives` is already in **continent coordinates** (verified in
//! `specs/07-spikes.md`), the same space as `continent_rect` of `/v2/maps`, so the projection is
//! a linear map between two boxes and nothing else: no tiles, no textures, no `map_rect`
//! (ADR-011). `map_rect` is kept in the model because the API sends it, not because it is used.
//!
//! Pure by design: the geometry has to be testable without a game, and getting it wrong puts an
//! objective in the wrong corner of the map without any error to blame.

use crate::model::WvwMap;

/// The box of one map in continent coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    /// `[min, max]` of X.
    pub x: [f64; 2],
    /// `[min, max]` of Y.
    pub y: [f64; 2],
}

impl Frame {
    /// Box of a map from `/v2/maps`. The corners may come in either order.
    pub fn of(map: &WvwMap) -> Self {
        let [[x0, y0], [x1, y1]] = map.continent_rect;
        Self {
            x: [x0.min(x1), x0.max(x1)],
            y: [y0.min(y1), y0.max(y1)],
        }
    }

    /// Whether the box has a width and a height to divide by.
    pub fn is_usable(&self) -> bool {
        self.x[1] > self.x[0] && self.y[1] > self.y[0]
    }

    /// Continent coordinates → `0..1` in both axes, with **Y up** as the API draws it.
    ///
    /// `None` when the box is degenerate or the coordinate is not a number: an objective must
    /// never end up silently in a corner.
    pub fn normalise(&self, coord: [f64; 2]) -> Option<[f32; 2]> {
        if !self.is_usable() || !coord[0].is_finite() || !coord[1].is_finite() {
            return None;
        }
        let nx = (coord[0] - self.x[0]) / (self.x[1] - self.x[0]);
        let ny = (coord[1] - self.y[0]) / (self.y[1] - self.y[0]);
        Some([nx as f32, ny as f32])
    }
}

/// `0..1` (Y up) → pixels inside a rectangle (Y down), which is how imgui thinks.
pub fn to_screen(normalised: [f32; 2], min: [f32; 2], size: [f32; 2]) -> [f32; 2] {
    [
        min[0] + normalised[0] * size[0],
        min[1] + (1.0 - normalised[1]) * size[1],
    ]
}

/// Continent coordinates → pixels, or `None` when there is nothing sensible to draw.
pub fn project(frame: &Frame, coord: [f64; 2], min: [f32; 2], size: [f32; 2]) -> Option<[f32; 2]> {
    let normalised = frame.normalise(coord)?;
    Some(to_screen(normalised, min, size))
}

/// Zoom level used for the real map: the highest the tile server answers (verified 2026-10-31).
pub const TILE_ZOOM: u32 = 5;

/// Continent units covered by one tile at `zoom`.
///
/// Calibrado con teselas reales (2026-10-31): a zoom 5 una tesela son **1024 unidades**. El
/// servidor es una pirámide de nueve niveles y el nivel 8 no existe como tal, así que la fórmula
/// es `256 · 2^(7 − zoom)`: a zoom 5 un mapa son 4×4 teselas y a zoom 0 el continente entero.
pub fn units_per_tile(zoom: u32) -> f64 {
    256.0 * 2f64.powi(7 - zoom as i32)
}

/// Continent box of one tile: `[min_x, min_y, max_x, max_y]`.
pub fn tile_box(x: u32, y: u32, zoom: u32) -> [f64; 4] {
    let step = units_per_tile(zoom);
    let x0 = f64::from(x) * step;
    let y0 = f64::from(y) * step;
    [x0, y0, x0 + step, y0 + step]
}

/// Tiles that cover a map.
///
/// Empty when the box is degenerate, so the caller can treat it as "no background" instead of
/// asking for tile 0,0 of the whole continent.
pub fn tiles_of(frame: &Frame, zoom: u32) -> Vec<(u32, u32)> {
    if !frame.is_usable() {
        return Vec::new();
    }
    let step = units_per_tile(zoom);
    let first_x = (frame.x[0] / step).floor().max(0.0) as u32;
    let first_y = (frame.y[0] / step).floor().max(0.0) as u32;
    let last_x = (frame.x[1] / step).floor().max(0.0) as u32;
    let last_y = (frame.y[1] / step).floor().max(0.0) as u32;

    let mut tiles = Vec::new();
    for x in first_x..=last_x {
        for y in first_y..=last_y {
            tiles.push((x, y));
        }
    }
    tiles
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ObjectiveInfo;

    const OBJECTIVES: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../api/tests/fixtures/objectives_all.json"
    ));
    const MAPS: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../api/tests/fixtures/maps_wvw.json"
    ));

    fn frame_of(id: u32) -> Frame {
        let maps: Vec<WvwMap> = serde_json::from_str(MAPS).unwrap();
        Frame::of(
            maps.iter()
                .find(|map| map.id == id)
                .expect("el mapa tiene que estar en la fixture"),
        )
    }

    #[test]
    fn rf30_the_corners_and_the_centre_land_where_they_should() {
        let frame = frame_of(38);

        assert_eq!(frame.normalise([frame.x[0], frame.y[0]]), Some([0.0, 0.0]));
        assert_eq!(frame.normalise([frame.x[1], frame.y[1]]), Some([1.0, 1.0]));
        assert_eq!(
            frame.normalise([
                (frame.x[0] + frame.x[1]) / 2.0,
                (frame.y[0] + frame.y[1]) / 2.0
            ]),
            Some([0.5, 0.5])
        );
    }

    #[test]
    fn rf30_north_is_up_on_screen() {
        let frame = Frame {
            x: [0.0, 100.0],
            y: [0.0, 100.0],
        };
        let north = project(&frame, [50.0, 100.0], [10.0, 20.0], [200.0, 100.0]).unwrap();
        let south = project(&frame, [50.0, 0.0], [10.0, 20.0], [200.0, 100.0]).unwrap();

        assert_eq!(north, [110.0, 20.0], "el norte va arriba: y mínima");
        assert_eq!(south, [110.0, 120.0], "el sur va abajo");
    }

    #[test]
    fn rf30_a_degenerate_box_has_no_projection() {
        let flat = Frame {
            x: [5.0, 5.0],
            y: [0.0, 10.0],
        };
        assert!(!flat.is_usable());
        assert_eq!(flat.normalise([5.0, 5.0]), None);
        assert_eq!(project(&flat, [5.0, 5.0], [0.0; 2], [1.0; 2]), None);

        let usable = Frame {
            x: [0.0, 10.0],
            y: [0.0, 10.0],
        };
        assert_eq!(
            usable.normalise([f64::NAN, 0.0]),
            None,
            "NaN no puede colarse en imgui"
        );
    }

    #[test]
    fn rf30_the_tile_grid_covers_the_whole_map() {
        let frame = frame_of(38);
        let tiles = tiles_of(&frame, TILE_ZOOM);

        assert!(!tiles.is_empty());
        let step = units_per_tile(TILE_ZOOM);
        assert_eq!(
            step, 1024.0,
            "zoom 5 son 1024 unidades por tesela (calibrado)"
        );

        // Cada esquina del mapa cae dentro de alguna de las teselas elegidas.
        for corner_x in [frame.x[0], frame.x[1]] {
            for corner_y in [frame.y[0], frame.y[1]] {
                assert!(
                    tiles.iter().any(|(x, y)| {
                        let [x0, y0, x1, y1] = tile_box(*x, *y, TILE_ZOOM);
                        corner_x >= x0 && corner_x <= x1 && corner_y >= y0 && corner_y <= y1
                    }),
                    "la esquina {corner_x},{corner_y} se queda sin tesela"
                );
            }
        }
        assert!(
            tiles.len() <= 16,
            "un mapa son cuatro teselas por lado a zoom 5, no {}",
            tiles.len()
        );
    }

    #[test]
    fn rf30_a_degenerate_box_asks_for_no_tiles() {
        let flat = Frame {
            x: [10.0, 10.0],
            y: [0.0, 5.0],
        };
        assert!(tiles_of(&flat, TILE_ZOOM).is_empty());
    }

    #[test]
    fn rf30_every_real_objective_of_eternals_falls_inside_the_box() {
        let frame = frame_of(38);
        let objectives: Vec<ObjectiveInfo> = serde_json::from_str(OBJECTIVES).unwrap();
        let of_map: Vec<_> = objectives
            .iter()
            .filter(|objective| objective.map_id == 38)
            .collect();

        assert!(of_map.len() >= 20, "Eternos tiene que traer objetivos");
        let mut drawn = 0;
        for objective in of_map {
            let Some(coord) = objective.coord else {
                continue; // ADR-001: `coord: null` se omite, no se dibuja en (0,0)
            };
            let normalised = frame.normalise([coord[0], coord[1]]).unwrap();
            assert!(
                (0.0..=1.0).contains(&normalised[0]) && (0.0..=1.0).contains(&normalised[1]),
                "{} cae fuera del mapa: {normalised:?}",
                objective.name
            );
            drawn += 1;
        }
        assert!(drawn >= 20, "solo se dibujarían {drawn} objetivos");
    }
}
