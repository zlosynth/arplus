use crate::chords::{Chords, GroupId};

use super::{discrete::PersistentConfig, Discrete};

pub struct ChordGroupId {
    discrete: Discrete,
}

impl ChordGroupId {
    pub fn new(config: PersistentConfig) -> Self {
        Self {
            discrete: Discrete::new(config, Chords::GROUPS, 0.1),
        }
    }

    pub fn reconcile(&mut self, pot: f32, cv: Option<f32>) -> bool {
        self.discrete.reconcile(linear_sum(pot, cv))
    }

    pub fn selected(&self) -> GroupId {
        // SAFETY: Parameter values used to get group id are statically limited
        // by the number of groups.
        self.discrete.selected_value().try_into().unwrap()
    }

    pub fn copy_config(&self) -> PersistentConfig {
        self.discrete.copy_config()
    }
}

// TODO: Move it to a lib?
fn linear_sum(pot: f32, cv: Option<f32>) -> f32 {
    let offset_cv = cv.unwrap_or(0.0) / 5.0;
    let sum = pot + offset_cv;
    let clamped = sum.clamp(0.0, 1.0);
    clamped
}
