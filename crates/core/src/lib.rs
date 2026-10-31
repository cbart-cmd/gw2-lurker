//! Pure domain logic: snapshots, diff, rules, guild following and history queries.
//!
//! No Nexus, no ImGui, no network and no file system access.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

pub mod diff;
pub mod follow;
pub mod gate;
pub mod i18n;
pub mod matchup;
pub mod model;
pub mod resolver;
pub mod teams;

pub use diff::{BaselineReason, DiffConfig, Differ, Event, Snapshot};
pub use follow::{favourites_first, FollowList};
pub use gate::{evaluate, GameState, Gate, Hidden};
pub use i18n::Language;
pub use matchup::{by_map, claims, rank_guilds, split_by_map, ClaimRow, GuildSummary};
pub use model::{
    MapRole, Match, MatchMap, MatchObjective, ObjectiveInfo, ObjectiveType, Region, TeamColor,
    TeamScores, WvwMap, HIGH_VALUE_TICK,
};
pub use resolver::Resolution;
pub use teams::TeamTable;
