//! The team picker, a window of its own (RF-03a, RF-03j).
//!
//! It asks in three steps — **region → colour → team** — because that is how a player recognises
//! their side: the in-game WvW panel shows the three colours of the matchup. With thirty-three
//! teams in the API, going straight to a name list is guesswork; with region and colour it is four
//! or five rows (RF-03a).
//!
//! It used to live inside *Settings*; the player asked for it to be a window of its own, centred
//! on screen, that opens **by itself** on entering a WvW map or Armistice Bastion when there is no
//! team yet, when the saved team no longer plays or when the matchup changed, and that the
//! *Settings* button opens too (2026-11-02). The decision of when is `gw2lurker_core::prompt`.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use gw2lurker_core::model::{Region, TeamColor};
use gw2lurker_core::prompt::{self, Prompt};
use gw2lurker_core::{select, Candidate, Gate, Selection, TeamTable};
use nexus::imgui::{ComboBox, Condition, Selectable, StyleColor, Ui, Window};

use crate::i18n;

/// Translation key (and stable id) of the window.
pub const KEY: &str = "Choose your team";

/// Width the window opens with; it grows with its content.
const WIDTH: f32 = 420.0;

/// Wait after entering a map before deciding: arcdps says which **account** we are in a moment
/// after the map loads, and until then the team is looked up by character and is not found, which
/// would open the picker for a player who already chose (RF-03a, memory per account).
const GRACE: Duration = Duration::from_secs(5);

/// A change to the team selection, decided while drawing and applied once the lock is released.
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

/// Whether the window is open and why it opened.
#[derive(Debug, Clone, Copy, Default)]
struct Picker {
    open: bool,
    reason: Option<Prompt>,
    narrowing: Narrowing,
}

/// The map the player entered last, and whether that entry was already checked.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Entry {
    map_id: u32,
    since: Option<Instant>,
    settled: bool,
}

impl Entry {
    /// Whether this entry still has to be checked. A new map re-arms it; only maps where the
    /// window may open (WvW and Armistice Bastion) count, and only after [`GRACE`].
    fn due(&mut self, map_id: u32, in_wvw: bool, now: Instant) -> bool {
        if map_id != self.map_id {
            *self = Self {
                map_id,
                since: in_wvw.then_some(now),
                settled: false,
            };
        }
        in_wvw
            && !self.settled
            && self
                .since
                .is_some_and(|at| now.saturating_duration_since(at) >= GRACE)
    }
}

/// What to do with one entry once its grace is over.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Step {
    Open(Prompt),
    /// Nothing was ever confirmed for this team (an upgrade or a fresh pick): this matchup is the
    /// reference from now on, without asking.
    Record(String),
    /// The team has no data yet: look again on the next frame.
    Wait,
    Done,
}

fn step(
    selected: Option<u32>,
    selection: &Selection,
    current: Option<&str>,
    confirmed: Option<&str>,
) -> Step {
    if let Some(reason) = prompt::prompt(selected, selection, current, confirmed) {
        return Step::Open(reason);
    }
    match (selected, current, confirmed) {
        (Some(_), Some(now), None) => Step::Record(now.to_owned()),
        (Some(_), None, _) => Step::Wait,
        _ => Step::Done,
    }
}

static PICKER: Mutex<Picker> = Mutex::new(Picker {
    open: false,
    reason: None,
    narrowing: Narrowing {
        region: None,
        colour: None,
    },
});

static ENTRY: Mutex<Entry> = Mutex::new(Entry {
    map_id: 0,
    since: None,
    settled: false,
});

fn picker() -> std::sync::MutexGuard<'static, Picker> {
    PICKER.lock().unwrap_or_else(|error| error.into_inner())
}

/// Opens the window. `reason` is `None` when the player asked for it (the *Settings* button).
pub fn open(reason: Option<Prompt>) {
    let mut state = picker();
    state.open = true;
    state.reason = reason;
}

