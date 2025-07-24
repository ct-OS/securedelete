use std::fs::{self, OpenOptions};
use std::io::{self, Seek, SeekFrom, Write};
use std::path::Path;

use crate::methods::WipeMethod;

const BUFFER_SIZE: usize = 1024 * 1024;

pub fn wipe_file(path: &Path, method: &dyn WipeMethod) -> io::Result<()> {
    let size = fs::metadata(path)?.len();

    if size == 0 {
        return fs::remove_file(path);
    }

    let mut file = OpenOptions::new().write(true).open(path)?;
    let mut rng = rand::rng();
    let mut buffer = vec![0u8; BUFFER_SIZE];

    for pass in 0..method.passes() {
        println!("pass {}/{} - {}", pass + 1, method.passes(), method.name());
        file.seek(SeekFrom::Start(0))?;

        let mut remaining = size;
        while remaining > 0 {
            let chunk_size = BUFFER_SIZE.min(remaining as usize);
            method.fill_buffer(pass, &mut buffer[..chunk_size], &mut rng);
            file.write_all(&buffer[..chunk_size])?;
            remaining -= chunk_size as u64;
        }
        file.sync_all()?;
    }

    // Truncate and flush so no residual data remains after unlink.
    file.set_len(0)?;
    file.sync_all()?;
    drop(file);

    fs::remove_file(path)
}
