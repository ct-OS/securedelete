//! Platform-specific helpers to abstract the differences between Unix and
//! Windows when dealing with files, block devices and system roots.

use std::fs::{File, OpenOptions};
use std::io;
use std::path::Path;

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

// ---------------------------------------------------------------------------
// Cross-platform helpers with Linux-specific implementations
// ---------------------------------------------------------------------------

/// Refuse to wipe a device that is mounted or that is the system root device.
/// Only enforced on Linux (reads /proc/self/mounts); a no-op elsewhere.
#[cfg(target_os = "linux")]
pub fn check_not_mounted(path: &Path) -> io::Result<()> {
    unix::check_not_mounted(path)
}

#[cfg(not(target_os = "linux"))]
pub fn check_not_mounted(_path: &Path) -> io::Result<()> {
    Ok(())
}

/// Whether the filesystem containing `path` lives on a solid-state device.
/// Best-effort, Linux only (BLKROTATIONAL); false when unknown.
#[cfg(target_os = "linux")]
pub fn file_on_solid_state(path: &Path) -> bool {
    unix::file_on_solid_state(path)
}

#[cfg(not(target_os = "linux"))]
pub fn file_on_solid_state(_path: &Path) -> bool {
    false
}

/// Whether `path` (a block device) is a solid-state device. Best-effort,
/// Linux only (BLKROTATIONAL); false when unknown.
#[cfg(target_os = "linux")]
pub fn device_is_ssd(path: &Path) -> bool {
    unix::device_is_ssd(path)
}

#[cfg(not(target_os = "linux"))]
pub fn device_is_ssd(_path: &Path) -> bool {
    false
}

/// Data extents `(start, length)` of a file via SEEK_DATA/SEEK_HOLE, so the
/// wiper can skip holes in sparse files. `None` means "holes not supported;
/// wipe the whole file". Linux only; elsewhere always `Ok(None)`.
#[cfg(target_os = "linux")]
pub fn data_ranges(file: &File, size: u64) -> io::Result<Option<Vec<(u64, u64)>>> {
    unix::data_ranges(file, size)
}

#[cfg(not(target_os = "linux"))]
pub fn data_ranges(_file: &File, _size: u64) -> io::Result<Option<Vec<(u64, u64)>>> {
    Ok(None)
}

/// Clear the read-only attribute on Windows so the file can be opened for
/// writing and removed. A no-op elsewhere.
#[cfg(windows)]
pub fn ensure_writable(path: &Path) -> io::Result<()> {
    windows::ensure_writable(path)
}

#[cfg(not(windows))]
pub fn ensure_writable(_path: &Path) -> io::Result<()> {
    Ok(())
}

/// Open a regular file for wiping. On Unix this adds `O_NOFOLLOW` to close the
/// TOCTOU race where a symlink replaces the file between validation and open.
#[cfg(any(
    target_os = "linux",
    target_os = "macos",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly"
))]
pub fn open_file_no_follow(path: &Path, read: bool, write: bool) -> io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt;

    let mut opts = OpenOptions::new();
    opts.read(read).write(write).custom_flags(libc::O_NOFOLLOW);
    opts.open(path)
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "macos",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly"
)))]
pub fn open_file_no_follow(path: &Path, read: bool, write: bool) -> io::Result<File> {
    OpenOptions::new().read(read).write(write).open(path)
}
