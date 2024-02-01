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
    pub fn new(scale: Scale, root: ScaleNote, chord: Chord) -> Self {
        Self {
            scale,
            root,
            chord,
            mode: Mode::Up,
            state: State::Up(0),
        }
    }

    pub fn pop(&mut self) -> Option<ScaleNote> {
        // XXX: Empty chords don't make sense. This check simplifies the rest
        // of the method.
        assert!(!self.chord.is_empty());

        let chord_degree = match self.state {
            State::Up(index) => {
                let last_degree_index = self.chord.len() - 1;
                if last_degree_index == 0 {
                    self.chord[0]
                } else if index >= last_degree_index {
                    let (new_state, new_index) = match self.mode {
                        Mode::Up => (State::Up(0), last_degree_index),
                        _ => todo!(),
                    };
                    self.state = new_state;
                    self.chord[new_index]
                } else {
                    self.state = State::Up(index + 1);
                    self.chord[index]
                }
            }
            _ => todo!(),
        };

        self.scale
            .get_note_in_interval_ascending(self.root, chord_degree)
    }
}

#[cfg(test)]
mod tests {
    use crate::scales::quarter_tones::QuarterTone;
    use crate::scales::scale::{S, T};
    use crate::scales::tonic::Tonic;

    use super::*;

    #[test]
    fn initialize() {
        let scale = Scale::new(Tonic::C, &[24]).unwrap();
        let root = ScaleNote::new(QuarterTone::C1, 0);
        let chord = Chord::from_slice(&[0, 1]).unwrap();
        let _arp = Arpeggiator::new(scale, root, chord);
    }

    #[test]
    fn up_arp() {
        let scale = Scale::new(Tonic::C, &[T, T, S, T, T, T, S]).unwrap();
        let root = ScaleNote::new(QuarterTone::D1, 1);
        let chord = Chord::from_slice(&[0, 1, 2]).unwrap();

        let mut arp = Arpeggiator::new(scale, root, chord);
        assert_eq!(arp.pop(), Some(ScaleNote::new(QuarterTone::D1, 1)));
        assert_eq!(arp.pop(), Some(ScaleNote::new(QuarterTone::E1, 2)));
        assert_eq!(arp.pop(), Some(ScaleNote::new(QuarterTone::F1, 3)));
        assert_eq!(arp.pop(), Some(ScaleNote::new(QuarterTone::D1, 1)));
    }
}
