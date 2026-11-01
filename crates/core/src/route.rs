//! Virtual route of a guild (T-40, RF-32a).
//!
//! The API never tells us where a guild came from: it only says what it claims now and when.
//! The route is therefore **inferred** from the claim order, and the origin has to be guessed
//! too. A located origin needs the current ownership, claim times and objective coordinates;
//! historical objective types alone cannot identify a garrison or a departure point.
//!
//! Everything here is labelled as inferred in the UI (ADR-005): a path drawn from claims is not
//! a player's trail, and the spec forbids pretending otherwise.

use crate::matchup::ClaimRow;
use crate::matchup::ObjectiveState;
use crate::model::{MapRole, ObjectiveInfo, ObjectiveType, TeamColor};
use std::borrow::Cow;

pub mod playback;
pub mod trail;

/// Where the route starts, and why.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    /// Inferred departure from the guild's previously claimed home garrison.
    Garrison,
    /// Previously claimed alpine hills/bay, for a segment starting and ending north of it.
    SideKeep,
    /// Nothing better known: the map spawn.
    Spawn,
    /// No defensible origin can be located with the available data.
    Unknown,
}

impl Origin {
    /// Short reason, for the caption under the route.
    pub fn reason(self) -> &'static str {
        match self {
            Origin::Garrison => "from their keep",
            Origin::SideKeep => "from their side keep",
            Origin::Spawn => "from the spawn point",
            Origin::Unknown => "Origin unknown",
        }
    }
}

/// One node of the drawn route.
///
/// Text borrows the history to avoid two string allocations per claim while preparing the map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step<'a> {
    /// Objective name, or its id when the table has no name.
    pub name: Cow<'a, str>,
    /// Objective id, for looking up its coordinates.
    pub objective_id: Cow<'a, str>,
    /// When the claim happened; `None` when the API did not say.
    pub at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Route of one guild on one map: the origin plus the claims in order.
#[derive(Debug, Clone, PartialEq)]
pub struct Route<'a> {
    pub origin: Origin,
    pub origin_point: Option<OriginPoint>,
    pub steps: Vec<Step<'a>>,
}

/// Located origin, distinct from an observed claim and without a made-up timestamp.
#[derive(Debug, Clone, PartialEq)]
pub struct OriginPoint {
    pub objective_id: String,
    pub position: [f64; 2],
    pub approximate: bool,
}

/// Builds the observed route on one map and locates its inferred departure (RF-32a).
pub fn with_origin<'a>(
    rows: &'a [ClaimRow],
    guild: &str,
    map_id: u32,
    states: &[ObjectiveState],
    names: &[ObjectiveInfo],
    since: Option<chrono::DateTime<chrono::Utc>>,
) -> Option<Route<'a>> {
    let mine = ordered(rows, guild, Some(map_id), since);
    let mut route = build_claims(&mine)?;
    (route.origin, route.origin_point) = locate_origin(&mine, guild, states, names);
    Some(route)
}

/// Cuántos pasos de la ruta se han «reproducido» con una fracción `progress` de 0 a 1 (RF-36).
///
/// Al menos uno desde el principio: una ruta sin su primer punto no dice de dónde sale.
pub fn visible_steps(steps: usize, progress: f32) -> usize {
    if steps == 0 {
        return 0;
    }
    let clamped = progress.clamp(0.0, 1.0);
    let count = (clamped * steps as f32).ceil() as usize;
    count.clamp(1, steps)
}

/// Fracción de la ruta reproducida tras `elapsed` segundos de una reproducción de `seconds`.
pub fn progress(elapsed: f32, seconds: f32) -> f32 {
    if seconds <= 0.0 || !seconds.is_finite() {
        return 1.0;
    }
    (elapsed / seconds).clamp(0.0, 1.0)
}

/// Builds chronological observations; locating an origin additionally requires [`with_origin`].
///
/// Claims already ordered by time (`guild_timeline` does that). The origin is **not** repeated
/// as a step: the UI draws it as the start of the polyline.
pub fn build<'a>(rows: &'a [ClaimRow], guild: &str) -> Option<Route<'a>> {
    build_claims(&ordered(rows, guild, None, None))
}

