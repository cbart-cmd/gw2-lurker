//! Followed guilds selector (T-44).
//!
//! It used to list **every** guild of the matchup, which grows without bound and ends up being a
//! wall of rows nobody reads (report of 2026-10-31). Now it shows only what the player follows,
//! and adding is a deliberate action with two ways in:
//!
//! 1. **What claimed something since the addon started** — the session has those claims already,
//!    so this costs no network at all.
//! 2. **Search by team and name** — the team is mandatory and comes from the player's region,
//!    because a guild only matters if it plays in your matchup.
//!
//! The cap is [`MAX_FOLLOWED`]: ten is more than anyone watches, and every followed guild is a
//! ring on the map plus a filter on the alerts.
//!
//! Since 0.18.0 the two sections are drawn by the **Guilds** tab and not by the settings:
//! following a guild is something you decide while looking at the match, not a preference you go
//! hunting for (report of 2026-10-31).

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use gw2lurker_core::model::Region;
use gw2lurker_core::{Candidate, FollowList, MAX_FOLLOWED};
use nexus::imgui::{ComboBox, InputText, Selectable, Ui};

use crate::i18n;

/// How many guilds the *Claimed since the addon started* shortcut offers.
const MAX_SESSION_ROWS: usize = 10;

/// What the player typed in the search box and for which team. Session state on purpose: it is
/// a tool, not a setting.
static SEARCH_TEXT: std::sync::OnceLock<std::sync::Mutex<String>> = std::sync::OnceLock::new();
static SEARCH_TEAM: std::sync::OnceLock<std::sync::Mutex<Option<u32>>> = std::sync::OnceLock::new();
static SEARCH_REGION: std::sync::OnceLock<std::sync::Mutex<Option<Region>>> =
    std::sync::OnceLock::new();

fn text_slot() -> std::sync::MutexGuard<'static, String> {
    SEARCH_TEXT
        .get_or_init(|| std::sync::Mutex::new(String::new()))
        .lock()
        .unwrap_or_else(|error| error.into_inner())
}

fn team_slot() -> std::sync::MutexGuard<'static, Option<u32>> {
    SEARCH_TEAM
        .get_or_init(|| std::sync::Mutex::new(None))
        .lock()
        .unwrap_or_else(|error| error.into_inner())
}

/// Region the player last picked in the search. Starts as their own, which is the useful default.
fn region_slot() -> std::sync::MutexGuard<'static, Option<Region>> {
    SEARCH_REGION
        .get_or_init(|| std::sync::Mutex::new(None))
        .lock()
        .unwrap_or_else(|error| error.into_inner())
}

/// Guild waiting for a *yes* before it stops being followed.
///
/// The `×` removes a followed guild and there is no undo, so it asks first, in the same row: a
/// modal for two buttons is heavier than the action (report of 2026-10-31).
static PENDING_REMOVE: std::sync::OnceLock<std::sync::Mutex<Option<String>>> =
    std::sync::OnceLock::new();

fn pending_remove() -> std::sync::MutexGuard<'static, Option<String>> {
    PENDING_REMOVE
        .get_or_init(|| std::sync::Mutex::new(None))
        .lock()
        .unwrap_or_else(|error| error.into_inner())
}

/// The guilds the player follows, with a `×` to stop following each one.
///
/// It lives in the *Guilds* tab and not in the settings: following a guild is something you
/// decide while looking at the match, not a preference you go hunting for (report of 2026-10-31).
pub fn followed_section(
    ui: &Ui,
    follow: &mut FollowList,
    names: &BTreeMap<String, String>,
) -> Option<String> {
    let count = format!("{}/{MAX_FOLLOWED}", follow.len());
    ui.text(i18n::format("Followed guilds ({0})", &[&count]));

    if follow.is_empty() {
        ui.text_disabled(i18n::text("None followed yet"));
        return None;
    }
    let mut remove = None;
    let mut chosen = None;
    let mut pending = pending_remove();
    for guild in follow.clone().iter() {
        if pending.as_deref() == Some(guild) {
            let question = i18n::format("Stop following {0}?", &[&label(names, guild)]);
            ui.text(question);
            ui.same_line();
            if ui.small_button(i18n::widget("Yes")) {
                remove = Some(guild.to_owned());
                *pending = None;
            }
            ui.same_line();
            if ui.small_button(i18n::widget("No")) {
                *pending = None;
            }
            continue;
        }
        // Se puede clicar: el jugador quiere ver la ruta de ese clan (petición de 2026-10-31).
        if Selectable::new(format!("{}###followed{guild}", label(names, guild))).build(ui) {
            chosen = Some(guild.to_owned());
        }
        ui.same_line();
        if ui.small_button(format!("×###unfollow{guild}")) {
            *pending = Some(guild.to_owned());
        }
    }
    drop(pending);
    if let Some(guild) = remove {
        follow.remove(&guild);
    }
    chosen
}

