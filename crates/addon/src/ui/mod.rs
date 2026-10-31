//! Windows of the addon. Every one of them goes through [`window::draw`] (RF-46).
//!
//! Every literal comes from the translation table through [`crate::i18n`] (RF-44); the
//! English text is the key, so nothing here needs touching when a language changes.

mod alerts;
mod guilds;
mod map;
mod matchup;
mod settings;
mod window;

use nexus::gui::{register_render, render, RenderType};

use std::sync::atomic::{AtomicBool, Ordering};

use crate::i18n;

/// Translation key (and stable id) of the only window of the addon (T-42).
///
/// One window with tabs, as in `docs/boceto.html`: alerts, map, guilds, log and — last — the
/// settings, so everything the player needs is in the same place.
pub const WIDGET: &str = "WvW Lurker";

/// Tabs of the widget, in order. The last one is always the settings (T-42).
const TABS: [&str; 5] = [
    alerts::KEY,
    map::KEY,
    guilds::KEY,
    alerts::LOG_KEY,
    settings::KEY,
];

/// Identifier of the lock keybind. Nexus lists it with this name, so the user can rebind it.
const LOCK_KEYBIND: &str = "GW2LURKER_CLICK_THROUGH";

/// Shortcut registered by default for the lock toggle.
const LOCK_SHORTCUT: &str = "ALT+SHIFT+L";

/// Set by the keybind handler and consumed by the render callback.
///
/// The handler runs on Nexus' input thread: taking the configuration lock there would mean
/// blocking a thread we do not own, so it only raises a flag (RF-18).
static LOCK_TOGGLE: AtomicBool = AtomicBool::new(false);

fn on_lock_keybind(_identifier: &str, is_release: bool) {
    if !is_release {
        LOCK_TOGGLE.store(true, Ordering::Relaxed);
    }
}

fn take_lock_toggle() -> bool {
    LOCK_TOGGLE.swap(false, Ordering::Relaxed)
}

/// Locks the widget (click-through) or unlocks it, and returns the new state.
///
/// Only a window that already exists is touched: asking for a missing one would create it with
/// `visible = true`, and a widget the user closed would reappear. Pure, so the rule is testable
/// without a game.
pub fn toggle_locked(config: &mut crate::config::Config) -> bool {
    let lock = !config.window(WIDGET).locked;
    config.window(WIDGET).locked = lock;
    lock
}

/// Registers the render callbacks. Must be called from `load`, never per frame.
pub fn register() {
    nexus::keybind::register_keybind_with_string(
        LOCK_KEYBIND,
        nexus::keybind::keybind_handler!(on_lock_keybind),
        LOCK_SHORTCUT,
    )
    .revert_on_unload();

    register_render(
        RenderType::Render,
        render!(|ui| {
            i18n::register_translations_once();
            let gate = crate::context::gate();
            crate::context::log_gate(gate);
            crate::worker::set_may_poll(gate.poll);

            if take_lock_toggle() {
                let mut config = crate::runtime();
                let locked = toggle_locked(&mut config);
                crate::request_save(&config);
                log::info!("click-through: {locked}");
            }

            if gate.show_ui || crate::config().debug_show_everywhere {
                draw_widget(ui);
            }
            // The team picker needs the list of live teams even outside WvW, where polling is
            // off: without this the drop-down was empty (bug of 2026-10-31).
            crate::worker::set_teams_needed(
                gate.settings_available() || crate::config().debug_show_everywhere,
            );
            crate::persist_if_changed();
        }),
    )
    .revert_on_unload();

    register_render(
        RenderType::OptionsRender,
        render!(|ui| {
            // Nexus' options pane is the way back to everything, including the widget itself
            // and the pop-ups (T-42): it draws even when the overlay is hidden.
            draw_openers(ui);
            let (candidates, guilds) = {
                let board = crate::worker::dashboard();
                (board.candidates.clone(), board.guilds.clone())
            };
            let change = settings::draw_inline(ui, &candidates, &guilds);
            match change {
                Some(settings::TeamChange::Pick(team_id)) => crate::team::remember(team_id),
                Some(settings::TeamChange::Clear) => crate::team::forget(),
                Some(settings::TeamChange::SaveNow) => {
                    let config = crate::runtime();
                    crate::request_save(&config);
                    log::info!("ajustes guardados a mano");
                }
                None => {}
            }
        }),
    )
    .revert_on_unload();
}

