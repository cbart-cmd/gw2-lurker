//! Tests of the polling worker.
//!
//! Apart from `worker.rs` so neither file goes past the 800-line limit (AGENTS.md §2.7).

use super::*;
use crate::alerts::MAX_ALERTS;

/// Filtro de «todos los mapas», para los tests que no van de filtros.
///
/// Desde el 2026-11-01 el defecto solo avisa del **mapa en el que estás** y de los clanes que
/// sigues; estos tests pasan `current: None` (no se sabe dónde está el jugador), así que piden los
/// cuatro mapas a propósito para poder mirar lo que de verdad prueban: los `late`, el nombre del
/// objetivo, etc.
fn any_map() -> gw2lurker_core::AlertFilter {
    gw2lurker_core::AlertFilter {
        ebg: true,
        blue: true,
        green: true,
        red: true,
        current_map: false,
        ..gw2lurker_core::AlertFilter::default()
    }
}

/// Full data path against the live API: WinHTTP + parse + the core view models.
/// Ignored by default because CI must not depend on the network.
#[test]
#[ignore = "requiere red"]
fn live_pipeline_builds_the_dashboard() {
    let api = Api::new(WinHttpTransport::default());
    let live = api.matches_all().expect("matches");
    let names = api.objectives("es").expect("objectives");

    let team = 12014;
    let selection = select::evaluate(Some(team), &live);
    let Selection::Confirmed { match_id, .. } = selection else {
        panic!("12014 deberia estar jugando: {selection:?}");
    };

    let entry = api.match_by_id(&match_id).expect("match");
    let rows = matchup::claims(&entry, &names);
    let guilds = matchup::rank_guilds(&rows);

    assert!(!rows.is_empty(), "el matchup real tiene claims");
    assert!(
        rows.iter().any(|row| row.name.is_some()),
        "las filas llevan nombre de objetivo en espanol"
    );
    assert!(!guilds.is_empty());
    assert!(
        guilds
            .windows(2)
            .all(|pair| pair[0].last_at >= pair[1].last_at),
        "el ranking va de mas reciente a menos"
    );
}

#[test]
fn rf03a_the_picker_asks_for_every_matchup_but_not_on_every_poll() {
    // Con el selector en pantalla hacen falta los nueve matchups para llenar la lista, pero la
    // lista solo cambia al reiniciarse el emparejamiento: a 5 s, pedirla en cada sondeo serían
    // 4,8 MB por minuto (informe del jugador, 2026-10-31).
    assert!(
        needs_all_now(true, false, true),
        "selector abierto y lista vieja: se piden los nueve"
    );
    assert!(
        !needs_all_now(true, false, false),
        "selector abierto con la lista reciente: basta una partida"
    );
    assert!(
        !needs_all_now(false, false, false),
        "sin selector basta el matchup del jugador (~44 KB en vez de ~400 KB)"
    );
    assert!(
        !needs_all_now(false, false, false),
        "sin equipo, la lista reciente no se vuelve a pedir en cada sondeo"
    );
    assert!(
        !needs_all_now(false, false, true),
        "sin selector, no hace falta refrescar la lista global"
    );
    assert!(
        needs_all_now(false, true, false),
        "toca buscar la partida del jugador otra vez"
    );
}

#[test]
fn rf03a_a_poll_without_a_known_team_or_match_does_not_fetch_all() {
    assert!(should_skip_poll(false, None, None, true));
    assert!(
        !should_skip_poll(true, None, None, false),
        "la detección inicial sí pide todo"
    );
    assert!(!should_skip_poll(false, Some("2-2"), None, false));
    assert!(!should_skip_poll(false, None, Some(12014), true));
}

#[test]
fn rnf02_only_the_watched_match_is_polled_at_the_configured_cadence() {
    assert!(
        !should_skip_poll(false, Some("2-2"), Some(12014), false),
        "la partida conocida se pide en cada sondeo aunque `ids=all` esté reciente"
    );
    assert!(
        should_skip_poll(false, None, Some(12014), false),
        "sin id de partida, buscar el equipo pide `ids=all` (~400 KB) y espera su enfriamiento"
    );
    assert!(
        !should_skip_poll(false, None, Some(12014), true),
        "pasado el enfriamiento, se vuelve a buscar"
    );
}

