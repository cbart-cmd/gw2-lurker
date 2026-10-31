//! Windows of the addon. Every one of them goes through [`window::draw`] (RF-46).
//!
//! Every literal comes from the translation table through [`crate::i18n`] (RF-44); the
//! English text is the key, so nothing here needs touching when a language changes.

mod alerts;
mod guilds;
mod matchup;
mod settings;
mod window;

use nexus::gui::{register_render, render, RenderType};

use std::sync::atomic::{AtomicBool, Ordering};

use crate::i18n;

/// Translation key (and stable id) of the status window.
const STATUS: &str = "WvW Lurker";

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

/// Every window of the addon, by translation key: the settings show one switch per entry.
pub fn window_keys() -> [&'static str; 6] {
    [
        STATUS,
        settings::KEY,
        alerts::KEY,
        alerts::LOG_KEY,
        matchup::KEY,
        guilds::KEY,
    ]
}

/// Locks every window (click-through) or unlocks them all, and returns what it did.
///
/// One shortcut for all of them on purpose: during a fight nobody wants to lock four windows
/// one by one. Only windows that already exist are touched: asking for a missing one would
/// create it with `visible = true`, and a window the user closed would reappear on every
/// keystroke. Pure, so the rule is testable without a game.
pub fn toggle_locked(config: &mut crate::config::Config) -> bool {
    let keys = window_keys();
    let mut any = false;
    let mut all_locked = true;
    for (key, state) in &config.windows {
        if keys.contains(&key.as_str()) {
            any = true;
            all_locked &= state.locked;
        }
    }
    let lock = any && !all_locked;
    for (key, state) in &mut config.windows {
        if keys.contains(&key.as_str()) {
            state.locked = lock;
        }
    }
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
                draw_status(ui);
                draw_matchup(ui);
                draw_guilds(ui);
                draw_alerts(ui);
                draw_alerts_log(ui);
            }
            // The team picker needs the list of live teams even outside WvW, where polling is
            // off: without this the drop-down was empty (bug of 2026-10-31).
            crate::worker::set_teams_needed(
                gate.settings_available() || crate::config().debug_show_everywhere,
            );
            if gate.settings_available() || crate::config().debug_show_everywhere {
                // Candidates and guilds come from the worker and are copied out before
                // touching the configuration: two mutexes are involved and the dashboard lock
                // must never be held while the configuration one is taken.
                let (candidates, guilds) = {
                    let board = crate::worker::dashboard();
                    (board.candidates.clone(), board.guilds.clone())
                };
                let change = {
                    let mut config = crate::runtime();
                    settings::draw(ui, &mut config, &candidates, &guilds)
                };
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
            }
            crate::persist_if_changed();
        }),
    )
    .revert_on_unload();

    register_render(
        RenderType::OptionsRender,
        render!(|ui| {
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

fn draw_status(ui: &nexus::imgui::Ui) {
    let state = crate::context::snapshot();
    let mut config = crate::runtime();
    let team_id = crate::team::selected_in(&config);

    let poll_secs = config.poll_secs;
    let team_label =
        team_id.map(|id| crate::team::display_name(&config, i18n::effective().code(), id));
    let mut open_settings = false;

    // The worker's view, read under a short lock: nothing here touches the network or disks.
    let (health, match_id, claims, guilds, updated) = {
        let board = crate::worker::dashboard();
        (
            board.health.clone(),
            board.match_id.clone(),
            board.claims.len(),
            board.guilds.len(),
            board.updated_at,
        )
    };

    let window_state = config.window(STATUS);
    window::draw(ui, STATUS, 0, [300.0, 170.0], window_state, |ui| {
        let map_id = state.map_id.to_string();
        let map_type = state.map_type.to_string();
        ui.text(i18n::format("Map {0} (type {1})", &[&map_id, &map_type]));

        match &team_label {
            Some(label) => ui.text(i18n::format("Team {0}", &[label])),
            None => ui.text(i18n::text("No team configured yet")),
        }

        if let Some(id) = &match_id {
            ui.text(i18n::format("Matchup {0}", &[id]));
        }
        match &health {
            crate::worker::Health::Idle => ui.text(i18n::text("No data yet")),
            crate::worker::Health::Ok { .. } => {
                let claims = claims.to_string();
                let guilds = guilds.to_string();
                ui.text(i18n::format("{0} claimed objectives", &[&claims]));
                ui.text(i18n::format("{0} guilds", &[&guilds]));
            }
            crate::worker::Health::Degraded { error, .. } => {
                ui.text(i18n::format("Error: {0}", &[error]));
            }
        }
        if let Some(at) = updated {
            ui.text(i18n::format(
                "Last update {0}",
                &[&at.format("%H:%M:%S").to_string()],
            ));
        }

        ui.separator();
        ui.text(i18n::format(
            "Polling every {0} s",
            &[&poll_secs.to_string()],
        ));
        open_settings = ui.button(i18n::widget("Settings"));
    });

    if open_settings {
        config.window(settings::KEY).visible = true;
    }
}

/// Guilds of the matchup, ordered by the most recent claim, with the history of the chosen one
/// (T-37, T-38). The star follows a guild so it alerts on every map (RF-73).
fn draw_guilds(ui: &nexus::imgui::Ui) {
    let (guilds, names) = {
        let board = crate::worker::dashboard();
        (board.guilds.clone(), board.guild_names.clone())
    };
    let follow = crate::config().followed.clone();
    let current = crate::context::current_map_role();

    let action = {
        let mut config = crate::runtime();
        let state = config.window(guilds::KEY);
        guilds::draw(ui, state, &guilds, &names, &follow, current, |guild| {
            crate::worker::dashboard().guild_history(guild)
        })
    };

    if let Some(guilds::Action::ToggleFollow(guild)) = action {
        let mut config = crate::runtime();
        config.followed.toggle(&guild);
    }
}

/// Captures and claims of the whole matchup (T-36). The worker owns the data; this only draws
/// and keeps the filter boxes of the session.
fn draw_matchup(ui: &nexus::imgui::Ui) {
    static FILTERS: std::sync::OnceLock<std::sync::Mutex<matchup::Filters>> =
        std::sync::OnceLock::new();
    let filters = FILTERS.get_or_init(|| std::sync::Mutex::new(matchup::Filters::all_kinds()));

    let (rows, names) = {
        let board = crate::worker::dashboard();
        (board.claims.clone(), board.guild_names.clone())
    };
    let current = crate::context::current_map_role();
    let follow = crate::config().followed.clone();
    let mut filters = filters.lock().unwrap_or_else(|error| error.into_inner());
    let mut config = crate::runtime();
    let state = config.window(matchup::KEY);
    matchup::draw(ui, state, &mut filters, &rows, current, &follow, &names);
}

/// Recent captures and claims (T-13). The worker resolves and filters the lines, so the render
/// callback only formats what it already has.
fn draw_alerts(ui: &nexus::imgui::Ui) {
    let (filter, follow, popup_secs, opacity) = {
        let mut config = crate::runtime();
        (
            config.alerts,
            config.followed.clone(),
            config.popup_secs,
            config.window(alerts::KEY).opacity,
        )
    };
    let current = crate::context::current_map_role();
    let lines = crate::worker::dashboard().recent_alerts(&filter, &follow, current, 50);
    // The compact widget only keeps what just happened; the log window has the whole session
    // (bug of 2026-10-31: "los avisos deberían desaparecer").
    let widget = alerts::within(&lines, chrono::Utc::now(), alerts::WIDGET_MINUTES);

    let mut config = crate::runtime();
    let state = config.window(alerts::KEY);
    alerts::draw(ui, state, &widget);
    drop(config);

    let showing = alerts::popup_lines(&lines, chrono::Utc::now(), popup_secs);
    alerts::draw_popup(ui, &showing, opacity);
}

/// The same alerts as the widget, complete and in a bigger window (T-13b).
fn draw_alerts_log(ui: &nexus::imgui::Ui) {
    let (filter, follow) = {
        let config = crate::runtime();
        (config.alerts, config.followed.clone())
    };
    let current = crate::context::current_map_role();
    let lines = crate::worker::dashboard().recent_alerts(&filter, &follow, current, 50);

    let mut config = crate::runtime();
    let state = config.window(alerts::LOG_KEY);
    alerts::draw_log(ui, state, &lines);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A configuration with every window already on disk, which is the normal state.
    fn config_with_windows() -> crate::config::Config {
        let mut config = crate::config::Config::default();
        for key in window_keys() {
            config.window(key);
        }
        config
    }

    #[test]
    fn rf18_the_shortcut_locks_and_unlocks_every_window() {
        let mut config = config_with_windows();

        assert!(toggle_locked(&mut config), "la primera pulsación bloquea");
        for key in window_keys() {
            assert!(
                config.window(key).locked,
                "{key} tiene que quedar en click-through, si no un clic se pierde"
            );
        }

        assert!(!toggle_locked(&mut config), "la segunda desbloquea");
        for key in window_keys() {
            assert!(
                !config.window(key).locked,
                "{key} vuelve a aceptar el ratón"
            );
        }
    }

    #[test]
    fn rf18_a_single_locked_window_is_enough_to_lock_them_all() {
        let mut config = config_with_windows();
        config.window(guilds::KEY).locked = true;

        assert!(
            toggle_locked(&mut config),
            "con una bloqueada se bloquean todas, no se desbloquea la suelta"
        );
        for key in window_keys() {
            assert!(config.window(key).locked, "{key}");
        }
    }

    #[test]
    fn rf46_locking_does_not_reopen_a_closed_window() {
        let mut config = config_with_windows();
        config.window(guilds::KEY).visible = false;

        toggle_locked(&mut config);

        assert!(
            !config.window(guilds::KEY).visible,
            "bloquear no puede resucitar una ventana que el usuario cerró"
        );
    }
}
