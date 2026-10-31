//! Resolves the player's team inside a match snapshot (RF-03b, RF-03g).

use crate::model::{Match, Region, TeamColor};

/// Outcome of resolving one team id against a match.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Resolution {
    /// Colour of the player's team in this match.
    pub color: TeamColor,
    /// The two rival colours.
    pub rivals: [TeamColor; 2],
    /// Region derived from the match id, when the id follows the known scheme.
    pub region: Option<Region>,
}

impl Resolution {
    /// Whether the given colour is the player's own team.
    pub fn is_own(&self, color: TeamColor) -> bool {
        self.color == color
    }
}

/// Finds the colour of `team_id` inside the match.
///
/// The id is looked up in `all_worlds`, never in `worlds`: the latter only holds the
/// legacy world per colour, while the former carries every linked id including the
/// newer team ids (RF-03b).
pub fn resolve(m: &Match, team_id: u32) -> Option<Resolution> {
    let color = m.color_of(team_id)?;
    Some(Resolution {
        color,
        rivals: m.others(color),
        region: m.region(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const MATCH: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../api/tests/fixtures/match_2-2.json"
    ));

    fn parsed() -> Match {
        serde_json::from_str(MATCH).unwrap()
    }

    #[test]
    fn rf03b_resolves_team_color_and_rivals() {
        let m = parsed();
        let r = resolve(&m, 12014).expect("team 12014 is in match 2-2");

        assert_eq!(r.color, TeamColor::Blue);
        assert_eq!(r.rivals, [TeamColor::Red, TeamColor::Green]);
        assert_eq!(r.region, Some(Region::Eu));
        assert!(r.is_own(TeamColor::Blue));
        assert!(!r.is_own(TeamColor::Red));
    }

    #[test]
    fn rf03b_linked_legacy_world_resolves_to_the_same_side() {
        let m = parsed();
        let team = resolve(&m, 12014).unwrap();
        let world = resolve(&m, 2014).expect("2014 is linked to the same colour");

        assert_eq!(team.color, world.color);
    }

    #[test]
    fn rf03b_unknown_team_is_not_resolved() {
        let m = parsed();
        assert_eq!(resolve(&m, 999_999), None);
    }
}
