use nb::block;

use crate::system::hal::adc::{Adc, Enabled};
use crate::system::hal::gpio;
use crate::system::hal::pac::{ADC1, ADC2};

use super::probe::Detector as ProbeDetector;

#[derive(defmt::Format)]
pub(super) struct Cvs {
    pub cv: [Cv; 6],
    pins: Pins,
}

#[derive(Default, defmt::Format)]
pub struct Cv {
    pub value: Option<f32>,
    probe: ProbeDetector,
}

#[derive(defmt::Format)]
pub struct Pins {
    pub cv_1: Cv1Pin,
    pub cv_2: Cv2Pin,
    pub cv_3: Cv3Pin,
    pub cv_4: Cv4Pin,
    pub cv_5: Cv5Pin,
    pub cv_6: Cv6Pin,
}

pub type Cv1Pin = gpio::gpioa::PA6<gpio::Analog>;
pub type Cv2Pin = gpio::gpioa::PA3<gpio::Analog>;
pub type Cv3Pin = gpio::gpioc::PC4<gpio::Analog>;
pub type Cv4Pin = gpio::gpioc::PC1<gpio::Analog>;
pub type Cv5Pin = gpio::gpioc::PC0<gpio::Analog>;
pub type Cv6Pin = gpio::gpiob::PB1<gpio::Analog>;

impl Cvs {
    pub fn new(pins: Pins) -> Self {
        Self {
            cv: [
                Cv::default(),
                Cv::default(),
                Cv::default(),
                Cv::default(),
                Cv::default(),
                Cv::default(),
            ],
            pins,
        }
    }

    pub fn sample(&mut self, adc_1: &mut Adc<ADC1, Enabled>, adc_2: &mut Adc<ADC2, Enabled>) {
        adc_1.start_conversion(&mut self.pins.cv_1);
        adc_2.start_conversion(&mut self.pins.cv_2);
        let sample_1: u32 = block!(adc_1.read_sample()).unwrap_or_default();
        let sample_2: u32 = block!(adc_2.read_sample()).unwrap_or_default();
        self.cv[0].set(sample_1, adc_1.slope());
        self.cv[1].set(sample_2, adc_2.slope());

        adc_1.start_conversion(&mut self.pins.cv_3);
        adc_2.start_conversion(&mut self.pins.cv_4);
        let sample_3: u32 = block!(adc_1.read_sample()).unwrap_or_default();
        let sample_4: u32 = block!(adc_2.read_sample()).unwrap_or_default();
        self.cv[2].set(sample_3, adc_1.slope());
        self.cv[3].set(sample_4, adc_2.slope());

        adc_1.start_conversion(&mut self.pins.cv_5);
        adc_2.start_conversion(&mut self.pins.cv_6);
        let sample_5: u32 = block!(adc_1.read_sample()).unwrap_or_default();
        let sample_6: u32 = block!(adc_2.read_sample()).unwrap_or_default();
        self.cv[4].set(sample_5, adc_1.slope());
        self.cv[5].set(sample_6, adc_2.slope());
    }
}

impl Cv {
    fn set(&mut self, sample: u32, slope: u32) {
        let value = transpose_adc(sample, slope);
        self.probe.write(value > 2.0);
        self.value = if self.probe.detected() {
            None
        } else {
            Some(value)
        };
    }
}

fn transpose_adc(sample: u32, slope: u32) -> f32 {
    // NOTE: The CV input spans between -5 and +5 V.
    let min = -5.0;
    let span = 10.0;
    // NOTE: Based on the measuring, the real span of measured CV is -4.97 to
    // +4.97 V. This compensation makes sure that control value can hit both
    // extremes.
    let compensation = 10.0 / 9.94;
    let phase = (slope as f32 - sample as f32) / slope as f32;
    let scaled = min + phase * span;
    (scaled * compensation).clamp(min, min + span)
}
