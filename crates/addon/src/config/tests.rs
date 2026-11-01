//! Tests de la configuración y del estado de las ventanas.

use super::*;

#[test]
fn rf41_settings_live_under_documents_like_the_neighbours() {
    let dir = addon_dir_in(Path::new(r"C:\Users\X\Documentos"));
    assert_eq!(
        dir,
        Path::new(r"C:\Users\X\Documentos\Guild Wars 2\addons\gw2_lurker")
    );
    assert_eq!(
        dir.file_name().and_then(|name| name.to_str()),
        Some(crate::ADDON_DIR),
        "la carpeta lleva el nombre del addon, como timers_wvw o gw2_builds_wvw"
    );
}

#[test]
fn rf19a_popup_time_defaults_to_two_seconds_and_clamps() {
    assert_eq!(Config::default().popup_secs, POPUP_SECS_DEFAULT);
    let mut c = Config {
        popup_secs: 30.0,
        ..Config::default()
    };
    c.sanitize();
    // El tope tiene que ser el mismo que el del control de Ajustes: con 5 s aquí y 20 en la
    // interfaz, cualquier valor por encima de cinco se guardaba como cinco y el jugador lo
    // veía «perdido» tras reiniciar (informe del 2026-11-01).
    assert_eq!(
        c.popup_secs, POPUP_SECS_MAX,
        "como mucho lo que ofrece el control"
    );
    let mut c = Config {
        popup_secs: 12.0,
        ..Config::default()
    };
    c.sanitize();
    assert_eq!(
        c.popup_secs, 12.0,
        "12 s se guarda tal cual: no se pierde al reiniciar"
    );
    let mut c = Config {
        popup_secs: -1.0,
        ..Config::default()
    };
    c.sanitize();
    assert_eq!(c.popup_secs, 0.0, "0 s es válido: sin pop-up");
    let mut c = Config {
        popup_secs: f32::NAN,
        ..Config::default()
    };
    c.sanitize();
    assert_eq!(c.popup_secs, POPUP_SECS_DEFAULT);
}

#[test]
fn rf19a_by_default_my_map_and_the_guilds_i_follow() {
    // Petición del jugador (2026-11-01): de fábrica, «el mapa en el que estoy» y «los clanes que
    // sigo», más las cuatro estructuras. Los cuatro mapas fijos vienen desmarcados.
    let filter = Config::default().alerts;
    assert!(filter.current_map && filter.followed);
    assert!(!filter.ebg && !filter.blue && !filter.green && !filter.red);
    assert!(
        filter.structures.camp
            && filter.structures.tower
            && filter.structures.keep
            && filter.structures.castle
    );
}

#[test]
fn rf41_defaults_are_usable() {
    let c = Config::default();
    assert_eq!(c.team_id, None);
    assert_eq!(
        c.poll_secs, POLL_SECS_DEFAULT,
        "RNF-02: 5 s por petición del jugador"
    );
    assert!(c.widget_locked);
    assert_eq!(c.alert_scope, AlertScope::CurrentMap);
    assert_eq!(c.language, LanguagePreference::Auto);
}

