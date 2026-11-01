//! Windows of the addon. Every one of them goes through [`window::draw`] (RF-46).
//!
//! Every literal comes from the translation table through [`crate::i18n`] (RF-44); the
//! English text is the key, so nothing here needs touching when a language changes.

mod alerts;
mod follow;
mod guilds;
mod icons;
mod map;
mod matchup;
mod popup;
mod settings;
mod status;
mod theme;
mod window;

use nexus::gui::{register_render, render, RenderType};

use std::sync::atomic::{AtomicBool, Ordering};

use crate::i18n;
use gw2lurker_core::model::MapRole;

pub use popup::start_test_popup;

/// Translation key (and stable id) of the only window of the addon (T-42).
///
/// One window with tabs, as in `docs/boceto.html`: alerts, map, guilds, log and — last — the
/// settings, so everything the player needs is in the same place.
pub const WIDGET: &str = "WvW Lurker";

/// Size the widget opens with, before the player resizes it.
pub const WIDGET_DEFAULT_SIZE: [f32; 2] = [470.0, 480.0];

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

/// Shortcut registered by default for bringing the window back.
///
/// **Not** `ALT+SHIFT+L`: that one is the arcDPS legacy window and the game gives it to arcDPS
/// first, so the player pressed our shortcut and got somebody else's menu (reported 2026-10-31).
/// Nexus lists this binding in its own UI, so it can be rebound without touching the addon.
pub const LOCK_SHORTCUT: &str = "CTRL+SHIFT+L";

/// Virtual-key code of `L`, which is what Nexus' keybind API takes.
///
/// The binding is registered **by key code and not by string**: a string the Nexus parser does
/// not like is accepted in silence, which is exactly how "the shortcut does nothing" looked
/// (report of 2026-10-31).
const VK_L: u16 = 0x4C;

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

/// Brings the window back: shown, not click-through, and not collapsed.
///
/// Before this the shortcut only flipped click-through, so with the window hidden — which is how
/// it is left after pressing `×` — nothing visible happened and it looked broken (report of
/// 2026-10-31). Returns the click-through state it had, for the log line.
fn reveal(config: &mut crate::config::Config) -> bool {
    let state = config.window(WIDGET);
    state.visible = true;
    state.collapsed = false;
    let through = state.click_through;
    state.click_through = false;
    through
}

/// Turns click-through on or off, and returns the new state.
///
/// Click-through only: blocking the position is a separate switch that the mouse can always
/// undo (bug of 2026-10-31: both were the same flag, and turning it on took the mouse away).
pub fn toggle_click_through(config: &mut crate::config::Config) -> bool {
    let through = !config.window(WIDGET).click_through;
    config.window(WIDGET).click_through = through;
    through
}

/// Registers the render callbacks. Must be called from `load`, never per frame.
pub fn register() {
    nexus::keybind::register_keybind_with_struct(
        LOCK_KEYBIND,
        nexus::keybind::keybind_handler!(on_lock_keybind),
        nexus::keybind::Keybind {
            key: VK_L,
            alt: false,
            ctrl: true,
            shift: true,
        },
    )
    .revert_on_unload();
    log::info!("atajo registrado: {LOCK_SHORTCUT} ({LOCK_KEYBIND}); Nexus dirá qué queda guardado");

    register_render(
        RenderType::Render,
        render!(|ui| {
            i18n::register_translations_once();
            let gate = crate::context::gate();
            crate::context::log_gate(gate);
            crate::worker::set_may_poll(gate.poll);

            if take_lock_toggle() {
                let mut config = crate::runtime();
                let had_click_through = reveal(&mut config);
                crate::request_save(&config);
                log::info!(
                    "atajo: ventana traída al frente (click-through era {had_click_through})"
                );
            }

            // Con click-through la ventana no recibe nada del ratón, así que la única salida era
            // el atajo o el panel de Nexus. El clic derecho sobre el widget lo desactiva
            // (petición de 2026-10-31): se mira el ratón desde aquí, que sí lo vemos.
            if window::right_click_on(WIDGET, WIDGET_DEFAULT_SIZE, ui) {
                let mut config = crate::runtime();
                let through = toggle_click_through(&mut config);
                crate::request_save(&config);
                log::info!("clic derecho en el widget: click-through {through}");
            }

            if gate.show_ui || crate::config().debug_show_everywhere {
                // A panic in a tab must not take the game down with it: `catch_unwind` keeps the
                // overlay alive and leaves the reason in the log (RNF-06). Clicking the *Map* tab
                // crashed the game once (reported 2026-10-31).
                guard("widget", || draw_widget(ui));
            }
            // El selector de equipo y el buscador de clanes encienden `set_teams_needed` mientras
            // se dibujan (y el worker lo recuerda unos segundos): antes se pedía «por si acaso» en
            // cada sondeo —`gate.settings_available()` es cierto casi siempre—, lo que a 5 s
            // serían 4,8 MB por minuto.
            crate::persist_if_changed();
        }),
    )
    .revert_on_unload();

    register_render(
        RenderType::OptionsRender,
        render!(|ui| {
            guard("options", || draw_options(ui));
        }),
    )
    .revert_on_unload();
}

