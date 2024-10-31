mod arp_mode;
mod chord;
mod contour;
mod cutoff;
mod cv_mapping;
mod gain;
mod pluck;
mod primitives;
mod reset_next;
mod resonance;
mod scale;
mod trigger;

use arp_mode::PersistentConfig as ArpModePersistentConfig;
use chord::PersistentConfig as ChordPersistentConfig;
use cv_mapping::PersistentConfig as CvMappingPersistentConfig;
use gain::PersistentConfig as GainPersistentConfig;
use scale::PersistentConfig as ScalePersistentConfig;

use crate::chords::Chords;
use crate::scales::Scales;

pub use self::arp_mode::ArpMode;
pub use self::chord::Chord;
pub use self::contour::Contour;
pub use self::cutoff::Cutoff;
pub use self::cv_mapping::{CvMapping, Socket as CvMappingSocket};
pub use self::gain::Gain;
pub use self::pluck::Pluck;
pub use self::reset_next::ResetNext;
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
    pub reset_next: ResetNext,
    pub gain: Gain,
    pub pluck: Pluck,
    pub cv_mapping: CvMapping,
}

#[derive(Default, PartialEq, Debug, Clone, Copy, defmt::Format)]
pub struct PersistentConfig {
    pub chord: ChordPersistentConfig,
    pub scale: ScalePersistentConfig,
    pub arp_mode: ArpModePersistentConfig,
    pub gain: GainPersistentConfig,
    pub cv_mapping: CvMappingPersistentConfig,
}

impl Parameters {
    pub fn new(config: PersistentConfig, chords: Chords, scales: Scales) -> Self {
        let scale = Scale::new(config.scale, scales);
        Self {
            chord: Chord::new(config.chord, chords, scale.selected_scale_size()),
            contour: Contour::new(),
            cutoff: Cutoff::new(),
            resonance: Resonance::new(),
            pluck: Pluck::new(),
            scale,
            arp_mode: ArpMode::new(config.arp_mode),
            trigger: Trigger::new(),
            reset_next: ResetNext::new(),
            gain: Gain::new(config.gain),
            cv_mapping: CvMapping::new(config.cv_mapping),
        }
    }

    pub fn copy_config(&self) -> PersistentConfig {
        PersistentConfig {
            chord: self.chord.copy_config(),
            scale: self.scale.copy_config(),
            arp_mode: self.arp_mode.copy_config(),
            gain: self.gain.copy_config(),
            cv_mapping: self.cv_mapping.copy_config(),
        }
    }
}
