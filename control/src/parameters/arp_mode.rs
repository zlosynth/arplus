use crate::arpeggiator::Mode;

// TODO: Once this handles more than one inputs (mapped CV), then
// this will need to define its own persistent config structure.
use super::toggle::PersistentConfig;
use super::toggle::Toggle;

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

    pub fn selected_mode(&self) -> Mode {
        // SAFETY: Parameter values used to get arp index are statically limited
        // by the maximum number of modes.
        Mode::try_from_index(self.toggle.selected_value()).unwrap()
    }
}
