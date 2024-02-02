use core::str::SplitAsciiWhitespace;

use heapless::Vec;

use crate::chords::Chord;
use crate::scales::scale::Scale;
use crate::scales::scale_note::ScaleNote;

pub trait Random {
    fn pop(&mut self) -> f32;
}

#[derive(Clone, Debug, defmt::Format)]
pub struct Arpeggiator {
    scale: Scale,
    root: ScaleNote,
    chord: Chord,
    mode: Mode,
    state: State,
}

#[derive(Clone, Copy, PartialEq, Debug, defmt::Format)]
pub enum Mode {
    Root,
    Up,
    UpDownNoRepeats,
    UpDownRepeats,
    Random,
    Moving,
}

#[derive(Clone, Debug, defmt::Format)]
pub enum State {
    Root,
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
    pub mode: Mode,
}

impl Arpeggiator {
    pub fn new_with_configuration(config: Configuration) -> Self {
        Self {
            scale: config.scale,
            root: config.root,
            mode: config.mode,
            state: match config.mode {
                Mode::Root => State::Root,
                Mode::Random => State::Random,
                Mode::Moving => State::Moving(0, config.chord.clone()),
                _ => State::Up(0),
            },
            chord: config.chord,
        }
    }

    pub fn apply_configuration(&mut self, configuration: Configuration) {
        if self.mode != configuration.mode {
            self.mode = configuration.mode;
            match self.mode {
                Mode::Root => self.state = State::Root,
                Mode::Up | Mode::UpDownRepeats | Mode::UpDownNoRepeats => {
                    if !matches!(self.state, State::Up(_)) {
                        self.state = State::Up(0);
                    }
                }
                Mode::Random => self.state = State::Random,
                Mode::Moving => self.state = State::Moving(0, self.chord.clone()),
            }
        }

        if self.chord != configuration.chord {
            if let State::Moving(_, schuffled_chord) = &mut self.state {
                *schuffled_chord = configuration.chord.clone();
            }
            self.chord = configuration.chord;
        }

        self.scale = configuration.scale;
        self.root = configuration.root;
    }

    pub fn pop(&mut self, random: &mut impl Random) -> Option<ScaleNote> {
        // XXX: Empty chords don't make sense. This check simplifies the rest
        // of the method.
        assert!(!self.chord.is_empty());

        let chord_degree = match self.state {
            State::Root => self.chord[0],
            State::Up(index) => {
                let last_index = self.chord.len() - 1;
                if last_index == 0 {
                    self.chord[0]
                } else if index < last_index {
                    self.state = State::Up(index + 1);
                    self.chord[index]
                } else if index == last_index {
                    let (new_state, new_index) = match self.mode {
                        Mode::Up => (State::Up(0), last_index),
                        Mode::UpDownRepeats => (State::Down(last_index), last_index),
                        Mode::UpDownNoRepeats => (State::Down(last_index - 1), last_index),
                        _ => unreachable!(),
                    };
                    self.state = new_state;
                    self.chord[new_index]
                } else {
                    let (new_state, new_index) = match self.mode {
                        Mode::Up => (State::Up(1), 0),
                        Mode::UpDownRepeats => (State::Down(last_index), last_index),
                        Mode::UpDownNoRepeats => (State::Down(last_index - 1), last_index),
                        _ => unreachable!(),
                    };
                    self.state = new_state;
                    self.chord[new_index]
                }
            }
            State::Down(index) => {
                let last_index = self.chord.len() - 1;
                if last_index == 0 {
                    self.chord[0]
                } else if index == 0 {
                    let (new_state, new_index) = match self.mode {
                        Mode::Up | Mode::UpDownNoRepeats => (State::Up(index + 1), index),
                        Mode::UpDownRepeats => (State::Up(index), index),
                        _ => unreachable!(),
                    };
                    self.state = new_state;
                    self.chord[new_index]
                } else {
                    let new_index = usize::min(index, last_index);
                    self.state = State::Down(new_index - 1);
                    self.chord[new_index]
                }
            }
            State::Random => {
                let last_index = self.chord.len() - 1;
                let random_phase = random.pop();
                let index = (random_phase * (last_index as f32 + 0.99)) as usize;
                self.chord[index]
            }
            State::Moving(ref mut index, ref mut schuffled_chord) => {
                let last_index = self.chord.len() - 1;
                if last_index == 0 {
                    self.chord[last_index]
                } else if *index >= last_index {
                    let degree = schuffled_chord[last_index];
                    let (random_a, random_b) = two_distinct_random_values(last_index, random);
                    schuffled_chord.swap(random_a, random_b);
                    *index = 0;
                    degree
                } else {
                    let degree = schuffled_chord[*index];
                    *index += 1;
                    degree
                }
            }
        };

        self.scale
            .get_note_in_interval_ascending(self.root, chord_degree)
    }
}

