//! Snapshot lifecycle: baseline and match change (RF-03d, RF-04, RF-26).
//!
//! Flip and claim events belong to the diff itself (T-07); this module only decides
//! *when* a snapshot starts a new baseline and what the application must be told.

use std::collections::BTreeMap;

use chrono::{DateTime, Duration, Utc};

use crate::model::{Match, MatchObjective, TeamColor};

/// One poll result, flattened so the diff does not care about map grouping.
#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot {
    pub match_id: String,
    pub taken_at: DateTime<Utc>,
    /// Keyed by objective id (`"38-20"`), ordered for deterministic iteration.
    pub objectives: BTreeMap<String, MatchObjective>,
}

impl Snapshot {
    /// Builds a snapshot from an API match payload.
    pub fn from_match(m: &Match, taken_at: DateTime<Utc>) -> Self {
        let objectives = m
            .maps
            .iter()
            .flat_map(|map| &map.objectives)
            .map(|o| (o.id.clone(), o.clone()))
            .collect();
        Self {
            match_id: m.id.clone(),
            taken_at,
            objectives,
        }
    }
}

/// Why a snapshot became a baseline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaselineReason {
    /// First snapshot of the session (RF-04).
    SessionStart,
    /// The match id changed, so colours and scores are not comparable (RF-03d).
    MatchChanged,
    /// Too much time since the previous valid snapshot (RF-26).
    Gap,
}

/// What the application needs to know after a poll.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// No flip alerts may be emitted from this snapshot onwards.
    Baseline { reason: BaselineReason },
    /// The match id changed; the caller must resolve colours again.
    MatchChanged { old: String, new: String },
    /// There is a hole in the observations between the two instants.
    Gap {
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    },
    /// An objective changed owner (RF-10). `at` is `last_flipped` when present.
    Flip {
        objective: String,
        from: Option<TeamColor>,
        to: Option<TeamColor>,
        at: DateTime<Utc>,
    },
    /// A guild now claims the objective (RF-21). `at` is `claimed_at` when present.
    Claim {
        objective: String,
        guild: String,
        at: DateTime<Utc>,
    },
    /// A guild no longer claims the objective; `at` is when it was observed (RF-22).
    Unclaim {
        objective: String,
        guild: String,
        at: DateTime<Utc>,
    },
}

/// Tuning for the differ.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiffConfig {
    /// Polling interval the user configured.
    pub interval: Duration,
}

impl Default for DiffConfig {
    fn default() -> Self {
        Self {
            interval: Duration::seconds(30),
        }
    }
}

impl DiffConfig {
    /// Multiple of the interval above which a hole counts as a gap (RF-26).
    pub const GAP_FACTOR: i32 = 3;

    /// Threshold as a duration.
    pub fn gap_threshold(&self) -> Duration {
        self.interval * Self::GAP_FACTOR
    }
}

/// Stateful differ: keeps the previous snapshot and decides the lifecycle events.
#[derive(Debug)]
pub struct Differ {
    prev: Option<Snapshot>,
    cfg: DiffConfig,
}

impl Differ {
    /// Creates a differ with the given tuning.
    pub fn new(cfg: DiffConfig) -> Self {
        Self { prev: None, cfg }
    }

    /// Last accepted snapshot, if any.
    pub fn prev(&self) -> Option<&Snapshot> {
        self.prev.as_ref()
    }

    /// Forgets the previous snapshot; the next one is a session baseline again.
    pub fn reset(&mut self) {
        self.prev = None;
    }

    /// Feeds a new snapshot and returns the lifecycle events it triggers.
    pub fn push(&mut self, snapshot: Snapshot) -> Vec<Event> {
        let mut events = Vec::new();

        let baseline = match &self.prev {
            None => Some(BaselineReason::SessionStart),
            Some(prev) if prev.match_id != snapshot.match_id => {
                events.push(Event::MatchChanged {
                    old: prev.match_id.clone(),
                    new: snapshot.match_id.clone(),
                });
                Some(BaselineReason::MatchChanged)
            }
            Some(prev) => {
                let delta = snapshot.taken_at - prev.taken_at;
                if delta > self.cfg.gap_threshold() {
                    events.push(Event::Gap {
                        from: prev.taken_at,
                        to: snapshot.taken_at,
                    });
                    Some(BaselineReason::Gap)
                } else {
                    None
                }
            }
        };

        if let Some(reason) = baseline {
            events.push(Event::Baseline { reason });
        } else if let Some(prev) = &self.prev {
            // A baseline never produces flips: there is nothing to compare against (RF-04).
            flips_and_claims(prev, &snapshot, &mut events);
        }

        self.prev = Some(snapshot);
        events
    }
}

/// Compares two comparable snapshots and appends the observed changes.
///
/// Iteration is ordered by objective id (the snapshots are `BTreeMap`s), and for a
/// single objective the `Unclaim` always precedes the `Claim` (RF-22).
fn flips_and_claims(prev: &Snapshot, cur: &Snapshot, out: &mut Vec<Event>) {
    for (id, now) in &cur.objectives {
        let Some(before) = prev.objectives.get(id) else {
            // Not comparable: the objective was not there before. A late claim for it
            // is the business of the baseline logic (T-09), not of this diff.
            continue;
        };

        if before.owner != now.owner {
            out.push(Event::Flip {
                objective: id.clone(),
                from: before.owner,
                to: now.owner,
                at: now.last_flipped.unwrap_or(cur.taken_at),
            });
        }

        match (before.claimed_by.as_deref(), now.claimed_by.as_deref()) {
            (Some(old), Some(new)) if old != new => {
                out.push(Event::Unclaim {
                    objective: id.clone(),
                    guild: old.to_string(),
                    at: cur.taken_at,
                });
                out.push(Event::Claim {
                    objective: id.clone(),
                    guild: new.to_string(),
                    at: now.claimed_at.unwrap_or(cur.taken_at),
                });
            }
            (Some(old), None) => out.push(Event::Unclaim {
                objective: id.clone(),
                guild: old.to_string(),
                at: cur.taken_at,
            }),
            (None, Some(new)) => out.push(Event::Claim {
                objective: id.clone(),
                guild: new.to_string(),
                at: now.claimed_at.unwrap_or(cur.taken_at),
            }),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;
