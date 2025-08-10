use std::fs::{File, OpenOptions};
use std::io;
use std::path::Path;

use windows_sys::Win32::Foundation::{BOOL, FALSE};
use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_WRITE_THROUGH;
use windows_sys::Win32::System::Ioctl::{GET_LENGTH_INFORMATION, IOCTL_DISK_GET_LENGTH_INFO};
use windows_sys::Win32::System::IO::DeviceIoControl;

pub fn is_block_device(path: &Path) -> bool {
    let s = path.to_string_lossy();
    let s = s.trim_start_matches("\\?\\");
    s.starts_with("\\\\.\\")
}

pub fn is_system_root(path: &Path) -> bool {
    let s = path.to_string_lossy();
    let b = s.as_bytes();
    b.len() >= 2 && b[1] == b':' && (b.len() == 2 || b[2] == b'\\' || b[2] == b'/')
}

pub fn open_device(path: &Path) -> io::Result<File> {
    use std::os::windows::fs::OpenOptionsExt;

    OpenOptions::new()
        .read(true)
        .write(true)
        .share_mode(0)
        .custom_flags(FILE_FLAG_WRITE_THROUGH)
        .open(path)
}

pub fn device_size(file: &mut File) -> io::Result<u64> {
    use std::os::windows::io::AsRawHandle;

    let mut length = GET_LENGTH_INFORMATION { Length: 0 };
    let mut returned: u32 = 0;
    let ok: BOOL = unsafe {
        DeviceIoControl(
            file.as_raw_handle(),
            IOCTL_DISK_GET_LENGTH_INFO,
            std::ptr::null(),
            0,
            &mut length as *mut _ as *mut std::ffi::c_void,
            std::mem::size_of::<GET_LENGTH_INFORMATION>() as u32,
            &mut returned,
            std::ptr::null_mut(),
        )
    };
    if ok == FALSE {
        return Err(io::Error::last_os_error());
    }
    if length.Length < 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid device length",
        ));
    }
    Ok(length.Length as u64)
}
