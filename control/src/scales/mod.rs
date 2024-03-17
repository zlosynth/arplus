// TODO: Refactor Chords and Scales to they have the same structure

mod quarter_tones;
mod scale;
mod scale_note;
mod tonic;

use heapless::Vec;

use self::scale::{Scale as ProjectedScale, Step, S, T};

pub use quarter_tones::QuarterTone;
pub use scale_note::ScaleNote;
pub use tonic::Tonic;

pub type Scale = LibraryScale<12>;

pub struct Scales {
    diatonic: LibraryGroup<7, 7>,
    chromatic: LibraryGroup<1, 12>,
    // blues: (),
    // arabic: (),
    // hexatonic: (),
    // tetratonic: (),
    // special heptatonic scales
    // indian scales
    // melakarta
}

// ALLOW: All the variants can be contructed via `try_from`.
#[allow(dead_code)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, defmt::Format)]
pub enum GroupId {
    Diatonic,
    Chromatic,
}

type LibraryGroup<const N: usize, const S: usize> = Vec<LibraryScale<S>, N>;

#[derive(Debug, Clone, defmt::Format)]
pub struct LibraryScale<const S: usize> {
    ascending: Vec<Step, S>,
}

impl Scales {
    pub const GROUPS: usize = 2;

    pub fn new() -> Self {
        let diatonic = initialize_group(&[
            (&[T, T, S, T, T, T, S], None), // Ionian
            (&[T, S, T, T, T, S, T], None), // Dorian
            (&[S, T, T, T, S, T, T], None), // Phrygian
            (&[T, T, T, S, T, T, S], None), // Lydian
            (&[T, T, S, T, T, S, T], None), // Mixolydian
            (&[T, S, T, T, S, T, T], None), // Aeolian
            (&[S, T, T, S, T, T, T], None), // Locrian
        ]);
        let chromatic = initialize_group(&[(&[S, S, S, S, S, S, S, S, S, S, S, S], None)]);
        Self {
            diatonic,
            chromatic,
        }
    }

    pub fn number_of_scales(&self, group_id: GroupId) -> usize {
        match group_id {
            GroupId::Diatonic => self.diatonic.scales_len(),
            GroupId::Chromatic => self.chromatic.scales_len(),
        }
    }

    pub fn scale(&self, group_id: GroupId, scale_index: usize) -> Result<Scale, ()> {
        if scale_index >= self.number_of_scales(group_id) {
            return Err(());
        }

        match group_id {
            // SAFETY: The index is checked on the entry.
            GroupId::Diatonic => Scale::new(&self.diatonic.get(scale_index).unwrap().ascending),
            GroupId::Chromatic => Scale::new(&self.chromatic.get(scale_index).unwrap().ascending),
        }
    }

    pub fn number_of_steps_in_group(&self, group_id: GroupId) -> usize {
        match group_id {
            // SAFETY: It is checked during the initialization that libraries
            // are never empty.
            GroupId::Diatonic => self.diatonic.get(0).unwrap().steps_len(),
            GroupId::Chromatic => self.chromatic.get(0).unwrap().steps_len(),
        }
    }
}

impl<const S: usize> LibraryScale<S> {
    fn new(ascending: &[Step]) -> Result<Self, ()> {
        Ok(Self {
            ascending: Vec::from_slice(ascending)?,
        })
    }

    const fn capacity() -> usize {
        S
    }

    // TODO: This is probably really suboptimal, cloning it every time.
    // Optimize it by keeping projected scale in control lib, passing
    // projected scale to arp. And only changing it on real input change.
    pub fn with_tonic(&self, tonic: Tonic) -> ProjectedScale<S> {
        // SAFETY: Size of the slice is already checked against S.
        ProjectedScale::new(tonic, &self.ascending).unwrap()
    }
}

trait LibraryGroupTrait {
    fn scales_len(&self) -> usize;
}

impl<const N: usize, const S: usize> LibraryGroupTrait for LibraryGroup<N, S> {
    fn scales_len(&self) -> usize {
        N
    }
}

trait LibraryScaleTrait {
    fn steps_len(&self) -> usize;
}

impl<const S: usize> LibraryScaleTrait for LibraryScale<S> {
    fn steps_len(&self) -> usize {
        S
    }
}

impl TryFrom<usize> for GroupId {
    type Error = ();

    fn try_from(index: usize) -> Result<Self, Self::Error> {
        if index >= Scales::GROUPS {
            return Err(());
        }
        Ok(unsafe { core::mem::transmute(index as u8) })
    }
}

fn initialize_group<const N: usize, const S: usize>(
    scales_slice: &[(&[Step], Option<&[Step]>)],
) -> LibraryGroup<N, S> {
    assert!(N > 0, "LibraryGroup must not be empty");
    assert!(
        S <= Scale::capacity(),
        "LibraryGroup would contain bigger scales than is the maximum Chord capacity"
    );
    assert!(
        scales_slice.len() == N,
        "LibraryGroup would be over or underutilized"
    );

    let mut group = LibraryGroup::new();

    for (ascending_scale_slice, _descending_scale_slice) in scales_slice {
        let chord = LibraryScale::new(ascending_scale_slice)
            .expect("Given scale is bigger than LibraryGroup allows");
        // SAFETY: The capacity is checked at the beginning of the function.
        group.push(chord).unwrap();
    }

    group
}
