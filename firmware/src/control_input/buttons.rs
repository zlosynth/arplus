use super::debouncer::Debouncer;
use crate::system::hal::gpio;

const BUTTONS: usize = 6;

#[derive(Debug, defmt::Format)]
pub struct Buttons {
    buttons: [Button; BUTTONS],
    pins: Pins,
}

#[derive(Debug, defmt::Format)]
pub struct Button {
    active: bool,
    debouncer: Debouncer<4>,
}

#[derive(Debug, defmt::Format)]
pub struct Pins {
    pub button_1: Button1Pin,
    pub button_2: Button2Pin,
    pub button_3: Button3Pin,
    pub button_4: Button4Pin,
    pub button_5: Button5Pin,
    pub button_6: Button6Pin,
}

pub type Button1Pin = gpio::gpiob::PB8<gpio::Input>;
pub type Button2Pin = gpio::gpiob::PB9<gpio::Input>;
pub type Button3Pin = gpio::gpiob::PB14<gpio::Input>;
pub type Button4Pin = gpio::gpiob::PB15<gpio::Input>;
pub type Button5Pin = gpio::gpioc::PC12<gpio::Input>;
pub type Button6Pin = gpio::gpioc::PC8<gpio::Input>;

impl Buttons {
    pub fn new(pins: Pins) -> Self {
        Self {
            buttons: [
                Button::new(),
                Button::new(),
                Button::new(),
                Button::new(),
                Button::new(),
                Button::new(),
            ],
            pins,
        }
    }

    pub fn sample(&mut self) {
        self.buttons[0].set(self.pins.button_1.is_low());
        self.buttons[1].set(self.pins.button_2.is_low());
        self.buttons[2].set(self.pins.button_3.is_low());
        self.buttons[3].set(self.pins.button_4.is_low());
        self.buttons[4].set(self.pins.button_5.is_low());
        self.buttons[5].set(self.pins.button_6.is_low());
    }

    pub fn values(&self) -> [bool; BUTTONS] {
        [
            self.buttons[0].active,
            self.buttons[1].active,
            self.buttons[2].active,
            self.buttons[3].active,
            self.buttons[4].active,
            self.buttons[5].active,
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
