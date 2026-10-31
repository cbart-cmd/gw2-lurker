//! Serde model of the public WvW endpoints of the GW2 API.
//!
//! Field names and nullability are taken from real responses captured on 2026-10-31
//! (`crates/api/tests/fixtures/`). The API omits nothing that matters but does send
//! `null` for unclaimed objectives, so every optional field is modelled as `Option`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Side of a matchup. Appears as `"Red"`, `"Green"`, `"Blue"` in the payload; the
/// per-colour containers use lowercase keys instead, which is why they are not
/// modelled with this enum's serde attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum TeamColor {
    Red,
    Green,
    Blue,
}

impl TeamColor {
    /// All colours, in the order the UI shows them.
    pub const ALL: [Self; 3] = [Self::Red, Self::Green, Self::Blue];
}

/// Region of a match, derived from the id prefix (RF-03b).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Region {
    Na,
    Eu,
}

/// A per-colour field of the match payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamScores<T> {
    pub red: T,
    pub green: T,
    pub blue: T,
}

impl<T> TeamScores<T> {
    /// Value of the given colour.
    pub fn get(&self, color: TeamColor) -> &T {
        match color {
            TeamColor::Red => &self.red,
            TeamColor::Green => &self.green,
            TeamColor::Blue => &self.blue,
        }
    }
}

/// Kind of objective, as sent in the `type` field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ObjectiveType {
    Camp,
    Castle,
    Keep,
    Tower,
    Ruins,
    Mercenary,
    Spawn,
    Resource,
    Generic,
    /// Bloodlust ruins (observed in real payloads).
    Bloodlust,
    /// Anything ArenaNet adds later.
    #[serde(other)]
    Unknown,
}

impl ObjectiveType {
    /// Whether a guild can claim it (RF-27b). Only these show the "unclaimed" state.
    pub fn is_claimable(self) -> bool {
        matches!(self, Self::Camp | Self::Tower | Self::Keep | Self::Castle)
    }

    /// Whether the objective scores for its owner. The rest is scenery for the map (RF-30a).
    pub fn scores_points(self) -> bool {
        !matches!(
            self,
            Self::Spawn | Self::Mercenary | Self::Ruins | Self::Bloodlust | Self::Unknown
        )
    }
}

/// `points_tick` at or above which an objective is "high value" (RF-30b).
pub const HIGH_VALUE_TICK: u32 = 8;

/// Role of a map inside the matchup (`type` of the entry in `maps`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MapRole {
    Center,
    BlueHome,
    GreenHome,
    RedHome,
    #[serde(other)]
    Unknown,
}

/// One objective as it appears inside a match snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchObjective {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: ObjectiveType,
    #[serde(default)]
    pub owner: Option<TeamColor>,
    #[serde(default)]
    pub last_flipped: Option<DateTime<Utc>>,
    #[serde(default)]
    pub claimed_by: Option<String>,
    #[serde(default)]
    pub claimed_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub points_tick: u32,
    #[serde(default)]
    pub points_capture: u32,
    #[serde(default)]
    pub yaks_delivered: Option<u32>,
    #[serde(default)]
    pub guild_upgrades: Vec<u32>,
}

impl MatchObjective {
    /// High value per RF-30b: derived from scoring data, never from a hard-coded list.
    pub fn is_high_value(&self) -> bool {
        self.points_tick >= HIGH_VALUE_TICK
    }

    /// Whether it is claimable and currently unclaimed (RF-27b).
    pub fn is_unclaimed(&self) -> bool {
        self.kind.is_claimable() && self.claimed_by.is_none()
    }
}

/// One of the maps of the matchup.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchMap {
    pub id: u32,
    #[serde(rename = "type")]
    pub role: MapRole,
    #[serde(default)]
    pub objectives: Vec<MatchObjective>,
}

/// Match snapshot from `/v2/wvw/matches/:id`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Match {
    pub id: String,
    #[serde(default)]
    pub start_time: Option<DateTime<Utc>>,
    #[serde(default)]
    pub end_time: Option<DateTime<Utc>>,
    pub scores: TeamScores<u32>,
    /// Every id on each side, including the newer team ids (11xxx/12xxx).
    pub all_worlds: TeamScores<Vec<u32>>,
    #[serde(default)]
    pub maps: Vec<MatchMap>,
}

