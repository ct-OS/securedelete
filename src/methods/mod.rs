use std::fmt;

use rand::RngCore;

pub trait WipeMethod: Send + Sync {
    fn name(&self) -> &str;
    fn passes(&self) -> usize;
    /// Fill `buffer` with the data of the given pass.
    ///
    /// `seed` is the wipe's random seed; `rng` is a random source that the
    /// wiper seeds deterministically per pass (see [`pass_seed`]). Methods
    /// that need to reproduce an earlier pass's data (e.g. a complement pass)
    /// MUST derive it from `seed` via [`pass_seed`] instead of storing state,
    /// so the wiper can replay any pass (e.g. to verify the final pass).
    fn fill_buffer(&self, pass: usize, seed: u64, buffer: &mut [u8], rng: &mut dyn RngCore);
}

/// Deterministic per-pass seed derivation, used by the wiper to seed the
/// random source of each pass and by methods that replay a previous pass's
/// random stream (complement, NIST purge, ...).
pub fn pass_seed(seed: u64, pass: usize) -> u64 {
    seed ^ (pass as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
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

/// Static registration of a wipe method: canonical name, aliases and builder.
pub struct MethodEntry {
    pub canonical: &'static str,
    pub aliases: &'static [&'static str],
    pub make: fn() -> Box<dyn WipeMethod>,
}

fn make_zero() -> Box<dyn WipeMethod> {
    Box::new(zero::ZeroPass)
}
fn make_ones() -> Box<dyn WipeMethod> {
    Box::new(standards::ONES)
}
fn make_usarmy() -> Box<dyn WipeMethod> {
    Box::new(random::USARMY)
}
fn make_navy() -> Box<dyn WipeMethod> {
    Box::new(random::NAVY)
}
fn make_usaf() -> Box<dyn WipeMethod> {
    Box::new(standards::USAF)
}
fn make_dod() -> Box<dyn WipeMethod> {
    Box::new(dod::DoD522022M)
}
fn make_dod7() -> Box<dyn WipeMethod> {
    Box::new(standards::DOD7)
}
fn make_hmg() -> Box<dyn WipeMethod> {
    Box::new(standards::HMG)
}
fn make_hmg5() -> Box<dyn WipeMethod> {
    Box::new(standards::HMG5)
}
fn make_rcmp() -> Box<dyn WipeMethod> {
    Box::new(standards::RCMP)
}
fn make_vsitr() -> Box<dyn WipeMethod> {
    Box::new(standards::VSITR)
}
fn make_gost() -> Box<dyn WipeMethod> {
    Box::new(standards::GOST)
}
fn make_nsa() -> Box<dyn WipeMethod> {
    Box::new(standards::NSA1301)
}
fn make_schneier() -> Box<dyn WipeMethod> {
    Box::new(standards::SCHNEIER)
}
fn make_userm() -> Box<dyn WipeMethod> {
    Box::new(standards::USERM)
}
fn make_userm2() -> Box<dyn WipeMethod> {
    Box::new(standards::USERM2)
}
fn make_randomc() -> Box<dyn WipeMethod> {
    Box::new(complement::RandomComplement::inverse())
}
fn make_randomcc() -> Box<dyn WipeMethod> {
    Box::new(complement::RandomComplement::inverse_cycle())
}
fn make_navy_mfm() -> Box<dyn WipeMethod> {
    Box::new(complement::RandomComplement::navy_mfm())
}
fn make_gutmann() -> Box<dyn WipeMethod> {
    Box::new(gutmann::Gutmann)
}
fn make_bmb21() -> Box<dyn WipeMethod> {
    Box::new(standards::BMB21)
}
fn make_bsi() -> Box<dyn WipeMethod> {
    Box::new(standards::BSI9)
}
fn make_doe() -> Box<dyn WipeMethod> {
    Box::new(standards::DOE)
}
fn make_rcmp7() -> Box<dyn WipeMethod> {
    Box::new(standards::RCMP7)
}
fn make_pfitzner() -> Box<dyn WipeMethod> {
    Box::new(random::PFITZNER)
}
fn make_nist() -> Box<dyn WipeMethod> {
    Box::new(complement::NistPurge)
}
fn make_wipe27() -> Box<dyn WipeMethod> {
    Box::new(wipe::WipeStatic)
}
fn make_arm38019() -> Box<dyn WipeMethod> {
    Box::new(standards::ARM38019)
}
fn make_afssi5020_official() -> Box<dyn WipeMethod> {
    Box::new(standards::AFSSI5020_OFFICIAL)
}
fn make_nsa1302() -> Box<dyn WipeMethod> {
    Box::new(standards::NSA1302)
}
fn make_vsitr_official() -> Box<dyn WipeMethod> {
    Box::new(standards::VSITR_OFFICIAL)
}
fn make_hmg_baseline() -> Box<dyn WipeMethod> {
    Box::new(standards::HMG_BASELINE)
}

static STATIC_METHODS: &[MethodEntry] = &[
    MethodEntry {
        canonical: "zeros",
        aliases: &["zero"],
        make: make_zero,
    },
    MethodEntry {
        canonical: "ones",
        aliases: &[],
        make: make_ones,
    },
    MethodEntry {
        canonical: "usarmy",
        aliases: &["army38019"],
        make: make_usarmy,
    },
    MethodEntry {
        canonical: "navy",
        aliases: &["navso", "navso5239"],
        make: make_navy,
    },
    MethodEntry {
        canonical: "usaf",
        aliases: &["afssi5020"],
        make: make_usaf,
    },
    MethodEntry {
        canonical: "dod",
        aliases: &["dod522022m"],
        make: make_dod,
    },
    MethodEntry {
        canonical: "dod7",
        aliases: &["dod522022m7", "dodece"],
        make: make_dod7,
    },
    MethodEntry {
        canonical: "hmg",
        aliases: &["isc5"],
        make: make_hmg,
    },
    MethodEntry {
        canonical: "hmg5",
        aliases: &["isc5e", "hmg_enhanced"],
        make: make_hmg5,
    },
    MethodEntry {
        canonical: "rcmp",
        aliases: &["tssit", "opsii"],
        make: make_rcmp,
    },
    MethodEntry {
        canonical: "vsitr",
        aliases: &[],
        make: make_vsitr,
    },
    MethodEntry {
        canonical: "gost",
        aliases: &["gost50739"],
        make: make_gost,
    },
    MethodEntry {
        canonical: "nsa",
        aliases: &["nsa1301"],
        make: make_nsa,
    },
    MethodEntry {
        canonical: "schneier",
        aliases: &[],
        make: make_schneier,
    },
    MethodEntry {
        canonical: "userm",
        aliases: &[],
        make: make_userm,
    },
    MethodEntry {
        canonical: "userm2",
        aliases: &[],
        make: make_userm2,
    },
    MethodEntry {
        canonical: "randomc",
        aliases: &["random_complement"],
        make: make_randomc,
    },
    MethodEntry {
        canonical: "randomcc",
        aliases: &["complement_cycle"],
        make: make_randomcc,
    },
    MethodEntry {
        canonical: "navy_mfm",
        aliases: &["navso_mfm", "navso5239mfm"],
        make: make_navy_mfm,
    },
    MethodEntry {
        canonical: "gutmann",
        aliases: &[],
        make: make_gutmann,
    },
    MethodEntry {
        canonical: "bmb21",
        aliases: &["bundeswehr"],
        make: make_bmb21,
    },
    MethodEntry {
        canonical: "bsi",
        aliases: &["bsi9"],
        make: make_bsi,
    },
    MethodEntry {
        canonical: "doe",
        aliases: &["doe20512"],
        make: make_doe,
    },
    MethodEntry {
        canonical: "rcmp7",
        aliases: &["b2002"],
        make: make_rcmp7,
    },
    MethodEntry {
        canonical: "pfitzner",
        aliases: &["pfitzner33"],
        make: make_pfitzner,
    },
    MethodEntry {
        canonical: "nist",
        aliases: &["nist80088"],
        make: make_nist,
    },
    MethodEntry {
        canonical: "wipe27",
        aliases: &["wipestatic"],
        make: make_wipe27,
    },
    MethodEntry {
        canonical: "arm38019",
        aliases: &["usarmy_official", "army3"],
        make: make_arm38019,
    },
    MethodEntry {
        canonical: "afssi5020_official",
        aliases: &["usaf_official"],
        make: make_afssi5020_official,
    },
    MethodEntry {
        canonical: "nsa1302",
        aliases: &["nsa_official"],
        make: make_nsa1302,
    },
    MethodEntry {
        canonical: "vsitr_official",
        aliases: &["vsitr_bsi"],
        make: make_vsitr_official,
    },
    MethodEntry {
        canonical: "hmg_baseline",
        aliases: &["isc5_base", "hmg1"],
        make: make_hmg_baseline,
    },
];

/// All statically registered methods (for --list-methods).
pub fn method_entries() -> &'static [MethodEntry] {
    STATIC_METHODS
}

