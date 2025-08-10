use std::fs::{self, OpenOptions};
use std::io::{self, Seek, SeekFrom, Write};
use std::path::Path;

use indicatif::{ProgressBar, ProgressStyle};

use crate::methods::WipeMethod;

const BUFFER_SIZE: usize = 1024 * 1024;

fn progress_style() -> ProgressStyle {
    ProgressStyle::with_template(
        "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}",
    )
    .unwrap()
    .progress_chars("#>-")
}

fn new_progress(size: u64) -> ProgressBar {
    let pb = ProgressBar::new(size);
    pb.set_style(progress_style());
    pb
}

fn pass_message(method: &dyn WipeMethod, pass: usize) -> String {
    format!("pass {}/{} - {}", pass + 1, method.passes(), method.name())
}

pub fn wipe_file(path: &Path, method: &dyn WipeMethod) -> io::Result<()> {
    let size = fs::metadata(path)?.len();

    if size == 0 {
        return fs::remove_file(path);
    }

    let mut file = OpenOptions::new().write(true).open(path)?;
    let pb = new_progress(size);
    let mut rng = rand::rng();
    let mut buffer = vec![0u8; BUFFER_SIZE];

    for pass in 0..method.passes() {
        pb.set_message(pass_message(method, pass));
        pb.set_position(0);
        file.seek(SeekFrom::Start(0))?;

        let mut remaining = size;
        while remaining > 0 {
            let chunk_size = BUFFER_SIZE.min(remaining as usize);
            method.fill_buffer(pass, &mut buffer[..chunk_size], &mut rng);
            file.write_all(&buffer[..chunk_size])?;
            remaining -= chunk_size as u64;
            pb.set_position(size - remaining);
        }
        file.sync_all()?;
    }

    pb.finish_and_clear();

    // Truncate and flush so no residual data remains after unlink.
    file.set_len(0)?;
    file.sync_all()?;
    drop(file);

    fs::remove_file(path)
}

pub fn wipe_device(path: &Path, method: &dyn WipeMethod) -> io::Result<()> {
    let mut device = crate::platform::open_device(path)?;
    let device_size = crate::platform::device_size(&mut device)?;

    if device_size == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "device size is 0 or cannot be determined",
        ));
    }

    let pb = new_progress(device_size);
    let mut rng = rand::rng();
    let mut buffer = vec![0u8; BUFFER_SIZE];

    for pass in 0..method.passes() {
        pb.set_message(pass_message(method, pass));
        pb.set_position(0);
        device.seek(SeekFrom::Start(0))?;

        let mut written: u64 = 0;
        while written < device_size {
            let chunk_size = BUFFER_SIZE.min((device_size - written) as usize);
            method.fill_buffer(pass, &mut buffer[..chunk_size], &mut rng);
            device.write_all(&buffer[..chunk_size])?;
            written += chunk_size as u64;
            pb.set_position(written);
        }
        device.sync_all()?;
    }

    pb.finish_and_clear();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::methods::zero::ZeroPass;
    use std::path::PathBuf;

    fn temp_path(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "secure_delete_test_{}_{}",
            std::process::id(),
            name
        ));
        let _ = fs::remove_file(&path);
        path
    }

    #[test]
    fn wipe_device_zeroes_content() {
        let path = temp_path("device_zero");
        let content = b"hello world this is a secret";
        fs::write(&path, content).unwrap();

        wipe_device(&path, &ZeroPass).unwrap();

        let data = fs::read(&path).unwrap();
        assert_eq!(data.len(), content.len());
        assert!(data.iter().all(|&b| b == 0));

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn wipe_device_preserves_size() {
        let path = temp_path("device_size");
        fs::write(&path, vec![0xABu8; 2_500_000]).unwrap();

        wipe_device(&path, &ZeroPass).unwrap();

        assert_eq!(fs::metadata(&path).unwrap().len(), 2_500_000);
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn wipe_file_removes_file() {
        let path = temp_path("file_remove");
        fs::write(&path, b"secret").unwrap();

        wipe_file(&path, &ZeroPass).unwrap();

        assert!(!path.exists());
    }

    #[test]
    fn wipe_file_removes_empty_file() {
        let path = temp_path("file_empty");
        fs::write(&path, []).unwrap();

        wipe_file(&path, &ZeroPass).unwrap();

        assert!(!path.exists());
    }
}
