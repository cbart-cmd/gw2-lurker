//! Where the addon keeps its data.
//!
//! Not next to the DLL: `addons` lives under `Program Files`, where writing needs elevation,
//! and every addon of this machine that has data uses
//! `<Documents>\Guild Wars 2\addons\<addon>\` (`timers_wvw`, `gw2_builds_wvw`,
//! `fight_analysis_danger`). Those addons resolve it with
//! `SHGetKnownFolderPath(FOLDERID_Documents)`, which also honours a redirected (OneDrive)
//! Documents folder, so the registry key would be the wrong thing to read here.
//!
//! `shell32` and `ole32` enter through `extern` blocks: no `windows` crate, same approach as
//! `keys.rs`.

use core::ffi::c_void;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;

/// `FOLDERID_Documents` = `{FDD39AD0-238F-46AF-ADB4-6C85480369C7}`.
#[repr(C)]
struct Guid {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

const FOLDERID_DOCUMENTS: Guid = Guid {
    data1: 0xFDD3_9AD0,
    data2: 0x238F,
    data3: 0x46AF,
    data4: [0xAD, 0xB4, 0x6C, 0x85, 0x48, 0x03, 0x69, 0xC7],
};

const KF_FLAG_DEFAULT: u32 = 0;

#[link(name = "shell32")]
extern "system" {
    fn SHGetKnownFolderPath(
        rfid: *const Guid,
        flags: u32,
        token: *mut c_void,
        path: *mut *mut u16,
    ) -> i32;
}

#[link(name = "ole32")]
extern "system" {
    fn CoTaskMemFree(pointer: *mut c_void);
}

/// The user's Documents folder, honouring a OneDrive redirect.
pub fn documents() -> Option<PathBuf> {
    let mut raw: *mut u16 = core::ptr::null_mut();

    // SAFETY: `FOLDERID_DOCUMENTS` is a valid GUID constant; a null token means "current user"
    // and `KF_FLAG_DEFAULT` is the documented flag. `raw` is a valid out-pointer that the call
    // fills with an allocated, NUL-terminated UTF-16 string, which is freed right below.
    let result = unsafe {
        SHGetKnownFolderPath(
            &FOLDERID_DOCUMENTS,
            KF_FLAG_DEFAULT,
            core::ptr::null_mut(),
            &mut raw,
        )
    };
    if result < 0 || raw.is_null() {
        return None;
    }

    // SAFETY: the call succeeded, so `raw` points to a NUL-terminated UTF-16 buffer owned by
    // COM. We only read it here and free it exactly once afterwards with `CoTaskMemFree`.
    let path = unsafe {
        let mut len = 0usize;
        while *raw.add(len) != 0 {
            len += 1;
        }
        let slice = std::slice::from_raw_parts(raw, len);
        let text = OsString::from_wide(slice);
        CoTaskMemFree(raw.cast());
        text
    };

    let path = PathBuf::from(path);
    (!path.as_os_str().is_empty()).then_some(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(windows)]
    fn documents_is_an_existing_folder() {
        let docs = documents().expect("Windows siempre tiene Documentos");
        assert!(docs.is_dir(), "{} deberia existir", docs.display());
    }

    #[test]
    fn the_guid_matches_the_documented_value() {
        // FDD39AD0-238F-46AF-ADB4-6C85480369C7, little endian for the first three fields.
        assert_eq!(FOLDERID_DOCUMENTS.data1, 0xFDD3_9AD0);
        assert_eq!(FOLDERID_DOCUMENTS.data2, 0x238F);
        assert_eq!(FOLDERID_DOCUMENTS.data3, 0x46AF);
        assert_eq!(
            FOLDERID_DOCUMENTS.data4,
            [0xAD, 0xB4, 0x6C, 0x85, 0x48, 0x03, 0x69, 0xC7]
        );
    }
}
