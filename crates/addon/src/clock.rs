//! Clock times as the player sees them.
//!
//! The API sends everything in UTC and the addon stores it that way, which is right: two events
//! that happened at the same instant must compare equal whatever the machine's timezone. What was
//! wrong was **drawing** them: the widget showed UTC hours, so a claim at 21:40 in Spain appeared
//! as 19:40 (report of 2026-10-31). Conversion happens here, at the edge, once.
//!
//! Pure formatting, no clock of its own, so the tests can pin an instant.

use chrono::{DateTime, Local, Utc};

/// `HH:MM` in the player's local time.
pub fn clock(at: DateTime<Utc>) -> String {
    local(at).format("%H:%M").to_string()
}

/// `HH:MM:SS` in the player's local time, for the alert lines where seconds matter.
pub fn clock_seconds(at: DateTime<Utc>) -> String {
    local(at).format("%H:%M:%S").to_string()
}

/// An instant from the API, moved to the machine's timezone.
pub fn local(at: DateTime<Utc>) -> DateTime<Local> {
    at.with_timezone(&Local)
}

/// The same instant built from a UTC timestamp, for the tests.
#[cfg(test)]
fn from_utc(seconds: i64, nanos: u32) -> Option<DateTime<Utc>> {
    use chrono::TimeZone;
    Utc.timestamp_opt(seconds, nanos).single()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    #[test]
    fn rf19_the_clock_uses_the_players_timezone_and_not_utc() {
        // 19:40 UTC. En Madrid (UTC+1/+2) la hora que se enseña no es la de UTC, que era el bug.
        let at = from_utc(1_793_475_600, 0).expect("instante válido");
        let shown = clock(at);
        assert_eq!(shown.len(), 5, "HH:MM");
        assert_eq!(clock_seconds(at).len(), 8, "HH:MM:SS");

        let offset = Local::now().offset().local_minus_utc();
        let expected = at + chrono::Duration::seconds(i64::from(offset));
        assert_eq!(shown, expected.format("%H:%M").to_string());
    }

    #[test]
    fn rf19_the_same_instant_is_formatted_the_same_way_twice() {
        let at = from_utc(1_793_475_600, 0).expect("instante válido");
        assert_eq!(clock(at), clock(at));
        assert_eq!(
            local(at).hour(),
            (at + chrono::Duration::seconds(i64::from(Local::now().offset().local_minus_utc())))
                .hour()
        );
    }
}
