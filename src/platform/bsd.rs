use std::fs::{File, OpenOptions};
use std::io;
use std::os::unix::fs::FileTypeExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;

#[cfg(target_os = "freebsd")]
const DIOCGMEDIASIZE: libc::c_ulong = 0x40086481;
#[cfg(target_os = "netbsd")]
const DIOCGMEDIASIZE: libc::c_ulong = 0x40086484;
#[cfg(target_os = "dragonfly")]
const DIOCGMEDIASIZE: libc::c_ulong = 0x40086487;

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
    let mut size: u64 = 0;
    let ret = unsafe { libc::ioctl(file.as_raw_fd(), DIOCGMEDIASIZE, &mut size) };
    if ret != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(size)
}
