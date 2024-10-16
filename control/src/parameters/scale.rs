use crate::scales::{GroupId, ProjectedScale, ScaleNote, Scales, Tonic};

use super::primitives::continuous::Continuous;
use super::primitives::discrete::{Discrete, PersistentConfig as DiscretePersistentConfig};
use super::primitives::toggle::{PersistentConfig as TogglePersistentConfig, Toggle};

const OCTAVES: usize = 7;

pub struct Scale {
    library: Scales,
    pot_note: Discrete,
    cv_note: Continuous,
    pot_octave: Discrete,
    cv_note_control: bool,
    button_group: Toggle,
    cv_group: Discrete,
    cv_controls_group: bool,
    button_scale: Toggle,
    cv_scale: Discrete,
    cv_controls_scale: bool,
    pot_tonic: Discrete,
    cv_tonic: Discrete,
    cv_controls_tonic: bool,
    scale_cache: Option<ProjectedScale>,
}

#[derive(Default, PartialEq, Debug, Clone, Copy, defmt::Format)]
pub struct PersistentConfig {
    note: DiscretePersistentConfig,
    octave: DiscretePersistentConfig,
    pot_tonic: DiscretePersistentConfig,
    cv_tonic: DiscretePersistentConfig,
    button_group: TogglePersistentConfig,
    cv_group: DiscretePersistentConfig,
    button_scale: TogglePersistentConfig,
    cv_scale: DiscretePersistentConfig,
}

