//! Language resolution for the UI (RF-44, RF-45).
//!
//! The table itself lives in `gw2lurker_core::i18n`, where it can be checked on the Linux CI
//! job. This module only decides **which** language applies and adds the right-Ctrl override.

use gw2lurker_core::i18n::{self, Language};
use nexus::rtapi::RealTimeApi;

use crate::config::LanguagePreference;

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

/// Saved preference unless it is *Auto*, then the game language, then English (RF-44).
pub fn effective() -> Language {
    resolve(crate::config().language, game_language())
}

/// Precedence between the saved preference and the language the game reports.
fn resolve(preference: LanguagePreference, game: Option<Language>) -> Language {
    match preference {
        LanguagePreference::Auto => game.unwrap_or(Language::En),
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

/// Game language as reported by Nexus RTAPI.
///
/// `None` when the RealTime API addon is not installed or has not published data yet, which
/// is not an error: the caller falls back to English.
fn game_language() -> Option<Language> {
    let api = RealTimeApi::get()?;
    let game = api.read_game()?;
    let code = u32::from(game.language.ok()?);
    Language::from_game_code(code)
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

/// Registers our strings in Nexus so its own UI can show them in the user's language.
///
/// Must be called from `load`: it needs the addon API, which panics before initialization.
pub fn register_translations() {
    for (key, cells) in i18n::TEXT {
        for language in Language::ALL {
            nexus::localization::set_translation(key, language.code(), cells[language.index()]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rf44_auto_follows_the_game_language() {
        assert_eq!(
            resolve(LanguagePreference::Auto, Some(Language::Fr)),
            Language::Fr
        );
        assert_eq!(
            resolve(LanguagePreference::Auto, Some(Language::De)),
            Language::De
        );
        assert_eq!(
            resolve(LanguagePreference::Auto, None),
            Language::En,
            "sin RTAPI no hay idioma del juego: inglés"
        );
    }

    #[test]
    fn rf44_explicit_preference_beats_the_game_language() {
        assert_eq!(
            resolve(LanguagePreference::De, Some(Language::Fr)),
            Language::De
        );
        assert_eq!(
            resolve(LanguagePreference::Es, Some(Language::En)),
            Language::Es
        );
    }

    #[test]
    fn rf45_english_override_uses_the_english_column() {
        use gw2lurker_core::i18n::t;
        assert_eq!(t(Language::En, "Following"), "Following");
        assert_ne!(t(Language::Es, "Following"), t(Language::En, "Following"));
    }
}
