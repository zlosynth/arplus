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
            // SAFETY: The group attribute is limited by the number of groups.
            let selected_group = group.selected_value().try_into().unwrap();
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

    pub fn reconcile_group_chord_and_scale_size(
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
            let selected_group = self.group.selected_value().try_into().unwrap();
            let chords_in_group = self.library.number_of_chords(selected_group);
            self.chord.set_output_values(chords_in_group);
        }

        let changed_chord = self.chord.reconcile(math::linear_sum(chord_pot, chord_cv));

        self.scale_size = scale_size;

        (changed_group, changed_chord)
    }

    pub fn selected_group_id(&self) -> GroupId {
        // SAFETY: Parameter values used to get group id are statically limited
        // by the number of groups.
        self.group.selected_value().try_into().unwrap()
    }

    pub fn selected_chord_index(&self) -> usize {
        // self.chord.selected_value()
        let c = self.chord.selected_value();
        defmt::info!("{:?}", c + 1);
        c
    }

    pub fn selected_chord(&self) -> crate::chords::Chord {
        // SAFETY: Parameter values used to get group and chord index
        // are always limited based on the selected chord group.
        self.library
            .chord(self.selected_group_id(), self.selected_chord_index())
            .unwrap()
    }

    pub fn selected_group_size(&self) -> usize {
        self.library.group_size(self.selected_group_id())
    }

    pub fn copy_config(&self) -> PersistentConfig {
        PersistentConfig {
            group: self.group.copy_config(),
            chord: self.chord.copy_config(),
        }
    }
}