impl Scale {
    pub fn new(config: PersistentConfig, library: Scales) -> Self {
        let button_group = Toggle::new(config.button_group, Scales::GROUPS);
        let cv_group = Discrete::new(config.cv_group, Scales::GROUPS, 0.1);

        // SAFETY: The group toggle is limited by the number of groups.
        let selected_group = button_group.selected_value().try_into().unwrap();

        let (button_scale, cv_scale) = {
            let scales_in_group = library.number_of_scales(selected_group);
            let button = Toggle::new(config.button_scale, scales_in_group);
            let cv = Discrete::new(config.cv_scale, scales_in_group, 0.1);
            (button, cv)
        };

        let pot_note = {
            // XXX: It is a little dirty to initialize it explicitly here. Too bad.
            // SAFETY: Maximum scale index is already limited by selected group.
            let selected_scale = library
                .scale(selected_group, button_scale.selected_value())
                .unwrap();
            let steps_in_scale = selected_scale.with_tonic(Tonic::C).steps_in_octave() as usize;
            Discrete::new(config.note, OCTAVES * steps_in_scale, 0.1)
        };

        let mut s = Self {
            library,
            pot_note,
            cv_note: Continuous::new(),
            pot_octave: Discrete::new(config.octave, 4, 0.1),
            cv_note_control: false,
            button_group,
            cv_group,
            cv_controls_group: false,
            button_scale,
            cv_scale,
            cv_controls_scale: false,
            pot_tonic: Discrete::new(config.pot_tonic, 12, 0.1),
            cv_tonic: Discrete::new(config.cv_tonic, 12, 0.1),
            cv_controls_tonic: false,
            scale_cache: None,
        };
        s.update_scale_cache();
        s
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconcile_note_tonic_group_and_scale(
        &mut self,
        note_pot: f32,
        note_cv: Option<f32>,
        group_toggle: bool,
        scale_toggle: bool,
        trigger_held: bool,
        group_cv: Option<f32>,
        scale_cv: Option<f32>,
        tonic_cv: Option<f32>,
    ) -> (bool, bool, bool, bool, bool) {
        // TODO: Handle configuration of tonic with BUTTON + TONE

        let old_cv_controls_group = self.cv_controls_group;
        let old_cv_controls_scale = self.cv_controls_scale;
        // TODO: This would go away, I think
        let old_cv_controls_tonic = self.cv_controls_tonic;

        self.cv_controls_group = group_cv.is_some();
        self.cv_controls_scale = scale_cv.is_some();
        self.cv_controls_tonic = tonic_cv.is_some();

        let switched_cv = old_cv_controls_group != self.cv_controls_group
            || old_cv_controls_scale != self.cv_controls_scale
            || old_cv_controls_tonic != self.cv_controls_tonic;

        let changed_group = if let Some(group_cv) = group_cv {
            self.cv_group.reconcile(group_cv)
        } else {
            self.button_group.reconcile(group_toggle)
        };

        if changed_group {
            let selected_group = if group_cv.is_some() {
                self.cv_group.selected_value().try_into().unwrap()
            } else {
                self.button_group.selected_value().try_into().unwrap()
            };
            let scales_in_group = self.library.number_of_scales(selected_group);
            self.button_scale.set_output_values(scales_in_group);
            self.cv_scale.set_output_values(scales_in_group);
        }

        let changed_scale = if let Some(scale_cv) = scale_cv {
            self.cv_scale.reconcile(scale_cv)
        } else {
            self.button_scale.reconcile(scale_toggle)
        };

        let changed_tonic = if let Some(tonic_cv) = tonic_cv {
            self.cv_tonic.reconcile(tonic_cv)
        } else if trigger_held {
            // TODO: It is already prepared for TONIC contrl here. Only now I have to combine it with CV
            self.pot_tonic.reconcile(note_pot)
        } else {
            false
        };

        if changed_group || changed_scale || changed_tonic || switched_cv {
            self.update_scale_cache();

            let steps_in_scale = self.scale_cache().steps_in_octave() as usize;
            self.pot_note.set_output_values(OCTAVES * steps_in_scale);
        }

        let (changed_note, changed_octave) = if let Some(note_cv) = note_cv {
            self.cv_note_control = true;
            self.cv_note.reconcile(note_cv);
            (false, self.pot_octave.reconcile(note_pot))
        } else {
            self.cv_note_control = false;
            (self.pot_note.reconcile(note_pot), false)
        };

        (
            changed_note,
            changed_octave,
            changed_group,
            changed_scale,
            changed_tonic,
        )
    }

    pub fn selected_group_id(&self) -> GroupId {
        // SAFETY: Parameter values used to get group id are statically limited
        // by the number of groups.
        if self.cv_controls_group {
            self.cv_group.selected_value().try_into().unwrap()
        } else {
            self.button_group.selected_value().try_into().unwrap()
        }
    }

    pub fn selected_scale_index(&self) -> usize {
        if self.cv_controls_scale {
            self.cv_scale.selected_value()
        } else {
            self.button_scale.selected_value()
        }
    }

    pub fn selected_scale(&self) -> ProjectedScale {
        self.scale_cache().clone()
    }

    pub fn selected_scale_size(&self) -> usize {
        self.scale_cache().steps_in_octave() as usize
    }

    pub fn selected_note(&self) -> ScaleNote {
        if self.cv_note_control {
            let offset = self.pot_octave.selected_value() as f32 - 2.0;
            let cv = self.cv_note.value();
            let sum = (cv + offset).clamp(0.0, OCTAVES as f32);
            // SAFETY: Limited by the same octave range as when using note Pot.
            // TODO FIXME: This sometimes panics.
            self.scale_cache().quantize_voct_ascending(sum).unwrap()
        } else {
            // SAFETY: Range of indices is limited in `new` and `reconcile`.
            self.scale_cache()
                .get_note_by_index_ascending(self.selected_note_index())
                .unwrap()
        }
    }

    pub fn selected_tonic(&self) -> Tonic {
        // SAFETY: The discrete parameter is limited by the maximum index of tonic.
        if self.cv_controls_tonic {
            self.cv_tonic.selected_value().try_into().unwrap()
        } else {
            self.pot_tonic.selected_value().try_into().unwrap()
        }
    }

    pub fn copy_config(&self) -> PersistentConfig {
        PersistentConfig {
            note: self.pot_note.copy_config(),
            octave: self.pot_octave.copy_config(),
            pot_tonic: self.pot_tonic.copy_config(),
            button_group: self.button_group.copy_config(),
            button_scale: self.button_scale.copy_config(),
            cv_tonic: self.cv_tonic.copy_config(),
            cv_group: self.cv_group.copy_config(),
            cv_scale: self.cv_scale.copy_config(),
        }
    }

    fn selected_note_index(&self) -> usize {
        self.pot_note.selected_value()
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
                .with_tonic(self.selected_tonic()),
        );
    }

    pub fn selected_octave_index(&self) -> usize {
        self.pot_octave.selected_value()
    }
}
