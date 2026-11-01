//! Which captures deserve an alert (RF-19, RF-19a).
//!
//! The player decides what is interesting: a set of maps (or "the map I am on right now") plus
//! the guilds they follow. Everything starts selected, because the default should be "tell me
//! everything" and not "tell me nothing until you find the setting".
//!
//! Pure logic: no clock, no drawing, no Nexus. The caller passes the current map role, so the
//! "only my map" option can change meaning as the player walks between borderlands.

use crate::follow::FollowList;
use crate::model::{MapRole, ObjectiveType};

/// Which structures deserve an alert.
///
/// The player asked for **structures only** (2026-11-01): «no me avises cuando alguien capture
/// ruinas, solo quiero campamentos, torres, fortalezas / garri / piedraniebla». Ruins, mercenary
/// camps and bloodlust are left out — no guild can claim them, and their owner changes were
/// drowning the log.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Structures {
    pub camp: bool,
    pub tower: bool,
    pub keep: bool,
    pub castle: bool,
}

impl Default for Structures {
    fn default() -> Self {
        Self {
            camp: true,
            tower: true,
            keep: true,
            castle: true,
        }
    }
}

impl Structures {
    /// Whether a change of owner of this kind of objective is worth an alert.
    ///
    /// An unknown kind is let through on purpose: the table of objectives arrives once per
    /// session and until then every objective is unknown, so dropping it would silence the
    /// addon instead of filtering it.
    pub fn wants(self, kind: ObjectiveType) -> bool {
        match kind {
            ObjectiveType::Camp => self.camp,
            ObjectiveType::Tower => self.tower,
            ObjectiveType::Keep => self.keep,
            ObjectiveType::Castle => self.castle,
            ObjectiveType::Unknown => true,
            ObjectiveType::Ruins
            | ObjectiveType::Bloodlust
            | ObjectiveType::Mercenary
            | ObjectiveType::Spawn
            | ObjectiveType::Resource
            | ObjectiveType::Generic => false,
        }
    }
}

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
    /// Only the guilds being followed, **everywhere**: the map boxes are ignored.
    ///
    /// It is the single switch the player asked for (2026-10-31, «solo ver avisos de clanes que
    /// sigo»). Ticking the four maps plus `followed` is a union, so asking for «only mine» meant
    /// unticking five boxes one by one.
    pub only_followed: bool,
    /// Which structures deserve an alert. Ruins never do (request of 2026-11-01).
    pub structures: Structures,
}

