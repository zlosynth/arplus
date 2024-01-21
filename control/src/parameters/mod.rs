mod continuous;
mod discrete;
mod toggle;
mod trigger;

use continuous::Continuous;
use discrete::{Discrete, PersistentConfig as DiscretePersistentConfig};
use toggle::{PersistentConfig as TogglePersistentConfig, Toggle};
use trigger::DualTrigger;

pub struct Parameters {
    // TODO: Switch this to something VOct specific later.
    pub tone: Discrete,
    pub chord: Discrete,
    pub contour: Continuous,
    pub gain: Continuous,
    pub cutoff: Continuous,
    pub resonance: Continuous,
    pub tonic: Toggle,
    pub mode: Toggle,
    pub arp: Toggle,
    pub trigger: DualTrigger,
}

#[derive(Default, PartialEq, Debug, Clone, Copy, defmt::Format)]
pub struct PersistentConfig {
    pub tone: DiscretePersistentConfig,
    pub chord: DiscretePersistentConfig,
    pub tonic: TogglePersistentConfig,
    pub mode: TogglePersistentConfig,
    pub arp: TogglePersistentConfig,
}

impl Parameters {
    pub fn new(config: PersistentConfig) -> Self {
        Self {
            // TODO: Set proper ranges
            tone: Discrete::new(config.tone, 7 * 6, 0.1),
            chord: Discrete::new(config.chord, 18, 0.1),
            contour: Continuous::new(),
            gain: Continuous::new(),
            cutoff: Continuous::new(),
            resonance: Continuous::new(),
            tonic: Toggle::new(config.tonic, 12),
            mode: Toggle::new(config.mode, 8),
            arp: Toggle::new(config.arp, 6),
            trigger: DualTrigger::new(),
        }
    }

    pub fn copy_config(&self) -> PersistentConfig {
        PersistentConfig {
            tone: self.tone.copy_config(),
            chord: self.chord.copy_config(),
            tonic: self.tonic.copy_config(),
            mode: self.mode.copy_config(),
            arp: self.arp.copy_config(),
        }
    }
}
