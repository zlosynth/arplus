use super::quarter_tones::QuarterTone;

#[derive(Debug, PartialEq, defmt::Format)]
pub struct ScaleNote {
    pub tone: QuarterTone,
    pub index: u8,
}

impl ScaleNote {
    pub fn new(tone: QuarterTone, index: u8) -> Self {
        Self { tone, index }
    }
}
