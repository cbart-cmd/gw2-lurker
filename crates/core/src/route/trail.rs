//! Pure temporal model for B - Estela (T-51, RF-32, RF-33, RF-36).
//!
//! Uses the existing route ordering; no clock, I/O, drawing or history persistence.
//! Indices are zero-based internally; the UI labels claims as 1..N and keeps origin 0 separate
//! through [`super::with_origin`]. A view never removes or renumbers observations.

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use chrono::{DateTime, TimeDelta, Utc};

use crate::{matchup::ClaimRow, Event, ObjectiveInfo};

/// Time span behind the selected claim; observations outside it are dimmed, not discarded.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Window {
    #[default]
    All,
    Hour,
    Minutes15,
}

impl Window {
    /// Window length, or no limit for the available history.
    fn duration(self) -> Option<TimeDelta> {
        match self {
            Self::All => None,
            Self::Hour => Some(TimeDelta::hours(1)),
            Self::Minutes15 => Some(TimeDelta::minutes(15)),
        }
    }
}

/// How a claim is presented relative to the selected step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Current,
    Active,
    Outside,
    Future,
    /// The finite window cannot be evaluated because a timestamp is unavailable.
    UnknownTime,
}

/// Whether two consecutive observations can have a curved connector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Connection {
    Curve,
    MapChange,
    MissingPosition,
    /// Consecutive claims on the same objective need no zero-length connector.
    Revisit,
}

/// Claim with its official position and temporal information, without invented data.
#[derive(Debug, Clone, PartialEq)]
pub struct Observation<'a> {
    pub claim: Cow<'a, ClaimRow>,
    pub position: Option<[f64; 2]>,
    /// Normalized time in 0..=1; 0.5 for equal timestamps, None when unknown.
    pub timeline: Option<f64>,
    /// Interval until the next claim across all maps, not an occupation duration.
    pub until_next: Option<TimeDelta>,
    /// Matched against the original claim event, including guild and timestamp.
    pub late: bool,
}

/// One adjacent pair of observations, including pairs that must not be joined on the map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Leg {
    pub from: usize,
    pub to: usize,
    pub connection: Connection,
    pub elapsed: Option<TimeDelta>,
    /// At least 20 minutes without a recorded claim, not evidence of inactivity.
    pub no_claims: bool,
    /// The interval overlaps a polling gap reported by the differ.
    pub poll_gap: bool,
}

/// All visits to an objective on one map, in stable chronological order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    pub steps: Vec<usize>,
}

/// All observations of one guild; borrowed history and copied metadata only.
#[derive(Debug, Clone, PartialEq)]
pub struct Trail<'a> {
    pub steps: Vec<Observation<'a>>,
    pub legs: Vec<Leg>,
    pub groups: Vec<Group>,
}

/// Selection and per-step visibility, with exactly one state per original observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct View {
    /// None for an empty trail; otherwise clamped to an existing observation.
    pub selected: Option<usize>,
    pub states: Vec<Visibility>,
}

