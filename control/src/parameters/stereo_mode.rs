use arplus_dsp::StereoMode as DSPStereoMode;

use super::primitives::discrete::{Discrete, PersistentConfig as DiscretePersistentConfig};

pub struct StereoMode {
    discrete: Discrete,
}

#[derive(Default, PartialEq, Debug, Clone, Copy, defmt::Format)]
pub struct PersistentConfig {
    discrete: DiscretePersistentConfig,
}

impl StereoMode {
    pub fn new(config: PersistentConfig) -> Self {
        Self {
            discrete: Discrete::new(config.discrete, 2, 0.1),
        }
    }

    pub fn copy_config(&self) -> PersistentConfig {
        PersistentConfig {
            discrete: self.discrete.copy_config(),
        }
    }

    pub fn reconcile(&mut self, value: f32) -> bool {
        self.discrete.reconcile(value)
    }

    pub fn selected(&self) -> DSPStereoMode {
        // SAFETY: Parameter values used are statically limited by the maximum
        // number of modes.
        DSPStereoMode::try_from(self.discrete.selected_value()).unwrap()
    }
}
