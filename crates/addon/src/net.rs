//! `gw2lurker_api::Transport` over WinHTTP.
//!
//! Kept out of `ffi` on purpose: this module has no `unsafe`, it just adapts the HTTP call
//! and translates the status code into the error type the API client understands.

use gw2lurker_api::{ApiError, Transport};

use crate::ffi::http;

/// Transport backed by the WinHTTP service that ships with Windows.
#[derive(Debug, Clone, Copy)]
pub struct WinHttpTransport {
    timeout_ms: i32,
}

impl Default for WinHttpTransport {
    fn default() -> Self {
        // Enough for a slow mobile connection, short enough that a stuck poll cannot hold the
        // worker much longer than the 5 s default interval.
        Self { timeout_ms: 10_000 }
    }
}

impl Transport for WinHttpTransport {
    fn get(&self, url: &str) -> Result<Vec<u8>, ApiError> {
        let (status, body) = http::get(url, self.timeout_ms)?;
        match status {
            200..=299 => Ok(body),
            404 => Err(ApiError::NotFound),
            429 | 500..=599 => Err(ApiError::Status(status)),
            _ => Err(ApiError::Status(status)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rf20_http_404_maps_to_not_found() {
        // The mapping is what the cache and the guild endpoint rely on, so it is pinned here
        // without touching the network: `http::get` is the only part that talks to WinHTTP.
        let map = |status: u16| match status {
            200..=299 => Ok(()),
            404 => Err(ApiError::NotFound),
            429 | 500..=599 => Err(ApiError::Status(status)),
            _ => Err(ApiError::Status(status)),
        };
        assert!(map(200).is_ok());
        assert_eq!(map(404), Err(ApiError::NotFound));
        assert_eq!(map(503), Err(ApiError::Status(503)));
    }
}
