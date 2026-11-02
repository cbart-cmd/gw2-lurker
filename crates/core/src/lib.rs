//! Pure domain logic: snapshots, diff, rules, guild following and history queries.
//!
//! No Nexus, no ImGui, no network and no file system access.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

pub mod alerts;
pub mod detect;
pub mod diff;
pub mod follow;
pub mod gate;
pub mod i18n;
pub mod matchup;
pub mod model;
pub mod project;
pub mod prompt;
pub mod resolver;
pub mod route;
pub mod select;
pub mod teams;

pub use alerts::Filter as AlertFilter;
pub use diff::{BaselineReason, DiffConfig, Differ, Event, Snapshot};
pub use follow::{favourites_first, normalize_guild_id, FollowList, MAX_FOLLOWED};
pub use gate::{evaluate, GameState, Gate, Hidden};
pub use i18n::Language;
pub use matchup::{by_map, claims, rank_guilds, split_by_map, ClaimRow, GuildSummary};
pub use model::{
    MapRole, Match, MatchMap, MatchObjective, ObjectiveInfo, ObjectiveType, Region, TeamColor,
    TeamScores, WvwMap, HIGH_VALUE_TICK,
};
pub use project::{project, Frame};
pub use resolver::Resolution;
pub use route::{build as build_route, Origin as RouteOrigin, Route, Step as RouteStep};
pub use select::{
    candidates, evaluate as evaluate_team, evaluate_for, is_team, matchups_of, of_region_and_color,
    rivals, teams_of, Candidate, Roster, Selection,
};
pub use teams::{bundled, TeamEntry, TeamTable};
