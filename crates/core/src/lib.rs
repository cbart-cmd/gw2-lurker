//! Pure domain logic: snapshots, diff, rules, guild following and history queries.
//!
//! No Nexus, no ImGui, no network and no file system access.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
