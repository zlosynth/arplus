use crate::system::hal::gpio;

use super::debouncer::Debouncer;

const TRIGGERS: usize = 1;

#[derive(defmt::Format)]
pub struct Gates {
    triggers: [Trigger; TRIGGERS],
    pins: Pins,
}

#[derive(Debug, defmt::Format)]
pub struct Trigger {
    active: bool,
    debouncer: Debouncer<4>,
}

#[derive(defmt::Format)]
pub struct Pins {
    pub gate_1: Trigger1Pin,
}

pub type Trigger1Pin = gpio::gpiog::PG13<gpio::Input>;

impl Gates {
    pub fn new(pins: Pins) -> Self {
        Self {
            triggers: [Trigger::new()],
            pins,
        }
    }

    pub fn sample(&mut self) {
        self.triggers[0].set(self.pins.gate_1.is_high());
    }

    pub fn values(&self) -> [bool; TRIGGERS] {
        [self.triggers[0].active]
    }
}

impl Trigger {
    fn new() -> Self {
        Self {
            debouncer: Debouncer::new(),
            active: false,
        }
    }

    fn set(&mut self, is_high: bool) {
        self.active = self.debouncer.update(is_high);
    }
}
