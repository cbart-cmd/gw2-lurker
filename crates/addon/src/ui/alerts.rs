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
use nexus::imgui::{Condition, ListClipper, StyleColor, Ui, Window};

use crate::i18n;
use crate::worker::{AlertKind, AlertLine};

/// Translation key (and stable id) of this window.
pub const KEY: &str = "Alerts";

/// Translation key (and stable id) of the expanded log window.
pub const LOG_KEY: &str = "Alert log";

/// Translation key of the pop-up: separate window, separate id, state never saved.
const POPUP_KEY: &str = "Alerts (pop-up)";

/// How many lines fit in the window without scrolling forever; the player can ask for fewer
/// (RF-19c).
const MAX_LINES: usize = 25;

/// Whether the player wants this kind of alert at all (RF-19c).
fn keeps(kind: AlertKind) -> bool {
    crate::config().alert_kinds.shows(kind)
}

/// How long an alert stays in the compact widget, in minutes.
///
/// The widget answers "what just happened"; the log window keeps the whole session. Without
/// this the list only grew (bug of 2026-10-31). Public so the render callback and the tests use
/// the same number.
pub const WIDGET_MINUTES: i64 = 10;

/// How many lines the *Alert log* draws at most: the newest ones. El registro era lento porque\n/// pintaba todo lo de la sesión (informe de 2026-10-31).\npub const MAX_LOGGED: usize = 50;\n\n/// How many lines the *Alert log* draws at most, the newest first.
///
/// El registro era lento porque pintaba todo lo de la sesión, y ahora cada línea son varios trozos
/// de texto (informe de 2026-10-31). Lo que no se dibuja sigue en el historial y en la ruta.
pub const MAX_LOGGED: usize = 50;

/// How many lines the pop-up shows at once.
const MAX_POPUP_LINES: usize = 5;

/// Space reserved ahead of each line for the drawing of the objective (T-47).
const GLYPH_WIDTH: f32 = 11.0;

/// Half size of that drawing: it has to fit in a line of text.
const GLYPH_HALF: f32 = 4.5;

/// Body of the *Alerts* tab: what just happened, so it is readable during a fight.
///
/// Returns the guild of the line the player clicked, if any: clicking an alert asks for the route
/// of that guild (request of 2026-10-31).
pub fn body(ui: &Ui, alerts: &[AlertLine], now: DateTime<Utc>) -> Option<String> {
    ui.text_disabled(i18n::format(
        "Last {0} minutes; the log window has the rest.",
        &[&WIDGET_MINUTES.to_string()],
    ));
    let recent: Vec<AlertLine> = within(alerts, now, WIDGET_MINUTES)
        .into_iter()
        .filter(|line| keeps(line.kind))
        .take(crate::config().max_lines.clamp(1, MAX_LINES))
        .collect();
    if recent.is_empty() {
        ui.text_disabled(i18n::text("No alerts yet"));
        return None;
    }
    let borrowed: Vec<&AlertLine> = recent.iter().collect();
    draw_lines(ui, &borrowed)
}

/// Body of the *Alert log* tab: the whole session, scrolling if needed.
///
/// Only the rows on screen are laid out ([`ListClipper`]): the log used to draw every line, and
/// with the coloured pieces of each one that is a lot of work per frame for text nobody is reading
/// (report of 2026-10-31). The cap is [`MAX_LOGGED`] lines, the newest first.
pub fn log_body(ui: &Ui, alerts: &[AlertLine]) -> Option<String> {
    if alerts.is_empty() {
        ui.text_disabled(i18n::text("No alerts yet"));
        return None;
    }
    let all: Vec<&AlertLine> = alerts
        .iter()
        .filter(|line| keeps(line.kind))
        .take(MAX_LOGGED)
        .collect();
    let mut chosen = None;
    let mut clipper = ListClipper::new(all.len() as i32).begin(ui);
    while clipper.step() {
        for index in clipper.display_start()..clipper.display_end() {
            if let Some(line) = all.get(index as usize) {
                if draw_line(ui, line, i18n::effective()) {
                    chosen = line.guild.clone();
                }
            }
        }
    }
    clipper.end();
    chosen
}

