//! HTTP client for the public GW2 API and its response parsing.
//!
//! The transport is a trait so the HTTP client itself (ureq, TLS choice, size budget) can
//! be decided by the T-02b measurement without touching parsing or the retry policy.
//! Everything here is testable against the fixtures in `crates/api/tests/fixtures`.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

use gw2lurker_core::model::{Match, ObjectiveInfo};
use serde::Deserialize;

pub mod cache;

pub use cache::Cache;

/// Base URL of the public API.
pub const BASE: &str = "https://api.guildwars2.com/v2";

/// Public guild data.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, serde::Serialize)]
pub struct GuildInfo {
    pub id: String,
    pub name: String,
    pub tag: String,
}

/// What can go wrong while talking to the API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiError {
    /// The HTTP client could not complete the request.
    Transport(String),
    /// Non-success status. 503 means "the API is having a bad day", retry later.
    Status(u16),
    /// The body did not match the expected shape.
    Parse(String),
    /// The requested entity is not part of the current data set (a team without match).
    NotFound,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transport(detail) => write!(f, "transporte: {detail}"),
            Self::Status(code) => write!(f, "estado HTTP {code}"),
            Self::Parse(detail) => write!(f, "respuesta ilegible: {detail}"),
            Self::NotFound => write!(f, "no existe"),
        }
    }
}

impl std::error::Error for ApiError {}

/// Minimal HTTP surface. Implementations only fetch bytes; parsing lives above.
pub trait Transport {
    /// Performs a GET and returns the raw body.
    fn get(&self, url: &str) -> Result<Vec<u8>, ApiError>;
}

/// API client over any [`Transport`].
#[derive(Debug)]
pub struct Api<T: Transport> {
    transport: T,
}

impl<T: Transport> Api<T> {
    /// Wraps a transport.
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    /// Access to the underlying transport (tests, diagnostics).
    pub fn transport(&self) -> &T {
        &self.transport
    }

    /// Every match of the region, from `/v2/wvw/matches?ids=all`.
    pub fn matches_all(&self) -> Result<Vec<Match>, ApiError> {
        let body = self.transport.get(&format!("{BASE}/wvw/matches?ids=all"))?;
        parse(&body)
    }

    /// Match that contains `team_id`.
    ///
    /// Scan `ids=all` instead of using `/wvw/matches?world=<id>`: that parameter takes a
    /// legacy world id, while the addon identifies teams by the newer 11xxx/12xxx ids
    /// that live in `all_worlds` (ADR-010). Scanning is verified; the parameter is not.
    pub fn match_for_team(&self, team_id: u32) -> Result<Match, ApiError> {
        let matches = self.matches_all()?;
        matches
            .into_iter()
            .find(|m| m.color_of(team_id).is_some())
            .ok_or(ApiError::NotFound)
    }

    /// A single match by id, from `/v2/wvw/matches/{id}`.
    ///
    /// The worker polls this once it knows the player's match: ~44 KB instead of the ~400 KB
    /// of `ids=all`, which matters at one poll every 30 s (RNF-05).
    pub fn match_by_id(&self, id: &str) -> Result<Match, ApiError> {
        let body = self.transport.get(&format!("{BASE}/wvw/matches/{id}"))?;
        parse(&body)
    }

    /// Static objective metadata in the given language (RF-44).
    pub fn objectives(&self, lang: &str) -> Result<Vec<ObjectiveInfo>, ApiError> {
        let body = self
            .transport
            .get(&format!("{BASE}/wvw/objectives?ids=all&lang={lang}"))?;
        parse(&body)
    }

    /// Public guild data.
    pub fn guild(&self, id: &str) -> Result<GuildInfo, ApiError> {
        let body = self.transport.get(&format!("{BASE}/guild/{id}"))?;
        parse(&body)
    }

    /// Ids of the guilds whose name contains `text` (T-44).
    ///
    /// `/v2/guild/search` needs no API key and only matches on the name, so the caller still has
    /// to resolve each id with [`Self::guild`] to get the tag and to know which team it plays for.
    pub fn guild_search(&self, text: &str) -> Result<Vec<String>, ApiError> {
        let body = self
            .transport
            .get(&format!("{BASE}/guild/search?name={}", url_encode(text)))?;
        parse(&body)
    }