#[test]
fn rf03a_the_poll_asks_for_the_team_the_game_gave() {
    // La detección vive cacheada un cuarto de hora: pedir la partida del equipo guardado y
    // evaluar luego el detectado hacía que el addon mirase la partida equivocada entre
    // detecciones (informe de 2026-10-31).
    assert_eq!(polled_team(Some(11011), Some(12010)), Some(12010));
    assert_eq!(
        polled_team(Some(11011), None),
        Some(11011),
        "sin detección se sigue usando el equipo guardado"
    );
    assert_eq!(polled_team(None, Some(12010)), Some(12010));
    assert_eq!(polled_team(None, None), None);
}

#[test]
fn rf03a_only_a_confirmed_selection_has_a_match_to_watch() {
    assert_eq!(selected_match_id(&Selection::Missing), None);
    assert_eq!(
        selected_match_id(&Selection::Stale { previous: 12014 }),
        None
    );
    assert_eq!(
        selected_match_id(&Selection::Confirmed {
            team_id: 12014,
            match_id: "2-2".to_owned()
        }),
        Some("2-2")
    );
    assert!(current_match(&[], None).is_none());
}

#[test]
fn rf03a_a_team_lookup_reuses_the_full_response_when_the_team_is_missing() {
    let live: Vec<Match> =
        serde_json::from_str(include_str!("../../../api/tests/fixtures/matches_all.json")).unwrap();
    let count = live.len();
    assert_eq!(matching_team_or_all(live.clone(), Some(12014)).len(), 1);
    assert_eq!(
        matching_team_or_all(live.clone(), Some(u32::MAX)).len(),
        count,
        "sin el equipo se conserva la misma respuesta para poblar el selector"
    );
    assert_eq!(
        matching_team_or_all(live.clone(), None).len(),
        count,
        "sin selección se reutiliza el listado global"
    );
}

#[test]
fn rf24_assignment_reset_clears_history_and_alerts_and_sets_new_boundary() {
    let at = Utc::now();
    let mut board = Dashboard {
        history: vec![ClaimRow {
            map_id: 38,
            map_role: MapRole::Center,
            objective_id: "38-20".to_owned(),
            name: Some("Tower".to_owned()),
            kind: gw2lurker_core::ObjectiveType::Tower,
            owner: Some(gw2lurker_core::TeamColor::Red),
            guild: "1DB147F0".to_owned(),
            claimed_at: Some(at),
            points_tick: 4,
        }],
        match_id: Some("2-4".to_owned()),
        payload_digest: Some(12),
        payload_since: Some(at),
        ..Dashboard::default()
    };
    board.push_alert(Event::Unclaim {
        objective: "38-20".to_owned(),
        guild: "1DB147F0".to_owned(),
        at,
    });

    board.reset_tracking_cycle(at);

    assert!(board.history.is_empty());
    assert!(board.alerts.is_empty());
    assert_eq!(board.history_since, Some(at));
    assert_eq!(board.match_id, None);
    assert_eq!(board.payload_digest, None);
    assert_eq!(board.payload_since, None);
}

#[test]
fn rf03d_confirmed_assignment_change_resets_the_differ_and_tracking_state() {
    let at = Utc::now();
    let mut board = Dashboard {
        match_id: Some("2-4".to_owned()),
        ..Dashboard::default()
    };
    let mut differ = Differ::new(gw2lurker_core::DiffConfig::default());
    differ.push(gw2lurker_core::diff::Snapshot {
        match_id: "2-4".to_owned(),
        taken_at: at,
        objectives: std::collections::BTreeMap::new(),
    });
    let mut assignment = crate::assignment::AssignmentMonitor::default();

    let reset = crate::assignment::reset_tracking_for_assignment(
        &mut board,
        &mut differ,
        &mut assignment,
        Some("2-4"),
        Some("2-5"),
        at,
    );

    assert!(reset);
    assert!(differ.prev().is_none());
    assert_eq!(board.history_since, Some(at));
    assert_eq!(board.match_id, None);
}

