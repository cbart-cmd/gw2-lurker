//! Settings window (T-17, T-30): the team picker, language, polling interval and alert scope.
//!
//! The picker asks in three steps — **region → colour → team** — because that is how a player
//! recognises their side: the in-game WvW panel shows the three colours of the matchup. With
//! thirty-three teams in the API, going straight to a name list is guesswork; with region and
//! colour it is four or five rows (RF-03a).
//!
//! All edits are collected in a local copy and written back after the window is drawn, because
//! `window::draw` already borrows the window state. The caller applies the returned
//! [`TeamChange`] once it has released the configuration lock: `team::*` takes it again.

use std::sync::Mutex;

use gw2lurker_core::model::{Region, TeamColor};
use gw2lurker_core::{select, AlertFilter, Candidate, FollowList, TeamTable};
use nexus::imgui::{ComboBox, Selectable, Slider, StyleColor, Ui};

use crate::config::{AlertScope, LanguagePreference};
use crate::i18n;

/// Translation key (and stable id) of this window.
pub const KEY: &str = "Settings";

/// A change to the team selection, decided here and applied by the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeamChange {
    /// The player chose this team.
    Pick(u32),
    /// The player cleared the selection.
    Clear,
    /// The player pressed *Save* on the translation: apply it and write the file now.
    SaveNow,
}

/// Which region and colour the player has already narrowed down to.
///
/// Session state on purpose: it is a way through the list, not a preference, so it starts
/// empty on every launch and then follows wherever the player clicks.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Narrowing {
    region: Option<Region>,
    colour: Option<TeamColor>,
}

static NARROWING: Mutex<Narrowing> = Mutex::new(Narrowing {
    region: None,
    colour: None,
});

fn narrowing() -> Narrowing {
    *NARROWING.lock().unwrap_or_else(|error| error.into_inner())
}

fn set_narrowing(value: Narrowing) {
    let mut slot = NARROWING.lock().unwrap_or_else(|error| error.into_inner());
    *slot = value;
}

/// Working copy of everything this window can edit.
struct Edits {
    team_id: Option<u32>,
    translation: String,
    language: LanguagePreference,
    poll_secs: i32,
    scope: AlertScope,
    narrowing: Narrowing,
    alerts: AlertFilter,
    popup_secs: f32,
    popup_enabled: bool,
    follow: FollowList,
    widget_visible: bool,
    widget_locked: bool,
    map_tiles: bool,
    theme: crate::config::Theme,
    background: crate::config::WidgetBackground,
    font_scale: f32,
    max_lines: i32,
    kinds: crate::config::AlertKinds,
    guild_index_url: String,
}

