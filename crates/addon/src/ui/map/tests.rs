//! Tests of the map tab: projection, hover, tiles and proportions.
//!
//! Kept out of `map.rs` so the tab stays under the 800-line limit (AGENTS.md §2.7).

use super::route::heading;
use super::route::route_steps;
use super::*;
use gw2lurker_core::project::{tiles_of, TILE_ZOOM};

#[test]
fn rf32a_the_drawn_route_includes_zero_and_keeps_claim_numbering() {
    use super::route::RouteCanvas;
    use super::scene::Scene;
    use gw2lurker_core::route::trail::{Trail, Window};
    let names: Vec<ObjectiveInfo> = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../api/tests/fixtures/objectives_all.json"
    )))
    .unwrap();
    let maps: Vec<WvwMap> = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../api/tests/fixtures/maps_wvw.json"
    )))
    .unwrap();
    let frame = Frame::of(maps.iter().find(|map| map.id == 95).unwrap());
    let garrison = names
        .iter()
        .find(|name| name.id == "95-37")
        .unwrap()
        .coord
        .unwrap();
    let canvas = RouteCanvas {
        frame: &frame,
        origin: [20.0, 30.0],
        size: [300.0, 420.0],
    };
    let step = |id: &str| gw2lurker_core::RouteStep {
        objective_id: id.to_owned().into(),
        name: id.to_owned().into(),
        at: Some(chrono::Utc::now()),
    };
    let mut route = gw2lurker_core::Route {
        origin: gw2lurker_core::RouteOrigin::Garrison,
        origin_point: Some(gw2lurker_core::route::OriginPoint {
            objective_id: "95-37".to_owned(),
            position: [garrison[0], garrison[1]],
            approximate: false,
        }),
        steps: vec![step("95-32"), step("95-33")],
    };
    let claim = |id: &str| ClaimRow {
        map_id: 95,
        map_role: MapRole::GreenHome,
        objective_id: id.into(),
        name: None,
        kind: ObjectiveType::Keep,
        owner: Some(TeamColor::Green),
        guild: "G".into(),
        claimed_at: None,
        points_tick: 10,
    };
    let rows = [claim("95-32"), claim("95-33")];
    let trail = Trail::new(&rows, "G", &names, &[]);
    let mut scene = Scene::prepare(&trail, &trail.view(0, Window::All), 95, &canvas);
    scene.attach_origin(&route, &rows[..1], 95, &canvas);
    assert!(
        scene.origin_link.is_some(),
        "zero connects to the first claim"
    );
    assert_eq!(
        scene.origin,
        project(
            &frame,
            [garrison[0], garrison[1]],
            canvas.origin,
            canvas.size
        )
    );
    assert_eq!(scene.markers[0].label, "1");
    route.steps = vec![step("95-37")];
    let rows = [claim("95-37")];
    let trail = Trail::new(&rows, "G", &names, &[]);
    let mut scene = Scene::prepare(&trail, &trail.view(0, Window::All), 95, &canvas);
    scene.attach_origin(&route, &rows, 95, &canvas);
    assert_eq!(scene.markers.len(), 1);
    assert_eq!(scene.markers[0].label, "0/1");
    assert!(scene.origin_link.is_none());
    let trail = Trail::new(&[], "G", &names, &[]);
    let mut scene = Scene::prepare(&trail, &trail.view(0, Window::All), 95, &canvas);
    scene.attach_origin(&route, &[], 95, &canvas);
    assert!(scene.origin.is_none(), "no orphaned zero");
}

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
fn p05_seared_borderlands_static_objectives_fit_the_verified_map_box() {
    let maps: Vec<WvwMap> = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../api/tests/fixtures/maps_wvw.json"
    )))
    .unwrap();
    let objectives: Vec<ObjectiveInfo> = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../api/tests/fixtures/objectives_seared_borderlands.json"
    )))
    .unwrap();
    let map = maps.iter().find(|map| map.id == 1652).unwrap();
    let frame = Frame::of(map);
    let static_objectives: Vec<_> = objectives
        .iter()
        .filter(|objective| objective.map_id == map.id)
        .collect();
    let located: Vec<_> = static_objectives
        .iter()
        .filter_map(|objective| objective.coord.map(|coord| (*objective, coord)))
        .collect();

    assert_eq!(map.name.trim(), "Seared Borderlands");
    assert_eq!(static_objectives.len(), 21);
    assert_eq!(located.len(), 18);
    for (objective, coord) in &located {
        assert!(
            frame.normalise([coord[0], coord[1]]).is_some(),
            "{} must be inside the map projection",
            objective.id
        );
    }
    assert_eq!(
        located
            .iter()
            .filter(|(objective, _)| is_drawable(objective.kind))
            .count(),
        13
    );
    assert_eq!(tiles_of(&frame, TILE_ZOOM).len(), 20);
    assert!(!has_live_state_for_map(&[], map.id));
}

