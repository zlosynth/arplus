// TODO: Introduce two submodules. One for building blocks and one for instrument params
mod arp_mode;
mod chord;
mod scale;
mod trigger;

mod continuous;
mod discrete;
mod toggle;

use chord::PersistentConfig as ChordPersistentConfig;
use discrete::PersistentConfig as DiscretePersistentConfig;
use scale::PersistentConfig as ScalePersistentConfig;
use toggle::PersistentConfig as TogglePersistentConfig;

// TODO: Do not use these directly. Instead, use them as primitives in types dedicated to what they represent.
pub use continuous::Continuous;
pub use discrete::Discrete;
pub use toggle::Toggle;
pub use trigger::Trigger;

use crate::{chords::Chords, scales::Scales};

pub use self::arp_mode::ArpMode;
pub use self::chord::Chord;
pub use self::scale::Scale;

pub struct Parameters {
    // TODO: Switch this to something VOct specific later.
    pub chord: Chord,
    pub contour: Continuous,
    pub cutoff: Continuous,
    pub resonance: Continuous,
    pub scale: Scale,
    pub arp_mode: ArpMode,
    pub trigger: Trigger,
}

#[derive(Default, PartialEq, Debug, Clone, Copy, defmt::Format)]
pub struct PersistentConfig {
    pub chord: ChordPersistentConfig,
    pub scale: ScalePersistentConfig,
    pub arp_mode: TogglePersistentConfig,
}

impl Parameters {
    pub fn new(config: PersistentConfig, chords: &Chords, scales: &Scales) -> Self {
        Self {
            // TODO: Allow configuration of tonic
            chord: Chord::new(config.chord, chords),
            contour: Continuous::new(),
            cutoff: Continuous::new(),
            resonance: Continuous::new(),
            scale: Scale::new(config.scale, scales),
            arp_mode: ArpMode::new(config.arp_mode),
            trigger: Trigger::new(),
        }
    }

    pub fn copy_config(&self) -> PersistentConfig {
        PersistentConfig {
            chord: self.chord.copy_config(),
            scale: self.scale.copy_config(),
            arp_mode: self.arp_mode.copy_config(),
        }
    }
}