/// Body of the *Settings* tab: the whole form, without window chrome (T-42).
///
/// `candidates` are the teams playing right now, from the worker. An empty list means the first
/// poll has not finished, and the picker says so instead of pretending.
pub fn body(
    ui: &Ui,
    config: &mut crate::config::Config,
    candidates: &[Candidate],
    worlds: &std::collections::BTreeMap<u32, String>,
) -> Option<TeamChange> {
    // Mientras el selector está en pantalla hacen falta los nueve emparejamientos: sin esto la
    // lista de equipos se queda como esté (bug de 2026-10-31, el desplegable salía vacío).
    crate::worker::set_teams_needed(true);
    let table = gw2lurker_core::bundled();
    let language = i18n::effective().code();
    let selected = crate::team::selected_in(config);
    let widget = config.window(crate::ui::WIDGET).clone();
    let mut edits = Edits {
        team_id: selected,
        translation: initial_translation(config, table, selected, language),
        language: config.language,
        poll_secs: config.poll_secs as i32,
        scope: config.alert_scope,
        narrowing: initial_narrowing(narrowing(), candidates, selected),
        alerts: config.alerts,
        popup_secs: config.popup_secs,
        popup_enabled: config.popup_enabled,
        follow: config.followed.clone(),
        widget_visible: widget.visible,
        widget_locked: widget.locked,
        map_tiles: config.map_tiles,
        theme: config.theme,
        background: config.widget_background,
        font_scale: config.font_scale,
        max_lines: config.max_lines as i32,
        kinds: config.alert_kinds,
        guild_index_url: config.guild_index_url.clone(),
    };
    let before = edits.team_id;

    let save_now = draw_body(ui, table, &mut edits, candidates, worlds);

    remember_translation(config, edits.translation.as_str(), language);
    config.language = edits.language;
    config.alert_scope = edits.scope;
    config.poll_secs = edits
        .poll_secs
        .clamp(crate::config::POLL_SECS_MIN as i32, 600) as u32;
    config.alerts = edits.alerts;
    config.popup_secs = edits.popup_secs.clamp(0.0, POPUP_MAX_SECS);
    config.popup_enabled = edits.popup_enabled;
    config.map_tiles = edits.map_tiles;
    config.theme = edits.theme;
    config.widget_background = edits.background;
    config.font_scale = edits.font_scale.clamp(0.8, 1.6);
    config.max_lines = edits.max_lines.clamp(1, 50) as usize;
    config.alert_kinds = edits.kinds;
    config.guild_index_url = edits.guild_index_url.trim().to_owned();
    config.followed = edits.follow;
    // The switches of the widget itself: written only when moved, so the chrome of this window
    // is not undone every frame (`×`, dragging, lock).
    if edits.widget_visible != widget.visible {
        config.window(crate::ui::WIDGET).visible = edits.widget_visible;
    }
    if edits.widget_locked != widget.locked {
        config.window(crate::ui::WIDGET).locked = edits.widget_locked;
    }

    set_narrowing(edits.narrowing);

    if save_now {
        return Some(TeamChange::SaveNow);
    }
    team_change(before, edits.team_id)
}

/// Draws the settings inline, without window chrome.
///
/// Used by Nexus' own options pane (`RenderType::OptionsRender`), which is reachable even
/// outside WvW: that is how RF-02 keeps the configuration available everywhere.
pub fn draw_inline(
    ui: &Ui,
    candidates: &[Candidate],
    worlds: &std::collections::BTreeMap<u32, String>,
) -> Option<TeamChange> {
    let mut config = crate::runtime();
    body(ui, &mut config, candidates, worlds)
}

fn team_change(before: Option<u32>, now: Option<u32>) -> Option<TeamChange> {
    match (before, now) {
        (Some(_), None) => Some(TeamChange::Clear),
        (old, Some(new)) if old != Some(new) => Some(TeamChange::Pick(new)),
        _ => None,
    }
}

/// Where the narrowing starts: the saved team's own region and colour when we know them, and
/// otherwise the region of the first candidate, so the picker is never a blank wall.
fn initial_narrowing(
    remembered: Narrowing,
    candidates: &[Candidate],
    selected: Option<u32>,
) -> Narrowing {
    if remembered.region.is_some() || remembered.colour.is_some() {
        return remembered;
    }

    if let Some(team) = selected {
        if let Some(found) = candidates
            .iter()
            .find(|candidate| candidate.team_id == team)
        {
            return Narrowing {
                region: found.region(),
                colour: Some(found.color),
            };
        }
    }

    Narrowing {
        region: candidates.first().and_then(Candidate::region),
        colour: None,
    }
}

/// Everything the label of a control can say, so the control never eats the label.
///
/// imgui puts the label to the **right** of the widget and clips it at the window edge: with a
/// 470 px window and a label column taken from the wrong string, «Intervalo de sondeo» was cut
/// to «INTERVALO DE SON…» (report of 2026-10-31). The column is measured over the labels that
/// are really drawn, in the current language.
/// Lo más largo que puede durar cada aviso del pop-up (segundos). Se pidió poder llegar a 20
/// porque, de uno en uno, cinco se quedaban cortos (informe de 2026-10-31).
const POPUP_MAX_SECS: f32 = crate::config::POPUP_SECS_MAX;