/// One line per alert, each preceded by the drawing of what was taken (T-47).
///
/// The drawing needs the draw list, and `get_window_draw_list` panics if two instances are alive
/// at once (`AGENTS.md` §11.12), so the list is taken **once** after the text pass, using the
/// positions remembered while the lines were laid out.
fn draw_lines(ui: &Ui, lines: &[&AlertLine]) -> Option<String> {
    let language = i18n::effective();
    let mut marks: Vec<([f32; 2], AlertLine)> = Vec::with_capacity(lines.len());
    let mut chosen = None;
    for line in lines {
        let start = ui.cursor_screen_pos();
        if super::icons::has_glyph(line.objective_kind) {
            marks.push((start, (*line).clone()));
            let x = ui.cursor_pos()[0];
            ui.dummy([GLYPH_WIDTH, ui.text_line_height()]);
            ui.same_line_with_pos(x + GLYPH_WIDTH);
        }
        if draw_line(ui, line, language) {
            chosen = line.guild.clone();
        }
    }
    if marks.is_empty() {
        return chosen;
    }
    let list = ui.get_window_draw_list();
    let middle = ui.text_line_height() / 2.0;
    // Recorte a la ventana: el cuerpo del widget vive en una ventana hija y, en imgui, la lista de
    // dibujo es la del padre, así que sin recorte propio (2026-10-31, con captura) los iconos se
    // salían de la ventana.
    let pos = ui.window_pos();
    let area = ui.window_size();
    list.with_clip_rect_intersect(pos, [pos[0] + area[0], pos[1] + area[1]], || {
        for (start, line) in marks {
            super::icons::glyph(
                &list,
                line.objective_kind,
                [start[0] + GLYPH_HALF, start[1] + middle],
                GLYPH_HALF,
                GREY,
            );
        }
    });
    chosen
}

/// The lines still worth a place in the widget: those that arrived in the last `minutes`.
///
/// Pure (`now` is a parameter) so the rule is testable without a game.
pub fn within(lines: &[AlertLine], now: DateTime<Utc>, minutes: i64) -> Vec<AlertLine> {
    let limit = chrono::Duration::minutes(minutes.max(0));
    lines
        .iter()
        .filter(|line| now.signed_duration_since(line.received) < limit)
        .cloned()
        .collect()
}

/// Draws the expanded log: every alert the worker still remembers, scrolling if needed.
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

/// Draws the transient pop-up: no frame, no title, and it can be dragged where you want it.
///
/// `position` is where the player left it, and comes back with the new one. The frame is gone on
/// purpose: with borders it reads as another window instead of a pop-up (bug of 2026-10-31).
pub fn draw_popup(
    ui: &Ui,
    lines: &[AlertLine],
    opacity: f32,
    position: &mut Option<[f32; 2]>,
    fade: f32,
) {
    if lines.is_empty() {
        return;
    }
    let [width, _] = ui.io().display_size;
    let at = position.unwrap_or([(width - 420.0) / 2.0, 90.0]);

    Window::new(i18n::widget(POPUP_KEY))
        .title_bar(false)
        .no_decoration()
        .resizable(false)
        .movable(true)
        .always_auto_resize(true)
        .position(at, Condition::FirstUseEver)
        .bg_alpha((opacity * fade).clamp(0.0, 1.0))
        .build(ui, || {
            let lines: Vec<&AlertLine> = lines.iter().collect();
            let _ = draw_lines(ui, &lines);
        });

    *position = Some(ui.window_pos());
}

/// One line, in coloured pieces. Returns `true` when the player clicked it and it names a guild.
fn draw_line(ui: &Ui, line: &AlertLine, language: gw2lurker_core::i18n::Language) -> bool {
    // The group makes the whole row one item, so it can be coloured piece by piece and still be
    // clickable as a single line.
    ui.group(|| {
        for (index, (text, colour)) in spans(line, language).iter().enumerate() {
            if index > 0 {
                ui.same_line_with_spacing(0.0, 0.0);
            }
            let _tint = ui.push_style_color(StyleColor::Text, *colour);
            ui.text(text);
        }
    });
    let hovered = ui.is_item_hovered();
    let clicked = hovered && ui.is_mouse_clicked(nexus::imgui::MouseButton::Left);
    if hovered && line.late {
        ui.tooltip_text(i18n::text("Already claimed when the addon started"));
    } else if hovered && line.guild.is_some() {
        ui.tooltip_text(i18n::text("Click to see this guild's route"));
    }
    clicked && line.guild.is_some()
}

