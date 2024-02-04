#![no_std]
#![allow(clippy::new_without_default)]

use karplus_strong::KarplusStrong;
use overdrive::Overdrive;
use oversampling::{Downsampler4, Upsampler4};

#[cfg(test)]
#[macro_use]
extern crate approx;

mod ad_envelope;
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
    upsampler_left: Upsampler4,
    upsampler_right: Upsampler4,
    downsampler_left: Downsampler4,
    downsampler_right: Downsampler4,
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
            upsampler_left: Upsampler4::new_4(memory_manager),
            upsampler_right: Upsampler4::new_4(memory_manager),
            downsampler_left: Downsampler4::new_4(memory_manager),
            downsampler_right: Downsampler4::new_4(memory_manager),
        }
    }

    pub fn process(&mut self, buffer: &mut [(f32, f32); 32], random: &mut impl Random) {
        let mut buffer_left = [0.0; 32];
        let mut buffer_right = [0.0; 32];

        self.strings[0].populate_add(&mut buffer_left, random);

        // TODO: DC blocker

        let mut buffer_left_os = [0.0; 32 * 4];
        self.upsampler_left
            .process(&buffer_left, &mut buffer_left_os);
        self.overdrive.process(&mut buffer_left_os);
        self.downsampler_left
            .process(&buffer_left_os, &mut buffer_left[..]);

        let mut buffer_right_os = [0.0; 32 * 4];
        self.upsampler_right
            .process(&buffer_right, &mut buffer_right_os);
        self.overdrive.process(&mut buffer_right_os);
        self.downsampler_right
            .process(&buffer_right_os, &mut buffer_right[..]);

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
            1.0 / self.strings.len() as f32 + attributes.gain * self.strings.len() as f32;
    }
}
