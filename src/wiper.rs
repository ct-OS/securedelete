use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;

use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::methods::{pass_seed, WipeMethod};

const BUFFER_SIZE: usize = 1024 * 1024;

/// Options controlling a wipe operation.
#[derive(Default)]
pub struct WipeOptions {
    /// Read back and compare the final pass after wiping.
    pub verify: bool,
}

/// Receives progress updates from a running wipe. The CLI renders them with
/// a terminal progress bar; a GUI can forward them to its event loop.
///
/// `set_position` counts bytes within the current phase (a pass or the
/// verification), which always spans `total` bytes.
pub trait Progress {
    /// Called once before any I/O with the bytes processed per phase.
    fn start(&self, _total: u64) {}
    /// A new phase begins (e.g. "pass 2/3 - DoD 5220.22-M" or "verifying").
    fn set_message(&self, _msg: &str) {}
    fn set_position(&self, _pos: u64) {}
    /// Called once when the wipe stops, successfully or not.
    fn finish(&self) {}
}

/// A [`Progress`] that ignores all updates.
pub struct NoProgress;

impl Progress for NoProgress {}

fn pass_message(method: &dyn WipeMethod, pass: usize) -> String {
    format!("pass {}/{} - {}", pass + 1, method.passes(), method.name())
}

/// `fdatasync` on Unix (skips metadata, enough for unchanged sizes), `fsync`
/// elsewhere.
fn sync_data(file: &File) -> io::Result<()> {
    #[cfg(unix)]
    {
        file.sync_data()
    }
    #[cfg(not(unix))]
    {
        file.sync_all()
    }
}

fn interrupted() -> bool {
    crate::signals::interrupted()
}

fn interrupted_error() -> io::Error {
    io::Error::new(io::ErrorKind::Interrupted, "interrupted by user (Ctrl-C)")
}

fn with_path(path: &Path, e: io::Error) -> io::Error {
    io::Error::new(e.kind(), format!("{}: {}", path.display(), e))
}

/// Make the unlink durable by fsyncing the parent directory (Unix).
fn sync_parent_dir(path: &Path) {
    #[cfg(unix)]
    if let Some(parent) = path.parent() {
        if let Ok(dir) = File::open(parent) {
            let _ = dir.sync_all();
        }
    }
    #[cfg(not(unix))]
    let _ = path;
}

/// Data extents `(start, length)` to wipe. Falls back to the whole file when
/// the filesystem does not report holes (SEEK_DATA/SEEK_HOLE unavailable).
fn wipe_ranges(file: &mut File, size: u64) -> io::Result<Vec<(u64, u64)>> {
    match crate::platform::data_ranges(file, size)? {
        Some(ranges) => Ok(ranges),
        None => Ok(vec![(0, size)]),
    }
}

fn ranges_total(ranges: &[(u64, u64)]) -> u64 {
    ranges.iter().map(|&(_, len)| len).sum()
}

fn wipe_passes(
    file: &mut File,
    ranges: &[(u64, u64)],
    method: &dyn WipeMethod,
    seed: u64,
    pb: &dyn Progress,
    buffer: &mut [u8],
) -> io::Result<()> {
    for pass in 0..method.passes() {
        if interrupted() {
            return Err(interrupted_error());
        }
        pb.set_message(&pass_message(method, pass));
        pb.set_position(0);
        let mut rng = StdRng::seed_from_u64(pass_seed(seed, pass));
        let mut done: u64 = 0;
        for &(start, len) in ranges {
            file.seek(SeekFrom::Start(start))?;
            let mut remaining = len;
            while remaining > 0 {
                if interrupted() {
                    return Err(interrupted_error());
                }
                let chunk = BUFFER_SIZE.min(remaining as usize);
                method.fill_buffer(pass, seed, &mut buffer[..chunk], &mut rng);
                file.write_all(&buffer[..chunk])?;
                remaining -= chunk as u64;
                pb.set_position(done + (len - remaining));
            }
            done += len;
        }
        sync_data(file)?;
    }
    Ok(())
}

