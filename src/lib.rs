//! Core of `secure-delete`: wipe methods, platform helpers and the wiper.
//!
//! Shared by the command-line tool (`src/main.rs`) and the graphical
//! front-end (`gui/`).

pub mod devices;
pub mod methods;
pub mod platform;
pub mod signals;
pub mod wiper;

use std::fs;
use std::path::Path;

/// Check that `target` is a regular file that may be wiped: not a system
/// root, not a symlink, not a directory.
pub fn validate_file_target(target: &Path) -> Result<(), String> {
    if platform::is_system_root(target) {
        return Err("refusing to wipe a system root".into());
    }

    let meta = fs::symlink_metadata(target).map_err(|e| e.to_string())?;

    if meta.file_type().is_symlink() {
        return Err("target is a symbolic link; pass the real path instead".into());
    }

    if !meta.file_type().is_file() {
        if meta.file_type().is_dir() {
            return Err("target is a directory".into());
        }
        return Err("target is not a regular file".into());
    }

    Ok(())
}

/// Check that `target` is an unmounted block device that is not a system
/// root.
pub fn validate_device_target(target: &Path) -> Result<(), String> {
    if platform::is_system_root(target) {
        return Err("refusing to wipe a system root".into());
    }
    if !platform::is_block_device(target) {
        return Err("target is not a block device; pass a device path (e.g. /dev/sdX or \\\\.\\PhysicalDrive0)".into());
    }
    platform::check_not_mounted(target).map_err(|e| e.to_string())
}