    /// Guild id → team id for a region, the official source of "which team is this guild on".
    ///
    /// It is a big answer (~280 KB for EU) that only changes at reset, so the caller has to cache
    /// it instead of asking per keystroke.
    pub fn wvw_guilds(
        &self,
        region: gw2lurker_core::Region,
    ) -> Result<std::collections::BTreeMap<String, u32>, ApiError> {
        let body = self.transport.get(&format!(
            "{BASE}/wvw/guilds/{}",
            match region {
                gw2lurker_core::Region::Eu => "eu",
                gw2lurker_core::Region::Na => "na",
            }
        ))?;
        parse(&body)
    }

    /// The four WvW maps with the boxes the projection needs (RF-30, ADR-011).
    ///
    /// Asking for the four ids instead of `ids=all` keeps the answer at a couple of kilobytes;
    /// the map list never changes, so the worker fetches it once per session.
    pub fn maps(&self) -> Result<Vec<gw2lurker_core::WvwMap>, ApiError> {
        let body = self
            .transport
            .get(&format!("{BASE}/maps?ids=38,95,96,1099"))?;
        parse(&body)
    }
}

/// Percent-encodes a query value: guild names have spaces, accents and apostrophes.
fn url_encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for byte in text.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

fn parse<T: for<'de> Deserialize<'de>>(body: &[u8]) -> Result<T, ApiError> {
    serde_json::from_slice(body).map_err(|e| ApiError::Parse(e.to_string()))
}

/// Exponential retry with a cap, shared by the polling worker (RNF-04).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Backoff {
    base_ms: u64,
    cap_ms: u64,
    attempt: u32,
}

impl Default for Backoff {
    fn default() -> Self {
        Self::new(2_000, 5 * 60 * 1_000)
    }
}

impl Backoff {
    /// Creates a backoff with the given base and cap, both in milliseconds.
    pub fn new(base_ms: u64, cap_ms: u64) -> Self {
        Self {
            base_ms: base_ms.max(1),
            cap_ms: cap_ms.max(base_ms.max(1)),
            attempt: 0,
        }
    }

    /// Resets after a successful request.
    pub fn reset(&mut self) {
        self.attempt = 0;
    }

    /// Current attempt number (0 after a reset).
    pub fn attempts(&self) -> u32 {
        self.attempt
    }

    /// Delay for the next retry, recording the failure.
    pub fn next_delay_ms(&mut self) -> u64 {
        let factor = 1u64.checked_shl(self.attempt.min(16)).unwrap_or(u64::MAX);
        self.attempt = self.attempt.saturating_add(1);
        self.base_ms.saturating_mul(factor).min(self.cap_ms)
    }

