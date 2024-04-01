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
    strings: [String; 8],
    active_string_index: usize,
    overdrive: Overdrive,
    dc_blocker: [DCBlocker; 2],
    upsampler: [Upsampler4; 2],
    downsampler: [Downsampler4; 2],
    stereo_mode: StereoMode,
}

pub struct String {
    karplus_strong: KarplusStrong,
    is_root: bool,
}

#[derive(Clone, Copy, Debug, defmt::Format)]
pub struct Attributes {
    pub resonance: f32,
    pub cutoff: f32,
    pub trigger: Option<TriggerAttributes>,
    pub gain: f32,
    pub stereo_mode: StereoMode,
}

#[derive(Clone, Copy, Debug, defmt::Format)]
pub struct TriggerAttributes {
    pub frequency: f32,
    pub contour: f32,
    pub is_root: bool,
}

#[repr(usize)]
#[derive(Clone, Copy, Debug, defmt::Format, PartialEq)]
pub enum StereoMode {
    RoundRobin,
    RootLeft,
}

impl Dsp {
    pub fn new(sample_rate: f32, memory_manager: &mut MemoryManager) -> Self {
        Self {
            strings: [
                String::new(sample_rate, memory_manager),
                String::new(sample_rate, memory_manager),
                String::new(sample_rate, memory_manager),
                String::new(sample_rate, memory_manager),
                String::new(sample_rate, memory_manager),
                String::new(sample_rate, memory_manager),
                String::new(sample_rate, memory_manager),
                String::new(sample_rate, memory_manager),
            ],
            active_string_index: 0,
            overdrive: Overdrive::new(),
            dc_blocker: [DCBlocker::new(), DCBlocker::new()],
            upsampler: [
                Upsampler4::new_4(memory_manager),
                Upsampler4::new_4(memory_manager),
            ],
            downsampler: [
                Downsampler4::new_4(memory_manager),
                Downsampler4::new_4(memory_manager),
            ],
            stereo_mode: StereoMode::default(),
        }
    }

    pub fn process(&mut self, buffer: &mut [(f32, f32); 32], random: &mut impl Random) {
        let mut buffer_left = [0.0; 32];
        let mut buffer_right = [0.0; 32];

        match self.stereo_mode {
            StereoMode::RoundRobin => {
                for string_pair in self.strings.chunks_mut(2) {
                    if let Some(left_string) = string_pair.get_mut(0) {
                        left_string
                            .karplus_strong
                            .populate_add(&mut buffer_left, random);
                    }
                    if let Some(right_string) = string_pair.get_mut(1) {
                        right_string
                            .karplus_strong
                            .populate_add(&mut buffer_right, random);
                    }
                }
            }
            StereoMode::RootLeft => {
                for string in self.strings.iter_mut() {
                    if string.is_root {
                        string.karplus_strong.populate_add(&mut buffer_left, random);
                    } else {
                        string
                            .karplus_strong
                            .populate_add(&mut buffer_right, random);
                    }
                }
            }
        }

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
        for string in self.strings.iter_mut() {
            string.karplus_strong.set_resonance(attributes.resonance);
            string.karplus_strong.set_cutoff(attributes.cutoff);
        }

        if let Some(trigger) = attributes.trigger {
            self.strings[self.active_string_index]
                .karplus_strong
                .trigger(0.99, trigger.frequency, trigger.contour);

            self.active_string_index += 1;
            self.active_string_index %= self.strings.len();

            let next_string = &mut self.strings[self.active_string_index];
            next_string.karplus_strong.reset();
        }

        self.overdrive.gain = attributes.gain;
    }
}

impl String {
    pub fn new(sample_rate: f32, memory_manager: &mut MemoryManager) -> Self {
        Self {
            karplus_strong: KarplusStrong::new(sample_rate, memory_manager),
            is_root: false,
        }
    }
}

impl Default for StereoMode {
    fn default() -> Self {
        Self::RoundRobin
    }
}

impl TryFrom<usize> for StereoMode {
    type Error = ();

    fn try_from(index: usize) -> Result<Self, Self::Error> {
        if index >= 2 {
            return Err(());
        }
        Ok(unsafe { core::mem::transmute(index) })
    }
}
