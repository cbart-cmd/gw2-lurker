//! Settings window (T-17): team picker, language, polling interval and alert scope.
//!
//! All edits are collected in a local copy and written back to the runtime configuration
//! after the window is drawn, because `window::draw` already borrows the window state.

use gw2lurker_core::TeamTable;
use nexus::imgui::{ComboBox, Ui};

use crate::config::{AlertScope, LanguagePreference};
use crate::i18n;

/// Translation key (and stable id) of this window.
pub const KEY: &str = "Settings";

/// Working copy of everything this window can edit.
struct Edits {
    team_id: Option<u32>,
    alias: String,
    language: LanguagePreference,
    poll_secs: i32,
    scope: AlertScope,
}

/// Draws the settings inline, without window chrome.
///
/// Used by Nexus' own options pane (`RenderType::OptionsRender`), which is reachable even
/// outside WvW: that is how RF-02 keeps the configuration available everywhere, and it also
/// gives us a view that does not depend on the gate at all.
pub fn draw_inline(ui: &Ui) {
    let mut config = crate::runtime();
    let table = gw2lurker_core::bundled();
    let mut edits = Edits {
        team_id: config.team_id,
        alias: config.team_alias.clone().unwrap_or_default(),
        language: config.language,
        poll_secs: config.poll_secs as i32,
        scope: config.alert_scope,
    };

    draw_body(ui, table, &mut edits);

    config.team_id = edits.team_id;
    config.team_alias = Some(edits.alias.trim().to_owned()).filter(|alias| !alias.is_empty());
    config.language = edits.language;
    config.alert_scope = edits.scope;
    config.poll_secs = edits.poll_secs.clamp(15, 600) as u32;
}

/// Draws the window; the per-window UI state comes from `config` like every other window.
pub fn draw(ui: &Ui, config: &mut crate::config::Config) {
    let table = gw2lurker_core::bundled();
    let mut edits = Edits {
        team_id: config.team_id,
        alias: config.team_alias.clone().unwrap_or_default(),
        language: config.language,
        poll_secs: config.poll_secs as i32,
        scope: config.alert_scope,
    };

    {
        let state = config.window(KEY);
        crate::ui::window::draw(ui, KEY, 1, state, |ui| draw_body(ui, table, &mut edits));
    }

    config.team_id = edits.team_id;
    config.team_alias = Some(edits.alias.trim().to_owned()).filter(|alias| !alias.is_empty());
    config.language = edits.language;
    config.alert_scope = edits.scope;
    config.poll_secs = edits.poll_secs.clamp(15, 600) as u32;
}

fn draw_body(ui: &Ui, table: &TeamTable, edits: &mut Edits) {
    let current = match edits.team_id {
        Some(id) => table.label(id),
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
            for team in table.sorted() {
                let label = format!("{}###team{}", table.label(team.id), team.id);
                if nexus::imgui::Selectable::new(label)
                    .selected(edits.team_id == Some(team.id))
                    .build(ui)
                {
                    edits.team_id = Some(team.id);
                }
            }
        });

    ui.input_text(i18n::widget("Alias"), &mut edits.alias)
        .build();

    let language = match edits.language {
        LanguagePreference::Auto => i18n::text("Auto (game language)").to_owned(),
        LanguagePreference::Es => i18n::text("Spanish").to_owned(),
        LanguagePreference::En => i18n::text("English").to_owned(),
        LanguagePreference::Fr => i18n::text("French").to_owned(),
        LanguagePreference::De => i18n::text("German").to_owned(),
    };
    ComboBox::new(i18n::widget("Language"))
        .preview_value(language)
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

    ui.input_int(i18n::widget("Poll interval"), &mut edits.poll_secs)
        .build();
    if ui.is_item_hovered() {
        ui.tooltip_text(i18n::text("Polling every {0} s").replace("{0}", "15-600"));
    }

    let scope = match edits.scope {
        AlertScope::CurrentMap => i18n::text("Current map only"),
        AlertScope::AllMaps => i18n::text("All maps of the matchup"),
    };
    ComboBox::new(i18n::widget("Alert scope"))
        .preview_value(scope)
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

#[cfg(test)]
mod tests {
    use crate::config::{AlertScope, LanguagePreference};

    #[test]
    fn rf43_poll_interval_is_clamped_before_saving() {
        for (input, expected) in [(1, 15), (30, 30), (5000, 600)] {
            assert_eq!(input.clamp(15, 600) as u32, expected);
        }
    }

    #[test]
    fn rf03f_blank_alias_becomes_none() {
        let alias = Some("   ".trim().to_owned()).filter(|a| !a.is_empty());
        assert_eq!(alias, None, "un alias vacío no debe tapar el nombre real");
        let alias = Some(" Mi equipo ".trim().to_owned()).filter(|a| !a.is_empty());
        assert_eq!(alias.as_deref(), Some("Mi equipo"));
    }

    #[test]
    fn rf12_scope_defaults_to_the_current_map() {
        assert_eq!(AlertScope::default(), AlertScope::CurrentMap);
        assert_eq!(LanguagePreference::default(), LanguagePreference::Auto);
    }
}
