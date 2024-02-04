//! Envelope follower.
//!
//! Based on <https://kferg.dev/posts/2020/audio-reactive-programming-envelope-followers/>.

use libm::{expf, fabsf, logf};

pub struct EnvelopeFollower {
    n_1: f32,
    alpha_attack: f32,
    alpha_decay: f32,
    treshold: f32,
}

impl EnvelopeFollower {
    pub fn new(attack: f32, decay: f32, treshold: f32, sample_rate: f32) -> Self {
        Self {
            n_1: 0.0,
            alpha_attack: expf(logf(0.5) / (sample_rate * attack)),
            alpha_decay: expf(logf(0.5) / (sample_rate * decay)),
            treshold,
        }
    }

    pub fn process(&mut self, x: f32) {
        let abs_x = (fabsf(x) - self.treshold).max(0.0);
        self.n_1 = if abs_x > self.n_1 {
            self.alpha_attack * self.n_1 + (1.0 - self.alpha_attack) * abs_x
        } else {
            self.alpha_decay * self.n_1 + (1.0 - self.alpha_decay) * abs_x
        };
    }

    pub fn level(&self) -> f32 {
        self.n_1
    }
}
