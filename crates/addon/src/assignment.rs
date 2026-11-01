//! Detects scheduled WvW team reassignment without polling global match data every frame.

use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use gw2lurker_api::{Api, ApiError, Transport};
use gw2lurker_core::diff::Differ;

use crate::worker::Dashboard;
const TIMER_TTL: Duration = Duration::from_secs(30 * 60);
const RETRY_AFTER_ERROR: Duration = Duration::from_secs(60);
const OVERVIEW_TTL: Duration = Duration::from_secs(60);

#[derive(Debug, Default)]
pub(crate) struct AssignmentMonitor {
    deadlines: Option<(DateTime<Utc>, DateTime<Utc>)>,
    refresh_after: Option<Instant>,
    overview_after: Option<Instant>,
}

impl AssignmentMonitor {
    pub(crate) fn rotation_confirmed<T: Transport>(
        &mut self,
        api: &Api<T>,
        match_id: &str,
        now: DateTime<Utc>,
        instant: Instant,
    ) -> Result<bool, ApiError> {
        let region = match match_id.split_once('-').map(|(region, _)| region) {
            Some("1") => 0,
            Some("2") => 1,
            _ => return Ok(false),
        };
        self.refresh_timers(api, instant)?;
        let Some((na, eu)) = self.deadlines else {
            return Ok(false);
        };
        let deadline = if region == 0 { na } else { eu };
        if now < deadline || self.overview_after.is_some_and(|at| instant < at) {
            return Ok(false);
        }

        self.overview_after = Some(instant + OVERVIEW_TTL);
        let active = api.matches_overview()?;
        Ok(!active.iter().any(|id| id == match_id))
    }

    pub(crate) fn reset_after_rotation(&mut self) {
        self.deadlines = None;
        self.refresh_after = None;
        self.overview_after = None;
    }

    fn refresh_timers<T: Transport>(
        &mut self,
        api: &Api<T>,
        instant: Instant,
    ) -> Result<(), ApiError> {
        if self.refresh_after.is_some_and(|at| instant < at) {
            return Ok(());
        }

        self.refresh_after = Some(instant + RETRY_AFTER_ERROR);
        let timers = api.team_assignment_timers()?;
        let na = parse_deadline(&timers.na)?;
        let eu = parse_deadline(&timers.eu)?;
        self.deadlines = Some((na, eu));
        self.refresh_after = Some(instant + TIMER_TTL);
        Ok(())
    }
}

pub(crate) fn reset_tracking_for_assignment(
    board: &mut Dashboard,
    differ: &mut Differ,
    assignment: &mut AssignmentMonitor,
    previous_match: Option<&str>,
    current_match: Option<&str>,
    at: DateTime<Utc>,
) -> bool {
    if previous_match == current_match {
        return false;
    }
    differ.reset();
    assignment.reset_after_rotation();
    board.reset_tracking_cycle(at);
    true
}

fn parse_deadline(value: &str) -> Result<DateTime<Utc>, ApiError> {
    DateTime::parse_from_rfc3339(value)
        .map(|date| date.with_timezone(&Utc))
        .map_err(|error| ApiError::Parse(format!("invalid team-assignment timer: {error}")))
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::BTreeMap;

    use chrono::{DateTime, Utc};
    use gw2lurker_api::{Api, ApiError, Transport};

    use super::*;

    const TIMERS: &[u8] = include_bytes!("../../api/tests/fixtures/timers_team_assignment.json");

    #[derive(Default)]
    struct Fake {
        routes: BTreeMap<String, Vec<u8>>,
        asked: RefCell<Vec<String>>,
    }

    impl Fake {
        fn with(url: &str, body: &[u8]) -> Self {
            let mut fake = Self::default();
            fake.routes.insert(url.to_owned(), body.to_vec());
            fake
        }

        fn and(mut self, url: &str, body: &[u8]) -> Self {
            self.routes.insert(url.to_owned(), body.to_vec());
            self
        }
    }

    impl Transport for Fake {
        fn get(&self, url: &str) -> Result<Vec<u8>, ApiError> {
            self.asked.borrow_mut().push(url.to_owned());
            self.routes.get(url).cloned().ok_or(ApiError::NotFound)
        }
    }

    fn at(value: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(value)
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn rf03d_expired_assignment_confirms_rotation_when_match_leaves_overview() {
        let api = Api::new(
            Fake::with(
                "https://api.guildwars2.com/v2/wvw/timers/teamAssignment",
                TIMERS,
            )
            .and(
                "https://api.guildwars2.com/v2/wvw/matches/overview",
                br#"["2-1","2-2","2-3","2-5","1-1"]"#,
            ),
        );
        let mut monitor = AssignmentMonitor::default();

        let changed = monitor
            .rotation_confirmed(&api, "2-4", at("2026-11-06T19:00:00Z"), Instant::now())
            .unwrap();

        assert!(changed);
    }

    #[test]
    fn rf03d_expired_assignment_keeps_tracking_if_match_is_still_live() {
        let api = Api::new(
            Fake::with(
                "https://api.guildwars2.com/v2/wvw/timers/teamAssignment",
                TIMERS,
            )
            .and(
                "https://api.guildwars2.com/v2/wvw/matches/overview",
                br#"["2-1","2-2","2-3","2-4","2-5","1-1"]"#,
            ),
        );
        let mut monitor = AssignmentMonitor::default();

        let changed = monitor
            .rotation_confirmed(&api, "2-4", at("2026-11-06T19:00:00Z"), Instant::now())
            .unwrap();

        assert!(!changed);
    }

    #[test]
    fn rf03d_future_assignment_does_not_poll_matches_overview() {
        let api = Api::new(Fake::with(
            "https://api.guildwars2.com/v2/wvw/timers/teamAssignment",
            TIMERS,
        ));
        let mut monitor = AssignmentMonitor::default();

        assert!(!monitor
            .rotation_confirmed(&api, "2-4", at("2026-11-06T17:00:00Z"), Instant::now(),)
            .unwrap());
        assert_eq!(api.transport().asked.borrow().len(), 1);
    }
}
