// TODO: Consider removing this altogether if not used in the
// final version.
use super::primitives::discrete::Discrete;
use super::primitives::discrete::PersistentConfig as DiscretePersistentConfig;

const LEVELS: usize = 4;
const MIN: f32 = 0.2;
const MAX: f32 = 0.6;

pub struct Gain {
    discrete: Discrete,
}

#[derive(Clone, Copy, defmt::Format, PartialEq, Debug)]
pub struct PersistentConfig {
    discrete: DiscretePersistentConfig,
}

impl Gain {
    pub fn new(config: PersistentConfig) -> Self {
        Self {
            discrete: Discrete::new(config.discrete, LEVELS, 0.1, 1.0),
        }
    }

    pub fn copy_config(&self) -> PersistentConfig {
        PersistentConfig {
            discrete: self.discrete.copy_config(),
        }
    }

    pub fn reconcile(&mut self, input_level: f32) -> bool {
        self.discrete.reconcile(input_level)
    }

    pub fn value(&self) -> f32 {
        let phase = self.discrete.selected_value() as f32 / (LEVELS - 1) as f32;
        MIN + (MAX - MIN) * phase
    }

    pub fn selected_index(&self) -> usize {
        self.discrete.selected_value()
    }
}

impl Default for PersistentConfig {
    fn default() -> Self {
        Self {
            discrete: DiscretePersistentConfig::new(LEVELS - 1),
        }
    }
}
