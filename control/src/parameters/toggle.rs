pub struct Toggle {
    values: usize,
    selected_value: usize,
}

#[derive(Default, PartialEq, Debug, Clone, Copy, defmt::Format)]
pub struct PersistentConfig {
    pub selected_value: usize,
}

impl Toggle {
    pub fn new(config: PersistentConfig, values: usize) -> Self {
        Self {
            values,
            selected_value: config.selected_value,
        }
    }

    pub fn copy_config(&self) -> PersistentConfig {
        PersistentConfig {
            selected_value: self.selected_value,
        }
    }

    pub fn reconcile(&mut self, toggle: bool) -> bool {
        if toggle {
            self.selected_value += 1;
            self.selected_value %= self.values;
            true
        } else {
            false
        }
    }

    pub fn selected_value(&self) -> usize {
        self.selected_value
    }
}
