use crate::chords::{Chords, GroupId};

use super::primitives::discrete::{Discrete, PersistentConfig as DiscretePersistentConfig};
use super::primitives::math;

pub struct Chord {
    library: Chords,
    group: Discrete,
    chord: Discrete,
    scale_size: usize,
}

#[derive(Default, PartialEq, Debug, Clone, Copy, defmt::Format)]
pub struct PersistentConfig {
    group: DiscretePersistentConfig,
    chord: DiscretePersistentConfig,
}

impl Chord {
    // NOTE: Passing scale size is awkward, but dynamic sizing of chords is
    // necessary to allow for an arpeggio with all notes of a scale.
    pub fn new(config: PersistentConfig, library: Chords, scale_size: usize) -> Self {
        let group = Discrete::new(config.group, Chords::GROUPS, 0.1, 1.0);

        let chord = {
            // PANIC: While `try_into` method is checking against the same
            // constant as used for the discrete attribute maximum, it can
            // still happen that a wrong value was recovered from a safe,
            // e.g. from a previous version of a module if token was not
            // updated. To cover that case, fail gracefully by logging an
            // error and returning the default group.
            let selected_group = group.selected_value().try_into().unwrap_or_else(|_| {
                defmt::error!("The recovered chord group is out of range");
                GroupId::Size1
            });
            let chords_in_group = library.number_of_chords(selected_group);
            Discrete::new(config.chord, chords_in_group, 0.1, 1.0)
        };

        Self {
            library,
            group,
            chord,
            scale_size,
        }
    }

    pub fn reconcile_size_chord_and_scale_size(
        &mut self,
        group_pot: f32,
        group_cv: Option<f32>,
        chord_pot: f32,
        chord_cv: Option<f32>,
        scale_size: usize,
    ) -> (bool, bool) {
        let changed_group = self.group.reconcile(math::linear_sum(group_pot, group_cv));

        let changed_scale_size = self.scale_size != scale_size;

        if changed_group || changed_scale_size {
            let selected_group = self.selected_group_id();
            let chords_in_group = self.library.number_of_chords(selected_group);
            self.chord.set_output_values(chords_in_group);
        }

        let changed_chord = self.chord.reconcile(math::linear_sum(chord_pot, chord_cv));

        self.scale_size = scale_size;

        (changed_group, changed_chord)
    }

    pub fn selected_group_id(&self) -> GroupId {
        // PANIC: While `try_into` method is checking against the same
        // constant as used for the discrete attribute maximum, it can
        // still happen that a wrong value was recovered from a safe,
        // e.g. from a previous version of a module if token was not
        // updated. To cover that case, fail gracefully by logging an
        // error and returning the default group.
        self.group.selected_value().try_into().unwrap_or_else(|_| {
            defmt::error!("The recovered chord group is out of range");
            GroupId::Size1
        })
    }

    pub fn selected_chord_index(&self) -> usize {
        self.chord.selected_value()
    }

    pub fn selected_chord(&self) -> crate::chords::Chord {
        // PANIC: The chord index is variable, either loaded from the previous
        // save or dialed in using the discrete parameter, with the maximum
        // set based on the selected group. There is a lot that goes on and
        // it is not all covered by type safety. Therefore, fail gracefully
        // by logging an error and returning the first chord of the group.
        self.library
            .chord(self.selected_group_id(), self.selected_chord_index())
            .unwrap_or_else(|_| {
                defmt::error!("Failed to find selected chord");
                // PANIC: Group is covered by type safety and there is always
                // at least one chord in any given group. This is safe.
                self.library.chord(self.selected_group_id(), 0).unwrap()
            })
    }

    pub fn selected_size(&self) -> usize {
        self.library.group_size(self.selected_group_id())
    }

    pub fn copy_config(&self) -> PersistentConfig {
        PersistentConfig {
            group: self.group.copy_config(),
            chord: self.chord.copy_config(),
        }
    }
}
