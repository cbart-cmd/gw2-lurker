//! When the addon may draw and poll (RF-01, RF-02, RF-02a).
//!
//! Pure decision logic: the caller feeds what Mumble and Nexus say and gets back
//! whether to show the overlay and whether to hit the network. Keeping it pure means
//! the loading-screen and character-select cases are testable without the game.

/// `map_type` values that identify a WvW map (RF-01).
pub mod map_type {
    /// Eternal Battlegrounds.
    pub const ETERNAL_BATTLEGROUNDS: u32 = 9;
    /// Blue borderlands.
    pub const BLUE_BORDERLANDS: u32 = 10;
    /// Green borderlands.
    pub const GREEN_BORDERLANDS: u32 = 11;
    /// Red borderlands.
    pub const RED_BORDERLANDS: u32 = 12;
    /// Edge of the Mists.
    pub const EDGE_OF_THE_MISTS: u32 = 15;

    /// Not a real map yet; the client is redirecting.
    pub const AUTO_REDIRECT: u32 = 0;
    /// Character creation.
    pub const CHARACTER_CREATION: u32 = 1;
    /// Fortune's Vale (reward map): has no objectives to watch.
    pub const WVW_REWARD: u32 = 13;
    /// Obsidian Sanctum: jump puzzle, no objectives.
    pub const WVW_OBSIDIAN_SANCTUM: u32 = 14;
    /// Armistice Bastion: lounge, no objectives.
    pub const WVW_LOUNGE: u32 = 18;
}

/// What the client currently reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GameState {
    /// `NexusLink::is_gameplay`. `None` when Nexus has not provided a link yet.
    pub is_gameplay: Option<bool>,
    /// `Context::map_id`; `0` means "no map".
    pub map_id: u32,
    /// `Context::map_type`.
    pub map_type: u32,
}

/// Why the overlay is hidden.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hidden {
    /// Loading screen or character select (RF-02a).
    NotGameplay,
    /// No map at all yet.
    NoMap,
    /// Character creation / redirect.
    Transition,
    /// Anywhere else: PvE, cities, PvP.
    OutsideWvw,
}

/// Decision for the current frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gate {
    /// Whether the alert widget and the map may be drawn.
    pub show_ui: bool,
    /// Whether the worker may poll the API.
    pub poll: bool,
    /// Why it is hidden, when it is.
    pub hidden: Option<Hidden>,
}

impl Gate {
    const SHOWN: Self = Self {
        show_ui: true,
        poll: true,
        hidden: None,
    };

    fn hidden(reason: Hidden) -> Self {
        Self {
            show_ui: false,
            poll: false,
            hidden: Some(reason),
        }
    }

    /// Whether the settings window may still be opened (RF-02 keeps it available).
    pub fn settings_available(&self) -> bool {
        !matches!(self.hidden, Some(Hidden::NotGameplay))
    }
}

/// Whether Mumble reports one of the WvW map types, including modes without matchup objectives.
pub fn is_wvw_map_type(kind: u32) -> bool {
    matches!(
        kind,
        map_type::ETERNAL_BATTLEGROUNDS
            | map_type::BLUE_BORDERLANDS
            | map_type::GREEN_BORDERLANDS
            | map_type::RED_BORDERLANDS
            | map_type::WVW_REWARD
            | map_type::WVW_OBSIDIAN_SANCTUM
            | map_type::EDGE_OF_THE_MISTS
            | map_type::WVW_LOUNGE
    )
}

/// Evaluates the gate. Order matters: the loading screen wins over any map data.
pub fn evaluate(state: GameState) -> Gate {
    if state.is_gameplay == Some(false) {
        return Gate::hidden(Hidden::NotGameplay);
    }
    if state.map_id == 0 {
        return Gate::hidden(Hidden::NoMap);
    }
    if matches!(
        state.map_type,
        map_type::AUTO_REDIRECT | map_type::CHARACTER_CREATION
    ) {
        return Gate::hidden(Hidden::Transition);
    }
    if is_wvw_map_type(state.map_type) {
        Gate::SHOWN
    } else {
        Gate::hidden(Hidden::OutsideWvw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wvw(map_type: u32) -> GameState {
        GameState {
            is_gameplay: Some(true),
            map_id: 38,
            map_type,
        }
    }

    #[test]
    fn rf01_playable_wvw_maps_show_and_poll() {
        for t in [
            map_type::ETERNAL_BATTLEGROUNDS,
            map_type::BLUE_BORDERLANDS,
            map_type::GREEN_BORDERLANDS,
            map_type::RED_BORDERLANDS,
            map_type::WVW_REWARD,
            map_type::WVW_OBSIDIAN_SANCTUM,
            map_type::EDGE_OF_THE_MISTS,
            map_type::WVW_LOUNGE,
        ] {
            let g = evaluate(wvw(t));
            assert!(g.show_ui && g.poll, "map_type {t} must be watched");
            assert_eq!(g.hidden, None);
        }
    }

    #[test]
    fn rf02a_loading_screen_wins_over_map_data() {
        let g = evaluate(GameState {
            is_gameplay: Some(false),
            map_id: 38,
            map_type: map_type::ETERNAL_BATTLEGROUNDS,
        });
        assert!(!g.show_ui && !g.poll);
        assert_eq!(g.hidden, Some(Hidden::NotGameplay));
        assert!(!g.settings_available(), "en carga no se debe abrir nada");
    }

    #[test]
    fn rf02a_character_select_and_redirect_are_transitions() {
        for (map_id, map_type) in [
            (0, map_type::AUTO_REDIRECT),
            (1, map_type::CHARACTER_CREATION),
        ] {
            let g = evaluate(GameState {
                is_gameplay: Some(true),
                map_id,
                map_type,
            });
            assert!(!g.show_ui && !g.poll);
            assert!(matches!(
                g.hidden,
                Some(Hidden::NoMap) | Some(Hidden::Transition)
            ));
        }
    }

    #[test]
    fn rf02a_without_any_data_nothing_is_drawn() {
        let g = evaluate(GameState::default());
        assert!(!g.show_ui && !g.poll);
        assert_eq!(g.hidden, Some(Hidden::NoMap));
    }

    #[test]
    fn rf02_outside_wvw_hides_but_allows_settings() {
        let g = evaluate(GameState {
            is_gameplay: Some(true),
            map_id: 50,
            map_type: 5, // PvE
        });
        assert!(!g.show_ui && !g.poll);
        assert_eq!(g.hidden, Some(Hidden::OutsideWvw));
        assert!(
            g.settings_available(),
            "la configuración sigue accesible (RF-02)"
        );
    }

    #[test]
    fn rf01_the_lounge_shows_and_polls_like_other_wvw_maps() {
        let g = evaluate(wvw(map_type::WVW_LOUNGE));
        assert!(g.show_ui && g.poll);
        assert!(g.settings_available());
        assert_eq!(g.hidden, None);
    }

    #[test]
    fn rf01_beta_seared_borderlands_1652_is_watched_by_map_type() {
        let g = evaluate(GameState {
            is_gameplay: Some(true),
            map_id: 1652,
            map_type: map_type::RED_BORDERLANDS,
        });
        assert!(g.show_ui && g.poll);
        assert_eq!(g.hidden, None);
    }
}
