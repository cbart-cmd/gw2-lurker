use chrono::{DateTime, Utc};
use gw2lurker_core::model::TeamColor;

use crate::worker::{AlertKind, AlertLine};

const MAX_POPUP_LINES: usize = 5;

#[derive(Clone)]
pub(super) struct PopupLine {
    pub line: AlertLine,
    pub fade: f32,
}

#[derive(Clone, Default)]
pub(super) struct PopupSnapshot {
    pub lines: Vec<PopupLine>,
    pub overflow: usize,
}

#[derive(Clone)]
struct ActiveLine {
    line: AlertLine,
    expires_at: DateTime<Utc>,
    priority: u8,
}

#[derive(Clone, PartialEq, Eq)]
struct Identity {
    at: DateTime<Utc>,
    received: DateTime<Utc>,
    objective_id: String,
    kind: AlertKind,
    guild: Option<String>,
    from: Option<TeamColor>,
    to: Option<TeamColor>,
}

impl From<&AlertLine> for Identity {
    fn from(line: &AlertLine) -> Self {
        Self {
            at: line.at,
            received: line.received,
            objective_id: line.objective_id.clone(),
            kind: line.kind,
            guild: line.guild.clone(),
            from: line.from,
            to: line.to,
        }
    }
}

#[derive(Default)]
pub(super) struct PopupStack {
    active: Vec<ActiveLine>,
    seen: std::collections::VecDeque<Identity>,
    overflow: usize,
}

impl PopupStack {
    pub(super) fn tick(
        &mut self,
        lines: &[AlertLine],
        now: DateTime<Utc>,
        secs: f32,
        followed: &gw2lurker_core::FollowList,
        states: &[gw2lurker_core::matchup::ObjectiveState],
        player_team: Option<TeamColor>,
    ) -> PopupSnapshot {
        let duration = (secs.is_finite() && secs > 0.0)
            .then(|| chrono::Duration::milliseconds((f64::from(secs) * 1000.0).ceil() as i64));
        if duration.is_none() {
            self.active.clear();
            self.overflow = 0;
        }
        self.active.retain(|line| now < line.expires_at);
        if self.active.is_empty() {
            self.overflow = 0;
        }

        // The dashboard already applies RF-25, including the recent followed-guild exception.
        let mut incoming: Vec<&AlertLine> = lines.iter().collect();
        incoming.sort_by_key(|line| line.received);
        for line in incoming {
            let identity = Identity::from(line);
            if self.seen.contains(&identity) {
                continue;
            }
            self.seen.push_back(identity);
            while self.seen.len() > crate::alerts::MAX_ALERTS {
                self.seen.pop_front();
            }

            let Some(duration) = duration else {
                continue;
            };
            let expires_at = line.received + duration;
            if now >= expires_at {
                continue;
            }

            self.active.push(ActiveLine {
                line: line.clone(),
                expires_at,
                priority: priority(line, followed, states, player_team),
            });
            self.active.sort_by(|left, right| {
                right
                    .priority
                    .cmp(&left.priority)
                    .then_with(|| right.line.received.cmp(&left.line.received))
            });
            if self.active.len() > MAX_POPUP_LINES {
                self.active.pop();
                self.overflow += 1;
            }
        }

        PopupSnapshot {
            lines: self
                .active
                .iter()
                .map(|active| {
                    let left = active.expires_at.signed_duration_since(now);
                    PopupLine {
                        line: active.line.clone(),
                        fade: (left.num_milliseconds() as f32 / 500.0).clamp(0.0, 1.0),
                    }
                })
                .collect(),
            overflow: self.overflow,
        }
    }
}

