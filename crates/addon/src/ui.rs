//! Minimal overlay for T-12: proves the wiring and gives the in-game test something to
//! look at. The alert widget itself is T-13.
//!
//! Text is in English on purpose: per RF-44 the English string is the translation key,
//! so T-31 only has to move these literals into the table.

use nexus::gui::{register_render, render, RenderType};
use nexus::imgui::Window;

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

    Window::new("WvW Lurker##gw2_lurker_status").build(ui, || {
        ui.text(format!("map {} (type {})", state.map_id, state.map_type));
        match config.team_id {
            Some(id) => ui.text(format!("team {id}")),
            None => ui.text("no team configured yet"),
        }
        ui.text(format!("poll every {} s", config.poll_secs));
        ui.separator();
        ui.text("alerts and map views land in T-13 / T-36");
    });
}
