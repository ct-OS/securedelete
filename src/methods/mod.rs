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

#[cfg(test)]
mod tests {
    use super::*;
    use dod::DoD522022M;
    use gutmann::Gutmann;
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use zero::ZeroPass;

    fn seeded_rng() -> StdRng {
        StdRng::seed_from_u64(42)
    }

    #[test]
    fn zero_fills_all_zeroes() {
        let mut buffer = [0xFFu8; 100];
        ZeroPass.fill_buffer(0, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().all(|&b| b == 0));
    }

    #[test]
    fn random_uses_rng() {
        let mut buffer = [0u8; 64];
        random::RandomPass::new(1).fill_buffer(0, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().any(|&b| b != 0));
    }

    #[test]
    fn random_is_deterministic_for_same_seed() {
        let mut a = [0u8; 64];
        let mut b = [0u8; 64];
        random::RandomPass::new(1).fill_buffer(0, &mut a, &mut seeded_rng());
        random::RandomPass::new(1).fill_buffer(0, &mut b, &mut seeded_rng());
        assert_eq!(a, b);
    }

    #[test]
    fn dod_sequences() {
        let mut buffer = [0u8; 16];
        DoD522022M.fill_buffer(0, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().all(|&b| b == 0x00));

        DoD522022M.fill_buffer(1, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().all(|&b| b == 0xFF));

        DoD522022M.fill_buffer(2, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().any(|&b| b != 0xFF));
    }

    #[test]
    fn ones_fills_all_ones() {
        let mut buffer = [0u8; 16];
        standards::ONES.fill_buffer(0, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().all(|&b| b == 0xFF));
    }

    #[test]
    fn usaf_sequences() {
        let mut buffer = [0u8; 16];
        standards::USAF.fill_buffer(0, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().all(|&b| b == 0x00));

        standards::USAF.fill_buffer(1, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().all(|&b| b == 0xFF));

        standards::USAF.fill_buffer(2, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().all(|&b| b == 0x00));
    }

    #[test]
    fn hmg_random_last_pass() {
        let mut buffer = [0u8; 64];
        standards::HMG.fill_buffer(2, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().any(|&b| b != 0));
    }

    #[test]
    fn dod7_sequences() {
        let mut buffer = [0u8; 16];
        for pass in 0..standards::DOD7.passes() {
            standards::DOD7.fill_buffer(pass, &mut buffer, &mut seeded_rng());
            match pass {
                0 | 3 | 6 => assert!(buffer.iter().all(|&b| b == 0x00)),
                1 | 4 => assert!(buffer.iter().all(|&b| b == 0xFF)),
                _ => assert!(buffer.iter().any(|&b| b != 0x00 && b != 0xFF)),
            }
        }
    }

    #[test]
    fn schneier_sequence() {
        let mut buffer = [0u8; 16];
        standards::SCHNEIER.fill_buffer(0, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().all(|&b| b == 0xFF));

        standards::SCHNEIER.fill_buffer(6, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().all(|&b| b == 0xFF));

        standards::SCHNEIER.fill_buffer(2, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().any(|&b| b != 0xFF));
    }

    #[test]
    fn gost_sequences() {
        let mut buffer = [0u8; 16];
        for pass in 0..standards::GOST.passes() {
            standards::GOST.fill_buffer(pass, &mut buffer, &mut seeded_rng());
            assert!(buffer.iter().all(|&b| b == 0x00));
        }
    }

    #[test]
    fn nsa1301_sequences() {
        let mut buffer = [0u8; 16];
        for pass in 0..standards::NSA1301.passes() {
            standards::NSA1301.fill_buffer(pass, &mut buffer, &mut seeded_rng());
            assert!(buffer.iter().all(|&b| b == 0xFF));
        }
    }

    #[test]
    fn userm_sequences() {
        let mut buffer = [0u8; 16];
        for pass in 0..standards::USERM.passes() {
            standards::USERM.fill_buffer(pass, &mut buffer, &mut seeded_rng());
            match pass {
                0 | 2 => assert!(buffer.iter().all(|&b| b == 0xFF)),
                1 => assert!(buffer.iter().all(|&b| b == 0x00)),
                _ => assert!(buffer.iter().any(|&b| b != 0xFF)),
            }
        }
    }

