use super::debouncer::Debouncer;
use crate::system::hal::gpio;

const BUTTONS: usize = 4;

#[derive(Debug, defmt::Format)]
pub(super) struct Buttons {
    buttons: [Button; BUTTONS],
    pins: Pins,
}

#[derive(Debug, defmt::Format)]
pub(super) struct Button {
    active: bool,
    debouncer: Debouncer<4>,
}

#[derive(Debug, defmt::Format)]
pub struct Pins {
    pub button_1: Button1Pin,
    pub button_2: Button2Pin,
    pub button_3: Button3Pin,
    pub button_4: Button4Pin,
}

pub(super) type Button1Pin = gpio::gpiob::PB9<gpio::Input>;
pub(super) type Button2Pin = gpio::gpiob::PB8<gpio::Input>;
pub(super) type Button3Pin = gpio::gpiob::PB15<gpio::Input>;
pub(super) type Button4Pin = gpio::gpiob::PB14<gpio::Input>;

impl Buttons {
    pub(super) fn new(pins: Pins) -> Self {
        Self {
            buttons: [Button::new(), Button::new(), Button::new(), Button::new()],
            pins,
        }
    }

    pub(super) fn sample(&mut self) {
        self.buttons[0].set(self.pins.button_1.is_high());
        self.buttons[1].set(self.pins.button_2.is_high());
        self.buttons[2].set(self.pins.button_3.is_high());
        self.buttons[3].set(self.pins.button_4.is_high());
    }

    pub(super) fn values(&self) -> [bool; BUTTONS] {
        [
            self.buttons[0].active,
            self.buttons[1].active,
            self.buttons[2].active,
            self.buttons[3].active,
        ]
    }
}

impl Button {
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
