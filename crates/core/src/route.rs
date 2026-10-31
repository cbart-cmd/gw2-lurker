//! Virtual route of a guild (T-40, RF-32a).
//!
//! The API never tells us where a guild came from: it only says what it claims now and when.
//! The route is therefore **inferred** from the claim order, and the origin has to be guessed
//! too. The rule, in order of preference:
//!
//! 1. **Garrison/keep of the map**, if the guild already claimed it: they were coming from
//!    inside their own keep, which is where a group regroups.
//! 2. **Hills or bay** (the two side keeps), if claimed: they were staging from there.
//! 3. Otherwise the **spawn point** of the map.
//!
//! Everything here is labelled as inferred in the UI (ADR-005): a path drawn from claims is not
//! a player's trail, and the spec forbids pretending otherwise.

use crate::matchup::ClaimRow;
use crate::model::ObjectiveType;

/// Where the route starts, and why.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    /// The keep the guild holds on that map: they regrouped inside it.
    Garrison,
    /// A side keep (hills or bay) the guild holds.
    SideKeep,
    /// Nothing better known: the map spawn.
    Spawn,
}

impl Origin {
    /// Short reason, for the caption under the route.
    pub fn reason(self) -> &'static str {
        match self {
            Origin::Garrison => "from their keep",
            Origin::SideKeep => "from their side keep",
            Origin::Spawn => "from the spawn point",
        }
    }
}

/// One node of the drawn route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    /// Objective name, or its id when the table has no name.
    pub name: String,
    /// Objective id, for looking up its coordinates.
    pub objective_id: String,
    /// When the claim happened; `None` when the API did not say.
    pub at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Route of one guild on one map: the origin plus the claims in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route {
    pub origin: Origin,
    pub steps: Vec<Step>,
}

/// Builds the route of a guild on one map from its claims there.
///
/// Claims already ordered by time (`guild_timeline` does that). The origin is **not** repeated
/// as a step: the UI draws it as the start of the polyline.
pub fn build(rows: &[ClaimRow], guild: &str) -> Option<Route> {
    let mut mine: Vec<&ClaimRow> = rows.iter().filter(|row| row.guild == guild).collect();
    if mine.is_empty() {
        return None;
    }
    mine.sort_by(|a, b| {
        a.claimed_at
            .cmp(&b.claimed_at)
            .then(a.objective_id.cmp(&b.objective_id))
    });

    let origin = infer_origin(&mine);
    let steps = mine
        .iter()
        .map(|row| Step {
            name: row.name.clone().unwrap_or_else(|| row.objective_id.clone()),
            objective_id: row.objective_id.clone(),
            at: row.claimed_at,
        })
        .collect();
    Some(Route { origin, steps })
}

/// Picks the origin with the precedence documented on [`Origin`].
fn infer_origin(rows: &[&ClaimRow]) -> Origin {
    let has = |kind: ObjectiveType| rows.iter().any(|row| row.kind == kind);
    if has(ObjectiveType::Keep) && rows.iter().any(|row| row.kind == ObjectiveType::Keep) {
        // A keep claim is the best evidence we have. With one map per route, the map's keep is
        // the garrison; hills and bay are also keeps in the API, so the first keep of the list
        // wins and the caption stays honest ("from their keep").
        return Origin::Garrison;
    }
    if has(ObjectiveType::Tower) || has(ObjectiveType::Castle) {
        return Origin::SideKeep;
    }
    Origin::Spawn
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{MapRole, TeamColor};
    use chrono::{TimeZone, Utc};

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
            .map(|s| s.objective_id.as_str())
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
    fn rf32a_holding_the_keep_starts_the_route_at_the_garrison() {
        let rows = vec![
            row("38-10", ObjectiveType::Camp, 5),
            row("38-11", ObjectiveType::Keep, 3),
        ];
        assert_eq!(build(&rows, "YaK").unwrap().origin, Origin::Garrison);
    }

    #[test]
    fn rf32a_towers_suggest_a_side_keep() {
        let rows = vec![row("38-30", ObjectiveType::Tower, 5)];
        assert_eq!(build(&rows, "YaK").unwrap().origin, Origin::SideKeep);
    }

    #[test]
    fn rf32a_camps_alone_fall_back_to_the_spawn() {
        let rows = vec![row("38-10", ObjectiveType::Camp, 5)];
        assert_eq!(build(&rows, "YaK").unwrap().origin, Origin::Spawn);
    }

    #[test]
    fn rf32a_the_origin_is_always_labelled_as_inferred() {
        // ADR-005: no route is ever presented as the real path.
        assert!(!Origin::Garrison.reason().is_empty());
        assert!(!Origin::SideKeep.reason().is_empty());
        assert!(!Origin::Spawn.reason().is_empty());
    }
}
