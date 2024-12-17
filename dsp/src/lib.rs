#![no_std]
#![allow(clippy::new_without_default)]

use dc_blocker::DCBlocker;
use karplus_strong::KarplusStrong;
use overdrive::Overdrive;

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
mod phase_delay;
mod random;
mod ring_buffer;
mod state_variable_filter;
mod taper;

pub use crate::memory_manager::MemoryManager;
pub use crate::random::Random;

pub struct Dsp {
    strings: [String; 8],
    root_strings_len: usize,
    active_root_string_index: usize,
    active_rest_string_index: usize,
    overdrive: Overdrive,
    dc_blocker: [DCBlocker; 2],
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
    pub chord_size: usize,
}

#[derive(Clone, Copy, Debug, defmt::Format)]
pub struct TriggerAttributes {
    pub frequency: f32,
    pub contour: f32,
    pub pluck: f32,
    pub is_root: bool,
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
            root_strings_len: 4,
            active_root_string_index: 0,
            active_rest_string_index: 0,
            overdrive: Overdrive::new(),
            dc_blocker: [DCBlocker::new(), DCBlocker::new()],
        }
    }

    pub fn process(&mut self, buffer: &mut [(f32, f32); 32], random: &mut impl Random) {
        let mut buffer_root = [0.0; 32];
        let mut buffer_rest = [0.0; 32];

        for string in self.strings.iter_mut() {
            if string.is_root {
                string.karplus_strong.populate_add(&mut buffer_root, random);
            } else {
                string.karplus_strong.populate_add(&mut buffer_rest, random);
            }
        }

        self.dc_blocker[0].process(&mut buffer_root);
        self.dc_blocker[1].process(&mut buffer_rest);

        // TODO: I probably should bring back oversampling.
        // but maybe LPF with 12 kHz on 2x oversampled signal is enough?
        // TODO: It is filtered and oversampled. so just apply FIR to 0.5, and then apply overdrive
        self.overdrive.process(&mut buffer_root);
        self.overdrive.process(&mut buffer_rest);

        buffer.iter_mut().enumerate().for_each(|(i, x)| {
            *x = (buffer_rest[i], buffer_root[i]);
        })
    }

    pub fn set_attributes(&mut self, attributes: Attributes) {
        for string in self.strings.iter_mut() {
            string.karplus_strong.set_resonance(attributes.resonance);
            string.karplus_strong.set_cutoff(attributes.cutoff);
        }

        self.set_root_strings_len(attributes.chord_size);

        if let Some(trigger) = attributes.trigger {
            let (string_index, next_string_index) = if trigger.is_root {
                let string_index = self.root_string_index();
                self.bump_root_string_index();
                let next_string_index = self.root_string_index();
                (string_index, next_string_index)
            } else {
                let string_index = self.rest_string_index();
                self.bump_rest_string_index();
                let next_string_index = self.rest_string_index();
                (string_index, next_string_index)
            };

            let string = &mut self.strings[string_index];
            string
                .karplus_strong
                .trigger(0.99, trigger.frequency, trigger.contour, trigger.pluck);
            // TODO: Set cutoff again? To make sure it gets in sync with the
            // new frequency before DSP starts doing anything.
            string.is_root = trigger.is_root;

            let next_string = &mut self.strings[next_string_index];
            next_string.karplus_strong.reset();
        }

        self.overdrive.gain = attributes.gain;
    }

    fn set_root_strings_len(&mut self, len: usize) {
        assert_eq!(self.strings.len(), 8);

        let new_root_strings_len = match len {
            1..=2 => 4, // NOTE: Even with size 1, interval can be used for non-root
            3..=6 => 3,
            _ => 2,
        };

        if new_root_strings_len == self.root_strings_len {
            return;
        }

        self.root_strings_len = new_root_strings_len;

        if self.active_root_string_index >= self.root_strings_len {
            self.active_root_string_index = 0;
        }

        if self.active_rest_string_index < self.root_strings_len {
            self.active_rest_string_index = self.root_strings_len;
        }
    }

    fn root_string_index(&mut self) -> usize {
        self.active_root_string_index
    }

    fn bump_root_string_index(&mut self) {
        self.active_root_string_index += 1;
        self.active_root_string_index %= self.root_strings_len;
    }

    fn rest_string_index(&mut self) -> usize {
        self.active_rest_string_index
    }

    fn bump_rest_string_index(&mut self) {
        self.active_rest_string_index += 1;
        if self.active_rest_string_index >= self.strings.len() {
            self.active_rest_string_index = self.root_strings_len;
        }
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
