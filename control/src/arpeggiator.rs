use heapless::Vec;

use crate::chords::Chord;
use crate::scales::scale::Scale;
use crate::scales::scale_note::ScaleNote;

pub struct Arpeggiator {
    scale: Scale,
    chord: Chord,
    root: ScaleNote,
    mode: Mode,
    state: State,
}

pub enum Mode {
    Root,
    Up,
    UpDownNoRepeat,
    UpDownRepeat,
    Random,
    Moving,
}

pub enum State {
    Up(usize),
    Down(usize),
    Random,
    Moving(usize, Progression),
}

pub type Progression = Vec<i16, 7>;

impl Arpeggiator {
    fn new(scale: Scale, root: ScaleNote, chord: Chord) -> Self {
        Self {
            scale,
            root,
            chord,
            mode: Mode::Up,
            state: State::Up(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::scales::quarter_tones::QuarterTone;
    use crate::scales::tonic::Tonic;

    use super::*;

    #[test]
    fn initialize() {
        let scale = Scale::new(Tonic::C, &[24]).unwrap();
        let root = ScaleNote::new(QuarterTone::C1, 0);
        let chord = Chord::from_slice(&[0, 1]).unwrap();
        let _arp = Arpeggiator::new(scale, root, chord);
    }
}
