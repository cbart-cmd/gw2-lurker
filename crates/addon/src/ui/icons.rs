//! Objective glyphs drawn with primitives: the little drawing inside the team-coloured circle
//! of the game (T-46) and the same glyph ahead of every alert line (T-47).
//!
//! No assets and no font glyphs: the game's icons are a camp, a tower, a keep and the castle, and
//! each one is a handful of rectangles and triangles. Drawing them ourselves means the shapes
//! scale with the map, cost nothing at load time and cannot be missing from a font.
//!
//! Shapes are built as a list of [`Part`]s in unit coordinates, which is what the tests check;
//! painting them is [`paint`]'s only job.

use gw2lurker_core::model::ObjectiveType;
use nexus::imgui::DrawListMut;

/// Colour of the drawing inside the circle: white, like the game, so the team colour stays the
/// only colour on screen and the shape carries the kind (RF-30).
pub const INK: [f32; 4] = [1.0, 1.0, 1.0, 0.95];

/// Dark outline around the circle, which is what makes it readable over the map (T-46).
const OUTLINE: [f32; 4] = [0.0, 0.0, 0.0, 0.65];

/// One primitive of a glyph.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shape {
    Rect([f32; 2], [f32; 2]),
    Tri([[f32; 2]; 3]),
}

/// Which colour a [`Part`] is painted with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ink {
    /// The drawing itself.
    Glyph,
    /// The hole: the door or the gate, painted in the colour behind so it reads as cut out.
    Paper,
}

/// One primitive of a glyph, in screen coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Part {
    pub shape: Shape,
    pub ink: Ink,
}

/// Whether a kind has a drawing of its own.
///
/// The four that score are the ones the game marks; anything else (ruins, spawns) gets the
/// generic mark, which is also what the alert lines use.
pub fn has_glyph(kind: ObjectiveType) -> bool {
    matches!(
        kind,
        ObjectiveType::Camp | ObjectiveType::Tower | ObjectiveType::Keep | ObjectiveType::Castle
    )
}

/// The primitives of a glyph, centred on `centre` and fitting in a box of `2 · half` a side.
///
/// Pure: no imgui, no draw list. That is what the tests assert on.
pub fn parts(kind: ObjectiveType, centre: [f32; 2], half: f32) -> Vec<Part> {
    let [cx, cy] = centre;
    let h = half.max(0.0);

    let rect = |x0: f32, y0: f32, x1: f32, y1: f32, ink: Ink| Part {
        shape: Shape::Rect([cx + x0 * h, cy + y0 * h], [cx + x1 * h, cy + y1 * h]),
        ink,
    };
    let tri = |a: [f32; 2], b: [f32; 2], c: [f32; 2], ink: Ink| Part {
        shape: Shape::Tri([
            [cx + a[0] * h, cy + a[1] * h],
            [cx + b[0] * h, cy + b[1] * h],
            [cx + c[0] * h, cy + c[1] * h],
        ]),
        ink,
    };

    match kind {
        // Una tienda de campaña: tejado triangular y la puerta recortada.
        ObjectiveType::Camp => vec![
            tri([0.0, -1.0], [-0.95, 0.85], [0.95, 0.85], Ink::Glyph),
            tri([0.0, -0.15], [-0.34, 0.85], [0.34, 0.85], Ink::Paper),
        ],
        // Una torre: cuerpo estrecho, dos almenas y la puerta.
        ObjectiveType::Tower => vec![
            rect(-0.45, -0.5, 0.45, 1.0, Ink::Glyph),
            rect(-0.45, -1.0, -0.12, -0.45, Ink::Glyph),
            rect(0.12, -1.0, 0.45, -0.45, Ink::Glyph),
            rect(-0.17, 0.4, 0.17, 1.0, Ink::Paper),
        ],
        // Una fortaleza: más ancha, tres almenas y la puerta más grande.
        ObjectiveType::Keep => vec![
            rect(-0.85, -0.4, 0.85, 1.0, Ink::Glyph),
            rect(-0.85, -0.9, -0.45, -0.35, Ink::Glyph),
            rect(-0.2, -0.9, 0.2, -0.35, Ink::Glyph),
            rect(0.45, -0.9, 0.85, -0.35, Ink::Glyph),
            rect(-0.24, 0.3, 0.24, 1.0, Ink::Paper),
        ],
        // El castillo: dos torres a los lados y la muralla en medio, más baja.
        ObjectiveType::Castle => vec![
            rect(-1.0, -0.95, -0.55, 1.0, Ink::Glyph),
            rect(0.55, -0.95, 1.0, 1.0, Ink::Glyph),
            rect(-0.55, -0.3, 0.55, 1.0, Ink::Glyph),
            rect(-0.32, -0.6, -0.06, -0.25, Ink::Glyph),
            rect(0.06, -0.6, 0.32, -0.25, Ink::Glyph),
            rect(-0.22, 0.35, 0.22, 1.0, Ink::Paper),
        ],
        // Lo que no es de los cuatro: una marca genérica, nunca nada.
        _ => vec![tri([0.0, -0.9], [-0.85, 0.6], [0.85, 0.6], Ink::Glyph)],
    }
}

/// The circle of the game with the drawing inside: team colour outside, white glyph inside.
pub fn badge(
    list: &DrawListMut<'_>,
    kind: ObjectiveType,
    centre: [f32; 2],
    radius: f32,
    fill: [f32; 4],
) {
    list.add_circle(centre, radius, OUTLINE)
        .thickness(1.0)
        .build();
    list.add_circle(centre, radius - 0.5, fill)
        .filled(true)
        .build();
    paint(list, &parts(kind, centre, radius * 0.58), INK, fill);
}