/// Lo que dura cada aviso del pop-up cuando se activa (segundos).
const POPUP_DEFAULT_SECS: f32 = 5.0;

const LABELS: &[&str] = &[
    "Region",
    "Colour",
    "Team",
    "Translation",
    "Language",
    "Poll interval",
    "Alert scope",
    "Pop-up seconds",
    "Show widget",
    "Lock position",
    "Click-through",
    "Pop-up alerts",
    "Test pop-up",
    "Real map",
    "Save",
    "Search",
    "Team (any)",
];

/// Width of the widest label of the form, in the current language, plus a margin.
fn label_column(ui: &Ui) -> f32 {
    LABELS
        .iter()
        .map(|key| ui.calc_text_size(i18n::text(key))[0])
        .fold(0.0_f32, f32::max)
        + 20.0
}

fn draw_body(
    ui: &Ui,
    table: &TeamTable,
    edits: &mut Edits,
    candidates: &[Candidate],
    worlds: &std::collections::BTreeMap<u32, String>,
) -> bool {
    // imgui puts the label to the right of the widget: cap the widget so the translated labels
    // of a narrow window are not cut off, whatever the language.
    let label_width = label_column(ui);
    let available = ui.content_region_avail()[0];
    let _item_width = ui.push_item_width((available - label_width).max(90.0));

    if candidates.is_empty() {
        ui.text_wrapped(i18n::text("Pick region and colour, then your team"));
        ui.text_disabled(i18n::text("No data yet"));
        ui.text_disabled(i18n::text(
            "The list of teams arrives with the first poll; wait a few seconds.",
        ));
        ui.separator();
    }

    draw_region(ui, edits, candidates);
    draw_colour(ui, edits);
    draw_team(ui, table, edits, candidates, worlds);

    let translation_id = i18n::widget("Translation");
    ui.input_text(translation_id, &mut edits.translation)
        .build();
    if ui.is_item_hovered() {
        ui.tooltip_text(i18n::text(
            "Prefilled with the name in the game language. Clear it to use the automatic one.",
        ));
    }

    draw_language(ui, edits);
    ui.input_int(i18n::widget("Poll interval"), &mut edits.poll_secs)
        .build();
    draw_scope(ui, edits);
    draw_alert_filters(ui, edits);
    ui.text_disabled(i18n::text(
        "Following guilds lives in the Guilds tab, next to the match you are looking at.",
    ));
    draw_guild_index(ui, edits);

    ui.separator();
    ui.text(i18n::text("Windows"));
    ui.checkbox(i18n::widget("Show widget"), &mut edits.widget_visible);
    ui.same_line();
    ui.checkbox(i18n::widget("Lock position"), &mut edits.widget_locked);
    // El click-through se pone con el **clic derecho** sobre el widget (petición del jugador,
    // 2026-11-01): tenerlo también aquí era la forma de quedarse sin ratón sin saber por qué.
    ui.text_disabled(i18n::text(
        "Click-through is turned on from the right-click menu, on the widget itself.",
    ));
    let popups = ui.checkbox(i18n::widget("Pop-up alerts"), &mut edits.popup_enabled);
    ui.same_line();
    if ui.button(i18n::widget("Test pop-up")) {
        crate::ui::start_test_popup();
    }
    if popups && edits.popup_enabled && edits.popup_secs <= 0.0 {
        edits.popup_secs = POPUP_DEFAULT_SECS;
    }
    ui.checkbox(i18n::widget("Real map"), &mut edits.map_tiles);
    draw_theme(ui, edits);
    ui.text_disabled(i18n::format(
        "Click-through keeps the mouse for the game. Right-click on the widget to use the mouse again; the Nexus shortcut for {0} does the same.",
        &[crate::keybinds::shortcut()],
    ));

    ui.separator();
    let saved = ui.button(i18n::widget("Save"));
    if saved {
        // Feedback is deliberately minimal: the button exists so pressing something persists,
        // and the tooltip says the changes are already applied while editing.
    }
    if ui.is_item_hovered() {
        ui.tooltip_text(i18n::text(
            "The changes are applied as you edit; this writes them now.",
        ));
    }
    saved
}

