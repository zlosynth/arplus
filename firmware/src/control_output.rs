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
    pub led_ser: LedSer,     // Data
    pub led_srclk: LedSrclk, // Clock
    pub led_rclk: LedRclk,   // Latch
}

type LedSer = gpio::gpiob::PB4<gpio::Output>;
type LedSrclk = gpio::gpioc::PC11<gpio::Output>;
type LedRclk = gpio::gpioc::PC10<gpio::Output>;

impl ControlOutputInterface {
    pub fn new(config: Config) -> Self {
        Self {
            pins: config.pins,
            dac: config.dac,
        }
    }

    pub fn set_state(&mut self, state: &ControlOutputState) {
        self.pins.led_rclk.set_low();
        // TODO: Make sure that the timing requirements of the chip are met.
        for i in 0..8 {
            self.pins.led_ser.set_state(state.leds[i].into());
            self.pins.led_srclk.set_high();
            self.pins.led_srclk.set_low();
        }
        self.pins.led_rclk.set_high();

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
