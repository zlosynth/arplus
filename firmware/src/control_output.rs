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

type LedSer = gpio::gpioc::PC10<gpio::Output>;
type LedSrclk = gpio::gpiob::PB4<gpio::Output>;
type LedRclk = gpio::gpioc::PC11<gpio::Output>;

impl ControlOutputInterface {
    pub fn new(config: Config) -> Self {
        Self {
            pins: config.pins,
            dac: config.dac,
        }
    }

    pub fn set_state(&mut self, state: &ControlOutputState) {
        // NOTE: Delay added to meet the timing requirements of SN54HC595.
        const USECOND: u32 = 480_000_000 / 1_000_000;
        // NOTE: Minimal pulse time for RCLK is guaranteed from both sides.
        self.pins.led_rclk.set_low();
        for i in 0..8 {
            // NOTE: Shift register is wired from top to bottom, but the
            // state expects bottom up.
            self.pins.led_ser.set_state(state.leds[7 - i].into());
            // NOTE: Minimal setup time of SER before SRCLK is 125 ns.
            cortex_m::asm::delay(USECOND);

            self.pins.led_srclk.set_high();
            // NOTE: Minimal pulse duration for SRCLK is 100 ns.
            // NOTE: Minimal setup time for SRCLK up before RCLK up is 94 ns.
            cortex_m::asm::delay(USECOND);
            self.pins.led_srclk.set_low();
        }
        self.pins.led_rclk.set_high();

        self.dac.set_value(f32_cv_to_u16(state.cv));
    }
}

fn f32_cv_to_u16(value: f32) -> u16 {
    const OUT_MIN: f32 = 0.0;
    const OUT_MAX: f32 = 5.0;
    // NOTE: The value should be already trimmed and scale. But clamp it again
    // just to be sure.
    let trimmed = value.clamp(OUT_MIN, OUT_MAX);
    let desired = (trimmed - OUT_MIN) / (OUT_MAX - OUT_MIN);
    // NOTE: Measuring of DAC showed that it actually starts above 0.0 V,
    // and does not get all the way to 5.0. This compensates for that.
    let compensated = desired * (4.0 / (3.94 - 0.009)) - (0.009 / 5.0);
    let scaled = (compensated * 4096.0).clamp(0.0, 4095.999);
    scaled as u16
}