#[test]
fn p05_seared_borderlands_map_id_supports_all_three_mumble_roles() {
    let maps: Vec<WvwMap> = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../api/tests/fixtures/maps_wvw.json"
    )))
    .unwrap();
    let matchup: gw2lurker_core::Match = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../api/tests/fixtures/match_2-2.json"
    )))
    .unwrap();
    let states = gw2lurker_core::matchup::states(&matchup);
    assert!(states.iter().any(|state| state.map_id == 1099));
    assert!(!has_live_state_for_map(&states, 1652));

    for (map_type, role) in [
        (10, MapRole::BlueHome),
        (11, MapRole::GreenHome),
        (12, MapRole::RedHome),
    ] {
        let current_role = MapRole::from_mumble(1652, map_type);
        assert_eq!(current_role, role);
        assert_eq!(
            map_for_role(&maps, &states, role, Some(role), 1652).map(|map| map.id),
            Some(1652)
        );
    }
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

#[test]
fn rf30b_missing_objective_state_does_not_get_a_high_value_ring() {
    assert!(!has_high_value_ring(None));
    let state = ObjectiveState {
        map_id: 38,
        map_role: MapRole::Center,
        objective_id: "38-20".to_owned(),
        kind: ObjectiveType::Keep,
        owner: Some(TeamColor::Blue),
        guild: None,
        claimed_at: None,
        points_tick: 12,
    };
    assert!(has_high_value_ring(Some(&state)));
}

#[test]
fn rf30_the_unknown_map_defaults_to_center_before_drawing_the_selector() {
    assert_eq!(resolved_map_role(MapRole::Unknown, None), MapRole::Center);
    assert_eq!(
        resolved_map_role(MapRole::Unknown, Some(MapRole::RedHome)),
        MapRole::RedHome
    );
    assert_eq!(
        resolved_map_role(MapRole::BlueHome, Some(MapRole::RedHome)),
        MapRole::BlueHome
    );
}
#[test]
fn rf30_the_hover_says_who_and_when_and_leaves_the_name_out() {
    // Petición del jugador (2026-11-01): el nombre del sitio no hace falta (ya sabe dónde está) y
    // alargaba la caja hasta tapar los objetivos vecinos. El alto valor, al final.
    let state = ObjectiveState {
        map_id: 38,
        map_role: MapRole::Center,
        objective_id: "38-20".to_owned(),
        kind: ObjectiveType::Keep,
        owner: Some(TeamColor::Blue),
        guild: Some("6C45F3C8".to_owned()),
        claimed_at: Some(
            chrono::DateTime::parse_from_rfc3339("2026-11-01T12:31:00Z")
                .expect("fecha")
                .with_timezone(&chrono::Utc),
        ),
        points_tick: 12,
    };
    let mut names = BTreeMap::new();
    names.insert("6C45F3C8".to_owned(), "[YaK] Dolyak Sanctuary".to_owned());
    let mut follow = FollowList::default();
    follow.add("6C45F3C8");

    use gw2lurker_core::i18n::{t, Language};
    let line = detail_line(Some(&state), &names, &follow, Language::Es);
    assert!(
        !line.contains("Palacio"),
        "el nombre del sitio no va en el hover: {line}"
    );

    // Orden: dueño, clan, hora, y «Alto valor» al final.
    let parts: Vec<&str> = line.split("  ·  ").collect();
    assert_eq!(parts.len(), 4, "{line}");
    assert_eq!(parts[0], t(Language::Es, "Blue"));
    assert_eq!(parts[1], "* [YaK] Dolyak Sanctuary");
    assert_eq!(
        parts[2],
        crate::clock::clock(state.claimed_at.expect("hora"))
    );
    assert_eq!(
        parts[3],
        t(Language::Es, "High value"),
        "el alto valor va el último"
    );
}

