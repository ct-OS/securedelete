use rand::RngCore;

use super::WipeMethod;

pub struct RandomPass {
    passes: usize,
    name: &'static str,
}

impl RandomPass {
    pub fn new(passes: usize) -> Self {
        Self::named(passes, "Random data")
    }

    pub const fn named(passes: usize, name: &'static str) -> Self {
        Self { passes, name }
    }
}

pub const USARMY: RandomPass = RandomPass::named(2, "US Army AR 380-19 (2 random passes)");
pub const NAVY: RandomPass = RandomPass::named(3, "US Navy NAVSO P-5239-26 RLL (3 random passes)");
pub const PFITZNER: RandomPass = RandomPass::named(33, "Pfitzner (33 random passes)");

impl WipeMethod for RandomPass {
    fn name(&self) -> &str {
        self.name
    }

    fn passes(&self) -> usize {
        self.passes
    }

    fn fill_buffer(&self, _pass: usize, _seed: u64, buffer: &mut [u8], rng: &mut dyn RngCore) {
        rng.fill_bytes(buffer);
    }
}
