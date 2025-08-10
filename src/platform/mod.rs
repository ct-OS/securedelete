//! Platform-specific helpers to abstract the differences between Unix and
//! Windows when dealing with files, block devices and system roots.

#[cfg(any(target_os = "freebsd", target_os = "netbsd", target_os = "dragonfly"))]
mod bsd;
#[cfg(not(any(unix, windows)))]
mod fallback;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "openbsd")]
mod openbsd;
#[cfg(target_os = "linux")]
pub mod secure_erase;
#[cfg(all(
    unix,
    not(any(
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "macos"
    ))
))]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(any(target_os = "freebsd", target_os = "netbsd", target_os = "dragonfly"))]
pub use bsd::*;
#[cfg(not(any(unix, windows)))]
pub use fallback::*;
#[cfg(target_os = "macos")]
pub use macos::*;
#[cfg(target_os = "openbsd")]
pub use openbsd::*;
#[cfg(all(
    unix,
    not(any(
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "macos"
    ))
))]
pub use unix::*;
#[cfg(windows)]
pub use windows::*;
