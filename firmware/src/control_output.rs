use crate::system::hal::gpio;

use arplus_control::ControlOutputState;

#[derive(Debug, defmt::Format)]
pub struct ControlOutputInterface {
    pins: Pins,
}

pub struct Config {
    pub pins: Pins,
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
        Self { pins: config.pins }
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
    }
}
