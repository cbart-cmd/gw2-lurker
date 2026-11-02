//! Tests of the settings window: the translated names (RF-03f).
//!
//! The picker and its tests live in `team_picker` since 2026-11-02 (RF-03j).

use super::*;

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