impl Match {
    /// Region derived from the id prefix: `1-x` is NA, `2-x` is EU (RF-03b).
    pub fn region(&self) -> Option<Region> {
        match self.id.split_once('-')?.0 {
            "1" => Some(Region::Na),
            "2" => Some(Region::Eu),
            _ => None,
        }
    }

    /// Colour of the given team (or legacy world) id in this match (RF-03b).
    pub fn color_of(&self, team_id: u32) -> Option<TeamColor> {
        TeamColor::ALL
            .into_iter()
            .find(|color| self.all_worlds.get(*color).contains(&team_id))
    }

    /// The two rival colours, in match order (RF-03g).
    pub fn others(&self, me: TeamColor) -> [TeamColor; 2] {
        match me {
            TeamColor::Red => [TeamColor::Green, TeamColor::Blue],
            TeamColor::Green => [TeamColor::Red, TeamColor::Blue],
            TeamColor::Blue => [TeamColor::Red, TeamColor::Green],
        }
    }

    /// Objectives of a given role, if the map is in this match.
    pub fn map(&self, role: MapRole) -> Option<&MatchMap> {
        self.maps.iter().find(|map| map.role == role)
    }
}

/// Static objective metadata from `/v2/wvw/objectives`.
///
/// `coord` is in continent coordinates and can be `null`; the projection must skip those.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectiveInfo {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: ObjectiveType,
    pub map_id: u32,
    #[serde(default)]
    pub coord: Option<[f64; 3]>,
    #[serde(default)]
    pub label_coord: Option<[f64; 2]>,
    #[serde(default)]
    pub marker: Option<String>,
}

