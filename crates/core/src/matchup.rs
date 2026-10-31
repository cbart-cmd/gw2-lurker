//! Claim-centric view models: what is claimed right now across the matchup (RF-70),
//! which guilds are active and how recently (RF-71) and how a guild's history splits
//! between the player's map and the others (RF-72).
//!
//! Pure data shaping: no drawing, no network, no clock. The UI decides how to show it.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};

use crate::model::{MapRole, Match, ObjectiveInfo, ObjectiveType, TeamColor};

/// One claimed objective, with everything the list views need.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimRow {
    /// Map the objective belongs to.
    pub map_id: u32,
    /// Role of that map inside the matchup.
    pub map_role: MapRole,
    /// Objective id (`"38-20"`).
    pub objective_id: String,
    /// Human name from `/v2/wvw/objectives`, when available.
    pub name: Option<String>,
    /// Objective kind.
    pub kind: ObjectiveType,
    /// Current owner colour.
    pub owner: Option<TeamColor>,
    /// Guild holding the claim.
    pub guild: String,
    /// Claim time reported by the API; `None` if the payload had no `claimed_at`.
    pub claimed_at: Option<DateTime<Utc>>,
    /// `points_tick` of the objective in this snapshot (RF-30b).
    ///
    /// It is what decides "high value": a fully upgraded tower is worth as much as a keep, so the
    /// list cannot be hard-coded by kind.
    pub points_tick: u32,
}

impl ClaimRow {
    /// Whether the objective scores enough to be marked (RF-30b, threshold in
    /// [`crate::model::HIGH_VALUE_TICK`]).
    pub fn is_high_value(&self) -> bool {
        self.points_tick >= crate::model::HIGH_VALUE_TICK
    }
}

/// Whether the guild that claimed the objective still holds it.
///
/// The API keeps `claimed_by` after the objective changes hands: only a **new guild claim**
/// replaces it. So a claim is stale as soon as the objective flipped after the claim was made
/// (`last_flipped` later than `claimed_at`), and listing it paints a guild's ring on a camp of
/// another colour — exactly what the player saw on 2026-10-31 («el mapa ponía que YaK tenía
/// reclamado un campa que en verdad era de otro color»).
fn claim_holds(objective: &crate::model::MatchObjective) -> bool {
    match (objective.claimed_at, objective.last_flipped) {
        (Some(claimed), Some(flipped)) => flipped <= claimed,
        // Without both stamps the claim cannot be dated, so it is kept: dropping it would hide
        // claims the API simply did not date.
        _ => true,
    }
}

/// Flattens every claimed objective of the match (RF-70).
///
/// Ordering is `(map id, objective id)` so the same snapshot always produces the same
/// list. `names` is the static table; unknown ids simply carry `None`.
pub fn claims(m: &Match, names: &[ObjectiveInfo]) -> Vec<ClaimRow> {
    let by_id: BTreeMap<&str, &ObjectiveInfo> = names.iter().map(|o| (o.id.as_str(), o)).collect();

    let mut rows: Vec<ClaimRow> = m
        .maps
        .iter()
        .flat_map(|map| {
            map.objectives
                .iter()
                .filter(|o| claim_holds(o))
                .filter_map(|o| {
                    o.claimed_by.as_ref().map(|guild| ClaimRow {
                        map_id: map.id,
                        map_role: map.role,
                        objective_id: o.id.clone(),
                        name: by_id.get(o.id.as_str()).map(|i| i.name.clone()),
                        kind: o.kind,
                        owner: o.owner,
                        guild: guild.clone(),
                        claimed_at: o.claimed_at,
                        points_tick: o.points_tick,
                    })
                })
        })
        .collect();

    rows.sort_by(|a, b| (a.map_id, &a.objective_id).cmp(&(b.map_id, &b.objective_id)));
    rows
}

/// Rows grouped by map role, in a stable order (RF-70).
pub fn by_map(rows: &[ClaimRow]) -> Vec<(MapRole, Vec<ClaimRow>)> {
    const ORDER: [MapRole; 4] = [
        MapRole::Center,
        MapRole::GreenHome,
        MapRole::BlueHome,
        MapRole::RedHome,
    ];

    let mut out: Vec<(MapRole, Vec<ClaimRow>)> = Vec::new();
    for role in ORDER {
        let group: Vec<ClaimRow> = rows
            .iter()
            .filter(|r| r.map_role == role)
            .cloned()
            .collect();
        if !group.is_empty() {
            out.push((role, group));
        }
    }
    out
}

/// Rows of the given kind only (RF-30a filter, applied to the claim list).
pub fn with_kinds(rows: &[ClaimRow], kinds: &[ObjectiveType]) -> Vec<ClaimRow> {
    rows.iter()
        .filter(|r| kinds.contains(&r.kind))
        .cloned()
        .collect()
}