/// Opens the window by itself when entering WvW or Armistice Bastion has something to ask.
///
/// Takes the configuration and the dashboard **one after the other**, never both, and only while
/// the entry is still pending: once decided it costs one lock per frame.
fn auto_open(gate: Gate) {
    let map_id = crate::context::snapshot().map_id;
    let due = ENTRY.lock().unwrap_or_else(|error| error.into_inner()).due(
        map_id,
        gate.show_ui,
        Instant::now(),
    );
    if !due {
        return;
    }
    let (selected, confirmed) = {
        let config = crate::runtime();
        let selected = crate::team::selected_in(&config);
        let confirmed = selected
            .and_then(|team| config.confirmed_matchups.get(&team.to_string()))
            .cloned();
        (selected, confirmed)
    };
    let (selection, current) = {
        let board = crate::worker::dashboard();
        let current = selected.and_then(|team| prompt::matchup_key(&board.candidates, team));
        (board.selection.clone(), current)
    };
    let decided = step(
        selected,
        &selection,
        current.as_deref(),
        confirmed.as_deref(),
    );
    match &decided {
        Step::Open(reason) => {
            log::info!("selector de equipo: se abre solo ({reason:?})");
            open(Some(*reason));
        }
        Step::Record(key) => {
            if let Some(team) = selected {
                crate::runtime()
                    .confirmed_matchups
                    .insert(team.to_string(), key.clone());
            }
        }
        // Armistice Bastion does not poll: the list of teams has to be asked for (one fetch every
        // ten minutes at most, `worker::TEAMS_TTL`).
        Step::Wait if !gate.poll => crate::worker::set_teams_needed(true),
        Step::Wait | Step::Done => {}
    }
    if !matches!(decided, Step::Wait) {
        ENTRY
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .settled = true;
    }
}

/// Draws the window when it is open, after deciding whether it has to open by itself.
pub fn draw(ui: &Ui, gate: Gate) {
    auto_open(gate);
    let state = *picker();
    if !state.open || !gate.settings_available() {
        return;
    }
    // Mientras el selector está en pantalla hacen falta los nueve emparejamientos: sin esto la
    // lista de equipos se queda como esté (bug de 2026-10-31, el desplegable salía vacío).
    crate::worker::set_teams_needed(true);
    let (candidates, worlds) = {
        let board = crate::worker::dashboard();
        (board.candidates.clone(), board.world_names.clone())
    };

    let mut config = crate::runtime();
    let table = gw2lurker_core::bundled();
    let language = i18n::effective().code();
    let selected = crate::team::selected_in(&config);
    let mut picked = selected;
    let mut narrowing = initial_narrowing(state.narrowing, &candidates, selected);
    let mut still_open = true;
    let mut confirmed = false;
    let [width, height] = ui.io().display_size;

    let _style = super::theme::apply(ui, config.theme);
    Window::new(i18n::widget(KEY))
        .position([width * 0.5, height * 0.5], Condition::Appearing)
        .position_pivot([0.5, 0.5])
        .size([WIDTH, 0.0], Condition::Appearing)
        .always_auto_resize(true)
        .collapsible(false)
        .focus_on_appearing(true)
        .opened(&mut still_open)
        .build(ui, || {
            if config.font_scale > 0.0 {
                ui.set_window_font_scale(config.font_scale.clamp(0.8, 1.6));
            }
            draw_reason(ui, state.reason);
            let _width = ui.push_item_width(WIDTH - 140.0);
            if candidates.is_empty() {
                ui.text_disabled(i18n::text(
                    "The list of teams arrives with the first poll; wait a few seconds.",
                ));
            }
            draw_region(ui, &mut narrowing, &candidates);
            draw_colour(ui, &mut narrowing);
            draw_team(
                ui,
                table,
                &config,
                language,
                &narrowing,
                &mut picked,
                &candidates,
                &worlds,
            );
            if let Some(team) = picked {
                draw_rivals(ui, &config, language, team, &candidates);
                ui.separator();
                if ui.button(i18n::widget("Confirm team")) {
                    confirmed = true;
                }
            }
        });

    let change = team_change(selected, picked);
    if let Some(TeamChange::Pick(team)) = change {
        // Elegido ahora: el emparejamiento de este equipo se toma como referencia en la próxima
        // entrada, sin volver a preguntar por él.
        config.confirmed_matchups.remove(&team.to_string());
    }
    if confirmed {
        if let Some(team) = picked {
            if let Some(key) = prompt::matchup_key(&candidates, team) {
                config.confirmed_matchups.insert(team.to_string(), key);
            }
        }
    }
    drop(config);
    match change {
        Some(TeamChange::Pick(team)) => crate::team::remember(team),
        Some(TeamChange::Clear) => crate::team::forget(),
        None => {}
    }

    let mut slot = picker();
    slot.narrowing = narrowing;
    if !still_open || confirmed {
        slot.open = false;
        slot.reason = None;
    }
}

