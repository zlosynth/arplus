use heapless::Vec;

pub type Chord = Vec<i16, 7>;

pub struct Chords {
    size_3: LibraryGroup<3, 3>,
    size_4: LibraryGroup<3, 4>,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, defmt::Format)]
pub enum GroupId {
    Size3 = 0,
    Size4,
}

type LibraryGroup<const N: usize, const D: usize> = Vec<LibraryChord<D>, N>;

type LibraryChord<const D: usize> = Vec<i16, D>;

impl Chords {
    pub const GROUPS: usize = 2;

    pub fn new() -> Self {
        let size_3 = initialize_group(&[&[0, 2, 4], &[0, 1, 4], &[0, 3, 4]]);
        let size_4 = initialize_group(&[&[0, 2, 4, 6], &[0, 1, 4, 6], &[0, 3, 4, 6]]);
        Self { size_3, size_4 }
    }

    pub fn number_of_chords(&self, group_id: GroupId) -> usize {
        match group_id {
            GroupId::Size3 => self.size_3.len(),
            GroupId::Size4 => self.size_4.len(),
        }
    }

    pub fn chord(&self, group_id: GroupId, chord_index: usize) -> Result<Chord, ()> {
        if chord_index >= self.number_of_chords(group_id) {
            return Err(());
        }

        match group_id {
            // SAFETY: Correct capacity is checked during the initialization.
            GroupId::Size3 => Chord::from_slice(self.size_3.get(chord_index).unwrap()),
            GroupId::Size4 => Chord::from_slice(self.size_4.get(chord_index).unwrap()),
        }
    }

    pub fn group_size(&self, group_id: GroupId) -> usize {
        // TODO: Add safety note, or implement it as a const method of group
        match group_id {
            GroupId::Size3 => self.size_3.get(0).unwrap().len(),
            GroupId::Size4 => self.size_4.get(0).unwrap().len(),
        }
    }
}

impl TryFrom<usize> for GroupId {
    type Error = ();

    fn try_from(index: usize) -> Result<Self, Self::Error> {
        if index >= Chords::GROUPS {
            return Err(());
        }
        Ok(unsafe { core::mem::transmute(index as u8) })
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
