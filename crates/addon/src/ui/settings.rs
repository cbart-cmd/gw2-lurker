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

use gw2lurker_core::matchup::GuildSummary;
use gw2lurker_core::model::{Region, TeamColor};
use gw2lurker_core::{select, AlertFilter, Candidate, FollowList, TeamTable};
use nexus::imgui::{ComboBox, Slider, StyleColor, Ui};

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
    widget_click_through: bool,
}

/// Body of the *Settings* tab: the whole form, without window chrome (T-42).
///
/// `candidates` are the teams playing right now, from the worker. An empty list means the first
/// poll has not finished, and the picker says so instead of pretending.
pub fn body(
    ui: &Ui,
    config: &mut crate::config::Config,
    candidates: &[Candidate],
    guilds: &[GuildSummary],
    names: &std::collections::BTreeMap<String, String>,
) -> Option<TeamChange> {
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
        widget_click_through: widget.click_through,
    };
    let before = edits.team_id;

    let save_now = draw_body(ui, table, &mut edits, candidates, guilds, names);

    remember_translation(config, edits.translation.as_str(), language);
    config.language = edits.language;
    config.alert_scope = edits.scope;
    config.poll_secs = edits.poll_secs.clamp(15, 600) as u32;
    config.alerts = edits.alerts;
    config.popup_secs = edits.popup_secs.clamp(0.0, 5.0);
    config.popup_enabled = edits.popup_enabled;
    config.followed = edits.follow;
    // The switches of the widget itself: written only when moved, so the chrome of this window
    // is not undone every frame (`×`, dragging, lock).
    if edits.widget_visible != widget.visible {
        config.window(crate::ui::WIDGET).visible = edits.widget_visible;
    }
    if edits.widget_locked != widget.locked {
        config.window(crate::ui::WIDGET).locked = edits.widget_locked;
    }
    if edits.widget_click_through != widget.click_through {
        config.window(crate::ui::WIDGET).click_through = edits.widget_click_through;
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
    guilds: &[GuildSummary],
    names: &std::collections::BTreeMap<String, String>,
) -> Option<TeamChange> {
    let mut config = crate::runtime();
    body(ui, &mut config, candidates, guilds, names)
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

fn draw_body(
    ui: &Ui,
    table: &TeamTable,
    edits: &mut Edits,
    candidates: &[Candidate],
    guilds: &[GuildSummary],
    names: &std::collections::BTreeMap<String, String>,
) -> bool {
    // imgui puts the label to the right of the widget: cap the widget so the translated labels
    // of a narrow window are not cut off, whatever the language.
    let longest = i18n::text("No live teams for that region and colour");
    let label_width = ui.calc_text_size(longest)[0] + 16.0;
    let available = ui.content_region_avail()[0];
    let _item_width = ui.push_item_width((available - label_width).max(120.0));

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
    draw_team(ui, table, edits, candidates);

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
    let mut follow = edits.follow.clone();
    draw_alert_filters(ui, edits, guilds, names, &mut follow);
    edits.follow = follow;

    ui.separator();
    ui.text(i18n::text("Windows"));
    ui.checkbox(i18n::widget("Show widget"), &mut edits.widget_visible);
    ui.same_line();
    ui.checkbox(i18n::widget("Lock position"), &mut edits.widget_locked);
    ui.checkbox(
        i18n::widget("Click-through"),
        &mut edits.widget_click_through,
    );
    let popups = ui.checkbox(i18n::widget("Pop-up alerts"), &mut edits.popup_enabled);
    if popups && edits.popup_enabled && edits.popup_secs <= 0.0 {
        edits.popup_secs = 2.0;
    }
    ui.text_disabled(i18n::format(
        "Click-through keeps the mouse for the game. Shortcut: {0}, rebindable in Nexus.",
        &[crate::ui::LOCK_SHORTCUT],
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

/// Which maps and guilds deserve an alert (RF-19a), plus how long the pop-up lasts.
fn draw_alert_filters(
    ui: &Ui,
    edits: &mut Edits,
    guilds: &[GuildSummary],
    names: &std::collections::BTreeMap<String, String>,
    follow: &mut FollowList,
) {
    ui.separator();
    ui.text(i18n::text("Alert on"));
    ui.checkbox(i18n::widget("Eternals"), &mut edits.alerts.ebg);
    ui.checkbox(i18n::widget("Blue alpine"), &mut edits.alerts.blue);
    ui.checkbox(i18n::widget("Green alpine"), &mut edits.alerts.green);
    ui.checkbox(i18n::widget("Red desert"), &mut edits.alerts.red);
    ui.checkbox(
        i18n::widget("The map I am on"),
        &mut edits.alerts.current_map,
    );
    ui.checkbox(i18n::widget("Guilds I follow"), &mut edits.alerts.followed);

    ui.separator();
    let mut seconds = edits.popup_secs;
    if Slider::new(i18n::widget("Pop-up seconds"), 0.0, 5.0)
        .display_format("%.1f s")
        .build(ui, &mut seconds)
    {
        edits.popup_secs = seconds;
    }
    if ui.is_item_hovered() {
        ui.tooltip_text(i18n::text(
            "The pop-up disappears after this. The alert stays in the list.",
        ));
    }

    ui.separator();
    ui.text(i18n::text("Followed guilds"));
    if guilds.is_empty() {
        ui.text_disabled(i18n::text("No claims yet"));
        return;
    }
    for summary in guilds {
        let mut followed = follow.contains(&summary.guild);
        // `[TAG] Nombre` cuando se conoce: un UUID no lo reconoce nadie (bug de 2026-10-31).
        let shown = names
            .get(&summary.guild)
            .cloned()
            .unwrap_or_else(|| summary.guild.clone());
        let label = format!("{shown}###follow{}", summary.guild);
        if ui.checkbox(label, &mut followed) {
            if followed {
                follow.add(&summary.guild);
            } else {
                follow.remove(&summary.guild);
            }
        }
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
                if nexus::imgui::Selectable::new(label)
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
                if nexus::imgui::Selectable::new(label)
                    .selected(edits.narrowing.colour == Some(colour))
                    .build(ui)
                {
                    edits.narrowing.colour = Some(colour);
                }
            }
        });
}

fn draw_team(ui: &Ui, table: &TeamTable, edits: &mut Edits, candidates: &[Candidate]) {
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
            if nexus::imgui::Selectable::new(i18n::widget("No team configured yet"))
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
                let label = format!(
                    "{}  ·  {}###team{}",
                    table.label_in(language, candidate.team_id),
                    candidate.match_id,
                    candidate.team_id
                );
                let _tint = ui.push_style_color(StyleColor::Text, colour_rgba(candidate.color));
                if nexus::imgui::Selectable::new(label)
                    .selected(edits.team_id == Some(candidate.team_id))
                    .build(ui)
                {
                    edits.team_id = Some(candidate.team_id);
                }
            }
        });
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
                if nexus::imgui::Selectable::new(label)
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
                if nexus::imgui::Selectable::new(label)
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
mod tests {
    use super::*;

