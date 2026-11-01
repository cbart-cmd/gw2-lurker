//! Prepared geometry for B - Estela (T-52), independent of Nexus.

use gw2lurker_core::route::trail::{Connection, Trail, View, Visibility};

use super::project;
use super::route::RouteCanvas;

/// Quadratic visual connector; not a traversable path.
#[derive(Debug, Clone)]
pub(super) struct Curve {
    pub from: [f32; 2],
    pub control: [f32; 2],
    pub to: [f32; 2],
}

impl Curve {
    /// Alternate sides in a canonical direction, so a return leg cannot retrace the same arc.
    pub fn between(from: [f32; 2], to: [f32; 2], index: usize) -> Self {
        let dx = to[0] - from[0];
        let dy = to[1] - from[1];
        let length = dx.hypot(dy).max(1.0);
        let direction = if from < to { 1.0 } else { -1.0 };
        let side = if index.is_multiple_of(2) {
            direction
        } else {
            -direction
        };
        let offset = (length * 0.22).clamp(14.0, 95.0) * side;
        Self {
            from,
            to,
            control: [
                (from[0] + to[0]) / 2.0 - dy / length * offset,
                (from[1] + to[1]) / 2.0 + dx / length * offset,
            ],
        }
    }

    /// Point on the quadratic curve, with t in 0..=1.
    pub fn point(&self, t: f32) -> [f32; 2] {
        let u = 1.0 - t;
        std::array::from_fn(|i| {
            u * u * self.from[i] + 2.0 * u * t * self.control[i] + t * t * self.to[i]
        })
    }

    /// Equivalent cubic control points for imgui's verified Bezier API.
    pub fn cubic(&self) -> ([f32; 2], [f32; 2]) {
        (
            std::array::from_fn(|i| self.from[i] + (self.control[i] - self.from[i]) * 2.0 / 3.0),
            std::array::from_fn(|i| self.to[i] + (self.control[i] - self.to[i]) * 2.0 / 3.0),
        )
    }
}

/// A precomputed connector and its style.
#[derive(Debug)]
pub(super) struct Stroke {
    pub curve: Curve,
    pub colour: [f32; 4],
    pub width: f32,
    pub dashed: bool,
    pub visibility: Visibility,
}

/// One marker for every visited objective, including all its original step numbers.
#[derive(Debug)]
pub(super) struct Marker {
    pub objective_id: String,
    pub point: [f32; 2],
    pub label: String,
    pub steps: Vec<usize>,
    pub current: bool,
    pub future: bool,
    pub outside: bool,
    pub late: bool,
    pub latest: bool,
    pub interval_radius: Option<f32>,
}

/// Prepared drawing for one map; rebuilt only when data, viewport or selection changes.
#[derive(Debug, Default)]
pub(super) struct Scene {
    pub curves: Vec<Stroke>,
    pub markers: Vec<Marker>,
    pub origin: Option<[f32; 2]>,
    pub origin_link: Option<Stroke>,
}

