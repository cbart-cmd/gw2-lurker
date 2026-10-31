//! Windows of the addon. Every one of them goes through [`window::draw`] (RF-46).
//!
//! Every literal comes from the translation table through [`crate::i18n`] (RF-44); the
//! English text is the key, so nothing here needs touching when a language changes.

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
            }
            if gate.settings_available() || crate::config().debug_show_everywhere {
                let change = {
                    let mut config = crate::runtime();
                    settings::draw(ui, &mut config)
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
            let change = settings::draw_inline(ui);
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
    let team_id = crate::team::selected();
    let team_alias = crate::config().team_alias.clone();
    let poll_secs = crate::config().poll_secs;
    let mut open_settings = false;
    let mut config = crate::runtime();

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

        match team_id {
            Some(id) => {
                let label = match team_alias.clone() {
                    Some(alias) => alias,
                    None => gw2lurker_core::bundled().label_in(i18n::effective().code(), id),
                };
                ui.text(i18n::format("Team {0}", &[&label]));
            }
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
