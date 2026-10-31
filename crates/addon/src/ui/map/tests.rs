//! Tests of the map tab: projection, hover, tiles and proportions.
//!
//! Kept out of `map.rs` so the tab stays under the 800-line limit (AGENTS.md §2.7).

use super::route::heading;
use super::*;

#[test]
fn rf30_the_map_keeps_the_real_proportions() {
    // Frontera alpina: 2560×3584 en `/v2/maps`, así que es 1,4 veces más alta que ancha.
    let alps = Frame {
        x: [0.0, 2560.0],
        y: [0.0, 3584.0],
    };
    let [w, h] = map_size(&alps, [400.0, 2000.0]);
    assert!((h / w - 1.4).abs() < 0.01, "sale {w}×{h}");

    // Eternos es cuadrada.
    let eternals = Frame {
        x: [8958.0, 12030.0],
        y: [12798.0, 15870.0],
    };
    let [w, h] = map_size(&eternals, [400.0, 2000.0]);
    assert!((h / w - 1.0).abs() < 0.01, "sale {w}×{h}");

    // Si no hay proporción que valga, el tamaño de siempre.
    let flat = Frame {
        x: [0.0, 100.0],
        y: [0.0, 100.0],
    };
    assert_eq!(map_size(&flat, [300.0, 1000.0]), [300.0, 300.0]);
}

#[test]
fn rf32_the_heading_is_the_last_jump_and_only_when_it_means_something() {
    // Hacia dónde va: la dirección del último salto, no el camino (ADR-005).
    let (ux, uy) = heading([0.0, 0.0], [100.0, 0.0]).expect("hay salto");
    assert!(
        (ux - 1.0).abs() < 0.001 && uy.abs() < 0.001,
        "hacia la derecha"
    );

    let (ux, uy) = heading([10.0, 10.0], [10.0, 110.0]).expect("hay salto");
    assert!(ux.abs() < 0.001 && (uy - 1.0).abs() < 0.001, "hacia abajo");

    assert_eq!(
        heading([10.0, 10.0], [14.0, 13.0]),
        None,
        "dos objetivos pegados no dan dirección"
    );
    assert_eq!(
        heading([10.0, 10.0], [10.0, 10.0]),
        None,
        "el mismo objetivo dos veces tampoco"
    );
}

#[test]
fn rf30_the_closest_objective_wins_the_hover() {
    let spots = [[10.0, 10.0], [100.0, 100.0], [20.0, 12.0]];

    assert_eq!(
        nearest(&spots, [11.0, 11.0], 12.0),
        Some(0),
        "el más cercano, no el primero"
    );
    assert_eq!(nearest(&spots, [95.0, 95.0], 12.0), Some(1));
    assert_eq!(
        nearest(&spots, [200.0, 200.0], 12.0),
        None,
        "lejos de todo no hay nada que enseñar"
    );
    assert_eq!(nearest(&[], [0.0, 0.0], 12.0), None);
}

#[test]
fn rf30_only_the_inside_of_the_board_has_hover() {
    let origin = [100.0, 50.0];
    let size = [200.0, 100.0];

    assert!(inside([150.0, 80.0], origin, size));
    assert!(inside([100.0, 50.0], origin, size), "la esquina cuenta");
    assert!(!inside([99.0, 80.0], origin, size), "a la izquierda, fuera");
    assert!(!inside([150.0, 151.0], origin, size), "por debajo, fuera");
}

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
