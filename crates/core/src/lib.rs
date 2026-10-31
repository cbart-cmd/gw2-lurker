//! Pure domain logic: snapshots, diff, rules, guild following and history queries.
//!
//! No Nexus, no ImGui, no network and no file system access.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

pub mod model;
pub mod resolver;
pub mod teams;

pub use model::{
    MapRole, Match, MatchMap, MatchObjective, ObjectiveInfo, ObjectiveType, Region, TeamColor,
    TeamScores, WvwMap, HIGH_VALUE_TICK,
};
pub use resolver::Resolution;
pub use teams::TeamTable;