/// One guild with its observed claims.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuildSummary {
    pub guild: String,
    pub claims: usize,
    /// Most recent claim time, `None` when no row had a timestamp.
    pub last_at: Option<DateTime<Utc>>,
    /// Map of the most recent claim.
    pub last_map: Option<(u32, MapRole)>,
}

/// Guilds ranked by most recent claim first (RF-71).
///
/// Ties fall back to more claims and then to the guild id, so the order is stable
/// across polls: a list that shuffles on every tick is unreadable.
pub fn rank_guilds(rows: &[ClaimRow]) -> Vec<GuildSummary> {
    let mut acc: BTreeMap<&str, GuildSummary> = BTreeMap::new();

    for row in rows {
        let entry = acc
            .entry(row.guild.as_str())
            .or_insert_with(|| GuildSummary {
                guild: row.guild.clone(),
                claims: 0,
                last_at: None,
                last_map: None,
            });
        entry.claims += 1;

        let newer = match (row.claimed_at, entry.last_at) {
            (Some(new), Some(old)) => new > old,
            (Some(_), None) => true,
            _ => false,
        };
        if newer {
            entry.last_at = row.claimed_at;
            entry.last_map = Some((row.map_id, row.map_role));
        }
    }

    let mut out: Vec<GuildSummary> = acc.into_values().collect();
    out.sort_by(|a, b| {
        b.last_at
            .cmp(&a.last_at)
            .then(b.claims.cmp(&a.claims))
            .then(a.guild.cmp(&b.guild))
    });
    out
}

/// Splits a guild's claims into "the map the player is on" and "the rest" (RF-72).
pub fn split_by_map(rows: &[ClaimRow], current: MapRole) -> (Vec<ClaimRow>, Vec<ClaimRow>) {
    let mut here = Vec::new();
    let mut elsewhere = Vec::new();
    for row in rows {
        if row.map_role == current {
            here.push(row.clone());
        } else {
            elsewhere.push(row.clone());
        }
    }
    (here, elsewhere)
}

