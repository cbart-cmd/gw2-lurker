//! Shared window contract (RF-46).
//!
//! Every window —status, settings, map, guilds, history, log— draws its chrome through
//! [`draw`]: custom title bar with `×` and double-click collapse, movable and resizable
//! unless locked, opacity applied to the background, and a right-click menu with *Lock
//! position*, *Opacity*, *Reset position and size*.
//!
//! The position comes from the configuration and is pushed every first use; it is read back
//! from the mouse only when the window is not locked, so a lock cannot be undone by dragging.

use nexus::imgui::{Condition, MenuItem, MouseButton, Slider, Ui, Window};

use crate::config::WindowState;
use crate::i18n;

/// Smallest window the user can shrink.
const MIN_SIZE: [f32; 2] = [180.0, 90.0];
/// Size used the first time a window appears.
const DEFAULT_SIZE: [f32; 2] = [320.0, 200.0];
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
    state: &mut WindowState,
    body: impl FnOnce(&Ui),
) -> bool {
    if !state.visible {
        return false;
    }

    let title_text = i18n::text(key);
    let title_id = i18n::widget(key);
    let menu_id = format!("###gw2lurker_menu_{key}");
    let lock_id = i18n::widget("Lock position");
    let opacity_id = i18n::widget("Opacity");
    let reset_id = i18n::widget("Reset position and size");
    let close_id = format!("×###gw2lurker_close_{key}");

    let size = clamp_size(state.size.unwrap_or(DEFAULT_SIZE));
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
                Slider::new("###opacity", 0.0, 1.0)
                    .display_format("%.0f")
                    .build(ui, &mut state.opacity);
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
