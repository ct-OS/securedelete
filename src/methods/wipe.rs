use rand::RngCore;

use super::WipeMethod;

/// The 27 fixed patterns used by the "static" mode of Thomas M. Vier's
/// "ya-wipe" tool (wipe_passes() in wipe.c), taken from Peter Gutmann's
/// 1996 USENIX paper. Each pattern repeats across the whole medium.
pub struct WipeStatic;

const WIPE27: &[&[u8]] = &[
    &[0x55],
    &[0xAA],
    &[0x92, 0x49, 0x24],
    &[0x49, 0x24, 0x92],
    &[0x24, 0x92, 0x49],
    &[0x00],
    &[0x11],
    &[0x22],
    &[0x33],
    &[0x44],
    &[0x55],
    &[0x66],
    &[0x77],
    &[0x88],
    &[0x99],
    &[0xAA],
    &[0xBB],
    &[0xCC],
    &[0xDD],
    &[0xEE],
    &[0xFF],
    &[0x92, 0x49, 0x24],
    &[0x49, 0x24, 0x92],
    &[0x24, 0x92, 0x49],
    &[0x6D, 0xB6, 0xDB],
    &[0xB6, 0xDB, 0x6D],
    &[0xDB, 0x6D, 0xB6],
];

impl WipeMethod for WipeStatic {
    fn name(&self) -> &str {
        "ya-wipe static patterns (27 passes)"
    }

    fn passes(&self) -> usize {
        WIPE27.len()
    }

    fn fill_buffer(&self, pass: usize, _seed: u64, buffer: &mut [u8], _rng: &mut dyn RngCore) {
        let pattern = WIPE27.get(pass).copied().unwrap_or(&[0x00]);
        for (i, byte) in buffer.iter_mut().enumerate() {
            *byte = pattern[i % pattern.len()];
        }
    }
}