impl Default for Filter {
    /// Por defecto se avisa de **lo del mapa en el que estás y de los clanes que sigues**, con las
    /// cuatro estructuras marcadas (petición del jugador, 2026-11-01). Antes venían marcados también
    /// los cuatro mapas fijos, y eso llenaba la lista de cosas de mapas en los que el jugador no
    /// estaba: quien quiera seguirlos los marca, y quien quiera solo sus clanes tiene
    /// [`Self::only_followed`].
    fn default() -> Self {
        Self {
            ebg: false,
            blue: false,
            green: false,
            red: false,
            current_map: true,
            followed: true,
            only_followed: false,
            structures: Structures::default(),
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
    /// that is the whole point of following one. `kind` is the objective: ruins and the rest of
    /// the scenery never raise an alert, whatever the map boxes say.
    pub fn notify(
        &self,
        role: MapRole,
        guild: Option<&str>,
        follow: &FollowList,
        current: Option<MapRole>,
        kind: ObjectiveType,
    ) -> bool {
        // Lo primero: sin estructura no hay aviso. Las ruinas fueron el ruido que el jugador
        // pidió quitar (2026-11-01).
        if !self.structures.wants(kind) {
            return false;
        }
        // «Solo los clanes que sigo» manda sobre los mapas: es lo que el jugador pidió con una
        // casilla, sin tener que destildar las otras cinco (2026-10-31).
        if self.only_followed {
            return guild.is_some_and(|guild| follow.contains(guild));
        }
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
    fn rf19a_by_default_only_my_map_and_the_guilds_i_follow() {
        // Petición del jugador (2026-11-01): «avisar de» arranca con el mapa en el que estoy y los
        // clanes que sigo, más las cuatro estructuras. Los mapas fijos vienen **desmarcados**.
        let filter = Filter::default();
        let follow = FollowList::default();
        assert!(filter.current_map, "el mapa en el que estoy, marcado");
        assert!(filter.followed, "los clanes que sigo, marcado");
        for role in [
            MapRole::Center,
            MapRole::BlueHome,
            MapRole::GreenHome,
            MapRole::RedHome,
        ] {
            assert!(!filter.wants_role(role), "{role:?} no se marca por defecto");
            assert!(
                !filter.notify(role, Some("OTRA"), &follow, None, ObjectiveType::Camp),
                "sin estar en ese mapa y sin seguir al clan, no avisa: {role:?}"
            );
        }
        // Donde sí estoy, avisa.
        assert!(filter.notify(
            MapRole::Center,
            Some("OTRA"),
            &follow,
            Some(MapRole::Center),
            ObjectiveType::Camp
        ));
        // Y las cuatro estructuras vienen marcadas.
        for kind in [
            ObjectiveType::Camp,
            ObjectiveType::Tower,
            ObjectiveType::Keep,
            ObjectiveType::Castle,
        ] {
            assert!(filter.structures.wants(kind), "{kind:?} avisa por defecto");
        }
    }

    #[test]
    fn rf19_unticking_a_map_silences_only_that_map() {
        // Los cuatro mapas marcados a mano: el defecto ya no los trae (RF-19a, 2026-11-01).
        let filter = Filter {
            ebg: false,
            blue: true,
            green: true,
            red: true,
            current_map: false,
            ..Filter::default()
        };
        let follow = FollowList::default();
        assert!(!filter.notify(
            MapRole::Center,
            Some("OTRA"),
            &follow,
            None,
            ObjectiveType::Camp
        ));
        assert!(filter.notify(
            MapRole::BlueHome,
            Some("OTRA"),
            &follow,
            None,
            ObjectiveType::Camp
        ));
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
            only_followed: false,
            structures: Structures::default(),
        };
        let follow = FollowList::default();

        assert!(
            filter.notify(
                MapRole::GreenHome,
                None,
                &follow,
                Some(MapRole::GreenHome),
                ObjectiveType::Tower
            ),
            "estoy en alpino verde"
        );
        assert!(
            !filter.notify(
                MapRole::BlueHome,
                None,
                &follow,
                Some(MapRole::GreenHome),
                ObjectiveType::Tower
            ),
            "y no me interesa el azul"
        );
        assert!(
            !filter.notify(
                MapRole::GreenHome,
                None,
                &follow,
                None,
                ObjectiveType::Tower
            ),
            "fuera de WvW no hay mapa actual"
        );
    }

    #[test]
    fn rf19a_only_the_followed_guilds_is_one_switch() {
        // Petición del jugador (2026-10-31): «solo ver avisos de clanes que sigo», sin tener que
        // destildar los cuatro mapas y el mapa actual una a una.
        let filter = Filter {
            ebg: true,
            blue: true,
            green: true,
            red: true,
            current_map: true,
            followed: true,
            only_followed: true,
            structures: Structures::default(),
        };
        let follow = follow();

        assert!(
            filter.notify(
                MapRole::Center,
                Some("1DB147F0"),
                &follow,
                Some(MapRole::Center),
                ObjectiveType::Keep
            ),
            "el clan seguido sigue avisando"
        );
        assert!(
            !filter.notify(
                MapRole::Center,
                Some("OTRA"),
                &follow,
                Some(MapRole::Center),
                ObjectiveType::Keep
            ),
            "un clan que no sigo no avisa, aunque su mapa esté marcado"
        );
        assert!(
            !filter.notify(
                MapRole::RedHome,
                None,
                &follow,
                Some(MapRole::RedHome),
                ObjectiveType::Keep
            ),
            "los cambios de dueño sin clan tampoco"
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
            only_followed: false,
            structures: Structures::default(),
        };
        let follow = follow();

        for role in [
            MapRole::Center,
            MapRole::BlueHome,
            MapRole::GreenHome,
            MapRole::RedHome,
        ] {
            assert!(
                filter.notify(role, Some("1DB147F0"), &follow, None, ObjectiveType::Camp),
                "el clan seguido avisa en {role:?}"
            );
        }
        assert!(
            !filter.notify(
                MapRole::Center,
                Some("OTRA"),
                &follow,
                None,
                ObjectiveType::Camp
            ),
            "un clan que no sigo no avisa si no he marcado su mapa"
        );
        assert!(
            !filter.notify(MapRole::Center, None, &follow, None, ObjectiveType::Camp),
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
        assert!(!filter.notify(
            MapRole::Center,
            Some("1DB147F0"),
            &follow(),
            None,
            ObjectiveType::Camp
        ));
    }

    #[test]
    fn rf19_unknown_map_never_alerts() {
        let filter = Filter::default();
        assert!(!filter.notify(
            MapRole::Unknown,
            None,
            &FollowList::default(),
            None,
            ObjectiveType::Camp
        ));
    }

    #[test]
    fn rf19e_ruins_never_alert() {
        // Petición del jugador (2026-11-01): «no me avises cuando alguien capture ruinas, solo
        // quiero campamentos, torres, fortalezas / garri / piedraniebla». Todas las casillas
        // marcadas, y aun así las ruinas no avisan.
        let filter = Filter::default();
        let follow = follow();
        for kind in [
            ObjectiveType::Ruins,
            ObjectiveType::Bloodlust,
            ObjectiveType::Mercenary,
            ObjectiveType::Spawn,
        ] {
            assert!(
                !filter.notify(
                    MapRole::Center,
                    Some("1DB147F0"),
                    &follow,
                    Some(MapRole::Center),
                    kind
                ),
                "{kind:?} no es una estructura con reclamo: no debe avisar"
            );
        }
        for kind in [
            ObjectiveType::Camp,
            ObjectiveType::Tower,
            ObjectiveType::Keep,
            ObjectiveType::Castle,
        ] {
            assert!(
                filter.notify(
                    MapRole::Center,
                    Some("1DB147F0"),
                    &follow,
                    Some(MapRole::Center),
                    kind
                ),
                "{kind:?} sí avisa"
            );
        }
    }

    #[test]
    fn rf19e_each_structure_can_be_turned_off() {
        let filter = Filter {
            structures: Structures {
                camp: false,
                ..Structures::default()
            },
            ..Filter::default()
        };
        let follow = follow();
        assert!(!filter.notify(
            MapRole::Center,
            None,
            &follow,
            Some(MapRole::Center),
            ObjectiveType::Camp
        ));
        assert!(filter.notify(
            MapRole::Center,
            None,
            &follow,
            Some(MapRole::Center),
            ObjectiveType::Tower
        ));
    }

    #[test]
    fn rf19e_an_unknown_kind_is_let_through() {
        // La tabla de objetivos llega una vez por sesión; hasta entonces todo es `Unknown`. Si se
        // descartara, el addon se quedaría mudo en vez de filtrar.
        let filter = Filter::default();
        assert!(filter.notify(
            MapRole::Center,
            Some("OTRA"),
            &FollowList::default(),
            Some(MapRole::Center),
            ObjectiveType::Unknown
        ));
    }
}
