mod continuous;
mod discrete;
mod toggle;
mod trigger;

use continuous::Continuous;
use discrete::{Discrete, PersistentConfig as DiscretePersistentConfig};
use toggle::{PersistentConfig as TogglePersistentConfig, Toggle};
use trigger::DualTrigger;

use crate::{chords::Chords, scales::Scales};

pub struct Parameters {
    // TODO: Switch this to something VOct specific later.
    pub tone: Discrete,
    pub chord: Discrete,
    pub chord_group: Discrete,
    pub contour: Continuous,
    pub cutoff: Continuous,
    pub resonance: Continuous,
    pub scale_group: Toggle,
    pub scale: Toggle,
    pub arp: Toggle,
    pub trigger: DualTrigger,
}

#[derive(Default, PartialEq, Debug, Clone, Copy, defmt::Format)]
pub struct PersistentConfig {
    pub tone: DiscretePersistentConfig,
    pub chord_group: DiscretePersistentConfig,
    pub chord: DiscretePersistentConfig,
    pub scale_group: TogglePersistentConfig,
    pub scale: TogglePersistentConfig,
    pub arp: TogglePersistentConfig,
}

impl Parameters {
    pub fn new(config: PersistentConfig, chords: &Chords, scales: &Scales) -> Self {
        let chord_group_parameter =
            Discrete::new(config.chord_group, chords.number_of_groups(), 0.1);

        let chord_parameter = {
            let selected_chord_group = chord_group_parameter.selected_value();
            // TODO: No unwrap or safety note
            let number_of_chords_in_the_group =
                chords.number_of_chords(selected_chord_group).unwrap();
            Discrete::new(config.chord, number_of_chords_in_the_group, 0.1)
        };

        let scale_group_parameter = Toggle::new(config.scale_group, scales.number_of_groups());

        let scale_parameter = {
            let selected_scale_group = scale_group_parameter.selected_value();
            // TODO: No unwrap or safety note
            let number_of_scales_in_the_group =
                scales.number_of_scales(selected_scale_group).unwrap();
            Toggle::new(config.scale, number_of_scales_in_the_group)
        };

        let tone_parameter = {
            const OCTAVES: usize = 7;
            // TODO: No unwrap or safety note
            let steps_in_scale = scales
                .number_of_steps_in_group(scale_group_parameter.selected_value())
                .unwrap();
            Discrete::new(config.tone, OCTAVES * steps_in_scale, 0.1)
        };

        Self {
            // TODO: Set proper ranges
            // TODO: Allow configuration of tonic
            tone: tone_parameter,
            chord: chord_parameter,
            chord_group: chord_group_parameter,
            contour: Continuous::new(),
            cutoff: Continuous::new(),
            resonance: Continuous::new(),
            scale_group: scale_group_parameter,
            scale: scale_parameter,
            arp: Toggle::new(config.arp, 6),
            trigger: DualTrigger::new(),
        }
    }

    pub fn copy_config(&self) -> PersistentConfig {
        PersistentConfig {
            tone: self.tone.copy_config(),
            chord: self.chord.copy_config(),
            chord_group: self.chord_group.copy_config(),
            scale_group: self.scale_group.copy_config(),
            scale: self.scale.copy_config(),
            arp: self.arp.copy_config(),
        }
    }
}
