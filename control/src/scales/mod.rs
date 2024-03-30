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

pub type Scale = LibraryScale<24>;
pub type ProjectedScale = GenericProjectedScale<24>;

pub struct Scales {
    diatonic: LibraryGroup<7, 7>,
    maqam: LibraryGroup<8, 7>,
    melakarta: LibraryGroup<8, 7>,
    japanese: LibraryGroup<5, 5>,
    full: LibraryGroup<3, 24>,
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
#[derive(Clone, Copy, Debug, defmt::Format, PartialEq)]
pub enum GroupId {
    Diatonic = 0,
    Maqam,
    Melakarta,
    Japanese,
    Full,
}

type LibraryGroup<const N: usize, const S: usize> = Vec<LibraryScale<S>, N>;

#[derive(Debug, Clone, defmt::Format)]
pub struct LibraryScale<const S: usize> {
    ascending: Vec<Step, S>,
}

impl Scales {
    pub const GROUPS: usize = 5;

    // NOTE: Keep the lists expanded to improve readability.
    #[rustfmt::skip]
    pub fn new() -> Self {
        const Q3: Step = 3 * Q;
        const S3: Step = 3 * S;
        const T2: Step = 2 * T;

        let diatonic = initialize_group(&[
            (&[T, T, S, T, T, T, S], None), // Ionian
            (&[T, S, T, T, T, S, T], None), // Dorian
            (&[S, T, T, T, S, T, T], None), // Phrygian
            (&[T, T, T, S, T, T, S], None), // Lydian
            (&[T, T, S, T, T, S, T], None), // Mixolydian
            (&[T, S, T, T, S, T, T], None), // Aeolian
            (&[S, T, T, S, T, T, T], None), // Locrian
        ]);
        // Sources:
        // * <https://www.maqamworld.com/en/maqam/bayati.php>
        // * <https://en.wikipedia.org/wiki/Arabic_maqam>
        let maqam = initialize_group(&[
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
        // Sources:
        // * <https://library.depauw.edu/library/musiclib/RagaDatabase/observations.asp>
        // * <https://musiclegato.com/Tutorial/Carnatic.aspx?ky=C&sty=NATABHAIRAVI&pat=1-3-4-6-8-9-11-13&comb=&tp=1&idx=19&dbid=21>
        // * <https://www.quora.com/What-are-some-ragas-which-are-more-popular-than-the-Melakartha-ragas-to-which-they-belong-to>
        // * <https://en.wikipedia.org/wiki/Melakarta>
        let melakarta = initialize_group(&[
            // Ratnangi (2) C Cs D F G Ab Bb C
            (&[S, S, S3, T, S, T, T], None),
            // Rupavati (12) C Cs Ds F G As B C
            (&[S, T, T, T, S3, S, S], None),
            // Mayamalavagowla (15) G Ab B C D Eb Fs G
            (&[S, S3, S, T, S, S3, S], None),
            // Natabhairavi (20) minor C D Ds F G Gs As C
            (&[T, S, T, T, S, T, T], None),
            // Karaharapriya (22) C D Ds F G A As C
            (&[T, S, T, T, T, S, T], None),
            // Sangarabharanam (29) major C D E F G A B C
            (&[T, T, S, T, T, T, S], None),
            // Jalavarali (39) C Cs D Fs G Gs B C
            (&[S, S, T2, S, S, S3, S], None),
            // Gamanasrama (53) C Cs E Fs G A B C
            (&[S, S3, T, S, T, T, S], None),
        ]);
        // Source: <https://www.musicnotes.com/blog/japanese-scales-in-music-theory/>
        let japanese = initialize_group(&[
            // Hirajoshi C D Eb G Ab C
            (&[T, S, T2, S, T2], None),
            // In C Db F G Ab C
            (&[S, T2, T, S, T2], None),
            // Insen C Db F G Bb C
            (&[S, T2, T, S3, T], None),
            // Iwato C Db F Gb Bb C
            (&[S, T2, S, T2, T], None),
            // Yo C D F G A C
            (&[T, S3, T, T, S3], None),
        ]);
        let full = initialize_group(&[
            (&[T, T, T, T, T, T], None),
            (&[S, S, S, S, S, S, S, S, S, S, S, S], None),
            (&[Q, Q, Q, Q, Q, Q, Q, Q, Q, Q, Q, Q,
               Q, Q, Q, Q, Q, Q, Q, Q, Q, Q, Q, Q], None),
        ]);
        Self {
            diatonic,
            maqam,
            melakarta,
            japanese,
            full,
        }
    }

    pub fn number_of_scales(&self, group_id: GroupId) -> usize {
        match group_id {
            GroupId::Diatonic => self.diatonic.capacity(),
            GroupId::Maqam => self.maqam.capacity(),
            GroupId::Melakarta => self.melakarta.capacity(),
            GroupId::Japanese => self.japanese.capacity(),
            GroupId::Full => self.full.capacity(),
        }
    }

    pub fn scale(&self, group_id: GroupId, scale_index: usize) -> Result<Scale, ()> {
        if scale_index >= self.number_of_scales(group_id) {
            return Err(());
        }

        match group_id {
            // SAFETY: The index is checked on the entry.
            GroupId::Diatonic => Scale::new(&self.diatonic.get(scale_index).unwrap().ascending),
            GroupId::Maqam => Scale::new(&self.maqam.get(scale_index).unwrap().ascending),
            GroupId::Melakarta => Scale::new(&self.melakarta.get(scale_index).unwrap().ascending),
            GroupId::Japanese => Scale::new(&self.japanese.get(scale_index).unwrap().ascending),
            GroupId::Full => Scale::new(&self.full.get(scale_index).unwrap().ascending),
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

    let mut some_utilize_capacity = false;

    for (ascending_slice, _descending_slice) in scales_slice {
        let scale_len = ascending_slice.len();
        assert!(scale_len <= S, "Given scale is too big or too small");
        some_utilize_capacity |= scale_len == S;
        let chord = LibraryScale::new(ascending_slice).unwrap();
        // SAFETY: The capacity is checked at the beginning of the function.
        group.push(chord).unwrap();
    }

    assert!(
        some_utilize_capacity,
        "None of the scales utilizes LibraryGroup's capacity"
    );

    group
}
