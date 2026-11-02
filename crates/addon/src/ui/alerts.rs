//! Alert widget (T-13, RF-19): the recent captures and claims, newest first.
//!
//! Two surfaces from the same data: the **window**, which keeps everything the filter accepts so
//! it can be read later, and the **pop-up**, which shows only what arrived in the last few
//! seconds (15 by default, configurable from 0 to 20) and then disappears without removing anything from the window
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

pub(super) struct LineContext<'a> {
    pub(super) now: DateTime<Utc>,
    pub(super) player_team: Option<TeamColor>,
    pub(super) states: &'a [gw2lurker_core::matchup::ObjectiveState],
}

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

/// How many lines the *Alert log* draws at most, the newest first.
///
/// El registro era lento porque pintaba todo lo de la sesión, y ahora cada línea son varios trozos
/// de texto (informe de 2026-10-31). Lo que no se dibuja sigue en el historial y en la ruta.
pub const MAX_LOGGED: usize = 50;

/// Space reserved ahead of each line for the drawing of the objective (T-47).
const GLYPH_WIDTH: f32 = 11.0;

/// Half size of that drawing: it has to fit in a line of text.
const GLYPH_HALF: f32 = 4.5;

/// Body of the *Alerts* tab: what just happened, so it is readable during a fight.
///
/// Returns the guild of the line the player clicked, if any: clicking an alert asks for the route
/// of that guild (request of 2026-10-31).
pub fn body(ui: &Ui, alerts: &[AlertLine], context: &LineContext<'_>) -> Option<String> {
    ui.text_disabled(i18n::format(
        "Last {0} minutes; the log window has the rest.",
        &[&WIDGET_MINUTES.to_string()],
    ));
    let recent: Vec<AlertLine> = within(alerts, context.now, WIDGET_MINUTES)
        .into_iter()
        .filter(|line| keeps(line.kind))
        .take(crate::config().max_lines.clamp(1, MAX_LINES))
        .collect();
    if recent.is_empty() {
        ui.text_disabled(i18n::text("No alerts yet"));
        return None;
    }
    let rows: Vec<(&AlertLine, f32)> = recent.iter().map(|line| (line, 1.0)).collect();
    draw_faded_lines(ui, &rows, context, false, i18n::effective())
}

