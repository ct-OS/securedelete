use rand::RngCore;

use super::WipeMethod;

pub struct ZeroPass;

impl WipeMethod for ZeroPass {
    fn name(&self) -> &str {
        "Zero fill (1 pass)"
    }

    fn passes(&self) -> usize {
        1
    }

    fn fill_buffer(&self, _pass: usize, _seed: u64, buffer: &mut [u8], _rng: &mut dyn RngCore) {
        buffer.fill(0x00);
    }
}
