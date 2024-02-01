use heapless::Vec;

use super::quarter_tones::QuarterTone;
use super::scale_note::ScaleNote;
use super::tonic::Tonic;

pub type Step = u8;

pub const Q: Step = 1;
pub const S: Step = 2;
pub const T: Step = 4;

pub struct Scale {
    tonic: Tonic,
    ascending: Vec<Step, 7>,
}

impl Scale {
    pub fn new(tonic: Tonic, ascending: &[Step]) -> Result<Self, ()> {
        Ok(Self {
            tonic,
            ascending: Vec::from_slice(ascending)?,
        })
    }

    pub fn quantize_voct_ascending(&self, voct: f32) -> Option<ScaleNote> {
        // XXX: This is making the method simpler by sacrificing a part of the
        // lowest octave.
        let lowest_tonic = self.lowest_tonic();
        if voct < lowest_tonic.voct() {
            return None;
        }

        let closest_tonic = self.find_closest_tonic(voct)?;
        let (note_below, note_above) =
            self.find_surrounding_notes_ascending(voct, closest_tonic)?;
        let center_voct = (note_below.tone.voct() + note_above.tone.voct()) / 2.0;
        if voct < center_voct {
            Some(note_below)
        } else {
            Some(note_above)
        }
    }

    pub fn get_note_in_interval_ascending(
        &self,
        source: ScaleNote,
        interval: i16,
    ) -> Option<ScaleNote> {
        let steps_in_octave = self.ascending.len();
        let octaves = interval / steps_in_octave as i16;
        let remaining_steps = interval % steps_in_octave as i16;

        let remaining_distance = if remaining_steps > 0 {
            let mut remaining_distance = 0;
            for i in 0..remaining_steps {
                remaining_distance += self.ascending
                    [((source.index + i as u8) % steps_in_octave as u8) as usize]
                    as i16;
            }
            remaining_distance
        } else {
            let mut remaining_distance = 0;
            for i in 1..=remaining_steps.abs() {
                remaining_distance -= self.ascending
                    [(source.index as i16 - i as i16).rem_euclid(steps_in_octave as i16) as usize]
                    as i16;
            }
            remaining_distance
        };

        let total_distance = octaves as i16 * 24 + remaining_distance;
        let tone = QuarterTone::try_from_u8((source.tone.index() as i16 + total_distance) as u8)?;
        let index = (source.index as i16 + remaining_steps as i16)
            .rem_euclid(self.ascending.len() as i16) as u8;

        Some(ScaleNote::new(tone, index))
    }

    fn lowest_tonic(&self) -> QuarterTone {
        match self.tonic {
            Tonic::C => QuarterTone::CMinus1,
            Tonic::CSharp => QuarterTone::CSharpMinus1,
            Tonic::D => QuarterTone::DMinus1,
            Tonic::DSharp => QuarterTone::DSharpMinus1,
            Tonic::E => QuarterTone::EMinus1,
            Tonic::F => QuarterTone::FMinus1,
            Tonic::FSharp => QuarterTone::FSharpMinus1,
            Tonic::G => QuarterTone::GMinus1,
            Tonic::GSharp => QuarterTone::GSharpMinus1,
            Tonic::A => QuarterTone::AMinus1,
            Tonic::ASharp => QuarterTone::ASharpMinus1,
            Tonic::B => QuarterTone::BMinus1,
        }
    }

    fn find_closest_tonic(&self, voct: f32) -> Option<QuarterTone> {
        let lowest_tonic = self.lowest_tonic();
        let distance_in_full_octaves = (voct - lowest_tonic.voct()) as u8;
        QuarterTone::try_from_u8(lowest_tonic.index() + 24 * distance_in_full_octaves)
    }

    fn find_surrounding_notes_ascending(
        &self,
        voct: f32,
        closest_tonic: QuarterTone,
    ) -> Option<(ScaleNote, ScaleNote)> {
        let mut below = ScaleNote::new(closest_tonic, 0);
        let mut above = None;

        let mut distance = 0;
        for (i, step) in self.ascending.iter().enumerate() {
            distance += step;
            let tone = QuarterTone::try_from_u8(closest_tonic.index() + distance)?;
            let index = (i as u8 + 1) % self.ascending.len() as u8;
            if tone.voct() > voct {
                above = Some(ScaleNote::new(tone, index));
                break;
            }
            below = ScaleNote::new(tone, index);
        }

        Some((below, above?))
    }
}

#[cfg(test)]
mod tests {
    use crate::scales::quarter_tones::QuarterTone;
    use crate::scales::scale_note::ScaleNote;

