use super::primitives::discrete::{Discrete, PersistentConfig as DiscretePersistentConfig};
use super::primitives::math;

const MIN_LENGTH: usize = 1;
const MAX_LENGTH: usize = 6;

pub struct Strings {
    length: Discrete,
}

#[derive(Default, PartialEq, Debug, Clone, Copy, defmt::Format)]
pub struct PersistentConfig {
    length: DiscretePersistentConfig,
}

impl Strings {
    // NOTE: Passing scale size is awkward, but dynamic sizing of chords is
    // necessary to allow for an arpeggio with all notes of a scale.
    pub fn new(config: PersistentConfig) -> Self {
        Self {
            length: Discrete::new(config.length, MAX_LENGTH - MIN_LENGTH + 1, 0.1, 1.0),
        }
    }

    pub fn reconcile(&mut self, strings_pot: f32, strings_cv: Option<f32>) -> bool {
        self.length
            .reconcile(math::linear_sum(strings_pot, strings_cv))
    }

    pub fn value(&self) -> usize {
        MIN_LENGTH + self.length.selected_value()
    }

    pub fn copy_config(&self) -> PersistentConfig {
        PersistentConfig {
            length: self.length.copy_config(),
        }
    }
}
