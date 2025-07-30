use rand::RngCore;

use super::WipeMethod;

/// A wipe method defined as an ordered list of passes.
/// `Some(byte)` fills the buffer with a fixed byte, `None` fills it with random data.
pub struct PatternMethod {
    name: &'static str,
    patterns: &'static [Option<u8>],
}

impl PatternMethod {
    pub const fn new(name: &'static str, patterns: &'static [Option<u8>]) -> Self {
        Self { name, patterns }
    }
}

impl WipeMethod for PatternMethod {
    fn name(&self) -> &str {
        self.name
    }

    fn passes(&self) -> usize {
        self.patterns.len()
    }

    fn fill_buffer(&self, pass: usize, buffer: &mut [u8], rng: &mut dyn RngCore) {
        match self.patterns.get(pass).copied().flatten() {
            Some(byte) => buffer.fill(byte),
            None => rng.fill_bytes(buffer),
        }
    }
}

pub const ONES: PatternMethod = PatternMethod::new("Ones fill (1 pass)", &[Some(0xFF)]);

pub const USAF: PatternMethod = PatternMethod::new(
    "US Air Force AFSSI-5020 (3 passes)",
    &[Some(0x00), Some(0xFF), Some(0x00)],
);

pub const HMG: PatternMethod =
    PatternMethod::new("UK HMG IS5 (3 passes)", &[Some(0x00), Some(0xFF), None]);

pub const RCMP: PatternMethod = PatternMethod::new(
    "Canadian RCMP TSSIT OPS-II (4 passes)",
    &[Some(0xFF), Some(0x00), Some(0xFF), Some(0x00)],
);

pub const DOD7: PatternMethod = PatternMethod::new(
    "DoD 5220.22-M ECE (7 passes)",
    &[
        Some(0x00),
        Some(0xFF),
        None,
        Some(0x00),
        Some(0xFF),
        None,
        Some(0x00),
    ],
);

pub const VSITR: PatternMethod = PatternMethod::new(
    "German VSITR (7 passes)",
    &[Some(0xFF), Some(0x00), Some(0xFF), None, None, None, None],
);

pub const SCHNEIER: PatternMethod = PatternMethod::new(
    "Schneier (7 passes)",
    &[Some(0xFF), Some(0x00), None, None, None, None, Some(0xFF)],
);

pub const GOST: PatternMethod =
    PatternMethod::new("GOST R 50739-95 (2 passes)", &[Some(0x00), Some(0x00)]);

pub const USERM: PatternMethod = PatternMethod::new(
    "Project-specific sequence userm (4 passes)",
    &[Some(0xFF), Some(0x00), Some(0xFF), None],
);

pub const USERM2: PatternMethod = PatternMethod::new(
    "Project-specific sequence userm2 (7 passes)",
    &[
        Some(0xFF),
        Some(0x00),
        None,
        Some(0x00),
        Some(0xFF),
        None,
        None,
    ],
);

pub const HMG5: PatternMethod = PatternMethod::new(
    "UK HMG IS5 Enhanced (5 passes)",
    &[Some(0x00), Some(0xFF), None, Some(0x00), Some(0xFF)],
);

pub const NSA1301: PatternMethod =
    PatternMethod::new("NSA 130-1 (2 passes)", &[Some(0xFF), Some(0xFF)]);

pub const BMB21: PatternMethod = PatternMethod::new(
    "German Bundeswehr BMB21-2019 (6 passes)",
    &[Some(0xFF), Some(0x00), None, None, None, Some(0xFF)],
);

pub const BSI9: PatternMethod = PatternMethod::new(
    "German BSI (9 passes)",
    &[
        Some(0xFF),
        Some(0xFE),
        Some(0xFD),
        Some(0xFB),
        Some(0xF7),
        Some(0xEF),
        Some(0xDF),
        Some(0xBF),
        Some(0x7F),
    ],
);

pub const DOE: PatternMethod = PatternMethod::new(
    "US DOE M 205.1-2 (3 passes)",
    &[Some(0x00), Some(0xFF), None],
);

pub const RCMP7: PatternMethod = PatternMethod::new(
    "Canadian RCMP B2-002 (7 passes)",
    &[
        Some(0x00),
        Some(0xFF),
        Some(0x00),
        Some(0xFF),
        Some(0x00),
        Some(0xFF),
        None,
    ],
);

pub const ARM38019: PatternMethod = PatternMethod::new(
    "US Army AR 380-19 (3 passes: random, specified char, complement)",
    &[None, Some(0x00), Some(0xFF)],
);

pub const AFSSI5020_OFFICIAL: PatternMethod = PatternMethod::new(
    "US Air Force AFSSI-5020 (3 passes: 0x00, 0xFF, random)",
    &[Some(0x00), Some(0xFF), None],
);

pub const NSA1302: PatternMethod = PatternMethod::new(
    "NSA/CSS 130-2 (3 passes: random, random, known pattern)",
    &[None, None, Some(0x00)],
);

pub const VSITR_OFFICIAL: PatternMethod = PatternMethod::new(
    "German VSITR/BSI Hinweisblatt Nr. 11 (7 passes)",
    &[
        Some(0x00),
        Some(0xFF),
        Some(0x00),
        Some(0xFF),
        Some(0x00),
        Some(0xFF),
        Some(0x55),
    ],
);

pub const HMG_BASELINE: PatternMethod =
    PatternMethod::new("UK HMG IS5 Baseline (1 pass)", &[Some(0x00)]);

/// A single-pass method that fills with a user-provided byte.
pub struct DynamicFill {
    name: String,
    byte: u8,
}

impl DynamicFill {
    pub fn new(byte: u8) -> Self {
        Self {
            name: format!("Fill with byte 0x{:02X} (1 pass)", byte),
            byte,
        }
    }
}

impl WipeMethod for DynamicFill {
    fn name(&self) -> &str {
        &self.name
    }

    fn passes(&self) -> usize {
        1
    }

    fn fill_buffer(&self, _pass: usize, buffer: &mut [u8], _rng: &mut dyn RngCore) {
        buffer.fill(self.byte);
    }
}
