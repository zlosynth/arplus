use crate::system::hal::gpio;

use arplus_control::ControlOutputState;
use stm32h7xx_hal::dac::{Enabled, C1};
use stm32h7xx_hal::device::DAC;
use stm32h7xx_hal::traits::DacOut;

pub struct ControlOutputInterface {
    pins: Pins,
    dac: C1<DAC, Enabled>,
}

pub struct Config {
    pub pins: Pins,
    pub dac: C1<DAC, Enabled>,
}

#[derive(Debug, defmt::Format)]
pub struct Pins {
    pub leds: (
        Led1Pin,
        Led2Pin,
        Led3Pin,
        Led4Pin,
        Led5Pin,
        Led6Pin,
        Led7Pin,
        Led8Pin,
    ),
}

type Led1Pin = gpio::gpiod::PD2<gpio::Output>; // D7
type Led2Pin = gpio::gpioc::PC12<gpio::Output>; // D6
type Led3Pin = gpio::gpioc::PC8<gpio::Output>; // D5
type Led4Pin = gpio::gpioc::PC9<gpio::Output>; // D4
type Led5Pin = gpio::gpioc::PC10<gpio::Output>; // D3
type Led6Pin = gpio::gpioc::PC11<gpio::Output>; // D2
type Led7Pin = gpio::gpiob::PB4<gpio::Output>; // D1
type Led8Pin = gpio::gpiod::PD3<gpio::Output>; // D10

impl ControlOutputInterface {
    pub fn new(config: Config) -> Self {
        Self {
            pins: config.pins,
            dac: config.dac,
        }
    }

    pub fn set_state(&mut self, state: &ControlOutputState) {
        self.pins.leds.0.set_state(state.leds[0].into());
        self.pins.leds.1.set_state(state.leds[1].into());
        self.pins.leds.2.set_state(state.leds[2].into());
        self.pins.leds.3.set_state(state.leds[3].into());
        self.pins.leds.4.set_state(state.leds[4].into());
        self.pins.leds.5.set_state(state.leds[5].into());
        self.pins.leds.6.set_state(state.leds[6].into());
        self.pins.leds.7.set_state(state.leds[7].into());
        self.dac.set_value(f32_cv_to_u16(state.cv));
    }
}

fn f32_cv_to_u16(value: f32) -> u16 {
    const OUT_MIN: f32 = 0.0;
    const OUT_MAX: f32 = 5.0;
    // NOTE: The DSP works with 7 octaves, but the module can output only 5.
    // Remove the first and the last octave.
    let trimmed = (value - 1.0).clamp(0.0, 5.0);
    let desired = (trimmed - OUT_MIN) / (OUT_MAX - OUT_MIN);
    // NOTE: Measuring of DAC showed that it actually starts above 0.0 V,
    // and does not get all the way to 5.0. This compensates for that.
    let compensated = desired * (4.0 / (3.94 - 0.009)) - (0.009 / 5.0);
    let scaled = (compensated * 4096.0).clamp(0.0, 4095.999);
    scaled as u16
}
