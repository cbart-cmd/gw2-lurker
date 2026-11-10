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
mod navigation;
mod popup;
mod popup_stack;
mod settings;
mod status;
mod team_picker;
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

/// ImGui allows only one active context, so native UI tests share this lock.
#[cfg(test)]
pub(crate) static IMGUI_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

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
            // Ventana propia del selector (RF-03j): se abre sola al entrar en MvM o en el
            // Bastión si falta equipo o cambió el emparejamiento, y desde Ajustes a mano.
            guard("picker", || team_picker::draw(ui, gate));
            // El selector de equipo y el buscador de clanes encienden `set_teams_needed` mientras
            // se dibujan (y el worker lo recuerda unos segundos): antes se pedía «por si acaso» en
            // cada sondeo —`gate.settings_available()` es cierto casi siempre—, lo que a 5 s
            // serían 4,8 MB por minuto.
            map::pause_unless_drawn(ui.frame_count());
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
    settings::draw_inline(ui);
}

/// Runs one render callback, turning a panic into a log line instead of a dead game.
///
/// A panic that crosses the `cdylib` boundary aborts the process, so one wrong tab would take
/// Guild Wars 2 with it. This keeps the overlay alive and, after the first panic, stops drawing
/// that surface for the rest of the session: repeating the same panic every frame is noise.
fn guard(name: &'static str, draw: impl FnOnce()) {
    static BROKEN_WIDGET: AtomicBool = AtomicBool::new(false);
    static BROKEN_PICKER: AtomicBool = AtomicBool::new(false);
    static BROKEN_OPTIONS: AtomicBool = AtomicBool::new(false);
    let broken = match name {
        "widget" => &BROKEN_WIDGET,
        "picker" => &BROKEN_PICKER,
        _ => &BROKEN_OPTIONS,
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

    let (visible, popup_enabled, alerts_cfg, follow, selected_team) = {
        let mut config = crate::runtime();
        (
            config.window(WIDGET).visible,
            config.popup_enabled,
            config.alerts,
            config.followed.clone(),
            crate::team::selected_in(&config),
        )
    };
    if !visible && !popup_enabled {
        return;
    }

    let filters = FILTERS.get_or_init(|| std::sync::Mutex::new(matchup::Filters::all_kinds()));
    let game = crate::context::snapshot();
    let current_map_id = game.map_id;
    let current = crate::context::map_role(game);
    let now = chrono::Utc::now();
    if !visible {
        let alerts = {
            let board = crate::worker::dashboard();
            board.recent_alerts(&alerts_cfg, &follow, current, 50)
        };
        popup::draw(ui, &alerts, now);
        return;
    }

    let (
        candidates,
        guilds,
        names,
        claims,
        states,
        alerts,
        objectives,
        maps,
        history_since,
        played_team,
    ) = {
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
            board.history_since,
            board.played_team,
        )
    };
    let player_team = played_team
        .or(selected_team)
        .and_then(|id| candidates.iter().find(|candidate| candidate.team_id == id))
        .map(|candidate| candidate.color);

    let mut config = crate::runtime();
    let mut state = config.window(WIDGET).clone();
    let wanted_visible = state.visible;
    let wanted_locked = state.locked;
    let wanted_through = state.click_through;
    let mut filters = filters.lock().unwrap_or_else(|error| error.into_inner());

    let still_open = window::draw(
        ui,
        WIDGET,
        0,
        WIDGET_DEFAULT_SIZE,
        &mut state,
        window::WindowAppearance {
            theme: config.theme,
            font_scale: config.font_scale,
            background: config.widget_background,
        },
        |ui| {
            draw_tabs(
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
                    current_map_id,
                    now,
                    team_id: selected_team,
                    player_team,
                    history_since,
                    objectives: &objectives,
                    maps: &maps,
                },
            );
        },
    );
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
    current_map_id: u32,
    now: chrono::DateTime<chrono::Utc>,
    team_id: Option<u32>,
    player_team: Option<gw2lurker_core::model::TeamColor>,
    history_since: Option<chrono::DateTime<chrono::Utc>>,
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
) {
    use nexus::imgui::{TabBar, TabBarFlags, TabItem, TabItemFlags};

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
    let team = data.team_id;
    status::draw_health(ui, config, data, team);
    if team.is_some() {
        draw_map_notice(ui, data);
    }
    ui.separator();

    // Scroll instead of shrinking: with six tabs in a narrow window imgui used to squeeze them
    // until the last ones were unreachable. With the scroll policy it draws the little arrows in
    // the bar, which is the «...» that was asked for (report of 2026-10-31).
    let jump = WANT_TAB.load(Ordering::Relaxed);
    // Fotograma del widget: un clic de clan solo manda en el fotograma siguiente, sin dejar una
    // marca viva que impida saltar al mapa del jugador al volver a *Mapa* a mano.
    let frame = FRAME.fetch_add(1, Ordering::Relaxed) + 1;
    // Pestaña que se dibuja este fotograma: imgui solo ejecuta el contenido de la elegida.
    let mut shown = usize::MAX;
    if let Some(_bar) = TabBar::new("###gw2lurker_tabs")
        .flags(TabBarFlags::FITTING_POLICY_SCROLL)
        .begin(ui)
    {
        for (index, key) in TABS.iter().enumerate() {
            let mut item = TabItem::new(i18n::widget(key));
            if jump == index {
                item = item.flags(TabItemFlags::SET_SELECTED);
            }
            item.build(ui, || {
                shown = index;
                if !navigation::tab_available(key, team) {
                    ui.text_wrapped(i18n::text("No data until you choose a team in Settings."));
                    return;
                }
                match index {
                    // Todas las ramas devuelven el clan que se ha clicado, si lo hay: con él se
                    // selecciona la ruta (petición de 2026-10-31).
                    0 => {
                        clicked = alerts::body(
                            ui,
                            data.alerts,
                            &alerts::LineContext {
                                now: data.now,
                                player_team: data.player_team,
                                states: data.states,
                            },
                        );
                    }
                    1 => {
                        let jump_at = GUILD_JUMP_AT.load(Ordering::Relaxed);
                        let entering = navigation::entering_by_hand(
                            index,
                            LAST_TAB.load(Ordering::Relaxed),
                            (jump_at != u64::MAX).then_some(jump_at),
                            frame,
                        );
                        if entering {
                            if let Some(here) = data.current {
                                *map_choice() = here;
                            }
                            // Entrar a mano en *Mapa* deja el desplegable de clan vacío: se ve
                            // todo lo que reclaman los clanes que sigues, sin ruta de nadie.
                            navigation::set_guild_choice(None);
                        }
                        let selected = navigation::guild_choice();
                        let history = selected
                            .as_deref()
                            .map(|guild| crate::worker::dashboard().guild_history(guild))
                            .unwrap_or_default();
                        let session = crate::worker::dashboard().session_guilds();
                        let (revision, prepared) = {
                            let board = crate::worker::dashboard();
                            (
                                (board.updated_at, board.history_since),
                                selected
                                    .as_deref()
                                    .and_then(|guild| board.route_models.get(guild))
                                    .cloned(),
                            )
                        };
                        clicked = map::body(
                            ui,
                            &map::View {
                                states: data.states,
                                names: data.objectives,
                                maps: data.maps,
                                current_map_id: data.current_map_id,
                                follow: data.follow,
                                guild_names: data.names,
                                opacity: config.window(WIDGET).opacity,
                                real_map: config.map_tiles,
                                guild: selected.as_deref(),
                                session_guilds: &session,
                                revision,
                                prepared: prepared.as_ref(),
                            },
                            &mut map_choice(),
                            data.current,
                        );
                        if let Some(guild) = selected.as_deref() {
                            // ADR-005: la ruta es una deducción y se dice siempre; RF-35: si sigue en
                            // otro mapa, el resumen lo anuncia. Va aquí, junto al dibujo, porque la
                            // pestaña *Clanes* ya no la escribe (petición del jugador, 2026-11-01).
                            if let Some(summary) = map::summary(&history, guild, i18n::effective())
                            {
                                ui.text_wrapped(summary);
                            }
                        }
                        // The list is the same data as the map, one click away: during a fight the
                        // text is faster to read than the picture.
                        ui.separator();
                        if nexus::imgui::CollapsingHeader::new(i18n::widget("Matchup claims"))
                            .build(ui)
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
                        match follow::followed_section(
                            ui,
                            &mut config.followed,
                            data.names,
                            &mut config.guild_names,
                        ) {
                            Some(follow::Action::Select(guild)) => guilds::select(&guild),
                            Some(follow::Action::ShowOnMap(guild)) => clicked = Some(guild),
                            None => {}
                        }
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
                            guilds::Context {
                                current: data.current,
                                history_since: data.history_since,
                            },
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
                        clicked = alerts::log_body(
                            ui,
                            data.alerts,
                            &alerts::LineContext {
                                now: data.now,
                                player_team: data.player_team,
                                states: data.states,
                            },
                        );
                    }
                    _ => {
                        settings::body(ui, config);
                    }
                }
            });
        }
    }
    // SET_SELECTED can take effect on a later frame; keep the request until its tab is drawn.
    WANT_TAB.store(
        navigation::pending_tab_after_draw(shown, jump),
        Ordering::Relaxed,
    );

    // Un clan clicado selecciona su ruta y lleva el mapa al último donde reclamó algo, que es lo
    // que se está pidiendo al clicar (petición de 2026-10-31). Además se cambia a la pestaña
    // *Mapa*, donde está esa ruta: antes había que ir a mano y parecía que el clic no hacía nada.
    if let Some(guild) = clicked {
        let target = data
            .guilds
            .iter()
            .find(|summary| summary.guild == guild)
            .and_then(|summary| summary.last_map.map(|(_, role)| role))
            .filter(|role| *role != MapRole::Unknown)
            .or_else(|| crate::worker::dashboard().last_map_of(&guild));
        let mut requested = WANT_TAB.load(Ordering::Relaxed);
        if navigation::request_guild_map(
            &guild,
            target,
            &mut map_choice(),
            &mut requested,
            &mut map_notice(),
        ) {
            guilds::select(&guild);
            navigation::set_guild_choice(Some(&guild));
            GUILD_JUMP_AT.store(frame, Ordering::Relaxed);
        }
        WANT_TAB.store(requested, Ordering::Relaxed);
    }

    LAST_TAB.store(shown, Ordering::Relaxed);
}

