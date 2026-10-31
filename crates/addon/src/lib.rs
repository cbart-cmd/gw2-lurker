//! Raidcore Nexus addon (cdylib).
//!
//! This is the only crate allowed to use `unsafe`, and only inside `ffi/`.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
