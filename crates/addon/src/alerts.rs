//! Alert lines: the events of the session turned into something the widget can draw (T-13).
//!
//! Split out of `worker` because that file was at the size limit and this is a self-contained
//! piece: the types, the building of a line from an event, and which lines the player asked for.
//! The dashboard stays in `worker`; here there are only `impl Dashboard` blocks for the alerts.

use chrono::{DateTime, Utc};
use gw2lurker_core::diff::Event;
use gw2lurker_core::model::{MapRole, TeamColor};

use crate::worker::Dashboard;

/// How many events the dashboard keeps. The widget only shows the last few minutes; the rest is
/// so the log window can show the session.
pub const MAX_ALERTS: usize = 50;

/// How recent a claim found at a baseline has to be to still deserve an alert.
///
/// At start-up the differ reports every objective that is already claimed. Those are not news —
/// showing them made the log look like it never restarted (bug of 2026-10-31) — but one from the
/// last few minutes is something we missed while loading (RF-21).
const LATE_GRACE: chrono::Duration = chrono::Duration::minutes(10);

impl Dashboard {
    pub fn push_alert(&mut self, event: Event) {
        if !matches!(event, Event::Baseline { .. } | Event::Gap { .. }) {
            self.alerts.push(Received {
                event,
                received: Utc::now(),
            });
            if self.alerts.len() > MAX_ALERTS {
                let excess = self.alerts.len() - MAX_ALERTS;
                self.alerts.drain(..excess);
            }
        }
    }

    /// Recent events as ready-to-draw lines, **newest first** (RF-19), already filtered.
    ///
    /// The objective ids are resolved here so the render callback never searches the table,
    /// and the result is owned so the dashboard lock is not held while drawing. `filter`,
    /// `follow` and `current` decide which ones the player asked for (RF-19a).
    pub fn recent_alerts(
        &self,
        filter: &gw2lurker_core::AlertFilter,
        follow: &gw2lurker_core::FollowList,
        current: Option<MapRole>,
        limit: usize,
    ) -> Vec<AlertLine> {
        self.alerts
            .iter()
            .rev()
            .filter_map(|received| self.alert_line(&received.event, received.received))
            // What was already claimed when the addon started is **not** an alert: it made the
            // log look like it never restarted (bugs of 2026-10-31). Those rows stay in the
            // history and in the route, which is where they belong.
            //
            // The only exception is a followed guild that claimed **minutes ago**: that is not
            // yesterday's news, it is something we missed while loading, which is exactly what
            // following a guild is for (RF-21).
            .filter(|line| {
                !line.late
                    || (line
                        .guild
                        .as_deref()
                        .is_some_and(|guild| follow.contains(guild))
                        && line.received.signed_duration_since(line.at) < LATE_GRACE)
            })
            .filter(|line| {
                filter.notify(
                    line.map_role,
                    line.guild.as_deref(),
                    follow,
                    current,
                    line.objective_kind,
                )
            })
            .take(limit)
            .collect()
    }

    fn alert_line(&self, event: &Event, received: DateTime<Utc>) -> Option<AlertLine> {
        let name = |id: &str| {
            self.objective_name(id)
                .map(str::to_owned)
                .unwrap_or_else(|| id.to_owned())
        };
        // Objective ids are `"<map id>-<n>"`, and `map_roles` maps that id to the matchup role
        // the alert filter talks about.
        let role = |id: &str| {
            id.split_once('-')
                .and_then(|(map, _)| map.parse::<u32>().ok())
                .map(|map_id| {
                    self.map_roles
                        .get(&map_id)
                        .copied()
                        .unwrap_or_else(|| MapRole::from_map_id(map_id))
                })
                .unwrap_or(MapRole::Unknown)
        };
        match event {
            Event::Flip {
                objective,
                from,
                to,
                at,
            } => Some(AlertLine {
                at: *at,
                received,
                map_role: role(objective),
                objective_kind: self.objective_kind(objective),
                kind: AlertKind::Flip,
                objective: name(objective),
                guild: None,
                guild_name: None,
                from: *from,
                to: *to,
                late: false,
            }),
            Event::Claim {
                objective,
                guild,
                at,
                late,
            } => Some(AlertLine {
                at: *at,
                received,
                map_role: role(objective),
                objective_kind: self.objective_kind(objective),
                kind: AlertKind::Claim,
                objective: name(objective),
                guild: Some(guild.clone()),
                guild_name: self.guild_names.get(guild).cloned(),
                from: None,
                to: None,
                late: *late,
            }),
            Event::Unclaim {
                objective,
                guild,
                at,
            } => Some(AlertLine {
                at: *at,
                received,
                map_role: role(objective),
                objective_kind: self.objective_kind(objective),
                kind: AlertKind::Unclaim,
                objective: name(objective),
                guild: Some(guild.clone()),
                guild_name: self.guild_names.get(guild).cloned(),
                from: None,
                to: None,
                late: false,
            }),
            Event::Baseline { .. } | Event::Gap { .. } | Event::MatchChanged { .. } => None,
        }
    }
}

/// What happened, for the alert widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertKind {
    Flip,
    Claim,
    Unclaim,
}

/// One line of the alert widget.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlertLine {
    /// When the game says it happened.
    pub at: DateTime<Utc>,
    /// When this addon learnt about it; the pop-up timer runs from here (RF-19a).
    pub received: DateTime<Utc>,
    /// Map of the matchup the objective belongs to, for the alert filter.
    pub map_role: MapRole,
    /// Camp, tower, keep or castle: the drawing that goes ahead of the line (T-47).
    pub objective_kind: gw2lurker_core::ObjectiveType,
    pub kind: AlertKind,
    pub objective: String,
    pub guild: Option<String>,
    /// `[TAG] Nombre` of [`Self::guild`] when it is known (RF-20). `guild` stays the raw id
    /// because that is what following a guild compares against.
    pub guild_name: Option<String>,
    pub from: Option<TeamColor>,
    pub to: Option<TeamColor>,
    /// Discovered at a baseline instead of seen live (RF-25).
    pub late: bool,
}

/// An event and the instant this addon learnt about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Received {
    pub event: Event,
    /// When the poll that found it finished; the pop-up timer runs from here (RF-19a).
    pub received: DateTime<Utc>,
}