#[test]
fn rf30_the_hover_without_a_claim_only_says_unclaimed() {
    use gw2lurker_core::i18n::{t, Language};
    let line = detail_line(None, &BTreeMap::new(), &FollowList::default(), Language::Es);
    assert_eq!(line, t(Language::Es, "Unclaimed"));
}

#[test]
fn rf30b_an_objective_owned_without_claim_keeps_its_colour_on_the_hover() {
    // Informe del jugador (2026-11-01): «Puesto avanzado abrasado» pasó a rojo y el mapa seguía
    // diciendo «sin reclamar». El dueño no sale del reclamo, sale del objetivo.
    use gw2lurker_core::i18n::{t, Language};
    let state = ObjectiveState {
        map_id: 38,
        map_role: MapRole::Center,
        objective_id: "38-6".to_owned(),
        kind: ObjectiveType::Camp,
        owner: Some(TeamColor::Red),
        guild: None,
        claimed_at: None,
        points_tick: 2,
    };
    let line = detail_line(
        Some(&state),
        &BTreeMap::new(),
        &FollowList::default(),
        Language::Es,
    );
    assert_eq!(
        line,
        format!(
            "{}  ·  {}",
            t(Language::Es, "Red"),
            t(Language::Es, "Unclaimed")
        ),
        "un objetivo rojo sin reclamar se lee «Rojo · Sin reclamar», no sólo «Sin reclamar»"
    );
}
#[test]
fn rf31_the_guild_menu_is_the_followed_ones_and_then_the_latest_five() {
    // Petición del jugador (2026-11-01): en la pestaña Mapa, los clanes que sigue y justo después
    // los últimos cinco que han reclamado algo. `session` llega ordenada por reclamo más reciente.
    let mut follow = FollowList::default();
    follow.add("SEGUIDO1");
    follow.add("SEGUIDO2");
    let session: Vec<(String, usize, Option<chrono::DateTime<chrono::Utc>>)> = vec![
        ("RECIENTE1".to_owned(), 3, None),
        ("SEGUIDO1".to_owned(), 2, None),
        ("RECIENTE2".to_owned(), 1, None),
        ("RECIENTE3".to_owned(), 1, None),
        ("RECIENTE4".to_owned(), 1, None),
        ("RECIENTE5".to_owned(), 1, None),
        ("VIEJO".to_owned(), 9, None),
    ];

    let choices = guild_choices(&follow, &session, GUILDS_IN_MENU);
    assert_eq!(
        choices,
        vec![
            "SEGUIDO1", // los seguidos primero, en el orden en que se añadieron
            "SEGUIDO2",
            "RECIENTE1", // y después los últimos, sin repetir el que ya salía arriba
            "RECIENTE2",
            "RECIENTE3",
            "RECIENTE4",
            "RECIENTE5",
        ],
        "el sexto más reciente se queda fuera: son cinco"
    );
    assert!(
        !choices.iter().any(|guild| guild == "VIEJO"),
        "un clan viejo con muchos claims no entra por número de claims: manda la hora"
    );
}

#[test]
fn rf31_the_guild_menu_without_anything_to_offer_is_empty() {
    let choices = guild_choices(&FollowList::default(), &[], GUILDS_IN_MENU);
    assert!(choices.is_empty(), "{choices:?}");
}

#[test]
fn rf35_a_route_that_crosses_maps_says_jump_to() {
    // Tres pasos, el último en otro mapa: entre el segundo y el tercero tiene que aparecer el
    // salto (RF-35). El resumen se pinta ahora junto al mapa y no debajo de la lista de clanes
    // (petición del jugador, 2026-11-01), así que el test vive aquí.
    let row = |objective_id: &str, role: MapRole| ClaimRow {
        map_id: 38,
        map_role: role,
        objective_id: objective_id.to_owned(),
        name: None,
        kind: ObjectiveType::Tower,
        owner: None,
        guild: "AAAA".to_owned(),
        claimed_at: None,
        points_tick: 0,
    };
    let rows = vec![
        row("38-10", MapRole::Center),
        row("38-20", MapRole::Center),
        row("96-30", MapRole::BlueHome),
    ];
    let step = |id: &str| gw2lurker_core::route::Step {
        name: id.to_owned().into(),
        objective_id: id.to_owned().into(),
        at: None,
    };
    let route = gw2lurker_core::Route {
        origin: gw2lurker_core::RouteOrigin::Spawn,
        origin_point: None,
        steps: vec![step("38-10"), step("38-20"), step("96-30")],
    };

    let text = route_steps(&route, &rows, gw2lurker_core::i18n::Language::Es);
    assert_eq!(text.matches("38-10").count(), 1, "{text}");
    assert!(
        text.contains("salto a"),
        "el cambio de mapa se anuncia: {text}"
    );
    let jump = text.find("salto a").expect("hay salto");
    assert!(
        jump > text.find("38-20").expect("paso 2") && jump < text.find("96-30").expect("paso 3"),
        "el salto va entre los dos pasos: {text}"
    );
}

