use std::fs::{File, OpenOptions};
use std::io::{self, Seek, SeekFrom};
use std::path::Path;

pub fn is_block_device(_path: &Path) -> bool {
    false
}

pub fn is_system_root(_path: &Path) -> bool {
    false
}

pub fn open_device(path: &Path) -> io::Result<File> {
    OpenOptions::new().read(true).write(true).open(path)
}

pub fn device_size(file: &mut File) -> io::Result<u64> {
    file.seek(SeekFrom::End(0))
}