#[cfg(test)]
pub fn get_all_methods() -> Vec<Box<dyn WipeMethod>> {
    let mut methods: Vec<Box<dyn WipeMethod>> = STATIC_METHODS.iter().map(|e| (e.make)()).collect();
    methods.insert(2, Box::new(random::RandomPass::new(1)));
    methods
}

pub fn get_method(name: &str, passes_override: Option<usize>) -> Option<Box<dyn WipeMethod>> {
    let passes_override = passes_override.filter(|&n| n > 0);
    let lower = name.to_lowercase();

    if lower == "random" {
        return Some(Box::new(random::RandomPass::new(
            passes_override.unwrap_or(1),
        )));
    }
    if let Some(n) = lower.strip_prefix("random:") {
        let passes: usize = n.parse().ok()?;
        if passes == 0 {
            return None;
        }
        return Some(Box::new(random::RandomPass::new(passes)));
    }
    if let Some(hex) = lower
        .strip_prefix("fill:")
        .or_else(|| lower.strip_prefix("pattern:"))
    {
        let hex = hex.strip_prefix("0x").unwrap_or(hex);
        let byte = u8::from_str_radix(hex, 16).ok()?;
        return Some(Box::new(standards::DynamicFill::new(byte)));
    }

    for entry in STATIC_METHODS {
        if entry.canonical == lower || entry.aliases.iter().any(|a| *a == lower) {
            return Some((entry.make)());
        }
    }
    None
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

    const TEST_SEED: u64 = 42;

    #[test]
    fn zero_fills_all_zeroes() {
        let mut buffer = [0xFFu8; 100];
        ZeroPass.fill_buffer(0, TEST_SEED, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().all(|&b| b == 0));
    }

    #[test]
    fn random_uses_rng() {
        let mut buffer = [0u8; 64];
        random::RandomPass::new(1).fill_buffer(0, TEST_SEED, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().any(|&b| b != 0));
    }

    #[test]
    fn random_is_deterministic_for_same_seed() {
        let mut a = [0u8; 64];
        let mut b = [0u8; 64];
        random::RandomPass::new(1).fill_buffer(0, TEST_SEED, &mut a, &mut seeded_rng());
        random::RandomPass::new(1).fill_buffer(0, TEST_SEED, &mut b, &mut seeded_rng());
        assert_eq!(a, b);
    }

    #[test]
    fn dod_sequences() {
        let mut buffer = [0u8; 16];
        DoD522022M.fill_buffer(0, TEST_SEED, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().all(|&b| b == 0x00));

        DoD522022M.fill_buffer(1, TEST_SEED, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().all(|&b| b == 0xFF));

        DoD522022M.fill_buffer(2, TEST_SEED, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().any(|&b| b != 0xFF));
    }

    #[test]
    fn ones_fills_all_ones() {
        let mut buffer = [0u8; 16];
        standards::ONES.fill_buffer(0, TEST_SEED, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().all(|&b| b == 0xFF));
    }

    #[test]
    fn usaf_sequences() {
        let mut buffer = [0u8; 16];
        standards::USAF.fill_buffer(0, TEST_SEED, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().all(|&b| b == 0x00));

        standards::USAF.fill_buffer(1, TEST_SEED, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().all(|&b| b == 0xFF));

        standards::USAF.fill_buffer(2, TEST_SEED, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().all(|&b| b == 0x00));
    }

    #[test]
    fn hmg_random_last_pass() {
        let mut buffer = [0u8; 64];
        standards::HMG.fill_buffer(2, TEST_SEED, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().any(|&b| b != 0));
    }

    #[test]
    fn dod7_sequences() {
        let mut buffer = [0u8; 16];
        for pass in 0..standards::DOD7.passes() {
            standards::DOD7.fill_buffer(pass, TEST_SEED, &mut buffer, &mut seeded_rng());
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
        standards::SCHNEIER.fill_buffer(0, TEST_SEED, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().all(|&b| b == 0xFF));

        standards::SCHNEIER.fill_buffer(6, TEST_SEED, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().all(|&b| b == 0xFF));

        standards::SCHNEIER.fill_buffer(2, TEST_SEED, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().any(|&b| b != 0xFF));
    }

    #[test]
    fn gost_sequences() {
        let mut buffer = [0u8; 16];
        for pass in 0..standards::GOST.passes() {
            standards::GOST.fill_buffer(pass, TEST_SEED, &mut buffer, &mut seeded_rng());
            assert!(buffer.iter().all(|&b| b == 0x00));
        }
    }

    #[test]
    fn nsa1301_sequences() {
        let mut buffer = [0u8; 16];
        for pass in 0..standards::NSA1301.passes() {
            standards::NSA1301.fill_buffer(pass, TEST_SEED, &mut buffer, &mut seeded_rng());
            assert!(buffer.iter().all(|&b| b == 0xFF));
        }
    }

    #[test]
    fn userm_sequences() {
        let mut buffer = [0u8; 16];
        for pass in 0..standards::USERM.passes() {
            standards::USERM.fill_buffer(pass, TEST_SEED, &mut buffer, &mut seeded_rng());
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
            standards::HMG5.fill_buffer(pass, TEST_SEED, &mut buffer, &mut seeded_rng());
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
        m.fill_buffer(0, TEST_SEED, &mut buffer, &mut seeded_rng());
        let original = buffer;

        m.fill_buffer(1, TEST_SEED, &mut buffer, &mut seeded_rng());
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
        m.fill_buffer(0, TEST_SEED, &mut buffer, &mut seeded_rng());
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
        assert_eq!(complement::NistPurge.passes(), 3);
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

        standards::ARM38019.fill_buffer(0, TEST_SEED, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().any(|&b| b != 0x00 && b != 0xFF));

        standards::ARM38019.fill_buffer(1, TEST_SEED, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().all(|&b| b == 0x00));

        standards::ARM38019.fill_buffer(2, TEST_SEED, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().all(|&b| b == 0xFF));

        standards::AFSSI5020_OFFICIAL.fill_buffer(0, TEST_SEED, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().all(|&b| b == 0x00));

        standards::AFSSI5020_OFFICIAL.fill_buffer(1, TEST_SEED, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().all(|&b| b == 0xFF));

        standards::AFSSI5020_OFFICIAL.fill_buffer(2, TEST_SEED, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().any(|&b| b != 0x00 && b != 0xFF));

        for pass in 0..standards::NSA1302.passes() - 1 {
            standards::NSA1302.fill_buffer(pass, TEST_SEED, &mut buffer, &mut seeded_rng());
            assert!(buffer.iter().any(|&b| b != 0x00));
        }

        standards::NSA1302.fill_buffer(2, TEST_SEED, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().all(|&b| b == 0x00));

        for pass in 0..standards::VSITR_OFFICIAL.passes() {
            standards::VSITR_OFFICIAL.fill_buffer(pass, TEST_SEED, &mut buffer, &mut seeded_rng());
            match pass {
                0 | 2 | 4 => assert!(buffer.iter().all(|&b| b == 0x00)),
                1 | 3 | 5 => assert!(buffer.iter().all(|&b| b == 0xFF)),
                _ => assert!(buffer.iter().all(|&b| b == 0x55)),
            }
        }

        standards::HMG_BASELINE.fill_buffer(0, TEST_SEED, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().all(|&b| b == 0x00));
    }

    #[test]
    fn bmb21_sequences() {
        let mut buffer = [0u8; 16];
        for pass in 0..standards::BMB21.passes() {
            standards::BMB21.fill_buffer(pass, TEST_SEED, &mut buffer, &mut seeded_rng());
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
            standards::BSI9.fill_buffer(pass, TEST_SEED, &mut buffer, &mut seeded_rng());
            assert!(buffer.iter().all(|&b| b == byte));
        }
    }

    #[test]
    fn doe_sequences() {
        let mut buffer = [0u8; 16];
        standards::DOE.fill_buffer(0, TEST_SEED, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().all(|&b| b == 0x00));

        standards::DOE.fill_buffer(1, TEST_SEED, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().all(|&b| b == 0xFF));

        standards::DOE.fill_buffer(2, TEST_SEED, &mut buffer, &mut seeded_rng());
        assert!(buffer.iter().any(|&b| b != 0xFF));
    }

    #[test]
    fn rcmp7_sequences() {
        let mut buffer = [0u8; 16];
        for pass in 0..standards::RCMP7.passes() {
            standards::RCMP7.fill_buffer(pass, TEST_SEED, &mut buffer, &mut seeded_rng());
            match pass {
                0 | 2 | 4 => assert!(buffer.iter().all(|&b| b == 0x00)),
                1 | 3 | 5 => assert!(buffer.iter().all(|&b| b == 0xFF)),
                _ => assert!(buffer.iter().any(|&b| b != 0x00 && b != 0xFF)),
            }
        }
    }

    #[test]
    fn nist_purge_sequences() {
        let m = complement::NistPurge;
        let mut buffer = [0u8; 64];
        m.fill_buffer(0, TEST_SEED, &mut buffer, &mut seeded_rng());
        let first = buffer;

        m.fill_buffer(1, TEST_SEED, &mut buffer, &mut seeded_rng());
        for (i, &b) in first.iter().enumerate() {
            assert_eq!(buffer[i], !b);
        }

        m.fill_buffer(2, TEST_SEED, &mut buffer, &mut seeded_rng());
        assert_eq!(buffer, first);
    }

    #[test]
    fn wipe_static_patterns() {
        let m = wipe::WipeStatic;
        assert_eq!(m.passes(), 27);

        let mut buffer = [0u8; 8];
        m.fill_buffer(0, TEST_SEED, &mut buffer, &mut seeded_rng());
        assert_eq!(buffer, [0x55; 8]);

        m.fill_buffer(2, TEST_SEED, &mut buffer, &mut seeded_rng());
        assert_eq!(buffer, [0x92, 0x49, 0x24, 0x92, 0x49, 0x24, 0x92, 0x49]);

        m.fill_buffer(26, TEST_SEED, &mut buffer, &mut seeded_rng());
        assert_eq!(buffer, [0xDB, 0x6D, 0xB6, 0xDB, 0x6D, 0xB6, 0xDB, 0x6D]);
    }

    #[test]
    fn many_methods_available() {
        assert!(get_all_methods().len() >= 28);
    }

    #[test]
    fn gutmann_pattern_passes() {
        let mut buffer = [0u8; 8];
        Gutmann.fill_buffer(4, TEST_SEED, &mut buffer, &mut seeded_rng());
        assert_eq!(buffer, [0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55]);

        Gutmann.fill_buffer(34, TEST_SEED, &mut buffer, &mut seeded_rng());
        assert_eq!(buffer, [0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33]);
    }

    #[test]
    fn gutmann_first_passes_are_random() {
        let mut buffer = [0u8; 64];
        Gutmann.fill_buffer(0, TEST_SEED, &mut buffer, &mut seeded_rng());
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
