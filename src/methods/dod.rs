use rand::RngCore;

use super::WipeMethod;

pub struct DoD522022M;

impl WipeMethod for DoD522022M {
    fn name(&self) -> &str {
        "DoD 5220.22-M (3 passes)"
    }

    fn passes(&self) -> usize {
        3
    }

    fn fill_buffer(&self, pass: usize, _seed: u64, buffer: &mut [u8], rng: &mut dyn RngCore) {
        match pass {
            0 => buffer.fill(0x00),
            1 => buffer.fill(0xFF),
            _ => rng.fill_bytes(buffer),
        }
    }
}
