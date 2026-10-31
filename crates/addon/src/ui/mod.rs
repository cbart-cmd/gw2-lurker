//! Windows of the addon. Every one of them goes through [`window::draw`] (RF-46).
//!
//! Every literal comes from the translation table through [`crate::i18n`] (RF-44); the
//! English text is the key, so nothing here needs touching when a language changes.

mod window;

use nexus::gui::{register_render, render, RenderType};

use crate::i18n;

/// Translation key (and stable id) of the status window.
const STATUS: &str = "WvW Lurker";

/// Registers the render callback. Must be called from `load`, never per frame.
pub fn register() {
    register_render(
        RenderType::Render,
        render!(|ui| {
            let gate = crate::context::gate();
            if !gate.show_ui {
                return;
            }
            draw_status(ui);
        }),
    )
    .revert_on_unload();
}

fn draw_status(ui: &nexus::imgui::Ui) {
    let state = crate::context::snapshot();
    let mut config = crate::runtime();
    let team_id = config.team_id;
    let poll_secs = config.poll_secs;
    let window_state = config.window(STATUS);

    window::draw(ui, STATUS, 0, window_state, |ui| {
        let map_id = state.map_id.to_string();
        let map_type = state.map_type.to_string();
        ui.text(i18n::format("Map {0} (type {1})", &[&map_id, &map_type]));

        match team_id {
            Some(id) => ui.text(i18n::format("Team {0}", &[&id.to_string()])),
            None => ui.text(i18n::text("No team configured yet")),
        }
        ui.text(i18n::format(
            "Polling every {0} s",
            &[&poll_secs.to_string()],
        ));
    });
}