    #[test]
    fn hmg5_sequences() {
        let mut buffer = [0u8; 16];
        for pass in 0..standards::HMG5.passes() {
            standards::HMG5.fill_buffer(pass, &mut buffer, &mut seeded_rng());
            match pass {
                0 | 3 => assert!(buffer.iter().all(|&b| b == 0x00)),
                1 | 4 => assert!(buffer.iter().all(|&b| b == 0xFF)),
                _ => assert!(buffer.iter().any(|&b| b != 0x00 && b != 0xFF)),
            }
        }
    }

    #[test]
    fn complement_inverts_previous_pass() {
        let mut buffer = [0u8; 64];
        let m = complement::RandomComplement::inverse();
        m.fill_buffer(0, &mut buffer, &mut seeded_rng());
        let original = buffer;

        m.fill_buffer(1, &mut buffer, &mut seeded_rng());
        for (i, &b) in original.iter().enumerate() {
            assert_eq!(buffer[i], !b);
        }
    }

    #[test]
    fn complement_cycle_counts() {
        assert_eq!(complement::RandomComplement::inverse().passes(), 2);
        assert_eq!(complement::RandomComplement::inverse_cycle().passes(), 3);
        assert_eq!(complement::RandomComplement::navy_mfm().passes(), 3);
    }

    #[test]
    fn dynamic_fill() {
        let m = standards::DynamicFill::new(0x5A);
        let mut buffer = [0u8; 16];
        m.fill_buffer(0, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().all(|&b| b == 0x5A));
        assert!(m.name().contains("0x5A"));
    }

    #[test]
    fn random_standards_pass_counts() {
        assert_eq!(random::USARMY.passes(), 2);
        assert_eq!(random::NAVY.passes(), 3);
        assert_eq!(random::PFITZNER.passes(), 33);
        assert_eq!(standards::ONES.passes(), 1);
        assert_eq!(standards::USAF.passes(), 3);
        assert_eq!(standards::HMG.passes(), 3);
        assert_eq!(standards::HMG5.passes(), 5);
        assert_eq!(standards::RCMP.passes(), 4);
        assert_eq!(standards::DOD7.passes(), 7);
        assert_eq!(standards::VSITR.passes(), 7);
        assert_eq!(standards::SCHNEIER.passes(), 7);
        assert_eq!(standards::GOST.passes(), 2);
        assert_eq!(standards::NSA1301.passes(), 2);
        assert_eq!(standards::USERM.passes(), 4);
        assert_eq!(standards::USERM2.passes(), 7);
        assert_eq!(standards::BMB21.passes(), 6);
        assert_eq!(standards::BSI9.passes(), 9);
        assert_eq!(standards::DOE.passes(), 3);
        assert_eq!(standards::RCMP7.passes(), 7);
        assert_eq!(complement::NistPurge::new().passes(), 3);
        assert_eq!(wipe::WipeStatic.passes(), 27);
        assert_eq!(standards::ARM38019.passes(), 3);
        assert_eq!(standards::AFSSI5020_OFFICIAL.passes(), 3);
        assert_eq!(standards::NSA1302.passes(), 3);
        assert_eq!(standards::VSITR_OFFICIAL.passes(), 7);
        assert_eq!(standards::HMG_BASELINE.passes(), 1);
    }

    #[test]
    fn official_variants_sequences() {
        let mut buffer = [0u8; 16];

        standards::ARM38019.fill_buffer(0, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().any(|&b| b != 0x00 && b != 0xFF));

        standards::ARM38019.fill_buffer(1, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().all(|&b| b == 0x00));

        standards::ARM38019.fill_buffer(2, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().all(|&b| b == 0xFF));

        standards::AFSSI5020_OFFICIAL.fill_buffer(0, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().all(|&b| b == 0x00));

        standards::AFSSI5020_OFFICIAL.fill_buffer(1, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().all(|&b| b == 0xFF));

        standards::AFSSI5020_OFFICIAL.fill_buffer(2, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().any(|&b| b != 0x00 && b != 0xFF));

        for pass in 0..standards::NSA1302.passes() - 1 {
            standards::NSA1302.fill_buffer(pass, &mut buffer, &mut seeded_rng());
            assert!(buffer.iter().any(|&b| b != 0x00));
        }

        standards::NSA1302.fill_buffer(2, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().all(|&b| b == 0x00));

        for pass in 0..standards::VSITR_OFFICIAL.passes() {
            standards::VSITR_OFFICIAL.fill_buffer(pass, &mut buffer, &mut seeded_rng());
            match pass {
                0 | 2 | 4 => assert!(buffer.iter().all(|&b| b == 0x00)),
                1 | 3 | 5 => assert!(buffer.iter().all(|&b| b == 0xFF)),
                _ => assert!(buffer.iter().all(|&b| b == 0x55)),
            }
        }

        standards::HMG_BASELINE.fill_buffer(0, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().all(|&b| b == 0x00));
    }

