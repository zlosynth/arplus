use super::sigmoid_lookup::SIGMOID;

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
            return -1.0;
        } else if value > 1.0 {
            return 1.0;
        }

        let position = (value + 1.0) / 2.0;

        interpolate_sigmoid(position)
    }
}

fn interpolate_sigmoid(position: f32) -> f32 {
    let array_position = position * (SIGMOID.len() - 1) as f32;
    let index_a = array_position as usize;
    let index_b = (array_position as usize + 1).min(SIGMOID.len() - 1);
    let remainder = libm::modff(array_position).0;

    let value_a = SIGMOID[index_a];
    let delta_to_b = SIGMOID[index_b] - SIGMOID[index_a];

    value_a + delta_to_b * remainder
}
