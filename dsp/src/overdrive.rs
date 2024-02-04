/// Basic sigmoid approximation overdrive.
pub struct Overdrive {
    gain: f32,
}

impl Overdrive {
    pub fn new(gain: f32) -> Self {
        Self { gain }
    }

    pub fn process(&self, mut value: f32) -> f32 {
        value *= self.gain;
        if value < -1.0 {
            -1.0
        } else if value > 1.0 {
            1.0
        } else {
            (3.0 / 2.0) * (value - libm::powf(value, 3.0) / 3.0)
        }
    }
}