/// Why the window opened by itself, in the colour of a warning.
fn draw_reason(ui: &Ui, reason: Option<Prompt>) {
    let text = match reason {
        Some(Prompt::NoTeam) => "Pick your team to see what happens in your matchup.",
        Some(Prompt::Stale) => "Your saved team is not playing right now: pick it again.",
        Some(Prompt::NewMatchup) => "The matchup has changed: confirm your team or pick another.",
        None => return,
    };
    let _tint = ui.push_style_color(StyleColor::Text, [1.0, 0.75, 0.3, 1.0]);
    ui.text_wrapped(i18n::text(text));
    ui.separator();
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

fn draw_region(ui: &Ui, narrowing: &mut Narrowing, candidates: &[Candidate]) {
    // The two regions always exist; listing only the ones with live teams left the drop-down
    // empty until the first poll finished, which looked like a broken control (bug report of
    // 2026-10-31). Regions without teams are still selectable and the team list explains why
    // it is empty.
    let current = narrowing
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
                    .selected(narrowing.region == Some(region))
                    .build(ui)
                {
                    narrowing.region = Some(region);
                }
            }
        });
}

fn draw_colour(ui: &Ui, narrowing: &mut Narrowing) {
    let current = narrowing
        .colour
        .map(super::status::colour_label)
        .unwrap_or_else(|| i18n::text("Colour"));

    ComboBox::new(i18n::widget("Colour"))
        .preview_value(current)
        .build(ui, || {
            for colour in TeamColor::ALL {
                let _tint = ui.push_style_color(StyleColor::Text, colour_rgba(colour));
                let label = format!(
                    "{}###colour{:?}",
                    super::status::colour_label(colour),
                    colour
                );
                if Selectable::new(label)
                    .selected(narrowing.colour == Some(colour))
                    .build(ui)
                {
                    narrowing.colour = Some(colour);
                }
            }
        });
}

#[allow(clippy::too_many_arguments)]
fn draw_team(
    ui: &Ui,
    table: &TeamTable,
    config: &crate::config::Config,
    language: &str,
    narrowing: &Narrowing,
    picked: &mut Option<u32>,
    candidates: &[Candidate],
    worlds: &std::collections::BTreeMap<u32, String>,
) {
    let options = match (narrowing.region, narrowing.colour) {
        (Some(region), Some(colour)) => select::of_region_and_color(candidates, region, colour),
        _ => Vec::new(),
    };

    let current = match *picked {
        Some(id) => crate::team::display_name(config, language, id),
        None => i18n::text("No team configured yet").to_owned(),
    };

    ComboBox::new(i18n::widget("Team"))
        .preview_value(current)
        .height(nexus::imgui::ComboBoxHeight::Large)
        .build(ui, || {
            if Selectable::new(i18n::widget("No team configured yet"))
                .selected(picked.is_none())
                .build(ui)
            {
                *picked = None;
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
                let region = match super::status::region_of(&candidate.match_id) {
                    Some(region) => region_label(region),
                    None => "?",
                };
                let head = format!(
                    "{}  ·  {region}",
                    table.label_in(language, candidate.team_id)
                );
                let label = format!("{head}{worlds}###team{}", candidate.team_id);
                let _tint = ui.push_style_color(StyleColor::Text, colour_rgba(candidate.color));
                if Selectable::new(label)
                    .selected(*picked == Some(candidate.team_id))
                    .build(ui)
                {
                    *picked = Some(candidate.team_id);
                }
            }
        });
}

/// The other two teams of the chosen team's matchup, to check it is the right one (RF-03g).
fn draw_rivals(
    ui: &Ui,
    config: &crate::config::Config,
    language: &str,
    team: u32,
    candidates: &[Candidate],
) {
    let Some(mine) = candidates
        .iter()
        .find(|candidate| candidate.team_id == team)
    else {
        return;
    };
    ui.text_disabled(i18n::text("Your matchup"));
    for candidate in candidates
        .iter()
        .filter(|candidate| candidate.match_id == mine.match_id)
    {
        let _tint = ui.push_style_color(StyleColor::Text, colour_rgba(candidate.color));
        ui.text(format!(
            "{} · {}",
            super::status::colour_label(candidate.color),
            crate::team::display_name(config, language, candidate.team_id)
        ));
    }
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
