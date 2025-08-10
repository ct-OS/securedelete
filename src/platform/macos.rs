use std::fs::{File, OpenOptions};
use std::io;
use std::os::unix::fs::FileTypeExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;

const DKIOCGETBLOCKSIZE: libc::c_ulong = 0x40046418;
const DKIOCGETBLOCKCOUNT: libc::c_ulong = 0x40086419;

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
    let mut block_size: u32 = 0;
    let mut block_count: u64 = 0;
    let ret = unsafe { libc::ioctl(file.as_raw_fd(), DKIOCGETBLOCKSIZE, &mut block_size) };
    if ret != 0 {
        return Err(io::Error::last_os_error());
    }
    let ret = unsafe { libc::ioctl(file.as_raw_fd(), DKIOCGETBLOCKCOUNT, &mut block_count) };
    if ret != 0 {
        return Err(io::Error::last_os_error());
    }
    if block_size == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid block size",
        ));
    }
    Ok(block_count * block_size as u64)
}
