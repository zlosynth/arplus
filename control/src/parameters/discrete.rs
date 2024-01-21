pub struct Discrete {
    block_width: f32,
    margin: f32,
    values: usize,
    selected_value: usize,
}

#[derive(Default, PartialEq, Debug, Clone, Copy, defmt::Format)]
pub struct PersistentConfig {
    selected_value: usize,
}

impl Discrete {
    pub fn new(config: PersistentConfig, output_values: usize, relative_margin: f32) -> Self {
        Self {
            block_width: 1.0 / output_values as f32,
            margin: relative_margin / output_values as f32,
            values: output_values,
            selected_value: config.selected_value,
        }
    }

    pub fn reconcile(&mut self, input_level: f32) -> bool {
        let current_output_value_f32 = self.selected_value as f32;

        let mut lower_bound = self.block_width * current_output_value_f32;
        if self.selected_value > 0 {
            lower_bound -= self.margin;
        }

        let mut upper_bound = self.block_width * (current_output_value_f32 + 1.0);
        if self.selected_value < self.values {
            upper_bound += self.margin;
        }

        if input_level < lower_bound || input_level > upper_bound {
            self.selected_value = (input_level * self.values as f32) as usize;
            true
        } else {
            false
        }
    }

    pub fn selected_value(&self) -> usize {
        self.selected_value
    }
}

impl Discrete {
    pub fn copy_config(&self) -> PersistentConfig {
        PersistentConfig {
            selected_value: self.selected_value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn after_crossing_border_to_right_it_takes_extra_effort_to_move_back_left() {
        const RESOLUTION: f32 = 0.01;

        const INITIAL_VALUE: usize = 0;
        const OUTPUT_VALUES: usize = 2;
        const MARGIN: f32 = 0.1;

        let mut d = Discrete::new(
            PersistentConfig {
                selected_value: INITIAL_VALUE,
            },
            OUTPUT_VALUES,
            MARGIN,
        );

        let mut travel_right = 0;
        loop {
            d.reconcile(RESOLUTION * travel_right as f32);
            let output = d.selected_value();
            if output == 1 {
                break;
            } else {
                travel_right += 1;
            }
        }

        let mut travel_left = 1;
        loop {
            d.reconcile(RESOLUTION * travel_right as f32 - RESOLUTION * travel_left as f32);
            let output = d.selected_value();
            if output == 0 {
                break;
            } else {
                travel_left += 1;
            }
        }

        assert!(travel_left as f32 / travel_right as f32 > MARGIN);
    }
}