/// Estilo y tamaño del texto de las ventanas, y qué se enseña en el widget (RF-19c, RF-60 a RF-63).
fn draw_theme(ui: &Ui, edits: &mut Edits) {
    ui.separator();
    let style = match edits.theme {
        crate::config::Theme::Inherited => i18n::text("Inherited from Nexus").to_owned(),
        crate::config::Theme::Own => i18n::text("Addon's own dark theme").to_owned(),
    };
    ComboBox::new(i18n::widget("Window style"))
        .preview_value(style)
        .build(ui, || {
            for (option, label) in [
                (
                    crate::config::Theme::Inherited,
                    i18n::text("Inherited from Nexus"),
                ),
                (
                    crate::config::Theme::Own,
                    i18n::text("Addon's own dark theme"),
                ),
            ] {
                if Selectable::new(label)
                    .selected(edits.theme == option)
                    .build(ui)
                {
                    edits.theme = option;
                }
            }
        });

    let background = match edits.background {
        crate::config::WidgetBackground::Dark => i18n::text("Dark, semi-transparent").to_owned(),
        crate::config::WidgetBackground::Transparent => i18n::text("Transparent").to_owned(),
    };
    ComboBox::new(i18n::widget("Background"))
        .preview_value(background)
        .build(ui, || {
            for (option, label) in [
                (
                    crate::config::WidgetBackground::Dark,
                    i18n::text("Dark, semi-transparent"),
                ),
                (
                    crate::config::WidgetBackground::Transparent,
                    i18n::text("Transparent"),
                ),
            ] {
                if Selectable::new(label)
                    .selected(edits.background == option)
                    .build(ui)
                {
                    edits.background = option;
                }
            }
        });

    Slider::new(i18n::widget("Text size"), 0.8, 1.6)
        .display_format("%.1f×")
        .build(ui, &mut edits.font_scale);
    ui.input_int(i18n::widget("Lines in the widget"), &mut edits.max_lines)
        .build();
    ui.checkbox(i18n::widget("Show captures"), &mut edits.kinds.flips);
    ui.checkbox(i18n::widget("Show claims"), &mut edits.kinds.claims);
    ui.checkbox(i18n::widget("Show releases"), &mut edits.kinds.unclaims);
}

/// Which maps and guilds deserve an alert (RF-19a), plus how long the pop-up lasts.
fn draw_alert_filters(ui: &Ui, edits: &mut Edits) {
    ui.separator();
    ui.text(i18n::text("Alert on"));
    // Las estructuras primero: es lo que el jugador pidió (2026-11-01). Las ruinas y el resto
    // del decorado no aparecen porque nunca avisan.
    ui.checkbox(i18n::widget("Camps"), &mut edits.alerts.structures.camp);
    ui.checkbox(i18n::widget("Towers"), &mut edits.alerts.structures.tower);
    ui.checkbox(
        i18n::widget("Keeps and garrisons"),
        &mut edits.alerts.structures.keep,
    );
    ui.checkbox(i18n::widget("Castles"), &mut edits.alerts.structures.castle);
    if ui.is_item_hovered() {
        ui.tooltip_text(i18n::text(
            "Only structures a guild can claim raise an alert: ruins and mercenary camps never do.",
        ));
    }
    ui.checkbox(i18n::widget("Eternals"), &mut edits.alerts.ebg);
    ui.checkbox(i18n::widget("Blue alpine"), &mut edits.alerts.blue);
    ui.checkbox(i18n::widget("Green alpine"), &mut edits.alerts.green);
    ui.checkbox(i18n::widget("Red desert"), &mut edits.alerts.red);
    ui.checkbox(
        i18n::widget("The map I am on"),
        &mut edits.alerts.current_map,
    );
    ui.checkbox(i18n::widget("Guilds I follow"), &mut edits.alerts.followed);
    ui.checkbox(
        i18n::widget("Only the guilds I follow"),
        &mut edits.alerts.only_followed,
    );
    if ui.is_item_hovered() {
        ui.tooltip_text(i18n::text(
            "One switch for «only mine»: the map boxes above stop counting.",
        ));
    }

    ui.separator();
    let mut seconds = edits.popup_secs;
    if Slider::new(i18n::widget("Pop-up seconds"), 0.0, POPUP_MAX_SECS)
        .display_format("%.1f s")
        .build(ui, &mut seconds)
    {
        edits.popup_secs = seconds;
    }
    if ui.is_item_hovered() {
        ui.tooltip_text(i18n::text(
            "How long each pop-up stays. They come one at a time; the list keeps them all.",
        ));
    }
}

