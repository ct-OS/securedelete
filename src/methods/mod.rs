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

pub mod zero;

pub fn get_method(name: &str) -> Option<Box<dyn WipeMethod>> {
    match name {
        "zeros" | "zero" => Some(Box::new(zero::ZeroPass)),
        _ => None,
    }
}
