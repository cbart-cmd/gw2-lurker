//! Map navigation rules, independent of Nexus and imgui (RF-30, RF-71).

use gw2lurker_core::MapRole;

use super::TAB_MAP;

pub(super) fn pending_tab_after_draw(shown: usize, requested: usize) -> usize {
    if shown == requested {
        usize::MAX
    } else {
        requested
    }
}

/// ¿Toca saltar al mapa del jugador al dibujar la pestaña *Mapa*?
///
/// `shown` es la pestaña dibujada este fotograma, `last` la del anterior, `jump_frame` el fotograma
/// del último clic de clan que pidió el mapa (`None` si no hubo) y `frame` el actual. Se salta
/// **solo** al entrar desde otra pestaña y **solo** si no se viene de clicar un clan en ese mismo
/// momento: la ruta de un clan se dibuja donde reclamó, no en el mapa del jugador.
///
/// La ventana es de un fotograma a propósito. Antes se miraba «¿queda una pestaña pedida?», y esa
/// marca se quedaba viva mientras imgui tardase en cambiar de pestaña: al volver a *Mapa* a mano
/// parecía que se venía de un clic de clan y no se saltaba al mapa del jugador (informe del
/// jugador, 2026-11-01).
pub(super) fn entering_by_hand(
    shown: usize,
    last: usize,
    jump_frame: Option<u64>,
    frame: u64,
) -> bool {
    let just_jumped = jump_frame.is_some_and(|at| frame <= at.saturating_add(1));
    shown == TAB_MAP && last != TAB_MAP && !just_jumped
}

/// Clan cuya ruta dibuja la pestaña *Mapa*, si hay alguno elegido.
///
/// Estado propio de esa pestaña y no el de *Clanes*: entrar a mano deja el desplegable vacío para
/// ver todo lo que reclaman los clanes que sigues, sin borrar el clan abierto en la otra pestaña.
fn guild_choice_state() -> std::sync::MutexGuard<'static, Option<String>> {
    static CHOICE: std::sync::OnceLock<std::sync::Mutex<Option<String>>> =
        std::sync::OnceLock::new();
    CHOICE
        .get_or_init(|| std::sync::Mutex::new(None))
        .lock()
        .unwrap_or_else(|error| error.into_inner())
}

/// Clan cuya ruta dibuja la pestaña *Mapa*, si hay alguno elegido.
pub(super) fn guild_choice() -> Option<String> {
    guild_choice_state().clone()
}

/// Fija o vacía el clan de la pestaña *Mapa* (`None` = sin ruta, solo el anillo de los seguidos).
pub(super) fn set_guild_choice(guild: Option<&str>) {
    *guild_choice_state() = guild.map(str::to_owned);
}

/// Lo elegido en el desplegable de clan → clan a dibujar.
///
/// La entrada «No guild» del desplegable llega como cadena vacía, que es no elegir ninguno: sin
/// ruta y con el anillo de los clanes seguidos como única marca.
pub(super) fn guild_from_pick(pick: &str) -> Option<String> {
    if pick.is_empty() {
        None
    } else {
        Some(pick.to_owned())
    }
}

