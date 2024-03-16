use crate::arpeggiator::Mode;

pub use super::primitives::toggle::PersistentConfig;
use super::primitives::toggle::Toggle;

pub struct ArpMode {
    toggle: Toggle,
}

impl ArpMode {
    pub fn new(config: PersistentConfig) -> Self {
        Self {
            toggle: Toggle::new(config, Mode::LAST_MODE as usize + 1),
        }
    }

    pub fn copy_config(&self) -> PersistentConfig {
        self.toggle.copy_config()
    }

    pub fn reconcile(&mut self, toggle: bool) -> bool {
        self.toggle.reconcile(toggle)
    }

    pub fn selected(&self) -> Mode {
        // SAFETY: Parameter values used to get arp index are statically limited
        // by the maximum number of modes.
        Mode::try_from_index(self.toggle.selected_value()).unwrap()
    }
}