#[test]
fn rf03d_expired_timer_does_not_reset_tracking_while_match_is_active() {
    let at = Utc::now();
    let mut board = Dashboard {
        match_id: Some("2-4".to_owned()),
        history_since: Some(at),
        ..Dashboard::default()
    };
    let mut differ = Differ::new(gw2lurker_core::DiffConfig::default());
    differ.push(gw2lurker_core::diff::Snapshot {
        match_id: "2-4".to_owned(),
        taken_at: at,
        objectives: std::collections::BTreeMap::new(),
    });
    let mut assignment = crate::assignment::AssignmentMonitor::default();

    let reset = crate::assignment::reset_tracking_for_assignment(
        &mut board,
        &mut differ,
        &mut assignment,
        Some("2-4"),
        Some("2-4"),
        at,
    );

    assert!(!reset);
    assert!(differ.prev().is_some());
    assert_eq!(board.history_since, Some(at));
    assert_eq!(board.match_id.as_deref(), Some("2-4"));
}

#[test]
fn rf25_a_restart_does_not_show_yesterday() {
    let mut board = Dashboard::default();
    board.push_alert(Event::Claim {
        objective: "38-20".to_owned(),
        guild: "1DB147F0".to_owned(),
        at: Utc::now(),
        late: true,
    });
    board.push_alert(Event::Claim {
        objective: "38-21".to_owned(),
        guild: "1DB147F0".to_owned(),
        at: Utc::now(),
        late: false,
    });

    let filter = any_map();
    let follow = gw2lurker_core::FollowList::new();
    let lines = board.recent_alerts(&filter, &follow, None, 50);

    assert_eq!(lines.len(), 1, "solo el claim visto en vivo");
    assert_eq!(lines[0].objective, "38-21");
    assert!(!lines[0].late);
}

#[test]
fn rf21_a_followed_guild_is_told_when_the_claim_is_fresh() {
    let now = Utc::now();
    let mut board = Dashboard::default();
    // Reclamado hace dos minutos: nos lo hemos perdido al arrancar, pero es de ahora.
    board.push_alert(Event::Claim {
        objective: "38-20".to_owned(),
        guild: "1DB147F0".to_owned(),
        at: now - chrono::Duration::minutes(2),
        late: true,
    });

    let mut follow = gw2lurker_core::FollowList::new();
    follow.add("1DB147F0");
    let filter = any_map();

    let lines = board.recent_alerts(&filter, &follow, None, 50);
    assert_eq!(
        lines.len(),
        1,
        "un clan seguido avisa de un claim de hace un momento, aunque fuera `late`"
    );
    assert!(lines[0].late);
}

#[test]
fn rf25_late_claim_grace_uses_when_the_addon_received_it() {
    let now = Utc::now();
    let board = Dashboard {
        alerts: vec![
            Received {
                event: Event::Claim {
                    objective: "38-20".to_owned(),
                    guild: "1DB147F0".to_owned(),
                    at: now - chrono::Duration::minutes(2),
                    late: true,
                },
                received: now,
            },
            Received {
                event: Event::Flip {
                    objective: "38-21".to_owned(),
                    from: Some(gw2lurker_core::TeamColor::Green),
                    to: Some(gw2lurker_core::TeamColor::Red),
                    at: now + chrono::Duration::minutes(20),
                },
                received: now + chrono::Duration::minutes(20),
            },
        ],
        ..Dashboard::default()
    };
    let mut follow = gw2lurker_core::FollowList::new();
    follow.add("1DB147F0");

    let lines = board.recent_alerts(&any_map(), &follow, None, 50);
    assert!(
        lines
            .iter()
            .any(|line| line.objective == "38-20" && line.late),
        "un evento posterior y ajeno no debe caducar este claim tardío"
    );
}

#[test]
fn rf25_a_restart_does_not_refill_the_log_with_yesterdays_claims() {
    // El bug de 2026-10-31: al reiniciar, el registro se llenaba con lo que ya estaba
    // reclamado. Un claim viejo no es un aviso, ni siquiera de un clan seguido.
    let now = Utc::now();
    let mut board = Dashboard::default();
    for hours in [1, 3, 26] {
        board.push_alert(Event::Claim {
            objective: format!("38-{hours}"),
            guild: "1DB147F0".to_owned(),
            at: now - chrono::Duration::hours(hours),
            late: true,
        });
    }

    let mut follow = gw2lurker_core::FollowList::new();
    follow.add("1DB147F0");
    let filter = any_map();
    assert!(
        board.recent_alerts(&filter, &follow, None, 50).is_empty(),
        "el registro empieza vacío en cada sesión"
    );
}