    /// Whether the error deserves a retry: network failures and 5xx do, 4xx do not.
    pub fn should_retry(error: &ApiError) -> bool {
        match error {
            ApiError::Transport(_) => true,
            ApiError::Status(code) => *code >= 500 || *code == 429,
            ApiError::Parse(_) | ApiError::NotFound => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;

    const MATCHES: &[u8] = include_bytes!("../tests/fixtures/matches_all.json");
    const MATCH: &[u8] = include_bytes!("../tests/fixtures/match_2-2.json");
    const OBJECTIVES: &[u8] = include_bytes!("../tests/fixtures/objectives_all.json");
    const GUILD: &[u8] = include_bytes!("../tests/fixtures/guild.json");
    const MAPS: &[u8] = include_bytes!("../tests/fixtures/maps_wvw.json");

    /// Transport that answers from a table of URLs and records what was asked.
    #[derive(Default)]
    struct Fake {
        routes: HashMap<String, Result<Vec<u8>, ApiError>>,
        asked: RefCell<Vec<String>>,
    }

    impl Fake {
        fn with(url: &str, body: &[u8]) -> Self {
            let mut fake = Self::default();
            fake.routes.insert(url.to_string(), Ok(body.to_vec()));
            fake
        }

        fn failing(url: &str, error: ApiError) -> Self {
            let mut fake = Self::default();
            fake.routes.insert(url.to_string(), Err(error));
            fake
        }
    }

    impl Transport for Fake {
        fn get(&self, url: &str) -> Result<Vec<u8>, ApiError> {
            self.asked.borrow_mut().push(url.to_string());
            self.routes
                .get(url)
                .cloned()
                .unwrap_or(Err(ApiError::Status(404)))
        }
    }

    #[test]
    fn t11b_matches_are_parsed_from_the_real_payload() {
        let api = Api::new(Fake::with(&format!("{BASE}/wvw/matches?ids=all"), MATCHES));
        let matches = api.matches_all().unwrap();
        assert_eq!(matches.len(), 1, "el fixture es un array con el match 2-2");
    }

    #[test]
    fn adr010_team_is_found_inside_all_worlds() {
        let api = Api::new(Fake::with(&format!("{BASE}/wvw/matches?ids=all"), MATCHES));
        let m = api.match_for_team(12014).unwrap();
        assert_eq!(m.id, "2-2");
        assert_eq!(m.color_of(12014), Some(gw2lurker_core::TeamColor::Blue));
    }

    #[test]
    fn adr010_unknown_team_is_not_found() {
        let api = Api::new(Fake::with(&format!("{BASE}/wvw/matches?ids=all"), MATCHES));
        assert_eq!(api.match_for_team(999_999), Err(ApiError::NotFound));
    }

    #[test]
    fn t11b_single_match_fixture_parses() {
        let m: Match = parse(MATCH).unwrap();
        assert_eq!(m.id, "2-2");
        assert_eq!(m.objectives_total(), 91);
    }

    #[test]
    fn t11b_single_match_is_fetched_by_id() {
        let url = format!("{BASE}/wvw/matches/2-2");
        let api = Api::new(Fake::with(&url, MATCH));
        let m = api.match_by_id("2-2").unwrap();
        assert_eq!(m.id, "2-2");
        assert_eq!(api.transport().asked.borrow().len(), 1, "una sola petición");
    }

    #[test]
    fn rf44_objectives_are_requested_in_the_asked_language() {
        let url = format!("{BASE}/wvw/objectives?ids=all&lang=fr");
        let api = Api::new(Fake::with(&url, OBJECTIVES));
        let all = api.objectives("fr").unwrap();
        assert!(all.len() > 100);
        assert_eq!(api.transport().asked.borrow().as_slice(), [url]);
    }

    #[test]
    fn rf20_guild_returns_name_and_tag() {
        let id = "230C9E95-E509-4B3E-AAF2-A320DB7506E5";
        let api = Api::new(Fake::with(&format!("{BASE}/guild/{id}"), GUILD));
        let g = api.guild(id).unwrap();
        assert_eq!(g.tag, "LAX");
        assert!(!g.name.is_empty());
    }

    #[test]
    fn rf30_maps_come_with_the_boxes_of_the_projection() {
        let url = format!("{BASE}/maps?ids=38,95,96,1099");
        let api = Api::new(Fake::with(&url, MAPS));
        let maps = api.maps().unwrap();

        assert!(
            maps.len() >= 4,
            "la fixture trae los cuatro mapas de MvM y algo más"
        );
        for id in [38, 95, 96, 1099] {
            assert!(
                maps.iter().any(|map| map.id == id),
                "falta el mapa {id} en la respuesta"
            );
        }
        let ebg = maps.iter().find(|map| map.id == 38).unwrap();
        assert_eq!(ebg.continent_rect, [[8958.0, 12798.0], [12030.0, 15870.0]]);
        assert_eq!(api.transport().asked.borrow().as_slice(), [url]);
    }

    #[test]
    fn rnf04_http_503_is_reported_as_status() {
        let api = Api::new(Fake::failing(
            &format!("{BASE}/wvw/matches?ids=all"),
            ApiError::Status(503),
        ));
        assert_eq!(api.matches_all(), Err(ApiError::Status(503)));
        assert!(Backoff::should_retry(&ApiError::Status(503)));
        assert!(!Backoff::should_retry(&ApiError::Status(404)));
        assert!(Backoff::should_retry(&ApiError::Transport(
            "timeout".into()
        )));
        assert!(!Backoff::should_retry(&ApiError::Parse("x".into())));
    }

    #[test]
    fn rnf04_truncated_json_is_a_parse_error() {
        let api = Api::new(Fake::with(
            &format!("{BASE}/wvw/matches?ids=all"),
            b"[{\"id\":",
        ));
        assert!(matches!(api.matches_all(), Err(ApiError::Parse(_))));
    }

    #[test]
    fn rnf04_backoff_grows_and_caps() {
        let mut b = Backoff::new(1_000, 8_000);
        assert_eq!(b.next_delay_ms(), 1_000);
        assert_eq!(b.next_delay_ms(), 2_000);
        assert_eq!(b.next_delay_ms(), 4_000);
        assert_eq!(b.next_delay_ms(), 8_000);
        assert_eq!(b.next_delay_ms(), 8_000, "no pasa del tope");
        assert_eq!(b.attempts(), 5);

        b.reset();
        assert_eq!(b.attempts(), 0);
        assert_eq!(b.next_delay_ms(), 1_000);
    }
}
