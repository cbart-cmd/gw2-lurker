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

            if gate.show_ui || crate::config().debug_show_everywhere {
                draw_status(ui);
            }
            if gate.settings_available() || crate::config().debug_show_everywhere {
                settings::draw(ui, &mut crate::runtime());
            }
            crate::persist_if_changed();
        }),
    )
    .revert_on_unload();

    register_render(
        RenderType::OptionsRender,
        render!(|ui| settings::draw_inline(ui)),
    )
    .revert_on_unload();
}

fn draw_status(ui: &nexus::imgui::Ui) {
    let state = crate::context::snapshot();
    let mut config = crate::runtime();
    let team_id = config.team_id;
    let team_alias = config.team_alias.clone();
    let poll_secs = config.poll_secs;
    let mut open_settings = false;
    let window_state = config.window(STATUS);

    window::draw(ui, STATUS, 0, [300.0, 150.0], window_state, |ui| {
        let map_id = state.map_id.to_string();
        let map_type = state.map_type.to_string();
        ui.text(i18n::format("Map {0} (type {1})", &[&map_id, &map_type]));

        match team_id {
            Some(id) => {
                let label = match team_alias {
                    Some(alias) => alias,
                    None => gw2lurker_core::bundled().label(id),
                };
                ui.text(i18n::format("Team {0}", &[&label]));
            }
            None => ui.text(i18n::text("No team configured yet")),
        }
        ui.text(i18n::format(
            "Polling every {0} s",
            &[&poll_secs.to_string()],
        ));
        ui.separator();
        open_settings = ui.button(i18n::widget("Settings"));
    });

    if open_settings {
        config.window(settings::KEY).visible = true;
    }
}
