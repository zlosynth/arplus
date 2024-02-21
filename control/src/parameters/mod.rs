mod continuous;
mod discrete;
mod toggle;
mod trigger;

use continuous::Continuous;
use discrete::{Discrete, PersistentConfig as DiscretePersistentConfig};
use toggle::{PersistentConfig as TogglePersistentConfig, Toggle};
use trigger::DualTrigger;

use crate::chords::Chords;

pub struct Parameters {
    // TODO: Switch this to something VOct specific later.
    pub tone: Discrete,
    pub chord: Discrete,
    pub size: Discrete,
    pub contour: Continuous,
    pub cutoff: Continuous,
    pub resonance: Continuous,
    pub scale: Toggle,
    pub mode: Toggle,
    pub arp: Toggle,
    pub trigger: DualTrigger,
}

#[derive(Default, PartialEq, Debug, Clone, Copy, defmt::Format)]
pub struct PersistentConfig {
    pub tone: DiscretePersistentConfig,
    pub chord: DiscretePersistentConfig,
    pub size: DiscretePersistentConfig,
    pub scale: TogglePersistentConfig,
    pub mode: TogglePersistentConfig,
    pub arp: TogglePersistentConfig,
}

impl Parameters {
    pub fn new(config: PersistentConfig, chords: &Chords) -> Self {
        // TODO: Consider selected chord group too. Recovered from save Discrete attribute.
        // TODO: No unwrap or safety note
        let number_of_groups = chords.number_of_groups();
        let number_of_chords_in_the_group = chords.number_of_chords(0).unwrap();
        Self {
            // TODO: Set proper ranges
            // TODO: Allow configuration of tonic
            tone: Discrete::new(config.tone, 7 * 6, 0.1),
            chord: Discrete::new(config.chord, number_of_chords_in_the_group, 0.1),
            size: Discrete::new(config.chord, number_of_groups, 0.1),
            contour: Continuous::new(),
            cutoff: Continuous::new(),
            resonance: Continuous::new(),
            scale: Toggle::new(config.scale, 12),
            mode: Toggle::new(config.mode, 8),
            arp: Toggle::new(config.arp, 6),
            trigger: DualTrigger::new(),
        }
    }

    pub fn copy_config(&self) -> PersistentConfig {
        PersistentConfig {
            tone: self.tone.copy_config(),
            chord: self.chord.copy_config(),
            size: self.size.copy_config(),
            scale: self.scale.copy_config(),
            mode: self.mode.copy_config(),
            arp: self.arp.copy_config(),
        }
    }
}
