use std::fmt;

use rand::RngCore;

pub trait WipeMethod: Send + Sync {
    fn name(&self) -> &str;
    fn passes(&self) -> usize;
    fn fill_buffer(&self, pass: usize, buffer: &mut [u8], rng: &mut dyn RngCore);
}

impl fmt::Display for dyn WipeMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

pub mod dod;
pub mod gutmann;
pub mod random;
pub mod zero;

pub fn get_method(name: &str, passes_override: Option<usize>) -> Option<Box<dyn WipeMethod>> {
    let passes_override = passes_override.filter(|&n| n > 0);
    match name.to_lowercase().as_str() {
        "zeros" | "zero" => Some(Box::new(zero::ZeroPass)),
        "random" => Some(Box::new(random::RandomPass::new(
            passes_override.unwrap_or(1),
        ))),
        "dod" | "dod522022m" => Some(Box::new(dod::DoD522022M)),
        "gutmann" => Some(Box::new(gutmann::Gutmann)),
        _ => {
            let lower = name.to_lowercase();
            let n = lower.strip_prefix("random:")?;
            let passes: usize = n.parse().ok()?;
            if passes == 0 {
                return None;
            }
            Some(Box::new(random::RandomPass::new(passes)))
        }
    }
}