/// Ways of following a new guild: what claimed something this session, and the search.
///
/// It is a collapsing header, closed by default: it is a tool, and the list above is the answer
/// most of the time.
pub fn add_section(
    ui: &Ui,
    follow: &mut FollowList,
    names: &BTreeMap<String, String>,
    candidates: &[Candidate],
    my_team: Option<u32>,
    session: &[(String, usize, Option<DateTime<Utc>>)],
) {
    if follow.is_full() {
        ui.text_disabled(i18n::text(
            "The list is full: stop following one to add another.",
        ));
        return;
    }
    if !nexus::imgui::CollapsingHeader::new(i18n::widget("Add guild"))
        .default_open(false)
        .build(ui)
    {
        return;
    }
    session_block(ui, follow, session, names);
    ui.separator();
    search_block(ui, follow, candidates, my_team);
}

/// `[TAG] Nombre` when it is known, the raw id otherwise.
fn label(names: &BTreeMap<String, String>, id: &str) -> String {
    names.get(id).cloned().unwrap_or_else(|| id.to_owned())
}

/// Guilds that claimed something during this session: one click and they are followed.
fn session_block(
    ui: &Ui,
    follow: &mut FollowList,
    session: &[(String, usize, Option<DateTime<Utc>>)],
    names: &BTreeMap<String, String>,
) {
    ui.text(i18n::text("Claimed since the addon started"));
    // The ten most recent: the list is a shortcut, not an inventory, and it was growing without
    // bound (report of 2026-10-31).
    let candidates: Vec<_> = session
        .iter()
        .filter(|(guild, ..)| !follow.contains(guild))
        .take(MAX_SESSION_ROWS)
        .collect();
    if candidates.is_empty() {
        ui.text_disabled(i18n::text("None yet"));
        return;
    }
    let mut add = None;
    for (guild, claims, last) in candidates {
        let when = last.map(crate::clock::clock).unwrap_or_default();
        let text = format!("{}  ·  {claims} · {when}", label(names, guild));
        ui.text(text);
        ui.same_line();
        if ui.small_button(format!("+###add{guild}")) {
            add = Some(guild.clone());
        }
    }
    if let Some(guild) = add {
        follow.add(&guild);
    }
}

