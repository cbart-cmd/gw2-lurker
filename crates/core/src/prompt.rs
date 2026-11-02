//! When the team picker has to open by itself (RF-03j).
//!
//! The picker is its own window and it opens on entering a WvW map or Armistice Bastion when the
//! player has to say something: there is no team yet, the saved one no longer plays, or the
//! matchup changed since the player last confirmed it (request of the player, 2026-11-02).

use crate::select::{Candidate, Selection};

/// Why the picker opened by itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prompt {
    /// The account has no team yet: a new player.
    NoTeam,
    /// The saved team is in no live match (monthly re-link).
    Stale,
    /// The saved team still plays, but against other teams than the last confirmed time.
    NewMatchup,
}

/// Identity of the matchup a team plays: the match id and the three teams in it.
///
/// The rivals are what changes at the weekly reset; the match id alone can stay the same
/// (`2-2` one week and the next) while the opponents are different. `None` when the team is not
/// among the candidates, so a missing answer is never mistaken for a new matchup.
pub fn matchup_key(all: &[Candidate], team_id: u32) -> Option<String> {
    let match_id = &all
        .iter()
        .find(|candidate| candidate.team_id == team_id)?
        .match_id;
    let mut teams: Vec<u32> = all
        .iter()
        .filter(|candidate| &candidate.match_id == match_id)
        .map(|candidate| candidate.team_id)
        .collect();
    teams.sort_unstable();
    teams.dedup();
    let teams: Vec<String> = teams.iter().map(u32::to_string).collect();
    Some(format!("{match_id}:{}", teams.join(",")))
}

/// Whether the picker has to open, and why.
///
/// - `selected`: the team of the current account.
/// - `selection`: the worker's verdict on the team it watches; only a verdict on **this** team
///   counts, so a team detected from the game does not open the picker for the saved one.
/// - `current`: [`matchup_key`] of the team right now, if the data has arrived.
/// - `confirmed`: the key the player last confirmed for this team. `None` means nothing was ever
///   confirmed (an upgrade, or a team picked a moment ago), and that is not a change.
pub fn prompt(
    selected: Option<u32>,
    selection: &Selection,
    current: Option<&str>,
    confirmed: Option<&str>,
) -> Option<Prompt> {
    let Some(team) = selected else {
        return Some(Prompt::NoTeam);
    };
    if matches!(selection, Selection::Stale { previous } if *previous == team) {
        return Some(Prompt::Stale);
    }
    match (current, confirmed) {
        (Some(now), Some(before)) if now != before => Some(Prompt::NewMatchup),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Match;

    const ALL: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../api/tests/fixtures/matches_all.json"
    ));

    fn live() -> Vec<Candidate> {
        let matches: Vec<Match> = serde_json::from_str(ALL).unwrap();
        crate::select::candidates(&matches)
    }

    /// The same list with one rival replaced: what the API gives after the weekly reset.
    fn next_week(all: &[Candidate], me: u32) -> Vec<Candidate> {
        let mut out = all.to_vec();
        let rival = out
            .iter_mut()
            .find(|candidate| candidate.team_id != me)
            .unwrap();
        rival.team_id += 1_000;
        out
    }

    #[test]
    fn rf03j_the_key_names_the_match_and_its_three_teams() {
        let all = live();
        let me = all[0].team_id;
        let key = matchup_key(&all, me).unwrap();

        let mine = &all[0].match_id;
        assert!(key.starts_with(&format!("{mine}:")), "{key}");
        for candidate in all.iter().filter(|c| &c.match_id == mine) {
            assert!(key.contains(&candidate.team_id.to_string()), "{key}");
        }
        for rival in all.iter().filter(|c| c.match_id == *mine) {
            assert_eq!(
                matchup_key(&all, rival.team_id).as_deref(),
                Some(key.as_str()),
                "los tres equipos comparten emparejamiento"
            );
        }
        assert_eq!(matchup_key(&all, 99_999), None, "sin datos no hay clave");
    }

    #[test]
    fn rf03j_new_rivals_are_a_new_matchup_with_the_same_match_id() {
        let all = live();
        let me = all[0].team_id;
        let before = matchup_key(&all, me).unwrap();
        let after = matchup_key(&next_week(&all, me), me).unwrap();
        assert_ne!(before, after);

        let confirmed = Selection::Confirmed {
            team_id: me,
            match_id: all[0].match_id.clone(),
        };
        assert_eq!(
            prompt(Some(me), &confirmed, Some(&after), Some(&before)),
            Some(Prompt::NewMatchup)
        );
        assert_eq!(
            prompt(Some(me), &confirmed, Some(&before), Some(&before)),
            None,
            "el mismo emparejamiento no vuelve a preguntar"
        );
    }

    #[test]
    fn rf03j_nothing_confirmed_or_no_data_is_not_a_change() {
        let all = live();
        let me = all[0].team_id;
        let key = matchup_key(&all, me).unwrap();
        let confirmed = Selection::Confirmed {
            team_id: me,
            match_id: all[0].match_id.clone(),
        };
        assert_eq!(prompt(Some(me), &confirmed, Some(&key), None), None);
        assert_eq!(prompt(Some(me), &confirmed, None, Some("2-9:1,2,3")), None);
    }

    #[test]
    fn rf03j_a_new_player_and_a_relinked_team_are_asked() {
        assert_eq!(
            prompt(None, &Selection::Missing, None, None),
            Some(Prompt::NoTeam)
        );
        assert_eq!(
            prompt(
                Some(12099),
                &Selection::Stale { previous: 12099 },
                None,
                None
            ),
            Some(Prompt::Stale)
        );
        assert_eq!(
            prompt(
                Some(12014),
                &Selection::Stale { previous: 11001 },
                None,
                None
            ),
            None,
            "el veredicto de otro equipo (el detectado del juego) no es el del guardado"
        );
    }
}