fn verify_last_pass(
    file: &mut File,
    method: &dyn WipeMethod,
    seed: u64,
    ranges: &[(u64, u64)],
    pb: &dyn Progress,
) -> io::Result<()> {
    let last = method.passes() - 1;
    let mut rng = StdRng::seed_from_u64(pass_seed(seed, last));
    let mut expected = vec![0u8; BUFFER_SIZE];
    let mut actual = vec![0u8; BUFFER_SIZE];
    let mut done: u64 = 0;
    for &(start, len) in ranges {
        file.seek(SeekFrom::Start(start))?;
        let mut remaining = len;
        while remaining > 0 {
            if interrupted() {
                return Err(interrupted_error());
            }
            let chunk = BUFFER_SIZE.min(remaining as usize);
            method.fill_buffer(last, seed, &mut expected[..chunk], &mut rng);
            file.read_exact(&mut actual[..chunk])?;
            if expected[..chunk] != actual[..chunk] {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "verification failed at offset {} (pass {}): data mismatch",
                        start + (len - remaining),
                        last + 1
                    ),
                ));
            }
            remaining -= chunk as u64;
            pb.set_position(done + (len - remaining));
        }
        done += len;
    }
    Ok(())
}

/// Overwrite a regular file with `method`, then truncate and delete it.
pub fn wipe_file(
    path: &Path,
    method: &dyn WipeMethod,
    opts: &WipeOptions,
    pb: &dyn Progress,
) -> io::Result<()> {
    wipe_file_inner(path, method, opts, pb).map_err(|e| with_path(path, e))
}

fn wipe_file_inner(
    path: &Path,
    method: &dyn WipeMethod,
    opts: &WipeOptions,
    pb: &dyn Progress,
) -> io::Result<()> {
    let size = fs::metadata(path)?.len();
    if size == 0 {
        fs::remove_file(path)?;
        sync_parent_dir(path);
        return Ok(());
    }

    crate::platform::ensure_writable(path)?;
    // O_NOFOLLOW closes the TOCTOU race between validation and open.
    let mut file = crate::platform::open_file_no_follow(path, opts.verify, true)?;
    let ranges = wipe_ranges(&mut file, size)?;
    let total = ranges_total(&ranges);
    let seed: u64 = rand::random();

    pb.start(total);
    let mut buffer = vec![0u8; BUFFER_SIZE];

    let result = (|| -> io::Result<()> {
        wipe_passes(&mut file, &ranges, method, seed, pb, &mut buffer)?;
        if opts.verify {
            pb.set_message("verifying");
            pb.set_position(0);
            verify_last_pass(&mut file, method, seed, &ranges, pb)?;
        }
        Ok(())
    })();
    pb.finish();
    result?;

    // Truncate and flush so no residual data remains after unlink.
    file.set_len(0)?;
    sync_data(&file)?;
    drop(file);

    fs::remove_file(path)?;
    sync_parent_dir(path);
    Ok(())
}

/// Overwrite a whole block device (or any file, keeping it) with `method`.
pub fn wipe_device(
    path: &Path,
    method: &dyn WipeMethod,
    opts: &WipeOptions,
    pb: &dyn Progress,
) -> io::Result<()> {
    wipe_device_inner(path, method, opts, pb).map_err(|e| with_path(path, e))
}

