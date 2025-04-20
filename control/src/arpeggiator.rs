use crate::chords::Chord;
use crate::random::Random;
use crate::scales::ProjectedScale;
use crate::scales::ScaleNote;

#[derive(Clone, Debug, defmt::Format)]
pub struct Arpeggiator {
    scale: ProjectedScale,
    root: ScaleNote,
    chord: Chord,
    mode: Mode,
    state: State,
    voct_cache: f32,
}

// ALLOW: All the values are constructed via `try_from_index`.
#[allow(dead_code)]
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Debug, defmt::Format)]
pub enum Mode {
    UpWithReset = 0,
    UpDownNoRepeatsWithReset,
    UpDownRepeatsWithReset,
    RandomWithReset,
    MovingWithReset,
    RandomWithNext,
    MovingWithNext,
}

impl Default for Mode {
    fn default() -> Self {
        Self::UpWithReset
    }
}

#[derive(Clone, Debug, defmt::Format)]
pub enum State {
    Up(usize),
    Down(usize),
    Shuffled(usize, Chord),
}

#[derive(Clone, Debug, defmt::Format)]
pub struct Configuration {
    pub scale: ProjectedScale,
    pub root: ScaleNote,
    pub chord: Chord,
    pub mode: Mode,
    pub reset_next: bool,
}

impl Arpeggiator {
    pub fn with_config(config: Configuration) -> Self {
        Self {
            scale: config.scale,
            root: config.root,
            mode: config.mode,
            state: match config.mode {
                Mode::UpWithReset
                | Mode::UpDownNoRepeatsWithReset
                | Mode::UpDownRepeatsWithReset => State::Up(0),
                Mode::RandomWithReset
                | Mode::RandomWithNext
                | Mode::MovingWithReset
                | Mode::MovingWithNext => State::Shuffled(0, config.chord.clone()),
            },
            chord: config.chord,
            voct_cache: 0.0,
        }
    }

    pub fn apply_config(&mut self, config: Configuration, random: &mut impl Random) {
        if self.mode != config.mode {
            self.mode = config.mode;
            match self.mode {
                Mode::UpWithReset
                | Mode::UpDownRepeatsWithReset
                | Mode::UpDownNoRepeatsWithReset => {
                    if !matches!(self.state, State::Up(_)) {
                        self.state = State::Up(0);
                    }
                }
                Mode::RandomWithReset
                | Mode::RandomWithNext
                | Mode::MovingWithReset
                | Mode::MovingWithNext => self.state = State::Shuffled(0, self.chord.clone()),
            }
        }

        if config.reset_next {
            match config.mode {
                Mode::UpWithReset
                | Mode::UpDownNoRepeatsWithReset
                | Mode::UpDownRepeatsWithReset => self.state = State::Up(0),
                Mode::RandomWithReset | Mode::MovingWithReset => {
                    self.state = State::Shuffled(0, config.chord.clone())
                }
                Mode::RandomWithNext => {
                    if let State::Shuffled(ref mut index, ref mut randomized_chord) = self.state {
                        let last_index = self.chord.len() - 1;
                        for d in randomized_chord.iter_mut() {
                            *d = self.chord[random.u8_mod(last_index as u8 + 1) as usize];
                        }
                        *index = 0;
                    } else {
                        unreachable!();
                    }
                }
                Mode::MovingWithNext => {
                    if let State::Shuffled(ref mut index, ref mut schuffled_chord) = self.state {
                        let last_index = self.chord.len() - 1;
                        let (random_a, random_b) = two_distinct_random_values(last_index, random);
                        schuffled_chord.swap(random_a, random_b);
                        *index = 0;
                    } else {
                        unreachable!();
                    }
                }
            };
        }

        if self.chord != config.chord {
            if let State::Shuffled(_, schuffled_chord) = &mut self.state {
                *schuffled_chord = config.chord.clone();
            }
            self.chord = config.chord;
        }

        self.scale = config.scale;
        self.root = config.root;
    }

