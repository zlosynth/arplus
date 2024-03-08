use crate::scales::{GroupId, Scales};

use super::{Discrete, DiscretePersistentConfig, Toggle, TogglePersistentConfig};

pub struct Scale {
    note: Discrete,
    group: Toggle,
    scale: Toggle,
    // TODO: Tonic
    // TODO: CV mapping
}

#[derive(Default, PartialEq, Debug, Clone, Copy, defmt::Format)]
pub struct PersistentConfig {
    note: DiscretePersistentConfig,
    group: TogglePersistentConfig,
    scale: TogglePersistentConfig,
}

// TODO: Should it own scales and handle note fetching as well?
impl Scale {
    pub fn new(config: PersistentConfig, scales: &Scales) -> Self {
        let group = Toggle::new(config.group, Scales::GROUPS);

        // TODO: Share the adjustement code with reconcile group
        let scale = {
            // TODO: Safety
            let selected_group = group.selected_value().try_into().unwrap();
            let number_of_scales_in_the_group = scales.number_of_scales(selected_group);
            Toggle::new(config.scale, number_of_scales_in_the_group)
        };

        let note = {
            const OCTAVES: usize = 7;
            // TODO: Safety
            let selected_group = group.selected_value().try_into().unwrap();
            let steps_in_scale = scales.number_of_steps_in_group(selected_group);
            Discrete::new(config.note, OCTAVES * steps_in_scale, 0.1)
        };

        Self { note, group, scale }
    }

    pub fn reconcile_note_group_and_scale(
        &mut self,
        note_pot: f32,
        note_cv: Option<f32>,
        group_toggle: bool,
        scale_toggle: bool,
        scales: &Scales,
    ) -> (bool, bool, bool) {
        let changed_group = self.group.reconcile(group_toggle);

        if changed_group {
            let selected_group = self.group.selected_value().try_into().unwrap();

            let number_of_chords_in_the_group = scales.number_of_scales(selected_group);
            self.scale.set_output_values(number_of_chords_in_the_group);

            const OCTAVES: usize = 7;
            let number_of_steps_in_the_group = scales.number_of_steps_in_group(selected_group);
            self.note.set_output_values(number_of_steps_in_the_group);
        }

        let changed_chord = self.scale.reconcile(scale_toggle);

        let changed_note = self.note.reconcile(linear_sum(note_pot, note_cv));

        (changed_note, changed_chord, changed_chord)
    }

    pub fn selected_group_id(&self) -> GroupId {
        // SAFETY: Parameter values used to get group id are statically limited
        // by the number of groups.
        self.group.selected_value().try_into().unwrap()
    }

    pub fn selected_scale_index(&self) -> usize {
        self.scale.selected_value()
    }

    pub fn selected_note_index(&self) -> usize {
        self.note.selected_value()
    }

    pub fn copy_config(&self) -> PersistentConfig {
        PersistentConfig {
            note: self.note.copy_config(),
            group: self.group.copy_config(),
            scale: self.scale.copy_config(),
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
