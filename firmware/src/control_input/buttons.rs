use super::debouncer::Debouncer;
use crate::system::hal::gpio;

const BUTTONS: usize = 7;

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
            ],
            pins,
        }
    }

    pub fn sample(&mut self, cycle: u8) {
        // NOTE: The order of multiplexer pins is not matching the numbering of
        // buttons on the module.
        const CYCLE_TO_BUTTON: [usize; 8] = [3, 2, 1, 6, 5, 999, 4, 0];
        // NOTE: No button is mapped to multiplexer's input 5.
        if cycle < 8 && cycle != 5 {
            let button_index = CYCLE_TO_BUTTON[cycle as usize];
            self.buttons[button_index].set(self.pins.button_mux.is_low());
        }
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
