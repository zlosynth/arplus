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
mod phase_delay_lookup;
mod random;
mod ring_buffer;
mod sigmoid_lookup;
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
    stereo_mode: StereoMode,
    width: f32,
    burst_input: bool,
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
    pub width: f32,
    pub stereo_mode: StereoMode,
}

#[derive(Clone, Copy, Debug, defmt::Format)]
pub struct TriggerAttributes {
    pub frequency: f32,
    pub contour: f32,
    pub pluck: f32,
    pub is_root: bool,
}

#[derive(Clone, Copy, Debug, defmt::Format)]
pub enum StereoMode {
    Haas,
    RootRest,
    PingPong,
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
            stereo_mode: StereoMode::Haas,
            width: 0.0,
            burst_input: false,
        }
    }

    pub fn process(
        &mut self,
        buffer: &mut [(f32, f32); 64],
        input_connected: bool,
        random: &mut impl Random,
    ) {
        let mut buffer_left = [0.0; 64];
        let mut buffer_right = [0.0; 64];
        let mut noise_buffer = [0.0; 64];

        let noise_buffer = if input_connected {
            for (i, x) in noise_buffer.iter_mut().enumerate() {
                *x = buffer[i].0;
            }
            Some(&noise_buffer[..])
        } else {
            None
        };
        self.burst_input = noise_buffer.is_some();

        match self.stereo_mode {
            StereoMode::Haas => {
                for string in self.strings.iter_mut() {
                    string.karplus_strong.populate_add(
                        &mut buffer_left,
                        &mut buffer_right,
                        noise_buffer,
                        karplus_strong::StereoMode::Haas,
                        self.width,
                        random,
                    );
                }
            }
            StereoMode::RootRest => {
                for string in self.strings.iter_mut() {
                    let focus = if string.is_root {
                        karplus_strong::StereoMode::FocusLeft
                    } else {
                        karplus_strong::StereoMode::FocusRight
                    };
                    string.karplus_strong.populate_add(
                        &mut buffer_left,
                        &mut buffer_right,
                        noise_buffer,
                        focus,
                        self.width,
                        random,
                    );
                }
            }
            StereoMode::PingPong => {
                for (i, string) in self.strings.iter_mut().enumerate() {
                    let focus = if i % 2 == 0 {
                        karplus_strong::StereoMode::FocusLeft
                    } else {
                        karplus_strong::StereoMode::FocusRight
                    };
                    string.karplus_strong.populate_add(
                        &mut buffer_left,
                        &mut buffer_right,
                        noise_buffer,
                        focus,
                        self.width,
                        random,
                    );
                }
            }
        }

        self.dc_blocker[0].process(&mut buffer_left);
        self.dc_blocker[1].process(&mut buffer_right);

        // NOTE: Despite the overdrive, there is no need for an additional
        // FIR filter here. Karplus strong is already filtered with the max
        // cutoff frequency of 10.5 kHz, which is 1/4 of the nyquist frequency.
        // When I tried adding a FIR, there was no noticeable difference in
        // the output quality.
        self.overdrive.process(&mut buffer_left);
        self.overdrive.process(&mut buffer_right);

        buffer.iter_mut().enumerate().for_each(|(i, x)| {
            *x = (buffer_left[i], buffer_right[i]);
        })
    }

    pub fn set_attributes(&mut self, attributes: Attributes, random: &mut impl Random) {
        for string in self.strings.iter_mut() {
            string.karplus_strong.set_resonance(attributes.resonance);
            string.karplus_strong.set_cutoff(attributes.cutoff);
        }

        self.stereo_mode = attributes.stereo_mode;

        if matches!(self.stereo_mode, StereoMode::PingPong) {
            // XXX: This effectively disables the split to root and rest.
            self.set_root_strings_len(8);
        } else {
            self.set_root_strings_len(attributes.chord_size);
        }

        if let Some(trigger) = attributes.trigger {
            let (string_index, next_string_index) = {
                match self.stereo_mode {
                    StereoMode::PingPong => {
                        // XXX: Ping pong does not distinguish between root and
                        // rest, so it can consistently shift left and right,
                        // no matter how big is the chord.
                        let string_index = self.root_string_index();
                        self.bump_root_string_index();
                        let next_string_index = self.root_string_index();
                        (string_index, next_string_index)
                    }
                    _ => {
                        if trigger.is_root {
                            let string_index = self.root_string_index();
                            self.bump_root_string_index();
                            let next_string_index = self.root_string_index();
                            (string_index, next_string_index)
                        } else {
                            let string_index = self.rest_string_index();
                            self.bump_rest_string_index();
                            let next_string_index = self.rest_string_index();
                            (string_index, next_string_index)
                        }
                    }
                }
            };

            // NOTE: With anything shorter than 60 ms, any input will sound
            // like a noise.
            let minimal_burst_interval = if self.burst_input { 0.06 } else { 0.0 };

            let string = &mut self.strings[string_index];
            string.karplus_strong.trigger(
                0.99,
                trigger.frequency,
                trigger.contour + minimal_burst_interval,
                trigger.pluck,
                attributes.cutoff,
                attributes.width,
                random,
            );
            string.is_root = trigger.is_root;

            let next_string = &mut self.strings[next_string_index];
            next_string.karplus_strong.reset();
        }

        self.overdrive.gain = attributes.gain;
        self.width = attributes.width;
    }

    fn set_root_strings_len(&mut self, len: usize) {
        // PANIC: This would fail right after boot if there was a mismatch in
        // constants. This is safe, only to make sure that this function is
        // adjusted if the number of strings changes.
        assert_eq!(self.strings.len(), 8);

        let new_root_strings_len = match len {
            1..=2 => 4, // NOTE: Even with size 1, interval can be used for non-root
            3..=6 => 3,
            8 => 8, // XXX: Special value for ping pong, treating all notes as root
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