    use super::*;

    const IONIAN: [Step; 7] = [T, T, S, T, T, T, S];

    #[test]
    fn initialize_a_scale() {
        let _scale = Scale::new(Tonic::C, &IONIAN).unwrap();
    }

    #[test]
    fn quantize_voct_to_scale_note_ascending_returns_note() {
        let scale = Scale::new(Tonic::C, &IONIAN).unwrap();
        let checks = [
            (1.0, ScaleNote::new(QuarterTone::C0, 0)),
            (1.0 + 0.9 / 12.0, ScaleNote::new(QuarterTone::C0, 0)),
            (1.0 + 1.1 / 12.0, ScaleNote::new(QuarterTone::D0, 1)),
            (2.0, ScaleNote::new(QuarterTone::C1, 0)),
        ];
        for (voct, expected_note) in checks {
            assert_eq!(scale.quantize_voct_ascending(voct), Some(expected_note));
        }
    }

    #[test]
    fn quantize_voct_to_scale_note_ascending_below_the_lowest_tonic_returns_none() {
        let scale = Scale::new(Tonic::D, &IONIAN).unwrap();
        assert!(scale.quantize_voct_ascending(0.0).is_none());
    }

    #[test]
    fn find_closest_tonic_below_voct() {
        let scale = Scale::new(Tonic::D, &IONIAN).unwrap();
        let checks = [
            (1.0, QuarterTone::DMinus1),
            (1.0 + 2.1 / 12.0, QuarterTone::D0),
            (2.0 + 2.1 / 12.0, QuarterTone::D1),
            (3.0 + 1.0 / 12.0, QuarterTone::D1),
            (3.0 + 3.0 / 12.0, QuarterTone::D2),
        ];
        for (voct, expected_note) in checks {
            assert_eq!(scale.find_closest_tonic(voct), Some(expected_note));
        }
    }

    #[test]
    fn find_surrounding_notes_ascending() {
        let scale = Scale::new(Tonic::D, &IONIAN).unwrap();
        let checks = [
            (
                1.0 + 1.9 / 12.0,
                ScaleNote::new(QuarterTone::CSharp0, 6),
                ScaleNote::new(QuarterTone::D0, 0),
            ),
            (
                1.0 + 2.1 / 12.0,
                ScaleNote::new(QuarterTone::D0, 0),
                ScaleNote::new(QuarterTone::E0, 1),
            ),
            (
                1.0 + 5.9 / 12.0,
                ScaleNote::new(QuarterTone::E0, 1),
                ScaleNote::new(QuarterTone::FSharp0, 2),
            ),
            (
                1.0 + 6.1 / 12.0,
                ScaleNote::new(QuarterTone::FSharp0, 2),
                ScaleNote::new(QuarterTone::G0, 3),
            ),
        ];
        for (voct, expected_below, expected_above) in checks {
            let closest_tonic = scale.find_closest_tonic(voct).unwrap();
            let (below, above) = scale
                .find_surrounding_notes_ascending(voct, closest_tonic)
                .unwrap();
            assert_eq!((below, above), (expected_below, expected_above));
        }
    }

    #[test]
    fn get_note_in_interval_ascending() {
        let scale = Scale::new(Tonic::D, &IONIAN).unwrap();
        let checks = [
            (
                ScaleNote::new(QuarterTone::D1, 0),
                0,
                ScaleNote::new(QuarterTone::D1, 0),
            ),
            (
                ScaleNote::new(QuarterTone::D1, 0),
                1,
                ScaleNote::new(QuarterTone::E1, 1),
            ),
            (
                ScaleNote::new(QuarterTone::D1, 0),
                7,
                ScaleNote::new(QuarterTone::D2, 0),
            ),
            (
                ScaleNote::new(QuarterTone::D1, 0),
                -1,
                ScaleNote::new(QuarterTone::CSharp1, 6),
            ),
            (
                ScaleNote::new(QuarterTone::D1, 0),
                -7,
                ScaleNote::new(QuarterTone::D0, 0),
            ),
            (
                ScaleNote::new(QuarterTone::D1, 0),
                -8,
                ScaleNote::new(QuarterTone::CSharp0, 6),
            ),
        ];
        for (source, interval, expected_note) in checks {
            assert_eq!(
                scale.get_note_in_interval_ascending(source, interval),
                Some(expected_note),
            );
        }
    }

    // TODO: Add fuzzy test making sure that no matter the operation, the
    // result is always in scale.
}
