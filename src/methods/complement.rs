use rand::rngs::StdRng;
use rand::RngCore;
use rand::SeedableRng;

use super::{pass_seed, WipeMethod};

const RANDOMC_NAME: &str = "Random + complement (2 passes)";
const RANDOMCC_NAME: &str = "Random/complement cycle (3 passes)";
const NAVY_MFM_NAME: &str = "US Navy NAVSO P-5239-26 MFM (3 passes)";

/// Wipe method that alternates random passes with the bitwise complement of
/// the data written by the previous random pass.
///
/// Stateless by design: the complement pass regenerates the previous pass's
/// random stream from the wipe seed via [`pass_seed`], so the wiper can replay
/// any pass deterministically (verification) and the method works correctly
/// regardless of the buffer chunking.
pub struct RandomComplement {
    name: &'static str,
    passes: usize,
}

impl RandomComplement {
    pub fn new(name: &'static str, passes: usize) -> Self {
        assert!(passes >= 2);
        Self { name, passes }
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

    fn fill_buffer(&self, pass: usize, seed: u64, buffer: &mut [u8], rng: &mut dyn RngCore) {
        match pass {
            0 => rng.fill_bytes(buffer),
            1 => {
                let mut replay = StdRng::seed_from_u64(pass_seed(seed, 0));
                let mut previous = vec![0u8; buffer.len()];
                replay.fill_bytes(&mut previous);
                for (out, &byte) in buffer.iter_mut().zip(previous.iter()) {
                    *out = !byte;
                }
            }
            _ => rng.fill_bytes(buffer),
        }
    }
}

const NIST_NAME: &str = "NIST SP 800-88 Purge (3 passes)";

/// NIST SP 800-88 Rev 1 optional ATA/SCSI Purge: a pseudorandom pattern,
/// its bitwise inversion, then the same pattern again. Stateless: passes 1
/// and 2 regenerate the pattern of pass 0 from the wipe seed.
pub struct NistPurge;

impl WipeMethod for NistPurge {
    fn name(&self) -> &str {
        NIST_NAME
    }

    fn passes(&self) -> usize {
        3
    }

    fn fill_buffer(&self, pass: usize, seed: u64, buffer: &mut [u8], rng: &mut dyn RngCore) {
        let mut replay = StdRng::seed_from_u64(pass_seed(seed, 0));
        match pass {
            0 => rng.fill_bytes(buffer),
            1 => {
                let mut previous = vec![0u8; buffer.len()];
                replay.fill_bytes(&mut previous);
                for (out, &byte) in buffer.iter_mut().zip(previous.iter()) {
                    *out = !byte;
                }
            }
            2 => {
                let mut previous = vec![0u8; buffer.len()];
                replay.fill_bytes(&mut previous);
                buffer.copy_from_slice(&previous);
            }
            _ => buffer.fill(0),
        }
    }
}
