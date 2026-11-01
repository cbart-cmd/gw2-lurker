use super::*;
use crate::model::{MapRole, ObjectiveType};
use chrono::{TimeDelta, TimeZone};

/// Synthetic clock with explicit offsets for temporal boundary tests.
fn at(seconds: i64) -> DateTime<Utc> {
    Utc.timestamp_opt(1_800_000_000 + seconds, 0).unwrap()
}

/// Synthetic observation; tests needing official coordinates load the objective fixture.
fn row(id: &str, map_id: u32, seconds: Option<i64>) -> ClaimRow {
    ClaimRow {
        map_id,
        map_role: MapRole::from_map_id(map_id),
        objective_id: id.into(),
        name: None,
        kind: ObjectiveType::Camp,
        owner: None,
        guild: "G".into(),
        claimed_at: seconds.map(at),
        points_tick: 2,
    }
}

/// Real objective catalogue shared with the existing origin tests.
fn names() -> Vec<ObjectiveInfo> {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../api/tests/fixtures/objectives_all.json"
    )))
    .unwrap()
}

#[test]
fn rf32_trail_preserves_revisits_and_orders_time_ties_by_objective() {
    let rows = [
        row("95-33", 95, Some(60)),
        row("95-32", 95, Some(0)),
        row("95-32", 95, Some(60)),
        row("96-32", 96, Some(180)),
    ];
    let trail = Trail::new(&rows, "G", &names(), &[]);
    let ids: Vec<_> = trail
        .steps
        .iter()
        .map(|s| s.claim.objective_id.as_str())
        .collect();
    assert_eq!(ids, ["95-32", "95-32", "95-33", "96-32"]);
    assert_eq!(trail.groups[0].steps, [0, 1]);
    assert_eq!(trail.groups.len(), 3);
    assert_eq!(trail.legs[0].connection, Connection::Revisit);
    assert_eq!(trail.legs[2].connection, Connection::MapChange);
    assert_eq!(trail.steps[1].timeline, trail.steps[2].timeline);
    assert_eq!(trail.steps[1].timeline, Some(1.0 / 3.0));
    assert_eq!(trail.steps[0].timeline, Some(0.0));
    assert_eq!(trail.steps[3].timeline, Some(1.0));
}

#[test]
fn rf36_trail_windows_include_the_exact_limit_without_renumbering() {
    for (window, limit) in [(Window::Minutes15, 900), (Window::Hour, 3600)] {
        let rows = [
            row("a", 95, Some(0)),
            row("b", 95, Some(1)),
            row("c", 95, Some(2)),
            row("d", 95, Some(limit + 1)),
            row("e", 95, Some(limit + 2)),
        ];
        let trail = Trail::new(&rows, "G", &[], &[]);
        let view = trail.view(3, window);
        assert_eq!(view.selected, Some(3));
        assert_eq!(
            view.states,
            [
                Visibility::Outside,
                Visibility::Active,
                Visibility::Active,
                Visibility::Current,
                Visibility::Future
            ]
        );
        assert_eq!(trail.steps.len(), rows.len());
        assert_eq!(trail.groups[0].steps, [0]);
        assert_eq!(Window::default(), Window::All);
        assert_eq!(trail.view(3, Window::All).states[0], Visibility::Active);
    }
}

#[test]
fn rf33_trail_claim_silence_is_not_a_polling_gap() {
    for seconds in [1199, 1200, 1201] {
        let rows = [row("a", 95, Some(0)), row("b", 95, Some(seconds))];
        let trail = Trail::new(&rows, "G", &[], &[]);
        assert_eq!(trail.legs[0].no_claims, seconds >= 1200);
        assert!(!trail.legs[0].poll_gap);
        assert_eq!(trail.steps[0].until_next, Some(TimeDelta::seconds(seconds)));
        assert_eq!(trail.steps[1].until_next, None);
    }
}

#[test]
fn rf33_trail_preserves_late_events_and_overlapping_poll_gaps() {
    let rows = [
        row("a", 95, Some(0)),
        row("b", 95, Some(60)),
        row("c", 95, Some(120)),
    ];
    let events = [
        Event::Claim {
            objective: "b".into(),
            guild: "G".into(),
            at: at(60),
            late: true,
        },
        Event::Claim {
            objective: "c".into(),
            guild: "other".into(),
            at: at(120),
            late: true,
        },
        Event::Gap {
            from: at(70),
            to: at(90),
        },
        Event::Gap {
            from: at(30),
            to: at(80),
        },
    ];
    let trail = Trail::new(&rows, "G", &[], &events);
    assert!(!trail.steps[0].late);
    assert!(trail.steps[1].late);
    assert!(!trail.steps[2].late);
    assert!(trail.legs.iter().all(|leg| leg.poll_gap && !leg.no_claims));
    let boundary = [Event::Gap {
        from: at(60),
        to: at(120),
    }];
    let trail = Trail::new(&rows, "G", &[], &boundary);
    assert!(!trail.legs[0].poll_gap);
    assert!(trail.legs[1].poll_gap);
}

