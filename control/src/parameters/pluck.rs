use super::primitives::continuous::Continuous;
use super::primitives::math;

pub struct Pluck {
    continuous: Continuous,
}

impl Pluck {
    pub fn new() -> Self {
        Self {
            continuous: Continuous::new(),
        }
    }

    pub fn reconcile(&mut self, pot: f32, cv: Option<f32>) {
        let linear_sum = math::linear_sum(pot, cv);
        let offset_sum = (linear_sum + 0.01).clamp(0.0, 1.0);
        let exp_sum = log(offset_sum);
        self.continuous.reconcile(exp_sum);
    }

    pub fn value(&self) -> f32 {
        self.continuous.value()
    }
}

// TODO: Move this to its own lib, use the same one as in dsp
const LOG: [f32; 22] = [
    0.0,
    0.005,
    0.019_996_643,
    0.040_958_643,
    0.062_983_93,
    0.086_186_11,
    0.110_698_28,
    0.136_677_15,
    0.164_309_44,
    0.19382,
    0.225_483,
    0.259_637_3,
    0.296_708_64,
    0.337_242_2,
    0.381_951_87,
    0.431_798_22,
    0.488_116_62,
    0.552_841_96,
    0.628_932_1,
    0.721_246_36,
    0.838_632,
    1.0,
];

pub fn log(position: f32) -> f32 {
    if position < 0.0 {
        return 0.0;
    } else if position > 1.0 {
        return 1.0;
    }

    let array_position = position * (LOG.len() - 1) as f32;
    let index_a = array_position as usize;
    let index_b = (array_position as usize + 1).min(LOG.len() - 1);
    let remainder = libm::modff(array_position).0;

    let value = LOG[index_a];
    let delta_to_next = LOG[index_b] - LOG[index_a];

    value + delta_to_next * remainder
}
