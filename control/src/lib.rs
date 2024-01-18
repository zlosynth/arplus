#![no_std]
#![allow(clippy::new_without_default)]

pub mod save;

use arplus_dsp::Attributes;

use crate::save::Save;

pub struct Controller;

pub struct Result {
    pub save: Option<Save>,
    pub instrument_attributes: Attributes,
}

pub struct ControlInputSnapshot {
    pub pots: [f32; 6],
    pub buttons: [bool; 4],
    pub cvs: [Option<f32>; 6],
    pub trigger: bool,
}

pub struct ControlOutputState {
    pub leds: [bool; 8],
}

impl Controller {
    pub fn new() -> Self {
        Self
    }

    pub fn warm_up(&mut self, _snapshot: ControlInputSnapshot) -> Result {
        todo!()
    }

    pub fn apply_input_snapshot(&mut self, _snapshot: ControlInputSnapshot) -> Result {
        todo!()
    }

    pub fn tick(&mut self) {
        todo!();
    }

    pub fn desired_output_state(&self) -> ControlOutputState {
        todo!();
    }
}

impl From<Save> for Controller {
    fn from(save: Save) -> Self {
        todo!("Make sure no warm-up is needed");
        Self
    }
}
