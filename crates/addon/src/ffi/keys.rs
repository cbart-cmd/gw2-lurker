//! Keyboard and focus state for RF-45.
//!
//! Part of `ffi`, the only module allowed to use `unsafe`. `user32.dll` is linked through
//! `extern` instead of pulling the `windows` crate as a new dependency: the two calls we
//! need live there and are stable since Windows 2000.

use core::ffi::c_void;

/// Virtual-key code of the right Control key.
const VK_RCONTROL: i32 = 0xA3;

#[link(name = "user32")]
extern "system" {
    fn GetAsyncKeyState(v_key: i32) -> i16;
    fn GetForegroundWindow() -> *mut c_void;
    fn GetWindowThreadProcessId(window: *mut c_void, process_id: *mut u32) -> u32;
    fn GetKeyNameTextW(l_param: i32, buffer: *mut u16, size: i32) -> i32;
}

#[link(name = "kernel32")]
extern "system" {
    fn GetUserDefaultUILanguage() -> u16;
}

/// Primary language id of the Windows display language.
///
/// Used only as a last resort to guess the game language when the RealTime API is not
/// installed (`GetUserDefaultUILanguage` returns a `LANGID`; the low 10 bits are the primary
/// language).
pub fn os_primary_language() -> u16 {
    // SAFETY: no arguments, no pointers: the function reads a value set by Windows and
    // returns it. Masking with 0x3FF is the documented way to get the primary language.
    unsafe { GetUserDefaultUILanguage() & 0x3FF }
}

/// Whether right Ctrl is being held.
pub fn right_ctrl_down() -> bool {
    // SAFETY: `GetAsyncKeyState` only reads the global keyboard state; it takes a value and
    // returns a value, so no pointer, no lifetime and no aliasing is involved.
    unsafe { GetAsyncKeyState(VK_RCONTROL) < 0 }
}

/// Name of the key with this set-1 scan code **in the player's keyboard layout** (`Ñ` on a
/// Spanish keyboard where the US table says `;`). `None` if Windows has no name for it.
///
/// UNVERIFIED(RF-18): how Nexus encodes extended keys (arrows, Insert…) in `InputBinds.json`;
/// anything above the low byte is taken as the extended flag.
pub fn layout_key_name(scan_code: u16) -> Option<String> {
    let extended = if scan_code > 0xFF { 1 << 24 } else { 0 };
    let l_param = (i32::from(scan_code & 0xFF) << 16) | extended;
    let mut buffer = [0u16; 64];
    // SAFETY: `buffer` is a live, writable array and `size` is its exact length in UTF-16 units,
    // so Windows never writes past it; the return value is the number of units written
    // (without the terminator), which is clamped to the buffer before slicing.
    let written = unsafe { GetKeyNameTextW(l_param, buffer.as_mut_ptr(), buffer.len() as i32) };
    let written = usize::try_from(written).ok()?.min(buffer.len());
    if written == 0 {
        return None;
    }
    String::from_utf16(&buffer[..written]).ok()
}

/// Whether the foreground window belongs to this process, that is, the game has focus.
///
/// Without this, holding right Ctrl while writing in another application would switch the
/// addon to English for no reason (RF-45).
pub fn game_has_focus() -> bool {
    // SAFETY: both functions are pure queries over a window handle. The handle comes from
    // `GetForegroundWindow` and may be null, which is checked before use;
    // `GetWindowThreadProcessId` accepts a valid out-pointer, which `process` is, and the
    // game process is the one running this DLL, so comparing with `std::process::id()` does
    // not need extra privileges.
    unsafe {
        let window = GetForegroundWindow();
        if window.is_null() {
            return false;
        }
        let mut process = 0u32;
        GetWindowThreadProcessId(window, &mut process);
        process == std::process::id()
    }
}
