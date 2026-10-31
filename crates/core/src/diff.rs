//! Snapshot lifecycle: baseline and match change (RF-03d, RF-04, RF-26).
//!
//! Flip and claim events belong to the diff itself (T-07); this module only decides
//! *when* a snapshot starts a new baseline and what the application must be told.

use std::collections::BTreeMap;

use chrono::{DateTime, Duration, Utc};

use crate::model::{Match, MatchObjective};

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
        }

        self.prev = Some(snapshot);
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MATCH: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../api/tests/fixtures/match_2-2.json"
    ));

    fn parsed() -> Match {
        serde_json::from_str(MATCH).unwrap()
    }

    fn at(offset_secs: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(1_800_000_000 + offset_secs, 0).unwrap()
    }

    fn snap(match_id: &str, offset_secs: i64) -> Snapshot {
        let mut m = parsed();
        m.id = match_id.to_string();
        Snapshot::from_match(&m, at(offset_secs))
    }

    fn differ() -> Differ {
        Differ::new(DiffConfig::default())
    }

    #[test]
    fn rf04_first_snapshot_is_a_session_baseline() {
        let mut d = differ();
        assert_eq!(
            d.push(snap("2-2", 0)),
            vec![Event::Baseline {
                reason: BaselineReason::SessionStart
            }]
        );
        assert!(d.prev().is_some());
    }

    #[test]
    fn rf04_snapshot_inside_the_interval_triggers_nothing() {
        let mut d = differ();
        d.push(snap("2-2", 0));
        assert!(d.push(snap("2-2", 30)).is_empty());
    }

    #[test]
    fn rf03d_match_change_resets_the_baseline() {
        let mut d = differ();
        d.push(snap("2-2", 0));

        assert_eq!(
            d.push(snap("2-3", 30)),
            vec![
                Event::MatchChanged {
                    old: "2-2".to_string(),
                    new: "2-3".to_string()
                },
                Event::Baseline {
                    reason: BaselineReason::MatchChanged
                }
            ]
        );
    }

    #[test]
    fn rf26_gap_above_three_intervals_is_reported() {
        let mut d = differ();
        d.push(snap("2-2", 0));

        assert_eq!(
            d.push(snap("2-2", 91)),
            vec![
                Event::Gap {
                    from: at(0),
                    to: at(91)
                },
                Event::Baseline {
                    reason: BaselineReason::Gap
                }
            ]
        );
    }

    #[test]
    fn rf26_exactly_three_intervals_is_not_a_gap() {
        let mut d = differ();
        d.push(snap("2-2", 0));
        assert!(d.push(snap("2-2", 90)).is_empty());
    }

    #[test]
    fn rf26_gap_threshold_follows_the_configured_interval() {
        let cfg = DiffConfig {
            interval: Duration::seconds(10),
        };
        assert_eq!(cfg.gap_threshold(), Duration::seconds(30));

        let mut d = Differ::new(cfg);
        d.push(snap("2-2", 0));
        assert!(d.push(snap("2-2", 25)).is_empty());
        assert!(!d.push(snap("2-2", 60)).is_empty());
    }

    #[test]
    fn rf04_reset_makes_the_next_snapshot_a_baseline_again() {
        let mut d = differ();
        d.push(snap("2-2", 0));
        d.reset();
        assert_eq!(
            d.push(snap("2-2", 30)),
            vec![Event::Baseline {
                reason: BaselineReason::SessionStart
            }]
        );
    }

    #[test]
    fn t06_snapshot_from_match_flattens_every_objective() {
        let m = parsed();
        let s = Snapshot::from_match(&m, at(0));

        assert_eq!(s.match_id, "2-2");
        assert_eq!(s.objectives.len(), 91);
        assert!(s.objectives.contains_key("38-20"));
    }
}
