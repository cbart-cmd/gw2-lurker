//! Reads what Nexus and Mumble expose. The only place with `unsafe`.

// The transport is only reachable from the polling worker, which is T-11c; until it exists
// the whole subtree is "dead code" for the compiler.
#[allow(dead_code)]
pub mod http;
pub mod keys;

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
/// which is exactly the granularity we need. Empty or unavailable names return `None`.
pub fn character_name() -> Option<String> {
    let link = get_mumble_link()?;
    let raw = link.read_identity_string();
    let text = raw.to_string_lossy().trim().to_owned();
    (!text.is_empty()).then_some(text)
}
