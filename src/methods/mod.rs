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

pub mod complement;
pub mod dod;
pub mod gutmann;
pub mod random;
pub mod standards;
pub mod wipe;
pub mod zero;

pub fn get_all_methods() -> Vec<Box<dyn WipeMethod>> {
    vec![
        Box::new(zero::ZeroPass),
        Box::new(standards::ONES),
        Box::new(random::RandomPass::new(1)),
        Box::new(random::USARMY),
        Box::new(random::NAVY),
        Box::new(standards::USAF),
        Box::new(dod::DoD522022M),
        Box::new(standards::DOD7),
        Box::new(standards::HMG),
        Box::new(standards::HMG5),
        Box::new(standards::RCMP),
        Box::new(standards::VSITR),
        Box::new(standards::GOST),
        Box::new(standards::NSA1301),
        Box::new(standards::SCHNEIER),
        Box::new(standards::USERM),
        Box::new(standards::USERM2),
        Box::new(gutmann::Gutmann),
        Box::new(complement::RandomComplement::inverse()),
        Box::new(complement::RandomComplement::inverse_cycle()),
        Box::new(complement::RandomComplement::navy_mfm()),
        Box::new(standards::BMB21),
        Box::new(standards::BSI9),
        Box::new(standards::DOE),
        Box::new(standards::RCMP7),
        Box::new(random::PFITZNER),
        Box::new(complement::NistPurge::new()),
        Box::new(wipe::WipeStatic),
        Box::new(standards::ARM38019),
        Box::new(standards::AFSSI5020_OFFICIAL),
        Box::new(standards::NSA1302),
        Box::new(standards::VSITR_OFFICIAL),
        Box::new(standards::HMG_BASELINE),
    ]
}

pub fn get_method(name: &str, passes_override: Option<usize>) -> Option<Box<dyn WipeMethod>> {
    let passes_override = passes_override.filter(|&n| n > 0);
    match name.to_lowercase().as_str() {
        "zeros" | "zero" => Some(Box::new(zero::ZeroPass)),
        "ones" => Some(Box::new(standards::ONES)),
        "random" => Some(Box::new(random::RandomPass::new(
            passes_override.unwrap_or(1),
        ))),
        "usarmy" | "army38019" => Some(Box::new(random::USARMY)),
        "navy" | "navso" | "navso5239" => Some(Box::new(random::NAVY)),
        "usaf" | "afssi5020" => Some(Box::new(standards::USAF)),
        "dod" | "dod522022m" => Some(Box::new(dod::DoD522022M)),
        "dod7" | "dod522022m7" | "dodece" => Some(Box::new(standards::DOD7)),
        "hmg" | "isc5" => Some(Box::new(standards::HMG)),
        "hmg5" | "isc5e" | "hmg_enhanced" => Some(Box::new(standards::HMG5)),
        "rcmp" | "tssit" | "opsii" => Some(Box::new(standards::RCMP)),
        "vsitr" => Some(Box::new(standards::VSITR)),
        "gost" | "gost50739" => Some(Box::new(standards::GOST)),
        "nsa" | "nsa1301" => Some(Box::new(standards::NSA1301)),
        "schneier" => Some(Box::new(standards::SCHNEIER)),
        "userm" => Some(Box::new(standards::USERM)),
        "userm2" => Some(Box::new(standards::USERM2)),
        "randomc" | "random_complement" => Some(Box::new(complement::RandomComplement::inverse())),
        "randomcc" | "complement_cycle" => {
            Some(Box::new(complement::RandomComplement::inverse_cycle()))
        }
        "navy_mfm" | "navso_mfm" | "navso5239mfm" => {
            Some(Box::new(complement::RandomComplement::navy_mfm()))
        }
        "gutmann" => Some(Box::new(gutmann::Gutmann)),
        "bmb21" | "bundeswehr" => Some(Box::new(standards::BMB21)),
        "bsi" | "bsi9" => Some(Box::new(standards::BSI9)),
        "doe" | "doe20512" => Some(Box::new(standards::DOE)),
        "rcmp7" | "b2002" => Some(Box::new(standards::RCMP7)),
        "pfitzner" | "pfitzner33" => Some(Box::new(random::PFITZNER)),
        "nist" | "nist80088" => Some(Box::new(complement::NistPurge::new())),
        "wipe27" | "wipestatic" => Some(Box::new(wipe::WipeStatic)),
        "arm38019" | "usarmy_official" | "army3" => Some(Box::new(standards::ARM38019)),
        "afssi5020_official" | "usaf_official" => Some(Box::new(standards::AFSSI5020_OFFICIAL)),
        "nsa1302" | "nsa_official" => Some(Box::new(standards::NSA1302)),
        "vsitr_official" | "vsitr_bsi" => Some(Box::new(standards::VSITR_OFFICIAL)),
        "hmg_baseline" | "isc5_base" | "hmg1" => Some(Box::new(standards::HMG_BASELINE)),
        _ => {
            let lower = name.to_lowercase();
            if let Some(hex) = lower
                .strip_prefix("fill:")
                .or_else(|| lower.strip_prefix("pattern:"))
            {
                let hex = hex.strip_prefix("0x").unwrap_or(hex);
                let byte = u8::from_str_radix(hex, 16).ok()?;
                return Some(Box::new(standards::DynamicFill::new(byte)));
            }
            let n = lower.strip_prefix("random:")?;
            let passes: usize = n.parse().ok()?;
            if passes == 0 {
                return None;
            }
            Some(Box::new(random::RandomPass::new(passes)))
        }
    }
}
