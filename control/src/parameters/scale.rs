use crate::scales::{GroupId, ProjectedScale, QuarterTone, ScaleNote, Scales, Tonic};

use super::primitives::continuous::Continuous;
use super::primitives::discrete::{Discrete, PersistentConfig as DiscretePersistentConfig};
use super::primitives::toggle::{PersistentConfig as TogglePersistentConfig, Toggle};

const OCTAVES: usize = 6;
// Offset from C-1 at 0 V.
const BOTTOM_OCTAVE_OFFSET: usize = 1;

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
        let cv_group = Discrete::new(config.cv_group, Scales::GROUPS, 0.1, 5.0);

        // PANIC: The limit of scale groups is static. The only danger is that
        // it changes between versions without a bump to the save token.
        // However, it is unlikely that that would happen and so this is safe
        // to risk.
        let selected_group = button_group.selected_value().try_into().unwrap();

        let (button_scale, cv_scale) = {
            let scales_in_group = library.number_of_scales(selected_group);
            let button = Toggle::new(config.button_scale, scales_in_group);
            let cv = Discrete::new(config.cv_scale, scales_in_group, 0.1, 5.0);
            (button, cv)
        };

        let pot_note = {
            // XXX: It is a little dirty to initialize it explicitly here. Too bad.
            // PANIC: The scale discrete parameter changes its maximum value based
            // on the selected group. The value is also recovered from the previous
            // save. None of that is particularly safe, so treat any issues
            // gracefully by logging an error and returning the first scale of the
            // group.
            let selected_scale = library
                .scale(selected_group, button_scale.selected_value())
                .unwrap_or_else(|_| {
                    defmt::error!("Invalid scale index for the given group");
                    // PANIC: Scale groups are never empty, this is safe.
                    library.scale(selected_group, 0).unwrap()
                });
            let steps_in_scale = selected_scale.with_tonic(Tonic::C).steps_in_octave() as usize;
            Discrete::new(config.note, OCTAVES * steps_in_scale, 0.1, 1.0)
        };

        let mut s = Self {
            library,
            pot_note,
            cv_note: Continuous::new(),
            pot_octave: Discrete::new(config.octave, 2, 0.1, 1.0),
            cv_note_control: false,
            button_group,
            cv_group,
            cv_controls_group: false,
            button_scale,
            cv_scale,
            cv_controls_scale: false,
            pot_tonic: Discrete::new(config.pot_tonic, 12, 0.1, 1.0),
            cv_tonic: Discrete::new(config.cv_tonic, 12 * 2 + 1, 0.1, 10.0),
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
        tonic_pot: f32,
        group_cv: Option<f32>,
        scale_cv: Option<f32>,
        tonic_cv: Option<f32>,
    ) -> (bool, bool, bool, bool, bool) {
        let old_cv_controls_group = self.cv_controls_group;
        let old_cv_controls_scale = self.cv_controls_scale;

        self.cv_controls_group = group_cv.is_some();
        self.cv_controls_scale = scale_cv.is_some();

        let switched_cv = old_cv_controls_group != self.cv_controls_group
            || old_cv_controls_scale != self.cv_controls_scale;

        let switched_group_cv = old_cv_controls_group != self.cv_controls_group;

        let changed_group = if let Some(group_cv) = group_cv {
            self.cv_group.reconcile(group_cv)
        } else {
            self.button_group.reconcile(group_toggle)
        };

        if changed_group || switched_group_cv {
            let selected_group = if group_cv.is_some() {
                // PANIC: This is called only after the value was just
                // reconciled. Because of that, it will be always in sync
                // and is therefore guaranteed to succeed casting.
                self.cv_group.selected_value().try_into().unwrap()
            } else {
                // PANIC: Ditto.
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

        let changed_tonic = {
            let tonic_cv = tonic_cv.unwrap_or(0.0) + 5.0;
            self.pot_tonic.reconcile(tonic_pot) || self.cv_tonic.reconcile(tonic_cv)
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
        if self.cv_controls_group {
            // PANIC: The limit of scale groups is static. The only danger is that
            // it changes between versions without a bump to the save token.
            // However, it is unlikely that that would happen and so this is safe
            // to risk.
            self.cv_group.selected_value().try_into().unwrap()
        } else {
            // PANIC: Ditto.
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
            // NOTE: Despite the CV input goes from -5 to +5, this function
            // choses to only care for the 5 V above 0. I think it makes it
            // easier to understand how the mapping works, and the only thing
            // that this causes is that either the top-most or bottom-most
            // octave is inaccessible through CV.
            let offset = self.pot_octave.selected_value() as f32;
            let cv = self.cv_note.value().clamp(0.0, 5.0);
            let sum = (cv + offset).clamp(0.0, OCTAVES as f32) + BOTTOM_OCTAVE_OFFSET as f32;
            // PANIC: This may fail if the closest tonic is the last note of
            // the quarter note range. Or if there is a bug somewhere in the
            // note calculation. This has failed once in the past and I don't
            // trust myself here. Hence failing gracefully with a log and the
            // lowest (inaudible) note.
            self.scale_cache()
                .quantize_voct_ascending(sum)
                .unwrap_or_else(|| {
                    defmt::error!("Failed to find the note");
                    ScaleNote::new(QuarterTone::CMinus1, 0)
                })
        } else {
            // PANIC: This may fail if the selected tone is at the end of the
            // range and the request interval would go over the edge. This
            // should never happen with the current input configuration, but
            // in case some glitch (bug) happens, this is failing gracefully.
            self.scale_cache()
                .get_note_by_index_ascending(self.selected_note_index())
                .unwrap_or_else(|| {
                    defmt::error!("Failed to find the note");
                    ScaleNote::new(QuarterTone::CMinus1, 0)
                })
        }
    }

    pub fn selected_tonic(&self) -> Tonic {
        let pot_tonic_index = self.pot_tonic.selected_value() as i32;
        let cv_tonic_offset = self.cv_tonic.selected_value() as i32 - 12;
        let tonic_index =
            (pot_tonic_index + cv_tonic_offset).clamp(0, Tonic::LAST_TONIC as i32) as usize;
        // PANIC: Number of tonics is clamped above. This is safe enough.
        tonic_index.try_into().unwrap()
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
        self.pot_note.selected_value() + BOTTOM_OCTAVE_OFFSET * self.selected_scale_size()
    }

    fn scale_cache(&self) -> &ProjectedScale {
        // PANIC: The cache is initialized in `new` and never taken away.
        // This is safe.
        self.scale_cache.as_ref().unwrap()
    }

    fn update_scale_cache(&mut self) {
        self.scale_cache = Some(
            self.library
                .scale(self.selected_group_id(), self.selected_scale_index())
                // PANIC: Gracefuly ignore inconsistent scale index. This
                // should work fine, but is error prone - and it already
                // suffered from a bug in the past.
                .unwrap_or_else(|_| {
                    defmt::error!(
                        "Unable to find scale for the given group group id {:?} scale index {:?}",
                        self.selected_group_id(),
                        self.selected_scale_index()
                    );
                    // PANIC: Scale groups are never empty, this is safe.
                    self.library.scale(self.selected_group_id(), 0).unwrap()
                })
                .with_tonic(self.selected_tonic()),
        );
    }

    pub fn selected_octave_index(&self) -> usize {
        self.pot_octave.selected_value()
    }
}