/// The line as coloured pieces: what the player asked for on 2026-10-31.
///
/// «Marca del tipo de objetivo; el mapa, con su color (Eternos en gris, Azul en azul, Verde en
/// verde, Rojo en rojo); y ya en gris: el nombre del sitio, un guion y el clan que lo tiene
/// reclamado ahora.» Colouring the whole line by the new owner — what it did before — made the
/// guild and the objective unreadable and hid which map it was.
///
/// The language is a parameter: reading the active one goes through Nexus' RealTime API, which
/// panics outside the game and would make this untestable.
fn spans(line: &AlertLine, language: gw2lurker_core::i18n::Language) -> Vec<(String, [f32; 4])> {
    use gw2lurker_core::i18n::t;
    let mut out: Vec<(String, [f32; 4])> = Vec::new();
    out.push((crate::clock::clock_seconds(line.at), GREY));
    if let Some((name, colour)) = map_piece(line.map_role, language) {
        out.push(("  ".to_owned(), GREY));
        out.push((name, colour));
    }
    out.push(("  ·  ".to_owned(), GREY));
    out.push((line.objective.clone(), GREY));

    match line.kind {
        AlertKind::Flip => {
            out.push(("  -  ".to_owned(), GREY));
            match line.from {
                Some(colour) => out.push((
                    colour_label(colour, language).to_owned(),
                    team_colour(colour),
                )),
                None => out.push((t(language, "Unclaimed").to_owned(), GREY)),
            }
            out.push((" → ".to_owned(), GREY));
            match line.to {
                Some(colour) => out.push((
                    colour_label(colour, language).to_owned(),
                    team_colour(colour),
                )),
                None => out.push((t(language, "Unclaimed").to_owned(), GREY)),
            }
        }
        AlertKind::Claim | AlertKind::Unclaim => {
            if let Some(guild) = line.guild_name.as_deref().or(line.guild.as_deref()) {
                out.push(("  -  ".to_owned(), GREY));
                out.push((guild.to_owned(), GREY));
            }
            if line.kind == AlertKind::Unclaim {
                out.push((" · ".to_owned(), GREY));
                out.push((t(language, "Unclaimed").to_owned(), GREY));
            }
        }
    }
    out
}

/// Neutral grey: everything that is not the map or an owner colour.
const GREY: [f32; 4] = [0.72, 0.72, 0.72, 1.0];

/// The map of an alert, **short and in its own colour**: «Eternos» in grey, «Azul», «Verde» and
/// «Rojo» in the colour of their borderland, which is how the player says it (request of
/// 2026-10-31). `None` when the id was not one of the four.
fn map_piece(
    role: gw2lurker_core::MapRole,
    language: gw2lurker_core::i18n::Language,
) -> Option<(String, [f32; 4])> {
    use gw2lurker_core::MapRole;
    match role {
        MapRole::Center => Some((
            gw2lurker_core::i18n::t(language, "Eternals").to_owned(),
            [0.85, 0.85, 0.85, 1.0],
        )),
        MapRole::BlueHome => Some((
            colour_label(TeamColor::Blue, language).to_owned(),
            team_colour(TeamColor::Blue),
        )),
        MapRole::GreenHome => Some((
            colour_label(TeamColor::Green, language).to_owned(),
            team_colour(TeamColor::Green),
        )),
        MapRole::RedHome => Some((
            colour_label(TeamColor::Red, language).to_owned(),
            team_colour(TeamColor::Red),
        )),
        MapRole::Unknown => None,
    }
}

/// Color de equipo, el mismo que en el mapa y en el historial (RF-62).
fn team_colour(colour: TeamColor) -> [f32; 4] {
    super::theme::team_colour(colour)
}

/// Whether an alert is worth interrupting for (RF-19d): taking a keep or the castle.
///
/// The pop-up already shows everything the player asked for; this is the extra layer for the two
/// objectives that change a matchup, and it is **off by default**.
pub fn is_critical(line: &AlertLine) -> bool {
    use gw2lurker_core::ObjectiveType;
    matches!(line.kind, AlertKind::Flip | AlertKind::Claim)
        && matches!(
            line.objective_kind,
            ObjectiveType::Keep | ObjectiveType::Castle
        )
}