#[test]
fn t02_the_poll_asks_for_the_match_it_already_knows() {
    // Un sondeo cada 5 s no puede pedir los nueve emparejamientos (400 KB): con el id conocido
    // se pide **solo** la partida, que ya trae sus cuatro mapas (44 KB).
    assert_eq!(
        fetch_plan(false, Some("1-1"), Some(11011)),
        Fetch::ById("1-1".to_owned())
    );
    assert_eq!(
        fetch_plan(false, None, Some(11011)),
        Fetch::Team(11011),
        "sin partida conocida se busca la del equipo"
    );
    assert_eq!(
        fetch_plan(false, None, None),
        Fetch::All,
        "sin equipo ni partida hay que pedirlo todo"
    );
    assert_eq!(
        fetch_plan(true, Some("1-1"), Some(11011)),
        Fetch::All,
        "el selector en pantalla necesita los nueve"
    );
}

#[test]
fn rf25_the_history_starts_with_what_is_already_claimed() {
    // ADR-012 punto 2: la ruta empieza con el mapa tal y como está. Sin esto, un clan seguido
    // que ya tenía un campa al cargar salía sin historial hasta que volviera a reclamar
    // (informe de 2026-10-31).
    let entry: Match = serde_json::from_str(
        r#"{
            "id": "1-1",
            "scores": {"green": 0, "blue": 0, "red": 0},
            "all_worlds": {"green": [], "blue": [], "red": []},
            "maps": [{
                "id": 38,
                "type": "Center",
                "objectives": [{
                    "id": "38-20", "type": "Camp", "owner": "Blue",
                    "claimed_by": "1DB147F0", "claimed_at": "2026-10-31T22:02:34Z",
                    "last_flipped": "2026-10-31T22:02:32Z"
                }]
            }]
        }"#,
    )
    .expect("match válido");

    let at = Utc::now() - chrono::Duration::hours(2);
    let event = Event::Claim {
        objective: "38-20".to_owned(),
        guild: "1DB147F0".to_owned(),
        at,
        late: true,
    };

    let row = history_row(&event, &entry, &[], &std::collections::BTreeMap::new())
        .expect("el reclamo del baseline entra en el historial");

    assert_eq!(row.objective_id, "38-20");
    assert_eq!(row.guild, "1DB147F0");
    assert_eq!(row.claimed_at, Some(at), "con su hora real, no la de ahora");
    assert_eq!(row.map_role, MapRole::Center);
}

#[test]
fn t51_clicking_a_guild_knows_its_last_map() {
    // Al clicar un clan se lleva el mapa al último donde reclamó algo (petición de
    // 2026-10-31). El id del objetivo ya lleva el mapa delante.
    let at = Utc::now();
    let row = |objective: &str, at| ClaimRow {
        map_id: 96,
        map_role: MapRole::BlueHome,
        objective_id: objective.to_owned(),
        name: Some("Algo".to_owned()),
        kind: gw2lurker_core::ObjectiveType::Tower,
        owner: Some(gw2lurker_core::TeamColor::Red),
        guild: "1DB147F0".to_owned(),
        claimed_at: Some(at),
        points_tick: 4,
    };
    let board = Dashboard {
        history: vec![
            row("38-20", at - chrono::Duration::minutes(30)),
            row("96-40", at - chrono::Duration::minutes(5)),
            row("1099-10", at - chrono::Duration::minutes(20)),
        ],
        ..Dashboard::default()
    };

    assert_eq!(
        board.last_map_of("1DB147F0"),
        Some(MapRole::BlueHome),
        "el más reciente, no el primero"
    );
    assert_eq!(
        board.last_map_of("OTRO"),
        None,
        "un clan sin historial no lleva a ningún mapa"
    );
}

#[test]
fn rf19_alerts_are_bounded_and_skip_baselines() {
    let mut board = Dashboard::default();
    board.push_alert(Event::Baseline {
        reason: gw2lurker_core::BaselineReason::SessionStart,
    });
    assert!(board.alerts.is_empty(), "un baseline no es un aviso");

    for _ in 0..(MAX_ALERTS + 10) {
        board.push_alert(Event::Unclaim {
            objective: "38-20".to_owned(),
            guild: "1DB147F0".to_owned(),
            at: Utc::now(),
        });
    }

    assert_eq!(board.alerts.len(), MAX_ALERTS);
}

