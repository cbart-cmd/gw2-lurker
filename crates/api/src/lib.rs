//! HTTP client for the public GW2 API and its response parsing.
//!
//! The transport is a trait: the addon implements it over WinHTTP (`net::WinHttpTransport`),
//! and the tests over the fixtures in `crates/api/tests/fixtures`, so parsing and the retry
//! policy are checked without the network.
//!
//! Every id that ends up inside a URL path or query is checked first ([`looks_like_guild_id`],
//! match ids, language codes): guild ids also come from `guild-names.json`, which is downloaded
//! from a configurable URL, and a `/`, `?` or `..` would change the endpoint requested.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

use gw2lurker_core::model::{Match, ObjectiveInfo};
use serde::Deserialize;

/// Base URL of the public API.
pub const BASE: &str = "https://api.guildwars2.com/v2";

/// Public guild data.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, serde::Serialize)]
pub struct GuildInfo {
    pub id: String,
    pub name: String,
    pub tag: String,
}

/// Regional team-assignment deadlines returned by `/v2/wvw/timers/teamAssignment`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TeamAssignmentTimers {
    pub na: String,
    pub eu: String,
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
            Self::Transport(detail) => write!(f, "transport error: {detail}"),
            Self::Status(code) => write!(f, "HTTP status {code}"),
            Self::Parse(detail) => write!(f, "invalid response: {detail}"),
            Self::NotFound => write!(f, "not found"),
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

    /// Every live match of **both** regions (NA `1-x` and EU `2-x`), from
    /// `/v2/wvw/matches?ids=all` (~400 KB).
    pub fn matches_all(&self) -> Result<Vec<Match>, ApiError> {
        let body = self.transport.get(&format!("{BASE}/wvw/matches?ids=all"))?;
        parse(&body)
    }

    /// Ids of the live matches, from the small `/v2/wvw/matches/overview` response.
    pub fn matches_overview(&self) -> Result<Vec<String>, ApiError> {
        let body = self
            .transport
            .get(&format!("{BASE}/wvw/matches/overview"))?;
        parse(&body)
    }

    /// Team-assignment deadlines for NA and EU.
    pub fn team_assignment_timers(&self) -> Result<TeamAssignmentTimers, ApiError> {
        let body = self
            .transport
            .get(&format!("{BASE}/wvw/timers/teamAssignment"))?;
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
    /// The worker polls this once it knows the player's match: ~44 KB with its four maps instead
    /// of the ~400 KB of `ids=all`. It is the only request made at the configured cadence
    /// (5 s by default, RNF-02).
    pub fn match_by_id(&self, id: &str) -> Result<Match, ApiError> {
        if !looks_like_match_id(id) {
            return Err(ApiError::NotFound);
        }
        let body = self.transport.get(&format!("{BASE}/wvw/matches/{id}"))?;
        parse(&body)
    }

    /// Static objective metadata in the given language (RF-44).
    pub fn objectives(&self, lang: &str) -> Result<Vec<ObjectiveInfo>, ApiError> {
        if !looks_like_language(lang) {
            return Err(ApiError::NotFound);
        }
        let body = self
            .transport
            .get(&format!("{BASE}/wvw/objectives?ids=all&lang={lang}"))?;
        parse(&body)
    }

    /// Public guild data.
    pub fn guild(&self, id: &str) -> Result<GuildInfo, ApiError> {
        let id = gw2lurker_core::normalize_guild_id(id).ok_or(ApiError::NotFound)?;
        let body = self.transport.get(&format!("{BASE}/guild/{id}"))?;
        parse(&body)
    }

    /// Ids of the guilds whose name **starts with** `text` (T-44).
    ///
    /// Measured on 2026-10-31/11-01: a prefix of the full name, at most one guild, and `[]` with
    /// HTTP 200 when nothing matches (not a 404). No API key is needed; the caller still resolves
    /// each id with [`Self::guild`] to get the tag and to know which team it plays for.
    pub fn guild_search(&self, text: &str) -> Result<Vec<String>, ApiError> {
        let body = self
            .transport
            .get(&format!("{BASE}/guild/search?name={}", url_encode(text)))?;
        let ids: Vec<String> = parse(&body)?;
        Ok(ids
            .into_iter()
            .map(|id| gw2lurker_core::normalize_guild_id(&id).unwrap_or(id))
            .collect())
    }

    /// Guild id → team id for a region, the official source of "which team is this guild on".
    ///
    /// It is a big answer (~280 KB for EU) that only changes at reset, so the caller has to cache
    /// it instead of asking per keystroke.
    ///
    /// The API sends the team as a **string**: `{ "0006EDDA-…": "12008" }` (measured 2026-10-31).
    /// Parsing straight into `u32` gave `invalid type: string "12008"`, which is the error the
    /// player saw in the guild search, so the value is read as text and converted — and a plain
    /// number is still accepted in case it flips back.
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
        let raw: std::collections::BTreeMap<String, TeamRef> = parse(&body)?;
        Ok(raw
            .into_iter()
            .filter_map(|(guild, team)| {
                let guild = gw2lurker_core::normalize_guild_id(&guild).unwrap_or(guild);
                team.into_id().map(|id| (guild, id))
            })
            .collect())
    }

    /// WvW map geometries used by the matchup and beta borderlands (RF-30, ADR-011).
    ///
    /// Asking for known ids instead of `ids=all` keeps the answer small; the map list never
    /// changes, so the worker fetches it once per session.
    pub fn maps(&self) -> Result<Vec<gw2lurker_core::WvwMap>, ApiError> {
        let body = self
            .transport
            .get(&format!("{BASE}/maps?ids=38,95,96,1099,1652"))?;
        parse(&body)
    }

    /// Legacy world id → name, to say which old worlds fight for each team (RF-03a).
    ///
    /// The new teams have names nobody has played under, so the picker shows the old world names
    /// next to them: that is what the player recognises (report of 2026-10-31). `ids=all` is ~51
    /// entries, a few kilobytes, and it never changes.
    pub fn worlds(&self) -> Result<std::collections::BTreeMap<u32, String>, ApiError> {
        let body = self.transport.get(&format!("{BASE}/worlds?ids=all"))?;
        let worlds: Vec<World> = parse(&body)?;
        Ok(worlds
            .into_iter()
            .map(|world| (world.id, world.name))
            .collect())
    }
}

