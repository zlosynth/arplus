use crate::arpeggiator::Mode;

use super::primitives::discrete::{Discrete, PersistentConfig as DiscretePersistentConfig};
use super::primitives::toggle::{PersistentConfig as TogglePersistentConfig, Toggle};

pub struct ArpMode {
    toggle: Toggle,
    discrete: Discrete,
    cv_control: bool,
}

#[derive(Default, PartialEq, Debug, Clone, Copy, defmt::Format)]
pub struct PersistentConfig {
    toggle: TogglePersistentConfig,
    discrete: DiscretePersistentConfig,
}

impl ArpMode {
    pub fn new(config: PersistentConfig) -> Self {
        Self {
            toggle: Toggle::new(config.toggle, Mode::LAST_MODE as usize + 1),
            discrete: Discrete::new(config.discrete, Mode::LAST_MODE as usize + 1, 0.1),
            cv_control: false,
        }
    }

    pub fn copy_config(&self) -> PersistentConfig {
        PersistentConfig {
            toggle: self.toggle.copy_config(),
            discrete: self.discrete.copy_config(),
        }
    }

    pub fn reconcile_button(&mut self, toggle: bool) -> bool {
        self.toggle.reconcile(toggle)
    }

    pub fn reconcile_cv(&mut self, value: f32) -> bool {
        self.discrete.reconcile(value)
    }

    pub fn set_cv_control(&mut self, enable: bool) {
        self.cv_control = enable;
    }

    // TODO: Return CV-set or button-set based on the current state
    pub fn selected(&self) -> Mode {
        let selected_value = if self.cv_control {
            self.discrete.selected_value()
        } else {
            self.toggle.selected_value()
        };
        // SAFETY: Parameter values used to get arp index are statically limited
        // by the maximum number of modes.
        Mode::try_from_index(selected_value).unwrap()
    }
}
