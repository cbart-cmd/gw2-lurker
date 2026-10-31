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
//! Two different switches (bug of 2026-10-31, they used to be the same one):
//!
//! - **Locked**: cannot be moved or resized, but it still answers the mouse.
//! - **Click-through**: the window keeps drawing and imgui gives it no mouse input at all
//!   (`NO_INPUTS`), so during a fight every click goes to the game. The way back is the
//!   shortcut or the Nexus options (RF-18).

use nexus::imgui::{
    ChildWindow, Condition, MenuItem, MouseButton, Slider, Ui, Window, WindowHoveredFlags,
};

use crate::config::WindowState;
use crate::i18n;

/// Smallest window the user can shrink.
const MIN_SIZE: [f32; 2] = [180.0, 90.0];
/// Smallest body region, so a window shrunk to nothing still has somewhere to draw.
const MIN_BODY_HEIGHT: f32 = 24.0;
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
    let lock_id = i18n::widget("Lock position");
    let through_id = i18n::widget("Click-through (no mouse)");
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
        .bg_alpha(bg_alpha(state.opacity));
    if state.collapsed {
        window = window.always_auto_resize(true);
    }
    if state.click_through {
        window = window.no_inputs();
    }

    window.build(ui, || {
        // RF-60/RF-61: estilo heredado de Nexus (nada) o el tema propio, **solo** dentro de esta
        // ventana.
        let _style = super::theme::apply(ui, crate::config().theme);
        if crate::config().font_scale > 0.0 {
            ui.set_window_font_scale(crate::config().font_scale.clamp(0.8, 1.6));
        }
        let start = ui.cursor_pos();
        let width = ui.content_region_avail()[0];
        // El botón de cerrar va **primero** y solo después el título: con una barra invisible a
        // toda la anchura se quedaba con el clic y la ventana no se podía arrastrar ni cerrar
        // (informe de 2026-10-31).
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
        // El texto del título no captura el arrastre, así que la ventana se mueve como siempre;
        // sobre él, doble clic contrae/despliega y un clic la abre si estaba contrada.
        if ui.is_item_hovered() {
            if ui.is_mouse_double_clicked(MouseButton::Left) {
                state.collapsed = !state.collapsed;
            } else if state.collapsed && ui.is_mouse_clicked(MouseButton::Left) {
                state.collapsed = false;
            }
        }

        // Con el cuerpo dentro de una ventana hija, `is_window_hovered()` a secas diría que no
        // mientras el ratón está sobre el cuerpo, que es casi todo el widget: hay que pedir
        // `CHILD_WINDOWS` o el menú del clic derecho solo abriría en el borde.
        if ui.is_window_hovered_with_flags(WindowHoveredFlags::CHILD_WINDOWS)
            && ui.is_mouse_clicked(MouseButton::Right)
        {
            ui.open_popup(&menu_id);
        }
        if let Some(_menu) = ui.begin_popup(&menu_id) {
            MenuItem::new(lock_id)
                .selected(state.locked)
                .build_with_ref(ui, &mut state.locked);
            MenuItem::new(through_id)
                .selected(state.click_through)
                .build_with_ref(ui, &mut state.click_through);
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
            // El cuerpo va en una ventana hija con su propia barra: si scrollease la ventana
            // entera, una pestaña larga (Ajustes) se llevaba la cabecera fuera y el `×` y el menú
            // del clic derecho dejaban de estar a mano (informe de 2026-10-31).
            let spacing = ui.clone_style().item_spacing[1];
            let avail = ui.content_region_avail();
            let height = (avail[1] - spacing).max(MIN_BODY_HEIGHT);
            let width = avail[0].max(1.0);
            let body_id = format!("###gw2lurker_body_{key}");
            let mut child = ChildWindow::new(body_id.as_str())
                .size([width, height])
                .scrollable(true)
                .draw_background(false)
                .border(false);
            // `no_inputs` en la ventana **no** lo hereda la hija: sin esto, con click-through
            // activado el cuerpo seguía comiéndose los clics (informe de 2026-10-31, «lo marco y
            // sigue sin hacer nada»).
            if state.click_through {
                child = child.no_inputs();
            }
            child.build(ui, || body(ui));
        }
    });

    // The box is recorded even in click-through: it cannot be dragged there, so there is nothing
    // to fight over, and the right-click escape needs to know where the window is (T-49). A
    // locked window still keeps its box, so the lock cannot be undone by dragging.
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

/// Opacidad del fondo de la ventana según el ajuste (RF-63).
///
/// El fondo transparente deja solo el texto: la opacidad de la ventana sigue mandando, así que se
/// puede tener «sin fondo» y aun así atenuado.
fn bg_alpha(opacity: f32) -> f32 {
    match crate::config().widget_background {
        crate::config::WidgetBackground::Dark => opacity.clamp(0.0, 1.0),
        crate::config::WidgetBackground::Transparent => 0.0,
    }
}

/// Whether a point falls inside a window drawn at `position` with that `size`.
///
/// Click-through means imgui gives the window **no** input at all (`NO_INPUTS`), so the only way
/// to notice the mouse is to look at it from outside: this is the test the right-click escape
/// uses (request of 2026-10-31).
pub fn covers(position: Option<[f32; 2]>, size: Option<[f32; 2]>, point: [f32; 2]) -> bool {
    let (Some(position), Some(size)) = (position, size) else {
        // Sin caja guardada no se adivina: mejor no quitar el click-through por accidente.
        return false;
    };
    let size = clamp_size(size);
    point[0] >= position[0]
        && point[1] >= position[1]
        && point[0] <= position[0] + size[0]
        && point[1] <= position[1] + size[1]
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

    #[test]
    fn rf18_a_right_click_on_the_widget_is_noticed_even_with_click_through() {
        // Con `NO_INPUTS` la ventana no recibe el ratón, así que el clic derecho se busca desde
        // fuera: dentro de su caja cuenta, fuera no.
        let position = Some([100.0, 200.0]);
        let size = Some([400.0, 480.0]);
        assert!(covers(position, size, [300.0, 400.0]), "en el centro");
        assert!(covers(position, size, [100.0, 200.0]), "en la esquina");
        assert!(
            !covers(position, size, [99.0, 400.0]),
            "un píxel a la izquierda"
        );
        assert!(!covers(position, size, [300.0, 681.0]), "por debajo");
        assert!(
            !covers(None, size, [300.0, 400.0]),
            "sin caja guardada no se toca nada"
        );
        assert!(
            !covers(position, None, [300.0, 400.0]),
            "sin tamaño tampoco, para no desactivarlo por accidente"
        );
    }
}
