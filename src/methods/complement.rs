use std::sync::Mutex;

use rand::RngCore;

use super::WipeMethod;

const RANDOMC_NAME: &str = "Random + complement (2 passes)";
const RANDOMCC_NAME: &str = "Random/complement cycle (3 passes)";
const NAVY_MFM_NAME: &str = "US Navy NAVSO P-5239-26 MFM (3 passes)";

/// Wipe method that alternates random passes with the bitwise complement of
/// the data written by the previous random pass.
pub struct RandomComplement {
    name: &'static str,
    passes: usize,
    last: Mutex<Vec<u8>>,
}

impl RandomComplement {
    pub fn new(name: &'static str, passes: usize) -> Self {
        assert!(passes >= 2);
        Self {
            name,
            passes,
            last: Mutex::new(Vec::new()),
        }
    }

    pub fn inverse() -> Self {
        Self::new(RANDOMC_NAME, 2)
    }

    pub fn inverse_cycle() -> Self {
        Self::new(RANDOMCC_NAME, 3)
    }

    pub fn navy_mfm() -> Self {
        Self::new(NAVY_MFM_NAME, 3)
    }
}

impl WipeMethod for RandomComplement {
    fn name(&self) -> &str {
        self.name
    }

    fn passes(&self) -> usize {
        self.passes
    }

    fn fill_buffer(&self, pass: usize, buffer: &mut [u8], rng: &mut dyn RngCore) {
        match pass {
            0 => {
                rng.fill_bytes(buffer);
                let mut last = self.last.lock().unwrap();
                last.clear();
                last.extend_from_slice(buffer);
            }
            1 => {
                let last = self.last.lock().unwrap();
                if last.len() == buffer.len() {
                    for (out, &b) in buffer.iter_mut().zip(last.iter()) {
                        *out = !b;
                    }
                } else {
                    rng.fill_bytes(buffer);
                }
            }
            _ => rng.fill_bytes(buffer),
        }
    }
}

const NIST_NAME: &str = "NIST SP 800-88 Purge (3 passes)";

/// NIST SP 800-88 Rev 1 optional ATA/SCSI Purge: a pseudorandom pattern,
/// its bitwise inversion, then the same pattern again.
pub struct NistPurge {
    last: Mutex<Vec<u8>>,
}

impl NistPurge {
    pub fn new() -> Self {
        Self {
            last: Mutex::new(Vec::new()),
        }
    }
}

impl Default for NistPurge {
    fn default() -> Self {
        Self::new()
    }
}

impl WipeMethod for NistPurge {
    fn name(&self) -> &str {
        NIST_NAME
    }

    fn passes(&self) -> usize {
        3
    }

    fn fill_buffer(&self, pass: usize, buffer: &mut [u8], rng: &mut dyn RngCore) {
        match pass {
            0 => {
                rng.fill_bytes(buffer);
                let mut last = self.last.lock().unwrap();
                last.clear();
                last.extend_from_slice(buffer);
            }
            1 => {
                let last = self.last.lock().unwrap();
                if last.len() == buffer.len() {
                    for (out, &b) in buffer.iter_mut().zip(last.iter()) {
                        *out = !b;
                    }
                } else {
                    rng.fill_bytes(buffer);
                }
            }
            _ => {
                let last = self.last.lock().unwrap();
                if last.len() == buffer.len() {
                    buffer.copy_from_slice(&last);
                } else {
                    rng.fill_bytes(buffer);
                }
            }
        }
    }
}