    pub fn pop(&mut self, random: &mut impl Random) -> Option<(ScaleNote, i16)> {
        // XXX: Empty chords don't make sense. This check simplifies the rest
        // of the method.
        // PANIC: The chord is set from the chord bank which has no empty
        // chords, which is asserted in the bank's initialization in
        // control/src/chords.rs. This will never panic.
        assert!(!self.chord.is_empty());

        let chord_degree = match self.state {
            State::Up(index) => {
                let last_index = self.chord.len() - 1;
                if last_index == 0 {
                    self.chord[0]
                } else if index < last_index {
                    self.state = State::Up(index + 1);
                    self.chord[index]
                } else if index == last_index {
                    let (new_state, new_index) = match self.mode {
                        Mode::UpWithReset => (State::Up(0), last_index),
                        Mode::UpDownRepeatsWithReset => (State::Down(last_index), last_index),
                        Mode::UpDownNoRepeatsWithReset => (State::Down(last_index - 1), last_index),
                        _ => unreachable!(),
                    };
                    self.state = new_state;
                    self.chord[new_index]
                } else {
                    let (new_state, new_index) = match self.mode {
                        Mode::UpWithReset => (State::Up(1), 0),
                        Mode::UpDownRepeatsWithReset => (State::Down(last_index), last_index),
                        Mode::UpDownNoRepeatsWithReset => (State::Down(last_index - 1), last_index),
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
                        Mode::UpWithReset | Mode::UpDownNoRepeatsWithReset => {
                            (State::Up(index + 1), index)
                        }
                        Mode::UpDownRepeatsWithReset => (State::Up(index), index),
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
            State::Shuffled(ref mut index, ref mut schuffled_chord) => {
                let last_index = self.chord.len() - 1;
                if last_index == 0 {
                    self.chord[last_index]
                } else if *index >= last_index {
                    let degree = schuffled_chord[last_index];
                    match self.mode {
                        Mode::RandomWithReset => {
                            for d in schuffled_chord.iter_mut() {
                                *d = self.chord[random.u8_mod(last_index as u8 + 1) as usize];
                            }
                        }
                        Mode::MovingWithReset => {
                            let (random_a, random_b) =
                                two_distinct_random_values(last_index, random);
                            schuffled_chord.swap(random_a, random_b);
                        }
                        Mode::MovingWithNext | Mode::RandomWithNext => (),
                        _ => unreachable!(),
                    }
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
            .map(|n| {
                self.voct_cache = n.tone().voct();
                (n, chord_degree)
            })
    }

    pub fn last_voct_output(&self) -> f32 {
        self.voct_cache
    }
}

impl Mode {
    pub const LAST_MODE: Self = Self::MovingWithNext;

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn try_from_index(index: usize) -> Option<Self> {
        if index <= Self::LAST_MODE.index() {
            Some(unsafe { core::mem::transmute::<u8, Self>(index as u8) })
        } else {
            None
        }
    }
}

fn two_distinct_random_values(max: usize, random: &mut impl Random) -> (usize, usize) {
    let a = random.u8_mod(max as u8 + 1) as usize;
    let mut b = random.u8_mod(max as u8 + 1) as usize;
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
    use crate::scales::QuarterTone;
    use crate::scales::Tonic;
    use crate::scales::{Scale, Scales};

    use super::*;

    struct TestRandom {
        values: [u8; 3],
        index: usize,
    }

    impl TestRandom {
        fn new() -> Self {
            Self {
                values: [0; 3],
                index: 0,
            }
        }

        fn new_with_values(values: [u8; 3]) -> Self {
            Self { values, index: 0 }
        }
    }

    impl Random for TestRandom {
        fn u8_mod(&mut self, _mod: u8) -> u8 {
            let value = self.values[self.index];
            self.index += 1;
            self.index %= self.values.len();
            value
        }
    }

    pub fn ionian() -> Scale {
        let scales = Scales::new();
        scales.scale(crate::scales::GroupId::Diatonic, 0).unwrap()
    }

    pub fn dorian() -> Scale {
        let scales = Scales::new();
        scales.scale(crate::scales::GroupId::Diatonic, 1).unwrap()
    }

    #[test]
    fn initialize() {
        let configuration = Configuration {
            scale: ionian().with_tonic(Tonic::C),
            root: ScaleNote::new(QuarterTone::C1, 0),
            chord: Chord::from_slice(&[0, 1]).unwrap(),
            mode: Mode::UpWithReset,
            reset_next: false,
        };
        let _arp = Arpeggiator::with_config(configuration);
    }

    #[test]
    fn up_arp() {
        let mut r = TestRandom::new();
        let configuration = Configuration {
            scale: ionian().with_tonic(Tonic::C),
            root: ScaleNote::new(QuarterTone::D1, 1),
            chord: Chord::from_slice(&[0, 1, 2]).unwrap(),
            mode: Mode::UpWithReset,
            reset_next: false,
        };
        let mut arp = Arpeggiator::with_config(configuration);
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::D1, 1), 0))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::E1, 2), 1))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::F1, 3), 2))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::D1, 1), 0))
        );
    }

    #[test]
    fn up_down_no_repeat_arp() {
        let mut r = TestRandom::new();
        let configuration = Configuration {
            scale: ionian().with_tonic(Tonic::C),
            chord: Chord::from_slice(&[0, 1, 2]).unwrap(),
            mode: Mode::UpDownNoRepeatsWithReset,
            root: ScaleNote::new(QuarterTone::D1, 1),
            reset_next: false,
        };
        let mut arp = Arpeggiator::with_config(configuration.clone());
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::D1, 1), 0))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::E1, 2), 1))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::F1, 3), 2))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::E1, 2), 1))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::D1, 1), 0))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::E1, 2), 1))
        );
    }

    #[test]
    fn up_down_repeat_arp() {
        let mut r = TestRandom::new();
        let configuration = Configuration {
            scale: ionian().with_tonic(Tonic::C),
            chord: Chord::from_slice(&[0, 1, 2]).unwrap(),
            mode: Mode::UpDownRepeatsWithReset,
            root: ScaleNote::new(QuarterTone::D1, 1),
            reset_next: false,
        };
        let mut arp = Arpeggiator::with_config(configuration.clone());
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::D1, 1), 0))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::E1, 2), 1))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::F1, 3), 2))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::F1, 3), 2))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::E1, 2), 1))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::D1, 1), 0))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::D1, 1), 0))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::E1, 2), 1))
        );
    }

    #[test]
    fn random_arp() {
        let mut r = TestRandom::new_with_values([0, 2, 1]);
        let configuration = Configuration {
            scale: ionian().with_tonic(Tonic::C),
            chord: Chord::from_slice(&[0, 1, 2]).unwrap(),
            mode: Mode::RandomWithReset,
            root: ScaleNote::new(QuarterTone::D1, 1),
            reset_next: false,
        };
        let mut arp = Arpeggiator::with_config(configuration.clone());
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::D1, 1), 0))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::E1, 2), 1))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::F1, 3), 2))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::D1, 1), 0))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::F1, 3), 2))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::E1, 2), 1))
        );
    }

    #[test]
    fn moving_arp() {
        let mut r = TestRandom::new_with_values([0, 2, 1]);
        let configuration = Configuration {
            scale: ionian().with_tonic(Tonic::C),
            chord: Chord::from_slice(&[0, 1, 2]).unwrap(),
            mode: Mode::MovingWithReset,
            root: ScaleNote::new(QuarterTone::D1, 1),
            reset_next: false,
        };
        let mut arp = Arpeggiator::with_config(configuration.clone());
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::D1, 1), 0))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::E1, 2), 1))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::F1, 3), 2))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::F1, 3), 2))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::E1, 2), 1))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::D1, 1), 0))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::E1, 2), 1))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::F1, 3), 2))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::D1, 1), 0))
        );
    }

    #[test]
    fn change_chord_but_keep_size() {
        let mut r = TestRandom::new();
        let configuration = Configuration {
            scale: ionian().with_tonic(Tonic::C),
            root: ScaleNote::new(QuarterTone::D1, 1),
            mode: Mode::UpWithReset,
            chord: Chord::from_slice(&[0, 1, 2]).unwrap(),
            reset_next: false,
        };
        let mut arp = Arpeggiator::with_config(configuration.clone());
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::D1, 1), 0))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::E1, 2), 1))
        );
        arp.apply_config(
            Configuration {
                chord: Chord::from_slice(&[0, 1, 3]).unwrap(),
                ..configuration.clone()
            },
            &mut r,
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::G1, 4), 3))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::D1, 1), 0))
        );
    }

    #[test]
    fn change_chord_and_reduce_size_with_up() {
        let mut r = TestRandom::new();
        let configuration = Configuration {
            scale: ionian().with_tonic(Tonic::C),
            root: ScaleNote::new(QuarterTone::D1, 1),
            mode: Mode::UpWithReset,
            chord: Chord::from_slice(&[0, 1, 2, 3]).unwrap(),
            reset_next: false,
        };
        let mut arp = Arpeggiator::with_config(configuration.clone());
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::D1, 1), 0))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::E1, 2), 1))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::F1, 3), 2))
        );
        arp.apply_config(
            Configuration {
                chord: Chord::from_slice(&[0, 1]).unwrap(),
                ..configuration.clone()
            },
            &mut r,
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::D1, 1), 0))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::E1, 2), 1))
        );
    }

    #[test]
    fn change_chord_and_reduce_size_with_up_down_no_repeat_when_going_up() {
        let mut r = TestRandom::new();
        let configuration = Configuration {
            scale: ionian().with_tonic(Tonic::C),
            root: ScaleNote::new(QuarterTone::D1, 1),
            mode: Mode::UpDownNoRepeatsWithReset,
            chord: Chord::from_slice(&[0, 1, 2, 3]).unwrap(),
            reset_next: false,
        };
        let mut arp = Arpeggiator::with_config(configuration.clone());
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::D1, 1), 0))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::E1, 2), 1))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::F1, 3), 2))
        );
        arp.apply_config(
            Configuration {
                chord: Chord::from_slice(&[0, 1]).unwrap(),
                ..configuration.clone()
            },
            &mut r,
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::E1, 2), 1))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::D1, 1), 0))
        );
    }

    #[test]
    fn change_chord_and_reduce_size_with_up_down_no_repeat_when_going_down() {
        let mut r = TestRandom::new();
        let configuration = Configuration {
            scale: ionian().with_tonic(Tonic::C),
            root: ScaleNote::new(QuarterTone::D1, 1),
            mode: Mode::UpDownNoRepeatsWithReset,
            chord: Chord::from_slice(&[0, 1, 2, 3, 4]).unwrap(),
            reset_next: false,
        };
        let mut arp = Arpeggiator::with_config(configuration.clone());
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::D1, 1), 0))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::E1, 2), 1))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::F1, 3), 2))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::G1, 4), 3))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::A1, 5), 4))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::G1, 4), 3))
        );
        arp.apply_config(
            Configuration {
                chord: Chord::from_slice(&[0, 1]).unwrap(),
                ..configuration.clone()
            },
            &mut r,
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::E1, 2), 1))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::D1, 1), 0))
        );
    }

    #[test]
    fn change_chord_and_reduce_size_with_up_down_repeat_when_going_up() {
        let mut r = TestRandom::new();
        let configuration = Configuration {
            scale: ionian().with_tonic(Tonic::C),
            root: ScaleNote::new(QuarterTone::D1, 1),
            mode: Mode::UpDownRepeatsWithReset,
            chord: Chord::from_slice(&[0, 1, 2, 3]).unwrap(),
            reset_next: false,
        };
        let mut arp = Arpeggiator::with_config(configuration.clone());
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::D1, 1), 0))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::E1, 2), 1))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::F1, 3), 2))
        );
        arp.apply_config(
            Configuration {
                chord: Chord::from_slice(&[0, 1]).unwrap(),
                ..configuration.clone()
            },
            &mut r,
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::E1, 2), 1))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::E1, 2), 1))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::D1, 1), 0))
        );
    }

    #[test]
    fn change_chord_and_reduce_size_with_up_down_repeat_when_going_down() {
        let mut r = TestRandom::new();
        let configuration = Configuration {
            scale: ionian().with_tonic(Tonic::C),
            root: ScaleNote::new(QuarterTone::D1, 1),
            mode: Mode::UpDownRepeatsWithReset,
            chord: Chord::from_slice(&[0, 1, 2, 3, 4]).unwrap(),
            reset_next: false,
        };
        let mut arp = Arpeggiator::with_config(configuration.clone());
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::D1, 1), 0))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::E1, 2), 1))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::F1, 3), 2))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::G1, 4), 3))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::A1, 5), 4))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::A1, 5), 4))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::G1, 4), 3))
        );
        arp.apply_config(
            Configuration {
                chord: Chord::from_slice(&[0, 1]).unwrap(),
                ..configuration.clone()
            },
            &mut r,
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::E1, 2), 1))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::D1, 1), 0))
        );
    }

    #[test]
    fn change_scale() {
        let mut r = TestRandom::new();
        let configuration = Configuration {
            root: ScaleNote::new(QuarterTone::D1, 1),
            chord: Chord::from_slice(&[0, 1, 2]).unwrap(),
            mode: Mode::UpWithReset,
            scale: ionian().with_tonic(Tonic::C),
            reset_next: false,
        };
        let mut arp = Arpeggiator::with_config(configuration.clone());
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::D1, 1), 0))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::E1, 2), 1))
        );
        arp.apply_config(
            Configuration {
                scale: dorian().with_tonic(Tonic::C),
                ..configuration.clone()
            },
            &mut r,
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::F1, 3), 2))
        );
    }

    #[test]
    fn change_root() {
        let mut r = TestRandom::new();
        let configuration = Configuration {
            scale: ionian().with_tonic(Tonic::C),
            chord: Chord::from_slice(&[0, 1, 2]).unwrap(),
            mode: Mode::UpWithReset,
            root: ScaleNote::new(QuarterTone::D1, 1),
            reset_next: false,
        };
        let mut arp = Arpeggiator::with_config(configuration.clone());
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::D1, 1), 0))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::E1, 2), 1))
        );
        arp.apply_config(
            Configuration {
                root: ScaleNote::new(QuarterTone::C1, 0),
                ..configuration.clone()
            },
            &mut r,
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::E1, 2), 2))
        );
    }

    #[test]
    fn change_mode() {
        let mut r = TestRandom::new();
        let configuration = Configuration {
            scale: ionian().with_tonic(Tonic::C),
            chord: Chord::from_slice(&[0, 1, 2]).unwrap(),
            mode: Mode::UpWithReset,
            root: ScaleNote::new(QuarterTone::D1, 1),
            reset_next: false,
        };
        let mut arp = Arpeggiator::with_config(configuration.clone());
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::D1, 1), 0))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::E1, 2), 1))
        );
        arp.apply_config(
            Configuration {
                mode: Mode::UpDownNoRepeatsWithReset,
                ..configuration.clone()
            },
            &mut r,
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::F1, 3), 2))
        );
        assert_eq!(
            arp.pop(&mut r),
            Some((ScaleNote::new(QuarterTone::E1, 2), 1))
        );
    }
}
