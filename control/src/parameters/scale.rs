use crate::scales::{GroupId, Scales};

use super::{Toggle, TogglePersistentConfig};

pub struct Scale {
    group: Toggle,
    scale: Toggle,
    // TODO: Tonic
    // TODO: CV mapping
}

#[derive(Default, PartialEq, Debug, Clone, Copy, defmt::Format)]
pub struct PersistentConfig {
    group: TogglePersistentConfig,
    scale: TogglePersistentConfig,
}

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

        Self { group, scale }
    }

    pub fn reconcile_group_and_scale(
        &mut self,
        group_toggle: bool,
        scale_toggle: bool,
        scales: &Scales,
    ) -> (bool, bool) {
        let changed_group = self.group.reconcile(group_toggle);

        if changed_group {
            let selected_group = self.group.selected_value().try_into().unwrap();
            let number_of_chords_in_the_group = scales.number_of_scales(selected_group);
            self.scale.set_output_values(number_of_chords_in_the_group);
        }

        let changed_chord = self.scale.reconcile(scale_toggle);

        (changed_chord, changed_chord)
    }

    pub fn selected_group_id(&self) -> GroupId {
        // SAFETY: Parameter values used to get group id are statically limited
        // by the number of groups.
        self.group.selected_value().try_into().unwrap()
    }

    pub fn selected_scale_index(&self) -> usize {
        self.scale.selected_value()
    }

    pub fn copy_config(&self) -> PersistentConfig {
        PersistentConfig {
            group: self.group.copy_config(),
            scale: self.scale.copy_config(),
        }
    }
}
