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
        let group_1 = initialize_group(&[&[0, 2, 4], &[0, 1, 4], &[0, 3, 4]]);
        let group_2 = initialize_group(&[&[0, 2, 4, 6], &[0, 1, 4, 6], &[0, 3, 4, 6]]);
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
            _ => panic!("A valid group index is not covered"),
        })
    }

    pub fn chord(&self, group_index: usize, chord_index: usize) -> Option<Chord> {
        if chord_index >= self.number_of_chords(group_index)? {
            return None;
        }

        match group_index {
            0 => Chord::from_slice(self.group_1.get(chord_index).unwrap()),
            1 => Chord::from_slice(self.group_2.get(chord_index).unwrap()),
            _ => panic!("A valid group index is not covered"),
        }
        .ok()
    }
}

fn initialize_group<const N: usize, const D: usize>(chords_slice: &[&[i16]]) -> LibraryGroup<N, D> {
    assert!(
        D <= Chord::new().capacity(),
        "LibraryGroup would contain bigger chords than is the maximum Chord capacity"
    );
    assert!(
        chords_slice.len() == N,
        "LibraryGroup would be over or underutilized"
    );

    let mut group = LibraryGroup::new();

    for chord_slice in chords_slice {
        let chord = LibraryChord::from_slice(chord_slice)
            .expect("Given chord is bigger than LibraryGroup allows");
        // SAFETY: The capacity is checked at the beginning of the function.
        group.push(chord).unwrap();
    }

    group
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
