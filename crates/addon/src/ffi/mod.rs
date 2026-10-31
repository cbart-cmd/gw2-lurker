//! Reads what Nexus and Mumble expose. The only place with `unsafe`.

pub mod http;
pub mod keys;
pub mod paths;

use nexus::data_link::mumble::get_mumble_link;

/// `NexusLink::is_gameplay`: `false` on loading screens and character select.
///
/// Returns `None` while Nexus has not published a link yet, which the caller treats as
/// "assume gameplay" so the addon never hides itself because of a missing pointer.
pub fn is_gameplay() -> Option<bool> {
    let link = nexus::data_link::get_nexus_link();
    if link.is_null() {
        return None;
    }

    // SAFETY: `get_nexus_link` returns the pointer Nexus publishes or null (checked
    // above). Nexus writes the field from another thread, so it is read volatilely and
    // no reference to the whole struct is created, only to that `bool`.
    Some(unsafe { std::ptr::read_volatile(std::ptr::addr_of!((*link).is_gameplay)) })
}

/// `(map_id, map_type)` from MumbleLink, or `None` when the link is not available.
pub fn mumble_map() -> Option<(u32, u32)> {
    let link = get_mumble_link()?;
    Some((link.read_map_id(), link.read_map_type()))
}

/// Character name from MumbleLink, used to key the per-character team roster (RF-03a).
///
/// Mumble exposes the character, not the account: a second account is a different character,
/// which is exactly the granularity we need.
///
/// The name comes from the **identity JSON**. The `name` field of MumbleLink looks like the
/// character but it is the game's own name: measured on 2026-10-31 it held `"Guild Wars 2"`, and
/// the roster ended up keyed by that (`settings.json` had a `"Guild Wars 2": 11011` entry instead
/// of the character), so the saved team was looked up under a key that is not the character.
pub fn character_name() -> Option<String> {
    let link = get_mumble_link()?;

    let identity = link.read_identity_string().to_string_lossy().into_owned();
    let mumble_name = link.read_name_string().to_string_lossy().into_owned();
    pick_character_name(&mumble_name, &identity)
}

/// The game name MumbleLink publishes in its `name` field, which is not a character.
const GAME_NAME: &str = "Guild Wars 2";

/// Character name out of the two fields Mumble publishes. Pure, so it can be tested: calling the
/// game pointers from a test panics (`AddonApi::get()` is not initialized outside the game).
fn pick_character_name(mumble_name: &str, identity: &str) -> Option<String> {
    // `identity.name` is the character; the field in `name` is only a fallback for a game that
    // stops publishing the JSON, and never the game's own name.
    identity_name(identity).or_else(|| {
        let fallback = mumble_name.trim();
        (!fallback.is_empty() && fallback != GAME_NAME).then(|| fallback.to_owned())
    })
}

/// Character name inside MumbleLink's `identity` JSON. Pure, so it can be tested: calling the
/// game pointers from a test panics (`AddonApi::get()` is not initialized outside the game).
pub fn identity_name(identity: &str) -> Option<String> {
    let start = identity.find("\"name\":\"")? + "\"name\":\"".len();
    let rest = identity.get(start..)?;
    let end = rest.find('"')?;
    let name = rest.get(..end)?.trim();
    (!name.is_empty()).then(|| name.to_owned())
}

/// What the game itself says about where the player fights: `(shard_id, team colour)`.
///
/// `shard_id` used to be the home world and the new teams reuse the same field; `team_color_id`
/// is Mumble's own team colour (1 red, 2 green, 3 blue). Both are only used to **warn** when the
/// team saved in the settings does not look like the one being played, which is the cause behind
/// every «no salen avisos» so far (report of 2026-10-31).
///
/// UNVERIFIED(T-19): the exact meaning of `shard_id` in WvW has not been seen in game yet; the
/// worker logs it so the next session settles it.
pub fn mumble_where() -> Option<(u32, Option<gw2lurker_core::TeamColor>)> {
    let link = get_mumble_link()?;
    let shard = link.read_shard_id();
    let identity = link.read_identity_string().to_string_lossy().into_owned();
    Some((shard, identity_team_colour(&identity)))
}

/// Team colour out of the `identity` JSON. Pure, so it can be tested.
pub fn identity_team_colour(identity: &str) -> Option<gw2lurker_core::TeamColor> {
    use gw2lurker_core::TeamColor;
    let start = identity.find("\"team_color_id\":")? + "\"team_color_id\":".len();
    let rest = identity.get(start..)?;
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    match digits.parse().ok()? {
        1 => Some(TeamColor::Red),
        2 => Some(TeamColor::Green),
        3 => Some(TeamColor::Blue),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rf03a_character_name_comes_out_of_the_identity_json() {
        // Real shape captured from MumbleLink on 2026-10-31.
        let identity = r#"{"name":"Jhoin","profession":6,"spec":80,"race":2,"map_id":1315,"world_id":268435457,"team_color_id":0,"commander":false,"map":1315,"fov":1.222,"uisz":1}"#;
        assert_eq!(identity_name(identity), Some("Jhoin".to_owned()));
    }

    #[test]
    fn rf03a_identity_without_name_is_none() {
        assert_eq!(identity_name(""), None);
        assert_eq!(identity_name(r#"{"profession":6}"#), None);
        assert_eq!(identity_name(r#"{"name":""}"#), None);
    }

    #[test]
    fn rf03a_the_character_comes_from_the_identity_and_not_from_the_game_name() {
        // Lo medido en el juego el 2026-10-31: `name` lleva «Guild Wars 2» y el personaje está
        // en el JSON de identidad. Con el orden al revés, el roster se guardaba como
        // «Guild Wars 2» y el equipo del personaje no se encontraba.
        let identity = r#"{"name":"Jhoin","profession":6,"spec":48,"race":2,"map_id":96,"world_id":2147483649,"team_color_id":9,"commander":false,"map":96,"fov":1.222,"uisz":1}"#;
        assert_eq!(
            pick_character_name("Guild Wars 2", identity),
            Some("Jhoin".to_owned())
        );
        assert_eq!(
            pick_character_name("Guild Wars 2", ""),
            None,
            "el nombre del juego no es un personaje"
        );
        assert_eq!(
            pick_character_name("Jhoin", ""),
            Some("Jhoin".to_owned()),
            "si algún día deja de llegar el JSON, el campo name sigue sirviendo"
        );
        assert_eq!(pick_character_name("  ", ""), None);
    }

    #[test]
    fn rf03a_the_team_colour_comes_out_of_the_identity_json() {
        use gw2lurker_core::TeamColor;
        let red = r#"{"name":"Jhoin","team_color_id":1,"commander":false}"#;
        let green = r#"{"name":"Jhoin","team_color_id":2}"#;
        let blue = r#"{"name":"Jhoin","team_color_id":3}"#;
        assert_eq!(identity_team_colour(red), Some(TeamColor::Red));
        assert_eq!(identity_team_colour(green), Some(TeamColor::Green));
        assert_eq!(identity_team_colour(blue), Some(TeamColor::Blue));
        // Fuera de WvW el juego manda 0 y eso no es un color de equipo.
        assert_eq!(identity_team_colour(r#"{"team_color_id":0}"#), None);
        assert_eq!(identity_team_colour(""), None);
    }
}
