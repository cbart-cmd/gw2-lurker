//! Native timeline state tests without Nexus or a running game.

use super::*;
use crate::ui::map::{
    trail_tests::{claim, fixtures},
    FollowList,
};
use std::collections::BTreeMap;
use std::time::Duration;

/// History and fixture-backed catalogues shared by the test views.
struct Data {
    rows: Vec<super::super::ClaimRow>,
    names: Vec<super::super::ObjectiveInfo>,
    maps: Vec<super::super::WvwMap>,
    follow: FollowList,
    guild_names: BTreeMap<String, String>,
    prepared: Arc<Trail<'static>>,
}

impl Data {
    /// Normal sequence with unequal intervals and an official map catalogue.
    fn new() -> Self {
        let (names, maps) = fixtures();
        let mut data = Self {
            rows: vec![claim("95-32", 0), claim("95-33", 60), claim("95-37", 180)],
            names,
            maps,
            follow: FollowList::default(),
            guild_names: BTreeMap::new(),
            prepared: Arc::new(Trail::new(&[], "", &[], &[])),
        };
        data.refresh();
        data
    }

    /// Emulates preparation on the worker, not inside the UI view.
    fn refresh(&mut self) {
        self.prepared = Arc::new(Trail::new(&self.rows, "G", &self.names, &[]).into_owned());
    }

    /// Gives the cache a deterministic worker revision.
    fn view<'a>(&'a self, guild: Option<&'a str>, revision: i64) -> View<'a> {
        View {
            states: &[],
            names: &self.names,
            maps: &self.maps,
            current_map_id: 0,
            follow: &self.follow,
            guild_names: &self.guild_names,
            opacity: 1.0,
            real_map: false,
            guild,
            session_guilds: &[],
            revision: (
                Some(self.rows[0].claimed_at.unwrap() + chrono::TimeDelta::seconds(revision)),
                None,
            ),
            prepared: (guild == Some("G")).then_some(&self.prepared),
        }
    }
}

#[test]
fn rf36_estela_hidden_map_and_changed_guild_pause_playback() {
    let data = Data::new();
    let mut state = Session::default();
    state.synchronize(&data.view(Some("G"), 0));
    assert_eq!(state.player.selected, 2);
    state.player.toggle(Duration::ZERO, 3);
    state.drawn_frame = 10;
    state.pause_if_hidden(10);
    assert!(state.player.playing);
    state.pause_if_hidden(11);
    assert!(!state.player.playing);
    state.player.toggle(Duration::ZERO, 3);
    state.synchronize(&data.view(Some("other"), 0));
    assert!(!state.player.playing);
    assert!(state.trail.steps.is_empty());
    state.synchronize(&data.view(Some("G"), 0));
    assert_eq!(state.player.selected, 2);
    assert!(!state.player.playing);
}

#[test]
fn rf36_estela_mouse_selection_uses_time_and_keeps_unknown_times_separate() {
    let mut data = Data::new();
    data.rows.insert(
        0,
        super::super::ClaimRow {
            claimed_at: None,
            ..claim("95-32", 0)
        },
    );
    let trail = Trail::new(&data.rows, "G", &data.names, &[]);
    assert_eq!(pick_at(&trail, 0.0, false), Some(1));
    assert_eq!(pick_at(&trail, 1.0 / 3.0, false), Some(2));
    assert_eq!(pick_at(&trail, 1.0, false), Some(3));
    assert_eq!(pick_at(&trail, 0.4, true), Some(0));
    assert_eq!(trail.steps[0].timeline, None);
}

#[test]
fn rf36_estela_refresh_preserves_the_inspected_claim_and_reuses_geometry() {
    let mut data = Data::new();
    let mut state = Session::default();
    state.synchronize(&data.view(Some("G"), 0));
    state.player.choose(1, 3);
    data.rows.insert(0, claim("95-37", -60));
    data.refresh();
    state.synchronize(&data.view(Some("G"), 1));
    assert_eq!(state.player.selected, 2, "same claim, not the old index");
    let frame = super::super::Frame::of(data.maps.iter().find(|m| m.id == 95).unwrap());
    let canvas = RouteCanvas {
        frame: &frame,
        origin: [0.0, 0.0],
        size: [300.0, 420.0],
    };
    state.prepare_scene(&data.view(Some("G"), 1), 95, &canvas);
    let marker_count = state.scene.markers.len();
    state.scene.markers[0].label = "cached".into();
    state.prepare_scene(&data.view(Some("G"), 1), 95, &canvas);
    assert_eq!(state.scene.markers[0].label, "cached");
    assert_eq!(state.scene.markers.len(), marker_count);
    state.player.choose(3, 4);
    state.prepare_scene(&data.view(Some("G"), 1), 95, &canvas);
    assert_ne!(state.scene.markers[0].label, "cached");
}