fn wipe_device_inner(
    path: &Path,
    method: &dyn WipeMethod,
    opts: &WipeOptions,
    pb: &dyn Progress,
) -> io::Result<()> {
    let mut device = crate::platform::open_device(path)?;
    let size = crate::platform::device_size(&mut device)?;
    if size == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "device size is 0 or cannot be determined",
        ));
    }

    let ranges = vec![(0u64, size)];
    let seed: u64 = rand::random();
    pb.start(size);
    let mut buffer = vec![0u8; BUFFER_SIZE];

    let result = (|| -> io::Result<()> {
        wipe_passes(&mut device, &ranges, method, seed, pb, &mut buffer)?;
        if opts.verify {
            pb.set_message("verifying");
            pb.set_position(0);
            verify_last_pass(&mut device, method, seed, &ranges, pb)?;
        }
        Ok(())
    })();
    pb.finish();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::methods::random::RandomPass;
    use crate::methods::zero::ZeroPass;
    use std::fs::OpenOptions;
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

        wipe_device(&path, &ZeroPass, &WipeOptions::default(), &NoProgress).unwrap();

        let data = fs::read(&path).unwrap();
        assert_eq!(data.len(), content.len());
        assert!(data.iter().all(|&b| b == 0));

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn wipe_device_preserves_size() {
        let path = temp_path("device_size");
        fs::write(&path, vec![0xABu8; 2_500_000]).unwrap();

        wipe_device(&path, &ZeroPass, &WipeOptions::default(), &NoProgress).unwrap();

        assert_eq!(fs::metadata(&path).unwrap().len(), 2_500_000);
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn wipe_device_verify_ok() {
        let path = temp_path("verify_ok");
        fs::write(&path, vec![0xABu8; 3_000_000]).unwrap();

        let opts = WipeOptions { verify: true };
        wipe_device(&path, &crate::methods::standards::ONES, &opts, &NoProgress).unwrap();

        let data = fs::read(&path).unwrap();
        assert!(data.iter().all(|&b| b == 0xFF));
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn wipe_file_removes_file() {
        let path = temp_path("file_remove");
        fs::write(&path, b"secret").unwrap();

        wipe_file(&path, &ZeroPass, &WipeOptions::default(), &NoProgress).unwrap();

        assert!(!path.exists());
    }

    #[test]
    fn wipe_file_removes_empty_file() {
        let path = temp_path("file_empty");
        fs::write(&path, []).unwrap();

        wipe_file(&path, &ZeroPass, &WipeOptions::default(), &NoProgress).unwrap();

        assert!(!path.exists());
    }

    #[test]
    fn wipe_file_sparse_completes() {
        let path = temp_path("sparse_wipe");
        let f = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .unwrap();
        f.set_len(64 * 1024 * 1024).unwrap();
        drop(f);

        wipe_file(&path, &ZeroPass, &WipeOptions::default(), &NoProgress).unwrap();

        assert!(!path.exists());
    }

    #[test]
    fn verify_passes_clean() {
        let path = temp_path("verify_clean");
        let size = 2_500_000u64;
        fs::write(&path, vec![0xABu8; size as usize]).unwrap();
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();
        let ranges = vec![(0u64, size)];
        let seed: u64 = 0x1234_5678;
        let method = RandomPass::new(3);
        let pb = NoProgress;
        let mut buffer = vec![0u8; BUFFER_SIZE];

        wipe_passes(&mut file, &ranges, &method, seed, &pb, &mut buffer).unwrap();
        assert!(verify_last_pass(&mut file, &method, seed, &ranges, &pb).is_ok());

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn verify_detects_corruption() {
        let path = temp_path("verify_bad");
        let size = 2_500_000u64;
        fs::write(&path, vec![0xABu8; size as usize]).unwrap();
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();
        let ranges = vec![(0u64, size)];
        let seed: u64 = 0x1234_5678;
        let method = RandomPass::new(3);
        let pb = NoProgress;
        let mut buffer = vec![0u8; BUFFER_SIZE];

        wipe_passes(&mut file, &ranges, &method, seed, &pb, &mut buffer).unwrap();
        file.seek(SeekFrom::Start(1_000_000)).unwrap();
        file.write_all(&[0x00]).unwrap();
        assert!(verify_last_pass(&mut file, &method, seed, &ranges, &pb).is_err());

        fs::remove_file(&path).unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn data_ranges_finds_extents() {
        let path = temp_path("sparse_ranges");
        let mut f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .unwrap();
        f.set_len(10 * 1024 * 1024).unwrap();
        f.seek(SeekFrom::Start(4 * 1024 * 1024)).unwrap();
        f.write_all(&[0x42; 4096]).unwrap();
        f.sync_all().unwrap();

        let ranges = crate::platform::data_ranges(&f, 10 * 1024 * 1024)
            .unwrap()
            .unwrap();
        assert_eq!(ranges, vec![(4 * 1024 * 1024, 4096)]);

        fs::remove_file(&path).unwrap();
    }
}
