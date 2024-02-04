use crate::ad_envelope::{Ad, Config};
use crate::envelope_follower::EnvelopeFollower;
use crate::memory_manager::MemoryManager;
use crate::ring_buffer::RingBuffer;
use crate::state_variable_filter::{Bandform, StateVariableFilter};
use crate::taper;
use crate::Random;

// With the sample rate of 48 kHz, this buffer size should allow frequencies
// of down to 11.7 Hz.
const SAMPLES: usize = 4096;

// Magical numbers producing the result I like the most.
const ATTACK: f32 = 0.02;
const DECAY: f32 = 0.14;
const TRESHOLD: f32 = 0.5;
const ENV_MAX: f32 = 1.0;
const ENV_COEF: f32 = 2.0;

const RESET: usize = 20;

pub struct KarplusStrong {
    sample_rate: f32,
    feedback: f32,
    resonance: f32,
    frequency: f32,
    contour: f32,
    noise_envelope: Ad,
    buffer: RingBuffer,
    filter: StateVariableFilter,
    envelope_follower: EnvelopeFollower,
    reset: usize,
}

impl KarplusStrong {
    pub fn new(sample_rate: f32, stack_manager: &mut MemoryManager) -> Self {
        let mut filter = StateVariableFilter::new(sample_rate as u32);
        filter.set_bandform(Bandform::LowPass);
        filter.set_q_factor(0.7);
        Self {
            sample_rate,
            feedback: 0.0,
            resonance: 0.0,
            frequency: 0.0,
            contour: 0.0,
            noise_envelope: Ad::new(sample_rate),
            buffer: RingBuffer::new_from_memory_manager(stack_manager, SAMPLES),
            filter,
            envelope_follower: EnvelopeFollower::new(ATTACK, DECAY, TRESHOLD, sample_rate),
            reset: RESET,
        }
    }

    pub fn populate_add(&mut self, buffer: &mut [f32], random: &mut impl Random) {
        if self.frequency < 12.0 {
            return;
        }

        let reset_phase = self.reset as f32 / RESET as f32;
        let buffer_len = buffer.len() as f32;
        for (i, x) in buffer.iter_mut().enumerate() {
            // Positive-only noise sounds more pleasant and produces ritcher
            // overtones with a short attack. However, with a longer attack it
            // does not get DC blocked fast enough.
            let offset_noise = if self.contour < 0.001 {
                random.normal() * 2.0
            } else {
                random.normal() * 2.0 - 1.0
            };
            let noise_sample = offset_noise * self.noise_envelope.pop();

            let delayed_sample = self
                .buffer
                .peek_interpolated(self.sample_rate / self.frequency);
            let mixed_sample = noise_sample + delayed_sample * self.feedback;

            let q = 0.5 + self.resonance / 2.0;
            let compressed_q = q - self.envelope_follower.level().min(ENV_MAX) * ENV_COEF;
            self.filter.set_q_factor(compressed_q);

            let filtered_sample = self.filter.tick(mixed_sample);

            self.envelope_follower.process(filtered_sample);

            self.buffer.write(filtered_sample);

            let sample_phase = i as f32 / buffer_len;
            let reset_fade = if self.reset == RESET {
                1.0
            } else {
                1.0 - (reset_phase + sample_phase / RESET as f32)
            };

            *x += (mixed_sample - noise_sample) * 0.5 * reset_fade;
        }

        if self.reset == RESET - 1 {
            self.buffer.reset();
        } else if self.reset < RESET {
            self.reset += 1;
        }
    }

    pub fn set_resonance(&mut self, resonance: f32) {
        self.resonance = resonance;
    }

    pub fn reset(&mut self) {
        self.reset = 0;
    }

    pub fn set_cutoff(&mut self, cutoff: f32) {
        // TODO: Play with this so the filter can be pushed up, while it feels
        // that it does anything in all tone positions.

        // Cutoff range is balanced so it produces audible difference with all
        // pitches.
        let cutoff = 1.5 + cutoff * 10.0;
        self.filter
            .set_frequency((cutoff * self.frequency).clamp(200.0, 6_000.0));
    }

    pub fn trigger(&mut self, feedback: f32, frequency: f32, contour: f32) {
        self.reset = RESET;
        self.feedback = feedback;
        self.frequency = frequency;

        self.contour = ((contour - 0.05) / 0.95).clamp(0.0, 1.0);
        let contour_time = if self.contour == 0.0 {
            0.0
        } else {
            taper::log(self.contour) * 5.0
        };
        self.noise_envelope.trigger(
            Config::new()
                .with_attack_time(contour_time)
                .with_decay_time(0.001 + contour_time),
        );
    }
}
