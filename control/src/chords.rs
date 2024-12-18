use heapless::Vec;

const MAX_CHORD_SIZE: usize = 25;

pub type Chord = Vec<i16, MAX_CHORD_SIZE>;

pub struct Chords {
    size_1: LibraryGroup<7, 1>,
    size_2: LibraryGroup<6, 2>,
    size_3: LibraryGroup<18, 3>,
    size_4: LibraryGroup<18, 4>,
    size_5: LibraryGroup<16, 5>,
    size_6: LibraryGroup<8, 6>,
    size_7: LibraryGroup<8, 7>,
    size_8: LibraryGroup<9, 8>,
}

// ALLOW: All the values are constructed via `try_from`.
#[allow(dead_code)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, defmt::Format)]
pub enum GroupId {
    Size1 = 0,
    Size2,
    Size3,
    Size4,
    Size5,
    Size6,
    Size7,
    Size8,
}

type LibraryGroup<const N: usize, const D: usize> = Vec<LibraryChord<D>, N>;

type LibraryChord<const D: usize> = Vec<i16, D>;

impl Chords {
    pub const GROUPS: usize = 8;

    // NOTE: Keep the lists expanded to improve readability.
    // TODO: Make sure all are ascending
    // TODO: Make it so incrasing size keeps all the intervals from the previous size. Or at least on sizes 3+
    #[rustfmt::skip]
    pub fn new() -> Self {
        let size_1 = initialize_group(&[
            &[0],
            &[1],
            &[2],
            &[3],
            &[4],
            &[5],
            &[6],
        ]);
        let size_2 = initialize_group(&[
            &[0, 1],
            &[0, 2],
            &[0, 3],
            &[0, 4],
            &[0, 5],
            &[0, 6],
        ]);
        let size_3 = initialize_group(&[
            &[0, 2, 4],
            &[0, 2, 5],
            &[0, 2, 6],
            &[0, 1, 4],
            &[0, 1, 5],
            &[0, 1, 6],
            &[0, 3, 4],
            &[0, 3, 5],
            &[0, 3, 6],
            &[0, 4, 7 + 2],
            &[0, 5, 7 + 2],
            &[0, 6, 7 + 2],
            &[0, 4, 7 + 1],
            &[0, 5, 7 + 1],
            &[0, 6, 7 + 1],
            &[0, 4, 7 + 3],
            &[0, 5, 7 + 3],
            &[0, 6, 7 + 3],
        ]);
        let size_4 = initialize_group(&[
            &[0, 2, 5, 7],
            &[0, 2, 6, 7],
            &[0, 2, 8, 7],
            &[0, 3, 5, 7],
            &[0, 3, 6, 7],
            &[0, 3, 8, 7],
            &[0, 1, 5, 7],
            &[0, 1, 6, 7],
            &[0, 1, 8, 7],
            &[0, 2, 4, 5],
            &[0, 2, 4, 6],
            &[0, 2, 4, 8],
            &[0, 3, 4, 5],
            &[0, 3, 4, 6],
            &[0, 3, 4, 8],
            &[0, 1, 4, 5],
            &[0, 1, 4, 6],
            &[0, 1, 4, 8],
        ]);
        let size_5 = initialize_group(&[
            &[0, 2, 4, 5, 7],
            &[0, 1, 4, 5, 7],
            &[0, 2, 4, 6, 7],
            &[0, 1, 4, 6, 7],
            &[0, 2, 3, 5, 7],
            &[0, 1, 3, 5, 7],
            &[0, 2, 3, 6, 7],
            &[0, 1, 3, 6, 7],
            &[0, 4, 7, 7 + 2, 7 + 5],
            &[0, 4, 7, 7 + 1, 7 + 5],
            &[0, 4, 7, 7 + 2, 7 + 6],
            &[0, 4, 7, 7 + 1, 7 + 6],
            &[0, 3, 7, 7 + 2, 7 + 5],
            &[0, 3, 7, 7 + 1, 7 + 5],
            &[0, 3, 7, 7 + 2, 7 + 6],
            &[0, 3, 7, 7 + 1, 7 + 6],
        ]);
        let size_6 = initialize_group(&[
            &[0, 2, 4, 5, 7, 7 + 2],
            &[0, 1, 4, 5, 7, 7 + 1],
            &[0, 2, 4, 6, 7, 7 + 2],
            &[0, 1, 4, 6, 7, 7 + 1],
            &[0, 2, 3, 5, 7, 7 + 2],
            &[0, 1, 3, 5, 7, 7 + 1],
            &[0, 2, 3, 6, 7, 7 + 2],
            &[0, 1, 3, 6, 7, 7 + 1],
        ]);
        let size_7 = initialize_group(&[
            &[0, 2, 4, 5, 7, 7 + 2, 7 + 4],
            &[0, 1, 4, 5, 7, 7 + 1, 7 + 4],
            &[0, 2, 4, 6, 7, 7 + 2, 7 + 4],
            &[0, 1, 4, 6, 7, 7 + 1, 7 + 4],
            &[0, 2, 3, 5, 7, 7 + 2, 7 + 3],
            &[0, 1, 3, 5, 7, 7 + 1, 7 + 3],
            &[0, 2, 3, 6, 7, 7 + 2, 7 + 3],
            &[0, 1, 3, 6, 7, 7 + 1, 7 + 3],
        ]);
        let size_8 = initialize_group(&[
            &[0, 2, 4, 5, 7, 7 + 2, 7 + 4, 7 + 5],
            &[0, 1, 4, 5, 7, 7 + 1, 7 + 4, 7 + 5],
            &[0, 2, 4, 6, 7, 7 + 2, 7 + 4, 7 + 6],
            &[0, 1, 4, 6, 7, 7 + 1, 7 + 4, 7 + 6],
            &[0, 2, 3, 5, 7, 7 + 2, 7 + 3, 7 + 5],
            &[0, 1, 3, 5, 7, 7 + 1, 7 + 3, 7 + 5],
            &[0, 2, 3, 6, 7, 7 + 2, 7 + 3, 7 + 6],
            &[0, 1, 3, 6, 7, 7 + 1, 7 + 3, 7 + 6],
            &[0, 1, 2, 3, 4, 5, 6, 7],
        ]);
        Self {
            size_1,
            size_2,
            size_3,
            size_4,
            size_5,
            size_6,
            size_7,
            size_8,
        }
    }

