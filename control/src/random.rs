use fastrand::Rng;

pub trait Random {
    fn u8_mod(&mut self, modulo: u8) -> u8;
}

pub struct RandomGenerator {
    rng: Rng,
}

impl RandomGenerator {
    pub fn with_seed(seed: u64) -> Self {
        Self {
            rng: Rng::with_seed(seed),
        }
    }
}

impl Random for RandomGenerator {
    fn u8_mod(&mut self, modulo: u8) -> u8 {
        self.rng.u8(0..modulo)
    }
}