impl Scene {
    /// Projects consecutive observations without bridging missing coordinates or map changes.
    pub fn prepare(trail: &Trail<'_>, view: &View, map: u32, canvas: &RouteCanvas<'_>) -> Self {
        let points: Vec<_> = trail
            .steps
            .iter()
            .map(|step| {
                (step.claim.map_id == map)
                    .then_some(step.position)
                    .flatten()
                    .and_then(|p| project(canvas.frame, p, canvas.origin, canvas.size))
            })
            .collect();
        let mut scene = Self::default();
        for leg in &trail.legs {
            if leg.connection != Connection::Curve {
                continue;
            }
            let (Some(a), Some(b)) = (points[leg.from], points[leg.to]) else {
                continue;
            };
            if (a[0] - b[0]).hypot(a[1] - b[1]) < 1.0 {
                continue;
            }
            let visibility = view.states[leg.to];
            let f = (leg.to as f32 / view.selected.unwrap_or(1).max(1) as f32).min(1.0);
            let (colour, width) = match visibility {
                Visibility::Future => ([0.62, 0.70, 0.77, 0.4], 1.4),
                Visibility::Outside => ([1.0, 0.75, 0.3, 0.16], 1.4),
                Visibility::UnknownTime => ([0.65, 0.67, 0.7, 0.7], 1.5),
                _ => (
                    [1.0, 0.75, 0.3, 0.36 + 0.64 * f],
                    1.8 + 3.2 * f
                        + if visibility == Visibility::Current {
                            1.4
                        } else {
                            0.0
                        },
                ),
            };
            scene.curves.push(Stroke {
                curve: Curve::between(a, b, leg.from),
                colour,
                width,
                visibility,
                dashed: leg.poll_gap
                    || trail.steps[leg.from].claim.claimed_at.is_none()
                    || trail.steps[leg.to].claim.claimed_at.is_none()
                    || trail.steps[leg.to].late
                    || matches!(visibility, Visibility::Future | Visibility::UnknownTime),
            });
        }
        for group in &trail.groups {
            let Some(point) = group.steps.iter().find_map(|&i| points[i]) else {
                continue;
            };
            let current = group
                .steps
                .iter()
                .any(|&i| view.states[i] == Visibility::Current);
            let future = group
                .steps
                .iter()
                .all(|&i| view.states[i] == Visibility::Future);
            let outside = group
                .steps
                .iter()
                .all(|&i| matches!(view.states[i], Visibility::Future | Visibility::Outside));
            let visible = group
                .steps
                .iter()
                .rev()
                .copied()
                .find(|&i| matches!(view.states[i], Visibility::Current | Visibility::Active));
            let interval_radius = visible
                .and_then(|i| trail.steps[i].until_next)
                .map(|dt| 13.0 + (4.2 * (dt.num_seconds().max(0) as f32 / 60.0).sqrt()).min(30.0));
            let mut labelled = if group.steps.len() <= 3 {
                group.steps.clone()
            } else {
                let focus = view.selected.filter(|i| group.steps.contains(i));
                vec![
                    group.steps[0],
                    focus.unwrap_or(group.steps[1]),
                    *group.steps.last().unwrap_or(&group.steps[0]),
                ]
            };
            labelled.sort_unstable();
            labelled.dedup();
            let mut label = labelled
                .iter()
                .map(|i| (i + 1).to_string())
                .collect::<Vec<_>>()
                .join("/");
            if group.steps.len() > 3 {
                label.push_str(&format!("/+{}", group.steps.len() - labelled.len()));
            }
            scene.markers.push(Marker {
                objective_id: trail.steps[group.steps[0]].claim.objective_id.clone(),
                point,
                label,
                steps: group.steps.clone(),
                current,
                future,
                outside,
                late: group.steps.iter().any(|&i| trail.steps[i].late),
                latest: group.steps.contains(&trail.steps.len().saturating_sub(1)),
                interval_radius,
            });
        }
        // Coincident arcs share the newest stroke of each semantic state; observations and
        // revisit numbers remain intact. Repeated loops must not exhaust imgui's 16-bit indices.
        let mut latest = std::collections::BTreeMap::new();
        let key = |s: &Stroke| {
            (
                s.curve.from.map(f32::to_bits),
                s.curve.control.map(f32::to_bits),
                s.curve.to.map(f32::to_bits),
                s.dashed,
                match s.visibility {
                    Visibility::Current | Visibility::Active => 0,
                    Visibility::Future => 1,
                    Visibility::Outside => 2,
                    Visibility::UnknownTime => 3,
                },
            )
        };
        for (i, stroke) in scene.curves.iter().enumerate() {
            latest.insert(key(stroke), i);
        }
        let mut index = 0;
        scene.curves.retain(|stroke| {
            let keep = latest.get(&key(stroke)) == Some(&index);
            index += 1;
            keep
        });
        scene
    }

    /// Origin is separate from observations and has no invented timestamp.
    pub fn attach_origin<'a>(
        &mut self,
        route: &gw2lurker_core::Route<'_>,
        rows: impl IntoIterator<Item = &'a super::ClaimRow>,
        map: u32,
        canvas: &RouteCanvas<'_>,
    ) {
        let Some(first) = rows.into_iter().find(|r| r.map_id == map) else {
            return;
        };
        let Some(origin) = route.origin_point.as_ref() else {
            return;
        };
        let Some(point) = project(canvas.frame, origin.position, canvas.origin, canvas.size) else {
            return;
        };
        self.origin = Some(point);
        if origin.objective_id == first.objective_id {
            if let Some(marker) = self
                .markers
                .iter_mut()
                .find(|m| m.objective_id == first.objective_id)
            {
                marker.label = format!("0/{}", marker.label);
            }
        } else if let Some(marker) = self
            .markers
            .iter()
            .find(|m| m.objective_id == first.objective_id)
        {
            self.origin_link = Some(Stroke {
                curve: Curve {
                    from: point,
                    control: [
                        (point[0] + marker.point[0]) / 2.0,
                        (point[1] + marker.point[1]) / 2.0,
                    ],
                    to: marker.point,
                },
                colour: [1.0, 0.75, 0.3, 0.8],
                width: 1.5,
                dashed: true,
                visibility: Visibility::UnknownTime,
            });
        }
    }
}
