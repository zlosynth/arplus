use nb::block;

use crate::system::hal::adc::{Adc, Enabled};
use crate::system::hal::gpio;
use crate::system::hal::pac::{ADC1, ADC2};

use super::one_pole_filter::OnePoleFilter;

const POTS: usize = 10;

#[derive(defmt::Format)]
pub struct Pots {
    pots: [Pot; POTS],
    pins: Pins,
}

#[derive(defmt::Format)]
pub struct Pot {
    value: f32,
    offset: f32,
    multiplier: f32,
    filter: OnePoleFilter,
}

#[derive(defmt::Format)]
pub struct Pins {
    pub pot_mux: PotMuxPin,
    pub pot_9: Pot9Pin,
    pub pot_10: Pot10Pin,
}

pub type PotMuxPin = gpio::gpioa::PA1<gpio::Analog>;
pub type Pot9Pin = gpio::gpioa::PA3<gpio::Analog>;
pub type Pot10Pin = gpio::gpioc::PC1<gpio::Analog>;

impl Pots {
    pub fn new(pins: Pins) -> Self {
        Self {
            pots: [
                // NOTE: To calibrate, set this to (0.0, 1.0), remove clamping
                // from Pot.set, and run diagnostics. Note the minimum and
                // maximum of each pot and then use it here.
                Pot::new(0.99, 0.01),
                Pot::new(0.99, 0.01),
                Pot::new(0.99, 0.01),
                Pot::new(0.99, 0.01),
                Pot::new(0.99, 0.01),
                Pot::new(0.99, 0.01),
                Pot::new(0.99, 0.01),
                Pot::new(0.99, 0.01),
                Pot::new(0.505, 0.99),
                Pot::new(0.505, 0.99),
            ],
            pins,
        }
    }

    pub fn sample(
        &mut self,
        cycle: u8,
        adc_1: &mut Adc<ADC1, Enabled>,
        adc_2: &mut Adc<ADC2, Enabled>,
    ) {
        assert!(cycle < POTS as u8);

        match cycle {
            0 => {
                adc_1.start_conversion(&mut self.pins.pot_mux);
                adc_2.start_conversion(&mut self.pins.pot_9);
                let sample_mux: u32 = block!(adc_1.read_sample()).unwrap_or_default();
                let sample_9: u32 = block!(adc_2.read_sample()).unwrap_or_default();
                self.pots[cycle as usize].set(sample_mux, adc_1.slope());
                self.pots[8].set(sample_9, adc_2.slope());
            }
            1 => {
                adc_1.start_conversion(&mut self.pins.pot_mux);
                adc_2.start_conversion(&mut self.pins.pot_10);
                let sample_mux: u32 = block!(adc_1.read_sample()).unwrap_or_default();
                let sample_10: u32 = block!(adc_2.read_sample()).unwrap_or_default();
                self.pots[cycle as usize].set(sample_mux, adc_1.slope());
                self.pots[9].set(sample_10, adc_2.slope());
            }
            _ => {
                adc_1.start_conversion(&mut self.pins.pot_mux);
                let sample_mux: u32 = block!(adc_1.read_sample()).unwrap_or_default();
                self.pots[cycle as usize].set(sample_mux, adc_1.slope());
            }
        }
    }

    pub fn values(&self) -> [f32; POTS] {
        [
            self.pots[0].value,
            self.pots[1].value,
            self.pots[2].value,
            self.pots[3].value,
            self.pots[4].value,
            self.pots[5].value,
            self.pots[6].value,
            self.pots[7].value,
            self.pots[8].value,
            self.pots[9].value,
        ]
    }
}

impl Pot {
    fn new(adc_min: f32, adc_max: f32) -> Self {
        let offset = -adc_min;
        let multiplier = 1.0 / (adc_max - adc_min);
        let filter = OnePoleFilter::new(1000.0, 10.0);
        Self {
            value: 0.0,
            offset,
            multiplier,
            filter,
        }
    }

    fn set(&mut self, sample: u32, slope: u32) {
        let phased = (slope as f32 - sample as f32) / slope as f32;
        let scaled = (phased + self.offset) * self.multiplier;
        let clamped = scaled.clamp(0.0, 1.0);
        self.value = self.filter.tick(clamped);
    }
}