pub(super) fn request_guild_map(
    guild: &str,
    target: Option<MapRole>,
    map: &mut MapRole,
    requested: &mut usize,
    notice: &mut Option<String>,
) -> bool {
    let Some(role) = target.filter(|role| *role != MapRole::Unknown) else {
        *requested = usize::MAX;
        *notice = Some(guild.to_owned());
        return false;
    };
    *map = role;
    *requested = TAB_MAP;
    *notice = None;
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rf30_a_guild_jump_waits_until_the_map_tab_is_drawn() {
        let pending = pending_tab_after_draw(2, TAB_MAP);
        assert_eq!(
            pending, TAB_MAP,
            "imgui has not drawn the requested tab yet"
        );
        assert!(
            !entering_by_hand(TAB_MAP, 2, Some(7), 8),
            "llegar por un clic de clan no salta al mapa del jugador"
        );
        assert_eq!(pending_tab_after_draw(TAB_MAP, pending), usize::MAX);
        assert_eq!(pending_tab_after_draw(2, usize::MAX), usize::MAX);
    }

    #[test]
    fn rf30_returning_to_the_map_tab_by_hand_snaps_to_the_players_map() {
        // Informe del jugador (2026-11-01): tras un clic de clan, cambiar de pestaña y volver a
        // *Mapa* se quedaba en el mapa del clan. Un clic del fotograma 7 no puede mandar en el 37.
        assert!(
            !entering_by_hand(TAB_MAP, 2, Some(7), 8),
            "el fotograma del salto dibuja el mapa del clan"
        );
        assert!(
            entering_by_hand(TAB_MAP, 2, Some(7), 37),
            "volver a mano tiene que saltar al mapa del jugador"
        );
        assert!(
            entering_by_hand(TAB_MAP, 0, None, 37),
            "sin clic de clan, siempre"
        );
        assert!(
            !entering_by_hand(TAB_MAP, TAB_MAP, None, 37),
            "quedarse en la pestaña no cambia el mapa elegido a mano"
        );
        assert!(!entering_by_hand(3, 0, None, 37), "solo la pestaña Mapa");
    }

    #[test]
    fn rf71_a_guild_without_events_keeps_the_tab_and_map() {
        let mut map = MapRole::BlueHome;
        let mut requested = usize::MAX;
        let mut notice = None;
        assert!(!request_guild_map(
            "NO-EVENTS",
            None,
            &mut map,
            &mut requested,
            &mut notice,
        ));
        assert_eq!(map, MapRole::BlueHome);
        assert_eq!(requested, usize::MAX, "do not request the Map tab");
        assert_eq!(notice.as_deref(), Some("NO-EVENTS"));
    }

    #[test]
    fn rf71_a_guild_jump_chooses_its_map_and_clears_the_notice() {
        let mut map = MapRole::Center;
        let mut requested = usize::MAX;
        let mut notice = Some("NO-EVENTS".to_owned());
        assert!(request_guild_map(
            "HAS-EVENTS",
            Some(MapRole::RedHome),
            &mut map,
            &mut requested,
            &mut notice,
        ));
        assert_eq!(map, MapRole::RedHome);
        assert_eq!(requested, TAB_MAP);
        assert_eq!(notice, None);
    }

    #[test]
    fn rf71_an_empty_guild_cancels_an_earlier_pending_jump() {
        let mut map = MapRole::Center;
        let mut requested = TAB_MAP;
        let mut notice = None;
        assert!(!request_guild_map(
            "NO-EVENTS",
            None,
            &mut map,
            &mut requested,
            &mut notice,
        ));
        assert_eq!(requested, usize::MAX);
        assert_eq!(map, MapRole::Center);
        assert_eq!(notice.as_deref(), Some("NO-EVENTS"));
    }

    #[test]
    fn rf32a_a_non_favourite_guild_opens_its_last_claim_map_with_a_route() {
        let entry: gw2lurker_core::Match = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../api/tests/fixtures/match_2-2.json"
        )))
        .expect("real matchup fixture");
        let rows = gw2lurker_core::matchup::claims(&entry, &[]);
        let latest = rows
            .iter()
            .max_by_key(|row| row.claimed_at)
            .expect("claims");
        let guild = latest.guild.clone();
        let expected_map = latest.map_role;
        let followed = gw2lurker_core::FollowList::default();
        assert!(!followed.contains(&guild), "the guild is not a favourite");
        let board = crate::worker::Dashboard {
            history: rows,
            ..Default::default()
        };
        let here = if expected_map == MapRole::Center {
            MapRole::BlueHome
        } else {
            MapRole::Center
        };
        let mut map = here;
        let mut requested = usize::MAX;
        let mut notice = None;
        assert!(request_guild_map(
            &guild,
            board.last_map_of(&guild),
            &mut map,
            &mut requested,
            &mut notice,
        ));
        requested = pending_tab_after_draw(2, requested);
        assert!(
            !entering_by_hand(TAB_MAP, 2, Some(7), 8),
            "el salto del clan manda en ese fotograma"
        );
        assert_eq!(map, expected_map, "not the player's map");
        assert!(super::super::map::summary(
            &board.guild_history(&guild),
            &guild,
            gw2lurker_core::i18n::Language::En,
        )
        .is_some());
        assert_eq!(pending_tab_after_draw(TAB_MAP, requested), usize::MAX);
    }

    #[test]
    fn rf71_entering_the_map_tab_by_hand_leaves_no_guild_chosen() {
        set_guild_choice(Some("1DB147F0"));
        assert_eq!(guild_choice().as_deref(), Some("1DB147F0"));
        // El snap de entrada (mod.rs) vacía la elección: mapa del jugador y sin ruta.
        set_guild_choice(None);
        assert_eq!(guild_choice(), None);
        assert_eq!(
            guild_from_pick(""),
            None,
            "la entrada «No guild» del desplegable no dibuja ruta"
        );
        assert_eq!(guild_from_pick("1DB147F0").as_deref(), Some("1DB147F0"));
    }
}
