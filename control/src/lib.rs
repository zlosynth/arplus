#![no_std]
#![allow(clippy::new_without_default)]

use arplus_dsp::Attributes;

pub struct Controller;

pub struct Result {
    pub save: Option<Save>,
    pub instrument_attributes: Attributes,
}

pub struct Save;

pub struct OutputState;
pub struct InputSnapshot;

impl Controller {
    pub fn new() -> Self {
        Self
    }

    pub fn apply_input_snapshot(&mut self, _snapshot: InputSnapshot) -> Result {
        todo!()
    }

    pub fn tick(&mut self) {
        todo!();
    }

    pub fn desired_output_state(&self) -> OutputState {
        todo!();
    }
}
