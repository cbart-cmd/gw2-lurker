//! Session cache and native timeline controls for B - Estela (T-53).

use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::Instant;

use chrono::{DateTime, Utc};
use gw2lurker_core::route::{
    playback::Playback,
    trail::{Trail, Visibility, Window},
};
use nexus::imgui::{ComboBox, Key, Selectable, Slider, Ui};

use super::{i18n, route::RouteCanvas, scene::Scene, View};

/// Inputs that affect the prepared geometry but not its cached source observations.
#[derive(Debug, PartialEq)]
struct SceneKey {
    map: u32,
    origin: [f32; 2],
    size: [f32; 2],
    x: [f64; 2],
    y: [f64; 2],
    selected: usize,
    window: Window,
}

/// Session-only state. No configuration mutex or worker lock is held by this cache.
#[derive(Debug)]
pub(super) struct Session {
    guild: Option<String>,
    revision: (Option<DateTime<Utc>>, Option<DateTime<Utc>>),
    pub trail: Arc<Trail<'static>>,
    player: Playback,
    window: Window,
    clock: Instant,
    drawn_frame: i32,
    scene_key: Option<SceneKey>,
    pub scene: Scene,
    pub origin: Option<gw2lurker_core::Route<'static>>,
}

impl Default for Session {
    /// Starts empty and paused, with no prepared geometry or persistent playback settings.
    fn default() -> Self {
        Self {
            guild: None,
            revision: (None, None),
            trail: Arc::new(Trail::new(&[], "", &[], &[])),
            player: Playback::default(),
            window: Window::All,
            clock: Instant::now(),
            drawn_frame: -1,
            scene_key: None,
            scene: Scene::default(),
            origin: None,
        }
    }
}

/// Locks only the map's own session state, never RUNTIME.
fn session() -> MutexGuard<'static, Session> {
    static SESSION: OnceLock<Mutex<Session>> = OnceLock::new();
    SESSION
        .get_or_init(|| Mutex::new(Session::default()))
        .lock()
        .unwrap_or_else(|error| error.into_inner())
}

/// Pauses even when the widget is collapsed, closed, outside WvW or not on the Map tab.
pub(crate) fn pause_unless_drawn(frame: i32) {
    let mut state = session();
    state.pause_if_hidden(frame);
}

/// Updates the source cache only on a worker revision or guild/cycle change, then draws controls.
pub(super) fn prepare<'a>(ui: &Ui, data: &View<'a>) -> MutexGuard<'static, Session> {
    let mut state = session();
    state.synchronize(data);
    state.drawn_frame = ui.frame_count();
    let now = state.clock.elapsed();
    let count = state.trail.steps.len();
    state.player.tick(now, count);
    if data.guild.is_some() {
        controls(ui, &mut state);
    }
    state
}

impl Session {
    /// Visibility rule shared by tab changes, collapsed windows and the WvW render gate.
    fn pause_if_hidden(&mut self, frame: i32) {
        if self.drawn_frame != frame {
            self.player.pause();
        }
    }

    /// Refreshes history without jumping away from a step the player is inspecting.
    fn synchronize(&mut self, data: &View<'_>) {
        if self.guild.as_deref() == data.guild && self.revision == data.revision {
            return;
        }
        let changed_guild = self.guild.as_deref() != data.guild;
        let changed_cycle = self.revision.1 != data.revision.1;
        let latest = self.player.selected == self.trail.steps.len().saturating_sub(1)
            && !self.player.playing;
        let previous = self.trail.steps.get(self.player.selected).map(|s| {
            (
                s.claim.objective_id.clone(),
                s.claim.claimed_at,
                s.claim.map_id,
            )
        });
        self.trail = data
            .prepared
            .cloned()
            .unwrap_or_else(|| Arc::new(Trail::new(&[], "", &[], &[])));
        self.guild = data.guild.map(str::to_owned);
        self.revision = data.revision;
        let count = self.trail.steps.len();
        if changed_guild || changed_cycle || latest {
            self.player.reset(count);
        } else if let Some(index) = self.trail.steps.iter().position(|s| {
            previous.as_ref().is_some_and(|(id, at, map)| {
                *id == s.claim.objective_id && *at == s.claim.claimed_at && *map == s.claim.map_id
            })
        }) {
            self.player.selected = index;
        } else {
            self.player.choose(self.player.selected, count);
        }
        self.scene_key = None;
    }