/// A WvW map from `/v2/maps`, with the boxes the projection needs (ADR-011).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WvwMap {
    pub id: u32,
    pub name: String,
    pub map_rect: [[f64; 2]; 2],
    pub continent_rect: [[f64; 2]; 2],
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
    const MAPS: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../api/tests/fixtures/maps_wvw.json"
    ));

    fn parsed_match() -> Match {
        serde_json::from_str(MATCH).unwrap()
    }

    #[test]
    fn rf03b_resolves_color_from_all_worlds_not_worlds() {
        let m = parsed_match();
        assert_eq!(m.color_of(12014), Some(TeamColor::Blue), "team id");
        assert_eq!(m.color_of(2014), Some(TeamColor::Blue), "legacy world id");
        assert_eq!(m.color_of(12015), Some(TeamColor::Red));
        assert_eq!(m.color_of(12007), Some(TeamColor::Green));
        assert_eq!(m.color_of(999_999), None);
    }

    #[test]
    fn rf03b_region_comes_from_match_id_prefix() {
        let m = parsed_match();
        assert_eq!(m.id, "2-2");
        assert_eq!(m.region(), Some(Region::Eu));

        let mut na = m.clone();
        na.id = "1-4".to_string();
        assert_eq!(na.region(), Some(Region::Na));

        na.id = "x-9".to_string();
        assert_eq!(na.region(), None);
    }

    #[test]
    fn rf03g_returns_the_two_rivals() {
        let m = parsed_match();
        assert_eq!(
            m.others(TeamColor::Blue),
            [TeamColor::Red, TeamColor::Green]
        );
    }

    #[test]
    fn rf27b_claimed_and_unclaimed_are_distinguished() {
        let m = parsed_match();
        let objectives: Vec<&MatchObjective> = m.maps.iter().flat_map(|x| &x.objectives).collect();

        let claimed = objectives.iter().filter(|o| o.claimed_by.is_some()).count();
        assert!(claimed > 0, "fixture must contain claims");
        assert!(claimed < objectives.len(), "fixture must contain unclaimed");

        for o in &objectives {
            assert_eq!(o.claimed_by.is_none(), o.claimed_at.is_none());
        }

        let unclaimed = objectives
            .iter()
            .find(|o| o.is_unclaimed())
            .expect("an unclaimed claimable objective");
        assert!(unclaimed.kind.is_claimable());
    }

    #[test]
    fn rf27b_non_claimable_types_are_never_unclaimed() {
        let m = parsed_match();
        for o in m.maps.iter().flat_map(|x| &x.objectives) {
            if !o.kind.is_claimable() {
                assert!(!o.is_unclaimed(), "{:?} is not claimable", o.kind);
            }
        }
    }

    #[test]
    fn rf30b_high_value_is_derived_from_points_tick() {
        let m = parsed_match();
        let objectives: Vec<&MatchObjective> = m.maps.iter().flat_map(|x| &x.objectives).collect();

        for o in objectives.iter().filter(|o| o.kind == ObjectiveType::Keep) {
            assert!(o.is_high_value(), "a keep must be high value: {}", o.id);
        }
        for o in objectives
            .iter()
            .filter(|o| o.kind == ObjectiveType::Castle)
        {
            assert!(o.is_high_value(), "a castle must be high value: {}", o.id);
        }
        for o in objectives.iter().filter(|o| {
            matches!(
                o.kind,
                ObjectiveType::Camp
                    | ObjectiveType::Spawn
                    | ObjectiveType::Mercenary
                    | ObjectiveType::Ruins
            )
        }) {
            assert!(!o.is_high_value(), "{} must not be high value", o.id);
        }

        // Fully upgraded towers do reach the threshold, so the rule cannot be type-based.
        let towers = objectives
            .iter()
            .filter(|o| o.kind == ObjectiveType::Tower)
            .count();
        let high_towers = objectives
            .iter()
            .filter(|o| o.kind == ObjectiveType::Tower && o.is_high_value())
            .count();
        assert!(
            high_towers > 0 && high_towers < towers,
            "expected some but not all towers to be high value, got {high_towers}/{towers}"
        );
    }

    #[test]
    fn rf30a_only_capture_points_score() {
        for kind in [
            ObjectiveType::Camp,
            ObjectiveType::Tower,
            ObjectiveType::Keep,
            ObjectiveType::Castle,
        ] {
            assert!(kind.is_claimable() && kind.scores_points(), "{kind:?}");
        }
        for kind in [
            ObjectiveType::Spawn,
            ObjectiveType::Mercenary,
            ObjectiveType::Ruins,
            ObjectiveType::Bloodlust,
        ] {
            assert!(!kind.is_claimable() && !kind.scores_points(), "{kind:?}");
        }
    }

    #[test]
    fn rf30a_unknown_objective_type_does_not_break_parsing() {
        let json = r#"{ "id": "1-1", "type": "Vine", "owner": "Red" }"#;
        let o: MatchObjective = serde_json::from_str(json).unwrap();
        assert_eq!(o.kind, ObjectiveType::Unknown);
        assert_eq!(o.owner, Some(TeamColor::Red));
    }

    #[test]
    fn t04_objective_info_tolerates_null_coord() {
        let all: Vec<ObjectiveInfo> = serde_json::from_str(OBJECTIVES).unwrap();
        assert!(all.len() > 100, "fixture should hold every objective");
        assert!(all.iter().any(|o| o.coord.is_some()));
        assert!(
            all.iter().any(|o| o.coord.is_none()),
            "81 of 178 objectives had no coord on the capture date"
        );
        assert!(all.iter().all(|o| o.map_id > 0));
    }

    #[test]
    fn rf30_map_bounds_come_from_continent_rect() {
        let maps: Vec<WvwMap> = serde_json::from_str(MAPS).unwrap();
        assert_eq!(maps.len(), 5);

        let ebg = maps.iter().find(|m| m.id == 38).unwrap();
        assert_eq!(ebg.name, "Eternal Battlegrounds");
        assert_eq!(ebg.continent_rect, [[8958.0, 12798.0], [12030.0, 15870.0]]);
        assert!(ebg.map_rect[1][0] > ebg.map_rect[0][0]);
    }

    #[test]
    fn rf30_objective_coords_fall_inside_continent_rect() {
        let maps: Vec<WvwMap> = serde_json::from_str(MAPS).unwrap();
        let all: Vec<ObjectiveInfo> = serde_json::from_str(OBJECTIVES).unwrap();

        for map in &maps {
            let [[x0, y0], [x1, y1]] = map.continent_rect;
            for o in all
                .iter()
                .filter(|o| o.map_id == map.id)
                .filter_map(|o| o.coord.map(|c| (o, c)))
                .map(|(_, c)| c)
            {
                assert!(
                    o[0] > x0 && o[0] < x1 && o[1] > y0 && o[1] < y1,
                    "{:?} outside {}: {:?}",
                    o,
                    map.id,
                    map.continent_rect
                );
            }
        }
    }
}
