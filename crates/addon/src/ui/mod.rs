//! Windows of the addon. Every one of them goes through [`window::draw`] (RF-46).
//!
//! Every literal comes from the translation table through [`crate::i18n`] (RF-44); the
//! English text is the key, so nothing here needs touching when a language changes.

mod alerts;
mod matchup;
mod settings;
mod window;

use nexus::gui::{register_render, render, RenderType};

use crate::i18n;

/// Translation key (and stable id) of the status window.
const STATUS: &str = "WvW Lurker";

/// Registers the render callbacks. Must be called from `load`, never per frame.
pub fn register() {
    register_render(
        RenderType::Render,
        render!(|ui| {
            i18n::register_translations_once();
            let gate = crate::context::gate();
            crate::context::log_gate(gate);
            crate::worker::set_may_poll(gate.poll);

            if gate.show_ui || crate::config().debug_show_everywhere {
                draw_status(ui);
                draw_matchup(ui);
                draw_alerts(ui);
            }
            // The team picker needs the list of live teams even outside WvW, where polling is
            // off: without this the drop-down was empty (bug of 2026-10-31).
            crate::worker::set_teams_needed(
                gate.settings_available() || crate::config().debug_show_everywhere,
            );
            if gate.settings_available() || crate::config().debug_show_everywhere {
                // Candidates and guilds come from the worker and are copied out before
                // touching the configuration: two mutexes are involved and the dashboard lock
                // must never be held while the configuration one is taken.
                let (candidates, guilds) = {
                    let board = crate::worker::dashboard();
                    (board.candidates.clone(), board.guilds.clone())
                };
                let change = {
                    let mut config = crate::runtime();
                    settings::draw(ui, &mut config, &candidates, &guilds)
                };
                match change {
                    Some(settings::TeamChange::Pick(team_id)) => crate::team::remember(team_id),
                    Some(settings::TeamChange::Clear) => crate::team::forget(),
                    None => {}
                }
            }
            crate::persist_if_changed();
        }),
    )
    .revert_on_unload();

    register_render(
        RenderType::OptionsRender,
        render!(|ui| {
            let (candidates, guilds) = {
                let board = crate::worker::dashboard();
                (board.candidates.clone(), board.guilds.clone())
            };
            let change = settings::draw_inline(ui, &candidates, &guilds);
            match change {
                Some(settings::TeamChange::Pick(team_id)) => crate::team::remember(team_id),
                Some(settings::TeamChange::Clear) => crate::team::forget(),
                None => {}
            }
        }),
    )
    .revert_on_unload();
}

fn draw_status(ui: &nexus::imgui::Ui) {
    let state = crate::context::snapshot();
    let mut config = crate::runtime();
    let team_id = crate::team::selected_in(&config);

    let poll_secs = config.poll_secs;
    let team_label =
        team_id.map(|id| crate::team::display_name(&config, i18n::effective().code(), id));
    let mut open_settings = false;

    // The worker's view, read under a short lock: nothing here touches the network or disks.
    let (health, match_id, claims, guilds, updated) = {
        let board = crate::worker::dashboard();
        (
            board.health.clone(),
            board.match_id.clone(),
            board.claims.len(),
            board.guilds.len(),
            board.updated_at,
        )
    };

    let window_state = config.window(STATUS);
    window::draw(ui, STATUS, 0, [300.0, 170.0], window_state, |ui| {
        let map_id = state.map_id.to_string();
        let map_type = state.map_type.to_string();
        ui.text(i18n::format("Map {0} (type {1})", &[&map_id, &map_type]));

        match &team_label {
            Some(label) => ui.text(i18n::format("Team {0}", &[label])),
            None => ui.text(i18n::text("No team configured yet")),
        }

        if let Some(id) = &match_id {
            ui.text(i18n::format("Matchup {0}", &[id]));
        }
        match &health {
            crate::worker::Health::Idle => ui.text(i18n::text("No data yet")),
            crate::worker::Health::Ok { .. } => {
                let claims = claims.to_string();
                let guilds = guilds.to_string();
                ui.text(i18n::format("{0} claimed objectives", &[&claims]));
                ui.text(i18n::format("{0} guilds", &[&guilds]));
            }
            crate::worker::Health::Degraded { error, .. } => {
                ui.text(i18n::format("Error: {0}", &[error]));
            }
        }
        if let Some(at) = updated {
            ui.text(i18n::format(
                "Last update {0}",
                &[&at.format("%H:%M:%S").to_string()],
            ));
        }

        ui.separator();
        ui.text(i18n::format(
            "Polling every {0} s",
            &[&poll_secs.to_string()],
        ));
        open_settings = ui.button(i18n::widget("Settings"));
    });

    if open_settings {
        config.window(settings::KEY).visible = true;
    }
}

/// Captures and claims of the whole matchup (T-36). The worker owns the data; this only draws
/// and keeps the filter boxes of the session.
fn draw_matchup(ui: &nexus::imgui::Ui) {
    static FILTERS: std::sync::OnceLock<std::sync::Mutex<matchup::Filters>> =
        std::sync::OnceLock::new();
    let filters = FILTERS.get_or_init(|| std::sync::Mutex::new(matchup::Filters::all_kinds()));

    let rows = crate::worker::dashboard().claims.clone();
    let current = crate::context::current_map_role();
    let mut filters = filters.lock().unwrap_or_else(|error| error.into_inner());
    let mut config = crate::runtime();
    let state = config.window(matchup::KEY);
    matchup::draw(ui, state, &mut filters, &rows, current);
}

/// Recent captures and claims (T-13). The worker resolves and filters the lines, so the render
/// callback only formats what it already has.
fn draw_alerts(ui: &nexus::imgui::Ui) {
    let (filter, follow, popup_secs, opacity) = {
        let mut config = crate::runtime();
        (
            config.alerts,
            config.followed.clone(),
            config.popup_secs,
            config.window(alerts::KEY).opacity,
        )
    };
    let current = crate::context::current_map_role();
    let lines = crate::worker::dashboard().recent_alerts(&filter, &follow, current, 30);

    let mut config = crate::runtime();
    let state = config.window(alerts::KEY);
    alerts::draw(ui, state, &lines);
    drop(config);

    let showing = alerts::popup_lines(&lines, chrono::Utc::now(), popup_secs);
    alerts::draw_popup(ui, &showing, opacity);
}
