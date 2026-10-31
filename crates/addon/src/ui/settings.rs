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
use gw2lurker_core::{select, Candidate, TeamTable};
use nexus::imgui::{ComboBox, StyleColor, Ui};

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
    alias: String,
    language: LanguagePreference,
    poll_secs: i32,
    scope: AlertScope,
    narrowing: Narrowing,
}

/// Draws the window; the per-window UI state comes from `config` like every other window.
///
/// `candidates` are the teams playing right now, from the worker. An empty list means the
/// first poll has not finished, and the picker says so instead of pretending.
pub fn draw(
    ui: &Ui,
    config: &mut crate::config::Config,
    candidates: &[Candidate],
) -> Option<TeamChange> {
    let table = gw2lurker_core::bundled();
    let selected = crate::team::selected_in(config);
    let mut edits = Edits {
        team_id: selected,
        alias: config.team_alias.clone().unwrap_or_default(),
        language: config.language,
        poll_secs: config.poll_secs as i32,
        scope: config.alert_scope,
        narrowing: initial_narrowing(candidates, selected),
    };
    let before = edits.team_id;

    {
        let state = config.window(KEY);
        crate::ui::window::draw(ui, KEY, 1, [470.0, 320.0], state, |ui| {
            draw_body(ui, table, &mut edits, candidates)
        });
    }

    config.team_alias = Some(edits.alias.trim().to_owned()).filter(|alias| !alias.is_empty());
    config.language = edits.language;
    config.alert_scope = edits.scope;
    config.poll_secs = edits.poll_secs.clamp(15, 600) as u32;
    set_narrowing(edits.narrowing);

    team_change(before, edits.team_id)
}

/// Draws the settings inline, without window chrome.
///
/// Used by Nexus' own options pane (`RenderType::OptionsRender`), which is reachable even
/// outside WvW: that is how RF-02 keeps the configuration available everywhere.
pub fn draw_inline(ui: &Ui, candidates: &[Candidate]) -> Option<TeamChange> {
    let table = gw2lurker_core::bundled();
    let mut config = crate::runtime();
    let selected = crate::team::selected_in(&config);
    let mut edits = Edits {
        team_id: selected,
        alias: config.team_alias.clone().unwrap_or_default(),
        language: config.language,
        poll_secs: config.poll_secs as i32,
        scope: config.alert_scope,
        narrowing: initial_narrowing(candidates, selected),
    };
    let before = edits.team_id;

    draw_body(ui, table, &mut edits, candidates);

    config.team_alias = Some(edits.alias.trim().to_owned()).filter(|alias| !alias.is_empty());
    config.language = edits.language;
    config.alert_scope = edits.scope;
    config.poll_secs = edits.poll_secs.clamp(15, 600) as u32;
    drop(config);
    set_narrowing(edits.narrowing);

    team_change(before, edits.team_id)
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
fn initial_narrowing(candidates: &[Candidate], selected: Option<u32>) -> Narrowing {
    let remembered = narrowing();
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

fn draw_body(ui: &Ui, table: &TeamTable, edits: &mut Edits, candidates: &[Candidate]) {
    // imgui puts the label to the right of the widget: cap the widget so the translated labels
    // of a narrow window are not cut off, whatever the language.
    let longest = i18n::text("No live teams for that region and colour");
    let label_width = ui.calc_text_size(longest)[0] + 16.0;
    let available = ui.content_region_avail()[0];
    let _item_width = ui.push_item_width((available - label_width).max(120.0));

    if candidates.is_empty() {
        ui.text_wrapped(i18n::text("Pick region and colour, then your team"));
        ui.text_disabled(i18n::text("No data yet"));
        ui.separator();
    }

    draw_region(ui, edits, candidates);
    draw_colour(ui, edits);
    draw_team(ui, table, edits, candidates);

    let alias_id = i18n::widget("Alias");
    ui.input_text(alias_id, &mut edits.alias).build();
    if ui.is_item_hovered() {
        ui.tooltip_text(i18n::text("Alias (only for you)"));
    }

    draw_language(ui, edits);
    ui.input_int(i18n::widget("Poll interval"), &mut edits.poll_secs)
        .build();
    draw_scope(ui, edits);
}

fn draw_region(ui: &Ui, edits: &mut Edits, candidates: &[Candidate]) {
    let present: Vec<Region> = [Region::Eu, Region::Na]
        .into_iter()
        .filter(|region| {
            candidates
                .iter()
                .any(|candidate| candidate.region() == Some(*region))
        })
        .collect();
    let current = edits
        .narrowing
        .region
        .map(region_label)
        .unwrap_or_else(|| i18n::text("Region"));

    ComboBox::new(i18n::widget("Region"))
        .preview_value(current)
        .build(ui, || {
            for region in present {
                let label = format!("{}###region{}", region_label(region), region_code(region));
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
        Some(id) => table_alias(&edits.alias).unwrap_or_else(|| table.label_in(language, id)),
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

/// The alias replaces the name of the chosen team, and only while there is one.
fn table_alias(alias: &str) -> Option<String> {
    let alias = alias.trim();
    (!alias.is_empty()).then(|| alias.to_owned())
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
        set_narrowing(Narrowing::default());
        let start = initial_narrowing(&list, Some(12015));
        assert_eq!(start.region, Some(Region::Eu));
        assert_eq!(start.colour, Some(TeamColor::Red));

        set_narrowing(Narrowing::default());
        let fallback = initial_narrowing(&list, None);
        assert_eq!(fallback.region, Some(Region::Eu), "la región del primero");
        assert_eq!(fallback.colour, None, "sin equipo no se adivina el color");

        set_narrowing(Narrowing::default());
        assert_eq!(initial_narrowing(&[], None), Narrowing::default());
    }

    #[test]
    fn rf03a_narrowing_is_remembered_between_frames() {
        set_narrowing(Narrowing {
            region: Some(Region::Na),
            colour: Some(TeamColor::Green),
        });
        let list = vec![candidate(12014, "2-2", TeamColor::Blue)];
        let start = initial_narrowing(&list, Some(12014));
        assert_eq!(
            start,
            Narrowing {
                region: Some(Region::Na),
                colour: Some(TeamColor::Green)
            },
            "lo que el jugador eligió gana a lo deducido"
        );
        set_narrowing(Narrowing::default());
    }

    #[test]
    fn rf03f_alias_only_replaces_a_team_name() {
        assert_eq!(table_alias("Mi equipo").as_deref(), Some("Mi equipo"));
        assert_eq!(table_alias("  Mi equipo  ").as_deref(), Some("Mi equipo"));
        assert_eq!(table_alias("   "), None);
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
