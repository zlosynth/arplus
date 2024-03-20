// TODO: Refactor Chords and Scales to they have the same structure

mod quarter_tones;
mod scale;
mod scale_note;
mod tonic;

use heapless::Vec;

use self::scale::{Step, Q, S, T};

pub use self::quarter_tones::QuarterTone;
pub use self::scale::Scale as GenericProjectedScale;
pub use self::scale_note::ScaleNote;
pub use self::tonic::Tonic;

pub type Scale = LibraryScale<12>;
pub type ProjectedScale = GenericProjectedScale<12>;

pub struct Scales {
    diatonic: LibraryGroup<7, 7>,
    chromatic: LibraryGroup<1, 12>,
    maqam: LibraryGroup<8, 7>,
    // melakarta: (), http://ecmc.rochester.edu/rdm/pdflib/mela.pdf https://www.quora.com/What-are-some-ragas-which-are-more-popular-than-the-Melakartha-ragas-to-which-they-belong-to
    // - TODO: Go over https://www.quora.com/What-are-some-ragas-which-are-more-popular-than-the-Melakartha-ragas-to-which-they-belong-to scales mentioned here, listen to them, pick up to 8
    // - Mayamalavagowla (misirlou)
    // blues: (),
    // hexatonic: (),
    // tetratonic: (),
    // special heptatonic scales
    // javan scale https://www.youtube.com/watch?v=YWfumqpFwaY
    // chinese https://www.youtube.com/watch?v=WHnrpZaif5w or https://www.youtube.com/watch?v=tc6-qk6RLFw
}

// ALLOW: All the variants can be contructed via `try_from`.
#[allow(dead_code)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, defmt::Format)]
pub enum GroupId {
    Diatonic,
    Chromatic,
    Maqam,
}

type LibraryGroup<const N: usize, const S: usize> = Vec<LibraryScale<S>, N>;

#[derive(Debug, Clone, defmt::Format)]
pub struct LibraryScale<const S: usize> {
    ascending: Vec<Step, S>,
}

impl Scales {
    pub const GROUPS: usize = 2;

    // NOTE: Keep the lists expanded to improve readability.
    #[rustfmt::skip]
    pub fn new() -> Self {
        const Q3: Step = 3 * Q;
        const S3: Step = 3 * S;

        let diatonic = initialize_group(&[
            (&[T, T, S, T, T, T, S], None), // Ionian
            (&[T, S, T, T, T, S, T], None), // Dorian
            (&[S, T, T, T, S, T, T], None), // Phrygian
            (&[T, T, T, S, T, T, S], None), // Lydian
            (&[T, T, S, T, T, S, T], None), // Mixolydian
            (&[T, S, T, T, S, T, T], None), // Aeolian
            (&[S, T, T, S, T, T, T], None), // Locrian
        ]);
        let chromatic = initialize_group(&[
            (&[S, S, S, S, S, S, S, S, S, S, S, S], None)
        ]);
        let maqab = initialize_group(&[
            // Bayati D Ep F G A Bb C D
            (&[Q3, Q3, T, T, S, T, T], None),
            // Hijaz D Eb Fs G A Bb C D
            (&[S, S3, S, T, S, T, T], None),
            // Kurd D Eb F G A Bb C D
            (&[S, T, T, T, S, T, T], None),
            // Nahawand C D Eb F G Ab B C
            (&[T, S, T, T, S, S3, S], None),
            // Nawa Athar C D Eb Fs G Ab B C
            (&[T, S, S3, S, S, S3, S], None),
            // Rast C D Ep F G A Bp C
            (&[T, Q3, Q3, T, T, Q3, Q3], None),
            // Saba D Ep F Gb A Bb C D
            (&[Q3, Q3, S, S3, S, T, T], None),
            // Sikah Ep F G A Bp C D Ep
            (&[Q3, T, T, Q3, Q3, T, Q3], None),
        ]);
        Self {
            diatonic,
            chromatic,
            maqam: maqab,
        }
    }

    pub fn number_of_scales(&self, group_id: GroupId) -> usize {
        match group_id {
            GroupId::Diatonic => self.diatonic.capacity(),
            GroupId::Chromatic => self.chromatic.capacity(),
            GroupId::Maqam => self.maqam.capacity(),
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
            GroupId::Maqam => Scale::new(&self.maqam.get(scale_index).unwrap().ascending),
        }
    }

    pub fn number_of_steps_in_group(&self, group_id: GroupId) -> usize {
        match group_id {
            GroupId::Diatonic => self.diatonic.steps_capacity(),
            GroupId::Chromatic => self.chromatic.steps_capacity(),
            GroupId::Maqam => self.maqam.steps_capacity(),
        }
    }
}

trait LibraryGroupTrait {
    fn steps_capacity(&self) -> usize;
}

impl<const N: usize, const S: usize> LibraryGroupTrait for LibraryGroup<N, S> {
    fn steps_capacity(&self) -> usize {
        S
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

    pub fn with_tonic(&self, tonic: Tonic) -> ProjectedScale {
        // SAFETY: Size of the slice is already checked against S.
        ProjectedScale::new(tonic, &self.ascending).unwrap()
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
    assert_eq!(
        scales_slice.len(),
        N,
        "LibraryGroup would be over or underutilized"
    );

    let mut group = LibraryGroup::new();

    for (ascending_slice, _descending_slice) in scales_slice {
        assert_eq!(
            ascending_slice.len(),
            S,
            "Given scale is too big or too small"
        );
        let chord = LibraryScale::new(ascending_slice).unwrap();
        // SAFETY: The capacity is checked at the beginning of the function.
        group.push(chord).unwrap();
    }

    group
}