    pub fn number_of_chords(&self, group_id: GroupId) -> usize {
        match group_id {
            GroupId::Size1 => self.size_1.capacity(),
            GroupId::Size2 => self.size_2.capacity(),
            GroupId::Size3 => self.size_3.capacity(),
            GroupId::Size4 => self.size_4.capacity(),
            GroupId::Size5 => self.size_5.capacity(),
            GroupId::Size6 => self.size_6.capacity(),
            GroupId::Size7 => self.size_7.capacity(),
            GroupId::Size8 => self.size_8.capacity(),
        }
    }

    pub fn chord(&self, group_id: GroupId, chord_index: usize) -> Result<Chord, ()> {
        if chord_index >= self.number_of_chords(group_id) {
            return Err(());
        }

        match group_id {
            // SAFETY: Correct capacity is checked during the initialization.
            GroupId::Size1 => Chord::from_slice(self.size_1.get(chord_index).unwrap()),
            GroupId::Size2 => Chord::from_slice(self.size_2.get(chord_index).unwrap()),
            GroupId::Size3 => Chord::from_slice(self.size_3.get(chord_index).unwrap()),
            GroupId::Size4 => Chord::from_slice(self.size_4.get(chord_index).unwrap()),
            GroupId::Size5 => Chord::from_slice(self.size_5.get(chord_index).unwrap()),
            GroupId::Size6 => Chord::from_slice(self.size_6.get(chord_index).unwrap()),
            GroupId::Size7 => Chord::from_slice(self.size_7.get(chord_index).unwrap()),
            GroupId::Size8 => Chord::from_slice(self.size_7.get(chord_index).unwrap()),
        }
    }

    pub fn group_size(&self, group_id: GroupId) -> usize {
        match group_id {
            GroupId::Size1 => self.size_1.degrees_capacity(),
            GroupId::Size2 => self.size_2.degrees_capacity(),
            GroupId::Size3 => self.size_3.degrees_capacity(),
            GroupId::Size4 => self.size_4.degrees_capacity(),
            GroupId::Size5 => self.size_5.degrees_capacity(),
            GroupId::Size6 => self.size_6.degrees_capacity(),
            GroupId::Size7 => self.size_7.degrees_capacity(),
            GroupId::Size8 => self.size_8.degrees_capacity(),
        }
    }
}

trait LibraryGroupTrait {
    fn degrees_capacity(&self) -> usize;
}

impl<const N: usize, const D: usize> LibraryGroupTrait for LibraryGroup<N, D> {
    fn degrees_capacity(&self) -> usize {
        D
    }
}

impl TryFrom<usize> for GroupId {
    type Error = ();

    fn try_from(index: usize) -> Result<Self, Self::Error> {
        if index >= Chords::GROUPS {
            return Err(());
        }
        Ok(unsafe { core::mem::transmute::<u8, Self>(index as u8) })
    }
}

fn initialize_group<const N: usize, const D: usize>(chords_slice: &[&[i16]]) -> LibraryGroup<N, D> {
    assert!(N > 0, "LibraryGroup must not be empty");
    assert!(
        D <= Chord::new().capacity(),
        "LibraryGroup would contain bigger chords than is the maximum Chord capacity"
    );
    assert_eq!(
        chords_slice.len(),
        N,
        "LibraryGroup would be over or underutilized"
    );

    let mut group = LibraryGroup::new();

    for chord_slice in chords_slice {
        assert_eq!(chord_slice.len(), D, "Given chord is too big or too small");
        let chord = LibraryChord::from_slice(chord_slice).unwrap();
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
        let chord = chords.chord(GroupId::Size3, 0).unwrap();
        assert_eq!(&chord, &[0, 2, 4]);
    }

    #[test]
    fn try_getting_chord_out_of_range() {
        let chords = Chords::new();
        assert!(chords
            .chord(GroupId::Size3, chords.number_of_chords(GroupId::Size3))
            .is_err());
    }
}
