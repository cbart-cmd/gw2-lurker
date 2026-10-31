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
/// which is exactly the granularity we need. The plain `name` field is the character name;
/// the `identity` field holds a JSON blob, which is only used as a fallback because the game
/// has been seen publishing an empty `name` while `identity` still has it.
pub fn character_name() -> Option<String> {
    let link = get_mumble_link()?;

    let name = link.read_name_string().to_string_lossy().trim().to_owned();
    if !name.is_empty() {
        return Some(name);
    }

    let identity = link.read_identity_string().to_string_lossy().into_owned();
    identity_name(&identity)
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
}