fn two_distinct_random_values(max: usize, random: &mut impl Random) -> (usize, usize) {
    let a = (random.pop() * (max as f32 + 0.99)) as usize;
    let mut b = (random.pop() * (max as f32 + 0.99)) as usize;
    if b == a {
        b += 1;
        if b > max {
            b -= max + 1;
        }
    }
    (a, b)
}

#[cfg(test)]
mod tests {
    use crate::scales::quarter_tones::QuarterTone;
    use crate::scales::scale::{Step, S, T};
    use crate::scales::tonic::Tonic;

    use super::*;

    const IONIAN: [Step; 7] = [T, T, S, T, T, T, S];
    const DORIAN: [Step; 7] = [T, S, T, T, T, S, T];

    struct TestRandom {
        values: [f32; 3],
        index: usize,
    }

    impl TestRandom {
        fn new() -> Self {
            Self {
                values: [0.0; 3],
                index: 0,
            }
        }

        fn new_with_values(values: [f32; 3]) -> Self {
            Self { values, index: 0 }
        }
    }

    impl Random for TestRandom {
        fn pop(&mut self) -> f32 {
            let value = self.values[self.index];
            self.index += 1;
            self.index %= self.values.len();
            value
        }
    }

    #[test]
    fn initialize() {
        let configuration = Configuration {
            scale: Scale::new(Tonic::C, &IONIAN).unwrap(),
            root: ScaleNote::new(QuarterTone::C1, 0),
            chord: Chord::from_slice(&[0, 1]).unwrap(),
            mode: Mode::Up,
        };
        let _arp = Arpeggiator::new_with_configuration(configuration);
    }