/// One entry of `/v2/worlds`: the old servers, still in `all_worlds` for compatibility.
#[derive(Debug, Clone, serde::Deserialize)]
struct World {
    id: u32,
    #[serde(default)]
    name: String,
}

/// The team of a guild in `/v2/wvw/guilds/:region`, which arrives as text.
///
/// `12008` today, a bare number if the API ever goes back to it; anything else is ignored instead
/// of failing the whole 280 KB answer over one guild.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(untagged)]
enum TeamRef {
    Number(u32),
    Text(String),
}

impl TeamRef {
    fn into_id(self) -> Option<u32> {
        match self {
            Self::Number(id) => Some(id),
            Self::Text(text) => text.trim().parse().ok(),
        }
    }
}

/// Whether a string has the shape of a guild id: 32 hexadecimal digits, with or without dashes.
///
/// `/v2/guild/<id>` answers with an opaque error for anything else, and a `/` or `?` would change
/// the request, so the shape is checked before spending one on it.
pub fn looks_like_guild_id(id: &str) -> bool {
    gw2lurker_core::normalize_guild_id(id).is_some()
}

/// `<region>-<tier>`, like `2-2`.
fn looks_like_match_id(id: &str) -> bool {
    let Some((region, tier)) = id.split_once('-') else {
        return false;
    };
    let digits = |part: &str| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit());
    matches!(region, "1" | "2") && digits(tier)
}