    /// Caches projection and origin inference until data, viewport or selection changes.
    pub fn prepare_scene(&mut self, data: &View<'_>, map: u32, canvas: &RouteCanvas<'_>) {
        let key = SceneKey {
            map,
            origin: canvas.origin,
            size: canvas.size,
            x: canvas.frame.x,
            y: canvas.frame.y,
            selected: self.player.selected,
            window: self.window,
        };
        if self.scene_key.as_ref() == Some(&key) {
            return;
        }
        let visibility = self.trail.view(self.player.selected, self.window);
        self.scene = Scene::prepare(&self.trail, &visibility, map, canvas);
        let rows: Vec<_> = self
            .trail
            .steps
            .iter()
            .zip(&visibility.states)
            .filter(|(_, s)| {
                matches!(
                    s,
                    Visibility::Current | Visibility::Active | Visibility::UnknownTime
                )
            })
            .map(|(s, _)| s.claim.as_ref())
            .filter(|r| r.map_id == map)
            .collect();
        self.origin = data.guild.filter(|_| !rows.is_empty()).map(|guild| {
            let (origin, origin_point) =
                gw2lurker_core::route::locate_origin(&rows, guild, data.states, data.names);
            gw2lurker_core::Route {
                origin,
                origin_point,
                steps: Vec::new(),
            }
        });
        if let Some(origin) = &self.origin {
            self.scene
                .attach_origin(origin, rows.iter().copied(), map, canvas);
        }
        self.scene_key = Some(key);
    }
}

/// Temporal position, or a separately labelled lane for observations with unknown times.
fn position(trail: &Trail<'_>, index: usize) -> (f32, bool) {
    match trail.steps[index].timeline {
        Some(t) => (t as f32, false),
        None => {
            let count = trail.steps.partition_point(|s| s.timeline.is_none());
            (
                if count <= 1 {
                    0.5
                } else {
                    index as f32 / (count - 1) as f32
                },
                true,
            )
        }
    }
}

/// Nearest observation in the chosen time lane; ties keep the stable step order.
fn pick_at(trail: &Trail<'_>, x: f32, unknown: bool) -> Option<usize> {
    (0..trail.steps.len())
        .filter(|&i| trail.steps[i].timeline.is_none() == unknown)
        .min_by(|&a, &b| {
            (position(trail, a).0 - x)
                .abs()
                .total_cmp(&(position(trail, b).0 - x).abs())
        })
}

/// Native controls precede the map so their draw list is dropped before drawing the board.
fn controls(ui: &Ui, state: &mut Session) {
    ui.text_wrapped(i18n::text(
        "Route inferred from claims; it is not the real path.",
    ));
    let count = state.trail.steps.len();
    if count == 0 {
        ui.text_wrapped(i18n::text("No claims recorded for this guild yet."));
        return;
    }
    if ui.button(i18n::widget(if state.player.playing {
        "Pause route"
    } else {
        "Play route"
    })) {
        state.player.toggle(state.clock.elapsed(), count);
    }
    ui.same_line();
    let label = window_label(state.window);
    ComboBox::new(i18n::widget("Trail window"))
        .preview_value(i18n::text(label))
        .build(ui, || {
            for value in [Window::All, Window::Hour, Window::Minutes15] {
                if Selectable::new(i18n::text(window_label(value)))
                    .selected(value == state.window)
                    .build(ui)
                {
                    state.window = value;
                }
            }
        });
    track(ui, state);
    let mut chosen = state.player.selected as i32 + 1;
    let slider_changed =
        Slider::new(i18n::widget("Route step"), 1, count as i32).build(ui, &mut chosen);
    if slider_changed {
        state
            .player
            .choose(chosen.saturating_sub(1) as usize, count);
    }
    if ui.is_item_focused() || ui.is_item_active() {
        let next = if ui.is_key_pressed(Key::Home) {
            Some(0)
        } else if ui.is_key_pressed(Key::End) {
            Some(count - 1)
        } else if !slider_changed
            && (ui.is_key_pressed(Key::LeftArrow) || ui.is_key_pressed(Key::DownArrow))
        {
            Some(state.player.selected.saturating_sub(1))
        } else if !slider_changed
            && (ui.is_key_pressed(Key::RightArrow) || ui.is_key_pressed(Key::UpArrow))
        {
            Some((state.player.selected + 1).min(count - 1))
        } else {
            None
        };
        if let Some(index) = next {
            state.player.choose(index, count);
        }
    }
    let step = &state.trail.steps[state.player.selected];
    let time = step
        .claim
        .claimed_at
        .map(crate::clock::clock_seconds)
        .unwrap_or_else(|| i18n::text("Unknown time").to_owned());
    ui.text_wrapped(i18n::format(
        "Step {0} of {1}: {2} - {3}",
        &[
            &(state.player.selected + 1).to_string(),
            &count.to_string(),
            &time,
            step.claim
                .name
                .as_deref()
                .unwrap_or(&step.claim.objective_id),
        ],
    ));
    if let Some(previous) = state.player.selected.checked_sub(1) {
        if let Some(dt) = state.trail.legs[previous].elapsed {
            ui.text_disabled(i18n::format(
                "{0} since the previous claim",
                &[&duration(dt)],
            ));
        }
    }
}

/// Translation key for each non-persistent trail window.
fn window_label(window: Window) -> &'static str {
    match window {
        Window::All => "All available history",
        Window::Hour => "Last hour",
        Window::Minutes15 => "Last 15 minutes",
    }
}