fn ordered<'a>(
    rows: &'a [ClaimRow],
    guild: &str,
    map_id: Option<u32>,
    since: Option<chrono::DateTime<chrono::Utc>>,
) -> Vec<&'a ClaimRow> {
    let mut mine: Vec<_> = rows
        .iter()
        .filter(|row| {
            row.guild == guild
                && map_id.is_none_or(|id| row.map_id == id)
                && since.is_none_or(|start| row.claimed_at.is_some_and(|at| at >= start))
        })
        .collect();
    mine.sort_by(|a, b| {
        a.claimed_at
            .cmp(&b.claimed_at)
            .then(a.objective_id.cmp(&b.objective_id))
    });
    mine
}

fn build_claims<'a>(mine: &[&'a ClaimRow]) -> Option<Route<'a>> {
    if mine.is_empty() {
        return None;
    }
    let steps = mine
        .iter()
        .map(|row| Step {
            name: Cow::Borrowed(row.name.as_deref().unwrap_or(&row.objective_id)),
            objective_id: Cow::Borrowed(&row.objective_id),
            at: row.claimed_at,
        })
        .collect();
    Some(Route {
        origin: Origin::Unknown,
        origin_point: None,
        steps,
    })
}

/// Infers an origin from chronological claims already restricted to one guild and map.
///
/// The temporal UI reuses this without allocating route steps or cloning the source claims.
pub fn locate_origin(
    rows: &[&ClaimRow],
    guild: &str,
    states: &[ObjectiveState],
    names: &[ObjectiveInfo],
) -> (Origin, Option<OriginPoint>) {
    let (Some(first), Some(last)) = (rows.first(), rows.last()) else {
        return (Origin::Unknown, None);
    };
    let map_id = first.map_id;
    let colour = states
        .iter()
        .filter(|state| state.map_id == map_id && state.guild.as_deref() == Some(guild))
        .max_by_key(|state| state.claimed_at)
        .and_then(|state| state.owner)
        .or(last.owner);
    let Some(colour) = colour else {
        return (Origin::Unknown, None);
    };
    let home = match colour {
        TeamColor::Red => MapRole::RedHome,
        TeamColor::Green => MapRole::GreenHome,
        TeamColor::Blue => MapRole::BlueHome,
    };
    // Verified objective identities: objectives_all.json (95/96-37; 1099-113).
    let garrison = match map_id {
        95 => "95-37",
        96 => "96-37",
        1099 => "1099-113",
        _ => "",
    };
    let eligible = |state: &&ObjectiveState| {
        state.map_id == map_id
            && state.kind == ObjectiveType::Keep
            && state.guild.as_deref() == Some(guild)
            && state.owner == Some(colour)
            && state
                .claimed_at
                .zip(first.claimed_at)
                .is_some_and(|(claimed, start)| claimed <= start)
    };
    if first.map_role == home {
        if let Some(point) = states
            .iter()
            .filter(eligible)
            .find(|state| state.objective_id == garrison)
            .and_then(|state| origin_point(state, names, false))
        {
            return (Origin::Garrison, Some(point));
        }
    }
    if matches!(map_id, 95 | 96) {
        let coords = |row: &&ClaimRow| {
            names
                .iter()
                .find(|name| name.id == row.objective_id)
                .and_then(|name| name.coord)
        };
        if let (Some(start), Some(end)) = (coords(first), coords(last)) {
            let candidate = states
                .iter()
                .filter(eligible)
                .filter(|state| {
                    matches!(
                        state.objective_id.as_str(),
                        "95-32" | "95-33" | "96-32" | "96-33"
                    )
                })
                .filter_map(|state| origin_point(state, names, false))
                .filter(|point| start[1] < point.position[1] && end[1] < point.position[1])
                .min_by(|a, b| {
                    let distance = |point: &OriginPoint| {
                        (point.position[0] - start[0]).powi(2)
                            + (point.position[1] - start[1]).powi(2)
                    };
                    distance(a)
                        .total_cmp(&distance(b))
                        .then(a.objective_id.cmp(&b.objective_id))
                });
            if let Some(point) = candidate {
                return (Origin::SideKeep, Some(point));
            }
        }
    }
    let spawn = states
        .iter()
        .find(|state| {
            state.map_id == map_id
                && state.kind == ObjectiveType::Spawn
                && state.owner == Some(colour)
        })
        .and_then(|state| origin_point(state, names, true));
    if spawn.is_some() {
        (Origin::Spawn, spawn)
    } else {
        (Origin::Unknown, None)
    }
}

