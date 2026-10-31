//! HTTP client for the public GW2 API and its response parsing.
//!
//! No Nexus and no ImGui; see [`gw2lurker_core`] for the domain types.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
