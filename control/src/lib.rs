#![no_std]
#![allow(clippy::new_without_default)]

mod inputs;
pub mod save;

use arplus_dsp::Attributes as DSPAttributes;

use crate::save::Save;
pub use inputs::ControlInputSnapshot;
use inputs::Inputs;

pub struct Controller {
    inputs: Inputs,
    // state: State,
    // queue: Queue,
}

// enum State {
//     Calibrating(StateCalibrating),
//     Normal,
// }

// struct StateCalibrating {
//     input: usize,
//     phase: CalibrationPhase,
// }

// enum CalibrationPhase {
//     Octave1,
//     Octave2(f32),
// }

// struct Queue {
//     queue: Vec<ControlAction; 8>,
// }

// enum ControlAction {
//     Calibrate(usize),
// }

pub struct Result {
    pub save: Option<Save>,
    pub instrument_attributes: DSPAttributes,
}

pub struct ControlOutputState {
    pub leds: [bool; 8],
}

impl Controller {
    pub fn new() -> Self {
        Self {
            inputs: Inputs::new(),
        }
    }

    pub fn apply_input_snapshot(&mut self, snapshot: ControlInputSnapshot) -> Result {
        self.inputs.apply_input_snapshot(snapshot);

        Result {
            save: None,
            instrument_attributes: DSPAttributes,
        }
    }

    pub fn tick(&mut self) {
        todo!();
    }

    pub fn desired_output_state(&self) -> ControlOutputState {
        todo!();
    }
}

impl From<Save> for Controller {
    fn from(_save: Save) -> Self {
        // TODO todo!("Make sure no warm-up is needed");
        Self::new()
    }
}
