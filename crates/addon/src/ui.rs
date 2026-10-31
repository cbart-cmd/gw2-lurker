//! Minimal overlay for T-12: proves the wiring and gives the in-game test something to
//! look at. The alert widget itself is T-13.
//!
//! Every literal comes from the translation table through [`crate::i18n`] (RF-44); the
//! English text is the key, so nothing here needs touching when a language changes.

use nexus::gui::{register_render, render, RenderType};
use nexus::imgui::Window;

use crate::i18n;

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
    let config = crate::config();
    let title = i18n::widget("WvW Lurker");

    Window::new(title).build(ui, || {
        let map_id = state.map_id.to_string();
        let map_type = state.map_type.to_string();
        ui.text(i18n::format("Map {0} (type {1})", &[&map_id, &map_type]));

        match config.team_id {
            Some(id) => ui.text(i18n::format("Team {0}", &[&id.to_string()])),
            None => ui.text(i18n::text("No team configured yet")),
        }
        ui.text(i18n::format(
            "Polling every {0} s",
            &[&config.poll_secs.to_string()],
        ));
    });
}
