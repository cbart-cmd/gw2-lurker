//! Acceptance of the prepared B scene without calling Nexus.

use super::scene::*;
use super::*;
use chrono::{TimeZone, Utc};
use gw2lurker_core::route::trail::{Trail, Window};
use gw2lurker_core::Event;

/// Real catalogues used for map projection and objective coordinates.
pub(super) fn fixtures() -> (Vec<ObjectiveInfo>, Vec<WvwMap>) {
    let names = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../api/tests/fixtures/objectives_all.json"
    )))
    .unwrap();
    let maps = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../api/tests/fixtures/maps_wvw.json"
    )))
    .unwrap();
    (names, maps)
}

/// Synthetic claim whose coordinates come from the real objective table.
pub(super) fn claim(id: &str, seconds: i64) -> ClaimRow {
    let map_id = id.split_once('-').unwrap().0.parse().unwrap();
    ClaimRow {
        map_id,
        map_role: MapRole::from_map_id(map_id),
        objective_id: id.into(),
        name: Some(id.into()),
        kind: ObjectiveType::Keep,
        owner: Some(TeamColor::Green),
        guild: "G".into(),
        claimed_at: Some(Utc.timestamp_opt(1_800_000_000 + seconds, 0).unwrap()),
        points_tick: 10,
    }
}

#[test]
fn rf32_estela_curves_preserve_endpoints_and_show_separate_return_paths() {
    let outward = Curve::between([0.0, 0.0], [100.0, 0.0], 0);
    let returning = Curve::between([100.0, 0.0], [0.0, 0.0], 1);
    assert_eq!(outward.point(0.0), [0.0, 0.0]);
    assert_eq!(outward.point(1.0), [100.0, 0.0]);
    assert!(
        outward.point(0.5)[1] * returning.point(0.5)[1] < 0.0,
        "A to B and B to A must not overlap"
    );
}

#[test]
fn rf32_estela_six_cases_keep_claims_maps_and_holes() {
    let (names, maps) = fixtures();
    let frame = Frame::of(maps.iter().find(|m| m.id == 95).unwrap());
    let canvas = RouteCanvas {
        frame: &frame,
        origin: [0.0, 0.0],
        size: [300.0, 420.0],
    };
    let cases = [
        vec![claim("95-32", 0), claim("95-33", 600), claim("95-37", 1200)],
        vec![claim("95-32", 0), claim("95-33", 80), claim("95-32", 160)],
        vec![claim("95-32", 0)],
        vec![claim("95-32", 0), claim("95-33", 50), claim("95-37", 100)],
        (0..28)
            .map(|i| {
                claim(
                    if i % 2 == 0 { "95-32" } else { "95-33" },
                    i * 120 + if i >= 14 { 2700 } else { 0 },
                )
            })
            .collect(),
        vec![
            claim("95-32", 0),
            claim("95-missing", 60),
            claim("96-32", 1400),
            claim("95-33", 3200),
        ],
    ];
    for (case, rows) in cases.iter().enumerate() {
        let trail = Trail::new(rows, "G", &names, &[]);
        let view = trail.view(rows.len() - 1, Window::All);
        let scene = Scene::prepare(&trail, &view, 95, &canvas);
        assert_eq!(trail.steps.len(), rows.len(), "case {case}");
        assert!(scene.markers.iter().all(|m| !m.label.is_empty()));
        assert!(scene
            .curves
            .iter()
            .all(|c| c.curve.point(0.0).iter().all(|n| n.is_finite())));
        match case {
            0 | 3 => {
                assert_eq!(scene.curves.len(), 2);
                assert!(scene.curves[0].width < scene.curves[1].width);
                assert!(scene.curves[0].colour[3] < scene.curves[1].colour[3]);
            }
            1 => {
                assert_eq!(scene.markers.len(), 2);
                assert!(scene.markers.iter().any(|m| m.label == "1/3"));
            }
            2 => assert!(scene.curves.is_empty()),
            4 => {
                assert!(trail.legs.iter().any(|l| l.no_claims));
                assert_eq!(scene.markers.len(), 2);
            }
            5 => assert!(
                scene.curves.is_empty(),
                "never jump over missing data or another map"
            ),
            _ => unreachable!(),
        }
    }
}

