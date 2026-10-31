//! Estilo de las ventanas del addon (T-28, RF-60 a RF-63).
//!
//! Por defecto **no se toca nada**: las ventanas heredan el estilo que aplica Nexus, que es el que
//! el jugador ya tiene en el resto del juego (RF-60). Con el tema propio se empuja una pila de
//! estilo alrededor de **nuestras** ventanas y se saca al terminar, así que Nexus y los demás addons
//! no se enteran (RF-61).
//!
//! Los colores de equipo son fijos y los mismos en todas partes (RF-62): el color no puede ser la
//! única información, por eso cada objetivo tiene además su dibujo y cada aviso su texto.

use gw2lurker_core::model::TeamColor;
use nexus::imgui::{ColorStackToken, StyleColor, StyleStackToken, StyleVar, Ui};

use crate::config::Theme;

/// Colores del tema propio: gris azulado oscuro, texto claro y acento azul.
///
/// Es el mismo tono que el fondo del mapa, para que el addon se vea como una sola cosa.
const OWN_COLORS: [(StyleColor, [f32; 4]); 20] = [
    (StyleColor::Text, [0.90, 0.92, 0.95, 1.00]),
    (StyleColor::TextDisabled, [0.55, 0.58, 0.62, 1.00]),
    (StyleColor::WindowBg, [0.06, 0.07, 0.09, 0.92]),
    (StyleColor::ChildBg, [0.04, 0.05, 0.07, 0.85]),
    (StyleColor::PopupBg, [0.05, 0.06, 0.08, 0.97]),
    (StyleColor::Border, [0.24, 0.27, 0.32, 0.60]),
    (StyleColor::FrameBg, [0.10, 0.12, 0.15, 1.00]),
    (StyleColor::FrameBgHovered, [0.15, 0.18, 0.23, 1.00]),
    (StyleColor::FrameBgActive, [0.18, 0.22, 0.28, 1.00]),
    (StyleColor::TitleBg, [0.08, 0.10, 0.13, 1.00]),
    (StyleColor::TitleBgActive, [0.11, 0.14, 0.18, 1.00]),
    (StyleColor::MenuBarBg, [0.07, 0.09, 0.11, 1.00]),
    (StyleColor::ScrollbarBg, [0.05, 0.06, 0.08, 0.60]),
    (StyleColor::ScrollbarGrab, [0.24, 0.28, 0.34, 0.80]),
    (StyleColor::CheckMark, [0.42, 0.72, 1.00, 1.00]),
    (StyleColor::SliderGrab, [0.35, 0.60, 0.90, 1.00]),
    (StyleColor::SliderGrabActive, [0.45, 0.72, 1.00, 1.00]),
    (StyleColor::Button, [0.14, 0.17, 0.22, 1.00]),
    (StyleColor::ButtonHovered, [0.20, 0.25, 0.32, 1.00]),
    (StyleColor::ButtonActive, [0.26, 0.33, 0.42, 1.00]),
];

/// Estilos del tema propio: esquinas suaves y algo de aire. Nada global.
const OWN_VARS: [StyleVar; 5] = [
    StyleVar::WindowRounding(6.0),
    StyleVar::FrameRounding(4.0),
    StyleVar::ChildRounding(4.0),
    StyleVar::WindowPadding([8.0, 6.0]),
    StyleVar::ItemSpacing([7.0, 5.0]),
];

/// Pila de estilo viva mientras se dibuja una ventana del addon.
///
/// Los `push` de esta versión de imgui devuelven un token que saca lo empujado al caer: no hay
/// `pop_style_color`, y llevar la cuenta a mano dejaría la pila descuadrada al primer `return`.
#[must_use = "si se descarta en el acto, el tema no llega a aplicarse"]
pub struct StyleGuard<'ui> {
    _colors: Vec<ColorStackToken<'ui>>,
    _vars: Vec<StyleStackToken<'ui>>,
}

/// Aplica el tema pedido a lo que se dibuje dentro de la ventana actual (RF-60, RF-61).
///
/// Con [`Theme::Inherited`] no empuja nada.
pub fn apply<'ui>(ui: &'ui Ui, theme: Theme) -> StyleGuard<'ui> {
    if theme == Theme::Inherited {
        return StyleGuard {
            _colors: Vec::new(),
            _vars: Vec::new(),
        };
    }
    StyleGuard {
        _colors: OWN_COLORS
            .iter()
            .map(|(colour, value)| ui.push_style_color(*colour, *value))
            .collect(),
        _vars: OWN_VARS.iter().map(|var| ui.push_style_var(*var)).collect(),
    }
}

/// Color de cada equipo, **el mismo en el widget, en el mapa y en el historial** (RF-62).
pub fn team_colour(colour: TeamColor) -> [f32; 4] {
    match colour {
        TeamColor::Red => [1.00, 0.48, 0.42, 1.00],
        TeamColor::Green => [0.58, 0.94, 0.58, 1.00],
        TeamColor::Blue => [0.50, 0.74, 1.00, 1.00],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rf62_the_three_teams_have_distinct_colours() {
        let red = team_colour(TeamColor::Red);
        let green = team_colour(TeamColor::Green);
        let blue = team_colour(TeamColor::Blue);
        assert_ne!(red, green);
        assert_ne!(green, blue);
        assert_ne!(red, blue);
        assert!(red[0] > red[2], "el rojo es rojo");
        assert!(blue[2] > blue[0], "el azul es azul");
        assert!(
            green[1] > green[0] && green[1] > green[2],
            "el verde es verde"
        );
    }

    #[test]
    fn rf61_every_style_colour_is_named_once() {
        // Un color repetido no rompe nada, pero indica copia-pega: el tema tiene que ser pequeño y
        // legible.
        for (index, (colour, _)) in OWN_COLORS.iter().enumerate() {
            for (other, _) in OWN_COLORS.iter().skip(index + 1) {
                assert_ne!(colour, other, "color repetido en el tema propio");
            }
        }
    }
}