#[test]
fn rf33_estela_retains_poll_gaps_without_turning_them_into_alerts() {
    let mut board = Dashboard::default();
    let now = Utc::now();
    let gap = Event::Gap {
        from: now - chrono::Duration::minutes(1),
        to: now,
    };
    for _ in 0..MAX_HISTORY + 10 {
        board.push_alert(gap.clone());
    }
    assert!(board.alerts.is_empty());
    assert_eq!(board.route_events.len(), MAX_HISTORY);
    assert!(board.route_events.iter().all(|e| e == &gap));
    let held = board.route_events.clone();
    board.reset_tracking_cycle(now);
    assert!(board.route_events.is_empty());
    assert_eq!(
        held.len(),
        MAX_HISTORY,
        "the UI snapshot stays valid across a reset"
    );
}
#[test]
fn rf33_estela_worker_prepares_immutable_models_and_keeps_current_baselines() {
    let at = Utc::now();
    let row = ClaimRow {
        guild: "G".to_owned(),
        objective_id: "95-32".to_owned(),
        claimed_at: Some(at),
        map_id: 95,
        map_role: MapRole::GreenHome,
        name: None,
        kind: gw2lurker_core::ObjectiveType::Camp,
        owner: Some(gw2lurker_core::TeamColor::Green),
        points_tick: 0,
    };
    let event = Event::Claim {
        guild: row.guild.clone(),
        objective: row.objective_id.clone(),
        at,
        late: true,
    };
    let prepared = routes::prepare(
        Vec::new(),
        Default::default(),
        Vec::new(),
        &[event],
        &[row],
        &[],
    );
    assert!(
        prepared.history.is_empty(),
        "baseline claims do not invent history events"
    );
    let model = prepared.models.get("G").unwrap();
    assert_eq!(model.steps.len(), 1);
    assert!(model.steps[0].late);
    let mut board = Dashboard {
        route_models: prepared.models,
        ..Dashboard::default()
    };
    let snapshot = board.route_models.get("G").unwrap().clone();
    board.reset_tracking_cycle(at);
    assert!(board.route_models.is_empty());
    assert_eq!(snapshot.steps.len(), 1);
}

#[test]
fn rf38_an_alert_says_which_kind_of_objective_it_is_about() {
    // T-47: la línea lleva el dibujo del objetivo, así que el tipo tiene que llegar hasta
    // ella. `names` es la tabla de objetivos de `/v2/wvw/objectives`.
    let mut board = Dashboard {
        names: vec![ObjectiveInfo {
            id: "38-20".to_owned(),
            name: "Zanja de Mendon".to_owned(),
            kind: gw2lurker_core::ObjectiveType::Tower,
            map_id: 38,
            coord: None,
            label_coord: None,
            marker: None,
        }],
        ..Dashboard::default()
    };
    board.push_alert(Event::Claim {
        objective: "38-20".to_owned(),
        guild: "1DB147F0".to_owned(),
        at: Utc::now(),
        late: false,
    });
    board.push_alert(Event::Claim {
        objective: "38-99".to_owned(),
        guild: "1DB147F0".to_owned(),
        at: Utc::now(),
        late: false,
    });

    let follow = gw2lurker_core::FollowList::new();
    let lines = board.recent_alerts(&any_map(), &follow, None, 50);
    assert_eq!(lines.len(), 2);
    let known = lines
        .iter()
        .find(|line| line.objective == "Zanja de Mendon")
        .expect("el objetivo se nombra");
    assert_eq!(known.objective_kind, gw2lurker_core::ObjectiveType::Tower);
    let unknown = lines
        .iter()
        .find(|line| line.objective == "38-99")
        .expect("un objetivo que no está en la tabla se enseña por su id");
    assert_eq!(
        unknown.objective_kind,
        gw2lurker_core::ObjectiveType::Unknown,
        "sin datos no se inventa un tipo"
    );
}

#[test]
fn rnf04_worker_uses_shared_api_backoff_only_for_retryable_errors() {
    let mut backoff = Backoff::default();
    assert_eq!(
        retry_delay(&mut backoff, &ApiError::Transport("timeout".to_owned())),
        Duration::from_secs(2)
    );
    assert_eq!(
        retry_delay(&mut backoff, &ApiError::Status(429)),
        Duration::from_secs(4)
    );
    assert_eq!(
        retry_delay(&mut backoff, &ApiError::Parse("invalid".to_owned())),
        Duration::ZERO
    );
    assert_eq!(backoff.attempts(), 0, "los errores permanentes reinician");
}