/// Índice de nombres de clan: estado y descarga (RF-20).
///
/// Se descarga solo la primera vez (o cuando el jugador lo pide), se verifica su MD5 y se
/// descomprime en la carpeta del addon. La URL es configurable para poder publicar el índice en
/// otro sitio sin recompilar.
fn draw_guild_index(ui: &Ui, edits: &mut Edits) {
    ui.separator();
    ui.text(i18n::text("Guild names index"));
    let known = crate::names::index_len();
    match crate::guild_index::status() {
        crate::guild_index::Status::Idle => {
            if known > 0 {
                ui.text_disabled(i18n::format(
                    "{0} guild names ready to search",
                    &[&known.to_string()],
                ));
            } else {
                ui.text_disabled(i18n::text(
                    "Not downloaded yet: the search only asks the API, which returns one guild at most.",
                ));
            }
        }
        crate::guild_index::Status::Downloading => {
            ui.text_disabled(i18n::text("Downloading the index…"));
        }
        crate::guild_index::Status::Done(count) => {
            ui.text_disabled(i18n::format(
                "{0} guild names ready to search",
                &[&count.to_string()],
            ));
        }
        crate::guild_index::Status::Failed(error) => {
            let _tint = ui.push_style_color(StyleColor::Text, [1.0, 0.55, 0.35, 1.0]);
            ui.text_wrapped(i18n::format("Index download failed: {0}", &[&error]));
        }
    }
    ui.input_text(i18n::widget("Index URL"), &mut edits.guild_index_url)
        .build();
    if ui.is_item_hovered() {
        ui.tooltip_text(i18n::text(
            "Where the ZIP with the guild names is published, next to its .md5.",
        ));
    }
    if ui.button(i18n::widget("Download the index")) {
        crate::guild_index::download(edits.guild_index_url.trim().to_owned());
    }
    if ui.is_item_hovered() {
        ui.tooltip_text(i18n::text(
            "Reads the published ZIP, checks its MD5 and replaces the index on disk.",
        ));
    }
}

fn draw_region(ui: &Ui, edits: &mut Edits, candidates: &[Candidate]) {
    // The two regions always exist; listing only the ones with live teams left the drop-down
    // empty until the first poll finished, which looked like a broken control (bug report of
    // 2026-10-31). Regions without teams are still selectable and the team list explains why
    // it is empty.
    let current = edits
        .narrowing
        .region
        .map(region_label)
        .unwrap_or_else(|| i18n::text("Region"));

    ComboBox::new(i18n::widget("Region"))
        .preview_value(current)
        .build(ui, || {
            for region in [Region::Eu, Region::Na] {
                let label = if candidates.is_empty() {
                    // No data yet: a bare "(0)" looks like a bug, so say nothing rather than
                    // lie about how many teams there are (bug of 2026-10-31).
                    format!("{}###region{}", region_label(region), region_code(region))
                } else {
                    let live = candidates
                        .iter()
                        .filter(|candidate| candidate.region() == Some(region))
                        .count();
                    format!(
                        "{} ({live})###region{}",
                        region_label(region),
                        region_code(region)
                    )
                };
                if Selectable::new(label)
                    .selected(edits.narrowing.region == Some(region))
                    .build(ui)
                {
                    edits.narrowing.region = Some(region);
                }
            }
        });
}

