#![no_std]
#![allow(clippy::new_without_default)]

use dc_blocker::DCBlocker;
use karplus_strong::KarplusStrong;
use overdrive::Overdrive;
use oversampling::{Downsampler4, Upsampler4};

#[cfg(test)]
#[macro_use]
extern crate approx;

mod ad_envelope;
mod dc_blocker;
mod envelope_follower;
mod karplus_strong;
mod math;
mod memory_manager;
mod overdrive;
mod oversampling;
mod random;
mod ring_buffer;
mod state_variable_filter;
mod taper;

pub use crate::memory_manager::MemoryManager;
pub use crate::random::Random;

pub struct Dsp {
    strings: [KarplusStrong; 1],
    overdrive: Overdrive,
    dc_blocker: [DCBlocker; 2],
    upsampler: [Upsampler4; 2],
    downsampler: [Downsampler4; 2],
}

#[derive(Clone, Copy, Debug, defmt::Format)]
pub struct Attributes {
    pub gain: f32,
    pub resonance: f32,
    pub cutoff: f32,
    pub trigger: Option<TriggerAttributes>,
}

#[derive(Clone, Copy, Debug, defmt::Format)]
pub struct TriggerAttributes {
    pub frequency: f32,
    pub contour: f32,
}

impl Dsp {
    pub fn new(sample_rate: f32, memory_manager: &mut MemoryManager) -> Self {
        Self {
            strings: [KarplusStrong::new(sample_rate, memory_manager)],
            overdrive: Overdrive::new(0.5),
            dc_blocker: [DCBlocker::new(), DCBlocker::new()],
            upsampler: [
                Upsampler4::new_4(memory_manager),
                Upsampler4::new_4(memory_manager),
            ],
            downsampler: [
                Downsampler4::new_4(memory_manager),
                Downsampler4::new_4(memory_manager),
            ],
        }
    }

    pub fn process(&mut self, buffer: &mut [(f32, f32); 32], random: &mut impl Random) {
        let mut buffer_left = [0.0; 32];
        let mut buffer_right = [0.0; 32];

        self.strings[0].populate_add(&mut buffer_left, random);

        self.dc_blocker[0].process(&mut buffer_left);
        self.dc_blocker[1].process(&mut buffer_right);

        let mut buffer_left_os = [0.0; 32 * 4];
        self.upsampler[0].process(&buffer_left, &mut buffer_left_os);
        self.overdrive.process(&mut buffer_left_os);
        self.downsampler[0].process(&buffer_left_os, &mut buffer_left[..]);

        let mut buffer_right_os = [0.0; 32 * 4];
        self.upsampler[1].process(&buffer_right, &mut buffer_right_os);
        self.overdrive.process(&mut buffer_right_os);
        self.downsampler[1].process(&buffer_right_os, &mut buffer_right[..]);

        buffer.iter_mut().enumerate().for_each(|(i, x)| {
            *x = (buffer_left[i], buffer_right[i]);
        })
    }

    pub fn set_attributes(&mut self, attributes: Attributes) {
        self.strings[0].set_resonance(attributes.resonance);
        self.strings[0].set_cutoff(attributes.cutoff);
        if let Some(trigger) = attributes.trigger {
            self.strings[0].trigger(0.99, trigger.frequency, trigger.contour);
        }
        self.overdrive.gain =
            0.5 / self.strings.len() as f32 + attributes.gain * self.strings.len() as f32 * 10.0;
    }
}