#[test]
fn rf32_trail_missing_coordinates_break_both_links_without_losing_the_step() {
    let rows = [
        row("95-32", 95, Some(0)),
        row("missing", 95, Some(60)),
        row("95-33", 95, Some(120)),
    ];
    let names = names();
    let trail = Trail::new(&rows, "G", &names, &[]);
    assert_eq!(trail.steps.len(), 3);
    assert_eq!(trail.steps[1].position, None);
    assert!(trail
        .legs
        .iter()
        .all(|leg| leg.connection == Connection::MissingPosition));
    for step in [&trail.steps[0], &trail.steps[2]] {
        let objective = names
            .iter()
            .find(|o| o.id == step.claim.objective_id)
            .unwrap();
        assert_eq!(step.position, objective.coord.map(|c| [c[0], c[1]]));
    }
}

#[test]
fn rf36_trail_empty_single_and_equal_times_have_finite_timelines() {
    let empty = Trail::new(&[], "G", &[], &[]);
    assert_eq!(empty.view(usize::MAX, Window::All).selected, None);
    assert!(empty.view(0, Window::All).states.is_empty());
    for rows in [
        vec![row("a", 95, Some(0))],
        vec![row("b", 95, Some(0)), row("a", 95, Some(0))],
    ] {
        let trail = Trail::new(&rows, "G", &[], &[]);
        assert!(trail.steps.iter().all(|s| s.timeline == Some(0.5)));
        assert_eq!(
            trail.view(usize::MAX, Window::All).selected,
            Some(rows.len() - 1)
        );
        assert_eq!(trail.legs.len(), rows.len() - 1);
    }
}

#[test]
fn rf36_trail_does_not_invent_times_or_intervals() {
    let rows = [
        row("a", 95, None),
        row("b", 95, Some(0)),
        row("c", 95, Some(60)),
    ];
    let trail = Trail::new(&rows, "G", &[], &[]);
    assert_eq!(trail.steps[0].timeline, None);
    assert_eq!(trail.steps[0].until_next, None);
    assert_eq!(trail.legs[0].elapsed, None);
    assert_eq!(
        trail.view(2, Window::Minutes15).states[0],
        Visibility::UnknownTime
    );
    assert_eq!(
        trail.view(0, Window::Hour).states,
        [Visibility::Current, Visibility::Future, Visibility::Future]
    );
}

#[test]
fn rf32_trail_filters_the_guild_and_keeps_fast_claims_as_one_sequence() {
    let mut rows: Vec<_> = (0..28).map(|i| row("95-32", 95, Some(i * 50))).collect();
    rows.push(ClaimRow {
        guild: "other".into(),
        ..row("95-33", 95, Some(1500))
    });
    let trail = Trail::new(&rows, "G", &names(), &[]);
    assert_eq!(trail.steps.len(), 28);
    assert_eq!(trail.groups[0].steps, (0..28).collect::<Vec<_>>());
    assert_eq!(trail.legs.len(), 27);
    assert!(trail.legs.iter().all(|leg| !leg.no_claims && !leg.poll_gap));
}

#[test]
fn rf36_trail_unknown_times_stay_explicit_and_do_not_create_silence() {
    let rows = [row("b", 95, None), row("a", 95, None)];
    let trail = Trail::new(&rows, "G", &[], &[]);
    assert!(trail
        .steps
        .iter()
        .all(|s| s.timeline.is_none() && s.until_next.is_none()));
    assert_eq!(
        trail.view(1, Window::Hour).states,
        [Visibility::UnknownTime, Visibility::Current]
    );
    assert!(!trail.legs[0].no_claims);
    assert!(!trail.legs[0].poll_gap);
}

#[test]
fn rf36_trail_window_uses_exact_durations_not_rounded_seconds() {
    let mut rows = [row("a", 95, Some(0)), row("b", 95, Some(900))];
    rows[1].claimed_at = Some(at(900) + TimeDelta::milliseconds(1));
    let trail = Trail::new(&rows, "G", &[], &[]);
    assert_eq!(
        trail.view(1, Window::Minutes15).states[0],
        Visibility::Outside
    );
}

#[test]
fn rf32_trail_cannot_use_another_maps_or_nonfinite_coordinates() {
    let rows = [row("95-32", 96, Some(0)), row("95-33", 95, Some(60))];
    let mut names = names();
    let second = names.iter_mut().find(|o| o.id == "95-33").unwrap();
    second.coord.as_mut().unwrap()[0] = f64::NAN;
    let trail = Trail::new(&rows, "G", &names, &[]);
    assert!(trail.steps.iter().all(|s| s.position.is_none()));
    assert_eq!(trail.legs[0].connection, Connection::MapChange);
}