#[test]
fn rf41_old_files_without_fields_still_load() {
    let c: Config = serde_json::from_str(r#"{"team_id": 12014}"#).unwrap();
    assert_eq!(c.team_id, Some(12014));
    assert_eq!(
        c.poll_secs, POLL_SECS_DEFAULT,
        "los campos nuevos toman su valor por defecto"
    );
}

#[test]
fn rf41_out_of_range_values_are_clamped() {
    let mut c = Config {
        poll_secs: 1,
        widget_opacity: 5.0,
        ..Config::default()
    };
    c.sanitize();
    assert_eq!(c.poll_secs, POLL_SECS_MIN);
    assert_eq!(c.widget_opacity, 1.0);

    let mut nan = Config {
        widget_opacity: f32::NAN,
        ..Config::default()
    };
    nan.sanitize();
    assert_eq!(nan.widget_opacity, 0.85, "NaN no puede llegar a imgui");
}

#[test]
fn rf20c_an_index_url_without_https_falls_back_to_the_published_one() {
    // La descarga del índice exige https; una URL escrita a mano sin él dejaría al jugador con
    // un error que no puede arreglar desde la ventana.
    for bad in ["", "   ", "http://ejemplo/x.zip", "ejemplo/x.zip"] {
        let mut c = Config {
            guild_index_url: bad.to_owned(),
            ..Config::default()
        };
        c.sanitize();
        assert_eq!(
            c.guild_index_url,
            crate::guild_index::DEFAULT_URL,
            "{bad:?}"
        );
    }
    let mut good = Config {
        guild_index_url: " https://ejemplo/x.zip ".to_owned(),
        ..Config::default()
    };
    good.sanitize();
    assert_eq!(good.guild_index_url, "https://ejemplo/x.zip");
}

#[test]
fn rf20c_the_broken_default_url_repairs_itself() {
    // La 0.25.0 guardó `gw2addonreleases` (una «s» menos) en el fichero del jugador y con ella
    // GitHub responde 404: «no se pudo descargar… no existe» (informe de 2026-11-01).
    let mut c = Config {
            guild_index_url: "https://raw.githubusercontent.com/cbart-cmd/gw2addonreleases/main/gw2_lurker_guilds.zip".to_owned(),
            ..Config::default()
        };
    c.sanitize();
    assert_eq!(c.guild_index_url, crate::guild_index::DEFAULT_URL);
    assert!(c.guild_index_url.contains("gw2addonsreleases"));
}

#[test]
fn rf20b_guild_names_are_stored_in_the_settings_file() {
    // Los nombres resueltos viven en `settings.json`, no en el índice de búsqueda: es lo que
    // evita clonar miles de entradas en cada sondeo (petición del jugador, 2026-11-01).
    let mut c = Config::default();
    c.guild_names.insert(
        "6C45F3C8-F21C-EF11-81AB-8B4A188BDE13".to_owned(),
        "[YaK] Dolyak Sanctuary".to_owned(),
    );
    let json = serde_json::to_string(&c).unwrap();
    let back: Config = serde_json::from_str(&json).unwrap();
    assert_eq!(
        back.guild_names
            .get("6C45F3C8-F21C-EF11-81AB-8B4A188BDE13")
            .map(String::as_str),
        Some("[YaK] Dolyak Sanctuary")
    );
}

#[test]
fn rf20b_only_the_followed_guilds_keep_their_name_in_the_settings() {
    // Petición del jugador (2026-11-01): en `settings.json` solo los nombres de los clanes que
    // sigue; los de la sesión, en memoria, para que el fichero no crezca sin freno.
    let mut c = Config::default();
    c.followed.add("SEGUIDO");
    c.guild_names
        .insert("SEGUIDO".to_owned(), "[YaK] Dolyak Sanctuary".to_owned());
    c.guild_names
        .insert("DE_PASO".to_owned(), "[DB] Die Barbaren".to_owned());
    c.guild_names
        .insert("OTRO".to_owned(), "[DG] Die Garde".to_owned());
    c.sanitize();
    assert_eq!(c.guild_names.len(), 1, "{:?}", c.guild_names);
    assert_eq!(
        c.guild_names.get("SEGUIDO").map(String::as_str),
        Some("[YaK] Dolyak Sanctuary")
    );
}

#[test]
fn rf03b_configured_team_resolves_its_colour() {
    const MATCH: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../api/tests/fixtures/match_2-2.json"
    ));
    let m: gw2lurker_core::Match = serde_json::from_str(MATCH).unwrap();

    let with_team = Config {
        team_id: Some(12014),
        ..Config::default()
    };
    assert_eq!(with_team.team_color(&m), Some(TeamColor::Blue));

    assert_eq!(
        Config::default().team_color(&m),
        None,
        "sin equipo no hay color"
    );
}

#[test]
fn rf73_followed_guilds_round_trip() {
    let mut c = Config::default();
    c.followed.add("1DB147F0");

    let json = serde_json::to_string(&c).unwrap();
    let back: Config = serde_json::from_str(&json).unwrap();
    assert_eq!(back, c);
    assert!(back.followed.contains("1DB147F0"));
}

#[test]
fn rf46_window_state_defaults_are_visible_and_movable() {
    let mut c = Config::default();
    let state = c.window("Settings");
    assert!(state.visible, "una ventana nueva se ve");
    assert!(!state.locked, "y se puede mover");
    assert!(!state.collapsed);
    assert_eq!(
        state.position, None,
        "la posición la decide el primer frame"
    );
    assert!(state.opacity > 0.5 && state.opacity <= 1.0);
}

#[test]
fn rf46_window_state_survives_a_restart() {
    let mut c = Config::default();
    {
        let state = c.window("Settings");
        state.position = Some([120.0, 340.0]);
        state.size = Some([420.0, 260.0]);
        state.opacity = 0.6;
        state.locked = true;
    }

    let json = serde_json::to_string(&c).unwrap();
    let mut back: Config = serde_json::from_str(&json).unwrap();
    back.sanitize();
    let state = back.window("Settings");
    assert_eq!(state.position, Some([120.0, 340.0]));
    assert_eq!(state.size, Some([420.0, 260.0]));
    assert_eq!(state.opacity, 0.6);
    assert!(state.locked, "bloqueada sigue bloqueada tras reiniciar");
}

#[test]
fn rf46_impossible_window_values_are_discarded() {
    let mut state = WindowState {
        opacity: f32::NAN,
        position: Some([f32::INFINITY, 10.0]),
        size: Some([4.0, 2.0]),
        ..WindowState::default()
    };
    state.sanitize();

    assert_eq!(
        state.opacity,
        WindowState::default().opacity,
        "NaN no llega a imgui"
    );
    assert_eq!(state.position, None, "una posición infinita se descarta");
    assert_eq!(
        state.size,
        Some([120.0, 60.0]),
        "el tamaño mínimo se respeta"
    );
}

#[test]
fn rf46_unknown_window_key_does_not_panic() {
    let mut c: Config = serde_json::from_str(r#"{"windows":{}}"#).unwrap();
    c.window("Settings").opacity = 0.3;
    assert_eq!(c.windows.len(), 1);
}
