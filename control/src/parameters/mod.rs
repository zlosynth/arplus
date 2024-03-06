mod arp_mode;
mod continuous;
mod discrete;
mod toggle;
mod trigger;

use discrete::PersistentConfig as DiscretePersistentConfig;
use toggle::PersistentConfig as TogglePersistentConfig;

// TODO: Do not use these directly. Instead, use them as primitives in types dedicated to what they represent.
pub use continuous::Continuous;
pub use discrete::Discrete;
pub use toggle::Toggle;
pub use trigger::Trigger;

use crate::{chords::Chords, scales::Scales};

pub use self::arp_mode::ArpMode;

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
    pub arp_mode: ArpMode,
    pub trigger: Trigger,
}

#[derive(Default, PartialEq, Debug, Clone, Copy, defmt::Format)]
pub struct PersistentConfig {
    pub tone: DiscretePersistentConfig,
    pub chord_group: DiscretePersistentConfig,
    pub chord: DiscretePersistentConfig,
    pub scale_group: TogglePersistentConfig,
    pub scale: TogglePersistentConfig,
    pub arp_mode: TogglePersistentConfig,
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
            arp_mode: ArpMode::new(config.arp_mode),
            trigger: Trigger::new(),
        }
    }

    pub fn copy_config(&self) -> PersistentConfig {
        PersistentConfig {
            tone: self.tone.copy_config(),
            chord: self.chord.copy_config(),
            chord_group: self.chord_group.copy_config(),
            scale_group: self.scale_group.copy_config(),
            scale: self.scale.copy_config(),
            arp_mode: self.arp_mode.copy_config(),
        }
    }
}