impl<'a> Trail<'a> {
    /// Builds the guild timeline and preserves late/gap annotations from retained events.
    ///
    /// Missing events are not reconstructed from claim spacing. Origins remain a separate
    /// inference (`with_origin`); they never become claims or acquire invented timestamps.
    pub fn new(
        rows: &'a [ClaimRow],
        guild: &str,
        names: &[ObjectiveInfo],
        events: &[Event],
    ) -> Self {
        let rows = super::ordered(rows, guild, None, None);
        let coordinates: HashMap<_, _> = names
            .iter()
            .filter_map(|name| {
                let c = name.coord?;
                c.iter()
                    .all(|v| v.is_finite())
                    .then_some(((name.map_id, name.id.as_str()), [c[0], c[1]]))
            })
            .collect();
        let late: HashSet<_> = events
            .iter()
            .filter_map(|event| match event {
                Event::Claim {
                    objective,
                    guild: g,
                    at,
                    late: true,
                } if g == guild => Some((objective.as_str(), *at)),
                _ => None,
            })
            .collect();
        let first = rows.iter().find_map(|r| r.claimed_at);
        let last = rows.iter().rev().find_map(|r| r.claimed_at);
        let axis = first
            .zip(last)
            .map(|(start, end)| (start, (end - start).num_milliseconds()));
        let steps: Vec<_> = rows
            .iter()
            .enumerate()
            .map(|(i, row)| Observation {
                claim: Cow::Borrowed(row),
                position: coordinates
                    .get(&(row.map_id, row.objective_id.as_str()))
                    .copied(),
                timeline: row.claimed_at.zip(axis).map(|(at, (start, span))| {
                    if span == 0 {
                        0.5
                    } else {
                        (at - start).num_milliseconds() as f64 / span as f64
                    }
                }),
                until_next: row
                    .claimed_at
                    .zip(rows.get(i + 1).and_then(|next| next.claimed_at))
                    .map(|(a, b)| b - a),
                late: row
                    .claimed_at
                    .is_some_and(|at| late.contains(&(row.objective_id.as_str(), at))),
            })
            .collect();
        let gaps = merged_gaps(events);
        let legs =
            steps
                .windows(2)
                .enumerate()
                .map(|(i, pair)| {
                    let (a, b) = (&pair[0], &pair[1]);
                    let connection = if a.claim.map_id != b.claim.map_id {
                        Connection::MapChange
                    } else if a.position.is_none() || b.position.is_none() {
                        Connection::MissingPosition
                    } else if a.claim.objective_id == b.claim.objective_id {
                        Connection::Revisit
                    } else {
                        Connection::Curve
                    };
                    Leg {
                        from: i,
                        to: i + 1,
                        connection,
                        elapsed: a.until_next,
                        no_claims: a.until_next.is_some_and(|d| d >= TimeDelta::minutes(20)),
                        poll_gap: a.claim.claimed_at.zip(b.claim.claimed_at).is_some_and(
                            |(a, b)| {
                                let next = gaps.partition_point(|(_, end)| *end <= a);
                                a < b && gaps.get(next).is_some_and(|(start, _)| *start < b)
                            },
                        ),
                    }
                })
                .collect();
        let groups = group_visits(&steps);
        Self {
            steps,
            legs,
            groups,
        }
    }

    /// Classifies every step without filtering the source history.
    ///
    /// Selection clamps at the last claim (also when history was shortened). Window boundaries
    /// are inclusive; unknown timestamps are explicit rather than treated as old or recent.
    pub fn view(&self, selected: usize, window: Window) -> View {
        let selected = (!self.steps.is_empty()).then(|| selected.min(self.steps.len() - 1));
        let mut states = Vec::with_capacity(self.steps.len());
        if let Some(current) = selected {
            let at = self.steps[current].claim.claimed_at;
            for (i, step) in self.steps.iter().enumerate() {
                let state = if i == current {
                    Visibility::Current
                } else if i > current {
                    Visibility::Future
                } else if let Some(limit) = window.duration() {
                    match step.claim.claimed_at.zip(at) {
                        Some((a, b)) if b - a > limit => Visibility::Outside,
                        Some(_) => Visibility::Active,
                        None => Visibility::UnknownTime,
                    }
                } else {
                    Visibility::Active
                };
                states.push(state);
            }
        }
        View { selected, states }
    }

    /// Owns the observations for a UI cache without borrowing the live dashboard.
    pub fn into_owned(self) -> Trail<'static> {
        Trail {
            steps: self
                .steps
                .into_iter()
                .map(|s| Observation {
                    claim: Cow::Owned(s.claim.into_owned()),
                    position: s.position,
                    timeline: s.timeline,
                    until_next: s.until_next,
                    late: s.late,
                })
                .collect(),
            legs: self.legs,
            groups: self.groups,
        }
    }
}

/// Groups by map and objective while preserving each visit's index.
fn group_visits(steps: &[Observation<'_>]) -> Vec<Group> {
    let mut groups: Vec<Group> = Vec::new();
    let mut indices = HashMap::new();
    for (i, step) in steps.iter().enumerate() {
        let key = (step.claim.map_id, step.claim.objective_id.as_str());
        let index = *indices.entry(key).or_insert_with(|| {
            groups.push(Group { steps: Vec::new() });
            groups.len() - 1
        });
        groups[index].steps.push(i);
    }
    groups
}

/// Merges polling gaps so interval lookups are logarithmic, even for overlapping input.
fn merged_gaps(events: &[Event]) -> Vec<(DateTime<Utc>, DateTime<Utc>)> {
    let mut gaps: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            Event::Gap { from, to } if from < to => Some((*from, *to)),
            _ => None,
        })
        .collect();
    gaps.sort_unstable();
    let mut merged: Vec<(DateTime<Utc>, DateTime<Utc>)> = Vec::new();
    for (start, end) in gaps {
        if let Some(last) = merged.last_mut().filter(|last| start <= last.1) {
            last.1 = last.1.max(end);
        } else {
            merged.push((start, end));
        }
    }
    merged
}

#[cfg(test)]
#[path = "trail_tests.rs"]
mod tests;
