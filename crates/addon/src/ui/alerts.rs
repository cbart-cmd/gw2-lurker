//! Alert widget (T-13, RF-19): the recent captures and claims, newest first.
//!
//! Two surfaces from the same data: the **window**, which keeps everything the filter accepts so
//! it can be read later, and the **pop-up**, which shows only what arrived in the last few
//! seconds (0–5, configurable) and then disappears without removing anything from the window
//! (RF-19a).
//!
//! It only draws; the worker owns the events and the store keeps the history. Lines are
//! resolved and filtered by the worker (`Dashboard::recent_alerts`), so this module never
//! searches the objective table and no lock is held while drawing.

use chrono::{DateTime, Utc};
use gw2lurker_core::model::TeamColor;
use nexus::imgui::{Condition, StyleColor, Ui, Window};

use crate::i18n;
use crate::worker::{AlertKind, AlertLine};

/// Translation key (and stable id) of this window.
pub const KEY: &str = "Alerts";

/// Translation key (and stable id) of the expanded log window.
pub const LOG_KEY: &str = "Alert log";

/// Translation key of the pop-up: separate window, separate id, state never saved.
const POPUP_KEY: &str = "Alerts (pop-up)";

/// How many lines fit in the window without scrolling forever.
const MAX_LINES: usize = 25;

/// How many lines the pop-up shows at once.
const MAX_POPUP_LINES: usize = 5;

/// Draws the window. `window_state` comes from `config.json` like every other window.
pub fn draw(ui: &Ui, window_state: &mut crate::config::WindowState, alerts: &[AlertLine]) -> bool {
    crate::ui::window::draw(ui, KEY, 2, [360.0, 240.0], window_state, |ui| {
        if alerts.is_empty() {
            ui.text_disabled(i18n::text("No alerts yet"));
            return;
        }
        let language = i18n::effective();
        for line in alerts.iter().take(MAX_LINES) {
            draw_line(ui, line, language);
        }
    })
}

/// Draws the expanded log: every alert the worker still remembers, scrolling if needed.
///
/// Same data and same lines as [`draw`], without the `MAX_LINES` cut, in a bigger window: during
/// a fight the compact one is what you glance at, after it the log is what you read.
pub fn draw_log(
    ui: &Ui,
    window_state: &mut crate::config::WindowState,
    alerts: &[AlertLine],
) -> bool {
    crate::ui::window::draw(ui, LOG_KEY, 5, [440.0, 420.0], window_state, |ui| {
        if alerts.is_empty() {
            ui.text_disabled(i18n::text("No alerts yet"));
            return;
        }
        let language = i18n::effective();
        for line in alerts {
            draw_line(ui, line, language);
        }
    })
}

/// Lines that still deserve a pop-up: the ones received less than `secs` ago.
///
/// Pure on purpose (`now` is a parameter): the render callback needs it every frame and the
/// rule has to be testable without a game. `0` seconds means no pop-up at all.
pub fn popup_lines(lines: &[AlertLine], now: DateTime<Utc>, secs: f32) -> Vec<AlertLine> {
    // `!(secs > 0.0)` would also be true for NaN; make the NaN case explicit instead.
    if secs.is_nan() || secs <= 0.0 {
        return Vec::new();
    }
    let window = chrono::Duration::milliseconds((f64::from(secs) * 1000.0) as i64);
    lines
        .iter()
        .filter(|line| !line.late && now.signed_duration_since(line.received) < window)
        .take(MAX_POPUP_LINES)
        .cloned()
        .collect()
}

/// Draws the transient pop-up, top centre of the screen.
pub fn draw_popup(ui: &Ui, lines: &[AlertLine], opacity: f32) {
    if lines.is_empty() {
        return;
    }
    let [width, _] = ui.io().display_size;
    let language = i18n::effective();

    Window::new(i18n::widget(POPUP_KEY))
        .title_bar(false)
        .resizable(false)
        .movable(false)
        .always_auto_resize(true)
        .position([(width - 420.0) / 2.0, 90.0], Condition::Always)
        .bg_alpha(opacity)
        .build(ui, || {
            for line in lines {
                draw_line(ui, line, language);
            }
        });
}

fn draw_line(ui: &Ui, line: &AlertLine, language: gw2lurker_core::i18n::Language) {
    let time = line.at.format("%H:%M:%S").to_string();
    let _tint = ui.push_style_color(StyleColor::Text, colour_of(line));
    let text = describe(line, language);
    ui.text(format!("{time}  {text}"));
    if line.late && ui.is_item_hovered() {
        ui.tooltip_text(i18n::text("Already claimed when the addon started"));
    }
}

