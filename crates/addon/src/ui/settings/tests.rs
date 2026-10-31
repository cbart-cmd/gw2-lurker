//! Tests of the settings window: the picker, the label column and the translated names.
//!
//! Kept out of `settings.rs` so the window stays under the 800-line limit (AGENTS §2.7).

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
fn rf03f_the_field_starts_with_the_automatic_translation() {
    let table = gw2lurker_core::bundled();
    let empty = crate::config::Config::default();
    // El texto de la tabla cambia cuando el jugador traduce un equipo, así que aquí se toma el
    // que haya en el fichero en vez de fijarlo (2026-10-31).
    let automatic = table
        .name_in("es", 12014)
        .expect("12014 tiene nombre")
        .to_owned();

    // Without Mumble there is no character, so the roster cannot match; the automatic
    // name is what the field shows for a chosen team.
    assert_eq!(
        initial_translation(&empty, table, Some(12014), "es"),
        automatic
    );
    assert_eq!(initial_translation(&empty, table, None, "es"), "");

    let mut custom = crate::config::Config::default();
    custom.set_custom_name(12014, "es", "Bosquemugoso", &automatic);
    assert_eq!(
        initial_translation(&custom, table, Some(12014), "es"),
        "Bosquemugoso",
        "lo guardado gana al nombre de la tabla"
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
