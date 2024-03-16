use super::quarter_tones::QuarterTone;

#[derive(Clone, Copy, Debug, PartialEq, defmt::Format)]
pub struct ScaleNote {
    tone: QuarterTone,
    index: u8,
}

impl ScaleNote {
    pub fn new(tone: QuarterTone, index: u8) -> Self {
        Self { tone, index }
    }

    pub fn tone(&self) -> QuarterTone {
        self.tone
    }

    pub fn index(&self) -> u8 {
        self.index
    }
}
