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
        // NOTE: In case the number of output values was changed,
        // stick to the previous value, even if it is beyond the range.
        // With this, it is possible to recover back to the original
        // range without losing the position. The position is only
        // lost on toggle.
        usize::min(self.selected_value, self.values - 1)
    }

    pub fn set_output_values(&mut self, output_values: usize) {
        if self.values == output_values {
            return;
        }

        self.values = output_values;
    }

    pub fn set_value(&mut self, value: usize) {
        self.selected_value = value;
    }
}