#[test]
fn rf32_the_cross_map_summary_lists_claims_without_inventing_an_origin() {
    let row = |objective_id: &str, at: &str| ClaimRow {
        map_id: 38,
        map_role: MapRole::Center,
        objective_id: objective_id.to_owned(),
        name: Some(objective_id.to_owned()),
        kind: ObjectiveType::Camp,
        owner: None,
        guild: "AAAA".to_owned(),
        claimed_at: Some(
            chrono::DateTime::parse_from_rfc3339(at)
                .expect("fecha")
                .with_timezone(&chrono::Utc),
        ),
        points_tick: 2,
    };
    let rows = vec![
        row("38-1", "2026-11-01T10:00:00Z"),
        row("38-2", "2026-11-01T10:05:00Z"),
    ];

    use gw2lurker_core::i18n::Language;
    assert_eq!(
        summary(&[], "AAAA", Language::Es),
        None,
        "sin reclamos no hay ruta que contar"
    );
    let line = summary(&rows, "AAAA", Language::Es).expect("hay ruta");
    assert_eq!(
        line, "38-1 -> 38-2",
        "the located origin is captioned on its map, not invented from claim types"
    );
}

#[test]
#[cfg(not(debug_assertions))]
#[ignore = "manual optimized CPU measurement; full render timing requires GW2"]
fn rnf01_origin_route_preparation_cpu_budget() {
    use super::route::RouteCanvas;
    use super::scene::Scene;
    use gw2lurker_core::route::trail::{Trail, Window};
    let names: Vec<ObjectiveInfo> = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../api/tests/fixtures/objectives_all.json"
    )))
    .unwrap();
    let maps: Vec<WvwMap> = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../api/tests/fixtures/maps_wvw.json"
    )))
    .unwrap();
    let entry: gw2lurker_core::Match = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../api/tests/fixtures/match_2-2.json"
    )))
    .unwrap();
    let states = gw2lurker_core::matchup::states(&entry);
    let base: Vec<_> = gw2lurker_core::matchup::claims(&entry, &names)
        .into_iter()
        .filter(|row| row.map_id == 95)
        .collect();
    assert!(!base.is_empty());
    let rows: Vec<_> = (0..crate::worker::MAX_HISTORY)
        .map(|index| {
            let mut row = base[index % base.len()].clone();
            row.guild = "BENCH".to_owned();
            row
        })
        .collect();
    let frame = Frame::of(maps.iter().find(|map| map.id == 95).unwrap());
    let canvas = RouteCanvas {
        frame: &frame,
        origin: [0.0, 0.0],
        size: [400.0, 560.0],
    };
    let trail = Trail::new(&rows, "BENCH", &names, &[]);
    let mut times = Vec::new();
    for _ in 0..128 {
        let start = std::time::Instant::now();
        let view = trail.view(rows.len() - 1, Window::All);
        let borrowed: Vec<_> = trail.steps.iter().map(|s| s.claim.as_ref()).collect();
        let (origin, origin_point) =
            gw2lurker_core::route::locate_origin(&borrowed, "BENCH", &states, &names);
        let route = gw2lurker_core::Route {
            origin,
            origin_point,
            steps: Vec::new(),
        };
        let mut scene = Scene::prepare(&trail, &view, 95, &canvas);
        scene.attach_origin(&route, borrowed.iter().copied(), 95, &canvas);
        std::hint::black_box(scene);
        times.push(start.elapsed());
    }
    times.sort_unstable();
    let p95 = times[121];
    println!(
        "RNF-01 CPU preparation for {} claims: median={:.3} ms, p95={:.3} ms",
        rows.len(),
        times[64].as_secs_f64() * 1000.0,
        p95.as_secs_f64() * 1000.0
    );
    assert!(
        p95 < std::time::Duration::from_micros(500),
        "CPU preparation alone exceeds RNF-01"
    );
}
