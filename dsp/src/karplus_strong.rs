use crate::ad_envelope::{Ad, Config};
use crate::envelope_follower::EnvelopeFollower;
use crate::memory_manager::MemoryManager;
use crate::phase_delay;
use crate::ring_buffer::RingBuffer;
use crate::state_variable_filter::{Bandform, StateVariableFilter};
use crate::taper;
use crate::Random;

// With the sample rate of 96 kHz, this buffer size should allow frequencies
// of down to 11.7 Hz.
const SAMPLES: usize = 8192;

// Magic numbers producing the result I like the most.
const ATTACK: f32 = 0.02;
const DECAY: f32 = 0.14;
const TRESHOLD: f32 = 0.5;
const ENV_MAX: f32 = 1.0;
const ENV_COEF: f32 = 2.0;

const RESET: usize = 40;

pub struct KarplusStrong {
    sample_rate: f32,
    feedback: f32,
    resonance: f32,
    frequency: f32,
    contour: f32,
    pluck: f32,
    phase_delay: f32,
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
        let mut s = Self {
            sample_rate,
            feedback: 0.0,
            resonance: 0.0,
            frequency: 0.0,
            contour: 0.0,
            pluck: 0.0,
            noise_envelope: Ad::new(sample_rate),
            buffer: RingBuffer::new_from_memory_manager(stack_manager, SAMPLES),
            filter,
            envelope_follower: EnvelopeFollower::new(ATTACK, DECAY, TRESHOLD, sample_rate),
            reset: RESET,
            phase_delay: 0.0,
        };
        // XXX: Without this silent trigger, the first trigger on the string is mute.
        s.trigger(0.0, 10.0, 0.0, 0.0, 0.0);
        s
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
            let noise_sample = offset_noise * self.noise_envelope.pop() * self.pluck;

            let interval_in_samples = self.sample_rate / self.frequency;
            let phase_delay_in_samples = (self.sample_rate / self.frequency) * self.phase_delay;
            let relative_index = interval_in_samples - phase_delay_in_samples;
            let delayed_sample = self.buffer.peek_interpolated(relative_index);

            let mixed_sample = noise_sample + delayed_sample * self.feedback;

            const MIN_Q: f32 = 0.6;
            const MAX_Q: f32 = 0.9;
            let q = MIN_Q + self.resonance * (MAX_Q - MIN_Q);
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

            *x += filtered_sample * 0.5 * reset_fade;
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
        // NOTE: During initialization, the frequency is set to 0.0. If that
        // happens, the function should return early, to prevent division
        // by zero.
        if !self.frequency.is_normal() {
            return;
        }

        // NOTE: Cutoff ratio is moving depending on the fundamental frequency.
        // That means that a cutoff that is harmonic for one tone will not be
        // for another.
        // However, the alternative of having fixed multiple would suffer
        // in high fundamental frequencies, where most of the cutoff would
        // not be usable due to filter stability.
        // Also, when I tried this with a stable cutoff ratio, the sound missed
        // some of its character and grittiness (although the harmonics were
        // pleasant in some settings).
        // The implemented behavior is to scale the cutoff knob based on the
        // available frequency "headroom". The minimum cutoff is constant and
        // the maximum is scaled down to always fit into the stable frequency
        // range of the filter.
        const MIN_CUTOFF: f32 = 1.5;
        // TODO: This should be matching the clamp?
        const MAX_CUTOFF_FREQUENCY: f32 = 12_000.0;
        let cutoff = MIN_CUTOFF + (MAX_CUTOFF_FREQUENCY / self.frequency) * taper::log(cutoff);

        self.filter
            .set_frequency((cutoff * self.frequency).clamp(20.0, 10_500.0));
    }

    pub fn trigger(
        &mut self,
        feedback: f32,
        frequency: f32,
        contour: f32,
        pluck: f32,
        cutoff: f32,
    ) {
        self.reset = RESET;
        self.feedback = feedback;
        self.pluck = pluck;

        self.frequency = frequency;
        // XXX: The filter cutoff frequency is calculated based on the note
        // frequency and cutoff knob. This needs to be recalculated on trigger
        // to reflect the new note frequency, so the filter value later read
        // to calculate the phase delay is correct.
        self.set_cutoff(cutoff);

        self.phase_delay = {
            let c = self.filter.frequency() / self.frequency;
            let q = self.filter.q_factor();
            phase_delay::phase_delay(c, q)
        };

        // NOTE: This never reaches infinity. While infinite sustain sounds
        // awesome, there is no control to silence all strings on the module,
        // so it would be confusing to leave this feature in the default
        // firmware.
        // TODO: Feature gate this and publish an alternative firmware with
        // infinite decay.
        self.contour = ((contour - 0.05) / 0.96).clamp(0.0, 1.0);
        let (attack, decay) = if self.contour == 0.0 {
            (0.0, 0.001)
        } else if self.contour == 1.0 {
            let attack = taper::log(self.contour) * 5.0;
            let decay = f32::INFINITY;
            (attack, decay)
        } else {
            let attack = taper::log(self.contour) * 5.0;
            let decay = attack + 0.001;
            (attack, decay)
        };
        self.noise_envelope.trigger(
            Config::new()
                .with_attack_time(attack)
                .with_decay_time(decay),
        );
    }
}
