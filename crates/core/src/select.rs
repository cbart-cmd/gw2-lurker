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
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Selection {
    /// No team saved yet: the picker must open.
    #[default]
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
    /// Legacy worlds fighting for this team, so the player recognises their own.
    ///
    /// The new teams are named after places nobody has played in, so the picker was a list of
    /// strangers; the old world name is what the player actually knows (report of 2026-10-31:
    /// picked an NA team called *Lutgardis Conservatory* while playing on the EU world Lutgardis).
    pub worlds: Vec<u32>,
}

impl Candidate {
    /// Region of the matchup, from its id prefix (`1-x` is NA, `2-x` is EU).
    pub fn region(&self) -> Option<crate::model::Region> {
        match self.match_id.split_once('-')?.0 {
            "1" => Some(crate::model::Region::Na),
            "2" => Some(crate::model::Region::Eu),
            _ => None,
        }
    }
}

/// Teams of one region and one colour: **one per matchup**, which is what the third step of
/// the picker offers (RF-03a).
///
/// This is the whole point of asking region and colour first: the player recognises their
/// colour in the in-game panel and goes from thirty-three names to four or five.
pub fn of_region_and_color(
    all: &[Candidate],
    region: crate::model::Region,
    color: TeamColor,
) -> Vec<Candidate> {
    all.iter()
        .filter(|candidate| candidate.color == color && candidate.region() == Some(region))
        .cloned()
        .collect()
}

/// The team chosen for each **account**.
///
/// Keyed by the account name arcdps reports (`Jhoin.1234`) since 2026-11-01: every character of
/// an account shares the team, so an alt does not ask again, and a second account gets its own
/// picker instead of inheriting the main's team. The character name stays as the fallback key
/// while arcdps has not said which account we are in — and it is the key older versions wrote,
/// so their entries keep resolving.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Roster {
    by_character: std::collections::BTreeMap<String, u32>,
}

impl Roster {
    /// Team remembered for a character.
    pub fn get(&self, character: &str) -> Option<u32> {
        self.by_character.get(character.trim()).copied()
    }

    /// Remembers the team of a character. An empty name is ignored: without a name the
    /// selection could not be told apart from any other.
    pub fn set(&mut self, character: &str, team_id: u32) {
        let character = character.trim();
        if character.is_empty() {
            return;
        }
        self.by_character.insert(character.to_owned(), team_id);
    }

    /// Forgets a character, used when its team is no longer in any live match.
    pub fn forget(&mut self, character: &str) {
        self.by_character.remove(character.trim());
    }

    /// Characters remembered.
    pub fn len(&self) -> usize {
        self.by_character.len()
    }

    /// Whether nobody has chosen a team yet.
    pub fn is_empty(&self) -> bool {
        self.by_character.is_empty()
    }
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

/// Same decision, but looking the key (account, or character as fallback) up in the roster first.
pub fn evaluate_for(key: &str, roster: &Roster, live: &[Match]) -> Selection {
    evaluate(roster.get(key), live)
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
            let ids = entry.all_worlds.get(color);
            // Los mundos antiguos de ese color: son los que el jugador reconoce.
            let worlds: Vec<u32> = ids.iter().copied().filter(|id| !is_team(*id)).collect();
            for team_id in ids {
                if is_team(*team_id) {
                    out.push(Candidate {
                        team_id: *team_id,
                        match_id: entry.id.clone(),
                        color,
                        worlds: worlds.clone(),
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

/// Matchups of a region, `EU` for ids starting with `2-` and `NA` for `1-` (RF-03b).
///
/// Used by the two-step picker: choose region, then matchup, then one of its three teams.
pub fn matchups_of(live: &[Match], region: crate::model::Region) -> Vec<&Match> {
    let mut out: Vec<&Match> = live
        .iter()
        .filter(|entry| entry.region() == Some(region))
        .collect();
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

/// The three teams of a matchup, in colour order.
pub fn teams_of(live: &[Match], match_id: &str) -> Vec<Candidate> {
    candidates(live)
        .into_iter()
        .filter(|candidate| candidate.match_id == match_id)
        .collect()
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
    fn rf03a_roster_keeps_one_team_per_key() {
        let all = live();
        let mut roster = Roster::default();
        assert_eq!(evaluate_for("Main", &roster, &all), Selection::Missing);

        roster.set("Main", 12014);
        assert_eq!(
            evaluate_for("Main", &roster, &all),
            Selection::Confirmed {
                team_id: 12014,
                match_id: "2-2".to_owned()
            }
        );
        assert_eq!(
            evaluate_for("Alt", &roster, &all),
            Selection::Missing,
            "el alt de otra cuenta no hereda el equipo del main"
        );

        roster.set("Alt", 11001);
        assert_eq!(
            evaluate_for("Alt", &roster, &all),
            Selection::Stale { previous: 11001 },
            "si su equipo ya no juega, se le vuelve a preguntar solo a él"
        );
        assert_eq!(roster.len(), 2);

        roster.set("   ", 12015);
        roster.set("", 12015);
        assert_eq!(roster.len(), 2, "un nombre vacío no se guarda");

        roster.forget("Alt");
        assert_eq!(roster.len(), 1);
    }

    #[test]
    fn rf03b_two_step_picker_groups_by_region_and_matchup() {
        use crate::model::Region;

        let all = live();
        let eu = matchups_of(&all, Region::Eu);
        assert_eq!(eu.len(), 1, "la fixture solo trae 2-2");
        assert_eq!(eu[0].id, "2-2");
        assert!(matchups_of(&all, Region::Na).is_empty());

        let three = teams_of(&all, "2-2");
        assert_eq!(three.len(), 3);
        assert!(three.iter().all(|c| c.match_id == "2-2"));
        assert_eq!(
            three.iter().map(|c| c.color).collect::<Vec<_>>(),
            vec![TeamColor::Red, TeamColor::Green, TeamColor::Blue],
            "el orden de TeamColor::ALL, el mismo que usa la UI"
        );
    }

    #[test]
    fn rf03a_region_and_colour_narrow_the_picker_to_one_team_per_matchup() {
        use crate::model::{Region, TeamColor};
        let all = candidates(&live());

        let blue_eu = of_region_and_color(&all, Region::Eu, TeamColor::Blue);
        assert_eq!(blue_eu.len(), 1, "un equipo azul por matchup de EU");
        assert_eq!(blue_eu[0].team_id, 12014);

        assert_eq!(
            of_region_and_color(&all, Region::Eu, TeamColor::Red)[0].team_id,
            12015
        );
        assert!(
            of_region_and_color(&all, Region::Na, TeamColor::Blue).is_empty(),
            "la fixture solo trae el matchup 2-2"
        );
        assert_eq!(all[0].region(), Some(Region::Eu));
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
