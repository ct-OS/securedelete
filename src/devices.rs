//! Discovery of the storage devices that could be wiped (Linux: `/sys/block`).

use std::path::PathBuf;

/// A whole-disk block device found on the system.
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    /// Device node, e.g. `/dev/sdb`.
    pub path: PathBuf,
    /// Kernel name, e.g. `sdb` or `nvme0n1`.
    pub name: String,
    /// Vendor and model as reported by the kernel (may be empty).
    pub model: String,
    /// Size in bytes.
    pub size: u64,
    pub removable: bool,
    /// Spinning disk (false for SSD / flash / unknown).
    pub rotational: bool,
    /// Attached through USB.
    pub usb: bool,
    /// For loop devices, the file backing them.
    pub loop_backing: Option<PathBuf>,
}

/// Whole-disk devices, sorted by name. Partitions, RAM disks, zram, optical
/// drives, device-mapper and unattached loop devices are left out. Returns an
/// empty list on platforms without discovery support.
pub fn list_devices() -> Vec<DeviceInfo> {
    #[cfg(target_os = "linux")]
    {
        linux::list_devices()
    }
    #[cfg(not(target_os = "linux"))]
    {
        Vec::new()
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::DeviceInfo;
    use std::fs;
    use std::path::{Path, PathBuf};

    const SKIP_PREFIXES: &[&str] = &["ram", "zram", "sr", "fd", "dm-", "md", "nbd"];

    fn read_trim(path: &Path) -> Option<String> {
        fs::read_to_string(path).ok().map(|s| s.trim().to_string())
    }

    pub fn list_devices() -> Vec<DeviceInfo> {
        let Ok(entries) = fs::read_dir("/sys/block") else {
            return Vec::new();
        };
        let mut devices: Vec<DeviceInfo> = entries
            .flatten()
            .filter_map(|e| describe(&e.file_name().to_string_lossy(), &e.path()))
            .collect();
        devices.sort_by(|a, b| a.name.cmp(&b.name));
        devices
    }

    fn describe(name: &str, sys: &Path) -> Option<DeviceInfo> {
        if SKIP_PREFIXES.iter().any(|p| name.starts_with(p)) {
            return None;
        }
        let loop_backing = if name.starts_with("loop") {
            // Unattached loop devices have no backing file: skip them.
            Some(PathBuf::from(read_trim(&sys.join("loop/backing_file"))?))
        } else {
            None
        };

        // `size` is always in 512-byte sectors.
        let sectors: u64 = read_trim(&sys.join("size"))?.parse().ok()?;
        if sectors == 0 {
            return None;
        }

        // "ATA" is the transport, not a vendor; leave it out.
        let vendor = read_trim(&sys.join("device/vendor"))
            .filter(|v| v != "ATA")
            .unwrap_or_default();
        let model = read_trim(&sys.join("device/model")).unwrap_or_default();
        let model = match (vendor.is_empty(), model.is_empty()) {
            (false, false) if !model.starts_with(&vendor) => format!("{vendor} {model}"),
            (_, false) => model,
            (false, true) => vendor,
            (true, true) => String::new(),
        };

        let usb = fs::canonicalize(sys)
            .map(|p| p.to_string_lossy().contains("/usb"))
            .unwrap_or(false);

        Some(DeviceInfo {
            path: PathBuf::from("/dev").join(name),
            name: name.to_string(),
            model,
            size: sectors * 512,
            removable: read_trim(&sys.join("removable")).as_deref() == Some("1"),
            rotational: read_trim(&sys.join("queue/rotational")).as_deref() == Some("1"),
            usb,
            loop_backing,
        })
    }
}

/// Human-readable size in decimal units, as drive vendors label them.
pub fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "kB", "MB", "GB", "TB", "PB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1000.0 && unit < UNITS.len() - 1 {
        value /= 1000.0;
        unit += 1;
    }
    if unit == 0 || value >= 100.0 {
        format!("{value:.0} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_size_units() {
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(50_000_000), "50.0 MB");
        assert_eq!(format_size(500_107_862_016), "500 GB");
        assert_eq!(format_size(2_000_398_934_016), "2.0 TB");
    }

    #[test]
    fn list_devices_does_not_panic() {
        for d in list_devices() {
            assert!(d.size > 0);
            assert!(d.path.starts_with("/dev"));
        }
    }
}