    #[test]
    fn root_arp() {
        let mut r = TestRandom::new();
        let configuration = Configuration {
            scale: Scale::new(Tonic::C, &IONIAN).unwrap(),
            root: ScaleNote::new(QuarterTone::D1, 1),
            chord: Chord::from_slice(&[0, 1, 2]).unwrap(),
            mode: Mode::Root,
        };
        let mut arp = Arpeggiator::new_with_configuration(configuration);
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::D1, 1)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::D1, 1)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::D1, 1)));
    }

    #[test]
    fn up_arp() {
        let mut r = TestRandom::new();
        let configuration = Configuration {
            scale: Scale::new(Tonic::C, &IONIAN).unwrap(),
            root: ScaleNote::new(QuarterTone::D1, 1),
            chord: Chord::from_slice(&[0, 1, 2]).unwrap(),
            mode: Mode::Up,
        };
        let mut arp = Arpeggiator::new_with_configuration(configuration);
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::D1, 1)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::E1, 2)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::F1, 3)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::D1, 1)));
    }

    #[test]
    fn up_down_no_repeat_arp() {
        let mut r = TestRandom::new();
        let configuration = Configuration {
            scale: Scale::new(Tonic::C, &IONIAN).unwrap(),
            chord: Chord::from_slice(&[0, 1, 2]).unwrap(),
            mode: Mode::UpDownNoRepeats,
            root: ScaleNote::new(QuarterTone::D1, 1),
        };
        let mut arp = Arpeggiator::new_with_configuration(configuration.clone());
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::D1, 1)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::E1, 2)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::F1, 3)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::E1, 2)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::D1, 1)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::E1, 2)));
    }

    #[test]
    fn up_down_repeat_arp() {
        let mut r = TestRandom::new();
        let configuration = Configuration {
            scale: Scale::new(Tonic::C, &IONIAN).unwrap(),
            chord: Chord::from_slice(&[0, 1, 2]).unwrap(),
            mode: Mode::UpDownRepeats,
            root: ScaleNote::new(QuarterTone::D1, 1),
        };
        let mut arp = Arpeggiator::new_with_configuration(configuration.clone());
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::D1, 1)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::E1, 2)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::F1, 3)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::F1, 3)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::E1, 2)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::D1, 1)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::D1, 1)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::E1, 2)));
    }

    #[test]
    fn random_arp() {
        let mut r = TestRandom::new_with_values([0.0, 1.0, 0.5]);
        let configuration = Configuration {
            scale: Scale::new(Tonic::C, &IONIAN).unwrap(),
            chord: Chord::from_slice(&[0, 1, 2]).unwrap(),
            mode: Mode::Random,
            root: ScaleNote::new(QuarterTone::D1, 1),
        };
        let mut arp = Arpeggiator::new_with_configuration(configuration.clone());
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::D1, 1)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::F1, 3)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::E1, 2)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::D1, 1)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::F1, 3)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::E1, 2)));
    }

    #[test]
    fn moving_arp() {
        let mut r = TestRandom::new_with_values([0.0, 1.0, 0.5]);
        let configuration = Configuration {
            scale: Scale::new(Tonic::C, &IONIAN).unwrap(),
            chord: Chord::from_slice(&[0, 1, 2]).unwrap(),
            mode: Mode::Moving,
            root: ScaleNote::new(QuarterTone::D1, 1),
        };
        let mut arp = Arpeggiator::new_with_configuration(configuration.clone());
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::D1, 1)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::E1, 2)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::F1, 3)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::F1, 3)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::E1, 2)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::D1, 1)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::E1, 2)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::F1, 3)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::D1, 1)));
    }

    #[test]
    fn change_chord_but_keep_size() {
        let mut r = TestRandom::new();
        let configuration = Configuration {
            scale: Scale::new(Tonic::C, &IONIAN).unwrap(),
            root: ScaleNote::new(QuarterTone::D1, 1),
            mode: Mode::Up,
            chord: Chord::from_slice(&[0, 1, 2]).unwrap(),
        };
        let mut arp = Arpeggiator::new_with_configuration(configuration.clone());
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::D1, 1)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::E1, 2)));
        arp.apply_configuration(Configuration {
            chord: Chord::from_slice(&[0, 1, 3]).unwrap(),
            ..configuration.clone()
        });
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::G1, 4)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::D1, 1)));
    }

    #[test]
    fn change_chord_and_reduce_size_with_up() {
        let mut r = TestRandom::new();
        let configuration = Configuration {
            scale: Scale::new(Tonic::C, &IONIAN).unwrap(),
            root: ScaleNote::new(QuarterTone::D1, 1),
            mode: Mode::Up,
            chord: Chord::from_slice(&[0, 1, 2, 3]).unwrap(),
        };
        let mut arp = Arpeggiator::new_with_configuration(configuration.clone());
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::D1, 1)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::E1, 2)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::F1, 3)));
        arp.apply_configuration(Configuration {
            chord: Chord::from_slice(&[0, 1]).unwrap(),
            ..configuration.clone()
        });
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::D1, 1)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::E1, 2)));
    }

    #[test]
    fn change_chord_and_reduce_size_with_up_down_no_repeat_when_going_up() {
        let mut r = TestRandom::new();
        let configuration = Configuration {
            scale: Scale::new(Tonic::C, &IONIAN).unwrap(),
            root: ScaleNote::new(QuarterTone::D1, 1),
            mode: Mode::UpDownNoRepeats,
            chord: Chord::from_slice(&[0, 1, 2, 3]).unwrap(),
        };
        let mut arp = Arpeggiator::new_with_configuration(configuration.clone());
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::D1, 1)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::E1, 2)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::F1, 3)));
        arp.apply_configuration(Configuration {
            chord: Chord::from_slice(&[0, 1]).unwrap(),
            ..configuration.clone()
        });
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::E1, 2)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::D1, 1)));
    }

    #[test]
    fn change_chord_and_reduce_size_with_up_down_no_repeat_when_going_down() {
        let mut r = TestRandom::new();
        let configuration = Configuration {
            scale: Scale::new(Tonic::C, &IONIAN).unwrap(),
            root: ScaleNote::new(QuarterTone::D1, 1),
            mode: Mode::UpDownNoRepeats,
            chord: Chord::from_slice(&[0, 1, 2, 3, 4]).unwrap(),
        };
        let mut arp = Arpeggiator::new_with_configuration(configuration.clone());
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::D1, 1)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::E1, 2)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::F1, 3)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::G1, 4)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::A1, 5)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::G1, 4)));
        arp.apply_configuration(Configuration {
            chord: Chord::from_slice(&[0, 1]).unwrap(),
            ..configuration.clone()
        });
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::E1, 2)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::D1, 1)));
    }

    #[test]
    fn change_chord_and_reduce_size_with_up_down_repeat_when_going_up() {
        let mut r = TestRandom::new();
        let configuration = Configuration {
            scale: Scale::new(Tonic::C, &IONIAN).unwrap(),
            root: ScaleNote::new(QuarterTone::D1, 1),
            mode: Mode::UpDownRepeats,
            chord: Chord::from_slice(&[0, 1, 2, 3]).unwrap(),
        };
        let mut arp = Arpeggiator::new_with_configuration(configuration.clone());
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::D1, 1)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::E1, 2)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::F1, 3)));
        arp.apply_configuration(Configuration {
            chord: Chord::from_slice(&[0, 1]).unwrap(),
            ..configuration.clone()
        });
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::E1, 2)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::E1, 2)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::D1, 1)));
    }

    #[test]
    fn change_chord_and_reduce_size_with_up_down_repeat_when_going_down() {
        let mut r = TestRandom::new();
        let configuration = Configuration {
            scale: Scale::new(Tonic::C, &IONIAN).unwrap(),
            root: ScaleNote::new(QuarterTone::D1, 1),
            mode: Mode::UpDownRepeats,
            chord: Chord::from_slice(&[0, 1, 2, 3, 4]).unwrap(),
        };
        let mut arp = Arpeggiator::new_with_configuration(configuration.clone());
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::D1, 1)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::E1, 2)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::F1, 3)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::G1, 4)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::A1, 5)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::A1, 5)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::G1, 4)));
        arp.apply_configuration(Configuration {
            chord: Chord::from_slice(&[0, 1]).unwrap(),
            ..configuration.clone()
        });
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::E1, 2)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::D1, 1)));
    }

    #[test]
    fn change_scale() {
        let mut r = TestRandom::new();
        let configuration = Configuration {
            root: ScaleNote::new(QuarterTone::D1, 1),
            chord: Chord::from_slice(&[0, 1, 2]).unwrap(),
            mode: Mode::Up,
            scale: Scale::new(Tonic::C, &IONIAN).unwrap(),
        };
        let mut arp = Arpeggiator::new_with_configuration(configuration.clone());
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::D1, 1)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::E1, 2)));
        arp.apply_configuration(Configuration {
            scale: Scale::new(Tonic::C, &DORIAN).unwrap(),
            ..configuration.clone()
        });
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::F1, 3)));
    }

    #[test]
    fn change_root() {
        let mut r = TestRandom::new();
        let configuration = Configuration {
            scale: Scale::new(Tonic::C, &IONIAN).unwrap(),
            chord: Chord::from_slice(&[0, 1, 2]).unwrap(),
            mode: Mode::Up,
            root: ScaleNote::new(QuarterTone::D1, 1),
        };
        let mut arp = Arpeggiator::new_with_configuration(configuration.clone());
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::D1, 1)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::E1, 2)));
        arp.apply_configuration(Configuration {
            root: ScaleNote::new(QuarterTone::C1, 0),
            ..configuration.clone()
        });
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::E1, 2)));
    }

    #[test]
    fn change_mode() {
        let mut r = TestRandom::new();
        let configuration = Configuration {
            scale: Scale::new(Tonic::C, &IONIAN).unwrap(),
            chord: Chord::from_slice(&[0, 1, 2]).unwrap(),
            mode: Mode::Up,
            root: ScaleNote::new(QuarterTone::D1, 1),
        };
        let mut arp = Arpeggiator::new_with_configuration(configuration.clone());
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::D1, 1)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::E1, 2)));
        arp.apply_configuration(Configuration {
            mode: Mode::UpDownNoRepeats,
            ..configuration.clone()
        });
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::F1, 3)));
        assert_eq!(arp.pop(&mut r), Some(ScaleNote::new(QuarterTone::E1, 2)));
    }
}