/// Search by team and name. The team is mandatory and only lists teams of the player's region.
fn search_block(ui: &Ui, follow: &mut FollowList, candidates: &[Candidate], my_team: Option<u32>) {
    // The list of live teams is fetched by the worker while this is on screen: inside WvW it is
    // only fetched when asked for, and nobody was asking (bug of 2026-10-31).
    crate::worker::set_teams_needed(true);

    ui.text(i18n::text("Search a guild"));

    // **Region first**: a guild only matters if it plays in the half of the world the player cares
    // about, and that is what the search really needs. It starts as the player's own and can be
    // changed, because everybody has friends on the other side (report of 2026-10-31).
    let mine = region_of(candidates, my_team);
    let mut region = region_slot().or(mine).unwrap_or(Region::Eu);
    let preview = region_label(region).to_owned();
    ComboBox::new(i18n::widget("Region"))
        .preview_value(preview)
        .build(ui, || {
            for option in [Region::Eu, Region::Na] {
                if Selectable::new(region_label(option))
                    .selected(region == option)
                    .build(ui)
                {
                    region = option;
                }
            }
        });
    *region_slot() = Some(region);

    let mut text = text_slot();
    let mut go = InputText::new(ui, "###gw2lurker_search", &mut text)
        .hint(i18n::text("Part of the name, one word"))
        .enter_returns_true(true)
        .build();
    if ui.is_item_hovered() {
        ui.tooltip_text(i18n::text(
            "The API searches one word by prefix: «Raven», not «avenous».",
        ));
    }
    go |= ui.is_item_deactivated_after_edit();
    ui.same_line();
    go |= ui.button(i18n::widget("Search"));

    // Optional: with no team the search covers the whole region, which is what the player wants
    // when they only remember part of the name.
    let teams: Vec<&Candidate> = candidates
        .iter()
        .filter(|candidate| candidate.region() == Some(region))
        .collect();
    let mut chosen = *team_slot();
    if chosen.is_some_and(|team| !teams.iter().any(|c| c.team_id == team)) {
        chosen = None;
    }
    if teams.is_empty() {
        ui.text_disabled(i18n::text("The list of teams arrives with the next poll."));
    } else {
        let any = i18n::text("Team (any)").to_owned();
        let preview = match chosen {
            Some(team) => team_label(team),
            None => any.clone(),
        };
        ComboBox::new(i18n::widget("Team (any)"))
            .preview_value(preview)
            .build(ui, || {
                if Selectable::new(any).selected(chosen.is_none()).build(ui) {
                    chosen = None;
                }
                for candidate in &teams {
                    let text = team_label(candidate.team_id);
                    if Selectable::new(text)
                        .selected(chosen == Some(candidate.team_id))
                        .build(ui)
                    {
                        chosen = Some(candidate.team_id);
                    }
                }
            });
    }
    *team_slot() = chosen;

    if go && !text.trim().is_empty() {
        crate::search::start(region, chosen, text.trim().to_owned());
    }
    drop(text);

    match crate::search::snapshot() {
        crate::search::State::Idle => {
            ui.text_disabled(i18n::text("Type part of the name and search."));
        }
        crate::search::State::Searching => {
            ui.text_disabled(i18n::text("Searching…"));
        }
        crate::search::State::Failed(error) => {
            ui.text_disabled(i18n::format("Search failed: {0}", &[&error]));
        }
        crate::search::State::Done(hits) => {
            if hits.is_empty() {
                ui.text_disabled(i18n::text("No guild matches."));
                return;
            }
            let mut add = None;
            for hit in &hits {
                // Con «Team (any)» el equipo del resultado importa: dice si es del emparejamiento
                // del jugador antes de seguir el clan.
                let text = match hit.team {
                    Some(team) if chosen.is_none() => {
                        format!("{}  ·  {}###hit{}", hit.label, team_label(team), hit.id)
                    }
                    _ => format!("{}###hit{}", hit.label, hit.id),
                };
                if Selectable::new(text).build(ui) {
                    add = Some(hit.id.clone());
                }
            }
            if let Some(guild) = add {
                follow.add(&guild);
            }
        }
    }
}
/// Name of a team as the player knows it: their own translation first, then the bundled name from
/// `data/wvw-teams.json`, and the id in brackets to disambiguate.
///
/// It used to fall back to the bare id, so the drop-down was a column of numbers (report of
/// 2026-10-31).
fn team_label(team: u32) -> String {
    let config = crate::config();
    let language = i18n::effective().code();
    let name = crate::team::display_name(config, language, team);
    format!("{name} ({team})")
}

/// `EU` / `NA` as the player thinks of them.
fn region_label(region: Region) -> &'static str {
    match region {
        Region::Eu => "EU",
        Region::Na => "NA",
    }
}

/// Region of the configured team, for the search thread.
pub fn region_of(candidates: &[Candidate], team: Option<u32>) -> Option<Region> {
    team.and_then(|team| {
        candidates
            .iter()
            .find(|candidate| candidate.team_id == team)
            .and_then(Candidate::region)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gw2lurker_core::model::TeamColor;

    fn candidate(team_id: u32, match_id: &str) -> Candidate {
        Candidate {
            team_id,
            match_id: match_id.to_owned(),
            color: TeamColor::Blue,
            worlds: Vec::new(),
        }
    }

    #[test]
    fn t44_the_search_region_is_the_one_of_the_players_team() {
        // 11011 juega en un emparejamiento 1-x (NA) y 12014 en uno 2-x (EU): comprobado contra
        // `/v2/wvw/matches?ids=all` (2026-10-31). El buscador solo mira la región del jugador.
        let teams = vec![candidate(11011, "1-1"), candidate(12014, "2-2")];
        assert_eq!(region_of(&teams, Some(11011)), Some(Region::Na));
        assert_eq!(region_of(&teams, Some(12014)), Some(Region::Eu));
    }

    #[test]
    fn t44_without_a_team_there_is_no_region_and_the_ui_says_so() {
        // Sin equipo no se busca a ciegas: la ventana pide elegirlo en vez de no hacer nada
        // (bug de 2026-10-31, el botón de buscar se quedaba mudo).
        let teams = vec![candidate(11011, "1-1")];
        assert_eq!(region_of(&teams, None), None);
        assert_eq!(
            region_of(&teams, Some(99999)),
            None,
            "un equipo que ya no juega no da región"
        );
        assert_eq!(region_of(&[], Some(11011)), None);
    }
}
