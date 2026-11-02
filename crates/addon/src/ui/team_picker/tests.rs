//! Tests of the team picker: the narrowing and when it opens by itself (RF-03a, RF-03j).

use super::*;

fn candidate(team_id: u32, match_id: &str, color: TeamColor) -> Candidate {
    Candidate {
        team_id,
        match_id: match_id.to_owned(),
        color,
        worlds: Vec::new(),
    }
}

#[test]
fn rf03a_narrowing_starts_at_the_saved_team() {
    let list = vec![
        candidate(12014, "2-2", TeamColor::Blue),
        candidate(12015, "2-2", TeamColor::Red),
        candidate(11001, "1-1", TeamColor::Green),
    ];
    let start = initial_narrowing(Narrowing::default(), &list, Some(12015));
    assert_eq!(start.region, Some(Region::Eu));
    assert_eq!(start.colour, Some(TeamColor::Red));

    let fallback = initial_narrowing(Narrowing::default(), &list, None);
    assert_eq!(fallback.region, Some(Region::Eu), "la región del primero");
    assert_eq!(fallback.colour, None, "sin equipo no se adivina el color");

    assert_eq!(
        initial_narrowing(Narrowing::default(), &[], None),
        Narrowing::default()
    );
}

#[test]
fn rf03a_both_regions_are_always_offered() {
    // Regression of the 2026-10-31 bug: with no live teams the drop-down was empty.
    // `draw_region` now offers EU and NA unconditionally, so the control always works;
    // what this pins is that no data does not fabricate a preselection either.
    let start = initial_narrowing(Narrowing::default(), &[], None);
    assert_eq!(start.region, None, "sin datos no se preselecciona región");
    assert_eq!(start.colour, None);
}

#[test]
fn rf03a_the_colour_is_offered_before_any_data() {
    // `draw_colour` iterates `TeamColor::ALL`, which is static: the control is never empty.
    assert_eq!(TeamColor::ALL.len(), 3);
}

#[test]
fn rf03a_narrowing_is_remembered_between_frames() {
    let remembered = Narrowing {
        region: Some(Region::Na),
        colour: Some(TeamColor::Green),
    };
    let list = vec![candidate(12014, "2-2", TeamColor::Blue)];
    let start = initial_narrowing(remembered, &list, Some(12014));
    assert_eq!(
        start,
        Narrowing {
            region: Some(Region::Na),
            colour: Some(TeamColor::Green)
        },
        "lo que el jugador eligió gana a lo deducido"
    );
}

#[test]
fn rf03a_three_steps_bring_the_list_down_to_a_handful() {
    let mut list = Vec::new();
    for index in 1..=5 {
        list.push(candidate(
            12000 + index,
            &format!("2-{index}"),
            TeamColor::Blue,
        ));
        list.push(candidate(
            12100 + index,
            &format!("2-{index}"),
            TeamColor::Red,
        ));
        list.push(candidate(
            11000 + index,
            &format!("1-{index}"),
            TeamColor::Blue,
        ));
    }
    assert_eq!(list.len(), 15, "lo que vería el jugador sin filtrar");

    let eu_blue = select::of_region_and_color(&list, Region::Eu, TeamColor::Blue);
    assert_eq!(eu_blue.len(), 5, "un equipo azul por matchup de EU");

    let na_blue = select::of_region_and_color(&list, Region::Na, TeamColor::Blue);
    assert_eq!(na_blue.len(), 5);
    assert!(
        select::of_region_and_color(&list, Region::Na, TeamColor::Red).is_empty(),
        "y se avisa en vez de mostrar una lista vacía sin explicación"
    );
}

#[test]
fn rf03a_picking_and_clearing_are_told_apart() {
    assert_eq!(
        team_change(None, Some(12014)),
        Some(TeamChange::Pick(12014))
    );
    assert_eq!(
        team_change(Some(12014), Some(12015)),
        Some(TeamChange::Pick(12015))
    );
    assert_eq!(team_change(Some(12014), None), Some(TeamChange::Clear));
    assert_eq!(team_change(Some(12014), Some(12014)), None);
    assert_eq!(team_change(None, None), None);
}

#[test]
fn rf03j_each_map_entry_is_checked_once_after_the_grace() {
    let start = Instant::now();
    let mut entry = Entry::default();
    const EBG: u32 = 38;
    const LOUNGE: u32 = 1315;

    assert!(
        !entry.due(EBG, true, start),
        "recién entrado: arcdps aún no dio la cuenta"
    );
    assert!(entry.due(EBG, true, start + GRACE));
    entry.settled = true;
    assert!(!entry.due(EBG, true, start + GRACE * 3), "ya decidido");

    assert!(
        !entry.due(15, false, start + GRACE * 4),
        "fuera de MvM nunca"
    );
    assert!(!entry.due(15, false, start + GRACE * 9));

    let back = start + GRACE * 10;
    assert!(!entry.due(LOUNGE, true, back), "otra entrada: otra espera");
    assert!(entry.due(LOUNGE, true, back + GRACE), "y otra comprobación");
}

#[test]
fn rf03j_an_entry_waits_for_data_and_records_the_first_matchup_silently() {
    let none = Selection::Missing;
    assert_eq!(
        step(None, &none, None, None),
        Step::Open(Prompt::NoTeam),
        "un jugador nuevo"
    );
    assert_eq!(
        step(Some(12014), &none, None, None),
        Step::Wait,
        "sin la lista de equipos no se decide nada"
    );
    assert_eq!(
        step(Some(12014), &none, Some("2-2:12014,12015,12016"), None),
        Step::Record("2-2:12014,12015,12016".to_owned()),
        "lo de antes de actualizar no es un cambio: se apunta sin preguntar"
    );
    assert_eq!(
        step(
            Some(12014),
            &none,
            Some("2-2:12014,12015,12016"),
            Some("2-2:12014,12015,12016")
        ),
        Step::Done
    );
    assert_eq!(
        step(
            Some(12014),
            &none,
            Some("2-2:12014,12015,12099"),
            Some("2-2:12014,12015,12016")
        ),
        Step::Open(Prompt::NewMatchup)
    );
}
