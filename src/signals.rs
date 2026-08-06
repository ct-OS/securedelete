//! Ctrl-C / SIGTERM handling: sets an atomic flag that the wiper checks
//! between chunks so a wipe can stop cleanly and report partial state.

use std::sync::atomic::{AtomicBool, Ordering};

static INTERRUPTED: AtomicBool = AtomicBool::new(false);

pub fn install() {
    #[cfg(unix)]
    {
        unsafe extern "C" fn on_signal(_: libc::c_int) {
            INTERRUPTED.store(true, Ordering::Relaxed);
        }
        // SAFETY: the handler only performs an atomic store, which is
        // async-signal-safe; the function pointer stays valid for the
        // process lifetime.
        unsafe {
            let handler: libc::sighandler_t = on_signal as *const () as libc::sighandler_t;
            let _ = libc::signal(libc::SIGINT, handler);
            let _ = libc::signal(libc::SIGTERM, handler);
        }
    }
    #[cfg(windows)]
    {
        unsafe extern "system" fn on_ctrl(_: u32) -> i32 {
            INTERRUPTED.store(true, Ordering::Relaxed);
            1
        }
        // SAFETY: registers a static handler function; it stays valid.
        unsafe {
            use windows_sys::Win32::System::Console::SetConsoleCtrlHandler;
            let _ = SetConsoleCtrlHandler(Some(on_ctrl), 1);
        }
    }
}

pub fn interrupted() -> bool {
    INTERRUPTED.load(Ordering::Relaxed)
}