/// Interval formatting, shared by timeline and route tooltips.
pub(super) fn duration(dt: chrono::TimeDelta) -> String {
    let seconds = dt.num_seconds().max(0);
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

/// Real-time axis plus an explicit separate lane for unknown timestamps.
/// Draws the real temporal axis, kept separate from localized controls for native measurements.
pub(super) fn track(ui: &Ui, state: &mut Session) {
    let origin = ui.cursor_screen_pos();
    let width = ui.content_region_avail()[0].max(80.0);
    let unknown = state
        .trail
        .steps
        .first()
        .is_some_and(|s| s.timeline.is_none());
    let height = if unknown { 48.0 } else { 28.0 };
    ui.invisible_button("###trail_time_axis", [width, height]);
    let mouse = ui.io().mouse_pos;
    let hovered = ui.is_item_hovered();
    if ui.is_item_active() {
        if let Some(index) = pick_at(
            &state.trail,
            ((mouse[0] - origin[0] - 6.0) / (width - 12.0)).clamp(0.0, 1.0),
            unknown && mouse[1] > origin[1] + 24.0,
        ) {
            state.player.choose(index, state.trail.steps.len());
        }
    }
    let list = ui.get_window_draw_list();
    let p = |i| {
        let (x, missing) = position(&state.trail, i);
        [
            origin[0] + 6.0 + x * (width - 12.0),
            origin[1] + if missing { 36.0 } else { 12.0 },
        ]
    };
    list.add_line(
        [origin[0] + 6.0, origin[1] + 12.0],
        [origin[0] + width - 6.0, origin[1] + 12.0],
        [0.6, 0.65, 0.7, 0.8],
    )
    .build();
    let mut hint = None;
    for leg in &state.trail.legs {
        let a = p(leg.from);
        let b = p(leg.to);
        if leg.no_claims {
            let colour = [1.0, 0.75, 0.3, 0.3];
            list.add_rect([a[0], origin[1] + 2.0], [b[0], origin[1] + 22.0], colour)
                .filled(true)
                .build();
            if hovered && mouse[0] >= a[0] && mouse[0] <= b[0] {
                hint = Some(i18n::format(
                    "No claims recorded for {0}",
                    &[&leg.elapsed.map(duration).unwrap_or_default()],
                ));
            }
        }
        if leg.poll_gap {
            let x = (a[0] + b[0]) / 2.0;
            list.add_line([x, origin[1]], [x, origin[1] + 6.0], [0.75, 0.5, 0.9, 1.0])
                .thickness(2.0)
                .build();
            if hovered && (mouse[0] - x).abs() < 5.0 {
                hint = Some(i18n::text("Polling gap between these claims").to_owned());
            }
        }
        if state.trail.steps[leg.from].claim.map_id != state.trail.steps[leg.to].claim.map_id {
            let x = (a[0] + b[0]) / 2.0;
            list.add_line([x, origin[1]], [x, origin[1] + 24.0], [0.4, 0.75, 1.0, 1.0])
                .build();
            if hovered && (mouse[0] - x).abs() < 5.0 {
                hint = Some(i18n::format(
                    "jump to {0}",
                    &[super::map_label(state.trail.steps[leg.to].claim.map_role)],
                ));
            }
        }
    }
    let mut last_pixel = [None; 4];
    for (i, step) in state.trail.steps.iter().enumerate() {
        let current = i == state.player.selected;
        let future = i > state.player.selected;
        let colour = if current {
            [1.0, 0.4, 0.2, 1.0]
        } else if future {
            [0.62, 0.70, 0.77, 0.8]
        } else {
            [1.0, 0.75, 0.3, 1.0]
        };
        let [x, y] = p(i);
        // More than one claim can occupy a screen pixel. Draw each style there only once;
        // hit-testing and the keyboard selector still use every original observation.
        let style = usize::from(future) + 2 * usize::from(step.late);
        let pixel = (x.round().to_bits(), y.round().to_bits());
        if !current && last_pixel[style] == Some(pixel) {
            continue;
        }
        if !current {
            last_pixel[style] = Some(pixel);
        }
        if current {
            list.add_circle([x, y], 5.0, colour)
                .filled(true)
                .num_segments(12)
                .build();
        } else if future || step.late {
            for offset in [-3.0, 2.0] {
                list.add_line([x, y + offset], [x, y + offset + 1.0], colour)
                    .build();
            }
        } else {
            list.add_line([x, y - 3.0], [x, y + 3.0], colour).build();
        }
    }
    drop(list);
    if hovered {
        if let Some(text) = hint {
            ui.tooltip_text(text);
        }
    }
    if unknown {
        ui.text_disabled(i18n::text("Lower lane: unknown claim time"));
    }
    let first = state.trail.steps.iter().find_map(|s| s.claim.claimed_at);
    let last = state
        .trail
        .steps
        .iter()
        .rev()
        .find_map(|s| s.claim.claimed_at);
    if let Some((a, b)) = first.zip(last) {
        ui.text_disabled(format!(
            "{} - {}",
            crate::clock::clock_seconds(a),
            crate::clock::clock_seconds(b)
        ));
    }
}

#[cfg(test)]
#[path = "timeline_tests.rs"]
mod tests;