/// Body of the *Alert log* tab: the whole session, scrolling if needed.
///
/// Only the rows on screen are laid out ([`ListClipper`]): the log used to draw every line, and
/// with the coloured pieces of each one that is a lot of work per frame for text nobody is reading
/// (report of 2026-10-31). The cap is [`MAX_LOGGED`] lines, the newest first.
pub fn log_body(ui: &Ui, alerts: &[AlertLine], context: &LineContext<'_>) -> Option<String> {
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
                if draw_line(ui, line, context, true) {
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
fn draw_faded_lines(
    ui: &Ui,
    lines: &[(&AlertLine, f32)],
    context: &LineContext<'_>,
    include_event_time: bool,
    language: gw2lurker_core::i18n::Language,
) -> Option<String> {
    let mut marks: Vec<([f32; 2], &AlertLine, f32)> = Vec::with_capacity(lines.len());
    let mut chosen = None;
    for &(line, fade) in lines {
        let start = ui.cursor_screen_pos();
        if super::icons::has_glyph(line.objective_kind) {
            marks.push((start, line, fade));
            let x = ui.cursor_pos()[0];
            ui.dummy([GLYPH_WIDTH, ui.text_line_height()]);
            ui.same_line_with_pos(x + GLYPH_WIDTH);
        }
        if draw_line_faded(ui, line, language, fade, context, include_event_time) {
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
        for (start, line, fade) in marks {
            let mut colour = GREY;
            colour[3] *= fade;
            super::icons::glyph(
                &list,
                line.objective_kind,
                [start[0] + GLYPH_HALF, start[1] + middle],
                GLYPH_HALF,
                colour,
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

/// Dibuja el emergente sin cromo; solo captura el ratón durante la colocación.
///
/// `position` is where the player left it, and comes back with the new one. The frame is gone on
/// purpose: with borders it reads as another window instead of a pop-up (bug of 2026-10-31).
pub fn draw_popup(
    ui: &Ui,
    lines: &[super::popup_stack::PopupLine],
    overflow: usize,
    opacity: f32,
    position: &mut Option<[f32; 2]>,
    placing: bool,
    context: &LineContext<'_>,
) {
    if lines.is_empty() && overflow == 0 {
        return;
    }
    let [width, _] = ui.io().display_size;
    let at = position.unwrap_or([(width - 420.0) / 2.0, 90.0]);

    let mut window = Window::new(i18n::widget(POPUP_KEY))
        .title_bar(false)
        .no_decoration()
        .resizable(false)
        .movable(placing)
        .always_auto_resize(true)
        .position(at, Condition::FirstUseEver)
        .bg_alpha(opacity.clamp(0.0, 1.0));
    if !placing {
        window = window.no_inputs();
    }
    build_popup_window(ui, window, position, || {
        let rows: Vec<(&AlertLine, f32)> = lines.iter().map(|row| (&row.line, row.fade)).collect();
        let _ = draw_faded_lines(ui, &rows, context, false, i18n::effective());
        if overflow > 0 {
            ui.text_disabled(i18n::format("And {0} more", &[&overflow.to_string()]));
        }
    });
}

fn build_popup_window<T: AsRef<str>>(
    ui: &Ui,
    window: Window<'_, T>,
    position: &mut Option<[f32; 2]>,
    content: impl FnOnce(),
) {
    window.build(ui, || {
        content();
        // Window::build ends the window after this closure (arcdps-imgui 0.8, window/mod.rs:547).
        *position = Some(ui.window_pos());
    });
}

/// One line, in coloured pieces. Returns `true` when the player clicked it and it names a guild.
fn draw_line(
    ui: &Ui,
    line: &AlertLine,
    context: &LineContext<'_>,
    include_event_time: bool,
) -> bool {
    draw_line_faded(
        ui,
        line,
        i18n::effective(),
        1.0,
        context,
        include_event_time,
    )
}

fn draw_line_faded(
    ui: &Ui,
    line: &AlertLine,
    language: gw2lurker_core::i18n::Language,
    fade: f32,
    context: &LineContext<'_>,
    include_event_time: bool,
) -> bool {
    let high_value = context
        .states
        .iter()
        .find(|state| state.objective_id == line.objective_id)
        .is_some_and(gw2lurker_core::matchup::ObjectiveState::is_high_value);
    let perspective = match line.kind {
        AlertKind::Flip => Some(gw2lurker_core::alerts::flip_perspective(
            line.from,
            line.to,
            context.player_team,
        )),
        AlertKind::Claim | AlertKind::Unclaim => None,
    };
    let bold_verb = matches!(
        perspective,
        Some(
            gw2lurker_core::alerts::FlipPerspective::Lost
                | gw2lurker_core::alerts::FlipPerspective::Taken
        )
    );

    // The group makes the whole row one item, so it can be coloured piece by piece and still be
    // clickable as a single line.
    ui.group(|| {
        for (index, (text, colour)) in spans(line, language, context.player_team, high_value)
            .iter()
            .enumerate()
        {
            if index > 0 {
                ui.same_line_with_spacing(0.0, 0.0);
            }
            let mut colour = *colour;
            colour[3] *= fade;
            let _tint = ui.push_style_color(StyleColor::Text, colour);
            let position = ui.cursor_pos();
            ui.text(text);
            if index == 0 && bold_verb {
                let end = ui.cursor_pos();
                ui.set_cursor_pos([position[0] + 1.0, position[1]]);
                ui.text(text);
                ui.set_cursor_pos(end);
            }
        }
        ui.same_line_with_spacing(0.0, 8.0);
        let age = age_text(context.now, line.received, language);
        let mut colour = GREY;
        colour[3] *= fade;
        let _tint = ui.push_style_color(StyleColor::Text, colour);
        ui.text(&age);
        if include_event_time {
            ui.same_line_with_spacing(0.0, 5.0);
            let event_time = format!("({})", crate::clock::clock_seconds(line.at));
            ui.text(&event_time);
        }
    });
    let hovered = ui.is_item_hovered();
    let clicked = hovered && ui.is_mouse_clicked(nexus::imgui::MouseButton::Left);
    if hovered {
        let event_time = crate::clock::clock_seconds(line.at);
        let mut details = i18n::format("Event time: {0}", &[&event_time]);
        if line.late {
            details.push('\n');
            details.push_str(i18n::text("Already claimed when the addon started"));
        }
        if line.guild.is_some() {
            details.push('\n');
            details.push_str(i18n::text("Click to see this guild's route"));
        }
        ui.tooltip_text(&details);
    }
    clicked && line.guild.is_some()
}

fn spans(
    line: &AlertLine,
    language: gw2lurker_core::i18n::Language,
    player_team: Option<TeamColor>,
    high_value: bool,
) -> Vec<(String, [f32; 4])> {
    use gw2lurker_core::i18n::t;
    let mut out: Vec<(String, [f32; 4])> = Vec::new();
    let (verb, verb_colour) = match line.kind {
        AlertKind::Flip => {
            match gw2lurker_core::alerts::flip_perspective(line.from, line.to, player_team) {
                gw2lurker_core::alerts::FlipPerspective::Lost => (
                    t(language, "LOST"),
                    player_team.map(team_colour).unwrap_or(GREY),
                ),
                gw2lurker_core::alerts::FlipPerspective::Taken => (
                    t(language, "TAKEN"),
                    player_team.map(team_colour).unwrap_or(GREY),
                ),
                gw2lurker_core::alerts::FlipPerspective::Change => (t(language, "CHANGE"), GREY),
            }
        }
        AlertKind::Claim | AlertKind::Unclaim => (t(language, "CLAIM"), GREY),
    };
    out.push((verb.to_owned(), verb_colour));
    out.push((" ".to_owned(), GREY));
    out.push((line.objective.clone(), GREY));
    if let Some((name, colour)) = map_piece(line.map_role, language) {
        out.push((" · ".to_owned(), GREY));
        out.push((name, colour));
    }

    match line.kind {
        AlertKind::Flip => {
            if let Some(colour) = line.to {
                out.push((" · ".to_owned(), GREY));
                out.push((format!("{} ", t(language, "Taken by")), GREY));
                out.push((
                    colour_label(colour, language).to_owned(),
                    team_colour(colour),
                ));
            }
        }
        AlertKind::Claim | AlertKind::Unclaim => {
            if let Some(guild) = line.guild_name.as_deref().or(line.guild.as_deref()) {
                out.push((" · ".to_owned(), GREY));
                out.push((guild.to_owned(), GREY));
            }
            if line.kind == AlertKind::Unclaim {
                out.push((" · ".to_owned(), GREY));
                out.push((t(language, "Unclaimed").to_owned(), GREY));
            }
        }
    }
    if high_value {
        out.push((" · ".to_owned(), GREY));
        out.push((t(language, "HIGH VALUE").to_owned(), HIGH_VALUE));
    }
    out
}

fn age_text(
    now: DateTime<Utc>,
    received: DateTime<Utc>,
    language: gw2lurker_core::i18n::Language,
) -> String {
    let seconds = now.signed_duration_since(received).num_seconds().max(0);
    if seconds < 60 {
        format!(
            "{seconds} {}",
            gw2lurker_core::i18n::t(language, "Seconds short")
        )
    } else if seconds < 60 * 60 {
        format!(
            "{} {}",
            seconds / 60,
            gw2lurker_core::i18n::t(language, "Minutes short")
        )
    } else {
        format!("{:02}:{:02}", seconds / 3600, (seconds / 60) % 60)
    }
}

/// Neutral grey: everything that is not the map or an owner colour.
const GREY: [f32; 4] = [0.72, 0.72, 0.72, 1.0];
const HIGH_VALUE: [f32; 4] = [1.0, 0.78, 0.2, 1.0];

/// The map label matches the rest of the addon; borderlands keep their team colour.
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
            gw2lurker_core::i18n::t(language, "Blue alpine").to_owned(),
            team_colour(TeamColor::Blue),
        )),
        MapRole::GreenHome => Some((
            gw2lurker_core::i18n::t(language, "Green alpine").to_owned(),
            team_colour(TeamColor::Green),
        )),
        MapRole::RedHome => Some((
            gw2lurker_core::i18n::t(language, "Red desert").to_owned(),
            team_colour(TeamColor::Red),
        )),
        MapRole::Unknown => None,
    }
}

/// Color de equipo, el mismo que en el mapa y en el historial (RF-62).
fn team_colour(colour: TeamColor) -> [f32; 4] {
    super::theme::team_colour(colour)
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
            objective_id: "38-1".to_owned(),
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
    fn rf19d_placement_remembers_the_popup_position_not_the_parent() {
        let _imgui_guard = crate::ui::IMGUI_TEST_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let mut context = nexus::imgui::Context::create();
        context.set_ini_filename(None);
        context.set_log_filename(None);
        context.io_mut().display_size = [1024.0, 768.0];
        context.io_mut().delta_time = 1.0 / 60.0;
        context.fonts().build_rgba32_texture();

        let expected = [240.0, 190.0];
        let mut position = None;
        let ui = context.frame();
        Window::new("popup position test parent")
            .position([20.0, 30.0], Condition::Always)
            .size([600.0, 400.0], Condition::Always)
            .build(&ui, || {
                let popup = Window::new("popup position test")
                    .position(expected, Condition::Always)
                    .size([160.0, 80.0], Condition::Always);
                build_popup_window(&ui, popup, &mut position, || ui.text("alert"));
            });
        ui.render();
        assert_eq!(position, Some(expected));
    }

    #[test]
    fn rnf01_popup_stack_and_native_draw_budget() {
        let _imgui_guard = crate::ui::IMGUI_TEST_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let mut imgui = nexus::imgui::Context::create();
        imgui.set_ini_filename(None);
        imgui.set_log_filename(None);
        imgui.io_mut().display_size = [1920.0, 1080.0];
        imgui.io_mut().delta_time = 1.0 / 60.0;
        imgui.io_mut().mouse_pos = [-100.0, -100.0];
        imgui.fonts().build_rgba32_texture();

        let now = Utc.with_ymd_and_hms(2026, 11, 2, 20, 0, 0).unwrap();
        let states: Vec<_> = (0..200)
            .map(|index| gw2lurker_core::matchup::ObjectiveState {
                map_id: 38,
                map_role: gw2lurker_core::MapRole::Center,
                objective_id: format!("38-{index}"),
                kind: gw2lurker_core::ObjectiveType::Tower,
                owner: Some(TeamColor::Blue),
                guild: None,
                claimed_at: None,
                points_tick: if index % 3 == 0 { 8 } else { 4 },
            })
            .collect();
        let guilds: Vec<_> = (0..gw2lurker_core::MAX_FOLLOWED)
            .map(|index| format!("{index:08X}-AAAA-BBBB-CCCC-DDDDDDDDDDDD"))
            .collect();
        let mut followed = gw2lurker_core::FollowList::default();
        for guild in &guilds {
            assert!(followed.add(guild));
        }
        let alerts: Vec<_> = (0..crate::alerts::MAX_ALERTS)
            .map(|index| {
                let guild = if index % 2 == 0 {
                    guilds[index % guilds.len()].as_str()
                } else {
                    "FFFFFFFF-AAAA-BBBB-CCCC-DDDDDDDDDDDD"
                };
                let kind = if index % 3 == 0 {
                    AlertKind::Claim
                } else {
                    AlertKind::Flip
                };
                let mut alert = line(kind, Some(TeamColor::Blue), Some(guild));
                alert.objective_id = states[states.len() - 1 - index].objective_id.clone();
                alert.received = now - chrono::Duration::milliseconds(index as i64);
                alert.at = alert.received;
                alert
            })
            .collect();
        let context = LineContext {
            now,
            player_team: Some(TeamColor::Blue),
            states: &states,
        };

        for burst in [false, true] {
            let mut stack = super::super::popup_stack::PopupStack::default();
            let mut position = None;
            let mut samples = Vec::new();
            for iteration in 0..110 {
                if burst {
                    stack = super::super::popup_stack::PopupStack::default();
                }
                let ui = imgui.frame();
                let start = std::time::Instant::now();
                let snapshot =
                    stack.tick(&alerts, now, 15.0, &followed, &states, context.player_team);
                let rows: Vec<_> = snapshot
                    .lines
                    .iter()
                    .map(|row| (&row.line, row.fade))
                    .collect();
                let popup = Window::new("popup performance test")
                    .no_decoration()
                    .no_inputs()
                    .always_auto_resize(true)
                    .position([100.0, 100.0], Condition::FirstUseEver);
                build_popup_window(&ui, popup, &mut position, || {
                    let _ = draw_faded_lines(&ui, &rows, &context, false, Language::Es);
                });
                let vertices = ui.render().total_vtx_count;
                let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                assert_eq!(snapshot.lines.len(), 5);
                assert!(snapshot.overflow > 0);
                if iteration >= 10 {
                    assert!(vertices > 0);
                    samples.push(elapsed);
                }
            }
            samples.sort_by(f64::total_cmp);
            let p95 = samples[samples.len() * 95 / 100];
            println!("RNF-01 popup stack and native draw (burst={burst}): p95={p95:.3} ms");
            if !cfg!(debug_assertions) {
                assert!(p95 < 0.5, "popup work alone exceeds RNF-01: {p95:.3} ms");
            }
        }
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

    /// Texto plano de una línea, para poder afirmar sobre él.
    fn text_of(line: &AlertLine, player_team: Option<TeamColor>) -> String {
        spans(line, Language::Es, player_team, false)
            .into_iter()
            .map(|(text, _)| text)
            .collect()
    }

    #[test]
    fn rf19d_every_alert_the_player_follows_can_be_interrupted() {
        // El pop-up de Nexus salía solo con fortalezas y castillo; el jugador pidió (2026-10-31)
        // que salga **todo lo que siga**, así que la regla ya no mira el tipo de objetivo: mira
        // los filtros de tipo de aviso (`AlertKinds`), que es lo que se prueba aquí.
        let kinds = crate::config::AlertKinds::default();
        for kind in [AlertKind::Flip, AlertKind::Claim, AlertKind::Unclaim] {
            let one = line(kind, Some(TeamColor::Blue), Some("YaK"));
            assert!(kinds.shows(one.kind), "{kind:?} tiene que poder salir");
        }
        let muted = crate::config::AlertKinds {
            claims: false,
            ..crate::config::AlertKinds::default()
        };
        let claim = line(AlertKind::Claim, None, Some("YaK"));
        assert!(
            !muted.shows(claim.kind),
            "si el jugador apaga las reclamaciones, no salen"
        );
    }

    #[test]
    fn rf19_a_flip_names_both_colours() {
        let text = text_of(&line(AlertKind::Flip, Some(TeamColor::Blue), None), None);
        assert!(text.contains("Zanja de Mendon"), "{text}");
        assert!(
            text.contains("se lo lleva"),
            "indica el nuevo dueño: {text}"
        );
        assert_eq!(text.matches("Azul").count(), 1, "azul es el nuevo dueño");
    }

    #[test]
    fn rf21_claims_name_the_guild() {
        let text = text_of(&line(AlertKind::Claim, None, Some("YaK")), None);
        assert!(text.contains("YaK"), "{text}");
        assert!(
            text.contains("Zanja"),
            "el sitio va antes que el clan: {text}"
        );
    }

    #[test]
    fn rf19_unclaim_says_released() {
        let text = text_of(&line(AlertKind::Unclaim, None, Some("YaK")), None);
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
        let pieces = spans(&blue, Language::Es, None, false);
        let map = pieces
            .iter()
            .find(|(text, _)| text == "Alpino azul")
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
            pieces.iter().any(|(text, _)| text == "reclamo"),
            "el verbo se muestra en texto: {pieces:?}"
        );
    }

    #[test]
    fn rf19f_line_uses_my_team_and_marks_high_value() {
        let mut alert = line(AlertKind::Flip, Some(TeamColor::Red), None);
        alert.from = Some(TeamColor::Green);
        alert.map_role = gw2lurker_core::MapRole::GreenHome;
        let pieces = spans(&alert, Language::Es, Some(TeamColor::Green), true);
        let text: String = pieces.iter().map(|(text, _)| text.as_str()).collect();
        assert!(text.starts_with("PERDIDO "), "{text}");
        assert!(text.contains("Alpino verde"), "{text}");
        assert!(text.contains("se lo lleva Rojo"), "{text}");
        assert!(text.contains("ALTO"), "{text}");
        assert_eq!(pieces[0].1, team_colour(TeamColor::Green));
    }

    #[test]
    fn rf19f_relative_age_uses_seconds_minutes_and_hours() {
        let base = Utc.with_ymd_and_hms(2026, 10, 31, 12, 0, 0).unwrap();
        assert_eq!(
            age_text(base + chrono::Duration::seconds(12), base, Language::Es),
            "12 s"
        );
        assert_eq!(
            age_text(base + chrono::Duration::minutes(4), base, Language::Es),
            "4 min"
        );
        assert_eq!(
            age_text(base + chrono::Duration::minutes(60), base, Language::Es),
            "01:00"
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
