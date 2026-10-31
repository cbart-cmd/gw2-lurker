//! Which matchup is the player actually in, worked out from the game itself (RF-03a).
//!
//! Every «no salen avisos» so far had the same cause: the addon was watching a matchup from the
//! saved team, and the player was somewhere else (report of 2026-10-31: «entré, reclamé campa y no
//! lo detectó», while the API had nothing new for half an hour in the matchup being polled).
//!
//! Mumble publishes `shard_id`, which in WvW is the world the player fights for, and
//! `team_color_id`, which is a team colour. `all_worlds` of every matchup lists **both** the old
//! world ids and the new team ids (`11xxx`/`12xxx`), so looking the shard up in all nine matchups
//! says where the player is, without asking them anything.
//!
//! UNVERIFIED(T-19): that `shard_id` is the WvW world has not been confirmed in game. This module
//! is written so a wrong shard simply detects nothing and the saved team is used as before.

use crate::model::{Match, TeamColor};

/// Where the player is playing, as the game tells it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Detected {
    /// Matchup id, `2-3` and the like.
    pub match_id: String,
    /// The **new** team id of the player's colour in that matchup (`11xxx`/`12xxx`).
    pub team: u32,
    /// Colour the game gave the player.
    pub colour: TeamColor,
    /// Whether the colour agreed with Mumble's `team_color_id`.
    ///
    /// It is the only cross-check available, so it is reported instead of hidden: when it is
    /// `false`, the shard was found but in another colour, which means one of the two is off.
    pub colour_confirmed: bool,
}

/// Finds the matchup of a shard, preferring the one where the colour also matches.
///
/// `colour` is Mumble's `team_color_id` translated; `None` when the game gave no colour (outside
/// WvW it sends `0`).
pub fn of_shard(matches: &[Match], shard: u32, colour: Option<TeamColor>) -> Option<Detected> {
    if shard == 0 {
        return None;
    }
    let mut fallback = None;
    for entry in matches {
        let Some(found) = entry.color_of(shard) else {
            continue;
        };
        let team = entry
            .all_worlds
            .get(found)
            .iter()
            .copied()
            .find(|id| (11_000..13_000).contains(id))
            .unwrap_or(shard);
        let candidate = Detected {
            match_id: entry.id.clone(),
            team,
            colour: found,
            colour_confirmed: colour.is_none_or(|wanted| wanted == found),
        };
        if candidate.colour_confirmed {
            return Some(candidate);
        }
        fallback.get_or_insert(candidate);
    }
    fallback
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, green: &[u32], blue: &[u32], red: &[u32]) -> Match {
        Match {
            id: id.to_owned(),
            start_time: None,
            end_time: None,
            scores: crate::model::TeamScores {
                green: 0,
                blue: 0,
                red: 0,
            },
            all_worlds: crate::model::TeamScores {
                green: green.to_vec(),
                blue: blue.to_vec(),
                red: red.to_vec(),
            },
            maps: Vec::new(),
        }
    }

    #[test]
    fn rf03a_the_shard_says_which_matchup_the_player_is_in() {
        // Los ids reales de la partida `1-1` del 2026-10-31, con equipos nuevos y mundos viejos
        // mezclados en `all_worlds`, que es como los manda el API.
        let live = vec![
            entry(
                "1-1",
                &[1015, 11012, 1012],
                &[11011, 1014, 1011],
                &[1016, 11001, 1001],
            ),
            entry(
                "2-3",
                &[12010, 2206, 2010],
                &[12009, 2205, 2009],
                &[12003, 2104, 2003],
            ),
        ];

        // Un mundo viejo de EU: tiene que caer en `2-3`, no en la partida de Norteamérica.
        let found = of_shard(&live, 2206, Some(TeamColor::Green)).expect("está en 2-3");
        assert_eq!(found.match_id, "2-3");
        assert_eq!(found.team, 12010, "el equipo nuevo de ese color");
        assert!(found.colour_confirmed);

        // Y un equipo nuevo, que es lo que Mumble puede mandar en el futuro.
        let found = of_shard(&live, 11011, Some(TeamColor::Blue)).expect("está en 1-1");
        assert_eq!(found.match_id, "1-1");
        assert_eq!(found.team, 11011);
    }

    #[test]
    fn rf03a_a_shard_nobody_knows_detects_nothing() {
        let live = vec![entry("1-1", &[1015], &[1014], &[1016])];
        assert_eq!(of_shard(&live, 2207, Some(TeamColor::Green)), None);
        assert_eq!(
            of_shard(&live, 0, Some(TeamColor::Green)),
            None,
            "sin shard"
        );
        assert_eq!(of_shard(&[], 2207, None), None);
    }

    #[test]
    fn rf03a_the_colour_is_reported_even_when_it_does_not_match() {
        // El shard está, pero el juego dice otro color: se devuelve igual (es donde está el
        // jugador) marcando que no cuadra, en vez de esconder el dato.
        let live = vec![entry("2-3", &[2206, 12010], &[2205], &[2204])];
        let found = of_shard(&live, 2206, Some(TeamColor::Red)).expect("se encuentra");
        assert_eq!(found.match_id, "2-3");
        assert_eq!(found.colour, TeamColor::Green);
        assert!(!found.colour_confirmed);
    }
}