/// Paints a glyph on its own, without the circle: used by the alert lines (T-47).
pub fn glyph(
    list: &DrawListMut<'_>,
    kind: ObjectiveType,
    centre: [f32; 2],
    half: f32,
    colour: [f32; 4],
) {
    // Sin fondo detrás no hay nada que recortar: la puerta se pinta del mismo color que el texto.
    paint(list, &parts(kind, centre, half), colour, colour);
}

/// Paints the primitives. `paper` is the colour of the holes.
pub fn paint(list: &DrawListMut<'_>, parts: &[Part], ink: [f32; 4], paper: [f32; 4]) {
    for part in parts {
        let colour = match part.ink {
            Ink::Glyph => ink,
            Ink::Paper => paper,
        };
        match part.shape {
            Shape::Rect(from, to) => {
                list.add_rect(from, to, colour).filled(true).build();
            }
            Shape::Tri(points) => {
                list.add_triangle(points[0], points[1], points[2], colour)
                    .filled(true)
                    .build();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn glyph_only(kind: ObjectiveType) -> Vec<Part> {
        parts(kind, [100.0, 100.0], 10.0)
            .into_iter()
            .filter(|part| part.ink == Ink::Glyph)
            .collect()
    }

    #[test]
    fn rf30_the_four_objectives_that_score_have_their_own_drawing() {
        let kinds = [
            ObjectiveType::Camp,
            ObjectiveType::Tower,
            ObjectiveType::Keep,
            ObjectiveType::Castle,
        ];
        for kind in kinds {
            assert!(has_glyph(kind), "{kind:?} puntúa y tiene dibujo");
        }
        for kind in [
            ObjectiveType::Ruins,
            ObjectiveType::Spawn,
            ObjectiveType::Mercenary,
            ObjectiveType::Unknown,
        ] {
            assert!(!has_glyph(kind), "{kind:?} no tiene dibujo propio");
        }
    }

    #[test]
    fn rf30_each_objective_is_told_apart_by_its_drawing_and_not_only_by_colour() {
        // Dos a dos distintos: forma y número de piezas tienen que bastar sin mirar el color.
        let kinds = [
            ObjectiveType::Camp,
            ObjectiveType::Tower,
            ObjectiveType::Keep,
            ObjectiveType::Castle,
        ];
        for (index, one) in kinds.iter().enumerate() {
            for other in &kinds[index + 1..] {
                assert_ne!(
                    glyph_only(*one).len(),
                    glyph_only(*other).len(),
                    "{one:?} y {other:?} no pueden dibujarse igual"
                );
            }
        }
    }

    #[test]
    fn rf30_a_tower_is_taller_than_a_camp_and_a_keep_wider_than_a_tower() {
        // Anchura total del dibujo: del extremo izquierdo al derecho de todas sus piezas.
        let bounds = |kind: ObjectiveType| {
            glyph_only(kind)
                .iter()
                .flat_map(|part| match part.shape {
                    Shape::Rect(from, to) => vec![from, to],
                    Shape::Tri(points) => points.to_vec(),
                })
                .fold(
                    ([f32::MAX; 2], [f32::MIN; 2]),
                    |(low, high): ([f32; 2], [f32; 2]), point| {
                        (
                            [low[0].min(point[0]), low[1].min(point[1])],
                            [high[0].max(point[0]), high[1].max(point[1])],
                        )
                    },
                )
        };
        let width = |kind: ObjectiveType| {
            let (low, high) = bounds(kind);
            high[0] - low[0]
        };
        let height = |kind: ObjectiveType| {
            let (low, high) = bounds(kind);
            high[1] - low[1]
        };
        assert!(
            width(ObjectiveType::Camp) > width(ObjectiveType::Tower),
            "la tienda es ancha y baja: {}, torre {}",
            width(ObjectiveType::Camp),
            width(ObjectiveType::Tower)
        );
        assert!(
            height(ObjectiveType::Tower) >= height(ObjectiveType::Camp),
            "la torre es más alta que la tienda"
        );
        assert!(
            width(ObjectiveType::Keep) > width(ObjectiveType::Tower),
            "la fortaleza es más ancha que la torre"
        );
        assert!(
            width(ObjectiveType::Castle) > width(ObjectiveType::Keep),
            "el castillo es el más ancho de todos"
        );
    }

    #[test]
    fn rf38_a_generic_kind_still_gets_a_mark() {
        let parts = parts(ObjectiveType::Ruins, [0.0, 0.0], 4.0);
        assert_eq!(parts.len(), 1, "una sola pieza, sin puerta");
        assert_eq!(parts[0].ink, Ink::Glyph);
    }

    #[test]
    fn rf38_the_glyph_is_centred_where_it_was_asked_to_be() {
        let centre = [50.0, 70.0];
        for kind in [
            ObjectiveType::Camp,
            ObjectiveType::Tower,
            ObjectiveType::Castle,
        ] {
            let bounds = parts(kind, centre, 8.0).iter().fold(
                ([f32::MAX; 2], [f32::MIN; 2]),
                |(mut low, mut high), part| {
                    let points: Vec<[f32; 2]> = match part.shape {
                        Shape::Rect(from, to) => vec![from, to],
                        Shape::Tri(points) => points.to_vec(),
                    };
                    for point in points {
                        low = [low[0].min(point[0]), low[1].min(point[1])];
                        high = [high[0].max(point[0]), high[1].max(point[1])];
                    }
                    (low, high)
                },
            );
            let (low, high) = bounds;
            assert!(
                (low[0] + high[0]) / 2.0 - centre[0] < 0.001,
                "{kind:?} descentrado en X"
            );
            assert!(
                (low[1] + high[1]) / 2.0 - centre[1] < 2.0,
                "{kind:?} descentrado en Y"
            );
        }
    }
}
