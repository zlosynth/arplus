#![no_std]
#![allow(clippy::new_without_default)]

pub struct Dsp;

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
    pub fn new(_sample_rate: f32) -> Self {
        Self
    }

    pub fn process(&mut self, _buffer: &mut [(f32, f32); 32]) {}

    pub fn set_attributes(&mut self, _attributes: Attributes) {}
}
