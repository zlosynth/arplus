mod arp_mode;
mod chord;
mod trigger;

mod continuous;
mod discrete;
mod toggle;

use chord::PersistentConfig as ChordPersistentConfig;
use discrete::PersistentConfig as DiscretePersistentConfig;
use toggle::PersistentConfig as TogglePersistentConfig;

// TODO: Do not use these directly. Instead, use them as primitives in types dedicated to what they represent.
pub use continuous::Continuous;
pub use discrete::Discrete;
pub use toggle::Toggle;
pub use trigger::Trigger;

use crate::{chords::Chords, scales::Scales};

pub use self::arp_mode::ArpMode;
pub use self::chord::Chord;

pub struct Parameters {
    // TODO: Switch this to something VOct specific later.
    pub tone: Discrete,
    // TODO: Merge both chord controlling structures together, so they abstract the size alignment
    pub chord: Chord,
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
    pub chord: ChordPersistentConfig,
    pub scale_group: TogglePersistentConfig,
    pub scale: TogglePersistentConfig,
    pub arp_mode: TogglePersistentConfig,
}

impl Parameters {
    pub fn new(config: PersistentConfig, chords: &Chords, scales: &Scales) -> Self {
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
            chord: Chord::new(config.chord, chords),
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
            scale_group: self.scale_group.copy_config(),
            scale: self.scale.copy_config(),
            arp_mode: self.arp_mode.copy_config(),
        }
    }
}