#[test]
fn rf33_estela_keeps_late_gap_and_future_styles() {
    let (names, maps) = fixtures();
    let rows = [claim("95-32", 0), claim("95-33", 60), claim("95-37", 120)];
    let events = [
        Event::Claim {
            objective: "95-33".into(),
            guild: "G".into(),
            at: rows[1].claimed_at.unwrap(),
            late: true,
        },
        Event::Gap {
            from: rows[0].claimed_at.unwrap(),
            to: rows[1].claimed_at.unwrap(),
        },
    ];
    let trail = Trail::new(&rows, "G", &names, &events);
    let frame = Frame::of(maps.iter().find(|m| m.id == 95).unwrap());
    let canvas = RouteCanvas {
        frame: &frame,
        origin: [0.0, 0.0],
        size: [300.0, 420.0],
    };
    let scene = Scene::prepare(&trail, &trail.view(1, Window::All), 95, &canvas);
    assert!(scene.curves.iter().all(|c| c.dashed));
    assert!(scene.markers.iter().any(|m| m.future && m.label == "3"));
    assert!(scene.markers.iter().any(|m| m.current && m.late));
}

#[test]
fn rnf01_estela_native_draw_budget() {
    // arcdps-imgui-sys 0.8 build.rs compiles its own cimgui. Context::create uses it directly;
    // this exercises the real draw list without Nexus, AddonApi, the game or a graphics driver.
    use nexus::imgui::{Condition, Context, Window as NativeWindow};
    use std::time::Instant;
    let _imgui_guard = crate::ui::IMGUI_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let (names, maps) = fixtures();
    let frame = Frame::of(maps.iter().find(|m| m.id == 95).unwrap());
    let canvas = RouteCanvas {
        frame: &frame,
        origin: [20.0, 40.0],
        size: [400.0, 560.0],
    };
    let mut context = Context::create();
    context.set_ini_filename(None);
    context.set_log_filename(None);
    context.io_mut().display_size = [640.0, 700.0];
    context.io_mut().delta_time = 1.0 / 60.0;
    context.io_mut().mouse_pos = [-100.0, -100.0];
    context.fonts().build_rgba32_texture();
    let catalogue: Vec<_> = names
        .iter()
        .filter(|n| n.map_id == 95 && n.coord.is_some())
        .collect();
    for (count, dense) in [(28, false), (2000, false), (2000, true)] {
        let mut seed = 42_u32;
        let rows: Vec<_> = (0..count)
            .map(|i| {
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                let id = if dense {
                    catalogue[seed as usize % catalogue.len()].id.as_str()
                } else if i % 2 == 0 {
                    "95-32"
                } else {
                    "95-33"
                };
                claim(id, i as i64 * 60)
            })
            .collect();
        let trail = Trail::new(&rows, "G", &names, &[]).into_owned();
        let view = trail.view(count - 1, Window::All);
        let scene = Scene::prepare(&trail, &view, 95, &canvas);
        let mut state = timeline::Session::default();
        state.trail = std::sync::Arc::new(trail);
        let mut samples = Vec::new();
        let mut vertices = 0;
        for iteration in 0..50 {
            let ui = context.frame();
            let start = Instant::now();
            NativeWindow::new("route benchmark")
                .position([0.0, 0.0], Condition::Always)
                .size([640.0, 700.0], Condition::Always)
                .build(&ui, || {
                    timeline::track(&ui, &mut state);
                    let list = ui.get_window_draw_list();
                    for marker in &scene.markers {
                        draw_objective(
                            &list,
                            marker.point,
                            ObjectiveType::Keep,
                            Some(TeamColor::Red),
                            true,
                            true,
                        );
                    }
                    draw_route(&list, &scene);
                });
            let rendered = ui.render();
            let elapsed = start.elapsed().as_secs_f64() * 1000.0;
            assert!(rendered.total_vtx_count > 0);
            assert!(
                rendered.total_vtx_count < 65_536,
                "16-bit draw-list index budget"
            );
            vertices = rendered.total_vtx_count;
            if iteration >= 5 {
                samples.push(elapsed);
            }
        }
        samples.sort_by(f64::total_cmp);
        let p95 = samples[samples.len() * 95 / 100];
        println!("RNF-01 native Estela draw {count} claims (dense={dense}, {} arcs, {vertices} vertices): p95={p95:.3} ms", scene.curves.len());
        if !cfg!(debug_assertions) {
            assert!(
                p95 < 0.5,
                "native route drawing alone exceeds RNF-01: {p95:.3} ms"
            );
        }
    }
}
