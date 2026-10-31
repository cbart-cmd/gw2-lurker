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

    fn objective<'a>(s: &'a mut Snapshot, id: &str) -> &'a mut MatchObjective {
        s.objectives.get_mut(id).expect("objective in fixture")
    }

    #[test]
    fn rf10_owner_change_emits_a_flip_with_the_real_time() {
        let mut d = differ();
        d.push(snap("2-2", 0));

        let mut next = snap("2-2", 30);
        {
            let o = objective(&mut next, "38-20");
            o.owner = Some(TeamColor::Blue);
            o.last_flipped = Some(at(20));
        }

        assert_eq!(
            d.push(next),
            vec![Event::Flip {
                objective: "38-20".to_string(),
                from: Some(TeamColor::Red),
                to: Some(TeamColor::Blue),
                at: at(20),
            }]
        );
    }

    #[test]
    fn rf21_new_claim_uses_claimed_at() {
        let mut d = differ();
        d.push(snap("2-2", 0));

        let mut next = snap("2-2", 30);
        {
            let o = objective(&mut next, "38-4");
            assert!(
                o.claimed_by.is_none(),
                "38-4 must be unclaimed in the fixture"
            );
            o.claimed_by = Some("AAAA-BBBB".to_string());
            o.claimed_at = Some(at(25));
        }

        assert_eq!(
            d.push(next),
            vec![Event::Claim {
                objective: "38-4".to_string(),
                guild: "AAAA-BBBB".to_string(),
                at: at(25),
            }]
        );
    }

    #[test]
    fn rf22_replaced_claim_emits_unclaim_then_claim() {
        let mut d = differ();
        let first = snap("2-2", 0);
        let old_guild = first.objectives["38-20"]
            .claimed_by
            .clone()
            .expect("38-20 is claimed in the fixture");
        d.push(first);

        let mut next = snap("2-2", 30);
        {
            let o = objective(&mut next, "38-20");
            o.claimed_by = Some("NEW-GUILD".to_string());
            o.claimed_at = Some(at(28));
        }

        assert_eq!(
            d.push(next),
            vec![
                Event::Unclaim {
                    objective: "38-20".to_string(),
                    guild: old_guild,
                    at: at(30),
                },
                Event::Claim {
                    objective: "38-20".to_string(),
                    guild: "NEW-GUILD".to_string(),
                    at: at(28),
                },
            ]
        );
    }

    #[test]
    fn rf22_released_claim_emits_only_unclaim() {
        let mut d = differ();
        let first = snap("2-2", 0);
        let old_guild = first.objectives["38-20"].claimed_by.clone().unwrap();
        d.push(first);

        let mut next = snap("2-2", 30);
        {
            let o = objective(&mut next, "38-20");
            o.claimed_by = None;
            o.claimed_at = None;
        }

        assert_eq!(
            d.push(next),
            vec![Event::Unclaim {
                objective: "38-20".to_string(),
                guild: old_guild,
                at: at(30),
            }]
        );
    }

    #[test]
    fn rf04_baseline_after_a_gap_reports_no_flips() {
        let mut d = differ();
        d.push(snap("2-2", 0));

        let mut far = snap("2-2", 200);
        objective(&mut far, "38-20").owner = Some(TeamColor::Blue);

        assert_eq!(
            d.push(far),
            vec![
                Event::Gap {
                    from: at(0),
                    to: at(200)
                },
                Event::Baseline {
                    reason: BaselineReason::Gap
                },
            ]
        );
    }

    #[test]
    fn t07_changes_are_ordered_by_objective_then_by_kind() {
        let mut d = differ();
        d.push(snap("2-2", 0));

        let mut next = snap("2-2", 30);
        objective(&mut next, "38-4").owner = Some(TeamColor::Red);
        objective(&mut next, "38-20").owner = Some(TeamColor::Blue);
        objective(&mut next, "38-20").claimed_by = Some("ZZZ".to_string());

        let ids: Vec<String> = d
            .push(next)
            .into_iter()
            .map(|e| match e {
                Event::Flip { objective, .. }
                | Event::Claim { objective, .. }
                | Event::Unclaim { objective, .. } => objective,
                other => panic!("unexpected lifecycle event: {other:?}"),
            })
            .collect();

        assert_eq!(ids, ["38-20", "38-20", "38-20", "38-4"]);
    }
}