/// Tab index of the *Map* tab inside [`TABS`].
const TAB_MAP: usize = 1;

fn map_notice() -> std::sync::MutexGuard<'static, Option<String>> {
    static NOTICE: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);
    NOTICE.lock().unwrap_or_else(|error| error.into_inner())
}

fn draw_map_notice(ui: &nexus::imgui::Ui, data: &Tabbed) {
    let mut notice = map_notice();
    let Some(guild) = notice.as_deref() else {
        return;
    };
    let name = data.names.get(guild).map(String::as_str).unwrap_or(guild);
    {
        let _warning = ui.push_style_color(nexus::imgui::StyleColor::Text, [1.0, 0.75, 0.3, 1.0]);
        ui.text_wrapped(i18n::format(
            "No events recorded for {0} since session start or the last team reassignment.",
            &[name],
        ));
    }
    if let Some(since) = data.history_since {
        ui.text(i18n::format(
            "Tracking since {0}",
            &[&crate::clock::clock(since)],
        ));
    }
    if ui.small_button(i18n::widget("Dismiss")) {
        *notice = None;
    }
}

/// Tab drawn on the previous frame, `usize::MAX` for «none». Session state, like the map choice.
static LAST_TAB: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(usize::MAX);

/// Fotograma actual del widget. Sirve para que un clic de clan mande solo en el fotograma
/// siguiente y no deje una marca viva que se coma los saltos posteriores al mapa del jugador.
static FRAME: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Fotograma del último clic de clan que abrió la pestaña *Mapa*, `u64::MAX` si no hubo.
static GUILD_JUMP_AT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(u64::MAX);

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
    fn rf30_the_map_tab_opens_on_the_map_i_am_on() {
        // Petición del jugador (2026-11-01): al entrar en *Mapa* desde otra pestaña se abre el mapa
        // en el que está, que es el que le importa; pero si llega por clicar un clan, manda el mapa
        // de ese clan (RF-32a). La regla vive en `navigation::entering_by_hand`.
        assert!(
            navigation::entering_by_hand(TAB_MAP, 0, None, 5),
            "entrar desde Alertas tiene que saltar al mapa del jugador"
        );
        assert!(
            navigation::entering_by_hand(TAB_MAP, 4, None, 5),
            "entrar desde Ajustes también"
        );
        assert!(
            navigation::entering_by_hand(TAB_MAP, usize::MAX, None, 5),
            "al abrir el widget por primera vez, el mapa del jugador"
        );
        assert!(
            !navigation::entering_by_hand(TAB_MAP, TAB_MAP, None, 5),
            "quedarse en la pestaña no cambia el mapa elegido a mano"
        );
        assert!(
            !navigation::entering_by_hand(3, 0, None, 5),
            "solo la pestaña Mapa"
        );
        assert!(
            !navigation::entering_by_hand(TAB_MAP, 2, Some(4), 5),
            "un clic de clan abre el mapa del clan, no el del jugador"
        );
    }

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
