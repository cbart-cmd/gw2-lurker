//! Which captures deserve an alert (RF-19, RF-19a).
//!
//! The player decides what is interesting: a set of maps (or "the map I am on right now") plus
//! the guilds they follow. Everything starts selected, because the default should be "tell me
//! everything" and not "tell me nothing until you find the setting".
//!
//! Pure logic: no clock, no drawing, no Nexus. The caller passes the current map role, so the
//! "only my map" option can change meaning as the player walks between borderlands.

use crate::follow::FollowList;
use crate::model::MapRole;

/// What the player wants to be told about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Filter {
    /// Eternal Battlegrounds (Eternos).
    pub ebg: bool,
    /// Blue alpine borderlands (Alpino azul).
    pub blue: bool,
    /// Green alpine borderlands (Alpino verde).
    pub green: bool,
    /// Red desert borderlands (Desierto).
    pub red: bool,
    /// Whatever map the player is standing on right now.
    pub current_map: bool,
    /// Every map, but only for the guilds being followed.
    pub followed: bool,
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            ebg: true,
            blue: true,
            green: true,
            red: true,
            current_map: true,
            followed: true,
        }
    }
}

impl Filter {
    /// Whether the fixed-map option for this role is ticked.
    pub fn wants_role(&self, role: MapRole) -> bool {
        match role {
            MapRole::Center => self.ebg,
            MapRole::BlueHome => self.blue,
            MapRole::GreenHome => self.green,
            MapRole::RedHome => self.red,
            MapRole::Unknown => false,
        }
    }

    /// Whether a capture on `role` by `guild` is worth an alert.
    ///
    /// `current` is the role the player is standing on, or `None` outside WvW. `follow` is the
    /// list of followed guilds. A followed guild is interesting **everywhere** in the matchup;
    /// that is the whole point of following one.
    pub fn notify(
        &self,
        role: MapRole,
        guild: Option<&str>,
        follow: &FollowList,
        current: Option<MapRole>,
    ) -> bool {
        if self.followed {
            if let Some(guild) = guild {
                if follow.contains(guild) {
                    return true;
                }
            }
        }
        if self.current_map && current == Some(role) && role != MapRole::Unknown {
            return true;
        }
        role != MapRole::Unknown && self.wants_role(role)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn follow() -> FollowList {
        let mut list = FollowList::default();
        list.add("1DB147F0");
        list
    }

    #[test]
    fn rf19_by_default_everything_is_told() {
        let filter = Filter::default();
        let follow = FollowList::default();
        for role in [
            MapRole::Center,
            MapRole::BlueHome,
            MapRole::GreenHome,
            MapRole::RedHome,
        ] {
            assert!(
                filter.notify(role, Some("OTRA"), &follow, None),
                "{role:?} debería avisar por defecto"
            );
        }
    }

    #[test]
    fn rf19_unticking_a_map_silences_only_that_map() {
        let filter = Filter {
            ebg: false,
            current_map: false,
            ..Filter::default()
        };
        let follow = FollowList::default();
        assert!(!filter.notify(MapRole::Center, Some("OTRA"), &follow, None));
        assert!(filter.notify(MapRole::BlueHome, Some("OTRA"), &follow, None));
    }

    #[test]
    fn rf19a_the_map_i_am_on_can_be_the_only_source() {
        let filter = Filter {
            ebg: false,
            blue: false,
            green: false,
            red: false,
            current_map: true,
            followed: false,
        };
        let follow = FollowList::default();

        assert!(
            filter.notify(MapRole::GreenHome, None, &follow, Some(MapRole::GreenHome)),
            "estoy en alpino verde"
        );
        assert!(
            !filter.notify(MapRole::BlueHome, None, &follow, Some(MapRole::GreenHome)),
            "y no me interesa el azul"
        );
        assert!(
            !filter.notify(MapRole::GreenHome, None, &follow, None),
            "fuera de WvW no hay mapa actual"
        );
    }

    #[test]
    fn rf73_a_followed_guild_is_interesting_on_every_map() {
        let filter = Filter {
            ebg: false,
            blue: false,
            green: false,
            red: false,
            current_map: false,
            followed: true,
        };
        let follow = follow();

        for role in [
            MapRole::Center,
            MapRole::BlueHome,
            MapRole::GreenHome,
            MapRole::RedHome,
        ] {
            assert!(
                filter.notify(role, Some("1DB147F0"), &follow, None),
                "el clan seguido avisa en {role:?}"
            );
        }
        assert!(
            !filter.notify(MapRole::Center, Some("OTRA"), &follow, None),
            "un clan que no sigo no avisa si no he marcado su mapa"
        );
        assert!(
            !filter.notify(MapRole::Center, None, &follow, None),
            "una captura sin clan tampoco"
        );
    }

    #[test]
    fn rf73_unticking_followed_guilds_silences_them() {
        let filter = Filter {
            followed: false,
            ebg: false,
            current_map: false,
            ..Filter::default()
        };
        assert!(!filter.notify(MapRole::Center, Some("1DB147F0"), &follow(), None));
    }

    #[test]
    fn rf19_unknown_map_never_alerts() {
        let filter = Filter::default();
        assert!(!filter.notify(MapRole::Unknown, None, &FollowList::default(), None));
    }
}
