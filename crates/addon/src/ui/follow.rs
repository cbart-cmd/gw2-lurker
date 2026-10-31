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
use nexus::imgui::{InputText, Selectable, Ui};

use crate::i18n;

/// How many rows a local search shows.
///
/// Más que los ocho del API: la caché es instantánea y gratuita, y con nombres parecidos
/// («die …») el jugador quiere ver varios.
const MAX_SEARCH_ROWS: usize = 12;

/// How many guilds the *Claimed since the addon started* shortcut offers.
const MAX_SESSION_ROWS: usize = 10;

/// What the player typed in the search box. Session state on purpose: it is a tool, not a
/// setting.
static SEARCH_TEXT: std::sync::OnceLock<std::sync::Mutex<String>> = std::sync::OnceLock::new();

fn text_slot() -> std::sync::MutexGuard<'static, String> {
    SEARCH_TEXT
        .get_or_init(|| std::sync::Mutex::new(String::new()))
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

/// Width reserved on the right of each followed row for the `×`.
///
/// The `Selectable` of the label spans the whole width of the tab, so a button drawn after it
/// with `same_line` ends up **outside** the window and cannot be clicked — which is exactly what
/// «pulsar x no lo borra» looked like (report of 2026-10-31). The space is reserved first, the
/// same trick the title bar uses for its own close button.
const REMOVE_WIDTH: f32 = 24.0;

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
            // La pregunta va en su propia línea y los botones en la siguiente: en una sola línea
            // los botones se salían por la derecha y no se podía confirmar (informe de
            // 2026-10-31, «pulsar x no lo borra»).
            ui.text_wrapped(i18n::format(
                "Remove {0} from the guilds you follow?",
                &[&label(names, guild)],
            ));
            if ui.small_button(i18n::widget("Yes, remove it")) {
                remove = Some(guild.to_owned());
                *pending = None;
            }
            ui.same_line();
            if ui.small_button(i18n::widget("Keep following")) {
                *pending = None;
            }
            ui.separator();
            continue;
        }
        // Se puede clicar: el jugador quiere ver la ruta de ese clan (petición de 2026-10-31).
        // La fila lleva **ancho fijo** en vez de estirarse: una `Selectable` a todo lo ancho se
        // comía el sitio de la `×` y la empujaba fuera de la ventana (informe de 2026-10-31).
        let mut unfollow = false;
        let start = ui.cursor_pos();
        let width = ui.content_region_avail()[0];
        let row = Selectable::new(format!("{}###followed{guild}", label(names, guild)));
        if width > REMOVE_WIDTH * 2.0 {
            let row_width = (width - REMOVE_WIDTH - ui.clone_style().item_spacing[0]).max(40.0);
            ui.set_cursor_pos([start[0] + width - REMOVE_WIDTH, start[1]]);
            unfollow = ui.small_button(format!("×###unfollow{guild}"));
            ui.set_cursor_pos(start);
            if row.size([row_width, 0.0]).build(ui) {
                chosen = Some(guild.to_owned());
            }
        } else if row.build(ui) {
            chosen = Some(guild.to_owned());
        }
        if width <= REMOVE_WIDTH * 2.0 {
            ui.same_line();
            unfollow = ui.small_button(format!("×###unfollow{guild}"));
        }
        if unfollow {
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
    ui.text(i18n::text("Holding something now"));
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

/// Buscador de clanes: **caché local primero** —subcadena, instantánea, sin red— y el API como
/// extra para lo que aún no esté en la caché.
///
/// `/v2/guild/search?name=` devuelve **un solo** clan y solo cuando el texto empieza su nombre
/// (medido el 2026-11-01: «die» → *Die*, no *Die Garde* ni las demás), así que la búsqueda de
/// verdad es la local, sobre los nombres que el addon ya conoce (`guild-names.json`, que también
/// puede rellenar `scripts/guild-search.ps1 -BuildIndex`).
fn search_block(ui: &Ui, follow: &mut FollowList, candidates: &[Candidate], my_team: Option<u32>) {
    // La lista de equipos vivos se pide mientras esto está en pantalla (el selector la usa).
    crate::worker::set_teams_needed(true);

    ui.text(i18n::text("Search a guild"));

    let mut text = text_slot();
    let mut go = InputText::new(ui, "###gw2lurker_search", &mut text)
        .hint(i18n::text("Part of the name"))
        .enter_returns_true(true)
        .build();
    if ui.is_item_hovered() {
        ui.tooltip_text(i18n::text(
            "Searches the guilds the addon already knows, plus one hit from the API.",
        ));
    }
    go |= ui.is_item_deactivated_after_edit();
    ui.same_line();
    go |= ui.button(i18n::widget("Search"));

    let needle = text.trim().to_owned();
    if go && !needle.is_empty() {
        // El API solo necesita la región del jugador; el equipo ya no descarta nada.
        let region = region_of(candidates, my_team).unwrap_or(Region::Eu);
        crate::search::start(region, None, needle.clone());
    }
    drop(text);

    // Caché local: al teclear ya hay resultados, sin gastar una petición.
    let known = crate::names::search(&needle, MAX_SEARCH_ROWS);
    let mut add = None;
    for (id, label) in &known {
        let star = if follow.contains(id) { "★ " } else { "" };
        if Selectable::new(format!("{star}{label}###known{id}")).build(ui) {
            add = Some(id.clone());
        }
    }
    if let Some(guild) = add {
        follow.add(&guild);
        return;
    }
    let cached = crate::names::all().len();
    if !needle.is_empty() {
        ui.text_disabled(i18n::format(
            "{0} guilds in the cache",
            &[&cached.to_string()],
        ));
    }

    match crate::search::snapshot() {
        crate::search::State::Idle => {
            ui.text_disabled(i18n::text(
                "Type part of the name; the list updates as you type.",
            ));
        }
        crate::search::State::Searching => {
            ui.text_disabled(i18n::text("Searching…"));
        }
        crate::search::State::Failed(error) => {
            ui.text_disabled(i18n::format("Search failed: {0}", &[&error]));
        }
        crate::search::State::Done(hits) => {
            let mut add = None;
            for hit in hits
                .iter()
                .filter(|hit| crate::names::get(&hit.id).is_none())
            {
                // Los que ya salieron de la caché no se repiten; la estrella marca lo que sigues.
                let star = if follow.contains(&hit.id) { "★ " } else { "" };
                let text = match hit.team {
                    Some(team) => format!(
                        "{star}{}  ·  {}###hit{}",
                        hit.label,
                        team_label(team),
                        hit.id
                    ),
                    None => format!("{star}{}###hit{}", hit.label, hit.id),
                };
                if Selectable::new(text).build(ui) {
                    add = Some(hit.id.clone());
                }
            }
            if let Some(guild) = add {
                follow.add(&guild);
                return;
            }
            if hits.is_empty() && known.is_empty() && needle.len() >= 2 {
                // El API devuelve **uno** y solo por el principio del nombre; la caché es la que
                // busca de verdad, así que se dice qué hay y cómo llenarla.
                ui.text_wrapped(i18n::format(
                    "Nothing matches in the cache ({0} guilds). The API returns one guild at most, and only when the text starts its name.",
                    &[&cached.to_string()],
                ));
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

    #[test]
    fn t44_the_guilds_you_follow_are_found_whatever_the_api_answers() {
        // El caso real del 2026-10-31: en EU, `/v2/guild/search?name=dolyak` devuelve
        // **DYAK · Dolyak**, no **YaK · Dolyak Sanctuary**, que es el clan del jugador y el que
        // sigue. La búsqueda local (subcadena, `crate::names`) sí lo encuentra.
        let yak = "[YaK] Dolyak Sanctuary";
        assert!(crate::names::matches_text(yak, "6C45F3C8", "dolyak"));
        assert!(crate::names::matches_text(yak, "6C45F3C8", "YAK"));
        assert!(crate::names::matches_text(yak, "6C45F3C8", " dolyak "));
        assert!(
            crate::names::matches_text(yak, "6C45F3C8", "6c45"),
            "también por id"
        );
        assert!(
            !crate::names::matches_text(yak, "6C45F3C8", "waifu"),
            "lo que no coincide no se cuela"
        );
    }
}
