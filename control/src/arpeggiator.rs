use heapless::Vec;

use crate::chords::Chord;
use crate::scales::scale::Scale;
use crate::scales::scale_note::ScaleNote;

#[derive(Clone, Debug, defmt::Format)]
pub struct Arpeggiator {
    scale: Scale,
    root: ScaleNote,
    chord: Chord,
    mode: Mode,
    state: State,
}

#[derive(Clone, Debug, defmt::Format)]
pub enum Mode {
    Root,
    Up,
    UpDownNoRepeat,
    UpDownRepeat,
    Random,
    Moving,
}

#[derive(Clone, Debug, defmt::Format)]
pub enum State {
    Up(usize),
    Down(usize),
    Random,
    Moving(usize, Progression),
}

pub type Progression = Vec<i16, 7>;

#[derive(Clone, Debug, defmt::Format)]
pub struct Configuration {
    pub scale: Scale,
    pub root: ScaleNote,
    pub chord: Chord,
}

impl Arpeggiator {
    pub fn new_with_configuration(config: Configuration) -> Self {
        Self {
            scale: config.scale,
            root: config.root,
            chord: config.chord,
            mode: Mode::Up,
            state: State::Up(0),
        }
    }

    pub fn apply_configuration(&mut self, configuration: Configuration) {
        self.scale = configuration.scale;
        self.root = configuration.root;
        self.chord = configuration.chord;
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
                } else if index < last_degree_index {
                    self.state = State::Up(index + 1);
                    self.chord[index]
                } else if index == last_degree_index {
                    let (new_state, new_index) = match self.mode {
                        Mode::Up => (State::Up(0), last_degree_index),
                        _ => todo!(),
                    };
                    self.state = new_state;
                    self.chord[new_index]
                } else {
                    let (new_state, new_index) = match self.mode {
                        Mode::Up => (State::Up(1), 0),
                        _ => todo!(),
                    };
                    self.state = new_state;
                    self.chord[new_index]
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
    use crate::scales::scale::{Step, S, T};
    use crate::scales::tonic::Tonic;

    use super::*;

    const IONIAN: [Step; 7] = [T, T, S, T, T, T, S];

    #[test]
    fn initialize() {
        let scale = Scale::new(Tonic::C, &IONIAN).unwrap();
        let root = ScaleNote::new(QuarterTone::C1, 0);
        let chord = Chord::from_slice(&[0, 1]).unwrap();
        let _arp = Arpeggiator::new_with_configuration(Configuration { scale, root, chord });
    }

    #[test]
    fn up_arp() {
        let scale = Scale::new(Tonic::C, &IONIAN).unwrap();
        let root = ScaleNote::new(QuarterTone::D1, 1);
        let chord = Chord::from_slice(&[0, 1, 2]).unwrap();

        let mut arp = Arpeggiator::new_with_configuration(Configuration { scale, root, chord });
        assert_eq!(arp.pop(), Some(ScaleNote::new(QuarterTone::D1, 1)));
        assert_eq!(arp.pop(), Some(ScaleNote::new(QuarterTone::E1, 2)));
        assert_eq!(arp.pop(), Some(ScaleNote::new(QuarterTone::F1, 3)));
        assert_eq!(arp.pop(), Some(ScaleNote::new(QuarterTone::D1, 1)));
    }

    #[test]
    fn change_chord_but_keep_size() {
        let scale = Scale::new(Tonic::C, &IONIAN).unwrap();
        let root = ScaleNote::new(QuarterTone::D1, 1);

        let mut arp = Arpeggiator::new_with_configuration(Configuration {
            scale: scale.clone(),
            root,
            chord: Chord::from_slice(&[0, 1, 2]).unwrap(),
        });
        assert_eq!(arp.pop(), Some(ScaleNote::new(QuarterTone::D1, 1)));
        assert_eq!(arp.pop(), Some(ScaleNote::new(QuarterTone::E1, 2)));
        arp.apply_configuration(Configuration {
            scale: scale.clone(),
            root,
            chord: Chord::from_slice(&[0, 1, 3]).unwrap(),
        });
        assert_eq!(arp.pop(), Some(ScaleNote::new(QuarterTone::G1, 4)));
        assert_eq!(arp.pop(), Some(ScaleNote::new(QuarterTone::D1, 1)));
    }

    #[test]
    fn change_chord_and_reduce_size() {
        let scale = Scale::new(Tonic::C, &IONIAN).unwrap();
        let root = ScaleNote::new(QuarterTone::D1, 1);

        let mut arp = Arpeggiator::new_with_configuration(Configuration {
            scale: scale.clone(),
            root,
            chord: Chord::from_slice(&[0, 1, 2, 3]).unwrap(),
        });
        assert_eq!(arp.pop(), Some(ScaleNote::new(QuarterTone::D1, 1)));
        assert_eq!(arp.pop(), Some(ScaleNote::new(QuarterTone::E1, 2)));
        assert_eq!(arp.pop(), Some(ScaleNote::new(QuarterTone::F1, 3)));
        arp.apply_configuration(Configuration {
            scale: scale.clone(),
            root,
            chord: Chord::from_slice(&[0, 1]).unwrap(),
        });
        assert_eq!(arp.pop(), Some(ScaleNote::new(QuarterTone::D1, 1)));
        assert_eq!(arp.pop(), Some(ScaleNote::new(QuarterTone::E1, 2)));
    }

    #[test]
    fn change_scale() {
        todo!();
    }

    #[test]
    fn change_root() {
        todo!();
    }
}