    #[test]
    fn bmb21_sequences() {
        let mut buffer = [0u8; 16];
        for pass in 0..standards::BMB21.passes() {
            standards::BMB21.fill_buffer(pass, &mut buffer, &mut seeded_rng());
            match pass {
                0 | 5 => assert!(buffer.iter().all(|&b| b == 0xFF)),
                1 => assert!(buffer.iter().all(|&b| b == 0x00)),
                _ => assert!(buffer.iter().any(|&b| b != 0x00 && b != 0xFF)),
            }
        }
    }

    #[test]
    fn bsi9_sequences() {
        let expected = [0xFF, 0xFE, 0xFD, 0xFB, 0xF7, 0xEF, 0xDF, 0xBF, 0x7F];
        let mut buffer = [0u8; 16];
        for (pass, &byte) in expected.iter().enumerate() {
            standards::BSI9.fill_buffer(pass, &mut buffer, &mut seeded_rng());
            assert!(buffer.iter().all(|&b| b == byte));
        }
    }

    #[test]
    fn doe_sequences() {
        let mut buffer = [0u8; 16];
        standards::DOE.fill_buffer(0, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().all(|&b| b == 0x00));

        standards::DOE.fill_buffer(1, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().all(|&b| b == 0xFF));

        standards::DOE.fill_buffer(2, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().any(|&b| b != 0xFF));
    }

    #[test]
    fn rcmp7_sequences() {
        let mut buffer = [0u8; 16];
        for pass in 0..standards::RCMP7.passes() {
            standards::RCMP7.fill_buffer(pass, &mut buffer, &mut seeded_rng());
            match pass {
                0 | 2 | 4 => assert!(buffer.iter().all(|&b| b == 0x00)),
                1 | 3 | 5 => assert!(buffer.iter().all(|&b| b == 0xFF)),
                _ => assert!(buffer.iter().any(|&b| b != 0x00 && b != 0xFF)),
            }
        }
    }

    #[test]
    fn nist_purge_sequences() {
        let m = complement::NistPurge::new();
        let mut buffer = [0u8; 64];
        m.fill_buffer(0, &mut buffer, &mut seeded_rng());
        let first = buffer;

        m.fill_buffer(1, &mut buffer, &mut seeded_rng());
        for (i, &b) in first.iter().enumerate() {
            assert_eq!(buffer[i], !b);
        }

        m.fill_buffer(2, &mut buffer, &mut seeded_rng());
        assert_eq!(buffer, first);
    }

    #[test]
    fn wipe_static_patterns() {
        let m = wipe::WipeStatic;
        assert_eq!(m.passes(), 27);

        let mut buffer = [0u8; 8];
        m.fill_buffer(0, &mut buffer, &mut seeded_rng());
        assert_eq!(buffer, [0x55; 8]);

        m.fill_buffer(2, &mut buffer, &mut seeded_rng());
        assert_eq!(buffer, [0x92, 0x49, 0x24, 0x92, 0x49, 0x24, 0x92, 0x49]);

        m.fill_buffer(26, &mut buffer, &mut seeded_rng());
        assert_eq!(buffer, [0xDB, 0x6D, 0xB6, 0xDB, 0x6D, 0xB6, 0xDB, 0x6D]);
    }

    #[test]
    fn many_methods_available() {
        assert!(get_all_methods().len() >= 28);
    }

    #[test]
    fn gutmann_pattern_passes() {
        let mut buffer = [0u8; 8];
        Gutmann.fill_buffer(4, &mut buffer, &mut seeded_rng());
        assert_eq!(buffer, [0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55]);

        Gutmann.fill_buffer(34, &mut buffer, &mut seeded_rng());
        assert_eq!(buffer, [0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33]);
    }