/// Draws the only window: a tab bar and the body of the selected tab (T-42).
///
/// The data comes from the worker and is copied out before the configuration lock is taken:
/// two mutexes are involved and the dashboard one must never be held while the other is.
fn draw_widget(ui: &nexus::imgui::Ui) {
    static FILTERS: std::sync::OnceLock<std::sync::Mutex<matchup::Filters>> =
        std::sync::OnceLock::new();

    let filters = FILTERS.get_or_init(|| std::sync::Mutex::new(matchup::Filters::all_kinds()));
    let current = crate::context::current_map_role();
    let (candidates, guilds, names, claims, alerts, objectives, maps) = {
        let board = crate::worker::dashboard();
        (
            board.candidates.clone(),
            board.guilds.clone(),
            board.guild_names.clone(),
            board.claims.clone(),
            board.recent_alerts(
                &crate::config().alerts,
                &crate::config().followed,
                current,
                50,
            ),
            board.names.clone(),
            board.maps.clone(),
        )
    };
    let follow = crate::config().followed.clone();
    let now = chrono::Utc::now();

    let mut config = crate::runtime();
    let mut state = config.window(WIDGET).clone();
    let wanted_visible = state.visible;
    let wanted_locked = state.locked;
    let mut change = None;
    let mut filters = filters.lock().unwrap_or_else(|error| error.into_inner());

    let still_open = window::draw(ui, WIDGET, 0, [470.0, 480.0], &mut state, |ui| {
        change = draw_tabs(
            ui,
            &mut config,
            &mut filters,
            &Tabbed {
                candidates: &candidates,
                guilds: &guilds,
                names: &names,
                claims: &claims,
                alerts: &alerts,
                follow: &follow,
                current,
                now,
                objectives: &objectives,
                maps: &maps,
            },
        );
    });
    drop(filters);

    // The settings tab can hide or lock the widget in the same frame the chrome runs, so the
    // copy is written back field by field: the chrome only wins on the fields nobody else
    // touched this frame.
    let slot = config.window(WIDGET);
    if slot.visible == wanted_visible {
        slot.visible = still_open && state.visible;
    }
    if slot.locked == wanted_locked {
        slot.locked = state.locked;
    }
    slot.collapsed = state.collapsed;
    slot.opacity = state.opacity;
    slot.position = state.position;
    slot.size = state.size;
    drop(config);

    match change {
        Some(settings::TeamChange::Pick(team_id)) => crate::team::remember(team_id),
        Some(settings::TeamChange::Clear) => crate::team::forget(),
        Some(settings::TeamChange::SaveNow) => {
            let config = crate::runtime();
            crate::request_save(&config);
            log::info!("ajustes guardados a mano");
        }
        None => {}
    }

    // The pop-up is drawn outside the window on purpose: it has to appear even when the widget
    // itself is closed, which is what the pop-up switch is for.
    draw_popup(ui, &alerts, now);
}

/// Everything the tabs draw, gathered once per frame.
struct Tabbed<'a> {
    candidates: &'a [gw2lurker_core::Candidate],
    guilds: &'a [gw2lurker_core::matchup::GuildSummary],
    names: &'a std::collections::BTreeMap<String, String>,
    claims: &'a [gw2lurker_core::matchup::ClaimRow],
    alerts: &'a [crate::worker::AlertLine],
    follow: &'a gw2lurker_core::FollowList,
    current: Option<gw2lurker_core::MapRole>,
    now: chrono::DateTime<chrono::Utc>,
    /// Objectives of the player's language, with the `coord` the map needs.
    objectives: &'a [gw2lurker_core::ObjectiveInfo],
    /// Boxes of the four WvW maps (RF-30).
    maps: &'a [gw2lurker_core::WvwMap],
}

/// Draws the tab bar and the body of the tab the player picked. The last tab is the settings.
fn draw_tabs(
    ui: &nexus::imgui::Ui,
    config: &mut crate::config::Config,
    filters: &mut matchup::Filters,
    data: &Tabbed,
) -> Option<settings::TeamChange> {
    use nexus::imgui::{TabBar, TabItem};

    let mut change = None;

    // The status of the worker, above the tabs: it is what tells a bug from a quiet matchup.
    draw_health(ui, config, data);
    ui.separator();

    if let Some(_bar) = TabBar::new("###gw2lurker_tabs").begin(ui) {
        for (index, key) in TABS.iter().enumerate() {
            TabItem::new(i18n::widget(key)).build(ui, || match index {
                0 => alerts::body(ui, data.alerts, data.now),
                1 => {
                    let history = guilds::selected_guild()
                        .map(|guild| crate::worker::dashboard().guild_history(&guild))
                        .unwrap_or_default();
                    map::body(
                        ui,
                        &map::View {
                            rows: data.claims,
                            names: data.objectives,
                            maps: data.maps,
                            follow: data.follow,
                            history: &history,
                        },
                        &mut map_choice(),
                        data.current,
                    );
                    if !history.is_empty() {
                        // ADR-005: la ruta es una deducción, y se dice siempre.
                        ui.text_disabled(i18n::text(
                            "Route inferred from claims; it is not the real path.",
                        ));
                    }
                    // The list is the same data as the map, one click away: during a fight the
                    // text is faster to read than the picture.
                    ui.separator();
                    if nexus::imgui::CollapsingHeader::new(i18n::widget("Matchup claims")).build(ui)
                    {
                        matchup::body(
                            ui,
                            filters,
                            data.claims,
                            data.current,
                            data.follow,
                            data.names,
                        );
                    }
                }
                2 => {
                    if let Some(guilds::Action::ToggleFollow(guild)) = guilds::body(
                        ui,
                        data.guilds,
                        data.names,
                        data.follow,
                        data.current,
                        |guild| crate::worker::dashboard().guild_history(guild),
                    ) {
                        config.followed.toggle(&guild);
                    }
                }
                3 => alerts::log_body(ui, data.alerts),
                _ => change = settings::body(ui, config, data.candidates, data.guilds),
            });
        }
    }
    change
}