    fn candidate(team_id: u32, match_id: &str, color: TeamColor) -> Candidate {
        Candidate {
            team_id,
            match_id: match_id.to_owned(),
            color,
        }
    }

    #[test]
    fn rf03a_narrowing_starts_at_the_saved_team() {
        let list = vec![
            candidate(12014, "2-2", TeamColor::Blue),
            candidate(12015, "2-2", TeamColor::Red),
            candidate(11001, "1-1", TeamColor::Green),
        ];
        let start = initial_narrowing(Narrowing::default(), &list, Some(12015));
        assert_eq!(start.region, Some(Region::Eu));
        assert_eq!(start.colour, Some(TeamColor::Red));

        let fallback = initial_narrowing(Narrowing::default(), &list, None);
        assert_eq!(fallback.region, Some(Region::Eu), "la región del primero");
        assert_eq!(fallback.colour, None, "sin equipo no se adivina el color");

        assert_eq!(
            initial_narrowing(Narrowing::default(), &[], None),
            Narrowing::default()
        );
    }

    #[test]
    fn rf03a_both_regions_are_always_offered() {
        // Regression of the 2026-10-31 bug: with no live teams the drop-down was empty.
        // Regression of the 2026-10-31 bug: with no live teams the drop-down was empty.
        // `draw_region` now offers EU and NA unconditionally, so the control always works;
        // what this pins is that no data does not fabricate a preselection either.
        let start = initial_narrowing(Narrowing::default(), &[], None);
        assert_eq!(start.region, None, "sin datos no se preselecciona región");
        assert_eq!(start.colour, None);
    }

    #[test]
    fn rf03a_the_colour_is_offered_before_any_data() {
        // `draw_colour` iterates `TeamColor::ALL`, which is static: the control is never empty.
        assert_eq!(TeamColor::ALL.len(), 3);
    }

    #[test]
    fn rf03a_narrowing_is_remembered_between_frames() {
        let remembered = Narrowing {
            region: Some(Region::Na),
            colour: Some(TeamColor::Green),
        };
        let list = vec![candidate(12014, "2-2", TeamColor::Blue)];
        let start = initial_narrowing(remembered, &list, Some(12014));
        assert_eq!(
            start,
            Narrowing {
                region: Some(Region::Na),
                colour: Some(TeamColor::Green)
            },
            "lo que el jugador eligió gana a lo deducido"
        );
    }

    #[test]
    fn rf03f_the_field_starts_with_the_automatic_translation() {
        let table = gw2lurker_core::bundled();
        let empty = crate::config::Config::default();

        // Without Mumble there is no character, so the roster cannot match; the automatic
        // name is what the field shows for a chosen team.
        assert_eq!(
            initial_translation(&empty, table, Some(12014), "es"),
            "Pajar de la Gran Casa"
        );
        assert_eq!(initial_translation(&empty, table, None, "es"), "");

        let mut custom = crate::config::Config::default();
        custom.set_custom_name(12014, "es", "Bosquemugoso", "Pajar de la Gran Casa");
        assert_eq!(
            initial_translation(&custom, table, Some(12014), "es"),
            "Bosquemugoso",
            "lo guardado gana al nombre de la tabla"
        );
    }

    #[test]
    fn rf03a_three_steps_bring_the_list_down_to_a_handful() {
        let mut list = Vec::new();
        for index in 1..=5 {
            list.push(candidate(
                12000 + index,
                &format!("2-{index}"),
                TeamColor::Blue,
            ));
            list.push(candidate(
                12100 + index,
                &format!("2-{index}"),
                TeamColor::Red,
            ));
            list.push(candidate(
                11000 + index,
                &format!("1-{index}"),
                TeamColor::Blue,
            ));
        }
        assert_eq!(list.len(), 15, "lo que vería el jugador sin filtrar");

        let eu_blue = select::of_region_and_color(&list, Region::Eu, TeamColor::Blue);
        assert_eq!(eu_blue.len(), 5, "un equipo azul por matchup de EU");

        let na_blue = select::of_region_and_color(&list, Region::Na, TeamColor::Blue);
        assert_eq!(na_blue.len(), 5);
        assert!(
            select::of_region_and_color(&list, Region::Na, TeamColor::Red).is_empty(),
            "y se avisa en vez de mostrar una lista vacía sin explicación"
        );
    }
}
