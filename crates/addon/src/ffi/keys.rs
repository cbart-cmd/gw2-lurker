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
}

/// Whether right Ctrl is being held.
pub fn right_ctrl_down() -> bool {
    // SAFETY: `GetAsyncKeyState` only reads the global keyboard state; it takes a value and
    // returns a value, so no pointer, no lifetime and no aliasing is involved.
    unsafe { GetAsyncKeyState(VK_RCONTROL) < 0 }
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
