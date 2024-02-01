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
