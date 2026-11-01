//! Language resolution for the UI (RF-44, RF-45).
//!
//! The table itself lives in `gw2lurker_core::i18n`, where it can be checked on the Linux CI
//! job. This module only decides **which** language applies and adds the right-Ctrl override.

use gw2lurker_api::ApiError;
use gw2lurker_core::i18n::{self, Language};
use std::sync::atomic::{AtomicU8, Ordering};

use crate::config::LanguagePreference;

static PREFERENCE: AtomicU8 = AtomicU8::new(0);

/// Updates the cached language preference without taking the runtime configuration lock.
pub fn set_preference(preference: LanguagePreference) {
    let value = match preference {
        LanguagePreference::Auto => 0,
        LanguagePreference::Es => 1,
        LanguagePreference::En => 2,
        LanguagePreference::Fr => 3,
        LanguagePreference::De => 4,
    };
    PREFERENCE.store(value, Ordering::Relaxed);
}

fn preference() -> LanguagePreference {
    match PREFERENCE.load(Ordering::Relaxed) {
        1 => LanguagePreference::Es,
        2 => LanguagePreference::En,
        3 => LanguagePreference::Fr,
        4 => LanguagePreference::De,
        _ => LanguagePreference::Auto,
    }
}

/// Language that must be used right now.
///
/// Holding right Ctrl with the game in front shows everything in English without touching
/// the saved preference (RF-45).
pub fn active() -> Language {
    if english_override() {
        Language::En
    } else {
        effective()
    }
}

/// Saved preference unless it is *Auto*, then the best source available, then English.
///
/// Order (RF-44): game language from Unofficial Extras → language Nexus is using → Windows
/// display language → English. The RealTime API feature of `nexus-rs` was dropped on
/// 2026-10-31: Extras already reports the real client language, so RTAPI only added size.
pub fn effective() -> Language {
    resolve(
        preference(),
        [
            crate::lang::from_extras(),
            crate::lang::from_nexus_settings(),
            Language::from_windows_language(crate::ffi::keys::os_primary_language()),
        ],
    )
}

/// Precedence between the saved preference and the sources, most reliable first.
fn resolve(preference: LanguagePreference, sources: [Option<Language>; 3]) -> Language {
    match preference {
        LanguagePreference::Auto => sources.into_iter().flatten().next().unwrap_or(Language::En),
        LanguagePreference::Es => Language::Es,
        LanguagePreference::En => Language::En,
        LanguagePreference::Fr => Language::Fr,
        LanguagePreference::De => Language::De,
    }
}

/// `true` while right Ctrl is held and the game is the foreground window.
pub fn english_override() -> bool {
    crate::ffi::keys::right_ctrl_down() && crate::ffi::keys::game_has_focus()
}

/// Translated text for the language in force.
pub fn text(key: &'static str) -> &'static str {
    i18n::t(active(), key)
}

/// Translated text with a stable imgui id, for windows and labels whose state must survive a
/// language change.
pub fn widget(key: &'static str) -> String {
    i18n::w(active(), key)
}

/// Translated text with `{0}`, `{1}`… replaced.
pub fn format(key: &'static str, args: &[&str]) -> String {
    i18n::f(active(), key, args)
}

/// API errors are translated when displayed, not when produced on a background thread.
pub fn format_api_error(error: &ApiError) -> String {
    format_api_error_for(active(), error)
}

fn format_api_error_for(language: Language, error: &ApiError) -> String {
    match error {
        ApiError::Transport(detail) => i18n::f(language, "API network error: {0}", &[detail]),
        ApiError::Status(code) => i18n::f(
            language,
            "API returned HTTP status {0}",
            &[&code.to_string()],
        ),
        ApiError::Parse(detail) => {
            i18n::f(language, "API response could not be parsed: {0}", &[detail])
        }
        ApiError::NotFound => i18n::t(language, "API data was not found").to_owned(),
    }
}

/// Registers our strings in Nexus so its own UI can show them in the user's language.
///
/// Must run **after** `load`: it needs the addon API, which panics before initialization.
/// Called on the first frame for that reason, guarded so it only happens once.
pub fn register_translations_once() {
    static DONE: std::sync::Once = std::sync::Once::new();
    DONE.call_once(|| {
        for (key, cells) in i18n::TEXT {
            for language in Language::ALL {
                nexus::localization::set_translation(key, language.code(), cells[language.index()]);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rf44_auto_takes_the_most_reliable_source() {
        assert_eq!(
            resolve(
                LanguagePreference::Auto,
                [Some(Language::De), Some(Language::Es), None]
            ),
            Language::De,
            "Extras (idioma real del juego) gana a lo que tenga Nexus"
        );
        assert_eq!(
            resolve(
                LanguagePreference::Auto,
                [None, Some(Language::Es), Some(Language::En)]
            ),
            Language::Es,
            "Nexus gana al idioma de Windows"
        );
        assert_eq!(
            resolve(LanguagePreference::Auto, [None, None, Some(Language::Es)]),
            Language::Es,
            "y Windows es el último recurso antes de inglés"
        );
        assert_eq!(
            resolve(LanguagePreference::Auto, [None, None, None]),
            Language::En
        );
    }

    #[test]
    fn rf44_explicit_preference_beats_everything() {
        let all = [Some(Language::Fr), Some(Language::Es), Some(Language::De)];
        assert_eq!(resolve(LanguagePreference::De, all), Language::De);
        assert_eq!(resolve(LanguagePreference::Es, all), Language::Es);
    }

    #[test]
    fn rf44_language_preference_is_cached_without_a_config_lock() {
        for preference in [
            LanguagePreference::Auto,
            LanguagePreference::Es,
            LanguagePreference::En,
            LanguagePreference::Fr,
            LanguagePreference::De,
        ] {
            set_preference(preference);
            assert_eq!(super::preference(), preference);
        }
    }

    #[test]
    fn rf44_api_errors_are_localized_by_variant() {
        use gw2lurker_api::ApiError;

        assert_eq!(
            format_api_error_for(Language::Es, &ApiError::Transport("timeout".into())),
            "Error de red de la API: timeout"
        );
        assert_eq!(
            format_api_error_for(Language::En, &ApiError::Status(503)),
            "API returned HTTP status 503"
        );
        assert_eq!(
            format_api_error_for(Language::Fr, &ApiError::Parse("JSON".into())),
            "Impossible d'analyser la réponse de l'API : JSON"
        );
        assert_eq!(
            format_api_error_for(Language::De, &ApiError::NotFound),
            "Die API-Daten wurden nicht gefunden"
        );
    }

    #[test]
    fn rf45_english_override_uses_the_english_column() {
        use gw2lurker_core::i18n::t;
        assert_eq!(t(Language::En, "Following"), "Following");
        assert_ne!(t(Language::Es, "Following"), t(Language::En, "Following"));
    }
}
