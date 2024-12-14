use core::f32::consts::PI;

#[derive(Debug, defmt::Format)]
pub struct StateVariableFilter {
    sample_rate: u32,
    bandform: Bandform,
    f: f32,
    q: f32,
    delay_1: f32,
    delay_2: f32,
    q_factor: f32,
}

impl StateVariableFilter {
    pub fn new(sample_rate: u32) -> Self {
        let mut filter = Self {
            sample_rate,
            bandform: BandPass,
            f: 0.0,
            q: 0.0,
            delay_1: 0.0,
            delay_2: 0.0,
            q_factor: 0.0,
        };
        filter.set_q_factor(0.7);
        filter.set_frequency(0.0);
        filter
    }

    pub fn set_bandform(&mut self, bandform: Bandform) -> &mut Self {
        self.bandform = bandform;
        self
    }

    pub fn set_frequency(&mut self, frequency: f32) -> &mut Self {
        self.f = 2.0 * libm::sinf((PI * frequency) / self.sample_rate as f32);
        self
    }

    pub fn set_q_factor(&mut self, q_factor: f32) -> &mut Self {
        self.q_factor = f32::max(q_factor, 0.5);
        self.q = 1.0 / self.q_factor;
        self
    }

    pub fn q_factor(&self) -> f32 {
        self.q_factor
    }

    // https://www.earlevel.com/main/2003/03/02/the-digital-state-variable-filter/
    //
    //             +----------------------------------------------------------+
    //             |                                                          |
    //             +-->[high pass]      +-->[band pass]                    [sum 4]-->[band reject]
    //             |                    |                                     |
    // -->[sum 1]--+--[mul f]--[sum 2]--+->[delay 1]--+--[mul f]--[sum 3]--+--+----+-->[low pass]
    //    - A  A -                A                   |              A     |       |
    //      |   \                 |                   |              |  [delay 2]  |
    //      |    \                +-------------------+              |     |       |
    //      |     \                                   |              +-----+       |
    //      |      \---[mut q]------------------------+                            |
    //      |                                                                      |
    //      +----------------------------------------------------------------------+
    //
    pub fn tick(&mut self, value: f32) -> f32 {
        let sum_3 = self.delay_1 * self.f + self.delay_2;
        let sum_1 = value - sum_3 - self.delay_1 * self.q;
        let sum_2 = sum_1 * self.f + self.delay_1;

        let value = match self.bandform {
            LowPass => sum_3,
            HighPass => sum_1,
            BandPass => sum_2,
            BandReject => {
                #[allow(clippy::let_and_return)]
                let sum_4 = sum_1 + sum_3;
                sum_4
            }
        };

        self.delay_1 = sum_2;
        self.delay_2 = sum_3;

        value.clamp(-1.0, 1.0)
    }
}

// NOTE: Allowing unused variants, so this can be easily used as a library.
#[allow(dead_code)]
#[derive(Debug, defmt::Format)]
pub enum Bandform {
    LowPass,
    HighPass,
    BandPass,
    BandReject,
}

pub use Bandform::*;
