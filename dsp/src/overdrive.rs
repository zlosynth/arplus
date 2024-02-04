/// Basic sigmoid approximation overdrive.
pub struct Overdrive {
    pub gain: f32,
}

impl Overdrive {
    pub fn new() -> Self {
        Self { gain: 1.0 }
    }

    pub fn process(&self, buffer: &mut [f32]) {
        buffer.iter_mut().for_each(|x| *x = self.process_sample(*x));
    }

    fn process_sample(&self, mut value: f32) -> f32 {
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
