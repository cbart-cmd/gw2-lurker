//! Team selection: when the addon has to ask the player which team is theirs (RF-03a).
//!
//! Nothing in the game tells us the player's WvW team: Mumble only gives `map_id`,
//! `map_type` and a socket address, and Unofficial Extras' squad info has no team field
//! (both checked). What the API does give is every live matchup with its three teams, so the
//! picker can offer exactly the teams that exist and, more importantly, we can **validate**
//! a saved team against the live data every time the player enters WvW.
//!
//! That validation is what makes the monthly re-link painless: if the saved team is not in
//! any live match, the picker opens again by itself.

use crate::model::{Match, TeamColor};

/// What the picker must do right now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selection {
    /// No team saved yet: the picker must open.
    Missing,
    /// The saved team is playing a live match: nothing to ask.
    Confirmed { team_id: u32, match_id: String },
    /// The saved team is not in any live match. Happens after the monthly re-link or when
    /// playing on a different account; the picker must open again, remembering the old team.
    Stale { previous: u32 },
}

/// One team the player may pick, with the matchup it is playing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    /// Team id (`11xxx`/`12xxx`) or legacy world id.
    pub team_id: u32,
    /// Match the team is playing, e.g. `2-2`.
    pub match_id: String,
    /// Colour it holds in that match.
    pub color: TeamColor,
}

/// Decides whether the saved team still makes sense.
pub fn evaluate(saved: Option<u32>, live: &[Match]) -> Selection {
    let Some(team_id) = saved else {
        return Selection::Missing;
    };
    match live.iter().find(|entry| entry.color_of(team_id).is_some()) {
        Some(entry) => Selection::Confirmed {
            team_id,
            match_id: entry.id.clone(),
        },
        None => Selection::Stale { previous: team_id },
    }
}

/// Every team playing a live match, ordered by match and then by colour.
///
/// Only real teams (`11xxx`/`12xxx`) are listed: `all_worlds` also carries the legacy world
/// ids for API compatibility, and a player picks a *team*, not a server.
///
/// The order is stable so the picker does not jump while the player is choosing.
pub fn candidates(live: &[Match]) -> Vec<Candidate> {
    let mut matches: Vec<&Match> = live.iter().collect();
    matches.sort_by(|a, b| a.id.cmp(&b.id));

    let mut out = Vec::new();
    for entry in matches {
        for color in TeamColor::ALL {
            for team_id in entry.all_worlds.get(color) {
                if is_team(*team_id) {
                    out.push(Candidate {
                        team_id: *team_id,
                        match_id: entry.id.clone(),
                        color,
                    });
                }
            }
        }
    }
    out
}

/// Whether an id is a WvW team id rather than a legacy world id.
pub fn is_team(id: u32) -> bool {
    (11_000..=12_999).contains(&id)
}

/// Teams of the match the player is in, for the "pick again" case: the three rivals are far
/// easier to recognise than a list of thirty.
pub fn rivals(live: &[Match], match_id: &str, me: u32) -> Vec<Candidate> {
    candidates(live)
        .into_iter()
        .filter(|candidate| candidate.match_id == match_id && candidate.team_id != me)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../api/tests/fixtures/matches_all.json"
    ));

    fn live() -> Vec<Match> {
        serde_json::from_str(ALL).unwrap()
    }

    #[test]
    fn rf03a_without_saved_team_the_picker_opens() {
        assert_eq!(evaluate(None, &live()), Selection::Missing);
        assert_eq!(
            evaluate(None, &[]),
            Selection::Missing,
            "sin red también hay que preguntar"
        );
    }

    #[test]
    fn rf03a_saved_team_still_playing_is_confirmed() {
        // Great House Aviary is blue in the live 2-2.
        let selection = evaluate(Some(12014), &live());
        assert_eq!(
            selection,
            Selection::Confirmed {
                team_id: 12014,
                match_id: "2-2".to_owned()
            }
        );
    }

    #[test]
    fn rf03a_relinked_team_opens_the_picker_again() {
        // 12099 does not exist in any live match, which is what a re-link looks like.
        assert_eq!(
            evaluate(Some(12099), &live()),
            Selection::Stale { previous: 12099 }
        );
    }

    #[test]
    fn rf03a_candidates_are_the_three_teams_of_each_match() {
        let all = live();
        let list = candidates(&all);
        assert_eq!(list.len(), all.len() * 3, "tres colores por matchup");
        assert!(
            list.iter().all(|candidate| is_team(candidate.team_id)),
            "los ids de mundo antiguos (2xxx) no se ofrecen"
        );

        let mut sorted: Vec<&str> = list.iter().map(|c| c.match_id.as_str()).collect();
        sorted.dedup();
        assert_eq!(sorted.len(), all.len(), "cada matchup sale seguido");
        assert_eq!(sorted, vec!["2-2"], "orden estable (la fixture trae 2-2)");
    }

    #[test]
    fn rf03a_the_two_teams_of_a_colour_are_not_confused() {
        // Blue in 2-2 is `12014` plus the legacy world `2014`; only the team is offered.
        let all = live();
        let r = all.iter().find(|entry| entry.id == "2-2").unwrap();
        assert!(r.all_worlds.blue.contains(&12014));
        let picked: Vec<u32> = candidates(std::slice::from_ref(r))
            .into_iter()
            .map(|candidate| candidate.team_id)
            .collect();
        assert_eq!(picked.len(), 3);
        assert!(picked.contains(&12014));
        assert!(!picked.contains(&2014));
    }

    #[test]
    fn rf03g_rivals_are_the_other_two_of_my_match() {
        let all = live();
        let others = rivals(&all, "2-2", 12014);
        assert_eq!(others.len(), 2, "un rival por color");
        assert!(others.iter().all(|c| c.match_id == "2-2"));
        assert!(others.iter().all(|c| c.team_id != 12014));
        assert!(
            others.iter().all(|c| c.color != TeamColor::Blue),
            "el mío era azul"
        );
    }
}