    #[test]
    fn gutmann_first_passes_are_random() {
        let mut buffer = [0u8; 64];
        Gutmann.fill_buffer(0, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().any(|&b| b != 0));
    }

    #[test]
    fn get_method_parsing() {
        assert!(get_method("dod", None).is_some());
        assert!(get_method("DoD", None).is_some());
        assert!(get_method("zeros", None).is_some());
        assert!(get_method("random", None).is_some());

        let m = get_method("random", Some(7)).unwrap();
        assert_eq!(m.passes(), 7);

        let m = get_method("random:5", None).unwrap();
        assert_eq!(m.passes(), 5);

        assert_eq!(get_method("ones", None).unwrap().passes(), 1);
        assert_eq!(get_method("usarmy", None).unwrap().passes(), 2);
        assert_eq!(get_method("navy", None).unwrap().passes(), 3);
        assert_eq!(get_method("usaf", None).unwrap().passes(), 3);
        assert_eq!(get_method("dod7", None).unwrap().passes(), 7);
        assert_eq!(get_method("hmg", None).unwrap().passes(), 3);
        assert_eq!(get_method("hmg5", None).unwrap().passes(), 5);
        assert_eq!(get_method("rcmp", None).unwrap().passes(), 4);
        assert_eq!(get_method("vsitr", None).unwrap().passes(), 7);
        assert_eq!(get_method("gost", None).unwrap().passes(), 2);
        assert_eq!(get_method("nsa", None).unwrap().passes(), 2);
        assert_eq!(get_method("schneier", None).unwrap().passes(), 7);
        assert_eq!(get_method("userm", None).unwrap().passes(), 4);
        assert_eq!(get_method("userm2", None).unwrap().passes(), 7);
        assert_eq!(get_method("randomc", None).unwrap().passes(), 2);
        assert_eq!(get_method("randomcc", None).unwrap().passes(), 3);
        assert_eq!(get_method("navy_mfm", None).unwrap().passes(), 3);
        assert_eq!(get_method("bmb21", None).unwrap().passes(), 6);
        assert_eq!(get_method("bsi", None).unwrap().passes(), 9);
        assert_eq!(get_method("doe", None).unwrap().passes(), 3);
        assert_eq!(get_method("rcmp7", None).unwrap().passes(), 7);
        assert_eq!(get_method("pfitzner", None).unwrap().passes(), 33);
        assert_eq!(get_method("nist", None).unwrap().passes(), 3);
        assert_eq!(get_method("wipe27", None).unwrap().passes(), 27);
        assert_eq!(get_method("arm38019", None).unwrap().passes(), 3);
        assert_eq!(get_method("usarmy_official", None).unwrap().passes(), 3);
        assert_eq!(get_method("afssi5020_official", None).unwrap().passes(), 3);
        assert_eq!(get_method("nsa1302", None).unwrap().passes(), 3);
        assert_eq!(get_method("vsitr_official", None).unwrap().passes(), 7);
        assert_eq!(get_method("hmg_baseline", None).unwrap().passes(), 1);
        assert!(get_method("isc5_base", None).is_some());
        assert!(get_method("usaf_official", None).is_some());
        assert!(get_method("nsa_official", None).is_some());
        assert!(get_method("vsitr_bsi", None).is_some());
        assert!(get_method("army3", None).is_some());
        assert!(get_method("dod522022m7", None).is_some());
        assert!(get_method("afssi5020", None).is_some());
        assert!(get_method("opsii", None).is_some());
        assert!(get_method("bundeswehr", None).is_some());
        assert!(get_method("nist80088", None).is_some());

        assert_eq!(get_method("fill:aa", None).unwrap().passes(), 1);
        assert_eq!(get_method("fill:0xAA", None).unwrap().passes(), 1);
        assert_eq!(get_method("pattern:ff", None).unwrap().passes(), 1);
        assert!(get_method("fill:zz", None).is_none());
        assert!(get_method("fill:", None).is_none());

        assert!(get_method("random:0", None).is_none());
        assert!(get_method("random:abc", None).is_none());
        assert!(get_method("bogus", None).is_none());
    }

    #[test]
    fn display_uses_name() {
        let m = get_method("dod", None).unwrap();
        assert_eq!(m.to_string(), m.name());
    }
}

