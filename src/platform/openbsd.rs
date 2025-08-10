use std::fs::{File, OpenOptions};
use std::io;
use std::os::unix::fs::FileTypeExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;

const DIOCGDINFO: libc::c_ulong = 0x41946465;

#[allow(dead_code)]
#[repr(C)]
struct OpenBsdPartition {
    p_size: u32,
    p_offset: u32,
    p_offseth: u16,
    p_sizeh: u16,
    p_fstype: u8,
    p_fragblock: u8,
    p_cpg: u16,
}

#[allow(dead_code)]
#[repr(C)]
struct OpenBsdDiskLabel {
    d_magic: u32,
    d_type: u16,
    d_subtype: u16,
    d_typename: [u8; 16],
    d_packname: [u8; 16],
    d_secsize: u32,
    d_nsectors: u32,
    d_ntracks: u32,
    d_ncylinders: u32,
    d_secpercyl: u32,
    d_secperunit: u32,
    d_uid: [u8; 8],
    d_acylinders: u32,
    d_bstarth: u16,
    d_bendh: u16,
    d_bstart: u32,
    d_bend: u32,
    d_flags: u32,
    d_spare4: [u32; 5],
    d_secperunith: u16,
    d_version: u16,
    d_spare: [u32; 4],
    d_magic2: u32,
    d_checksum: u16,
    d_npartitions: u16,
    d_spare2: u32,
    d_spare3: u32,
    d_partitions: [OpenBsdPartition; 16],
}

pub fn is_block_device(path: &Path) -> bool {
    std::fs::metadata(path)
        .map(|m| m.file_type().is_block_device() || m.file_type().is_char_device())
        .unwrap_or(false)
}

pub fn is_system_root(path: &Path) -> bool {
    path == Path::new("/")
}

pub fn open_device(path: &Path) -> io::Result<File> {
    OpenOptions::new().read(true).write(true).open(path)
}

pub fn device_size(file: &mut File) -> io::Result<u64> {
    let mut label: OpenBsdDiskLabel = unsafe { std::mem::zeroed() };
    let ret = unsafe { libc::ioctl(file.as_raw_fd(), DIOCGDINFO, &mut label) };
    if ret != 0 {
        return Err(io::Error::last_os_error());
    }
    if label.d_secsize == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid sector size",
        ));
    }
    let dsize = ((label.d_secperunith as u64) << 32) + label.d_secperunit as u64;
    Ok(dsize * label.d_secsize as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disklabel_layout_matches_openbsd() {
        assert_eq!(std::mem::size_of::<OpenBsdPartition>(), 16);
        assert_eq!(std::mem::size_of::<OpenBsdDiskLabel>(), 404);
        assert_eq!(std::mem::offset_of!(OpenBsdDiskLabel, d_secsize), 40);
        assert_eq!(std::mem::offset_of!(OpenBsdDiskLabel, d_secperunit), 60);
        assert_eq!(std::mem::offset_of!(OpenBsdDiskLabel, d_secperunith), 112);
        assert_eq!(std::mem::offset_of!(OpenBsdDiskLabel, d_partitions), 148);
    }
}
