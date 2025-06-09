mod arp_mode;
mod chord;
mod contour;
mod cutoff;
mod cv_assignment;
mod gain;
mod pluck;
mod primitives;
mod reset_next;
mod resonance;
mod scale;
mod scale_offsets;
mod stereo_mode;
mod trigger;
mod width;

use arp_mode::PersistentConfig as ArpModePersistentConfig;
use chord::PersistentConfig as ChordPersistentConfig;
use cv_assignment::PersistentConfig as CvAssignmentPersistentConfig;
use scale::PersistentConfig as ScalePersistentConfig;
use scale_offsets::PersistentConfig as ScaleOffsetsPersistentConfig;
use stereo_mode::PersistentConfig as StereoModePersistentConfig;

use crate::chords::Chords;
use crate::scales::Scales;

pub use self::arp_mode::ArpMode;
pub use self::chord::Chord;
pub use self::contour::Contour;
pub use self::cutoff::Cutoff;
pub use self::cv_assignment::{CvAssignment, CvAssignmentHandler};
pub use self::gain::Gain;
pub use self::pluck::Pluck;
pub use self::reset_next::ResetNext;
pub use self::resonance::Resonance;
pub use self::scale::Scale;
pub use self::scale_offsets::{ScaleOffsets, MAX_STEPS as SCALE_OFFSET_MAX_STEPS};
pub use self::stereo_mode::{StereoMode, StereoModeHandler};
pub use self::trigger::Trigger;
pub use self::width::Width;

pub struct Parameters {
    pub chord: Chord,
    pub contour: Contour,
    pub cutoff: Cutoff,
    pub resonance: Resonance,
    pub scale: Scale,
    pub arp_mode: ArpMode,
    pub trigger: Trigger,
    pub reset_next: ResetNext,
    pub pluck: Pluck,
    pub gain: Gain,
    pub width: Width,
    pub stereo_mode: StereoModeHandler,
    pub cv_assignment: CvAssignmentHandler,
    pub scale_offsets: ScaleOffsets,
}

#[derive(Default, PartialEq, Debug, Clone, Copy, defmt::Format)]
pub struct PersistentConfig {
    pub chord: ChordPersistentConfig,
    pub scale: ScalePersistentConfig,
    pub arp_mode: ArpModePersistentConfig,
    pub stereo_mode: StereoModePersistentConfig,
    pub cv_mapping: CvAssignmentPersistentConfig,
    pub scale_offsets: ScaleOffsetsPersistentConfig,
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
            gain: Gain::new(),
            width: Width::new(),
            stereo_mode: StereoModeHandler::new(config.stereo_mode),
            cv_assignment: CvAssignmentHandler::new(config.cv_mapping),
            scale_offsets: ScaleOffsets::new(config.scale_offsets),
        }
    }

    pub fn copy_config(&self) -> PersistentConfig {
        PersistentConfig {
            chord: self.chord.copy_config(),
            scale: self.scale.copy_config(),
            arp_mode: self.arp_mode.copy_config(),
            stereo_mode: self.stereo_mode.copy_config(),
            cv_mapping: self.cv_assignment.copy_config(),
            scale_offsets: self.scale_offsets.copy_config(),
        }
    }
}
