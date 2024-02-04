#![no_std]
#![allow(clippy::new_without_default)]

use karplus_strong::KarplusStrong;

#[cfg(test)]
#[macro_use]
extern crate approx;

mod ad_envelope;
mod envelope_follower;
mod karplus_strong;
mod math;
mod memory_manager;
mod overdrive;
mod random;
mod ring_buffer;
mod state_variable_filter;
mod taper;

pub use crate::memory_manager::MemoryManager;
pub use crate::random::Random;

pub struct Dsp {
    string: KarplusStrong,
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
            string: KarplusStrong::new(sample_rate, memory_manager),
        }
    }

    pub fn process(&mut self, buffer: &mut [(f32, f32); 32], random: &mut impl Random) {
        let mut buffer_left = [0.0; 32];
        let mut buffer_right = [0.0; 32];

        self.string.populate_add(&mut buffer_left, random);

        // TODO: DC blocker
        // TODO: Overdrive

        buffer.iter_mut().enumerate().for_each(|(i, x)| {
            *x = (buffer_left[i], buffer_right[i]);
        })
    }

    pub fn set_attributes(&mut self, attributes: Attributes) {
        self.string.set_resonance(attributes.resonance);
        self.string.set_cutoff(attributes.cutoff);
        if let Some(trigger) = attributes.trigger {
            defmt::info!("{:?}", attributes);
            self.string
                .trigger(0.99, trigger.frequency, trigger.contour);
        }
    }
}
