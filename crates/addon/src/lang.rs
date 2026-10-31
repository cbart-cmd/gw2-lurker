//! Where the interface language comes from (RF-44).
//!
//! Three real sources, no guessing:
//!
//! 1. **Unofficial Extras** (the plugin is present in this install) raises
//!    `EV_UNOFFICIAL_EXTRAS_LANGUAGE_CHANGED` with the **game client language** and, per its
//!    own documentation, *"will also be called directly after initialization, with the current
//!    language, to get the startup language"*. Nexus already bridges that event, so
//!    `nexus::event::extras::LANGUAGE_CHANGED` is all we need: no dependency, no polling.
//! 2. **Nexus itself** does not expose the language through its addon API (checked
//!    `NexusLink` and every `AddonApi` field), but it stores it in
//!    `<juego>/Nexus/Settings.json` as a display name (`"Language": "Español"`) and loads the
//!    matching `Nexus/Locales/<id>_Main.json`, which carries the two-letter `Identifier`.
//! 3. The Windows display language, as a last resort.

use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::OnceLock;

use gw2lurker_core::i18n::Language;
use nexus::event::{event_consume, extras::LANGUAGE_CHANGED};
use serde::Deserialize;

/// Raw GW2 language code reported by Extras; `-1` until the first event arrives.
static EXTRAS_CODE: AtomicI32 = AtomicI32::new(-1);

/// Nexus language, resolved once because it costs two file reads.
static NEXUS_LANGUAGE: OnceLock<Option<Language>> = OnceLock::new();

/// Subscribes to the Extras language event.
///
/// Must be called from `load`. `revert_on_unload` makes Nexus unsubscribe when the addon
/// goes away, so no callback outlives the DLL.
pub fn subscribe() {
    LANGUAGE_CHANGED
        .subscribe(event_consume!(<nexus::event::extras::Language> on_language))
        .revert_on_unload();
}

/// Extras sends the current language right after initialization, so this also covers startup.
fn on_language(payload: Option<&nexus::event::extras::Language>) {
    if let Some(language) = payload {
        EXTRAS_CODE.store(i32::from(*language), Ordering::Relaxed);
    }
}

/// Game client language as reported by Extras, if it has told us yet.
pub fn from_extras() -> Option<Language> {
    let code = EXTRAS_CODE.load(Ordering::Relaxed);
    if code < 0 {
        return None;
    }
    Language::from_game_code(code as u32)
}

/// Nexus language, read once from its own files.
///
/// Called during `load` so the file reads never happen in the render callback (rule 6).
pub fn from_nexus_settings() -> Option<Language> {
    *NEXUS_LANGUAGE.get_or_init(read_nexus_language)
}

fn read_nexus_language() -> Option<Language> {
    let nexus_dir = nexus::paths::get_game_dir().ok()?.join("Nexus");
    let settings = std::fs::read_to_string(nexus_dir.join("Settings.json")).ok()?;
    let display = parse_settings_language(&settings)?;
    let identifier = locale_identifier(&nexus_dir.join("Locales"), &display)?;
    Language::from_code(&identifier)
}

#[derive(Deserialize)]
struct NexusSettings {
    #[serde(rename = "Language", default)]
    language: Option<String>,
}

/// Display name Nexus has stored (`"Español"`), trimmed.
fn parse_settings_language(json: &str) -> Option<String> {
    let settings: NexusSettings = serde_json::from_str(json).ok()?;
    let language = settings.language?;
    let language = language.trim();
    (!language.is_empty()).then(|| language.to_owned())
}

#[derive(Deserialize)]
struct LocaleFile {
    #[serde(rename = "Identifier", default)]
    identifier: String,
    #[serde(rename = "DisplayName", default)]
    display_name: String,
}

/// Two-letter identifier of the locale whose display name matches, e.g. `Español` → `es`.
fn locale_identifier(dir: &std::path::Path, display: &str) -> Option<String> {
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|extension| extension != "json") {
            continue;
        }
        let Some(raw) = std::fs::read_to_string(&path).ok() else {
            continue;
        };
        let Ok(locale) = serde_json::from_str::<LocaleFile>(&raw) else {
            continue;
        };
        if locale.display_name.trim().eq_ignore_ascii_case(display) && !locale.identifier.is_empty()
        {
            return Some(locale.identifier);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rf44_nexus_settings_language_is_read() {
        let json =
            "{\n\t\"AcceptEULA\": true,\n\t\"Language\": \"Español\",\n\t\"FontSize\": 13.0\n}";
        assert_eq!(parse_settings_language(json).as_deref(), Some("Español"));
    }

    #[test]
    fn rf44_nexus_settings_without_language_is_none() {
        assert_eq!(parse_settings_language(r#"{"AcceptEULA": true}"#), None);
        assert_eq!(parse_settings_language(r#"{"Language": "   "}"#), None);
        assert_eq!(parse_settings_language("no json"), None);
    }

    #[test]
    fn rf44_locale_identifier_matches_display_name() {
        // Shape taken from the real `Nexus/Locales/es_Main.json`.
        let locale = r#"{"Identifier": "es", "DisplayName": "Español", "Texts": {}}"#;
        let parsed: LocaleFile = serde_json::from_str(locale).unwrap();
        assert_eq!(parsed.identifier, "es");
        assert!(parsed.display_name.trim().eq_ignore_ascii_case("español"));
    }

    #[test]
    fn rf44_extras_codes_match_the_game_enum() {
        // `unofficial_extras::Language`: English = 0, French = 2, German = 3, Spanish = 4.
        for (raw, expected) in [
            (0, Language::En),
            (2, Language::Fr),
            (3, Language::De),
            (4, Language::Es),
        ] {
            assert_eq!(Language::from_game_code(raw), Some(expected));
        }
    }

    #[test]
    fn rf44_no_extras_event_means_no_answer() {
        // The atomic starts at -1 and `from_extras` must not invent a language.
        assert_eq!(EXTRAS_CODE.load(Ordering::Relaxed), -1);
        assert_eq!(from_extras(), None);
    }
}