fn origin_point(
    state: &ObjectiveState,
    names: &[ObjectiveInfo],
    allow_label: bool,
) -> Option<OriginPoint> {
    let objective = names.iter().find(|name| name.id == state.objective_id)?;
    let (position, approximate) = if let Some(coord) = objective.coord {
        ([coord[0], coord[1]], false)
    } else if allow_label {
        (objective.label_coord?, true)
    } else {
        return None;
    };
    if !position.iter().all(|value| value.is_finite()) {
        return None;
    }
    Some(OriginPoint {
        objective_id: objective.id.clone(),
        position,
        approximate,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{MapRole, TeamColor};
    use chrono::{TimeZone, Utc};

    fn origin_fixture() -> (Vec<ObjectiveInfo>, Vec<ObjectiveState>) {
        let names: Vec<ObjectiveInfo> = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../api/tests/fixtures/objectives_all.json"
        )))
        .unwrap();
        let m: crate::Match = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../api/tests/fixtures/match_2-2.json"
        )))
        .unwrap();
        let spawns = crate::matchup::states(&m)
            .into_iter()
            .filter(|state| state.kind == ObjectiveType::Spawn)
            .collect();
        (names, spawns)
    }

    fn real_row(names: &[ObjectiveInfo], id: &str, minute: u32, owner: TeamColor) -> ClaimRow {
        let objective = names.iter().find(|objective| objective.id == id).unwrap();
        let mut claim = row(id, objective.kind, minute);
        claim.map_id = objective.map_id;
        claim.map_role = MapRole::from_map_id(objective.map_id);
        claim.owner = Some(owner);
        claim
    }

    fn held(row: &ClaimRow) -> ObjectiveState {
        ObjectiveState {
            map_id: row.map_id,
            map_role: row.map_role,
            objective_id: row.objective_id.clone(),
            kind: row.kind,
            owner: row.owner,
            guild: Some(row.guild.clone()),
            claimed_at: row.claimed_at,
            points_tick: row.points_tick,
        }
    }

    fn near_keep(names: &[ObjectiveInfo], keep_id: &str, north: bool, minute: u32) -> ClaimRow {
        let keep = names
            .iter()
            .find(|objective| objective.id == keep_id)
            .unwrap();
        let camp = names
            .iter()
            .find(|objective| {
                objective.map_id == keep.map_id
                    && objective.kind == ObjectiveType::Camp
                    && objective
                        .coord
                        .is_some_and(|coord| (coord[1] < keep.coord.unwrap()[1]) == north)
            })
            .unwrap();
        real_row(names, &camp.id, minute, TeamColor::Green)
    }

    #[test]
    fn rf32a_step_zero_locates_a_previously_claimed_home_garrison() {
        let (names, mut states) = origin_fixture();
        let garrison = real_row(&names, "95-37", 1, TeamColor::Green);
        states.push(held(&garrison));
        let claims = [near_keep(&names, "95-37", true, 5)];
        let route = with_origin(&claims, "YaK", 95, &states, &names, None).unwrap();
        assert_eq!(route.origin, Origin::Garrison);
        let origin = route.origin_point.unwrap();
        assert_eq!(origin.objective_id, garrison.objective_id);
        assert!(!origin.approximate);
        assert_eq!(route.steps.len(), 1, "zero is not another claim");
    }

    #[test]
    fn rf32a_an_alpine_side_keep_requires_the_whole_segment_to_start_north() {
        let (names, mut states) = origin_fixture();
        states.push(held(&real_row(&names, "95-33", 1, TeamColor::Green)));
        let north = near_keep(&names, "95-33", true, 5);
        let route = with_origin(
            std::slice::from_ref(&north),
            "YaK",
            95,
            &states,
            &names,
            None,
        )
        .unwrap();
        assert_eq!(route.origin, Origin::SideKeep);
        assert_eq!(route.origin_point.unwrap().objective_id, "95-33");
        let south = near_keep(&names, "95-33", false, 2);
        let rows = [south, north];
        let route = with_origin(&rows, "YaK", 95, &states, &names, None).unwrap();
        assert_eq!(route.origin, Origin::Spawn);
    }

    #[test]
    fn rf32a_a_future_garrison_cannot_be_the_departure_for_earlier_claims() {
        let (names, mut states) = origin_fixture();
        states.push(held(&real_row(&names, "95-37", 10, TeamColor::Green)));
        let rows = [near_keep(&names, "95-37", true, 5)];
        let route = with_origin(&rows, "YaK", 95, &states, &names, None).unwrap();
        assert_eq!(route.origin, Origin::Spawn);
        assert_eq!(route.origin_point.unwrap().objective_id, "95-111");
    }

    #[test]
    fn rf32a_respawn_uses_the_guild_colour_and_the_official_label_position() {
        let (names, states) = origin_fixture();
        for spawn in states.iter().filter(|state| state.map_id == 95) {
            let owner = spawn.owner.unwrap();
            let mut claim = near_keep(&names, "95-37", true, 5);
            claim.owner = Some(owner);
            let route = with_origin(
                std::slice::from_ref(&claim),
                "YaK",
                95,
                &states,
                &names,
                None,
            )
            .unwrap();
            assert_eq!(route.origin, Origin::Spawn);
            let point = route.origin_point.unwrap();
            let objective = names.iter().find(|o| o.id == spawn.objective_id).unwrap();
            assert_eq!(point.position, objective.label_coord.unwrap());
            assert_eq!(point.objective_id, spawn.objective_id);
            assert!(point.approximate);
        }
    }

    #[test]
    fn rf32a_time_ranges_and_map_segments_do_not_change_earlier_departures() {
        let (names, mut states) = origin_fixture();
        states.push(held(&real_row(&names, "95-33", 1, TeamColor::Green)));
        let rows = [
            near_keep(&names, "95-33", false, 2),
            near_keep(&names, "95-33", true, 5),
        ];
        let since = row("x", ObjectiveType::Camp, 4).claimed_at;
        let route = with_origin(&rows, "YaK", 95, &states, &names, since).unwrap();
        assert_eq!(route.origin, Origin::SideKeep);
        assert_eq!(route.steps.len(), 1);
        assert!(with_origin(&rows, "YaK", 38, &states, &names, None).is_none());
        assert!(with_origin(
            &rows,
            "YaK",
            95,
            &states,
            &names,
            row("x", ObjectiveType::Camp, 6).claimed_at
        )
        .is_none());
    }

    #[test]
    fn rf32a_enemy_garrisons_and_eternal_keeps_are_not_home_departures() {
        let (names, mut states) = origin_fixture();
        states.push(held(&real_row(&names, "95-37", 1, TeamColor::Blue)));
        let mut claim = near_keep(&names, "95-37", true, 5);
        claim.owner = Some(TeamColor::Blue);
        let route = with_origin(
            std::slice::from_ref(&claim),
            "YaK",
            95,
            &states,
            &names,
            None,
        )
        .unwrap();
        assert_eq!(route.origin, Origin::Spawn);
        assert_eq!(route.origin_point.unwrap().objective_id, "95-112");
        let keep = real_row(&names, "38-2", 1, TeamColor::Blue);
        states.push(held(&keep));
        let route = with_origin(
            std::slice::from_ref(&keep),
            "YaK",
            38,
            &states,
            &names,
            None,
        )
        .unwrap();
        assert_eq!(route.origin, Origin::Spawn);
        assert_eq!(route.origin_point.unwrap().objective_id, "38-130");
    }

    #[test]
    fn rf32a_desert_uses_its_home_garrison_but_not_the_alpine_side_keep_rule() {
        let (names, mut states) = origin_fixture();
        let mut claim = near_keep(&names, "1099-114", true, 5);
        claim.owner = Some(TeamColor::Red);
        states.push(held(&real_row(&names, "1099-114", 1, TeamColor::Red)));
        let route = with_origin(
            std::slice::from_ref(&claim),
            "YaK",
            1099,
            &states,
            &names,
            None,
        )
        .unwrap();
        assert_eq!(route.origin, Origin::Spawn);
        states.push(held(&real_row(&names, "1099-113", 1, TeamColor::Red)));
        let route = with_origin(
            std::slice::from_ref(&claim),
            "YaK",
            1099,
            &states,
            &names,
            None,
        )
        .unwrap();
        assert_eq!(route.origin, Origin::Garrison);
        assert_eq!(route.origin_point.unwrap().objective_id, "1099-113");
    }

    #[test]
    fn rf32a_missing_colour_or_coordinates_never_fabricates_step_zero() {
        let (mut names, states) = origin_fixture();
        let mut claim = near_keep(&names, "95-37", true, 5);
        claim.owner = None;
        let route = with_origin(
            std::slice::from_ref(&claim),
            "YaK",
            95,
            &states,
            &names,
            None,
        )
        .unwrap();
        assert_eq!(route.origin, Origin::Unknown);
        assert_eq!(route.origin_point, None);
        claim.owner = Some(TeamColor::Green);
        let spawn = names.iter_mut().find(|name| name.id == "95-111").unwrap();
        spawn.coord = None;
        spawn.label_coord = None;
        let route = with_origin(
            std::slice::from_ref(&claim),
            "YaK",
            95,
            &states,
            &names,
            None,
        )
        .unwrap();
        assert_eq!(route.origin, Origin::Unknown);
        assert_eq!(route.origin_point, None);
    }

    fn row(id: &str, kind: ObjectiveType, minute: u32) -> ClaimRow {
        ClaimRow {
            map_id: 38,
            map_role: MapRole::Center,
            objective_id: id.to_owned(),
            name: Some(id.to_owned()),
            kind,
            owner: Some(TeamColor::Blue),
            guild: "YaK".to_owned(),
            claimed_at: Some(Utc.with_ymd_and_hms(2026, 10, 31, 12, minute, 0).unwrap()),
            points_tick: 2,
        }
    }

    #[test]
    fn rf32a_route_follows_the_claim_order() {
        let rows = vec![
            row("38-30", ObjectiveType::Tower, 20),
            row("38-10", ObjectiveType::Camp, 5),
            row("38-20", ObjectiveType::Camp, 12),
        ];
        let route = build(&rows, "YaK").unwrap();
        let order: Vec<&str> = route
            .steps
            .iter()
            .map(|s| s.objective_id.as_ref())
            .collect();
        assert_eq!(
            order,
            ["38-10", "38-20", "38-30"],
            "de más antiguo a más nuevo"
        );
    }

    #[test]
    fn rf32a_a_guild_without_claims_has_no_route() {
        let rows = vec![row("38-10", ObjectiveType::Camp, 5)];
        assert_eq!(build(&rows, "Otra"), None);
    }

    #[test]
    fn rf32a_claim_types_without_context_cannot_identify_a_garrison() {
        let rows = vec![
            row("38-10", ObjectiveType::Camp, 5),
            row("38-11", ObjectiveType::Keep, 3),
        ];
        assert_eq!(build(&rows, "YaK").unwrap().origin, Origin::Unknown);
    }

    #[test]
    fn rf32a_towers_do_not_identify_a_side_keep() {
        let rows = vec![row("38-30", ObjectiveType::Tower, 5)];
        assert_eq!(build(&rows, "YaK").unwrap().origin, Origin::Unknown);
    }

    #[test]
    fn rf32a_spawn_without_coordinates_remains_unknown() {
        let rows = vec![row("38-10", ObjectiveType::Camp, 5)];
        assert_eq!(build(&rows, "YaK").unwrap().origin, Origin::Unknown);
    }

    #[test]
    fn rf32a_the_origin_is_always_labelled_as_inferred() {
        // ADR-005: no route is ever presented as the real path.
        assert!(!Origin::Garrison.reason().is_empty());
        assert!(!Origin::SideKeep.reason().is_empty());
        assert!(!Origin::Spawn.reason().is_empty());
    }

    #[test]
    fn rf36_playback_shows_the_steps_it_has_reached() {
        assert_eq!(visible_steps(0, 0.5), 0, "sin ruta no hay nada que enseñar");
        assert_eq!(visible_steps(4, 0.0), 1, "al empezar ya está el origen");
        assert_eq!(visible_steps(4, 0.5), 2, "a mitad de camino, dos pasos");
        assert_eq!(visible_steps(4, 1.0), 4, "al final, todos");
        assert_eq!(
            visible_steps(4, 2.0),
            4,
            "una fracción imposible no se sale de la ruta"
        );
    }

    #[test]
    fn rf36_progress_goes_from_zero_to_one() {
        assert_eq!(progress(0.0, 10.0), 0.0);
        assert_eq!(progress(5.0, 10.0), 0.5);
        assert_eq!(progress(20.0, 10.0), 1.0, "no pasa del final");
        assert_eq!(
            progress(5.0, 0.0),
            1.0,
            "sin duración (o con basura) la ruta está entera"
        );
        assert_eq!(progress(5.0, f32::NAN), 1.0);
    }
}
