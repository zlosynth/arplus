use super::debouncer::Debouncer;
use crate::system::hal::gpio;

const BUTTONS: usize = 8;

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
    pub button_mux: ButtonMuxPin,
}

pub type ButtonMuxPin = gpio::gpiob::PB8<gpio::Input>;

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
                Button::new(),
                Button::new(),
            ],
            pins,
        }
    }

    pub fn sample(&mut self, cycle: u8) {
        assert!(cycle < BUTTONS as u8);
        self.buttons[cycle as usize].set(self.pins.button_mux.is_low());
    }

    pub fn values(&self) -> [bool; BUTTONS] {
        [
            self.buttons[0].active,
            self.buttons[1].active,
            self.buttons[2].active,
            self.buttons[3].active,
            self.buttons[4].active,
            self.buttons[5].active,
            self.buttons[6].active,
            self.buttons[7].active,
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
