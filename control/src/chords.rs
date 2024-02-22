use heapless::Vec;

pub type Chord = Vec<i16, 7>;

pub struct Chords {
    group_1: LibraryGroup<3, 3>,
    group_2: LibraryGroup<3, 4>,
}

type LibraryGroup<const N: usize, const D: usize> = Vec<LibraryChord<D>, N>;

type LibraryChord<const D: usize> = Vec<i16, D>;

impl Chords {
    const GROUPS: usize = 2;

    pub fn new() -> Self {
        let group_1 = LibraryGroup::from_slice(&[
            LibraryChord::from_slice(&[0, 2, 4]).unwrap(),
            LibraryChord::from_slice(&[0, 1, 4]).unwrap(),
            LibraryChord::from_slice(&[0, 3, 4]).unwrap(),
        ])
        .unwrap();
        let group_2 = LibraryGroup::from_slice(&[
            LibraryChord::from_slice(&[0, 2, 4, 6]).unwrap(),
            LibraryChord::from_slice(&[0, 1, 4, 6]).unwrap(),
            LibraryChord::from_slice(&[0, 3, 4, 6]).unwrap(),
        ])
        .unwrap();
        // TODO: Check that groups are utilized to full capacity
        // TODO: Check that no LibraryChord is bigger than Chord
        Self { group_1, group_2 }
    }

    pub fn number_of_groups(&self) -> usize {
        Self::GROUPS
    }

    pub fn number_of_chords(&self, group_index: usize) -> Option<usize> {
        if group_index >= self.number_of_groups() {
            return None;
        }

        Some(match group_index {
            0 => self.group_1.len(),
            1 => self.group_2.len(),
            // TODO: Is this statically checked?
            // Self::GROUPS.. => unreachable!(),
            _ => unreachable!(),
        })
    }

    pub fn chord(&self, group_index: usize, chord_index: usize) -> Option<Chord> {
        if chord_index >= self.number_of_chords(group_index)? {
            return None;
        }

        match group_index {
            0 => Chord::from_slice(self.group_1.get(chord_index).unwrap()),
            1 => Chord::from_slice(self.group_2.get(chord_index).unwrap()),
            // TODO: Is this statically checked?
            // Self::GROUPS.. => unreachable!(),
            _ => unreachable!(),
        }
        .ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_chords() {
        let _chords = Chords::new();
    }

    #[test]
    fn get_chord() {
        let chords = Chords::new();
        let chord = chords.chord(0, 0).unwrap();
        assert_eq!(&chord, &[0, 2, 4]);
    }

    #[test]
    fn try_getting_group_out_of_range() {
        let chords = Chords::new();
        assert!(chords.chord(chords.number_of_groups(), 0).is_none());
    }

    #[test]
    fn try_getting_chord_out_of_range() {
        let chords = Chords::new();
        assert!(chords
            .chord(0, chords.number_of_chords(0).unwrap())
            .is_none());
    }

    #[test]
    fn try_getting_number_of_chords_out_of_range() {
        let chords = Chords::new();
        assert!(chords.number_of_chords(chords.number_of_groups()).is_none());
    }
}
