// TODO: Review. Keep Projected scale here. Review callers of methods.

use crate::scales::{GroupId, ProjectedScale, ScaleNote, Scales, Tonic};

use super::primitives::discrete::{Discrete, PersistentConfig as DiscretePersistentConfig};
use super::primitives::math;
use super::primitives::toggle::{PersistentConfig as TogglePersistentConfig, Toggle};

const OCTAVES: usize = 7;

pub struct Scale {
    library: Scales,
    note: Discrete,
    group: Toggle,
    scale: Toggle,
    tonic: Tonic,
    scale_cache: Option<ProjectedScale>,
}

#[derive(Default, PartialEq, Debug, Clone, Copy, defmt::Format)]
pub struct PersistentConfig {
    note: DiscretePersistentConfig,
    group: TogglePersistentConfig,
    scale: TogglePersistentConfig,
}

impl Scale {
    pub fn new(config: PersistentConfig, library: Scales) -> Self {
        let group = Toggle::new(config.group, Scales::GROUPS);

        // SAFETY: The group toggle is limited by the number of groups.
        let selected_group = group.selected_value().try_into().unwrap();

        let scale = {
            let scales_in_group = library.number_of_scales(selected_group);
            Toggle::new(config.scale, scales_in_group)
        };

        let note = {
            // XXX: It is a little dirty to initialize it explicitly here. Too bad.
            // SAFETY: Maximum scale index is already limited by selected group.
            let selected_scale = library
                .scale(selected_group, scale.selected_value())
                .unwrap();
            let steps_in_scale = selected_scale.with_tonic(Tonic::C).steps_in_octave() as usize;
            Discrete::new(config.note, OCTAVES * steps_in_scale, 0.1)
        };

        let mut s = Self {
            library,
            note,
            group,
            scale,
            tonic: Tonic::C,
            scale_cache: None,
        };
        s.update_scale_cache();
        s
    }

    pub fn reconcile_note_group_and_scale(
        &mut self,
        note_pot: f32,
        note_cv: Option<f32>,
        group_toggle: bool,
        scale_toggle: bool,
    ) -> (bool, bool, bool) {
        let changed_group = self.group.reconcile(group_toggle);

        if changed_group {
            let selected_group = self.group.selected_value().try_into().unwrap();

            let scales_in_group = self.library.number_of_scales(selected_group);
            self.scale.set_output_values(scales_in_group);
        }

        let changed_scale = self.scale.reconcile(scale_toggle);

        if changed_group || changed_scale {
            self.update_scale_cache();

            let steps_in_scale = self.scale_cache().steps_in_octave() as usize;
            self.note.set_output_values(OCTAVES * steps_in_scale);
        }

        let changed_note = self.note.reconcile(math::linear_sum(note_pot, note_cv));

        (changed_note, changed_scale, changed_scale)
    }

    pub fn selected_group_id(&self) -> GroupId {
        // SAFETY: Parameter values used to get group id are statically limited
        // by the number of groups.
        self.group.selected_value().try_into().unwrap()
    }

    pub fn selected_scale_index(&self) -> usize {
        self.scale.selected_value()
    }

    pub fn selected_scale(&self) -> ProjectedScale {
        self.scale_cache().clone()
    }

    pub fn selected_scale_size(&self) -> usize {
        self.scale_cache().steps_in_octave() as usize
    }

    pub fn selected_note(&self) -> ScaleNote {
        // SAFETY: Range of indices is limited in `new` and `reconcile`.
        self.scale_cache()
            .get_note_by_index_ascending(self.selected_note_index())
            .unwrap()
    }

    pub fn selected_tonic(&self) -> Tonic {
        self.tonic
    }

    pub fn copy_config(&self) -> PersistentConfig {
        PersistentConfig {
            note: self.note.copy_config(),
            group: self.group.copy_config(),
            scale: self.scale.copy_config(),
        }
    }

    fn selected_note_index(&self) -> usize {
        self.note.selected_value()
    }

    fn scale_cache(&self) -> &ProjectedScale {
        // SAFETY: The cache is initialized in `new` and never taken away.
        self.scale_cache.as_ref().unwrap()
    }

    fn update_scale_cache(&mut self) {
        self.scale_cache = Some(
            self.library
                .scale(self.selected_group_id(), self.selected_scale_index())
                .unwrap()
                .with_tonic(self.tonic),
        );
    }
}
