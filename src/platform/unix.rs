use std::fs::{self, File, OpenOptions};
use std::io::{self, Seek, SeekFrom};
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::os::unix::io::AsRawFd;
use std::path::Path;

pub fn is_block_device(path: &Path) -> bool {
    std::fs::metadata(path)
        .map(|m| m.file_type().is_block_device())
        .unwrap_or(false)
}

pub fn is_system_root(path: &Path) -> bool {
    path == Path::new("/")
}

/// Prefer a read+write open (needed for verification); fall back to a
/// write-only open so write-only media still works.
pub fn open_device(path: &Path) -> io::Result<File> {
    match OpenOptions::new().read(true).write(true).open(path) {
        Ok(f) => Ok(f),
        Err(_) => OpenOptions::new().write(true).open(path),
    }
}

pub fn device_size(file: &mut File) -> io::Result<u64> {
    file.seek(SeekFrom::End(0))
}

// ---------------------------------------------------------------------------
// Linux-only helpers (BLKROTATIONAL, /proc/self/mounts, SEEK_DATA/SEEK_HOLE)
// ---------------------------------------------------------------------------

/// `BLKROTATIONAL = _IOR(0x12, 126, unsigned short)` → 0x127E.
#[cfg(target_os = "linux")]
const BLKROTATIONAL: libc::c_ulong = 0x127E;

#[cfg(target_os = "linux")]
fn unescape_mount_field(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut it = s.bytes().peekable();
    while let Some(&b) = it.peek() {
        if b == b'\\' {
            it.next();
            let mut code = 0u32;
            let mut digits = 0;
            while digits < 3 {
                match it.peek() {
                    Some(c @ b'0'..=b'7') => {
                        code = code * 8 + (u32::from(*c) - u32::from(b'0'));
                        it.next();
                        digits += 1;
                    }
                    _ => break,
                }
            }
            if digits == 3 {
                out.push(char::from_u32(code).unwrap_or('?'));
                continue;
            }
            out.push('\\');
            continue;
        }
        out.push(b as char);
        it.next();
    }
    out
}

#[cfg(target_os = "linux")]
pub fn device_is_ssd(path: &Path) -> bool {
    let f = match OpenOptions::new().read(true).open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut val: libc::c_ushort = 0;
    // SAFETY: BLKROTATIONAL writes a single c_ushort.
    let rc = unsafe { libc::ioctl(f.as_raw_fd(), BLKROTATIONAL, &mut val) };
    rc >= 0 && val == 0
}

#[cfg(target_os = "linux")]
pub fn file_on_solid_state(path: &Path) -> bool {
    let dev = match fs::metadata(path) {
        Ok(m) => m.dev(),
        Err(_) => return false,
    };
    let mounts = fs::read_to_string("/proc/self/mounts").unwrap_or_default();
    for line in mounts.lines() {
        let mut fields = line.splitn(6, ' ');
        let device = unescape_mount_field(fields.next().unwrap_or(""));
        let mountpoint = unescape_mount_field(fields.next().unwrap_or(""));
        if !device.starts_with('/') || mountpoint.is_empty() {
            continue;
        }
        if let Ok(m) = fs::metadata(&mountpoint) {
            if m.dev() == dev {
                return device_is_ssd(Path::new(&device));
            }
        }
    }
    false
}

#[cfg(target_os = "linux")]
pub fn check_not_mounted(path: &Path) -> io::Result<()> {
    let meta = fs::metadata(path)?;
    let target_rdev = meta.rdev();
    let target_major = (target_rdev >> 8) & 0xFFF;
    let target_minor = target_rdev & 0xFF;

    if let Ok(root) = fs::metadata("/") {
        if root.dev() == target_rdev {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "device is the system root filesystem device",
            ));
        }
    }

    let mounts = fs::read_to_string("/proc/self/mounts").unwrap_or_default();
    for line in mounts.lines() {
        let mut fields = line.splitn(6, ' ');
        let device = unescape_mount_field(fields.next().unwrap_or(""));
        let mountpoint = unescape_mount_field(fields.next().unwrap_or(""));
        if !device.starts_with('/') || mountpoint.is_empty() {
            continue;
        }
        if let Ok(m) = fs::metadata(&device) {
            let mounted_rdev = m.rdev();
            let same_device = mounted_rdev == target_rdev;
            // Whole disk vs its partitions: same major, target is the whole
            // disk (minor multiple of 16) and the mounted device is one of
            // its partition minors (e.g. /dev/sda vs /dev/sda1).
            let is_partition_of_target = (mounted_rdev >> 8) & 0xFFF == target_major
                && target_minor % 16 == 0
                && mounted_rdev & 0xFF > target_minor
                && mounted_rdev & 0xFF < target_minor + 16;
            if same_device || is_partition_of_target {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    format!(
                        "device (or a partition of it) is currently mounted at {}",
                        mountpoint
                    ),
                ));
            }
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn data_ranges(file: &File, size: u64) -> io::Result<Option<Vec<(u64, u64)>>> {
    let mut ranges = Vec::new();
    let mut pos: i64 = 0;
    loop {
        // SAFETY: plain lseek with SEEK_DATA/SEEK_HOLE on a valid fd.
        let data = unsafe { libc::lseek(file.as_raw_fd(), pos, libc::SEEK_DATA) };
        if data < 0 {
            let err = io::Error::last_os_error();
            return match err.raw_os_error() {
                Some(libc::EINVAL) | Some(libc::EOPNOTSUPP) => Ok(None),
                Some(libc::ENXIO) => Ok(Some(ranges)),
                _ => Err(err),
            };
        }
        if data as u64 >= size {
            break;
        }
        // SAFETY: as above.
        let hole = unsafe { libc::lseek(file.as_raw_fd(), data, libc::SEEK_HOLE) };
        let end = if hole < 0 {
            size
        } else {
            (hole as u64).min(size)
        };
        ranges.push((data as u64, end - data as u64));
        if end >= size {
            break;
        }
        pos = end as i64;
    }
    Ok(Some(ranges))
}