/// Language supported by the addon: `es`, `en`, `fr` or `de`.
fn looks_like_language(lang: &str) -> bool {
    matches!(lang, "es" | "en" | "fr" | "de")
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
    const MATCHES_ALL_LIVE: &[u8] = include_bytes!("../tests/fixtures/matches_all_20261102.json");
    const MATCH: &[u8] = include_bytes!("../tests/fixtures/match_2-2.json");
    const OBJECTIVES: &[u8] = include_bytes!("../tests/fixtures/objectives_all.json");
    const GUILD: &[u8] = include_bytes!("../tests/fixtures/guild.json");
    const MAPS: &[u8] = include_bytes!("../tests/fixtures/maps_wvw.json");
    const TEAM_ASSIGNMENT: &[u8] = include_bytes!("../tests/fixtures/timers_team_assignment.json");
    const MATCHES_OVERVIEW: &[u8] = include_bytes!("../tests/fixtures/matches_overview.json");

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
                .unwrap_or(Err(ApiError::NotFound))
        }
    }

    #[test]
    fn t11b_matches_are_parsed_from_the_real_payload() {
        let api = Api::new(Fake::with(&format!("{BASE}/wvw/matches?ids=all"), MATCHES));
        let matches = api.matches_all().unwrap();
        assert_eq!(matches.len(), 1, "el fixture es un array con el match 2-2");
    }

    #[test]
    fn rf03a_live_ids_all_fixture_has_nine_unique_matches() {
        let api = Api::new(Fake::with(
            &format!("{BASE}/wvw/matches?ids=all"),
            MATCHES_ALL_LIVE,
        ));
        let matches = api.matches_all().unwrap();
        let ids: std::collections::BTreeSet<_> =
            matches.iter().map(|entry| entry.id.as_str()).collect();

        assert_eq!(matches.len(), 9);
        assert_eq!(ids.len(), matches.len(), "cada match tiene un id único");
        assert!(matches
            .iter()
            .any(|entry| entry.region() == Some(gw2lurker_core::Region::Eu)));
        assert!(matches
            .iter()
            .any(|entry| entry.region() == Some(gw2lurker_core::Region::Na)));
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
    fn t44_guild_search_encodes_spaces_ampersands_and_utf8() {
        let url = format!("{BASE}/guild/search?name=Die+%26+G%C3%A4ng");
        let api = Api::new(Fake::with(&url, b"[]"));
        assert!(api.guild_search("Die & Gäng").unwrap().is_empty());
        assert_eq!(api.transport().asked.borrow().as_slice(), [url]);
    }

    #[test]
    fn rf20_guild_search_returns_canonical_uuid() {
        const CANONICAL: &str = "230C9E95-E509-4B3E-AAF2-A320DB7506E5";
        let url = format!("{BASE}/guild/search?name=Le+Coq");
        let api = Api::new(Fake::with(&url, br#"["230c9e95e5094b3eaaf2a320db7506e5"]"#));

        assert_eq!(api.guild_search("Le Coq").unwrap(), [CANONICAL]);
    }

    #[test]
    fn rf20_wvw_guild_team_keys_are_canonical() {
        const CANONICAL: &str = "230C9E95-E509-4B3E-AAF2-A320DB7506E5";
        let url = format!("{BASE}/wvw/guilds/eu");
        let api = Api::new(Fake::with(
            &url,
            br#"{"230c9e95e5094b3eaaf2a320db7506e5":"12014"}"#,
        ));

        let teams = api.wvw_guilds(gw2lurker_core::Region::Eu).unwrap();
        assert_eq!(teams.get(CANONICAL), Some(&12014));
    }

    #[test]
    fn rf03a_the_team_of_a_guild_is_read_whatever_the_api_uses() {
        // El API manda el equipo **como cadena** (`{ "id-de-clan": "12008" }`, medido el
        // 2026-10-31). Parsearlo directo a `u32` daba `invalid type: string "12008"`, que es lo
        // que el jugador veía como «LA BÚSQUEDA FALLÓ … INVALID TYPE: STRING "12008"» en el
        // buscador de clanes.
        let url = format!("{BASE}/wvw/guilds/eu");
        let text = br#"{ "AAAA-1": "12008", "BBBB-2": "12014" }"#;
        let api = Api::new(Fake::with(&url, text));
        let teams = api.wvw_guilds(gw2lurker_core::Region::Eu).unwrap();
        assert_eq!(teams.get("AAAA-1"), Some(&12008));
        assert_eq!(teams.get("BBBB-2"), Some(&12014));

        // Y un número pelado sigue valiendo, por si el API vuelve a mandarlo así.
        let numbers = br#"{ "CCCC-3": 12014 }"#;
        let api = Api::new(Fake::with(&url, numbers));
        let teams = api.wvw_guilds(gw2lurker_core::Region::Eu).unwrap();
        assert_eq!(teams.get("CCCC-3"), Some(&12014));
    }

    #[test]
    fn rf20_guild_returns_name_and_tag() {
        let id = "230c9e95e5094b3eaaf2a320db7506e5";
        let api = Api::new(Fake::with(
            &format!("{BASE}/guild/230C9E95-E509-4B3E-AAF2-A320DB7506E5"),
            GUILD,
        ));
        let g = api.guild(id).unwrap();
        assert_eq!(g.tag, "LAX");
        assert!(!g.name.is_empty());
    }

    #[test]
    fn rf20_a_malformed_guild_id_never_reaches_the_network() {
        // Los ids salen también de `guild-names.json`, que se descarga de una URL configurable:
        // uno con `/`, `?` o `..` cambiaría el endpoint pedido.
        let api = Api::new(Fake::default());
        for id in [
            "../wvw/matches?ids=all",
            "230C9E95-E509-4B3E-AAF2-A320DB7506E5?x=1",
            "230C9E95",
            "",
        ] {
            assert_eq!(api.guild(id), Err(ApiError::NotFound), "id {id:?}");
        }
        assert!(api.transport().asked.borrow().is_empty());
        assert!(
            looks_like_guild_id("230c9e95e5094b3eaaf2a320db7506e5"),
            "sin guiones y en minúsculas sigue siendo un id"
        );
    }

    #[test]
    fn rf03d_team_assignment_timer_parses_both_regions() {
        let url = format!("{BASE}/wvw/timers/teamAssignment");
        let api = Api::new(Fake::with(&url, TEAM_ASSIGNMENT));
        let timers = api.team_assignment_timers().unwrap();

        assert_eq!(timers.na, "2026-11-07T02:00:00Z");
        assert_eq!(timers.eu, "2026-11-06T18:00:00Z");
    }

    #[test]
    fn rf03d_matches_overview_returns_the_live_match_ids() {
        let url = format!("{BASE}/wvw/matches/overview");
        let api = Api::new(Fake::with(&url, MATCHES_OVERVIEW));
        let ids = api.matches_overview().unwrap();

        assert_eq!(ids.len(), 9);
        assert!(ids.contains(&"2-2".to_owned()));
        assert!(ids.contains(&"1-4".to_owned()));
    }

    #[test]
    fn rf20_a_malformed_match_id_or_language_never_reaches_the_network() {
        let api = Api::new(Fake::default());
        assert_eq!(api.match_by_id("2-2/../1-1"), Err(ApiError::NotFound));
        assert_eq!(api.match_by_id("2-"), Err(ApiError::NotFound));
        assert_eq!(api.objectives("es&ids=1"), Err(ApiError::NotFound));
        assert!(api.transport().asked.borrow().is_empty());
    }

    #[test]
    fn rf20_unknown_routes_answer_like_the_real_transport() {
        // `net::WinHttpTransport` traduce el 404 a `NotFound`; el doble de pruebas hace lo mismo
        // para que un test no pase por un camino que en producción no existe.
        let api = Api::new(Fake::default());
        assert_eq!(api.match_by_id("9-9"), Err(ApiError::NotFound));
        assert!(!Backoff::should_retry(&ApiError::NotFound));
    }

    #[test]
    fn rf30_maps_come_with_the_boxes_of_the_projection() {
        let url = format!("{BASE}/maps?ids=38,95,96,1099,1652");
        let api = Api::new(Fake::with(&url, MAPS));
        let maps = api.maps().unwrap();

        assert!(
            maps.len() >= 5,
            "la fixture incluye las cinco variantes conocidas de mapas de MvM"
        );
        for id in [38, 95, 96, 1099, 1652] {
            assert!(
                maps.iter().any(|map| map.id == id),
                "falta el mapa {id} en la respuesta"
            );
        }
        let ebg = maps.iter().find(|map| map.id == 38).unwrap();
        assert_eq!(ebg.continent_rect, [[8958.0, 12798.0], [12030.0, 15870.0]]);
        let seared = maps.iter().find(|map| map.id == 1652).unwrap();
        assert_eq!(seared.name.trim(), "Seared Borderlands");
        assert_eq!(
            seared.continent_rect,
            [[9214.0, 8958.0], [11774.0, 12542.0]]
        );
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
