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

    pub fn offset_tone(&mut self, offset: i8) {
        if let Some(quarter_note) = QuarterTone::try_from_u8(
            (self.tone as i16 + offset as i16).clamp(0, QuarterTone::HIGHEST_NOTE as i16) as u8,
        ) {
            self.tone = quarter_note;
        }
    }

    pub fn index(&self) -> u8 {
        self.index
    }
}
