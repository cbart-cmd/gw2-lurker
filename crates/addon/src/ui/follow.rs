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

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use gw2lurker_core::model::Region;
use gw2lurker_core::{Candidate, FollowList, MAX_FOLLOWED};
use nexus::imgui::{ComboBox, InputText, Selectable, Ui};

use crate::i18n;

/// What the player typed in the search box and for which team. Session state on purpose: it is
/// a tool, not a setting.
static SEARCH_TEXT: std::sync::OnceLock<std::sync::Mutex<String>> = std::sync::OnceLock::new();
static SEARCH_TEAM: std::sync::OnceLock<std::sync::Mutex<Option<u32>>> = std::sync::OnceLock::new();

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

/// The whole section: the followed list plus the two ways of adding.
pub fn body(
    ui: &Ui,
    follow: &mut FollowList,
    names: &BTreeMap<String, String>,
    candidates: &[Candidate],
    my_team: Option<u32>,
    session: &[(String, usize, Option<DateTime<Utc>>)],
) {
    let label = |id: &str| names.get(id).cloned().unwrap_or_else(|| id.to_owned());

    ui.separator();
    let count = format!("{}/{MAX_FOLLOWED}", follow.len());
    ui.text(i18n::format("Followed guilds ({0})", &[&count]));

    if follow.is_empty() {
        ui.text_disabled(i18n::text("None followed yet"));
    }
    let mut remove = None;
    for guild in follow.clone().iter() {
        ui.text(label(guild));
        ui.same_line();
        if ui.small_button(format!("×###unfollow{guild}")) {
            remove = Some(guild.to_owned());
        }
    }
    if let Some(guild) = remove {
        follow.remove(&guild);
    }

    if follow.is_full() {
        ui.text_disabled(i18n::text(
            "The list is full: stop following one to add another.",
        ));
        return;
    }

    ui.separator();
    if nexus::imgui::CollapsingHeader::new(i18n::widget("Add guild"))
        .default_open(false)
        .build(ui)
    {
        session_block(ui, follow, session, &label);
        ui.separator();
        search_block(ui, follow, candidates, my_team);
    }
}

/// Guilds that claimed something during this session: one click and they are followed.
fn session_block(
    ui: &Ui,
    follow: &mut FollowList,
    session: &[(String, usize, Option<DateTime<Utc>>)],
    label: &impl Fn(&str) -> String,
) {
    ui.text(i18n::text("Claimed since the addon started"));
    let candidates: Vec<_> = session
        .iter()
        .filter(|(guild, ..)| !follow.contains(guild))
        .collect();
    if candidates.is_empty() {
        ui.text_disabled(i18n::text("None yet"));
        return;
    }
    let mut add = None;
    for (guild, claims, last) in candidates {
        let when = last
            .map(|at| at.format("%H:%M").to_string())
            .unwrap_or_default();
        let text = format!("{}  ·  {claims} · {when}", label(guild));
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
    ui.text(i18n::text("Search a guild by team"));
    let region = region_of(candidates, my_team);

    let teams: Vec<&Candidate> = candidates
        .iter()
        .filter(|candidate| region.is_none() || candidate.region() == region)
        .collect();

    let mut chosen = *team_slot();
    let preview = chosen
        .map(|team| team_label(candidates, team))
        .unwrap_or_else(|| i18n::text("Team").to_owned());
    ComboBox::new(i18n::widget("Team"))
        .preview_value(preview)
        .build(ui, || {
            for candidate in &teams {
                let text = team_label(candidates, candidate.team_id);
                if Selectable::new(text)
                    .selected(chosen == Some(candidate.team_id))
                    .build(ui)
                {
                    chosen = Some(candidate.team_id);
                }
            }
        });

    let mut text = text_slot();
    let mut go = ui.button(i18n::widget("Search"));
    ui.same_line();
    InputText::new(ui, "###gw2lurker_search", &mut text)
        .hint(i18n::text("Part of the name, one word"))
        .enter_returns_true(true)
        .build();
    if ui.is_item_hovered() {
        ui.tooltip_text(i18n::text(
            "The API searches one word by prefix: «Raven», not «avenous».",
        ));
    }
    go |= ui.is_item_deactivated_after_edit();

    let ready = chosen.is_some() && !text.trim().is_empty();
    if go && ready {
        if let (Some(team), Some(region)) = (chosen, region) {
            crate::search::start(region, team, text.trim().to_owned());
        }
    }
    *team_slot() = chosen;
    drop(text);

    match crate::search::snapshot() {
        crate::search::State::Idle => {
            ui.text_disabled(i18n::text("Pick a team, type a word and search."));
        }
        crate::search::State::Searching => {
            ui.text_disabled(i18n::text("Searching…"));
        }
        crate::search::State::Failed(error) => {
            ui.text_disabled(i18n::format("Search failed: {0}", &[&error]));
        }
        crate::search::State::Done(hits) => {
            if hits.is_empty() {
                ui.text_disabled(i18n::text("No guild of that team matches."));
                return;
            }
            let mut add = None;
            for hit in &hits {
                let text = format!("{}###hit{}", hit.label, hit.id);
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

/// Name of a team as the player knows it (their own translation), with the id to disambiguate.
fn team_label(_candidates: &[Candidate], team: u32) -> String {
    let name = crate::config()
        .team_translations
        .get(&team.to_string())
        .and_then(|by_language| by_language.values().next().cloned());
    match name {
        Some(name) => format!("{name} ({team})"),
        None => team.to_string(),
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
