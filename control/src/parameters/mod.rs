// TODO: Review

mod arp_mode;
mod chord;
mod contour;
mod cutoff;
mod primitives;
mod resonance;
mod scale;
mod trigger;

use arp_mode::PersistentConfig as ArpModePersistentConfig;
use chord::PersistentConfig as ChordPersistentConfig;
use scale::PersistentConfig as ScalePersistentConfig;

use crate::chords::Chords;
use crate::scales::Scales;

pub use self::arp_mode::ArpMode;
pub use self::chord::Chord;
pub use self::contour::Contour;
pub use self::cutoff::Cutoff;
pub use self::resonance::Resonance;
pub use self::scale::Scale;
pub use self::trigger::Trigger;

pub struct Parameters {
    pub chord: Chord,
    pub contour: Contour,
    pub cutoff: Cutoff,
    pub resonance: Resonance,
    pub scale: Scale,
    pub arp_mode: ArpMode,
    pub trigger: Trigger,
}

#[derive(Default, PartialEq, Debug, Clone, Copy, defmt::Format)]
pub struct PersistentConfig {
    pub chord: ChordPersistentConfig,
    pub scale: ScalePersistentConfig,
    pub arp_mode: ArpModePersistentConfig,
}

impl Parameters {
    pub fn new(config: PersistentConfig, chords: Chords, scales: Scales) -> Self {
        Self {
            // TODO: Allow configuration of tonic
            chord: Chord::new(config.chord, chords),
            contour: Contour::new(),
            cutoff: Cutoff::new(),
            resonance: Resonance::new(),
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
