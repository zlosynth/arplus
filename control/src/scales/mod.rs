use heapless::Vec;

use self::{
    scale::{Scale as ProjectedScale, Step, S, T},
    tonic::Tonic,
};

// TODO: Re-export what's needed, do not pub mod
pub mod quarter_tones;
pub mod scale;
pub mod scale_note;
pub mod tonic;

// TODO: Store all scales here
// TODO: Store scales in groups
// TODO: Handle scale modes here
// TODO: Impl defmt for each scale?
// TODO: Refactor Chords and Scales to they have the same structure

pub type Scale = LibraryScale<12>;

pub struct Scales {
    group_1: LibraryGroup<7, 7>, // Diatonic
    group_2: LibraryGroup<1, 12>, // Chromatic
                                 // blues: (),
                                 // arabic: (),
                                 // hexatonic: (),
                                 // tetratonic: (),
                                 // special heptatonic scales
                                 // indian scales
                                 // melakarta
}

type LibraryGroup<const N: usize, const S: usize> = Vec<LibraryScale<S>, N>;

#[derive(Debug, Clone, defmt::Format)]
pub struct LibraryScale<const S: usize> {
    ascending: Vec<Step, S>,
}

impl Scales {
    const GROUPS: usize = 2;

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
            group_1: diatonic,
            group_2: chromatic,
        }
    }

    pub fn number_of_groups(&self) -> usize {
        Self::GROUPS
    }

    pub fn number_of_scales(&self, group_index: usize) -> Result<usize, ()> {
        if group_index >= self.number_of_groups() {
            return Err(());
        }

        Ok(match group_index {
            0 => self.group_1.len(),
            1 => self.group_2.len(),
            _ => panic!("A valid group index is not covered"),
        })
    }

    pub fn scale(&self, group_index: usize, scale_index: usize) -> Result<Scale, ()> {
        if scale_index >= self.number_of_scales(group_index)? {
            return Err(());
        }

        match group_index {
            // SAFETY: Correct capacity is checked during the initialization.
            // SAFETY: The index is checked on the entry.
            0 => Scale::new(&self.group_1.get(scale_index).unwrap().ascending),
            1 => Scale::new(&self.group_2.get(scale_index).unwrap().ascending),
            _ => panic!("A valid group index is not covered"),
        }
    }

    pub fn number_of_steps_in_group(&self, group_index: usize) -> Result<usize, ()> {
        if group_index >= self.number_of_groups() {
            return Err(());
        }

        // TODO: Get size. Maybe from a trait over the wrapper type?
        Ok(match group_index {
            // TODO: No unwrap or safety note
            0 => self.group_1.get(0).unwrap().steps(),
            1 => self.group_2.get(0).unwrap().steps(),
            _ => panic!("A valid group index is not covered"),
        })
    }
}

impl<const S: usize> LibraryScale<S> {
    // TODO: This does not have to be pub
    pub fn new(ascending: &[Step]) -> Result<Self, ()> {
        Ok(Self {
            ascending: Vec::from_slice(ascending)?,
        })
    }

    fn capacity() -> usize {
        S
    }

    // TODO: This is probably really suboptimal, cloning it every time.
    // Optimize it by keeping projected scale in control lib, passing
    // projected scale to arp. And only changing it on real input change.
    pub fn with_tonic(&self, tonic: Tonic) -> ProjectedScale<S> {
        // TODO: Initialize it unchecked.
        // SAFETY: Size of the slice is already checked against S.
        ProjectedScale::new(tonic, &self.ascending).unwrap()
    }

    // TODO: Get a specific instance with tonic too
}

trait LibraryScaleTrait {
    fn steps(&self) -> usize;
}

impl<const S: usize> LibraryScaleTrait for LibraryScale<S> {
    fn steps(&self) -> usize {
        S
    }
}

fn initialize_group<const N: usize, const S: usize>(
    scales_slice: &[(&[Step], Option<&[Step]>)],
) -> LibraryGroup<N, S> {
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
