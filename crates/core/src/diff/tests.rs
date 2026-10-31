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
