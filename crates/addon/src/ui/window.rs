//! Shared window contract (RF-46).
//!
//! Every window —status, settings, map, guilds, history, log— draws its chrome through
//! [`draw`]: custom title bar with `×` and double-click collapse, movable and resizable
//! unless locked, opacity applied to the background, and a right-click menu with
//! *Click-through*, *Opacity*, *Reset position and size*.
//!
//! The position comes from the configuration and is pushed every first use; it is read back
//! from the mouse only when the window is not locked, so a lock cannot be undone by dragging.
//!
//! Locked means **click-through**: the window keeps drawing but imgui gives it no mouse input
//! at all (`NO_INPUTS`), so during a fight every click goes to the game. The way back is the
//! `Alt+Shift+L` shortcut or the *Windows* section of the settings (RF-18).

use nexus::imgui::{Condition, MenuItem, MouseButton, Slider, Ui, Window};

use crate::config::WindowState;
use crate::i18n;

/// Smallest window the user can shrink.
const MIN_SIZE: [f32; 2] = [180.0, 90.0];
/// Width reserved for the close button.
const CLOSE_WIDTH: f32 = 24.0;

fn default_position(slot: usize) -> [f32; 2] {
    let offset = 30.0 * slot as f32;
    [60.0 + offset, 120.0 + offset]
}

/// Draws one window and returns `false` when the user closed it this frame.
pub fn draw(
    ui: &Ui,
    key: &'static str,
    slot: usize,
    default_size: [f32; 2],
    state: &mut WindowState,
    body: impl FnOnce(&Ui),
) -> bool {
    if !state.visible {
        return false;
    }

    let title_text = i18n::text(key);
    let title_id = i18n::widget(key);
    let menu_id = format!("###gw2lurker_menu_{key}");
    let lock_id = i18n::widget("Click-through (no mouse)");
    let opacity_id = i18n::widget("Opacity");
    let reset_id = i18n::widget("Reset position and size");
    let close_id = format!("×###gw2lurker_close_{key}");

    let size = clamp_size(state.size.unwrap_or(default_size));
    let position = state.position.unwrap_or_else(|| default_position(slot));

    let mut window = Window::new(title_id)
        .title_bar(false)
        .movable(!state.locked)
        .resizable(!state.locked)
        .collapsed(false, Condition::Always)
        .size(size, Condition::FirstUseEver)
        .position(position, Condition::FirstUseEver)
        .size_constraints(MIN_SIZE, [4096.0, 4096.0])
        .bg_alpha(state.opacity);
    if state.collapsed {
        window = window.always_auto_resize(true);
    }
    if state.locked {
        window = window.no_inputs();
    }

    window.build(ui, || {
        let start = ui.cursor_pos();
        let width = ui.content_region_avail()[0];
        if width > CLOSE_WIDTH * 2.0 {
            ui.set_cursor_pos([start[0] + width - CLOSE_WIDTH, start[1]]);
            if ui.small_button(close_id) {
                state.visible = false;
            }
            ui.set_cursor_pos(start);
        } else if ui.small_button(close_id) {
            state.visible = false;
        }

        ui.text(title_text);
        if ui.is_item_hovered() && ui.is_mouse_double_clicked(MouseButton::Left) {
            state.collapsed = !state.collapsed;
        }

        if ui.is_window_hovered() && ui.is_mouse_clicked(MouseButton::Right) {
            ui.open_popup(&menu_id);
        }
        if let Some(_menu) = ui.begin_popup(&menu_id) {
            MenuItem::new(lock_id)
                .selected(state.locked)
                .build_with_ref(ui, &mut state.locked);
            ui.menu(opacity_id, || {
                let mut percent = opacity_percent(state.opacity);
                if Slider::new("###opacity", 0.0, 100.0)
                    .display_format("%.0f%%")
                    .build(ui, &mut percent)
                {
                    state.opacity = opacity_from_percent(percent);
                }
            });
            if MenuItem::new(reset_id).build(ui) {
                state.position = None;
                state.size = None;
            }
        }

        if !state.collapsed {
            ui.separator();
            body(ui);
        }
    });

    if !state.locked {
        state.position = Some(ui.window_pos());
        if !state.collapsed {
            state.size = Some(clamp_size(ui.window_size()));
        }
    }

    state.visible
}

fn clamp_size(size: [f32; 2]) -> [f32; 2] {
    [
        size[0].clamp(MIN_SIZE[0], 4096.0),
        size[1].clamp(MIN_SIZE[1], 4096.0),
    ]
}

/// Opacity as whole percent, which is how the user thinks about it (0–100).
///
/// The configuration keeps `0.0–1.0` because that is what imgui's `bg_alpha` takes; the
/// slider shows percent so nobody has to reason in fractions. `bg_alpha` only affects the
/// window background, so 0 % still leaves the text readable.
pub fn opacity_percent(opacity: f32) -> f32 {
    if opacity.is_finite() {
        (opacity * 100.0).round().clamp(0.0, 100.0)
    } else {
        WindowState::default().opacity * 100.0
    }
}

/// Back to the 0.0–1.0 stored in the configuration.
pub fn opacity_from_percent(percent: f32) -> f32 {
    if percent.is_finite() {
        (percent / 100.0).clamp(0.0, 1.0)
    } else {
        WindowState::default().opacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rf46_opacity_is_shown_as_percent() {
        assert_eq!(opacity_percent(0.85), 85.0);
        assert_eq!(opacity_percent(0.0), 0.0, "0 % es válido");
        assert_eq!(opacity_percent(1.0), 100.0, "100 % es válido");
        assert_eq!(opacity_percent(0.604), 60.0, "se redondea, no se trunca");
        assert_eq!(
            opacity_percent(f32::NAN),
            90.0,
            "un NaN no puede llegar al slider"
        );
    }

    #[test]
    fn rf46_percent_round_trip_keeps_the_stored_value() {
        for percent in [0.0, 5.0, 60.0, 100.0] {
            let stored = opacity_from_percent(percent);
            assert!((0.0..=1.0).contains(&stored));
            assert_eq!(opacity_percent(stored), percent, "ida y vuelta");
        }
        assert_eq!(opacity_from_percent(150.0), 1.0, "se recorta al rango");
        assert_eq!(opacity_from_percent(-5.0), 0.0);
        assert_eq!(opacity_from_percent(f32::INFINITY), 0.9);
    }
}