fn priority(
    line: &AlertLine,
    followed: &gw2lurker_core::FollowList,
    states: &[gw2lurker_core::matchup::ObjectiveState],
    player_team: Option<TeamColor>,
) -> u8 {
    if line
        .guild
        .as_deref()
        .is_some_and(|guild| followed.contains(guild))
    {
        return 4;
    }
    if states
        .iter()
        .find(|state| state.objective_id == line.objective_id)
        .is_some_and(gw2lurker_core::matchup::ObjectiveState::is_high_value)
    {
        return 3;
    }
    match (line.kind, player_team) {
        (AlertKind::Flip, Some(team)) if line.from == Some(team) => 2,
        (AlertKind::Flip, Some(team)) if line.to == Some(team) => 1,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn line(kind: AlertKind, to: Option<TeamColor>, guild: Option<&str>) -> AlertLine {
        let at = Utc.with_ymd_and_hms(2026, 10, 31, 12, 0, 0).unwrap();
        AlertLine {
            at,
            received: at,
            map_role: gw2lurker_core::MapRole::Center,
            objective_id: "38-1".to_owned(),
            objective_kind: gw2lurker_core::ObjectiveType::Tower,
            kind,
            objective: "Mendon's Gap".to_owned(),
            guild: guild.map(str::to_owned),
            guild_name: guild.map(|tag| format!("[{tag}]")),
            from: Some(TeamColor::Red),
            to,
            late: false,
        }
    }

    #[test]
    fn rf19d_alerts_stack_and_expire_independently() {
        let base = Utc.with_ymd_and_hms(2026, 10, 31, 12, 0, 0).unwrap();
        let mut first = line(AlertKind::Claim, None, Some("YaK"));
        first.received = base;
        first.at = base;
        let mut second = line(AlertKind::Claim, None, Some("Otra"));
        second.received = base + chrono::Duration::milliseconds(200);
        second.at = second.received;
        let lines = vec![second.clone(), first.clone()];
        let followed = gw2lurker_core::FollowList::default();
        let mut stack = PopupStack::default();

        let first_only = stack.tick(&[first], base, 15.0, &followed, &[], None);
        assert_eq!(first_only.lines.len(), 1);
        let both = stack.tick(
            &lines,
            base + chrono::Duration::milliseconds(200),
            15.0,
            &followed,
            &[],
            None,
        );
        assert_eq!(both.lines.len(), 2, "both show without waiting in a queue");
        assert_eq!(both.lines[0].line.guild.as_deref(), Some("Otra"));

        let one = stack.tick(
            &lines,
            base + chrono::Duration::seconds(15),
            15.0,
            &followed,
            &[],
            None,
        );
        assert_eq!(one.lines.len(), 1);
        assert_eq!(one.lines[0].line.guild.as_deref(), Some("Otra"));
        assert!(stack
            .tick(
                &lines,
                base + chrono::Duration::seconds(15) + chrono::Duration::milliseconds(200),
                15.0,
                &followed,
                &[],
                None,
            )
            .lines
            .is_empty());
    }

    #[test]
    fn rf19d_priority_is_followed_then_valuable_then_lost_then_taken() {
        let base = Utc.with_ymd_and_hms(2026, 10, 31, 12, 0, 0).unwrap();
        let mut alerts = vec![
            line(AlertKind::Claim, None, Some("YaK")),
            line(AlertKind::Flip, Some(TeamColor::Red), None),
            line(AlertKind::Flip, Some(TeamColor::Red), None),
            line(AlertKind::Flip, Some(TeamColor::Green), None),
            line(AlertKind::Claim, None, None),
        ];
        for (index, alert) in alerts.iter_mut().enumerate() {
            alert.objective_id = format!("38-{index}");
            alert.received = base + chrono::Duration::seconds(index as i64);
            alert.at = alert.received;
        }
        alerts[1].from = Some(TeamColor::Blue);
        alerts[2].from = Some(TeamColor::Green);
        alerts[3].from = Some(TeamColor::Blue);
        alerts[4].from = Some(TeamColor::Blue);
        alerts[4].to = Some(TeamColor::Red);
        let mut followed = gw2lurker_core::FollowList::default();
        assert!(followed.add("YaK"));
        let states = [gw2lurker_core::matchup::ObjectiveState {
            map_id: 38,
            map_role: gw2lurker_core::MapRole::Center,
            objective_id: alerts[1].objective_id.clone(),
            kind: gw2lurker_core::ObjectiveType::Keep,
            owner: Some(TeamColor::Green),
            guild: None,
            claimed_at: None,
            points_tick: gw2lurker_core::model::HIGH_VALUE_TICK,
        }];
        let mut stack = PopupStack::default();
        let snapshot = stack.tick(
            &alerts,
            base + chrono::Duration::seconds(4),
            15.0,
            &followed,
            &states,
            Some(TeamColor::Green),
        );
        let ids: Vec<&str> = snapshot
            .lines
            .iter()
            .map(|row| row.line.objective_id.as_str())
            .collect();
        assert_eq!(ids, ["38-0", "38-1", "38-2", "38-3", "38-4"]);
    }

    #[test]
    fn rf19d_high_value_lost_replaces_the_oldest_taken_line() {
        let base = Utc.with_ymd_and_hms(2026, 10, 31, 12, 0, 0).unwrap();
        let mut alerts: Vec<AlertLine> = (0..5)
            .map(|index| {
                let mut alert = line(AlertKind::Flip, Some(TeamColor::Green), None);
                alert.objective_id = format!("taken-{index}");
                alert.from = Some(TeamColor::Blue);
                alert.received = base + chrono::Duration::seconds(index);
                alert.at = alert.received;
                alert
            })
            .collect();
        let mut lost = line(AlertKind::Flip, Some(TeamColor::Red), None);
        lost.objective_id = "high-value-lost".to_owned();
        lost.from = Some(TeamColor::Green);
        lost.received = base + chrono::Duration::seconds(5);
        lost.at = lost.received;
        alerts.push(lost);
        let states = [gw2lurker_core::matchup::ObjectiveState {
            map_id: 38,
            map_role: gw2lurker_core::MapRole::Center,
            objective_id: "high-value-lost".to_owned(),
            kind: gw2lurker_core::ObjectiveType::Keep,
            owner: Some(TeamColor::Red),
            guild: None,
            claimed_at: None,
            points_tick: gw2lurker_core::model::HIGH_VALUE_TICK,
        }];
        let mut stack = PopupStack::default();
        let snapshot = stack.tick(
            &alerts,
            base + chrono::Duration::seconds(5),
            15.0,
            &gw2lurker_core::FollowList::default(),
            &states,
            Some(TeamColor::Green),
        );

        assert_eq!(snapshot.lines.len(), MAX_POPUP_LINES);
        assert_eq!(
            snapshot.lines[0].line.objective_id, "high-value-lost",
            "el perdido de alto valor asciende"
        );
        assert!(
            snapshot
                .lines
                .iter()
                .all(|row| row.line.objective_id != "taken-0"),
            "sale la captura tomada más antigua"
        );
        assert_eq!(snapshot.overflow, 1);
    }

    #[test]
    fn rf19d_discarded_lines_are_counted_and_never_queued() {
        let base = Utc.with_ymd_and_hms(2026, 10, 31, 12, 0, 0).unwrap();
        let lines: Vec<AlertLine> = (0..6)
            .map(|index| {
                let mut alert = line(AlertKind::Claim, None, None);
                alert.objective_id = format!("38-{index}");
                alert.received = base + chrono::Duration::seconds(index);
                alert.at = alert.received;
                alert
            })
            .collect();
        let followed = gw2lurker_core::FollowList::default();
        let mut stack = PopupStack::default();
        let full = stack.tick(
            &lines,
            base + chrono::Duration::seconds(5),
            15.0,
            &followed,
            &[],
            None,
        );
        assert_eq!(full.lines.len(), MAX_POPUP_LINES);
        assert_eq!(full.overflow, 1);
        assert!(full.lines.iter().all(|row| row.line.objective_id != "38-0"));

        let expired = stack.tick(
            &lines,
            base + chrono::Duration::seconds(20),
            15.0,
            &followed,
            &[],
            None,
        );
        assert!(expired.lines.is_empty());
        assert_eq!(expired.overflow, 0);
        assert!(stack
            .tick(
                &lines,
                base + chrono::Duration::seconds(21),
                15.0,
                &followed,
                &[],
                None,
            )
            .lines
            .is_empty());
    }

    #[test]
    fn rf19d_text_fades_during_the_last_half_second() {
        let base = Utc.with_ymd_and_hms(2026, 10, 31, 12, 0, 0).unwrap();
        let mut alert = line(AlertKind::Flip, Some(TeamColor::Blue), None);
        alert.received = base;
        let mut stack = PopupStack::default();
        let snapshot = stack.tick(
            &[alert],
            base + chrono::Duration::milliseconds(4750),
            5.0,
            &gw2lurker_core::FollowList::default(),
            &[],
            None,
        );
        assert!(snapshot.lines[0].fade > 0.0 && snapshot.lines[0].fade < 1.0);
    }

    #[test]
    fn rf25_r07_only_recent_followed_late_claims_reach_popup() {
        let base = Utc.with_ymd_and_hms(2026, 10, 31, 12, 0, 0).unwrap();
        let received = |objective: &str, guild: &str, age: i64| crate::alerts::Received {
            event: gw2lurker_core::Event::Claim {
                objective: objective.to_owned(),
                guild: guild.to_owned(),
                at: base - chrono::Duration::minutes(age),
                late: true,
            },
            received: base,
        };
        let board = crate::worker::Dashboard {
            alerts: vec![
                received("38-20", "YaK", 2),
                received("38-21", "Otra", 2),
                received("38-22", "YaK", 11),
            ],
            ..Default::default()
        };
        let mut followed = gw2lurker_core::FollowList::default();
        assert!(followed.add("YaK"));
        let alerts = board.recent_alerts(
            &gw2lurker_core::AlertFilter::default(),
            &followed,
            Some(gw2lurker_core::MapRole::Center),
            crate::alerts::MAX_ALERTS,
        );
        assert_eq!(
            alerts.len(),
            1,
            "old and unfollowed late claims are filtered"
        );

        let mut stack = PopupStack::default();
        let snapshot = stack.tick(&alerts, base, 15.0, &followed, &[], None);
        assert_eq!(snapshot.lines.len(), 1);
        assert_eq!(snapshot.lines[0].line.guild.as_deref(), Some("YaK"));
        assert!(snapshot.lines[0].line.late);
        assert!(stack
            .tick(
                &alerts,
                base + chrono::Duration::seconds(15),
                15.0,
                &followed,
                &[],
                None,
            )
            .lines
            .is_empty());
    }
}
