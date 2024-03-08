use crate::chords::{Chords, GroupId};

use super::{discrete::PersistentConfig as DiscretePersistentConfig, Discrete};

pub struct Chord {
    group: Discrete,
    chord: Discrete,
}

#[derive(Default, PartialEq, Debug, Clone, Copy, defmt::Format)]
pub struct PersistentConfig {
    group: DiscretePersistentConfig,
    chord: DiscretePersistentConfig,
}

// TODO: Should it own chords and handle their fetching as well?
impl Chord {
    pub fn new(config: PersistentConfig, chords: &Chords) -> Self {
        let group = Discrete::new(config.group, Chords::GROUPS, 0.1);

        // TODO: Share the adjustement code with reconcile group
        let chord = {
            // TODO: Safety
            let selected_group = group.selected_value().try_into().unwrap();
            let number_of_chords_in_the_group = chords.number_of_chords(selected_group);
            Discrete::new(config.chord, number_of_chords_in_the_group, 0.1)
        };

        Self { group, chord }
    }

    pub fn reconcile_group_and_chord(
        &mut self,
        group_pot: f32,
        group_cv: Option<f32>,
        chord_pot: f32,
        chord_cv: Option<f32>,
        chords: &Chords,
    ) -> (bool, bool) {
        let changed_group = self.group.reconcile(linear_sum(group_pot, group_cv));

        if changed_group {
            let selected_group = self.group.selected_value().try_into().unwrap();
            let number_of_chords_in_the_group = chords.number_of_chords(selected_group);
            self.chord.set_output_values(number_of_chords_in_the_group);
        }

        let changed_chord = self.chord.reconcile(linear_sum(chord_pot, chord_cv));

        (changed_chord, changed_chord)
    }

    pub fn selected_group_id(&self) -> GroupId {
        // SAFETY: Parameter values used to get group id are statically limited
        // by the number of groups.
        self.group.selected_value().try_into().unwrap()
    }

    pub fn selected_chord_index(&self) -> usize {
        self.chord.selected_value()
    }

    pub fn copy_config(&self) -> PersistentConfig {
        PersistentConfig {
            group: self.group.copy_config(),
            chord: self.chord.copy_config(),
        }
    }
}

// TODO: Move it to a lib?
fn linear_sum(pot: f32, cv: Option<f32>) -> f32 {
    let offset_cv = cv.unwrap_or(0.0) / 5.0;
    let sum = pot + offset_cv;
    let clamped = sum.clamp(0.0, 1.0);
    clamped
}