fn draw_colour(ui: &Ui, edits: &mut Edits) {
    let current = edits
        .narrowing
        .colour
        .map(colour_label)
        .unwrap_or_else(|| i18n::text("Colour"));

    ComboBox::new(i18n::widget("Colour"))
        .preview_value(current)
        .build(ui, || {
            for colour in TeamColor::ALL {
                let _tint = ui.push_style_color(StyleColor::Text, colour_rgba(colour));
                let label = format!("{}###colour{:?}", colour_label(colour), colour);
                if Selectable::new(label)
                    .selected(edits.narrowing.colour == Some(colour))
                    .build(ui)
                {
                    edits.narrowing.colour = Some(colour);
                }
            }
        });
}

fn draw_team(
    ui: &Ui,
    table: &TeamTable,
    edits: &mut Edits,
    candidates: &[Candidate],
    worlds: &std::collections::BTreeMap<u32, String>,
) {
    let language = i18n::effective().code();
    let options = match (edits.narrowing.region, edits.narrowing.colour) {
        (Some(region), Some(colour)) => select::of_region_and_color(candidates, region, colour),
        _ => Vec::new(),
    };

    let current = match edits.team_id {
        Some(_) if !edits.translation.trim().is_empty() => edits.translation.trim().to_owned(),
        Some(id) => table.label_in(language, id),
        None => i18n::text("No team configured yet").to_owned(),
    };

    ComboBox::new(i18n::widget("Team"))
        .preview_value(current)
        .height(nexus::imgui::ComboBoxHeight::Large)
        .build(ui, || {
            if Selectable::new(i18n::widget("No team configured yet"))
                .selected(edits.team_id.is_none())
                .build(ui)
            {
                edits.team_id = None;
            }
            if options.is_empty() {
                ui.text_disabled(i18n::text("No live teams for that region and colour"));
                return;
            }
            for candidate in options {
                // Los mundos antiguos de ese equipo, que es como el jugador lo reconoce: los
                // nombres de equipo son nuevos y no dicen nada (informe de 2026-10-31). Nada de
                // «1-1»: la región, en palabras (insistencia del jugador, 2026-10-31).
                let worlds = worlds_label(&candidate.worlds, worlds);
                let region = match super::region_of(&candidate.match_id) {
                    Some(Region::Eu) => region_label(Region::Eu),
                    Some(Region::Na) => region_label(Region::Na),
                    None => "?",
                };
                let head = format!(
                    "{}  ·  {region}",
                    table.label_in(language, candidate.team_id)
                );
                let label = format!("{head}{worlds}###team{}", candidate.team_id);
                let _tint = ui.push_style_color(StyleColor::Text, colour_rgba(candidate.color));
                if Selectable::new(label)
                    .selected(edits.team_id == Some(candidate.team_id))
                    .build(ui)
                {
                    edits.team_id = Some(candidate.team_id);
                }
            }
        });
}

/// `  ·  Lutgardis, Vabbi` con los mundos antiguos del equipo, o vacío si no se saben.
///
/// Es la única forma de reconocer un equipo nuevo: los nombres rotan cada temporada y el jugador
/// sigue pensando en su mundo de siempre.
fn worlds_label(ids: &[u32], worlds: &std::collections::BTreeMap<u32, String>) -> String {
    let names: Vec<&str> = ids
        .iter()
        .filter_map(|id| worlds.get(id).map(String::as_str))
        .collect();
    if names.is_empty() {
        return String::new();
    }
    format!("  ·  {}", names.join(", "))
}