/// La línea en texto plano, para el toast de Nexus.
pub fn plain(line: &AlertLine) -> String {
    spans(line, i18n::effective())
        .into_iter()
        .map(|(text, _)| text)
        .collect::<Vec<_>>()
        .join("")
        .trim()
        .to_owned()
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
            objective_kind: gw2lurker_core::ObjectiveType::Tower,
            kind,
            objective: "Zanja de Mendon".to_owned(),
            guild: guild.map(str::to_owned),
            guild_name: guild.map(|tag| format!("[{tag}] Algo")),
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
    fn rf19_the_widget_forgets_what_is_old() {
        let base = Utc.with_ymd_and_hms(2026, 10, 31, 12, 0, 0).unwrap();
        let mut old = line(AlertKind::Flip, Some(TeamColor::Blue), None);
        old.received = base;
        let mut fresh = line(AlertKind::Claim, None, Some("YaK"));
        fresh.received = base + chrono::Duration::minutes(9);
        fresh.at = fresh.received;

        let now = base + chrono::Duration::minutes(12);
        let shown = within(&[fresh.clone(), old], now, WIDGET_MINUTES);
        assert_eq!(shown.len(), 1, "lo de hace 12 min ya no sale en el widget");
        assert_eq!(shown[0].guild_name.as_deref(), Some("[YaK] Algo"));
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

    /// Texto plano de una línea, para poder afirmar sobre él.
    fn text_of(line: &AlertLine) -> String {
        spans(line, Language::Es)
            .into_iter()
            .map(|(text, _)| text)
            .collect()
    }

    #[test]
    fn rf19d_only_keeps_and_castles_interrupt_with_a_toast() {
        let mut keep = line(AlertKind::Flip, Some(TeamColor::Blue), None);
        keep.objective_kind = gw2lurker_core::ObjectiveType::Keep;
        assert!(is_critical(&keep));

        let mut castle = line(AlertKind::Claim, None, Some("YaK"));
        castle.objective_kind = gw2lurker_core::ObjectiveType::Castle;
        assert!(is_critical(&castle));

        let mut camp = line(AlertKind::Claim, None, Some("YaK"));
        camp.objective_kind = gw2lurker_core::ObjectiveType::Camp;
        assert!(!is_critical(&camp), "un campamento no interrumpe");

        let mut released = line(AlertKind::Unclaim, None, Some("YaK"));
        released.objective_kind = gw2lurker_core::ObjectiveType::Keep;
        assert!(
            !is_critical(&released),
            "soltar una fortaleza no es crítico"
        );
    }

    #[test]
    fn rf19_a_flip_names_both_colours() {
        let text = text_of(&line(AlertKind::Flip, Some(TeamColor::Blue), None));
        assert!(text.contains("Zanja de Mendon"), "{text}");
        assert!(text.contains('→'), "se ve de quién a quién: {text}");
        assert_eq!(text.matches("Azul").count(), 1, "azul es el nuevo dueño");
    }

    #[test]
    fn rf21_claims_name_the_guild() {
        let text = text_of(&line(AlertKind::Claim, None, Some("YaK")));
        assert!(text.contains("YaK"), "{text}");
        assert!(
            text.contains("Zanja"),
            "el sitio va antes que el clan: {text}"
        );
    }

    #[test]
    fn rf19_unclaim_says_released() {
        let text = text_of(&line(AlertKind::Unclaim, None, Some("YaK")));
        assert!(
            text.contains("Sin reclamar"),
            "se dice que ya no lo tiene: {text}"
        );
    }

    #[test]
    fn rf19_the_map_is_drawn_in_its_own_colour_and_the_rest_in_grey() {
        // El formato que pidió el jugador el 2026-10-31: el mapa con su color y el sitio y el clan
        // en gris, en vez de toda la línea del color del que ha capturado.
        let mut blue = line(AlertKind::Claim, None, Some("YaK"));
        blue.map_role = gw2lurker_core::MapRole::BlueHome;
        let pieces = spans(&blue, Language::Es);
        let map = pieces
            .iter()
            .find(|(text, _)| text == "Azul")
            .expect("el mapa se nombra");
        assert_eq!(map.1, team_colour(TeamColor::Blue), "azul en azul");
        let objective = pieces
            .iter()
            .find(|(text, _)| text == "Zanja de Mendon")
            .expect("el sitio se nombra");
        assert_eq!(objective.1, GREY, "el sitio en gris");
        let guild = pieces
            .iter()
            .find(|(text, _)| text.contains("YaK"))
            .expect("el clan se nombra");
        assert_eq!(guild.1, GREY, "el clan en gris");
        assert!(
            pieces.iter().any(|(text, _)| text.contains('-')),
            "sitio y clan van separados por un guion: {pieces:?}"
        );
    }

    #[test]
    fn rf19_the_owner_colours_are_told_apart() {
        let red = team_colour(TeamColor::Red);
        let blue = team_colour(TeamColor::Blue);
        assert_ne!(red, blue, "cada color se distingue");
        assert!(red[0] > red[2], "el rojo tira a rojo");
        assert!(blue[2] > blue[0], "el azul tira a azul");
    }
}