/// Rows of one guild, oldest first (the order the route needs).
pub fn guild_timeline(rows: &[ClaimRow], guild: &str) -> Vec<ClaimRow> {
    let mut out: Vec<ClaimRow> = rows.iter().filter(|r| r.guild == guild).cloned().collect();
    out.sort_by(|a, b| {
        a.claimed_at
            .cmp(&b.claimed_at)
            .then(a.objective_id.cmp(&b.objective_id))
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const MATCH: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../api/tests/fixtures/match_2-2.json"
    ));
    const OBJECTIVES: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../api/tests/fixtures/objectives_all.json"
    ));

    fn fixture() -> (Match, Vec<ObjectiveInfo>) {
        (
            serde_json::from_str(MATCH).unwrap(),
            serde_json::from_str(OBJECTIVES).unwrap(),
        )
    }

    #[test]
    fn rf70_a_claim_that_flipped_away_is_not_listed() {
        // La API no borra `claimed_by` al cambiar el color: solo un **nuevo reclamo** lo
        // sustituye. Si el objetivo dio la vuelta después del reclamo, el clan ya no lo tiene y
        // pintarlo deja un anillo en un campa de otro color (informe de 2026-10-31).
        let json = r#"{
            "id": "1-1",
            "scores": {"green": 0, "blue": 0, "red": 0},
            "all_worlds": {"green": [], "blue": [], "red": []},
            "maps": [{
                "id": 38,
                "type": "Center",
                "objectives": [
                    {"id": "38-1", "type": "Camp", "owner": "Blue",
                     "claimed_by": "AAAA", "claimed_at": "2026-10-31T22:02:34Z",
                     "last_flipped": "2026-10-31T22:02:32Z"},
                    {"id": "38-2", "type": "Camp", "owner": "Red",
                     "claimed_by": "AAAA", "claimed_at": "2026-10-31T20:00:00Z",
                     "last_flipped": "2026-10-31T21:30:00Z"},
                    {"id": "38-3", "type": "Camp", "owner": "Green",
                     "claimed_by": "BBBB"}
                ]
            }]
        }"#;
        let m: Match = serde_json::from_str(json).unwrap();
        let rows = claims(&m, &[]);

        let ids: Vec<&str> = rows.iter().map(|r| r.objective_id.as_str()).collect();
        assert_eq!(
            ids,
            ["38-1", "38-3"],
            "38-2 se dio la vuelta después del reclamo; 38-3 no tiene fechas y se queda"
        );
        assert!(
            rows.iter()
                .all(|r| r.guild != "AAAA" || r.objective_id == "38-1"),
            "un reclamo caducado no debe aparecer ni como fila del clan"
        );
    }

    #[test]
    fn rf70_the_fixture_claims_are_all_current() {
        // Invariante de los datos reales: en la fixture el reclamo es posterior a la captura.
        // Si algún día no lo es, este test lo dice antes de que la lista se vacíe sin motivo.
        let (m, names) = fixture();
        let claimed: Vec<_> = m
            .maps
            .iter()
            .flat_map(|map| &map.objectives)
            .filter(|o| o.claimed_by.is_some())
            .collect();
        assert!(!claimed.is_empty());
        for objective in claimed {
            if let (Some(claimed_at), Some(flipped)) =
                (objective.claimed_at, objective.last_flipped)
            {
                assert!(
                    flipped <= claimed_at,
                    "{} se capturó después de reclamarse: regla o fixture, una de las dos está mal",
                    objective.id
                );
            }
        }
        assert_eq!(claims(&m, &names).len(), 41, "la fixture no pierde filas");
    }

    #[test]
    fn rf70_every_claim_of_the_match_is_listed() {
        let (m, names) = fixture();
        let rows = claims(&m, &names);
        assert_eq!(rows.len(), 41, "the fixture carried 41 claims");
        assert!(rows.iter().all(|r| !r.guild.is_empty()));

        let expected: usize = m
            .maps
            .iter()
            .flat_map(|mp| &mp.objectives)
            .filter(|o| o.claimed_by.is_some())
            .count();
        assert_eq!(rows.len(), expected);
    }

    #[test]
    fn rf70_rows_are_grouped_by_map_in_a_stable_order() {
        let (m, names) = fixture();
        let groups = by_map(&claims(&m, &names));

        assert!(groups.len() >= 3, "a full matchup has four populated maps");
        let order: Vec<MapRole> = groups.iter().map(|(role, _)| *role).collect();
        let mut sorted = order.clone();
        sorted.sort_by_key(|r| match r {
            MapRole::Center => 0,
            MapRole::GreenHome => 1,
            MapRole::BlueHome => 2,
            MapRole::RedHome => 3,
            MapRole::Unknown => 4,
        });
        assert_eq!(order, sorted);
        assert!(groups.iter().all(|(_, g)| !g.is_empty()));
    }

    #[test]
    fn rf30a_filter_keeps_only_the_requested_kinds() {
        let (m, names) = fixture();
        let rows = with_kinds(&claims(&m, &names), &[ObjectiveType::Keep]);
        assert!(!rows.is_empty());
        assert!(rows.iter().all(|r| r.kind == ObjectiveType::Keep));
    }

    #[test]
    fn rf71_guilds_are_ranked_by_most_recent_claim() {
        let (m, names) = fixture();
        let ranked = rank_guilds(&claims(&m, &names));

        assert!(!ranked.is_empty());
        assert_eq!(
            ranked.iter().map(|g| g.claims).sum::<usize>(),
            41,
            "the ranking must not lose claims"
        );
        for pair in ranked.windows(2) {
            assert!(
                pair[0].last_at >= pair[1].last_at,
                "{} must not come before {}",
                pair[0].guild,
                pair[1].guild
            );
        }
        assert!(ranked[0].last_map.is_some());
    }

    #[test]
    fn rf71_ranking_is_deterministic_for_ties() {
        let now = DateTime::from_timestamp(1_800_000_000, 0).unwrap();
        let row = |guild: &str| ClaimRow {
            map_id: 38,
            map_role: MapRole::Center,
            objective_id: "38-1".to_string(),
            name: None,
            kind: ObjectiveType::Camp,
            owner: Some(TeamColor::Red),
            guild: guild.to_string(),
            claimed_at: Some(now),
            points_tick: 2,
        };

        let a = rank_guilds(&[row("bbb"), row("aaa")]);
        let b = rank_guilds(&[row("aaa"), row("bbb")]);
        assert_eq!(a, b, "same data, same order");
        assert_eq!(a[0].guild, "aaa");
    }

    #[test]
    fn rf72_history_splits_current_map_from_the_rest() {
        let (m, names) = fixture();
        let rows = claims(&m, &names);
        let guild = rows[0].guild.clone();
        let timeline = guild_timeline(&rows, &guild);
        let (here, elsewhere) = split_by_map(&timeline, rows[0].map_role);

        assert_eq!(here.len() + elsewhere.len(), timeline.len());
        assert!(here.iter().all(|r| r.map_role == rows[0].map_role));
        assert!(elsewhere.iter().all(|r| r.map_role != rows[0].map_role));
    }

    #[test]
    fn rf72_timeline_is_oldest_first() {
        let (m, names) = fixture();
        let rows = claims(&m, &names);
        let guild = rows[0].guild.clone();
        let timeline = guild_timeline(&rows, &guild);
        for pair in timeline.windows(2) {
            assert!(pair[0].claimed_at <= pair[1].claimed_at);
        }
    }
}