/// The translation the field starts with: what the player saved, or the automatic name.
///
/// The language is a parameter, not read here, so the rule can be tested: reading the active
/// language touches Nexus' RealTime API, which panics outside the game.
fn initial_translation(
    config: &crate::config::Config,
    table: &TeamTable,
    selected: Option<u32>,
    language: &str,
) -> String {
    if let Some(team_id) = selected {
        if let Some(custom) = config.custom_name(team_id, language) {
            return custom.to_owned();
        }
    }
    match selected {
        Some(id) => table
            .name_in(language, id)
            .map(str::to_owned)
            .unwrap_or_default(),
        None => String::new(),
    }
}

/// Stores what the player left in the field. Writing the automatic name back is not an
/// override: the configuration drops the entry so the bundled translations keep applying.
fn remember_translation(config: &mut crate::config::Config, written: &str, language: &str) {
    let Some(team_id) = crate::team::selected_in(config) else {
        // No team chosen: there is nothing to name yet.
        return;
    };
    let automatic = crate::team::automatic_name(language, team_id);
    config.set_custom_name(team_id, language, written, &automatic);
}

fn draw_language(ui: &Ui, edits: &mut Edits) {
    let current = match edits.language {
        LanguagePreference::Auto => i18n::text("Auto (game language)"),
        LanguagePreference::Es => i18n::text("Spanish"),
        LanguagePreference::En => i18n::text("English"),
        LanguagePreference::Fr => i18n::text("French"),
        LanguagePreference::De => i18n::text("German"),
    };
    ComboBox::new(i18n::widget("Language"))
        .preview_value(current)
        .build(ui, || {
            for (preference, id, key) in [
                (LanguagePreference::Auto, "auto", "Auto (game language)"),
                (LanguagePreference::Es, "es", "Spanish"),
                (LanguagePreference::En, "en", "English"),
                (LanguagePreference::Fr, "fr", "French"),
                (LanguagePreference::De, "de", "German"),
            ] {
                let label = format!("{}###lang{id}", i18n::text(key));
                if Selectable::new(label)
                    .selected(edits.language == preference)
                    .build(ui)
                {
                    edits.language = preference;
                }
            }
        });
}

fn draw_scope(ui: &Ui, edits: &mut Edits) {
    let current = match edits.scope {
        AlertScope::CurrentMap => i18n::text("Current map only"),
        AlertScope::AllMaps => i18n::text("All maps of the matchup"),
    };
    ComboBox::new(i18n::widget("Alert scope"))
        .preview_value(current)
        .build(ui, || {
            for (scope, key) in [
                (AlertScope::CurrentMap, "Current map only"),
                (AlertScope::AllMaps, "All maps of the matchup"),
            ] {
                let label = format!("{}###scope{key}", i18n::text(key));
                if Selectable::new(label)
                    .selected(edits.scope == scope)
                    .build(ui)
                {
                    edits.scope = scope;
                }
            }
        });
}

fn region_label(region: Region) -> &'static str {
    match region {
        Region::Eu => i18n::text("Europe"),
        Region::Na => i18n::text("North America"),
    }
}

fn region_code(region: Region) -> &'static str {
    match region {
        Region::Eu => "eu",
        Region::Na => "na",
    }
}

fn colour_label(colour: TeamColor) -> &'static str {
    match colour {
        TeamColor::Red => i18n::text("Red"),
        TeamColor::Green => i18n::text("Green"),
        TeamColor::Blue => i18n::text("Blue"),
    }
}

/// Same colours as the in-game WvW panel, so the list is recognisable at a glance.
fn colour_rgba(colour: TeamColor) -> [f32; 4] {
    match colour {
        TeamColor::Red => [1.0, 0.42, 0.42, 1.0],
        TeamColor::Green => [0.55, 0.92, 0.55, 1.0],
        TeamColor::Blue => [0.48, 0.72, 1.0, 1.0],
    }
}

#[cfg(test)]
mod tests;
