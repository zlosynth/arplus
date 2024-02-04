use stm32h7xx_hal::rng::{ErrorKind, Rng};

pub struct RandomGenerator {
    rng: Rng,
}

impl RandomGenerator {
    pub fn from_rng(rng: Rng) -> Self {
        Self { rng }
    }

    pub fn u16(&mut self) -> Result<u16, ErrorKind> {
        use daisy::hal::rng::RngCore;
        RngCore::<u16>::gen(&mut self.rng)
    }
}

impl arplus_dsp::Random for RandomGenerator {
    fn normal(&mut self) -> f32 {
        match self.u16() {
            Ok(x) => x as f32 / (2 << 15) as f32,
            Err(_) => 0.0,
        }
    }
}