/// One line of text, translated (RF-44). The English original is the key.
///
/// The language is a parameter: reading the active one goes through Nexus' RealTime API, which
/// panics outside the game and would make this untestable.
fn describe(line: &AlertLine, language: gw2lurker_core::i18n::Language) -> String {
    use gw2lurker_core::i18n::{f, t};
    match line.kind {
        AlertKind::Flip => {
            let to = match line.to {
                Some(colour) => colour_label(colour, language).to_owned(),
                None => t(language, "Unclaimed").to_owned(),
            };
            let from = match line.from {
                Some(colour) => colour_label(colour, language).to_owned(),
                None => t(language, "Unclaimed").to_owned(),
            };
            f(language, "{0}: {1} → {2}", &[&line.objective, &from, &to])
        }
        AlertKind::Claim => match &line.guild {
            Some(guild) => f(language, "{0} claimed by {1}", &[&line.objective, guild]),
            None => line.objective.clone(),
        },
        AlertKind::Unclaim => match &line.guild {
            Some(guild) => f(language, "{0} released by {1}", &[&line.objective, guild]),
            None => line.objective.clone(),
        },
    }
}

/// Colour of the text: the new owner when it flipped, and a neutral tone for claims.
fn colour_of(line: &AlertLine) -> [f32; 4] {
    match line.kind {
        AlertKind::Flip => match line.to {
            Some(TeamColor::Red) => [1.0, 0.45, 0.45, 1.0],
            Some(TeamColor::Green) => [0.6, 0.95, 0.6, 1.0],
            Some(TeamColor::Blue) => [0.52, 0.75, 1.0, 1.0],
            None => [0.75, 0.75, 0.75, 1.0],
        },
        AlertKind::Claim => [0.95, 0.85, 0.55, 1.0],
        AlertKind::Unclaim => [0.75, 0.75, 0.75, 1.0],
    }
}

fn colour_label(colour: TeamColor, language: gw2lurker_core::i18n::Language) -> &'static str {
    use gw2lurker_core::i18n::t;
    match colour {
        TeamColor::Red => t(language, "Red"),
        TeamColor::Green => t(language, "Green"),
        TeamColor::Blue => t(language, "Blue"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use gw2lurker_core::i18n::Language;

    fn line(kind: AlertKind, to: Option<TeamColor>, guild: Option<&str>) -> AlertLine {
        let at = Utc.with_ymd_and_hms(2026, 10, 31, 12, 0, 0).unwrap();
        AlertLine {
            at,
            received: at,
            map_role: gw2lurker_core::model::MapRole::Center,
            kind,
            objective: "Zanja de Mendon".to_owned(),
            guild: guild.map(str::to_owned),
            from: Some(TeamColor::Red),
            to,
            late: false,
        }
    }

    #[test]
    fn rf19a_the_popup_only_shows_what_just_arrived() {
        let base = Utc.with_ymd_and_hms(2026, 10, 31, 12, 0, 0).unwrap();
        let mut old = line(AlertKind::Flip, Some(TeamColor::Blue), None);
        old.received = base;
        let mut fresh = line(AlertKind::Claim, None, Some("YaK"));
        fresh.received = base + chrono::Duration::seconds(10);

        let now = base + chrono::Duration::seconds(11);
        let shown = popup_lines(&[fresh.clone(), old.clone()], now, 2.0);
        assert_eq!(shown.len(), 1, "el de hace 11 s ya no sale");
        assert_eq!(shown[0].guild.as_deref(), Some("YaK"));

        assert!(
            popup_lines(&[fresh.clone()], now, 0.0).is_empty(),
            "0 s significa sin pop-up"
        );
        assert_eq!(
            popup_lines(&[fresh.clone()], now, 5.0).len(),
            1,
            "5 s es el máximo y sigue mostrándolo"
        );
    }

    #[test]
    fn rf25_late_claims_are_not_popped_up() {
        let base = Utc.with_ymd_and_hms(2026, 10, 31, 12, 0, 0).unwrap();
        let mut late = line(AlertKind::Claim, None, Some("YaK"));
        late.late = true;
        late.received = base;
        assert!(
            popup_lines(&[late], base, 2.0).is_empty(),
            "lo que ya estaba reclamado al arrancar no interrumpe"
        );
    }

    #[test]
    fn rf19_a_flip_names_both_colours() {
        let text = describe(
            &line(AlertKind::Flip, Some(TeamColor::Blue), None),
            Language::Es,
        );
        assert!(text.contains("Zanja de Mendon"), "{text}");
        assert!(text.contains('→'), "se ve de quién a quién: {text}");
        assert_eq!(text.matches("Azul").count(), 1, "azul es el nuevo dueño");
    }

    #[test]
    fn rf21_claims_name_the_guild() {
        let text = describe(&line(AlertKind::Claim, None, Some("YaK")), Language::Es);
        assert!(text.contains("YaK"), "{text}");
        assert!(text.contains("reclamado"), "traducido al español: {text}");
    }

    #[test]
    fn rf19_unclaim_says_released() {
        let text = describe(&line(AlertKind::Unclaim, None, Some("YaK")), Language::Es);
        assert!(text.contains("liberado"), "{text}");
    }

    #[test]
    fn rf19_colours_follow_the_new_owner() {
        let red = colour_of(&line(AlertKind::Flip, Some(TeamColor::Red), None));
        let blue = colour_of(&line(AlertKind::Flip, Some(TeamColor::Blue), None));
        assert_ne!(red, blue, "cada color se distingue");
        assert!(red[0] > red[2], "el rojo tira a rojo");
        assert!(blue[2] > blue[0], "el azul tira a azul");
    }
}