/// Which map the *Map* tab draws: the session's choice, `Unknown` until the player picks one.
///
/// Session state on purpose: the default is always the map the player is on, which is the one
/// they care about.
fn map_choice() -> std::sync::MutexGuard<'static, gw2lurker_core::MapRole> {
    static CHOICE: std::sync::OnceLock<std::sync::Mutex<gw2lurker_core::MapRole>> =
        std::sync::OnceLock::new();
    CHOICE
        .get_or_init(|| std::sync::Mutex::new(gw2lurker_core::MapRole::Unknown))
        .lock()
        .unwrap_or_else(|error| error.into_inner())
}

/// One line with the state of the worker and the configured team (RF-02, RF-46).
fn draw_health(ui: &nexus::imgui::Ui, config: &mut crate::config::Config, data: &Tabbed) {
    let state = crate::context::snapshot();
    let (health, match_id, updated) = {
        let board = crate::worker::dashboard();
        (
            board.health.clone(),
            board.match_id.clone(),
            board.updated_at,
        )
    };
    let team_id = crate::team::selected_in(config);
    let language = i18n::effective().code();

    let map_id = state.map_id.to_string();
    let map_type = state.map_type.to_string();
    ui.text_disabled(i18n::format("Map {0} (type {1})", &[&map_id, &map_type]));
    match team_id.map(|id| crate::team::display_name(config, language, id)) {
        Some(label) => ui.text(i18n::format("Team {0}", &[&label])),
        None => ui.text(i18n::text("No team configured yet")),
    }
    if let Some(id) = &match_id {
        ui.text_disabled(i18n::format("Matchup {0}", &[id]));
    }
    let summary = match &health {
        crate::worker::Health::Idle => i18n::text("No data yet").to_owned(),
        crate::worker::Health::Ok { .. } => {
            i18n::format("{0} claimed objectives", &[&data.claims.len().to_string()])
        }
        crate::worker::Health::Degraded { error, .. } => i18n::format("Error: {0}", &[error]),
    };
    let when = updated
        .map(|at| at.format("%H:%M:%S").to_string())
        .unwrap_or_default();
    if when.is_empty() {
        ui.text_disabled(summary);
    } else {
        ui.text_disabled(format!("{summary} · {} {when}", i18n::text("Last update")));
    }
}

/// Pop-up with what just happened, on top of the screen (RF-19a).
fn draw_popup(
    ui: &nexus::imgui::Ui,
    alerts: &[crate::worker::AlertLine],
    now: chrono::DateTime<chrono::Utc>,
) {
    let (enabled, secs, opacity) = {
        let config = crate::config();
        let opacity = config
            .windows
            .get(WIDGET)
            .map(|state| state.opacity)
            .unwrap_or_default();
        (config.popup_enabled, config.popup_secs, opacity)
    };
    if !enabled {
        return;
    }
    let showing = alerts::popup_lines(alerts, now, secs);
    alerts::draw_popup(ui, &showing, opacity);
}

/// The two switches Nexus' options pane offers first (T-42).
fn draw_openers(ui: &nexus::imgui::Ui) {
    let mut config = crate::runtime();
    let mut visible = config.window(WIDGET).visible;
    let shown = ui.checkbox(i18n::widget("Show widget"), &mut visible);
    let popups = ui.checkbox(i18n::widget("Pop-up alerts"), &mut config.popup_enabled);
    if shown {
        config.window(WIDGET).visible = visible;
    }
    if popups {
        // Turning them back on with a duration of zero would do nothing: 2 s is the default.
        if config.popup_enabled && config.popup_secs <= 0.0 {
            config.popup_secs = 2.0;
        }
        crate::request_save(&config);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rf18_the_shortcut_locks_and_unlocks_the_widget() {
        let mut config = crate::config::Config::default();

        assert!(toggle_locked(&mut config), "la primera pulsación bloquea");
        assert!(
            config.window(WIDGET).locked,
            "bloqueada = click-through; si no, un clic se pierde en la pelea"
        );

        assert!(!toggle_locked(&mut config), "la segunda desbloquea");
        assert!(!config.window(WIDGET).locked);
    }

    #[test]
    fn rf46_locking_does_not_reopen_a_closed_widget() {
        let mut config = crate::config::Config::default();
        config.window(WIDGET).visible = false;

        toggle_locked(&mut config);

        assert!(
            !config.window(WIDGET).visible,
            "bloquear no puede resucitar una ventana que el usuario cerró"
        );
    }

    #[test]
    fn t42_the_settings_are_the_last_tab() {
        assert_eq!(
            *TABS.last().expect("hay pestañas"),
            settings::KEY,
            "los ajustes van al final, como en docs/boceto.html"
        );
        assert_eq!(TABS.len(), 5, "Avisos, Mapa, Clanes, Registro y Ajustes");
    }
}
