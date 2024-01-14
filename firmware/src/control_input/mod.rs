pub mod buttons;
mod debouncer;
mod one_pole_filter;
pub mod pots;

use arplus_control::ControlInputSnapshot;

use self::buttons::{Buttons, Pins as ButtonsPins};
use self::pots::{Pins as PotsPins, Pots};
use crate::system::hal::adc::{Adc, Enabled};
use crate::system::hal::pac::{ADC1, ADC2};

pub struct ControlInputInterface {
    pots: Pots,
    buttons: Buttons,
    #[allow(dead_code)]
    cvs: (),
    adc_1: Adc<ADC1, Enabled>,
    adc_2: Adc<ADC2, Enabled>,
}

pub struct Config {
    pub pots_pins: PotsPins,
    pub buttons_pins: ButtonsPins,
    pub adc_1: Adc<ADC1, Enabled>,
    pub adc_2: Adc<ADC2, Enabled>,
}

impl ControlInputInterface {
    pub fn new(config: Config) -> Self {
        Self {
            pots: Pots::new(config.pots_pins),
            buttons: Buttons::new(config.buttons_pins),
            cvs: (),
            adc_1: config.adc_1,
            adc_2: config.adc_2,
        }
    }

    pub fn sample(&mut self) {
        self.pots.sample(&mut self.adc_1, &mut self.adc_2);
        self.buttons.sample();
    }

    pub fn snapshot(&self) -> ControlInputSnapshot {
        todo!();
    }
}