/// Nexus' options pane: the two openers, then the whole form (T-42).
///
/// It draws even when the overlay is hidden or click-through, so it is the way back to
/// everything that the mouse cannot reach (RF-02).
fn draw_options(ui: &nexus::imgui::Ui) {
    draw_openers(ui);
    let (candidates, worlds) = {
        let board = crate::worker::dashboard();
        (board.candidates.clone(), board.world_names.clone())
    };
    let change = settings::draw_inline(ui, &candidates, &worlds);
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

/// Runs one render callback, turning a panic into a log line instead of a dead game.
///
/// A panic that crosses the `cdylib` boundary aborts the process, so one wrong tab would take
/// Guild Wars 2 with it. This keeps the overlay alive and, after the first panic, stops drawing
/// that surface for the rest of the session: repeating the same panic every frame is noise.
fn guard(name: &'static str, draw: impl FnOnce()) {
    static BROKEN_WIDGET: AtomicBool = AtomicBool::new(false);
    static BROKEN_OPTIONS: AtomicBool = AtomicBool::new(false);
    let broken = if name == "widget" {
        &BROKEN_WIDGET
    } else {
        &BROKEN_OPTIONS
    };
    if broken.load(Ordering::Relaxed) {
        return;
    }
    if std::panic::catch_unwind(std::panic::AssertUnwindSafe(draw)).is_err() {
        broken.store(true, Ordering::Relaxed);
        log::error!("ui: pánico dibujando «{name}»; se deja de dibujar esa parte esta sesión");
    }
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
    // De la copia editable, no de la del arranque: un clan que el jugador acaba de añadir o de
    // dejar de seguir tiene que verse en la pestaña y en los filtros en el mismo momento
    // (informe de 2026-10-31, «pulsar x no lo borra»).
    let (alerts_cfg, follow) = {
        let config = crate::runtime();
        (config.alerts, config.followed.clone())
    };
    let (candidates, guilds, names, claims, states, alerts, objectives, maps, worlds) = {
        let board = crate::worker::dashboard();
        (
            board.candidates.clone(),
            board.guilds.clone(),
            board.guild_names.clone(),
            board.claims.clone(),
            board.states.clone(),
            board.recent_alerts(&alerts_cfg, &follow, current, 50),
            board.names.clone(),
            board.maps.clone(),
            board.world_names.clone(),
        )
    };
    let now = chrono::Utc::now();

    let mut config = crate::runtime();
    let mut state = config.window(WIDGET).clone();
    let wanted_visible = state.visible;
    let wanted_locked = state.locked;
    let wanted_through = state.click_through;
    let mut change = None;
    let mut filters = filters.lock().unwrap_or_else(|error| error.into_inner());

    let still_open = window::draw(ui, WIDGET, 0, WIDGET_DEFAULT_SIZE, &mut state, |ui| {
        change = draw_tabs(
            ui,
            &mut config,
            &mut filters,
            &Tabbed {
                candidates: &candidates,
                guilds: &guilds,
                names: &names,
                claims: &claims,
                states: &states,
                alerts: &alerts,
                follow: &follow,
                current,
                now,
                objectives: &objectives,
                maps: &maps,
                worlds: &worlds,
            },
        );
    });
    drop(filters);

    // The settings tab can hide or lock the widget in the same frame the chrome runs, so the
    // copy is written back field by field: the chrome only wins on the fields nobody else
    // touched this frame.
    let slot = config.window(WIDGET);
    if window::chrome_wins(slot.visible, wanted_visible) {
        slot.visible = still_open && state.visible;
    }
    if window::chrome_wins(slot.locked, wanted_locked) {
        slot.locked = state.locked;
    }
    // Sin esta línea el clic derecho del menú «Click-through» no hacía nada: cambiaba la copia
    // que se tira al final del fotograma (informe de 2026-10-31).
    if window::chrome_wins(slot.click_through, wanted_through) {
        slot.click_through = state.click_through;
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
    popup::draw(ui, &alerts, now);
}

/// Everything the tabs draw, gathered once per frame.
pub(super) struct Tabbed<'a> {
    candidates: &'a [gw2lurker_core::Candidate],
    guilds: &'a [gw2lurker_core::matchup::GuildSummary],
    names: &'a std::collections::BTreeMap<String, String>,
    claims: &'a [gw2lurker_core::matchup::ClaimRow],
    /// Dueño y claim de **cada** objetivo del match (RF-30b): el mapa no puede colorear sólo los
    /// reclamados.
    states: &'a [gw2lurker_core::matchup::ObjectiveState],
    alerts: &'a [crate::worker::AlertLine],
    follow: &'a gw2lurker_core::FollowList,
    current: Option<MapRole>,
    now: chrono::DateTime<chrono::Utc>,
    /// Objectives of the player's language, with the `coord` the map needs.
    objectives: &'a [gw2lurker_core::ObjectiveInfo],
    /// Boxes of the four WvW maps (RF-30).
    maps: &'a [gw2lurker_core::WvwMap],
    /// Legacy world id → name, para que el selector de equipo se pueda reconocer (RF-03a).
    worlds: &'a std::collections::BTreeMap<u32, String>,
}

/// Draws the tab bar and the body of the tab the player picked. The last tab is the settings.
fn draw_tabs(
    ui: &nexus::imgui::Ui,
    config: &mut crate::config::Config,
    filters: &mut matchup::Filters,
    data: &Tabbed,
) -> Option<settings::TeamChange> {
    use nexus::imgui::{TabBar, TabBarFlags, TabItem, TabItemFlags};

    let mut change = None;
    // Clan clicado en un aviso o en la lista de seguidos: se selecciona para la ruta.
    let mut clicked: Option<String> = None;

    // RF-51: a broken settings file says so, in red, above everything else.
    if let Some(report) = crate::safe_mode() {
        let _tint = ui.push_style_color(nexus::imgui::StyleColor::Text, [1.0, 0.45, 0.45, 1.0]);
        ui.text_wrapped(i18n::format(
            "Settings could not be read ({0}); running with defaults.",
            &[&report],
        ));
    }

    // The status of the worker, above the tabs: it is what tells a bug from a quiet matchup.
    status::draw_health(ui, config, data);
    ui.separator();

    // Scroll instead of shrinking: with six tabs in a narrow window imgui used to squeeze them
    // until the last ones were unreachable. With the scroll policy it draws the little arrows in
    // the bar, which is the «...» that was asked for (report of 2026-10-31).
    let jump = WANT_TAB.load(Ordering::Relaxed);
    if let Some(_bar) = TabBar::new("###gw2lurker_tabs")
        .flags(TabBarFlags::FITTING_POLICY_SCROLL)
        .begin(ui)
    {
        for (index, key) in TABS.iter().enumerate() {
            let mut item = TabItem::new(i18n::widget(key));
            if jump == index {
                item = item.flags(TabItemFlags::SET_SELECTED);
            }
            item.build(ui, || match index {
                // Todas las ramas devuelven el clan que se ha clicado, si lo hay: con él se
                // selecciona la ruta (petición de 2026-10-31).
                0 => {
                    clicked = alerts::body(ui, data.alerts, data.now);
                }
                1 => {
                    let selected = guilds::selected_guild();
                    let history = selected
                        .as_deref()
                        .map(|guild| crate::worker::dashboard().guild_history(guild))
                        .unwrap_or_default();
                    let session = crate::worker::dashboard().session_guilds();
                    map::body(
                        ui,
                        &map::View {
                            states: data.states,
                            names: data.objectives,
                            maps: data.maps,
                            follow: data.follow,
                            guild_names: data.names,
                            opacity: config.window(WIDGET).opacity,
                            real_map: config.map_tiles,
                            history: &history,
                            guild: selected.as_deref(),
                            session_guilds: &session,
                        },
                        &mut map_choice(),
                        data.current,
                    );
                    if let Some(guild) = selected.as_deref() {
                        // ADR-005: la ruta es una deducción y se dice siempre; RF-35: si sigue en
                        // otro mapa, el resumen lo anuncia. Va aquí, junto al dibujo, porque la
                        // pestaña *Clanes* ya no la escribe (petición del jugador, 2026-11-01).
                        if let Some(summary) = map::summary(&history, guild, i18n::effective()) {
                            ui.text_disabled(i18n::text(
                                "Route inferred from claims; it is not the real path.",
                            ));
                            ui.text(summary);
                        }
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
                    // T-44: seguir un clan y añadirlo se decide aquí, mirando la partida, no en
                    // los ajustes (petición de 2026-10-31).
                    let team = crate::team::selected_in(config);
                    let session = crate::worker::dashboard().session_guilds();
                    clicked = follow::followed_section(
                        ui,
                        &mut config.followed,
                        data.names,
                        &mut config.guild_names,
                    );
                    follow::add_section(
                        ui,
                        &mut config.followed,
                        data.names,
                        &mut config.guild_names,
                        data.candidates,
                        team,
                        &session,
                    );
                    ui.separator();
                    match guilds::body(
                        ui,
                        data.guilds,
                        data.names,
                        &mut config.guild_labels,
                        data.follow,
                        data.current,
                        |guild| crate::worker::dashboard().guild_history(guild),
                    ) {
                        Some(guilds::Action::Follow(guild)) => follow::add_followed(
                            &mut config.followed,
                            data.names,
                            &mut config.guild_names,
                            &guild,
                        ),
                        // `i`: la ruta se dibuja en el mapa, con el resto de la inferencia.
                        Some(guilds::Action::ShowOnMap(guild)) => clicked = Some(guild),
                        None => {}
                    }
                }
                3 => {
                    clicked = alerts::log_body(ui, data.alerts);
                }
                _ => {
                    change = settings::body(ui, config, data.candidates, data.worlds);
                }
            });
        }
    }
    // El cambio de pestaña dura un fotograma: al siguiente manda el jugador.
    if jump != usize::MAX {
        WANT_TAB.store(usize::MAX, Ordering::Relaxed);
    }

    // Un clan clicado selecciona su ruta y lleva el mapa al último donde reclamó algo, que es lo
    // que se está pidiendo al clicar (petición de 2026-10-31). Además se cambia a la pestaña
    // *Mapa*, donde está esa ruta: antes había que ir a mano y parecía que el clic no hacía nada.
    if let Some(guild) = clicked {
        guilds::select(&guild);
        if let Some(role) = crate::worker::dashboard().last_map_of(&guild) {
            *map_choice() = role;
        }
        WANT_TAB.store(TAB_MAP, Ordering::Relaxed);
    }
    change
}

/// Tab index of the *Map* tab inside [`TABS`].
const TAB_MAP: usize = 1;

/// Tab the widget has to switch to on this frame, `usize::MAX` for «stay where the player is».
///
/// Session state, like the map choice: it exists for the frame after a click on a guild.
static WANT_TAB: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(usize::MAX);

/// Which map the *Map* tab draws: the session's choice, `Unknown` until the player picks one.
///
/// Session state on purpose: the default is always the map the player is on, which is the one
/// they care about.
fn map_choice() -> std::sync::MutexGuard<'static, MapRole> {
    static CHOICE: std::sync::OnceLock<std::sync::Mutex<MapRole>> = std::sync::OnceLock::new();
    CHOICE
        .get_or_init(|| std::sync::Mutex::new(MapRole::Unknown))
        .lock()
        .unwrap_or_else(|error| error.into_inner())
}

/// One line with the state of the worker and the configured team (RF-02, RF-46).
fn draw_openers(ui: &nexus::imgui::Ui) {
    let mut config = crate::runtime();
    let mut visible = config.window(WIDGET).visible;
    let mut locked = config.window(WIDGET).click_through;
    let shown = ui.checkbox(i18n::widget("Show widget"), &mut visible);
    let clicked = ui.checkbox(i18n::widget("Click-through"), &mut locked);
    let popups = ui.checkbox(i18n::widget("Pop-up alerts"), &mut config.popup_enabled);
    if shown {
        config.window(WIDGET).visible = visible;
    }
    if clicked {
        config.window(WIDGET).click_through = locked;
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
    fn rf18_the_shortcut_turns_click_through_on_and_off() {
        let mut config = crate::config::Config::default();

        assert!(
            toggle_click_through(&mut config),
            "la primera pulsación activa"
        );
        assert!(
            config.window(WIDGET).click_through,
            "click-through: los clics se van al juego, por eso el atajo lo enciende sin más"
        );
        assert!(
            !config.window(WIDGET).locked,
            "click-through no es bloquear: la posición se queda como estaba (bug de 2026-10-31)"
        );

        assert!(!toggle_click_through(&mut config), "la segunda desactiva");
        assert!(!config.window(WIDGET).click_through);
    }

    #[test]
    fn rf18_the_shortcut_brings_the_window_back() {
        // El bug de 2026-10-31: el atajo solo alternaba click-through, así que con la ventana
        // cerrada (que es como queda al pulsar `×`) no pasaba nada visible y parecía roto.
        let mut config = crate::config::Config::default();
        let state = config.window(WIDGET);
        state.visible = false;
        state.collapsed = true;
        state.click_through = true;

        let had = reveal(&mut config);

        let state = config.window(WIDGET);
        assert!(had, "dice de dónde venía, para el registro");
        assert!(state.visible, "la ventana vuelve");
        assert!(!state.collapsed, "y desplegada");
        assert!(!state.click_through, "y con ratón");
    }

    #[test]
    fn rf46_click_through_does_not_reopen_a_closed_widget() {
        let mut config = crate::config::Config::default();
        config.window(WIDGET).visible = false;

        toggle_click_through(&mut config);

        assert!(
            !config.window(WIDGET).visible,
            "cambiar click-through no puede resucitar una ventana que el usuario cerró"
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
