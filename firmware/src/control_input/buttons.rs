use super::debouncer::Debouncer;
use crate::system::hal::gpio;

#[derive(Debug, defmt::Format)]
pub(super) struct Buttons {
    buttons: [Button; 4],
    pins: Pins,
}

#[derive(Debug, defmt::Format)]
pub(super) struct Button {
    pub active: bool,
    pub active_no_filter: bool,
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
}

impl Button {
    fn new() -> Self {
        Self {
            debouncer: Debouncer::new(),
            active: false,
            active_no_filter: false,
        }
    }

    fn set(&mut self, is_high: bool) {
        self.active_no_filter = is_high;
        self.active = self.debouncer.update(is_high);
    }
}
